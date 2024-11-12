use crate::services::database::DatabaseLayer;

pub async fn setup_database() -> surrealdb::Result<DatabaseLayer> {
    DatabaseLayer::new(
        String::from("root"),
        String::from("root"),
        String::from("127.0.0.1:8000"),
        String::from("orvane"),
        String::from("test"),
    )
    .await
}
