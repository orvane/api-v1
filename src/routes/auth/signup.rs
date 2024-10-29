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
}

impl User {
    pub fn new(email: String) -> User {
        User { email }
    }
}

#[derive(Debug, Display)]
pub enum SignupError {
    ValidationError,
    EmailUnavailableError,
    DatabaseError(surrealdb::Error),
    EmailError(String),
}

impl IntoResponse for SignupError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            SignupError::ValidationError => StatusCode::BAD_REQUEST,
            SignupError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SignupError::EmailUnavailableError => StatusCode::BAD_REQUEST,
            SignupError::EmailError(_) => StatusCode::BAD_REQUEST,
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

    let email_available = check_if_user_exists(&email, &app_state.db).await;

    match email_available {
        Ok(false) => return Err(SignupError::EmailUnavailableError),
        Ok(true) => {
            let new_user = User::new(String::from(email.clone()));

            let _: Option<User> = app_state
                .db
                .create(("user", &payload.email))
                .content(new_user.clone())
                .await
                .map_err(SignupError::DatabaseError)?;

            if let Err(e) = email_layer
                .send_email_verification(email.clone(), String::from("123456"))
                .await
            {
                return Err(SignupError::EmailError(e));
            }

            Ok((StatusCode::CREATED, Json(new_user)))
        }
        Err(e) => Err(SignupError::DatabaseError(e)),
    }
}
