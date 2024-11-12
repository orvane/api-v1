use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, Surreal};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct PasswordResetRequest {
    #[validate(email)]
    email: String,
}

impl PasswordResetRequest {
    pub fn new(email: String) -> Self {
        Self { email }
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
    pub async fn create(
        &self,
        id: String,
        email: String,
    ) -> Result<Option<PasswordResetRequest>, surrealdb::Error> {
        let query = r#"
            CREATE password_reset_request:$id SET 
                email = $email
                created_at = time::now()
        "#;

        let mut result: surrealdb::Response = self
            .db
            .query(query)
            .bind(("id", id))
            .bind(("email", email))
            .await?;

        let password_reset_request: Vec<PasswordResetRequest> = result.take(0)?;

        Ok(password_reset_request.into_iter().next())
    }
}
