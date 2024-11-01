use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    errors::auth::email_verification::{self, EmailVerificationError},
    services::database_service::DatabaseLayer,
    utils::validation::{
        validate_email_verification_code_format, validate_email_verification_code_length,
    },
};

#[derive(Debug, Deserialize, Validate)]
pub struct CodeStruct {
    #[validate(custom(function = "validate_email_verification_code_length"))]
    #[validate(custom(function = "validate_email_verification_code_format"))]
    code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteOutput {
    message: String,
}

// TODO: Make the route work only if the session with user id was provided
#[axum::debug_handler]
pub async fn email_verification(
    Extension(database_layer): Extension<DatabaseLayer>,
    Json(payload): Json<CodeStruct>,
) -> Result<(StatusCode, Json<RouteOutput>), EmailVerificationError> {
    // DUMMY DATA
    let user_id = "some_id";
    let email_verification_id = "some_id";

    // 1. Validate code input
    let code_instace = CodeStruct {
        code: payload.code.clone(),
    };

    match code_instace.validate() {
        Ok(_) => println!("Validation passed successfully!"),
        Err(e) => return Err(EmailVerificationError::ValidationError(e)),
    }

    // TODO: 2. Validate unauthorized session
    // Compare it via the token in the url

    // 3. Update user verified status
    let modify_user_verified_status = database_layer
        .query()
        .user
        .verify_user(String::from(user_id))
        .await;

    match modify_user_verified_status {
        Ok(_) => println!("User verified status updated successfully!"),
        Err(e) => return Err(EmailVerificationError::DatabaseError(e)),
    }

    // 4. Remove email verification

    let remove_email_verification = database_layer
        .query()
        .email_verification
        .remove(String::from(email_verification_id))
        .await;

    match remove_email_verification {
        Ok(_) => println!("Email verification removed successfully!"),
        Err(e) => return Err(EmailVerificationError::DatabaseError(e)),
    }

    // TODO: 5. remove unauthorized session

    Ok((
        StatusCode::OK,
        Json(RouteOutput {
            message: String::from("Email verified successfully"),
        }),
    ))
}
