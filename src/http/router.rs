use std::sync::Arc;

use crate::mailform::{Error, MailformSender, Message};
use axum::{
    extract::Form,
    extract::{Json, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Extension, Router,
};

#[derive(Debug, serde::Deserialize)]
pub struct FormQueryParams {
    redirect_path: String,
}

pub async fn mail_post_json(
    Extension(mailform): Extension<Arc<MailformSender>>,
    Json(payload): Json<Message>,
) -> Result<impl IntoResponse, Error> {
    tracing::trace!(
        endpoint = "/v1/send/json",
        body_bytes = payload.body.len(),
        from_address = payload.from_address,
        "Message received",
    );
    mailform.queue_mail(payload).map(|_| StatusCode::NO_CONTENT)
}

pub async fn mail_post_form(
    Extension(mailform): Extension<Arc<MailformSender>>,
    Query(query_params): Query<FormQueryParams>,
    Form(payload): Form<Message>,
) -> Result<impl IntoResponse, Error> {
    tracing::trace!(
        endpoint = "/v1/send/form",
        body_bytes = payload.body.len(),
        from_address = payload.from_address,
        redirect_path = query_params.redirect_path,
        "Message received",
    );
    mailform.queue_mail(payload).map(|_| {
        (
            StatusCode::SEE_OTHER,
            [("Location", query_params.redirect_path)],
        )
    })
}

pub fn get_router(mailform: Arc<MailformSender>) -> Router {
    Router::new()
        .route("/v1/send/json", post(mail_post_json))
        .route("/v1/send/form", post(mail_post_form))
        .layer(Extension(mailform))
}
