use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbiddden: insufficient role")]
    Forbidden,
    #[error("Validation: {0}")]
    Validation(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("External API error: {0}")]
    ExternalApi(String),
    #[error("Internal: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::NotFound(m)    => (StatusCode::NOT_FOUND, "NOT_FOUND", m.clone()),
            AppError::Unauthorized   => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "Invalid credentials".into()),
            AppError::Forbidden      => (StatusCode::FORBIDDEN, "FORBIDDEN", "Insufficient permissions".into()),
            AppError::Validation(m)  => (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_ERROR", m.clone()),
            AppError::Conflict(m)    => (StatusCode::CONFLICT, "CONFLICT", m.clone()),
            AppError::Database(e)    => {
                tracing::error!("DB error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "DB_ERROR", "Database error".into())
            }
            AppError::Encryption(m) => (StatusCode::INTERNAL_SERVER_ERROR, "ENCRYPTION_ERROR", m.clone()),
            AppError::ExternalApi(m) => (StatusCode::BAD_GATEWAY, "EXTERNAL_API_ERROR", m.clone()),
            AppError::Internal(e) => {
                tracing::error!("Internal: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error".into())
            }
        };

        (status, Json(json!({ "error": code, "message": message}))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
