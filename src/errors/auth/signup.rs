use axum::{
    response::{IntoResponse, Response},
    Json,
};
use derive_more::derive::Display;
use hyper::StatusCode;

// #[allow(dead_code)]
#[derive(Debug, Display)]
pub enum SignupError {
    ValidationError,
    EmailUnavailableError,
    DatabaseError(surrealdb::Error),
    EmailError(String),
    HashingError,
    OtherError(String),
}

impl IntoResponse for SignupError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            // TODO: Make these errors more descriptive (also make them use their actual message)
            SignupError::ValidationError => StatusCode::BAD_REQUEST,
            SignupError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SignupError::EmailUnavailableError => StatusCode::BAD_REQUEST,
            SignupError::EmailError(_) => StatusCode::BAD_REQUEST,
            SignupError::HashingError => StatusCode::INTERNAL_SERVER_ERROR,
            SignupError::OtherError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(serde_json::json!({"error": self.to_string()}));
        (status_code, body).into_response()
    }
}
