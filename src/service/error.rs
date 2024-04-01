use snafu::prelude::*;

use super::WrappedMessage;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Failed to create email message: {:#}", source))]
    BuildMessage { source: lettre::error::Error },
    #[snafu(display("Failed to send message: {:#}", source))]
    SendMessage {
        source: lettre::transport::smtp::Error,
    },
    #[snafu(display("Failed to queue message: {:#}", source))]
    QueueMessage {
        source: std::sync::mpsc::SendError<WrappedMessage>,
    },
    #[snafu(display("Failed to process request: {:#}", msg))]
    Request { msg: String },
}
