use std::task::{Context, Poll};

use axum::{body::Body, http::Request, response::Response};
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Uuid,
    Surreal,
};
use tower::{Layer, Service};
use validator::Validate;

use crate::errors::auth::email_verification;

#[derive(Clone)]
pub struct DatabaseQuery<'a> {
    #[allow(dead_code)]
    db: &'a Surreal<Client>,
    pub user: UserQuery<'a>,
    pub email_verification: EmailVerificationQuery<'a>,
}

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
    fn new(db: &'a Surreal<Client>) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        email: String,
        password_hash: String,
    ) -> Result<Option<User>, surrealdb::Error> {
        let new_user = User::new(String::from(email.clone()), password_hash);

        let user: Option<User> = self
            .db
            .create(("user", email.clone()))
            .content(new_user)
            .await?;

        Ok(user)
    }

    // TODO: Instead of using string referance use normal string and clone the input in the
    // implemenation
    pub async fn check_if_exists(&self, email: &String) -> Result<bool, surrealdb::Error> {
        let user: Option<User> = self.db.select(("user", email)).await?;

        match user {
            Some(_) => Ok(false),
            None => Ok(true),
        }
    }

    pub async fn get_all(&self) -> Result<Vec<User>, surrealdb::Error> {
        self.db.select("user").await
    pub async fn verify_user(&self, user_id: String) -> Result<(), surrealdb::Error> {
        let query = r#"
            UPDATE user
            SET email_verified = true
            WHERE id = $user_id
        "#;

        let result: surrealdb::Response = self.db.query(query).bind(("user_id", user_id)).await?;

        // TODO: Return an error if it affects no rows

        Ok(())
    }
}

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
    fn new(db: &'a Surreal<Client>) -> Self {
        Self { db }
    }

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

        let result: surrealdb::Response = self
            .db
            .query(query)
            .bind(("email_verification_id", email_verification_id))
            .await?;

        // TODO: Return an error if it affects no rows

        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct DatabaseLayer {
    pub username: String,
    password: String,
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub db: Surreal<Client>,
}

impl DatabaseLayer {
    pub async fn new(
        username: String,
        password: String,
        url: String,
        namespace: String,
        database: String,
    ) -> Result<Self, surrealdb::Error> {
        let db = Surreal::new::<Ws>(url.clone()).await?;

        db.signin(Root {
            username: username.as_str(),
            password: password.as_str(),
        })
        .await?;

        db.use_ns(namespace.clone())
            .use_db(database.clone())
            .await?;

        Ok(Self {
            username,
            password,
            url,
            namespace,
            database,
            db,
        })
    }

    pub fn query(&self) -> DatabaseQuery {
        DatabaseQuery {
            db: &self.db,
            user: UserQuery::new(&self.db),
            email_verification: EmailVerificationQuery::new(&self.db),
        }
    }
}
impl<S> Layer<S> for DatabaseLayer {
    type Service = DatabaseService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        DatabaseService {
            inner,
            username: self.username.clone(),
            password: self.username.clone(),
            url: self.username.clone(),
            namespace: self.username.clone(),
            database: self.username.clone(),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct DatabaseService<S> {
    pub inner: S,
    pub username: String,
    password: String,
    pub url: String,
    pub namespace: String,
    pub database: String,
}

impl<S> Service<Request<Body>> for DatabaseService<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let future = self.inner.call(request);
        Box::pin(async move {
            let response: Response = future.await?;
            Ok(response)
        })
    }
}
