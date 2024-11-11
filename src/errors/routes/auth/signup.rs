use derive_more::Display;
use hyper::StatusCode;
use serde_json::{json, Value};

use crate::errors::{response::ApiError, CommonError, ErrorResponse};

#[derive(Debug, Display)]
pub enum SignupError {
    Common(CommonError),
    EmailAlreadyExists,
    WeakPassword,
    InvalidReferralCode,
    RegistrationClosed,
}

impl ErrorResponse for SignupError {
    fn error_name(&self) -> &str {
        match self {
            SignupError::Common(e) => e.error_name(),
            SignupError::EmailAlreadyExists => "Email Already Exists",
            SignupError::WeakPassword => "Weak Password",
            SignupError::InvalidReferralCode => "Invalid Referral Code",
            SignupError::RegistrationClosed => "Registration Closed",
        }
    }

    fn error_message(&self) -> Value {
        match self {
            SignupError::Common(e) => e.error_message(),
            SignupError::EmailAlreadyExists => {
                json!("An account with this email already exists")
            }
            SignupError::WeakPassword => {
                json!("Password does not meet the minimum strength requirements")
            }
            SignupError::InvalidReferralCode => {
                json!("The provided referral code is invalid")
            }
            SignupError::RegistrationClosed => {
                json!("Registration is currently closed for new users")
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            SignupError::Common(e) => e.status_code(),
            SignupError::EmailAlreadyExists => StatusCode::CONFLICT,
            SignupError::WeakPassword => StatusCode::BAD_REQUEST,
            SignupError::InvalidReferralCode => StatusCode::BAD_REQUEST,
            SignupError::RegistrationClosed => StatusCode::FORBIDDEN,
        }
    }
}

impl From<CommonError> for SignupError {
    fn from(error: CommonError) -> Self {
        SignupError::Common(error)
    }
}

impl From<SignupError> for ApiError<SignupError> {
    fn from(error: SignupError) -> Self {
        ApiError(error)
    }
}

// Automatic Error Conversion

impl From<validator::ValidationErrors> for ApiError<SignupError> {
    fn from(error: validator::ValidationErrors) -> Self {
        ApiError(SignupError::Common(CommonError::Validation(error)))
    }
}

impl From<surrealdb::Error> for ApiError<SignupError> {
    fn from(error: surrealdb::Error) -> Self {
        ApiError(SignupError::Common(CommonError::Database(error)))
    }
}

impl From<resend_rs::Error> for ApiError<SignupError> {
    fn from(error: resend_rs::Error) -> Self {
        ApiError(SignupError::Common(CommonError::Email(error)))
    }
}

impl From<argon2::password_hash::Error> for ApiError<SignupError> {
    fn from(error: argon2::password_hash::Error) -> Self {
        ApiError(SignupError::Common(CommonError::Hashing(error)))
    }
}
