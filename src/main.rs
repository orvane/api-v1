mod routes;
use axum::{routing::post, Router};
use routes::auth::signup::user_signup;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/api/v1/auth/signup", post(user_signup));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap()
}
