use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    errors::auth::email_verification::{self, EmailVerificationError},
    services::database_service::DatabaseLayer,
    utils::validation::{
        validate_email_verification_code_format, validate_email_verification_code_length,
    },
};

#[derive(Debug, Deserialize, Validate)]
pub struct RoutePayload {
    #[validate(custom(function = "validate_email_verification_code_length"))]
    #[validate(custom(function = "validate_email_verification_code_format"))]
    code: String,
    user_id: String,
    email_verification_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteOutput {
    message: String,
}

// TODO: Make the route work only if the session with user id was provided
#[axum::debug_handler]
pub async fn email_verification(
    Extension(database_layer): Extension<DatabaseLayer>,
    Json(payload): Json<RoutePayload>,
) -> Result<(StatusCode, Json<RouteOutput>), EmailVerificationError> {
    // 1. Validate payload input
    let payload_instace = RoutePayload {
        code: payload.code.clone(),
        user_id: payload.user_id.clone(),
        email_verification_id: payload.email_verification_id.clone(),
    };

    match payload_instace.validate() {
        Ok(_) => println!("Validation passed successfully!"),
        Err(e) => return Err(EmailVerificationError::ValidationError(e)),
    }

    // TODO: 2. Validate unauthorized session
    // Compare it via the token in the url

    // 3. Update user verified status
    let modify_user_verified_status = database_layer
        .query()
        .user
        .verify_user(String::from(payload.user_id))
        .await;

    match modify_user_verified_status {
        Ok(_) => println!("User verified status updated successfully!"),
        Err(e) => return Err(EmailVerificationError::DatabaseError(e)),
    }

    // 4. Remove email verification

    let remove_email_verification = database_layer
        .query()
        .email_verification
        .remove(String::from(payload.email_verification_id))
        .await;

    match remove_email_verification {
        Ok(_) => println!("Email verification removed successfully!"),
        Err(e) => return Err(EmailVerificationError::DatabaseError(e)),
    }

    // TODO: 5. remove unauthorized session

    Ok((
        StatusCode::OK,
        Json(RouteOutput {
            message: String::from("Email verified successfully"),
        }),
    ))
}
