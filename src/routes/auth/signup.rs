use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};

use derive_more::Display;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::AppState;

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
    DatabaseError(surrealdb::Error),
}

impl IntoResponse for SignupError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            SignupError::ValidationError => StatusCode::BAD_REQUEST,
            SignupError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(serde_json::json!({"error": self.to_string()}));
        (status_code, body).into_response()
    }
}

// async fn check_if_user_exists(email: &str, db: &Surreal<Client>) -> Result<bool, surrealdb::Error> {
//     let data: Result<Option<User>, surrealdb::Error> = db.select(("user", email)).await;

//     match data {
//         Ok(Some(_user)) => {
//             println!("SOMETHING");
//             Ok(true)
//         }
//         Ok(None) => {
//             println!("NOTHING");
//             Ok(false)
//         }
//         Err(e) => Err(e),
//     }
// }

#[axum::debug_handler]
pub async fn signup(
    State(app_state): State<AppState>,
    Json(payload): Json<User>,
) -> Result<(StatusCode, Json<User>), SignupError> {
    let email = payload.email.clone();
    let email_input = EmailInput {
        email: email.clone(),
    };

    if email_input.validate().is_err() {
        return Err(SignupError::ValidationError);
    }

    let new_user = User::new(String::from(email.clone()));

    // TODO: Check if the user exists manually as surrealdb only returns string like errors with
    // no code
    let create: Option<User> = app_state
        .db
        .create(("user", &payload.email))
        .content(new_user.clone())
        .await
        .map_err(SignupError::DatabaseError)?;

    Ok((StatusCode::CREATED, Json(new_user)))
}
