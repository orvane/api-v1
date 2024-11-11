use axum::{extract::State, Extension, Json};

use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    errors::{auth::SignupError, response::ApiError},
    services::{
        database::{user::User, DatabaseLayer},
        email::EmailLayer,
    },
    utils::{
        crypto::{hash_password, hash_string},
        random::generate_random_code,
    },
    AppState,
};

#[derive(Debug, Deserialize, Validate)]
pub struct RoutePayload {
    #[validate(email)]
    email: String,
    // TODO: custom password validation functions
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteOutput {
    message: String,
}

// TODO: In case if any of the steps fails the previous steps need to be reverted
// For example: If user gets created, but email-verification creation fails it needs to reverse the
// user creation and throw the INTERNAL_SERVER_ERROR
// The whole process should be handled manually since even if the email doesn't get sent to the
// user the user record should still stay in the database as the email-verification request can be
// created at any time
#[axum::debug_handler]
pub async fn signup(
    State(app_state): State<AppState>,
    Extension(database_layer): Extension<DatabaseLayer>,
    Extension(email_layer): Extension<EmailLayer>,
    Json(payload): Json<RoutePayload>,
) -> Result<(StatusCode, Json<RouteOutput>), ApiError<SignupError>> {
    // 1. Validate payload input
    let payload_instance = RoutePayload {
        email: payload.email.clone(),
        password: payload.password.clone(),
    };

    payload_instance.validate()?;
    println!("1. Validation passed successfully!");

    // 2. Check if the email is available

    let check_if_exists = database_layer
        .query()
        .user
        .check_if_exists(payload.email.clone())
        .await?;

    if check_if_exists {
        return Err(ApiError(SignupError::EmailAlreadyExists));
    }
    println!("2. Email availability check completed successfully!");

    // 3. Create a new user in the database

    let password_hash = hash_password(payload.password.clone()).await?;
    println!("3. Password hashed successfully!");

    database_layer
        .query()
        .user
        .create(payload.email.clone(), password_hash)
        .await?;
    println!("4. User created successfully!");

    // 4. Create email verification in the database

    let verification_code = generate_random_code(6);
    let verification_code_hash = hash_string(verification_code.clone());

    database_layer
        .query()
        .email_verification
        .create(verification_code_hash, payload.email.clone())
        .await?;
    println!("5. Email verification created successfully!");

    // 5. Send email verification email

    email_layer
        .send_email_verification(payload.email, verification_code)
        .await?;
    println!("6. Email verification email sent successfully!");

    // TODO: Return an unauthorized cookie (the cookie is also going to be constructed in case a
    // user wants to verify an account on another device)
    Ok((
        StatusCode::OK,
        Json(RouteOutput {
            message: String::from("Signup completed successfully!"),
        }),
    ))
}
