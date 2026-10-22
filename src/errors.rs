use axum::{
    Json,
    extract::multipart::MultipartError,
    http::StatusCode,
    response::{IntoResponse, Response},
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
    #[error("conversion timed out")]
    ConversionTimeout,
    #[error("the file is password protected")]
    PasswordProtected,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("multipart error: {0}")]
    Multipart(String),
}

impl From<MultipartError> for AppError {
    fn from(error: MultipartError) -> Self {
        // The body limit surfaces as a multipart read error with status 413.
        if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
            Self::PayloadTooLarge
        } else {
            Self::Multipart(error.body_text())
        }
    }
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
                format!("Unsupported file format or conversion: '{mime}'"),
            ),
            Self::ConversionFailed(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "CONVERSION_FAILED".into(),
                message,
            ),
            Self::ConversionTimeout => (
                StatusCode::GATEWAY_TIMEOUT,
                "CONVERSION_TIMEOUT".into(),
                "This file is taking too long to convert".into(),
            ),
            Self::PasswordProtected => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "PASSWORD_PROTECTED".into(),
                "This file is password-protected".into(),
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

        (
            status,
            Json(ErrorBody {
                error: message,
                code,
            }),
        )
            .into_response()
    }
}
