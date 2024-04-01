use super::Mailbox;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub smtp_host: String,

    pub smtp_username: String,

    pub smtp_password: String,

    pub from_address: Mailbox,

    pub to_address: Mailbox,

    pub send_retries: usize,
}
