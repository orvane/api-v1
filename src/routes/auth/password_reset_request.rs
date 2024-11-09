use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    errors::auth::password_reset_request::PasswordResetRequestError,
    services::database::DatabaseLayer,
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
    Json(payload): Json<RoutePayload>,
) -> Result<(StatusCode, Json<RouteOutput>), PasswordResetRequestError> {
    // 1. Validate payload input
    let payload_instance = RoutePayload {
        email: payload.email.clone(),
    };

    // 2. Create password reset request in the database

    // let create_password_reset_request = database_layer
    //     .query()
    //     .password_reset
    //     .create(String::from(payload.email))
    //     .await;

    match payload_instance.validate() {
        Ok(_) => println!("Validation passed succesfully."),
        Err(e) => return Err(PasswordResetRequestError::ValidationError(e)),
    }

    Ok((
        StatusCode::OK,
        Json(RouteOutput {
            message: String::from("Password reset request created successfully"),
        }),
    ))
}
