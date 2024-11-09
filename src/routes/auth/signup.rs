use axum::{extract::State, Extension, Json};

use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    errors::auth::signup::SignupError,
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
// TODO: Clean up match statements (remove matches on Result and Option when possible)
#[axum::debug_handler]
pub async fn signup(
    State(app_state): State<AppState>,
    Extension(database_layer): Extension<DatabaseLayer>,
    Extension(email_layer): Extension<EmailLayer>,
    Json(payload): Json<RoutePayload>,
) -> Result<(StatusCode, Json<RouteOutput>), SignupError> {
    // 1. Validate payload input
    let payload_instance = RoutePayload {
        email: payload.email.clone(),
        password: payload.password.clone(),
    };

    match payload_instance.validate() {
        Ok(_) => println!("Validation passed successfully!"),
        Err(e) => return Err(SignupError::ValidationError(e)),
    }

    // 2. Check if the email is available

    let check_if_exists = database_layer
        .query()
        .user
        .check_if_exists(payload.email.clone())
        .await;

    match check_if_exists {
        Ok(_) => println!("Email availability check completed successfully!"),
        // TODO: Add a custom error in case the email is already taken
        Err(e) => return Err(SignupError::DatabaseError(e)),
    }

    // 3. Create a new user in the database

    let password_hash = hash_password(payload.password.clone()).await;

    let password_hash = match password_hash {
        Ok(hash) => {
            println!("Password hashed successfully!");
            hash
        }
        Err(e) => return Err(SignupError::HashingError(e)),
    };

    let create_new_user = database_layer
        .query()
        .user
        .create(payload.email.clone(), password_hash)
        .await;

    match create_new_user {
        Ok(_) => println!("User created successfully!"),
        Err(e) => return Err(SignupError::DatabaseError(e)),
    }

    // 4. Create email verification in the database

    let verification_code = generate_random_code(6);
    let verification_code_hash = hash_string(verification_code.clone());

    let create_new_email_verification = database_layer
        .query()
        .email_verification
        .create(verification_code_hash, payload.email.clone())
        .await;

    match create_new_email_verification {
        Ok(_) => println!("Email verification created successfully!"),
        Err(e) => return Err(SignupError::DatabaseError(e)),
    }

    // 5. Send email verification email

    let send_email_verification_email = email_layer
        .send_email_verification(payload.email, verification_code)
        .await;

    match send_email_verification_email {
        Ok(_) => println!("Email verification email sent successfully!"),
        Err(e) => return Err(SignupError::EmailServiceError(e)),
    }

    Ok((
        StatusCode::OK,
        Json(RouteOutput {
            message: String::from("Signup completed successfully!"),
        }),
    ))
}
