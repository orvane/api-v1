use crate::{
    routes,
    services::{database::DatabaseLayer, email::EmailLayer},
};
use axum::{Extension, Router};
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct AppState {}

pub async fn setup_api_router(
    database_layer: DatabaseLayer,
    email_layer: EmailLayer,
) -> surrealdb::Result<(Router, TcpListener)> {
    let shared_state = AppState {};

    let app = routes::main_router()
        .layer(Extension(database_layer))
        .layer(Extension(email_layer))
        .with_state(shared_state);

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    Ok((app, listener))
}
