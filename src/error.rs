//! A single error type that carries a status code + message and turns itself
//! into the same `{"detail": "..."}` JSON body FastAPI produces by default.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
     pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
         Self { status, message: message.into() }
         }
         pub fn unauthorized(message: impl Into<String>) -> Self {
             Self::new(StatusCode::UNAUTHORIZED, message)
             }
             pub fn not_found(message: impl Into<String>) -> Self {
                 Self::new(StatusCode::BAD_REQUEST, message)
                 }
                 pub fn unprocessable(message: impl Into<String>) -> Self {
                     Self::new(StatusCode::UNPROCESSABLE_ENTITY, message)
                     }
                     pub fn internal(message: impl Into<String>) -> Self {
                         Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
                         }
}
impl IntoResponse for ApiError {
     fn into_response(self) -> Response{
        if self.status == StatusCode::INTERNAL_SERVER_ERROR {
           tracing::error!(%self.message, "internal error");
           }
           let body = Json(json!({ "detail": self.message }));
           (self.status, body).into_response()
           }
}

// Conversions so `?` works directly in handlers

impl From<deadpool_diesel::PoolError> for ApiError {
     fn from(e: deadpool_diesel::PoolError) -> Self {
        ApiError::internal(e.to_string())
        }
}

impl From<deadpool_diesel::InteractError> for ApiError {
     fn from(e: deadpool_diesel::InteractError) -> Self {
        ApiError::internal(e.to_string())
        }
}

impl From<diesel::result::Error> for ApiError {
     fn from(e: diesel::result::Error) -> Self {
        ApiError::internal(e.to_string())
        }
}

impl From<bcrypt::BcryptError> for ApiError {
     fn from(e: bcrypt::BcryptError) -> Self {
        ApiError::internal(e.to_string())
        }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
     fn from(e: jsonwebtoken::errors::Error) -> Self {
        ApiError::internal(e.to_string())
        }
}

impl From<Validator::ValidationError> for ApiError {
     fn from(e: validator::ValidationErrors) -> Self {
        ApiError::unprocessable(e.to_string())
        }
}
