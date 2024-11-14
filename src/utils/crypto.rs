use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use data_encoding::BASE32_NOPAD;
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub async fn hash_password(password: String) -> Result<String, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;

    Ok(password_hash.to_string())
}

pub async fn verify_password_hash(
    password: String,
    hash: String,
) -> Result<bool, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let hash = PasswordHash::new(hash.as_str()).unwrap();

    match argon2.verify_password(password.as_bytes(), &hash) {
        Ok(_) => Ok(true),
        Err(e) => Err(e),
    }
}

pub fn hash_string(input: String) -> String {
    let mut hasher = Sha256::new();

    hasher.update(input.as_bytes());

    let result = hasher.finalize();

    result.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub fn verify_string_hash(input: String, hash: String) -> bool {
    let new_hash = hash_string(input);

    hash == new_hash
}

pub fn generate_uuid() -> String {
    let new_uuid = Uuid::new_v4().simple().to_string();

    new_uuid
}

pub fn generate_token() -> String {
    let mut bytes = [0u8; 20];

    rand::thread_rng().fill_bytes(&mut bytes);

    BASE32_NOPAD.encode(&bytes).to_lowercase()
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();

    hasher.update(token);
    hex::encode(hasher.finalize())
}
