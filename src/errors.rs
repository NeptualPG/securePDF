use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid file: {detail}")]
    InvalidFile { code: String, detail: String },
    #[error("payload too large")]
    PayloadTooLarge,
    #[error("unsupported source format: {mime}")]
    UnsupportedFormat { mime: String },
    #[error("conversion failed: {0}")]
    ConversionFailed(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("multipart error: {0}")]
    Multipart(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    code: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::InvalidFile { code, detail } => (StatusCode::BAD_REQUEST, code, detail),
            Self::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "PAYLOAD_TOO_LARGE".into(),
                "Payload too large".into(),
            ),
            Self::UnsupportedFormat { mime } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "UNSUPPORTED_FORMAT".into(),
                format!("Source format '{mime}' cannot be converted to PDF"),
            ),
            Self::ConversionFailed(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "CONVERSION_FAILED".into(),
                message,
            ),
            Self::Io(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL".into(),
                error.to_string(),
            ),
            Self::Multipart(message) => (
                StatusCode::BAD_REQUEST,
                "MALFORMED_MULTIPART".into(),
                message,
            ),
        };

        (status, Json(ErrorBody { error: message, code })).into_response()
    }
}
