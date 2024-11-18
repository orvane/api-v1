pub mod email_verification;
pub mod password_reset_request;
pub mod session;
pub mod user;

use std::task::{Context, Poll};

use axum::{body::Body, extract::Request, response::Response};
use futures_util::future::BoxFuture;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct DatabaseQuery<'a> {
    #[allow(dead_code)]
    db: &'a Surreal<Client>,
    pub user: user::UserQuery<'a>,
    pub email_verification: email_verification::EmailVerificationQuery<'a>,
    pub password_reset_request: password_reset_request::PasswordResetRequestQuery<'a>,
    pub session: session::SessionQuery<'a>,
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

    pub async fn initialize_schemas(&self, schemas: Vec<&str>) -> Result<(), surrealdb::Error> {
        for schema_query in schemas {
            self.db.query(schema_query).await?;
        }

        Ok(())
    }

    pub fn query(&self) -> DatabaseQuery {
        DatabaseQuery {
            db: &self.db,
            user: user::UserQuery::new(&self.db),
            email_verification: email_verification::EmailVerificationQuery::new(&self.db),
            password_reset_request: password_reset_request::PasswordResetRequestQuery::new(
                &self.db,
            ),
            session: session::SessionQuery::new(&self.db),
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
