use std::{fs::File, io::BufReader, sync::Arc, thread};

use axum::Router;
use clap_serde_derive::ClapSerde;

use crate::mailform::{Config, Mailform};

use clap::Parser;

fn app(service: &Mailform) -> Router {
    crate::http::get_router(Arc::new(service.get_sender()))
}

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    // TODO make optional
    /// Config file
    #[arg(
        short,
        long = "config",
        default_value = "config.json",
        env = "MAILFORM_CONFIG"
    )]
    config_path: std::path::PathBuf,

    /// Rest of arguments
    #[command(flatten)]
    pub config: <Config as clap_serde_derive::ClapSerde>::Opt,
}

pub(crate) async fn main() {
    let mut args = Args::parse();

    let config = if let Ok(cfg_file) = File::open(&args.config_path) {
        match serde_json::from_reader::<_, <Config as ClapSerde>::Opt>(BufReader::new(cfg_file)) {
            Ok(config) => Config::from(config).merge(&mut args.config),
            Err(err) => panic!("Error in configuration file:\n{err:#?}"),
        }
    } else {
        Config::from(&mut args.config)
    };

    let service = Mailform::from(config.clone());

    let app = app(&service);

    let handle = thread::spawn(move || service.process_mail());

    // Start the web server
    let addr = config.address;
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    handle.join().unwrap();
}
