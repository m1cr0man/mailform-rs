use std::net::SocketAddr;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub address: SocketAddr,

    pub smtp_host: String,

    pub smtp_username: String,

    pub smtp_password: String,

    pub from_address: String,

    pub to_address: String,

    pub send_retries: usize,
}
