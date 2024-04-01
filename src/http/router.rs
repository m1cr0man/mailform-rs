use std::sync::Arc;

use crate::service::{Error, MailformSender, Message, RequestSnafu};
use axum::{
    extract::Form,
    extract::{Json, Query},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::post,
    Extension, Router,
};

const TRACING_ID_HEADER: &str = "X-Mailform-Tracing-Id";

#[derive(Debug, serde::Deserialize)]
pub struct FormQueryParams {
    redirect_path: Option<String>,
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

    let response_code = match query_params.redirect_path {
        Some(_) => StatusCode::SEE_OTHER,
        None => StatusCode::NO_CONTENT,
    };

    let mut headers = HeaderMap::new();
    headers.insert(TRACING_ID_HEADER, tracing_id.parse().unwrap());
    if let Some(redirect_path) = query_params.redirect_path {
        match HeaderValue::try_from(format!("{}?mailform_success=true", redirect_path)) {
            Ok(val) => headers.insert("Location", val),
            Err(_) => {
                return Err(RequestSnafu {
                    msg: "Invalid redirect_path",
                }
                .build())
            }
        };
    }

    Ok((response_code, headers))
}

pub fn get_router(mailform: Arc<MailformSender>) -> Router {
    Router::new()
        .route("/v1/send/json", post(mail_post_json))
        .route("/v1/send/form", post(mail_post_form))
        .layer(Extension(mailform))
}
