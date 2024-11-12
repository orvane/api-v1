use crate::services::email::EmailLayer;
use std::env;

pub fn setup_email_service() -> EmailLayer {
    EmailLayer::new(
        env::var("RESEND_API_KEY").unwrap_or_else(|_| {
            println!("Resend API key error");
            String::new()
        }),
        String::from("blazar.lol"),
    )
}
