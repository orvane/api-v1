use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub async fn hash_password(password: String) -> Result<String, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2.hash_password(password.as_bytes(), salt.as_salt())?;

    Ok(password_hash.to_string())
}

pub async fn verify_password(
    password: String,
    hash: String,
) -> Result<bool, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let hash = PasswordHash::new(hash.as_str()).unwrap();

    let password_verifies = argon2.verify_password(password.as_bytes(), &hash);

    println!("{:?}", password_verifies);

    Ok(true)
}

pub fn string_hash(input: String) -> String {
    let mut hasher = Sha256::new();

    hasher.update(input.as_bytes());

    let result = hasher.finalize();

    result.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub fn verify_string_hash(input: String, hash: String) -> bool {
    let new_hash = string_hash(input);

    hash == new_hash
}

pub fn generate_uuid() -> String {
    let new_uuid = Uuid::new_v4().simple().to_string();

    new_uuid
}
