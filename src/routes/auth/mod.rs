pub mod email_verification;
pub mod password_reset_request;
pub mod signin;
pub mod signup;

use axum::{routing::post, Router};
pub use email_verification::email_verification;
pub use password_reset_request::password_reset_request;
pub use signin::signin;
pub use signup::signup;

use crate::setup::AppState;

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/signup", post(signup))
        .route("/signin", post(signin))
        .route("/email-verification", post(email_verification))
        .route("/password-reset-request", post(password_reset_request))
}
