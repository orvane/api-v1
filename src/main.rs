mod routes;
mod services;

use axum::{routing::post, Extension, Router};
use dotenv::dotenv;
use routes::auth::signup::{self};
use services::email_service::EmailLayer;
use std::env;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};

#[derive(Clone)]
struct AppState {
    db: Surreal<Client>,
}

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    dotenv().ok();

    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;
    db.use_ns("orvane").use_db("test").await?;

    let shared_state = AppState { db };

    let email_layer = EmailLayer::new(
        env::var("RESEND_API_KEY").unwrap_or_else(|_| {
            println!("Resend API key error");
            String::new()
        }),
        String::from("blazar.lol"),
    );

    let app = Router::new()
        .route("/api/v1/auth/signup", post(signup::signup))
        .layer(Extension(email_layer))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
