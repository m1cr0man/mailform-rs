#[cfg(feature = "cli")]
use clap_serde_derive::ClapSerde;
use std::net::{Ipv4Addr, SocketAddr};

#[cfg(not(feature = "cli"))]
const fn default_three() -> usize {
    3
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[cfg_attr(
    feature = "cli",
    derive(clap_serde_derive::clap::Parser, ClapSerde),
    command(author, version, about)
)]
pub struct Config {
    #[cfg_attr(
        feature = "cli",
        default(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3001))),
        arg(
            short,
            long,
            // Sadly, clap_serde_derive defaults aren't used in clap's help output.
            // Emulate it via help text.
            help = "Address to listen on. [default: 0.0.0.0:3001]",
            env = "MAILFORM_ADDRESS"
        )
    )]
    pub address: SocketAddr,

    #[cfg_attr(
        feature = "cli",
        arg(long, help = "SMTP server host name", env = "MAILFORM_SMTP_HOST")
    )]
    pub smtp_host: String,

    #[cfg_attr(
        feature = "cli",
        arg(
            long,
            help = "SMTP server login username",
            env = "MAILFORM_SMTP_USERNAME"
        )
    )]
    pub smtp_username: String,

    #[cfg_attr(
        feature = "cli",
        arg(
            long,
            help = "SMTP server login password",
            env = "MAILFORM_SMTP_PASSWORD"
        )
    )]
    pub smtp_password: String,

    #[cfg_attr(
        feature = "cli",
        arg(
            long,
            help = "The address to send emails from",
            env = "MAILFORM_FROM_ADDRESS"
        )
    )]
    pub from_address: String,

    #[cfg_attr(
        feature = "cli",
        arg(
            long,
            help = "The address to send emails to",
            env = "MAILFORM_TO_ADDRESS"
        )
    )]
    pub to_address: String,

    #[cfg_attr(not(feature = "cli"), serde(default = "default_three"))]
    #[cfg_attr(
        feature = "cli",
        default(3),
        arg(
            long,
            help = "How many times to try sending an email before dropping it",
            env = "MAILFORM_SEND_RETRIES"
        )
    )]
    pub send_retries: usize,
}
