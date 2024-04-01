use axum::Router;
use clap::{crate_authors, crate_description, crate_version, Arg, ArgAction, Command};
use std::env;
use std::io::Write;
use std::{error::Error, process::exit, sync::Arc, thread};

use crate::service::{Config, Mailform};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct AppConfig {
    listener_address: tokio_listener::ListenerAddress,
}

fn parse_config<'a, T: serde::Deserialize<'a>>(prefix: &str) -> Result<T, Box<dyn Error>> {
    let cfg_source = config::Config::builder()
        .add_source(
            config::Environment::with_prefix(prefix)
                .convert_case(config::Case::ScreamingSnake)
                .try_parsing(true),
        )
        .build()?;

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

    let app_config: AppConfig = parse_config("MAILFORM").unwrap();
    let config: Config = parse_config("MAILFORM").unwrap();
    let user_opts: tokio_listener::UserOptions = parse_config("MAILFORM_LISTENER").unwrap();

    if args.get_flag("check") {
        tracing::info!("Configuration is valid.");
        exit(0);
    }

    let service = Mailform::from(config.clone());

    let app = app(&service);

    let handle = thread::spawn(move || service.process_mail());

    // Start the web server
    let listener = tokio_listener::Listener::bind(
        &app_config.listener_address,
        &tokio_listener::SystemOptions::default(),
        &user_opts,
    )
    .await
    .map_err(|err| {
        tracing::error!("Failed to configure listener: {}", err);
        exit(3);
    })
    .unwrap();

    tracing::info!("Listening on {}", app_config.listener_address);
    tokio_listener::axum07::serve(listener, app.into_make_service())
        .await
        .unwrap();

    handle.join().unwrap();
}
