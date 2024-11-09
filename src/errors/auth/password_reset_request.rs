use axum::{
    response::{IntoResponse, Response},
    Json,
};
use derive_more::derive::Display;
use hyper::StatusCode;
use serde_json::{json, Value};

// TODO: A LOT OF THIS CODE IS A COPY OF email_verification.rs
// FIND A WAY TO CREATE BOILERPLATE LIKE THIS EASIER WITHOUT COPYING EXECESSIVELY
#[derive(Debug, Display)]
pub enum PasswordResetRequestError {
    ValidationError(validator::ValidationErrors),
    DatabaseError(surrealdb::Error),
    EmailServiceError(resend_rs::Error),
}

impl PasswordResetRequestError {
    fn name(&self) -> &str {
        match self {
            PasswordResetRequestError::ValidationError(_) => "Validation Error",
            PasswordResetRequestError::DatabaseError(_) => "Database Error",
            PasswordResetRequestError::EmailServiceError(_) => "Email Service Error",
        }
    }

    fn message(&self) -> Value {
        match self {
            PasswordResetRequestError::ValidationError(errors) => {
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
            PasswordResetRequestError::DatabaseError(_) => {
                json!("An error occured while accessing database")
            }
            PasswordResetRequestError::EmailServiceError(_) => {
                json!("An error has occured while sending an email")
            }
        }
    }
}

impl IntoResponse for PasswordResetRequestError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            PasswordResetRequestError::ValidationError(_) => StatusCode::BAD_REQUEST,
            PasswordResetRequestError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PasswordResetRequestError::EmailServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({
            "error": self.name(),
            "message": self.message()
        }));

        (status_code, body).into_response()
    }
}
