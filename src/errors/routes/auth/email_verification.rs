use derive_more::Display;
use hyper::StatusCode;
use serde_json::{json, Value};

use crate::errors::{response::ApiError, CommonError, ErrorResponse};

#[derive(Debug, Display)]
pub enum EmailVerificationError {
    Common(CommonError),
    TokenExpired,
    InvalidToken,
    EmailAlreadyVerified,
    InvalidCode,
    ExpiredCode,
    CodeAlreadyUsed,
}

impl ErrorResponse for EmailVerificationError {
    fn error_name(&self) -> &str {
        match self {
            EmailVerificationError::Common(e) => e.error_name(),
            EmailVerificationError::TokenExpired => "Token Expired",
            EmailVerificationError::InvalidToken => "Invalid Token",
            EmailVerificationError::EmailAlreadyVerified => "Email Already Verified",
            EmailVerificationError::InvalidCode => "Invalid Code",
            EmailVerificationError::ExpiredCode => "Expired Code",
            EmailVerificationError::CodeAlreadyUsed => "Code Already Used",
        }
    }

    fn error_message(&self) -> Value {
        match self {
            EmailVerificationError::Common(e) => e.error_message(),
            EmailVerificationError::TokenExpired => json!("The verification token has expired"),
            EmailVerificationError::InvalidToken => json!("The verification token is invalid"),
            EmailVerificationError::EmailAlreadyVerified => json!("The email is already verified"),
            EmailVerificationError::InvalidCode => json!("The verification code is invalid"),
            EmailVerificationError::ExpiredCode => json!("The verification code has expired"),
            EmailVerificationError::CodeAlreadyUsed => {
                json!("The verification code has already been used")
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            EmailVerificationError::Common(e) => e.status_code(),
            EmailVerificationError::TokenExpired => StatusCode::BAD_REQUEST,
            EmailVerificationError::InvalidToken => StatusCode::BAD_REQUEST,
            EmailVerificationError::EmailAlreadyVerified => StatusCode::CONFLICT,
            EmailVerificationError::InvalidCode => StatusCode::BAD_REQUEST,
            EmailVerificationError::ExpiredCode => StatusCode::BAD_REQUEST,
            EmailVerificationError::CodeAlreadyUsed => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<CommonError> for EmailVerificationError {
    fn from(error: CommonError) -> Self {
        EmailVerificationError::Common(error)
    }
}

impl From<EmailVerificationError> for ApiError<EmailVerificationError> {
    fn from(error: EmailVerificationError) -> Self {
        ApiError(error)
    }
}

// Automatic Error Conversion

impl From<validator::ValidationErrors> for ApiError<EmailVerificationError> {
    fn from(error: validator::ValidationErrors) -> Self {
        ApiError(EmailVerificationError::Common(CommonError::Validation(
            error,
        )))
    }
}

impl From<surrealdb::Error> for ApiError<EmailVerificationError> {
    fn from(error: surrealdb::Error) -> Self {
        ApiError(EmailVerificationError::Common(CommonError::Database(error)))
    }
}

impl From<resend_rs::Error> for ApiError<EmailVerificationError> {
    fn from(error: resend_rs::Error) -> Self {
        ApiError(EmailVerificationError::Common(CommonError::Email(error)))
    }
}

impl From<argon2::password_hash::Error> for ApiError<EmailVerificationError> {
    fn from(error: argon2::password_hash::Error) -> Self {
        ApiError(EmailVerificationError::Common(CommonError::Hashing(error)))
    }
}
