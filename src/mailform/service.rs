use super::{
    error::Error, BuildMessageSnafu, Message, QueueMessageSnafu, SendMessageSnafu, WrappedMessage,
};

use lettre::{transport::smtp::authentication::Credentials, SmtpTransport, Transport};
use std::sync::mpsc;
use uuid::Uuid;

#[derive(Debug)]
pub struct Mailform {
    recv: mpsc::Receiver<WrappedMessage>,
    send: mpsc::SyncSender<WrappedMessage>,
    config: super::Config,
    credentials: Credentials,
}
use snafu::prelude::*;

impl Mailform {
    pub fn new(config: super::Config) -> Self {
        let (send, recv) = mpsc::sync_channel(8);
        let username = config.smtp_username.clone();
        let password = config.smtp_password.clone();
        Self {
            send,
            recv,
            config,
            credentials: Credentials::new(username, password),
        }
    }

    pub fn get_sender(&self) -> MailformSender {
        MailformSender {
            send: self.send.clone(),
            retries: self.config.send_retries,
        }
    }

    pub fn send_mail(&self, message: Message) -> Result<(), Error> {
        let email = lettre::Message::builder()
            .from(self.config.from_address.clone())
            .reply_to(message.from_address)
            .to(self.config.to_address.clone())
            .subject(message.subject)
            .body(message.body)
            .context(BuildMessageSnafu {})?;

        // Open a remote connection to the SMTP relay server
        let mailer = SmtpTransport::starttls_relay(&self.config.smtp_host)
            .context(SendMessageSnafu {})?
            .credentials(self.credentials.clone())
            .build();

        // Send the email
        mailer
            .send(&email)
            .context(SendMessageSnafu {})
            .map(|_| (()))
    }

    pub fn process_mail(self) {
        for mut msg in self.recv.iter() {
            match self.send_mail(msg.message.clone()) {
                Ok(_) => {
                    tracing::info!(tracing_id = msg.tracing_id.to_string(), "Message sent")
                }
                Err(err) => {
                    tracing::error!(
                        tracing_id = msg.tracing_id.to_string(),
                        ttl = msg.ttl,
                        "{err:#?}"
                    );
                    if msg.ttl > 1 {
                        msg.ttl -= 1;
                        self.send.send(msg).unwrap();
                    }
                }
            }
        }
    }
}

impl From<super::Config> for Mailform {
    fn from(config: super::Config) -> Self {
        Self::new(config)
    }
}

#[derive(Debug)]
pub struct MailformSender {
    send: mpsc::SyncSender<WrappedMessage>,
    retries: usize,
}

impl MailformSender {
    pub fn queue_mail(&self, message: Message) -> Result<Uuid, Error> {
        let tracing_id = Uuid::new_v4();
        self.send
            .send(WrappedMessage {
                ttl: self.retries,
                message,
                tracing_id,
            })
            .context(QueueMessageSnafu {})?;
        Ok(tracing_id)
    }
}
