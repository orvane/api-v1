use axum::{
    response::{IntoResponse, Response},
    Extension, Json,
};
use hyper::{header::SET_COOKIE, StatusCode};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    errors::{auth::SigninError, response::ApiError},
    services::database::DatabaseLayer,
    utils::{cookies::set_session_cookie, crypto::verify_password_hash},
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
    // TODO: Same stuff with the Response type as signup
) -> Result<(StatusCode, Response), ApiError<SigninError>> {
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

    let session = database_layer.query().session.create(user.id, true).await?;
    println!("4. Session created successfully!");

    // 5. Create a session cookie

    let cookie = set_session_cookie(session.id.clone().id.to_string(), true);
    println!("Session cookie created successfully!");

    let mut response = (
        StatusCode::OK,
        Json(RouteOutput {
            message: String::from("Signin completed successfully!"),
        }),
    )
        .into_response();

    response
        .headers_mut()
        .insert(SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok((StatusCode::OK, response))
}
