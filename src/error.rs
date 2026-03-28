use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("authorization error: {0}")]
    Auth(String),
    #[error("invalid input: {0}")]
    Validation(String),
    #[error("provider unavailable: {0}")]
    Provider(String),
    #[error("request failed: {0}")]
    Http(String),
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Config(_) | Self::Internal(_) | Self::Serialization(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::Auth(_) => StatusCode::UNAUTHORIZED,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Provider(_) | Self::Http(_) => StatusCode::BAD_GATEWAY,
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value.to_string())
    }
}

impl From<url::ParseError> for AppError {
    fn from(value: url::ParseError) -> Self {
        Self::Config(value.to_string())
    }
}

#[derive(Debug, Serialize)]
struct ErrorBody<'a> {
    code: u16,
    error: &'a str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.to_string();
        let body = Json(ErrorBody {
            code: status.as_u16(),
            error: &message,
        });

        (status, body).into_response()
    }
}
