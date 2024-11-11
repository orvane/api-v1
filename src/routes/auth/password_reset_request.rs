use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    errors::{auth::PasswordResetRequestError, response::ApiError},
    services::{database::DatabaseLayer, email::EmailLayer},
    utils::crypto::{generate_uuid, hash_string},
};

#[derive(Debug, Deserialize, Validate)]
pub struct RoutePayload {
    #[validate(email)]
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteOutput {
    message: String,
}

#[axum::debug_handler]
pub async fn password_reset_request(
    Extension(database_layer): Extension<DatabaseLayer>,
    Extension(email_layer): Extension<EmailLayer>,
    Json(payload): Json<RoutePayload>,
) -> Result<(StatusCode, Json<RouteOutput>), ApiError<PasswordResetRequestError>> {
    // 1. Validate payload input
    let payload_instance = RoutePayload {
        email: payload.email.clone(),
    };

    payload_instance.validate()?;
    println!("Validation passed successfully.");

    // 2. Create password reset request in the database
    let id = generate_uuid();
    let id_hash = hash_string(id.clone());

    database_layer
        .query()
        .password_reset_request
        .create(id_hash.clone(), payload.email.clone())
        .await?;
    println!("Password reset request creation completed successfully.");

    // 3. Send an email with the details on how to reset the password
    email_layer
        .send_password_reset(payload.email.clone(), id.clone())
        .await?;
    println!("Password reset email sent successfully!");

    Ok((
        StatusCode::OK,
        Json(RouteOutput {
            message: String::from("Password reset request created successfully"),
        }),
    ))
}
