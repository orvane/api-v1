use derive_more::Display;
use hyper::StatusCode;
use serde_json::{json, Value};

use crate::errors::{response::ApiError, CommonError, ErrorResponse};

#[derive(Debug, Display)]
pub enum PasswordResetRequestError {
    Common(CommonError),
    TokenExpired,
    InvalidToken,
    InvalidEmail,
}

impl ErrorResponse for PasswordResetRequestError {
    fn error_name(&self) -> &str {
        match self {
            PasswordResetRequestError::Common(e) => e.error_name(),
            PasswordResetRequestError::TokenExpired => "Token Expired",
            PasswordResetRequestError::InvalidToken => "Invalid Token",
            PasswordResetRequestError::InvalidEmail => "Invalid Email",
        }
    }

    fn error_message(&self) -> Value {
        match self {
            PasswordResetRequestError::Common(e) => e.error_message(),
            PasswordResetRequestError::TokenExpired => {
                json!("The password reset token has expired")
            }
            PasswordResetRequestError::InvalidToken => json!("The password reset token is invalid"),
            PasswordResetRequestError::InvalidEmail => json!("The provided email is invalid"),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            PasswordResetRequestError::Common(e) => e.status_code(),
            PasswordResetRequestError::TokenExpired => StatusCode::BAD_REQUEST,
            PasswordResetRequestError::InvalidToken => StatusCode::BAD_REQUEST,
            PasswordResetRequestError::InvalidEmail => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<CommonError> for PasswordResetRequestError {
    fn from(error: CommonError) -> Self {
        PasswordResetRequestError::Common(error)
    }
}

impl From<PasswordResetRequestError> for ApiError<PasswordResetRequestError> {
    fn from(error: PasswordResetRequestError) -> Self {
        ApiError(error)
    }
}

// Automatic Error Conversion

impl From<validator::ValidationErrors> for ApiError<PasswordResetRequestError> {
    fn from(error: validator::ValidationErrors) -> Self {
        ApiError(PasswordResetRequestError::Common(CommonError::Validation(
            error,
        )))
    }
}

impl From<surrealdb::Error> for ApiError<PasswordResetRequestError> {
    fn from(error: surrealdb::Error) -> Self {
        ApiError(PasswordResetRequestError::Common(CommonError::Database(
            error,
        )))
    }
}

impl From<resend_rs::Error> for ApiError<PasswordResetRequestError> {
    fn from(error: resend_rs::Error) -> Self {
        ApiError(PasswordResetRequestError::Common(CommonError::Email(error)))
    }
}

impl From<argon2::password_hash::Error> for ApiError<PasswordResetRequestError> {
    fn from(error: argon2::password_hash::Error) -> Self {
        ApiError(PasswordResetRequestError::Common(CommonError::Hashing(
            error,
        )))
    }
}
