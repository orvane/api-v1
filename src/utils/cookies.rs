use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{Duration, Utc};
use cookie::time::OffsetDateTime;

pub fn set_session_cookie(session_token: String, authorized: bool) -> Cookie<'static> {
    let now = Utc::now();

    let expiration_time = if authorized {
        now + Duration::days(30)
    } else {
        now + Duration::hours(12)
    };

    let expiration_time = OffsetDateTime::from_unix_timestamp(expiration_time.timestamp()).unwrap();

    Cookie::build(("session_id", session_token))
        .path("/")
        .same_site(SameSite::Lax)
        .secure(true)
        .http_only(true)
        .expires(expiration_time)
        .build()
}
