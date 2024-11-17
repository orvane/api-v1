use axum::{body::Body, http::Request, response::Response};
use futures_util::future::BoxFuture;
use resend_rs::{types::CreateEmailBaseOptions, Resend};
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct EmailLayer {
    api_key: String,
    pub domain: String,
}

impl EmailLayer {
    pub fn new(api_key: String, domain: String) -> Self {
        Self { api_key, domain }
    }

    pub async fn send_email_verification(
        &self,
        to: String,
        activation_code: String,
        token_hash: String,
    ) -> Result<(), resend_rs::Error> {
        let resend = Resend::new(&self.api_key);

        let from = format!("Orvane <noreply@{}>", &self.domain);
        let to = [to];
        let subject = "Orvane - Account Activation";

        let email = CreateEmailBaseOptions::new(from, to, subject).with_html(
            format!(
                "
                <strong>{}</strong>
                <span>{}</span>
            ",
                activation_code, token_hash
            )
            .as_str(),
        );

        let _email = resend.emails.send(email).await?;

        Ok(())
    }

    pub async fn send_email_verification_confirmation(
        &self,
        to: String,
    ) -> Result<(), resend_rs::Error> {
        let resend = Resend::new(&self.api_key);

        let from = format!("Orvane <noreply@{}>", &self.domain);
        let to = [to];
        let subject = "Orvane - Email verified successfully";

        let email = CreateEmailBaseOptions::new(from, to, subject).with_html("Email verified!");

        let _email = resend.emails.send(email).await?;

        Ok(())
    }

    pub async fn send_password_reset(
        &self,
        to: String,
        password_reset_request_id: String,
    ) -> Result<(), resend_rs::Error> {
        let resend = Resend::new(&self.api_key);

        let from = format!("Orvane <noreply@{}>", &self.domain);
        let to = [to];
        let subject = "Orvane - Password Reset";

        let password_reset_url = format!(
            "https://{}/auth/password-reset/{}",
            &self.domain, password_reset_request_id
        );

        let email = CreateEmailBaseOptions::new(from, to, subject)
            .with_html(format!("<a href=\"{}\">Reset</strong>", password_reset_url).as_str());

        let _email = resend.emails.send(email).await?;

        Ok(())
    }
}

impl<S> Layer<S> for EmailLayer {
    type Service = EmailService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        EmailService {
            inner,
            api_key: self.api_key.clone(),
            domain: self.domain.clone(),
        }
    }
}

#[derive(Clone)]
pub struct EmailService<S> {
    pub inner: S,
    pub domain: String,
    api_key: String,
}

impl<S> Service<Request<Body>> for EmailService<S>
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
