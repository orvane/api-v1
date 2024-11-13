use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    errors::{auth::SigninError, response::ApiError},
    services::database::DatabaseLayer,
    utils::crypto::verify_password_hash,
};

#[derive(Debug, Deserialize, Validate)]
pub struct RoutePayload {
    #[validate(email)]
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteOutput {
    message: String,
}

// TODO: Add 2FA
#[axum::debug_handler]
pub async fn signin(
    Extension(database_layer): Extension<DatabaseLayer>,
    Json(payload): Json<RoutePayload>,
) -> Result<(StatusCode, Json<RouteOutput>), ApiError<SigninError>> {
    // 1. Validate payload input
    let payload_instance = RoutePayload {
        email: payload.email.clone(),
        password: payload.password.clone(),
    };

    payload_instance.validate()?;
    println!("1. Validation passed successfully!");

    // 2. Retrive user from database

    let user = match database_layer.query().user.get(payload.email.clone()).await {
        Ok(user) => user,
        Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidParams(_))) => {
            return Err(ApiError(SigninError::InvalidCredentials));
        }
        Err(err) => {
            return Err(ApiError(SigninError::Common(
                crate::errors::CommonError::Database(err),
            )))
        }
    };
    println!("2. User existence check completed successfully!");

    // 3. Verify password

    let password_matches =
        verify_password_hash(payload.password.clone(), user.password_hash).await?;

    if !password_matches {
        return Err(ApiError(SigninError::InvalidCredentials));
    }
    println!("3. Password confirmed successfully!");

    // 4. Create a session in database

    let session = database_layer.query().session.create(user.id).await?;
    println!("4. Session created successfully!");

    // 5. Create a session cookie

    Ok((
        StatusCode::OK,
        Json(RouteOutput {
            message: String::from("Signin completed successfully!"),
        }),
    ))
}
