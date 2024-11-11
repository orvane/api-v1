use derive_more::Display;

#[derive(Debug, Display)]
pub enum CommonError {
    Validation(validator::ValidationErrors),
    Database(surrealdb::Error),
    Email(resend_rs::Error),
    Hashing(argon2::password_hash::Error),
}
