use chrono::Utc;
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::Client,
    sql::{thing, Datetime, Thing},
    Surreal,
};
use validator::Validate;

use crate::utils::crypto::generate_uuid;

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct User {
    pub id: Thing,
    #[validate(email)]
    pub email: String,
    //TODO: Add custom valdation for the password
    pub password_hash: String,

    #[serde(default)]
    pub email_verified: bool,
    created_at: Thing,
}

impl User {
    pub fn new(id: Thing, email: String, password_hash: String, created_at: Thing) -> Self {
        User {
            id,
            email,
            password_hash,
            email_verified: false,
            created_at,
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
        let created_at = Datetime::from(Utc::now());

        let query = r#"
            CREATE type::thing("user", $id) SET
                email = $email,
                email_verified = false,
                password_hash = $password_hash,
                create_at = $created_at
        "#;

        let id = generate_uuid();

        let mut response: surrealdb::Response = self
            .db
            .query(query)
            .bind(("id", id))
            .bind(("email", email.clone()))
            .bind(("password_hash", password_hash.clone()))
            .bind(("created_at", created_at.clone()))
            .await?;

        let result: Option<User> = response.take(0)?;

        Ok(result)
    }

    pub async fn get(&self, email: String) -> Result<User, surrealdb::Error> {
        let query = r#"
            SELECT * FROM user
            WHERE email = $user_email
        "#;

        let mut response: surrealdb::Response = self
            .db
            .query(query)
            .bind(("user_email", email.clone()))
            .await?;

        let mut result: Vec<Option<User>> = response.take(0)?;

        match result.pop().flatten() {
            Some(user) => Ok(user),
            None => Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidParams(
                String::from("User with provided email couldn't be found"),
            ))),
        }
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
