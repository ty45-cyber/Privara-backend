use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed")]
    Unauthorized,

    #[error("Resource not found")]
    NotFound,

  #[error("Validation error: {0}")]
  Validation(String),
  
  #[error("Encryption failure: {0}")]
  Encryption(String),

  #[error("Database error: {0}")]
  Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
fn into_response(self) -> Response {
    let (status, message) = match &self {
        AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
        AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
        AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        AppError::Encryption(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        AppError::Database(e) => {
            tracing::error!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into())
        }
        AppError::Internal(e) => {
            tracing::error!("Internal error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into())
        }
    };

    (status, Json(json!({ "error": message}))).into_response()
}    
}

pub type AppResult<T, AppError>;