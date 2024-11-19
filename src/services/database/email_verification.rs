use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::Client,
    sql::{Datetime, Thing},
    Surreal,
};
use validator::Validate;

use crate::{routes::auth::email_verification, utils::crypto::generate_token};

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct EmailVerification {
    pub id: Thing,
    pub code: String,

    #[serde(default)]
    pub created_at: Datetime,
    #[serde(default)]
    expires_at: Datetime,

    #[serde(rename = "user")]
    user: Thing,
}

impl EmailVerification {
    pub fn new(
        id: Thing,
        code: String,
        created_at: Datetime,
        expires_at: Datetime,
        user_id: Thing,
    ) -> Self {
        EmailVerification {
            id,
            code,
            created_at,
            expires_at,
            user: user_id,
        }
    }
}

#[derive(Clone)]
pub struct EmailVerificationQuery<'a> {
    db: &'a Surreal<Client>,
}

impl<'a> EmailVerificationQuery<'a> {
    pub(crate) fn new(db: &'a Surreal<Client>) -> Self {
        Self { db }
    }
}

impl<'a> EmailVerificationQuery<'a> {
    // TODO: This function needs better error exception handling, take a look at it in free time
    // TODO: Check if providing email is necessary since we already provide user_id
    pub async fn create(
        &self,
        code: String,
        email: String,
        user_id: Thing,
    ) -> Result<EmailVerification, surrealdb::Error> {
        let email_verification_id_str = generate_token();
        let email_verification_id = Thing::from((
            "email_verification".to_string(),
            email_verification_id_str.clone(),
        ));

        let now: DateTime<Utc> = Utc::now();
        let expires: DateTime<Utc> = now + Duration::minutes(5);

        let created_at = Datetime::from(now);
        let expires_at = Datetime::from(expires);

        let check_query = r#"
            SELECT * FROM email_verification
            WHERE email = $email
        "#;

        let mut check = self
            .db
            .query(check_query)
            .bind(("email", email.clone()))
            .await?;

        let email_verifications: Vec<EmailVerification> = check.take(0)?;

        if !email_verifications.is_empty() {
            let delete_query = r#"
                DELETE FROM email_verification
                WHERE email = $email
            "#;

            self.db
                .query(delete_query)
                .bind(("email", email.clone()))
                .await?;
        }

        let create_query = r#"
            CREATE email_verification CONTENT {
                id: $id,
                code: $code,
                created_at: $created_at,
                expires_at: $expires_at,
                user: $user_id
            };
        "#;

        let mut response: surrealdb::Response = self
            .db
            .query(create_query)
            .bind(("id", email_verification_id.clone()))
            .bind(("code", code.clone()))
            .bind(("expires_at", expires_at.clone()))
            .bind(("created_at", created_at.clone()))
            .bind(("user_id", user_id.clone()))
            .await?;

        let created: Option<EmailVerification> = response.take(0)?;

        match created {
            Some(email_verification) => Ok(email_verification),
            None => Err(surrealdb::Error::Api(
                surrealdb::error::Api::InvalidRequest(
                    "Failed to create email verification".to_string(),
                ),
            )),
        }
    }

    pub async fn get(&self, user_id: Thing) -> Result<EmailVerification, surrealdb::Error> {
        let query = r#"
            SELECT * FROM email_verification
            WHERE user.id = $id
        "#;

        let mut response: surrealdb::Response =
            self.db.query(query).bind(("id", user_id.clone())).await?;

        let mut result: Vec<Option<EmailVerification>> = response.take(0)?;

        match result.pop().flatten() {
            Some(user) => Ok(user),
            None => Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidParams(
                String::from("User with provided email couldn't be found"),
            ))),
        }
    }

    pub async fn remove(&self, email_verification_id: Thing) -> Result<(), surrealdb::Error> {
        let query = r#"
            DELETE FROM email_verification
            WHERE id = $email_verification_id
            RETURN BEFORE
        "#;

        let mut response: surrealdb::Response = self
            .db
            .query(query)
            .bind(("email_verification_id", email_verification_id))
            .await?;

        let result: Vec<EmailVerification> = response.take(0)?;

        // Check if the deletion affected any rows (result should be empty if nothing was deleted)
        if result.is_empty() {
            return Err(surrealdb::Error::Api(
                surrealdb::error::Api::InvalidRequest(String::from(
                    "Email verification either doesn't exist or is already deleted",
                )),
            ));
        }

        Ok(())
    }
}
