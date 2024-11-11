use super::common::CommonError;
use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use serde_json::{json, Value};

pub trait ErrorResponse {
    fn error_name(&self) -> &str;
    fn error_message(&self) -> Value;
    fn status_code(&self) -> StatusCode;
}

pub struct ApiError<T>(pub T);

impl<T: ErrorResponse> IntoResponse for ApiError<T> {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": self.0.error_name(),
            "message": self.0.error_message()
        }));
        (self.0.status_code(), body).into_response()
    }
}

impl ErrorResponse for CommonError {
    fn error_name(&self) -> &str {
        match self {
            CommonError::Validation(_) => "Validation Error",
            CommonError::Database(_) => "Database Error",
            CommonError::Email(_) => "Email Service Error",
            CommonError::Hashing(_) => "Hashing Error",
        }
    }

    fn error_message(&self) -> Value {
        match self {
            CommonError::Validation(errors) => {
                let mut field_errors = serde_json::Map::new();
                for (field, error_vec) in errors.field_errors() {
                    let messages: Vec<String> = error_vec
                        .iter()
                        .filter_map(|error| error.message.as_ref().map(|msg| msg.to_string()))
                        .collect();
                    if !messages.is_empty() {
                        field_errors.insert(field.to_string(), json!(messages));
                    }
                }
                json!(field_errors)
            }
            CommonError::Database(e) => json!(e.to_string()),
            CommonError::Email(_) => json!("An error occurred while sending email"),
            CommonError::Hashing(_) => json!("An error occurred while processing credentials"),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            CommonError::Validation(_) => StatusCode::BAD_REQUEST,
            CommonError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CommonError::Email(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CommonError::Hashing(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
