use std::sync::Arc;

use crate::mailform::{Error, MailformSender, Message};
use axum::{
    extract::Json, http::StatusCode, response::IntoResponse, routing::post, Extension, Router,
};

#[derive(serde::Deserialize)]
pub struct PostPayload {
    message: Message,
}

pub async fn mail_post(
    Extension(mailform): Extension<Arc<MailformSender>>,
    Json(payload): Json<PostPayload>,
) -> Result<impl IntoResponse, Error> {
    mailform
        .queue_mail(payload.message)
        .map(|_| StatusCode::NO_CONTENT)
}

pub fn get_router(mailform: Arc<MailformSender>) -> Router {
    Router::new()
        .route("/v1/send", post(mail_post))
        .layer(Extension(mailform))
}
