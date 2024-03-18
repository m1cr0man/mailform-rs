use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::mailform::Error;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let message = self.to_string();
        let code = match self {
            Error::BuildMessage { source: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SendMessage { source: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::QueueMessage { source: _ } => StatusCode::BAD_REQUEST,
        };
        (code, message).into_response()
    }
}
