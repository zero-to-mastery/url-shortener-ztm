use argon2::{
    Argon2, PasswordHash,
    password_hash::{self, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
type HmacSha256 = Hmac<Sha256>;

pub fn hash_password(pwd: &str, pepper: &str) -> anyhow::Result<Vec<u8>> {
    let salt = SaltString::generate(&mut OsRng);
    let mut mac = HmacSha256::new_from_slice(pepper.as_bytes())?;
    mac.update(pwd.as_bytes());
    let pre = mac.finalize().into_bytes();

    let hash = Argon2::default()
        .hash_password(&pre, &salt)
        .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    Ok(hash.to_string().into_bytes())
}

pub fn verify_password(input_pwd: &str, stored_phc: &[u8], pepper: &str) -> anyhow::Result<bool> {
    let parsed = PasswordHash::new(std::str::from_utf8(stored_phc)?)
        .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    let mut mac = HmacSha256::new_from_slice(pepper.as_bytes())?;
    mac.update(input_pwd.as_bytes());
    let pre = mac.finalize().into_bytes();

    match Argon2::default().verify_password(&pre, &parsed) {
        Ok(_) => Ok(true),
        Err(password_hash::Error::Password) => Ok(false),
        Err(e) => Err(anyhow::Error::msg(e.to_string())),
    }
}
