use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::service::Error;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let message = self.to_string();
        let code = match self {
            Error::QueueMessage { source: _ } => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (code, message).into_response()
    }
}
