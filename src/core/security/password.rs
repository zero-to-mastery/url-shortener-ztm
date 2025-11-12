use anyhow::{Result, anyhow};
use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use rand::RngCore;
use unicode_normalization::UnicodeNormalization;
use zeroize::Zeroize;

const MIN_PW_BYTES: usize = 10;
const MAX_PW_BYTES: usize = 128;
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789";
const CODE_LEN: usize = 8;

fn normalize_and_validate(pw: &str) -> anyhow::Result<Vec<u8>> {
    let norm: String = pw.nfc().collect();

    anyhow::ensure!(
        !norm
            .chars()
            .any(|c| c == '\0' || (c.is_control() && c != ' ')),
        "password contains disallowed control characters"
    );

    let b = norm.as_bytes();
    anyhow::ensure!(b.len() >= MIN_PW_BYTES, "password too short");
    anyhow::ensure!(b.len() <= MAX_PW_BYTES, "password too long");

    Ok(b.to_vec())
}

fn validate_verification_code(code: &str) -> anyhow::Result<Vec<u8>> {
    anyhow::ensure!(
        code.chars().all(|c| c.is_ascii_alphanumeric()),
        "code must contain only letters and numbers"
    );

    anyhow::ensure!(
        !code.is_empty() && code.len() == CODE_LEN,
        "code length invalid"
    );

    Ok(code.as_bytes().to_vec())
}

fn build_argon2(pepper: &[u8]) -> Result<Argon2<'_>> {
    let params =
        Params::new(16 * 1024, 3, 1, None).map_err(|e| anyhow!("invalid argon2 params: {e}"))?;
    Argon2::new_with_secret(pepper, Algorithm::Argon2id, Version::V0x13, params)
        .map_err(|e| anyhow!("argon2 init failed: {e}"))
}

fn hash_with_argon2(material: &[u8], pepper: &str) -> Result<Vec<u8>> {
    let salt = SaltString::generate(&mut OsRng);
    let hasher = build_argon2(pepper.as_bytes())?;
    let phc = hasher
        .hash_password(material, &salt)
        .map_err(|e| anyhow!("argon2 hash error: {e}"))?
        .to_string()
        .into_bytes();

    Ok(phc)
}

fn verify_with_argon2(material: &[u8], stored_phc: &[u8], pepper: &str) -> Result<bool> {
    let parsed = PasswordHash::new(std::str::from_utf8(stored_phc)?)
        .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    let hasher = build_argon2(pepper.as_bytes())?;
    let ok = hasher.verify_password(material, &parsed).is_ok();

    Ok(ok)
}

pub fn hash_password(pwd: &str, pepper: &str) -> Result<Vec<u8>> {
    let mut material = normalize_and_validate(pwd)?;
    let result = hash_with_argon2(&material, pepper);
    material.zeroize();
    result
}

pub fn verify_password(pwd: &str, stored_phc: &[u8], pepper: &str) -> Result<bool> {
    let mut material = normalize_and_validate(pwd)?;
    let result = verify_with_argon2(&material, stored_phc, pepper);
    material.zeroize();
    result
}

pub fn hash_verification_code(code: &str, pepper: &str) -> Result<Vec<u8>> {
    let mut material = validate_verification_code(code)?;
    let result = hash_with_argon2(&material, pepper);
    material.zeroize();
    result
}

pub fn verify_verification_code(code: &str, stored_phc: &[u8], pepper: &str) -> Result<bool> {
    let mut material = validate_verification_code(code)?;
    let result = verify_with_argon2(&material, stored_phc, pepper);
    material.zeroize();
    result
}

pub fn generate_verification_code() -> String {
    let mut rng = rand::rng();
    let code: String = (0..CODE_LEN)
        .map(|_| {
            let idx = (rng.next_u32() as usize) % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect();
    code
}
