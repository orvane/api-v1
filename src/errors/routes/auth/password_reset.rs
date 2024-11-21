use derive_more::Display;
use hyper::StatusCode;
use serde_json::{json, Value};

use crate::errors::{response::ApiError, CommonError, ErrorResponse};

#[derive(Debug, Display)]
pub enum PasswordResetError {
    Common(CommonError),
    TokenExpired,
    InvalidToken,
    InvalidEmail,
}

impl ErrorResponse for PasswordResetError {
    fn error_name(&self) -> &str {
        match self {
            PasswordResetError::Common(e) => e.error_name(),
            PasswordResetError::TokenExpired => "Token Expired",
            PasswordResetError::InvalidToken => "Invalid Token",
            PasswordResetError::InvalidEmail => "Invalid Email",
        }
    }

    fn error_message(&self) -> Value {
        match self {
            PasswordResetError::Common(e) => e.error_message(),
            PasswordResetError::TokenExpired => {
                json!("The password reset token has expired")
            }
            PasswordResetError::InvalidToken => json!("The password reset token is invalid"),
            PasswordResetError::InvalidEmail => json!("The provided email is invalid"),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            PasswordResetError::Common(e) => e.status_code(),
            PasswordResetError::TokenExpired => StatusCode::BAD_REQUEST,
            PasswordResetError::InvalidToken => StatusCode::BAD_REQUEST,
            PasswordResetError::InvalidEmail => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<CommonError> for PasswordResetError {
    fn from(error: CommonError) -> Self {
        PasswordResetError::Common(error)
    }
}

impl From<PasswordResetError> for ApiError<PasswordResetError> {
    fn from(error: PasswordResetError) -> Self {
        ApiError(error)
    }
}

// Automatic Error Conversion

impl From<validator::ValidationErrors> for ApiError<PasswordResetError> {
    fn from(error: validator::ValidationErrors) -> Self {
        ApiError(PasswordResetError::Common(CommonError::Validation(error)))
    }
}

impl From<surrealdb::Error> for ApiError<PasswordResetError> {
    fn from(error: surrealdb::Error) -> Self {
        ApiError(PasswordResetError::Common(CommonError::Database(error)))
    }
}

impl From<resend_rs::Error> for ApiError<PasswordResetError> {
    fn from(error: resend_rs::Error) -> Self {
        ApiError(PasswordResetError::Common(CommonError::Email(error)))
    }
}

impl From<argon2::password_hash::Error> for ApiError<PasswordResetError> {
    fn from(error: argon2::password_hash::Error) -> Self {
        ApiError(PasswordResetError::Common(CommonError::Hashing(error)))
    }
}
