use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Extension, Json,
};

use derive_more::Display;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, Surreal};
use validator::Validate;

use crate::{services::email_service::EmailLayer, AppState};

#[derive(Debug, Deserialize, Validate)]
struct EmailInput {
    #[validate(email)]
    email: String,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct User {
    #[validate(email)]
    pub email: String,
    //TODO: Add custom valdation for the password
    pub password: String,
}

impl User {
    pub fn new(email: String, password: String) -> User {
        User { email, password }
    }
}

#[derive(Debug, Display)]
pub enum SignupError {
    ValidationError,
    EmailUnavailableError,
    DatabaseError(surrealdb::Error),
    EmailError(String),
    HashingError,
}

impl IntoResponse for SignupError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            SignupError::ValidationError => StatusCode::BAD_REQUEST,
            SignupError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SignupError::EmailUnavailableError => StatusCode::BAD_REQUEST,
            SignupError::EmailError(_) => StatusCode::BAD_REQUEST,
            SignupError::HashingError => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(serde_json::json!({"error": self.to_string()}));
        (status_code, body).into_response()
    }
}

async fn check_if_user_exists(
    email: &String,
    db: &Surreal<Client>,
) -> Result<bool, surrealdb::Error> {
    let user: Option<User> = db.select(("user", email)).await?;

    match user {
        Some(_) => Ok(false),
        None => Ok(true),
    }
}

// TODO: Create a hashing service
pub async fn hash_password(password: String) -> Result<String, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2.hash_password(password.as_bytes(), salt.as_salt())?;

    Ok(password_hash.to_string())
}

pub async fn verify_password(
    password: String,
    hash: String,
) -> Result<bool, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let hash = PasswordHash::new(hash.as_str()).unwrap();

    let password_verifies = argon2.verify_password(password.as_bytes(), &hash);

    println!("{:?}", password_verifies);

    Ok(true)
}

#[axum::debug_handler]
pub async fn signup(
    State(app_state): State<AppState>,
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

    let email_available = check_if_user_exists(&email, &app_state.db).await;

    match email_available {
        Ok(false) => return Err(SignupError::EmailUnavailableError),
        Ok(true) => {
            let new_user = User::new(String::from(email.clone()), password_hash);

            let _: Option<User> = app_state
                .db
                .create(("user", &payload.email))
                .content(new_user.clone())
                .await
                .map_err(SignupError::DatabaseError)?;

            if let Err(e) = email_layer
                // TODO: add a service that would have a function that generates a code like below,
                // might combine it with the hashing service and name it utils or something
                .send_email_verification(email.clone(), String::from("123456"))
                .await
            {
                return Err(SignupError::EmailError(e));
            }

            // TODO: Once the route handler is fully complete remove this response and make it more
            // generic (so it doesn't expose any data)
            // TODO: As a response return a session that is unauthorized
            Ok((StatusCode::CREATED, Json(new_user)))
        }
        Err(e) => Err(SignupError::DatabaseError(e)),
    }
}
