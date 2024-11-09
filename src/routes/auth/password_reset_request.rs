use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    errors::auth::password_reset_request::PasswordResetRequestError,
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
) -> Result<(StatusCode, Json<RouteOutput>), PasswordResetRequestError> {
    // 1. Validate payload input
    let payload_instance = RoutePayload {
        email: payload.email.clone(),
    };

    match payload_instance.validate() {
        Ok(_) => println!("Validation passed succesfully."),
        Err(e) => return Err(PasswordResetRequestError::ValidationError(e)),
    }

    // 2. Create password reset request in the database

    let id = generate_uuid();
    let id_hash = hash_string(id.clone());

    let create_password_reset_request = database_layer
        .query()
        .password_reset_request
        .create(id_hash.clone(), String::from(payload.email.clone()))
        .await;

    match create_password_reset_request {
        Ok(_) => println!("Password reset request creation completed successfully."),
        Err(e) => return Err(PasswordResetRequestError::DatabaseError(e)),
    }

    // 3. Send an email with the details on how to reset the password

    let send_password_reset = email_layer
        .send_password_reset(payload.email.clone(), id.clone())
        .await;

    match send_password_reset {
        Ok(_) => println!("Password reset email sent successfully!"),
        Err(e) => return Err(PasswordResetRequestError::EmailServiceError(e)),
    }

    Ok((
        StatusCode::OK,
        Json(RouteOutput {
            message: String::from("Password reset request created successfully"),
        }),
    ))
}
