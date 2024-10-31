mod errors;
mod routes;
mod services;
mod utils;

use axum::{routing::post, Extension, Router};
use dotenv::dotenv;
use routes::auth::signup::{self};
use services::{database_service::DatabaseLayer, email_service::EmailLayer};
use std::env;

#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    dotenv().ok();
    let shared_state = AppState {};

    // TODO: Put values from below as the env sercrets (for now)
    let database_layer = DatabaseLayer::new(
        String::from("root"),
        String::from("root"),
        String::from("127.0.0.1:8000"),
        String::from("orvane"),
        String::from("test"),
    )
    .await?;

    let email_layer = EmailLayer::new(
        env::var("RESEND_API_KEY").unwrap_or_else(|_| {
            println!("Resend API key error");
            String::new()
        }),
        String::from("blazar.lol"),
    );

    let app = Router::new()
        .route("/api/v1/auth/signup", post(signup::signup))
        .layer(Extension(database_layer))
        .layer(Extension(email_layer))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
