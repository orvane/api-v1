use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

#[derive(Debug, Deserialize, Validate)]
struct EmailInput {
    #[validate(email)]
    email: String,
}

// #[derive(Debug, Deserialize, Validate)]
// struct PasswordInput {
//     password: String,
// }

#[derive(Debug, Deserialize, Validate)]
pub struct SignUpRequest {
    email: String,
}

#[derive(Serialize, Validate)]
pub struct User {
    id: Uuid,
    #[validate(email)]
    email: String,
}

async fn validate_email_input(email: &str) -> Result<(), ValidationErrors> {
    let email_input = EmailInput {
        email: email.to_string(),
    };

    match email_input.validate() {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

async fn check_if_user_exists(email: &str) -> Result<bool, String> {
    validate_email_input(email)
        .await
        .map_err(|_| "Invalid email format".to_string())?;

    // Some logic to call a database, throw errors if any, validate input and throw validation
    // errors if there are any
    let db_dummy_data = vec![
        "user1@example.com",
        "user2@example.com",
        "user3@example.com",
    ];
    let result = db_dummy_data.contains(&email);

    match result {
        true => Ok(true),
        false => Ok(false),
    }
}

pub async fn user_signup(Json(payload): Json<SignUpRequest>) -> Result<Response, StatusCode> {
    if let Err(e) = validate_email_input(&payload.email).await {
        let response_body = Json(json!({"message": "Bad request", "data": e.to_string()}));

        return Ok((StatusCode::BAD_REQUEST, response_body).into_response());
    }

    match check_if_user_exists(&payload.email).await {
        Ok(false) => {}
        Ok(true) => {
            let response_body = Json(json!({"message": "User already exists"}));

            return Ok((StatusCode::CONFLICT, response_body).into_response());
        }
        Err(e) => {
            let response_body =
                Json(json!({"message": "Internal server error", "data": e.to_string()}));

            return Ok((StatusCode::INTERNAL_SERVER_ERROR, response_body).into_response());
        }
    }

    let response_body = Json(json!({"message": "User created successfully"}));

    Ok((StatusCode::CREATED, response_body).into_response())
}
