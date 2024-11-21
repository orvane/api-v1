use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::Client,
    sql::{
        statements::{BeginStatement, CommitStatement},
        Datetime, Thing,
    },
    Surreal,
};
use validator::Validate;

use crate::utils::crypto::generate_token;

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct PasswordResetRequest {
    pub id: Thing,

    #[serde(default)]
    pub created_at: Datetime,
    #[serde(default)]
    pub expires_at: Datetime,

    user: Thing,
}

impl PasswordResetRequest {
    pub fn new(id: Thing, created_at: Datetime, expires_at: Datetime, user: Thing) -> Self {
        Self {
            id,
            created_at,
            expires_at,
            user,
        }
    }
}

#[derive(Clone)]
pub struct PasswordResetRequestQuery<'a> {
    db: &'a Surreal<Client>,
}

impl<'a> PasswordResetRequestQuery<'a> {
    pub(crate) fn new(db: &'a Surreal<Client>) -> Self {
        Self { db }
    }
}

impl<'a> PasswordResetRequestQuery<'a> {
    pub async fn create(&self, user: Thing) -> Result<PasswordResetRequest, surrealdb::Error> {
        let password_reset_request_id_str = generate_token();
        let password_reset_request_id = Thing::from((
            "password_reset_request".to_string(),
            password_reset_request_id_str.clone(),
        ));

        let now: DateTime<Utc> = Utc::now();
        let expires: DateTime<Utc> = now + Duration::hours(1);

        let created_at = Datetime::from(now);
        let expires_at = Datetime::from(expires);

        let delete_query = r#"
                DELETE FROM password_reset_request
                WHERE user.id = $user
            "#;

        let create_query = r#"
            CREATE password_reset_request CONTENT {
                id: $id,
                created_at: $created_at,
                expires_at: $expires_at,
                user: $user
            }
        "#;

        let mut create_response: surrealdb::Response = self
            .db
            .query(BeginStatement::default())
            .query(delete_query)
            .query(create_query)
            .query(CommitStatement::default())
            .bind(("id", password_reset_request_id))
            .bind(("created_at", created_at))
            .bind(("expires_at", expires_at))
            .bind(("user", user))
            .await?;

        let create_result: Option<PasswordResetRequest> = create_response.take(1)?;

        match create_result {
            Some(password_reset_request) => Ok(password_reset_request),
            None => Err(surrealdb::Error::Api(
                surrealdb::error::Api::InvalidRequest(
                    "Failed to create password reset request".to_string(),
                ),
            )),
        }
    }
}
