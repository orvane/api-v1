use axum::{
    response::{IntoResponse, Response},
    Json,
};
use derive_more::derive::Display;
use hyper::StatusCode;
use serde_json::{json, Value};

#[derive(Debug, Display)]
pub enum SignupError {
    ValidationError(validator::ValidationErrors),
    DatabaseError(surrealdb::Error),
    HashingError(argon2::password_hash::Error),
    EmailServiceError(resend_rs::Error),
}

impl SignupError {
    fn name(&self) -> &str {
        match self {
            SignupError::ValidationError(_) => "Validation Error",
            SignupError::DatabaseError(_) => "Database Error",
            SignupError::HashingError(_) => "Hashing Error",
            SignupError::EmailServiceError(_) => "Email Service Error",
        }
    }

    fn message(&self) -> Value {
        match self {
            // TODO: Make sure the message is brought from the validation function (for example:
            // "Code should contain 6 numbers, etc.")
            SignupError::ValidationError(errors) => {
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
            SignupError::DatabaseError(_) => {
                json!("An error occurred while accessing database")
            }
            SignupError::HashingError(_) => {
                json!("An error occured while hashing the password")
            }
            SignupError::EmailServiceError(_) => {
                json!("An error occured while sending email verification email")
            }
        }
    }
}

impl IntoResponse for SignupError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            SignupError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SignupError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SignupError::HashingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SignupError::EmailServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(serde_json::json!({
            "error": self.name(),
            "message": self.message()
        }));

        (status_code, body).into_response()
    }
}
