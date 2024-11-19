use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use validator::Validate;

use crate::{
    errors::{auth::EmailVerificationError, response::ApiError},
    services::{
        database::DatabaseLayer,
        email::{self, EmailLayer},
    },
    utils::{
        crypto::{hash_string, verify_string_hash},
        validation::{
            validate_email_verification_code_format, validate_email_verification_code_length,
        },
    },
};

#[derive(Debug, Deserialize, Validate)]
pub struct RoutePayload {
    #[validate(custom(function = "validate_email_verification_code_length"))]
    #[validate(custom(function = "validate_email_verification_code_format"))]
    code: String,
    user_id: String,
    email_verification_id_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteOutput {
    message: String,
}

// TODO: Make the route work only if the session with user id was provided
pub async fn email_verification(
    Extension(database_layer): Extension<DatabaseLayer>,
    Extension(email_layer): Extension<EmailLayer>,
    Json(payload): Json<RoutePayload>,
) -> Result<(StatusCode, Json<RouteOutput>), ApiError<EmailVerificationError>> {
    // 1. Validate payload input
    let payload_instance = RoutePayload {
        code: payload.code.clone(),
        user_id: payload.user_id.clone(),
        email_verification_id_hash: payload.email_verification_id_hash.clone(),
    };

    payload_instance.validate()?;
    println!("1. Validation passed successfully!");

    // 2. Check if email verification exists for a user

    let user_id = Thing::from((String::from("user"), payload.user_id.clone()));

    let email_verification_response = database_layer
        .query()
        .email_verification
        .get(user_id.clone())
        .await?;
    println!("2. Email verification existence checked successfully!");

    // 3. Validate unauthorized session

    let email_verification_id_matches = verify_string_hash(
        email_verification_response.id.id.to_string(),
        payload.email_verification_id_hash.clone(),
    );

    if !email_verification_id_matches {
        return Err(ApiError(EmailVerificationError::InvalidToken));
    }

    let email_verification_id = Thing::from((
        String::from("email_verification"),
        email_verification_response.id.id.to_string(),
    ));

    // 3. Update user verified status
    let user = database_layer
        .query()
        .user
        .verify_user(user_id.clone())
        .await?;
    println!("2. User verified status updated successfully!");

    // 4. Remove email verification
    database_layer
        .query()
        .email_verification
        .remove(email_verification_id.clone())
        .await?;
    println!("3. Email verification removed successfully!");

    // 5. Remove all user sessions (should only have unauthrized ones)

    database_layer
        .query()
        .session
        .invalidate_all(user_id)
        .await?;

    // 6. Send email to user confirming the account verification

    email_layer
        .send_email_verification_confirmation(user.first().unwrap().email.clone())
        .await?;

    Ok((
        StatusCode::OK,
        Json(RouteOutput {
            message: String::from("Email verified successfully"),
        }),
    ))
}
