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
use rand::{thread_rng, Rng};
use resend_rs::types::ErrorResponse;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use surrealdb::{engine::remote::ws::Client, sql::Uuid, Surreal};
use validator::Validate;

use crate::{services::email_service::EmailLayer, AppState};

#[derive(Debug, Deserialize, Validate)]
struct EmailInput {
    #[validate(email)]
    email: String,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct EmailVerification {
    code: String,
    email: String,
}

impl EmailVerification {
    pub async fn new(
        code: String,
        email: String,
        db: &Surreal<Client>,
    ) -> Result<EmailVerification, surrealdb::Error> {
        let id = Uuid::new_v4().to_string();

        let check_query = r#"
            SELECT * FROM email_verification
            WHERE email = $email
        "#;

        let mut check = db.query(check_query).bind(("email", email.clone())).await?;

        let email_verifications: Vec<EmailVerification> = check.take(0)?;

        if !email_verifications.is_empty() {
            let delete_query = r#"
                DELETE FROM email_verification
                WHERE email = $email
            "#;

            db.query(delete_query)
                .bind(("email", email.clone()))
                .await?;
        }

        let create_query = r#"
            CREATE email_verification
            SET
                id = $id,
                email = $email,
                code = $code,
                created_ay = time::now()
        "#;

        let mut result = db
            .query(create_query)
            .bind(("id", id))
            .bind(("email", email))
            .bind(("code", code))
            .await?;

        let created: Vec<EmailVerification> = result.take(0)?;

        Ok(created.into_iter().next().unwrap())
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct User {
    #[validate(email)]
    pub email: String,
    //TODO: Add custom valdation for the password
    pub password: String,

    #[serde(default)]
    pub email_verified: bool,
}

impl User {
    pub fn new(email: String, password: String) -> User {
        User {
            email,
            password,
            email_verified: false,
        }
    }
}

#[derive(Debug, Display)]
pub enum SignupError {
    ValidationError,
    EmailUnavailableError,
    DatabaseError(surrealdb::Error),
    EmailError(String),
    HashingError,
    OtherError(String),
}

impl IntoResponse for SignupError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            // TODO: Make these errors more descriptive (also make them use their actual message)
            // TODO: Move errors to a different folder
            SignupError::ValidationError => StatusCode::BAD_REQUEST,
            SignupError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SignupError::EmailUnavailableError => StatusCode::BAD_REQUEST,
            SignupError::EmailError(_) => StatusCode::BAD_REQUEST,
            SignupError::HashingError => StatusCode::INTERNAL_SERVER_ERROR,
            SignupError::OtherError(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

pub fn generate_random_code(length: usize) -> String {
    let mut rng = thread_rng();

    let code: String = (0..length)
        .map(|_| rng.gen_range(0..10).to_string())
        .collect();

    code
}

pub fn string_hash(input: String) -> String {
    let mut hasher = Sha256::new();

    hasher.update(input.as_bytes());

    let result = hasher.finalize();

    result.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub fn verify_string_hash(input: String, hash: String) -> bool {
    let new_hash = string_hash(input);

    hash == new_hash
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

            //TODO: create an email_verification in a database (new table)
            let verification_code = generate_random_code(6);

            let verification_code_hash = string_hash(verification_code.clone());

            let email_verification =
                EmailVerification::new(verification_code_hash, email.clone(), &app_state.db).await;

            match email_verification {
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
            Ok((StatusCode::CREATED, Json(new_user)))
        }
        Err(e) => Err(SignupError::DatabaseError(e)),
    }
}
