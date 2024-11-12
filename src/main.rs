mod errors;
mod routes;
mod services;
mod setup;
mod utils;

use dotenv::dotenv;

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    dotenv().ok();

    let database = setup::setup_database().await?;
    let email = setup::setup_email_service();
    let (app, listener) = setup::setup_api_router(database, email).await?;

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
