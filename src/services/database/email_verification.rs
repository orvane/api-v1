use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::Client,
    sql::{Datetime, Thing},
    Surreal,
};
use validator::Validate;

use crate::utils::crypto::generate_token;

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct EmailVerification {
    pub id: Thing,
    pub code: String,
    pub email: String,

    #[serde(default)]
    pub created_at: Datetime,
    #[serde(default)]
    expires_at: Datetime,
}

impl EmailVerification {
    pub fn new(
        id: Thing,
        email: String,
        code: String,
        created_at: Datetime,
        expires_at: Datetime,
    ) -> Self {
        EmailVerification {
            id,
            email,
            code,
            created_at,
            expires_at,
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
            BEGIN TRANSACTION;

            CREATE type::thing("email_verification", $id) SET
                id = $id,
                email = $email,
                code = $code,
                created_at = $created_at,
                expires_at = $expires_at;

            RELATE $user_id -> email_verification -> $id;

            COMMIT TRANSACTION;
        "#;

        self.db
            .query(create_query)
            .bind(("id", email_verification_id.clone()))
            .bind(("email", email.clone()))
            .bind(("code", code.clone()))
            .bind(("user_id", user_id.clone()))
            .bind(("expires_at", expires_at.clone()))
            .bind(("created_at", created_at.clone()))
            .await?;

        Ok(EmailVerification::new(
            email_verification_id,
            email,
            code,
            created_at,
            expires_at,
        ))
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
