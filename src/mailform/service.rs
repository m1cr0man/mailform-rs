use super::{
    error::Error, BuildMessageSnafu, Message, QueueMessageSnafu, SendMessageSnafu, WrappedMessage,
};

use lettre::{transport::smtp::authentication::Credentials, SmtpTransport, Transport};
use std::{
    sync::mpsc::{self, RecvTimeoutError},
    time::Duration,
};

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
            .from(self.config.from_address.parse().unwrap())
            .reply_to(message.from_address.parse().unwrap())
            .to(self.config.to_address.parse().unwrap())
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
        loop {
            match self.recv.recv_timeout(Duration::from_secs(30)) {
                Ok(msg) => match self.send_mail(msg.message.clone()) {
                    Ok(_) => tracing::debug!("Message sent"),
                    Err(err) => {
                        tracing::error!("{err:#?}");
                        if msg.ttl > 1 {
                            self.send
                                .send(WrappedMessage {
                                    ttl: msg.ttl - 1,
                                    message: msg.message,
                                })
                                .unwrap();
                        }
                    }
                },
                Err(RecvTimeoutError::Disconnected) => return,
                Err(_) => {}
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
    pub fn queue_mail(&self, message: Message) -> Result<(), Error> {
        self.send
            .send(WrappedMessage {
                ttl: self.retries,
                message,
            })
            .context(QueueMessageSnafu {})
    }
}
