use crate::{
    services::database::DatabaseLayer,
    utils::schemas::{
        EMAIL_VERIFICATION_SCHEMA, PASSWORD_RESET_REQUEST_SCHEMA, SESSION_SCHEMA, USER_SCHEMA,
    },
};

pub async fn setup_database() -> surrealdb::Result<DatabaseLayer> {
    let db_layer = DatabaseLayer::new(
        String::from("root"),
        String::from("root"),
        String::from("127.0.0.1:8000"),
        String::from("orvane"),
        String::from("test"),
    )
    .await?;

    db_layer
        .initialize_schemas(vec![
            USER_SCHEMA,
            EMAIL_VERIFICATION_SCHEMA,
            SESSION_SCHEMA,
            PASSWORD_RESET_REQUEST_SCHEMA,
        ])
        .await?;

    Ok(db_layer)
}
