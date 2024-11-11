use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    errors::{auth::EmailVerificationError, response::ApiError},
    services::database::DatabaseLayer,
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
pub async fn email_verification(
    Extension(database_layer): Extension<DatabaseLayer>,
    Json(payload): Json<RoutePayload>,
) -> Result<(StatusCode, Json<RouteOutput>), ApiError<EmailVerificationError>> {
    // 1. Validate payload input
    let payload_instance = RoutePayload {
        code: payload.code.clone(),
        user_id: payload.user_id.clone(),
        email_verification_id: payload.email_verification_id.clone(),
    };

    payload_instance.validate()?;
    println!("Validation passed successfully!");

    // TODO: 2. Validate unauthorized session
    // Compare it via the token in the url

    // 3. Update user verified status
    database_layer
        .query()
        .user
        .verify_user(payload.user_id.clone())
        .await?;
    println!("User verified status updated successfully!");

    // 4. Remove email verification
    database_layer
        .query()
        .email_verification
        .remove(payload.email_verification_id.clone())
        .await?;
    println!("Email verification removed successfully!");

    // TODO: 5. remove unauthorized session
    // TODO: 6. Send email to user confirming the account verification

    Ok((
        StatusCode::OK,
        Json(RouteOutput {
            message: String::from("Email verified successfully"),
        }),
    ))
}
