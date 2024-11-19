use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::Client,
    sql::{Datetime, Thing},
    Surreal,
};
use validator::Validate;

use crate::utils::crypto::{generate_token, hash_token};

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct Session {
    pub id: Thing,
    pub authorized: bool,

    #[serde(default)]
    pub created_at: Datetime,
    #[serde(default)]
    pub expires_at: Datetime,
    #[serde(default)]
    pub last_accessed_at: Datetime,

    pub user: Thing,
}

#[derive(Clone)]
pub struct SessionQuery<'a> {
    db: &'a Surreal<Client>,
}

impl<'a> SessionQuery<'a> {
    pub(crate) fn new(db: &'a Surreal<Client>) -> Self {
        Self { db }
    }
}

impl<'a> SessionQuery<'a> {
    // TODO: Split into two functions (create_unauthorized) to avoid booleans as an argument
    pub async fn create(
        &self,
        user_id: Thing,
        authorized: bool,
    ) -> Result<Session, surrealdb::Error> {
        let token = generate_token();

        let session_id_str = hash_token(&token);
        let session_id = Thing::from(("session".to_string(), session_id_str.clone()));

        let now: DateTime<Utc> = Utc::now();
        let expires: DateTime<Utc> = now + Duration::days(30);

        let created_at = Datetime::from(now);
        let expires_at = Datetime::from(expires);

        let query = r#"
            CREATE session CONTENT {
                id: $id,
                authorized: $authorized,
                created_at: $created_at,
                expires_at: $expires_at,
                last_accessed_at: $last_accessed_at,
                user: $user
            }
        "#;

        let mut response: surrealdb::Response = self
            .db
            .query(query)
            .bind(("id", session_id.clone()))
            .bind(("authorized", authorized.clone()))
            .bind(("created_at", created_at.clone()))
            .bind(("expires_at", expires_at.clone()))
            .bind(("last_accessed_at", created_at.clone()))
            .bind(("user", user_id.clone()))
            .await?;

        let created: Option<Session> = response.take(0)?;

        match created {
            Some(session) => Ok(session),
            None => Err(surrealdb::Error::Api(
                surrealdb::error::Api::InvalidRequest("Failed to create session".to_string()),
            )),
        }

        // Ok(Session {
        //     id: session_id,
        //     created_at: created_at.clone(),
        //     expires_at,
        //     authorized,
        //     last_accessed_at: created_at,
        //     user: user_id,
        // })
    }

    pub async fn invalidate_all(&self, user_id: Thing) -> Result<(), surrealdb::Error> {
        let query = r#"
            DELETE session 
            WHERE user = $id
            RETURN BEFORE;
        "#;

        let mut response: surrealdb::Response =
            self.db.query(query).bind(("id", user_id.clone())).await?;

        let result: Vec<Session> = response.take(0)?;

        // Check if the deletion affected any rows (result should be empty if nothing was deleted)
        if result.is_empty() {
            return Err(surrealdb::Error::Api(
                surrealdb::error::Api::InvalidRequest(String::from(
                    "No session found for the specified user, or the user does not exist.",
                )),
            ));
        }

        Ok(())
    }
}
