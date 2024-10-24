use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Validate)]
pub struct User {
    id: Uuid,
    #[validate(email)]
    email: String,
}

async fn check_if_user_exists(email: &str) -> Result<bool, String> {
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

pub async fn user_signup() -> Result<Response, StatusCode> {
    let user_exists = check_if_user_exists("john@doe.com").await;

    match user_exists {
        Ok(true) => {
            let response_body = Json(json!({"message": "User already exists"}));
            Ok((StatusCode::CONFLICT, response_body).into_response())
        }
        Ok(false) => {
            let response_body = Json(json!({"message": "Email available"}));
            Ok((StatusCode::CREATED, response_body).into_response())
        }
        Err(_) => {
            let response_body = Json(json!({"message": "An error occured"}));
            Ok((StatusCode::INTERNAL_SERVER_ERROR, response_body).into_response())
        }
    }
}
