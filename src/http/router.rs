use std::sync::Arc;

use crate::service::{Error, MailformSender, Message};
use axum::{
    extract::Form,
    extract::{Json, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Extension, Router,
};

const TRACING_ID_HEADER: &str = "X-Mailform-Tracing-Id";

#[derive(Debug, serde::Deserialize)]
pub struct FormQueryParams {
    redirect_path: String,
}

pub async fn mail_post_json(
    Extension(mailform): Extension<Arc<MailformSender>>,
    Json(payload): Json<Message>,
) -> Result<impl IntoResponse, Error> {
    let body_bytes = payload.body.len();
    let tracing_id = mailform.queue_mail(payload)?.to_string();

    tracing::info!(
        endpoint = "/v1/send/json",
        body_bytes = body_bytes,
        tracing_id = tracing_id,
        "Message received",
    );

    Ok((StatusCode::NO_CONTENT, [(TRACING_ID_HEADER, tracing_id)]))
}

pub async fn mail_post_form(
    Extension(mailform): Extension<Arc<MailformSender>>,
    Query(query_params): Query<FormQueryParams>,
    Form(payload): Form<Message>,
) -> Result<impl IntoResponse, Error> {
    let body_bytes = payload.body.len();
    let tracing_id = mailform.queue_mail(payload)?.to_string();

    tracing::info!(
        endpoint = "/v1/send/form",
        body_bytes = body_bytes,
        tracing_id = tracing_id,
        redirect_path = query_params.redirect_path,
        "Message received",
    );

    Ok((
        StatusCode::NO_CONTENT,
        [
            (TRACING_ID_HEADER, tracing_id),
            ("Location", query_params.redirect_path),
        ],
    ))
}

pub fn get_router(mailform: Arc<MailformSender>) -> Router {
    Router::new()
        .route("/v1/send/json", post(mail_post_json))
        .route("/v1/send/form", post(mail_post_form))
        .layer(Extension(mailform))
}
