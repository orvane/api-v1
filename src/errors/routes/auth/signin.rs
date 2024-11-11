use derive_more::Display;
use hyper::StatusCode;
use serde_json::{json, Value};

use crate::errors::{response::ApiError, CommonError, ErrorResponse};

#[derive(Debug, Display)]
pub enum SigninError {
    Common(CommonError),
    InvalidCredentials,
    AccountLocked,
    AccountNotVerified,
}

impl ErrorResponse for SigninError {
    fn error_name(&self) -> &str {
        match self {
            SigninError::Common(e) => e.error_name(),
            SigninError::InvalidCredentials => "Invalid Credentials",
            SigninError::AccountLocked => "Account Locked",
            SigninError::AccountNotVerified => "Account Not Verified",
        }
    }

    fn error_message(&self) -> Value {
        match self {
            SigninError::Common(e) => e.error_message(),
            SigninError::InvalidCredentials => json!("The provided credentials are invalid"),
            SigninError::AccountLocked => json!("The account is locked"),
            SigninError::AccountNotVerified => json!("The account is not verified"),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            SigninError::Common(e) => e.status_code(),
            SigninError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            SigninError::AccountLocked => StatusCode::FORBIDDEN,
            SigninError::AccountNotVerified => StatusCode::FORBIDDEN,
        }
    }
}

impl From<CommonError> for SigninError {
    fn from(error: CommonError) -> Self {
        SigninError::Common(error)
    }
}

impl From<SigninError> for ApiError<SigninError> {
    fn from(error: SigninError) -> Self {
        ApiError(error)
    }
}

// Automatic Error Conversion

impl From<validator::ValidationErrors> for ApiError<SigninError> {
    fn from(error: validator::ValidationErrors) -> Self {
        ApiError(SigninError::Common(CommonError::Validation(error)))
    }
}

impl From<surrealdb::Error> for ApiError<SigninError> {
    fn from(error: surrealdb::Error) -> Self {
        ApiError(SigninError::Common(CommonError::Database(error)))
    }
}

impl From<resend_rs::Error> for ApiError<SigninError> {
    fn from(error: resend_rs::Error) -> Self {
        ApiError(SigninError::Common(CommonError::Email(error)))
    }
}

impl From<argon2::password_hash::Error> for ApiError<SigninError> {
    fn from(error: argon2::password_hash::Error) -> Self {
        ApiError(SigninError::Common(CommonError::Hashing(error)))
    }
}
