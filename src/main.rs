pub mod http;
pub mod mailform;
pub mod extensions;

#[cfg(feature = "cli")]
mod cli;

#[tokio::main]
async fn main() {
    #[cfg(not(feature = "cli"))]
    panic!("cli feature is not enabled");
    #[cfg(feature = "cli")]
    cli::main().await
}
