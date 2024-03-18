#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub from_address: String,
    pub body: String,
    pub subject: String,
}

#[derive(Debug, Clone)]
pub struct WrappedMessage {
    pub ttl: usize,
    pub message: Message,
}
