use std::{process::exit, sync::Arc, thread};

use axum::Router;
use clap::{crate_authors, crate_description, crate_version, Arg, ArgAction, Command};

use crate::mailform::{Config, Mailform};

fn app(service: &Mailform) -> Router {
    crate::http::get_router(Arc::new(service.get_sender()))
}

pub(crate) async fn main() {
    let cli = Command::new("Mailform")
        .about(format!(
            "{}\n{} {}",
            crate_description!(),
            "Configuration is managed using environment variables.",
            "See the docs for more information.",
        ))
        .arg(
            Arg::new("check")
                .action(ArgAction::SetTrue)
                .short('c')
                .long("check")
                .help("Check the configuration"),
        )
        .version(crate_version!())
        .author(crate_authors!("\n"));

    let args = cli.get_matches();

    let cfg_source = config::Config::builder()
        .add_source(
            config::Environment::with_prefix("MAILFORM")
                .convert_case(config::Case::ScreamingSnake)
                .try_parsing(true),
        )
        .build()
        .unwrap();

    let config: Config = cfg_source
        .try_deserialize()
        .map_err(|err| {
            println!("Error in the provided configuration: {}", err);
            exit(2);
        })
        .unwrap();

    if args.get_flag("check") {
        println!("Configuration is valid.");
        exit(0);
    }

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
