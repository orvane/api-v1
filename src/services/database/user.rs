use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, Surreal};
use validator::{Validate, ValidateRequired};

use crate::utils::crypto::generate_uuid;

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct User {
    #[validate(email)]
    pub email: String,
    //TODO: Add custom valdation for the password
    pub password: String,

    #[serde(default)]
    pub email_verified: bool,
}

impl User {
    pub fn new(email: String, password: String) -> Self {
        User {
            email,
            password,
            email_verified: false,
        }
    }
}

#[derive(Clone)]
pub struct UserQuery<'a> {
    db: &'a Surreal<Client>,
}

impl<'a> UserQuery<'a> {
    pub(crate) fn new(db: &'a Surreal<Client>) -> Self {
        Self { db }
    }
}

impl<'a> UserQuery<'a> {
    pub async fn create(
        &self,
        email: String,
        password_hash: String,
    ) -> Result<Option<User>, surrealdb::Error> {
        let id = generate_uuid();
        let new_user = User::new(String::from(email.clone()), password_hash);

        let user: Option<User> = self.db.create(("user", id)).content(new_user).await?;

        Ok(user)
    }

    // TODO: Instead of using string referance use normal string and clone the input in the
    // implemenation
    pub async fn check_if_exists(&self, email: String) -> Result<bool, surrealdb::Error> {
        let query = r#"
            SELECT * FROM user
            WHERE email = $user_email
        "#;

        let mut response: surrealdb::Response = self
            .db
            .query(query)
            .bind(("user_email", email.clone()))
            .await?;

        let result: Vec<Option<User>> = response.take(0)?;

        Ok(!result.is_empty())
    }

    pub async fn verify_user(&self, user_id: String) -> Result<(), surrealdb::Error> {
        let query = r#"
            UPDATE user
            SET email_verified = true
            WHERE id = $user_id
        "#;

        let mut result: surrealdb::Response =
            self.db.query(query).bind(("user_id", user_id)).await?;

        let affected: Vec<User> = result.take(0)?;

        if affected.is_empty() {
            return Err(surrealdb::Error::Api(
                surrealdb::error::Api::InvalidRequest(String::from(
                    "User either doesn't exist or is already verified",
                )),
            ));
        }

        Ok(())
    }
}
