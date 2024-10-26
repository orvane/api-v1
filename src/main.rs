mod routes;

use axum::{routing::post, Router};
use routes::auth::signup::{self};
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
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;
    db.use_ns("orvane").use_db("test").await?;

    let shared_state = AppState { db };

    let app = Router::new()
        .route("/api/v1/auth/signup", post(signup::signup))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
