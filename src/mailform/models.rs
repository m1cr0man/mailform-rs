pub use lettre::message::Mailbox;
pub use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub from_address: Mailbox,
    pub body: String,
    pub subject: String,
}

#[derive(Debug, Clone)]
pub struct WrappedMessage {
    pub ttl: usize,
    pub message: Message,
    pub tracing_id: Uuid,
}
