use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use validator::Validate;

use crate::{
    errors::{auth::PasswordResetError, response::ApiError},
    services::{database::DatabaseLayer, email::EmailLayer},
};

#[derive(Debug, Deserialize, Validate)]
pub struct RoutePayload {
    password_reset_request_id: Thing,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteOutput {
    message: String,
}

#[axum::debug_handler]
pub async fn password_reset(
    Extension(database_layer): Extension<DatabaseLayer>,
    Extension(email_layer): Extension<EmailLayer>,
    Json(payload): Json<RoutePayload>,
) -> Result<(StatusCode, Json<RouteOutput>), ApiError<PasswordResetError>> {
    // 1. Validate payload input
    let payload_instance = RoutePayload {
        password_reset_request_id: payload.password_reset_request_id.clone(),
    };

    payload_instance.validate()?;
    println!("Validation passed successfully.");

    Ok((
        StatusCode::OK,
        Json(RouteOutput {
            message: String::from("Password reset completed successfully"),
        }),
    ))
}
