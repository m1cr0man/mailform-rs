use axum::Router;
use clap::{crate_authors, crate_description, crate_version, Arg, ArgAction, Command};
use hyperlocal::UnixServerExt;
use std::env;
use std::io::Write;
use std::net::SocketAddr;
use std::{error::Error, path::PathBuf, process::exit, sync::Arc, thread};

use crate::mailform::{Config, Mailform};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct AppConfig {
    listen_address: Option<SocketAddr>,
    listen_socket: Option<PathBuf>,
}

fn parse_config<'a, T: serde::Deserialize<'a>>(
    cfg_source: config::Config,
) -> Result<T, Box<dyn Error>> {
    cfg_source.try_deserialize().map_err(|err| {
        tracing::error!("Error in the provided configuration: {}", err);
        exit(2);
    })
}

fn app(service: &Mailform) -> Router {
    crate::http::get_router(Arc::new(service.get_sender()))
}

fn setup_logger() {
    // Set a default level
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    // Adapted from env_logger examples. <3 Systemd support
    match std::env::var("RUST_LOG_STYLE") {
        Ok(s) if s == "SYSTEMD" => env_logger::builder()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "<{}>{}: {}",
                    match record.level() {
                        log::Level::Error => 3,
                        log::Level::Warn => 4,
                        log::Level::Info => 6,
                        log::Level::Debug => 7,
                        log::Level::Trace => 7,
                    },
                    record.target(),
                    record.args()
                )
            })
            .init(),
        _ => pretty_env_logger::init(),
    };
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

    setup_logger();

    let cfg_source = config::Config::builder()
        .add_source(
            config::Environment::with_prefix("MAILFORM")
                .convert_case(config::Case::ScreamingSnake)
                .try_parsing(true),
        )
        .build()
        .unwrap();

    let app_config: AppConfig = parse_config(cfg_source.clone()).unwrap();
    let config: Config = parse_config(cfg_source).unwrap();

    // unix | listen | valid
    // X    |        | X
    // X    | X      |
    //      |        |
    //      | X      | X
    if app_config.listen_address.is_none() == app_config.listen_socket.is_none() {
        tracing::error!("Only one of listen address or unix socket must/should be configured.");
        exit(2);
    }

    if args.get_flag("check") {
        tracing::info!("Configuration is valid.");
        exit(0);
    }

    let service = Mailform::from(config.clone());

    let app = app(&service);

    let handle = thread::spawn(move || service.process_mail());

    // Start the web server
    match app_config.listen_address {
        Some(addr) => {
            tracing::info!("Listening on {}", addr);
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        None => {
            let path = app_config.listen_socket.expect("Logically checked earlier");
            let server = axum::Server::bind_unix(&path)
                .map_err(|err| {
                    tracing::error!("Failed to bind to the provided path: {}", err);
                    exit(3);
                })
                .unwrap();

            tracing::info!("Listening at {}", path.display());
            server.serve(app.into_make_service()).await.unwrap();
        }
    }

    handle.join().unwrap();
}
