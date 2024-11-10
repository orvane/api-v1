use axum::{
    response::{IntoResponse, Response},
    Json,
};
use derive_more::derive::Display;
use hyper::StatusCode;
use serde_json::{json, Value};

#[derive(Debug, Display)]
pub enum EmailVerificationError {
    ValidationError(validator::ValidationErrors),
    DatabaseError(surrealdb::Error),
    InvalidCodeError,
}

impl EmailVerificationError {
    fn name(&self) -> &str {
        match self {
            EmailVerificationError::ValidationError(_) => "Validation Error",
            EmailVerificationError::DatabaseError(_) => "Database Error",
            EmailVerificationError::InvalidCodeError => "Invalid Code Error",
        }
    }

    fn message(&self) -> Value {
        match self {
            // TODO: Make sure the message is brought from the validation function (for example:
            // "Code should contain 6 numbers, etc.")
            EmailVerificationError::ValidationError(errors) => {
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

            // TODO: Make use of the internal surrealdb::Error message
            // TODO: Normalize this error so it's not nested like: {Api: ""}
            EmailVerificationError::DatabaseError(e) => {
                json!(e)
            }
            EmailVerificationError::InvalidCodeError => {
                json!("The provided code is invalid or has expired")
            }
        }
    }
}

impl IntoResponse for EmailVerificationError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            EmailVerificationError::ValidationError(_) => StatusCode::BAD_REQUEST,
            EmailVerificationError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            EmailVerificationError::InvalidCodeError => StatusCode::FORBIDDEN,
        };

        let body = Json(serde_json::json!({
            "error": self.name(),
            "message": self.message()
        }));

        (status_code, body).into_response()
    }
}
