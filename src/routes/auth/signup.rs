use axum::{extract::State, Extension, Json};

use hyper::StatusCode;
use serde::Deserialize;
use validator::Validate;

use crate::{
    errors::auth::signup::SignupError,
    services::{
        database::{user::User, DatabaseLayer},
        email::EmailLayer,
    },
    utils::{
        crypto::{hash_password, string_hash},
        random::generate_random_code,
    },
    AppState,
};

#[derive(Debug, Deserialize, Validate)]
struct EmailInput {
    #[validate(email)]
    email: String,
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
    Json(payload): Json<User>,
) -> Result<(StatusCode, Json<User>), SignupError> {
    let email = payload.email.clone();
    let email_input = EmailInput {
        email: email.clone(),
    };

    if email_input.validate().is_err() {
        return Err(SignupError::ValidationError);
    }

    let password_hash = hash_password(String::from(payload.password))
        .await
        .map_err(|_| SignupError::HashingError)?;

    let email_available = database_layer
        .query()
        .user
        .check_if_exists(email.clone())
        .await;

    match email_available {
        Ok(false) => return Err(SignupError::EmailUnavailableError),
        Ok(true) => {
            let new_user_result = database_layer
                .query()
                .user
                .create(payload.email.clone(), password_hash)
                .await;

            let new_user = match new_user_result {
                Ok(Some(user)) => user,
                Ok(None) => return Err(SignupError::OtherError("Error".to_string())),
                Err(e) => return Err(SignupError::DatabaseError(e)),
            };

            //TODO: create an email_verification in a database (new table)
            let verification_code = generate_random_code(6);

            let verification_code_hash = string_hash(verification_code.clone());

            let new_email_verification = database_layer
                .query()
                .email_verification
                .create(verification_code_hash, email.clone())
                .await;

            match new_email_verification {
                Ok(_) => {
                    if let Err(e) = email_layer
                        .send_email_verification(email.clone(), verification_code)
                        .await
                    {
                        return Err(SignupError::EmailError(e));
                    }
                }
                Err(e) => return Err(SignupError::DatabaseError(e)),
            }

            // TODO: Once the route handler is fully complete remove this response and make it more
            // generic (so it doesn't expose any data)
            // TODO: As a response return a session that is unauthorized
            Ok((
                StatusCode::CREATED,
                Json(User {
                    email: "whatever".to_string(),
                    password: "whatever".to_string(),
                    email_verified: false,
                }),
            ))
        }
        Err(e) => Err(SignupError::DatabaseError(e)),
    }
}
