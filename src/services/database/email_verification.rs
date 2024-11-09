use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Uuid, Surreal};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct EmailVerification {
    pub code: String,
    pub email: String,
}

impl EmailVerification {
    pub fn new(email: String, code: String) -> Self {
        EmailVerification { email, code }
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
    pub async fn create(
        &self,
        code: String,
        email: String,
    ) -> Result<EmailVerification, surrealdb::Error> {
        let new_email_verification = EmailVerification::new(code.clone(), email.clone());

        let id = Uuid::new_v4().to_string();

        let check_query = r#"
            SELECT * FROM email_verification
            WHERE email = $email
        "#;

        let mut check = self
            .db
            .query(check_query)
            .bind(("email", new_email_verification.email.clone()))
            .await?;

        let email_verifications: Vec<EmailVerification> = check.take(0)?;

        if !email_verifications.is_empty() {
            let delete_query = r#"
                DELETE FROM email_verification
                WHERE email = $email
            "#;

            self.db
                .query(delete_query)
                .bind(("email", new_email_verification.email.clone()))
                .await?;
        }

        let create_query = r#"
            CREATE email_verification
            SET
                id = $id,
                email = $email,
                code = $code,
                created_ay = time::now()
        "#;

        let mut result = self
            .db
            .query(create_query)
            .bind(("id", id))
            .bind(("email", new_email_verification.email))
            .bind(("code", new_email_verification.code))
            .await?;

        let email_verification: Vec<EmailVerification> = result.take(0)?;

        Ok(email_verification.into_iter().next().unwrap())
    }

    pub async fn remove(&self, email_verification_id: String) -> Result<(), surrealdb::Error> {
        let query = r#"
            DELETE FROM email_verification
            WHERE id = $email_verification_id
        "#;

        let mut result: surrealdb::Response = self
            .db
            .query(query)
            .bind(("email_verification_id", email_verification_id))
            .await?;

        let affected: Vec<EmailVerification> = result.take(0)?;

        if affected.is_empty() {
            return Err(surrealdb::Error::Api(
                surrealdb::error::Api::InvalidRequest(String::from(
                    "Email verification either doesn't exist or is already verified",
                )),
            ));
        }

        Ok(())
    }
}
