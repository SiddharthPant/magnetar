use anyhow::Result;
use argon2::{
    Argon2, PasswordHasher,
};

/// Hashes password
///
/// # Errors
///
/// This function will return an error if password hash fails.
pub fn hash_password(password: &str) -> Result<String> {
    Ok(Argon2::default()
        .hash_password(password.as_bytes())?.to_string())
}
