pub mod auth;

use axum::Router;

use crate::setup::AppState;

fn api_v1_router() -> Router<AppState> {
    Router::new().nest("/auth", auth::auth_router())
}

// Main router that serves as the entry point for all routes
pub fn main_router() -> Router<AppState> {
    Router::new().nest("/api/v1", api_v1_router())
}
