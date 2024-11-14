use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use surrealdb::{
    engine::remote::ws::Client,
    sql::{Datetime, Thing},
    Surreal,
};
use validator::Validate;

use crate::utils::crypto::{generate_session_token, generate_uuid};

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct Session {
    id: Thing,
    #[serde(rename = "user")]
    user_id: Thing,
    created_at: Datetime,
    expires_at: Datetime,
    authorized: bool,
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
    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();

        hasher.update(token);
        hex::encode(hasher.finalize())
    }

    pub async fn create(
        &self,
        user_id: Thing,
        authorized: bool,
    ) -> Result<Session, surrealdb::Error> {
        let token = generate_session_token();

        let session_id_str = self.hash_token(&token);
        let session_id = Thing::from(("session".to_string(), session_id_str.clone()));

        let now: DateTime<Utc> = Utc::now();
        let expires: DateTime<Utc> = now + Duration::days(30);

        let created_at = Datetime::from(now);
        let expires_at = Datetime::from(expires);

        let query = r#"
            CREATE type::thing("session", $id) SET
                user = $user_id,
                created_at = $created_at,
                expires_at = $expires_at,
                authorized = $authorized
        "#;

        self.db
            .query(query)
            .bind(("id", session_id.clone()))
            .bind(("user_id", user_id.clone()))
            .bind(("created_at", created_at.clone()))
            .bind(("expires_at", expires_at.clone()))
            .bind(("authorized", authorized.clone()))
            .await?;

        Ok(Session {
            id: session_id,
            user_id,
            created_at,
            expires_at,
            authorized,
        })
    }
}
