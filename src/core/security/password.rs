use std::ops::Deref;

use anyhow::{Result, anyhow};
use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use rand::{TryRngCore, rngs::OsRng as ROSrnd};
use secrecy::{ExposeSecret, SecretString};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;
use zeroize::{Zeroize, Zeroizing};
use zxcvbn::{Score, zxcvbn};

const MIN_PW_CHARS: usize = 10;
const MAX_PW_BYTES: usize = 128;
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789";
const CODE_LEN: usize = 8;

fn is_disallowed_format_char(c: char) -> bool {
    matches!(
        c,
        '\u{200B}' // ZERO WIDTH SPACE
            | '\u{200C}' // ZERO WIDTH NON-JOINER
            | '\u{200D}' // ZERO WIDTH JOINER
            | '\u{200E}' // LEFT-TO-RIGHT MARK
            | '\u{200F}' // RIGHT-TO-LEFT MARK
            | '\u{202A}' // LEFT-TO-RIGHT EMBEDDING
            | '\u{202B}' // RIGHT-TO-LEFT EMBEDDING
            | '\u{202C}' // POP DIRECTIONAL FORMATTING
            | '\u{202D}' // LEFT-TO-RIGHT OVERRIDE
            | '\u{202E}' // RIGHT-TO-LEFT OVERRIDE
            | '\u{2060}' // WORD JOINER
            | '\u{2066}' // LEFT-TO-RIGHT ISOLATE
            | '\u{2067}' // RIGHT-TO-LEFT ISOLATE
            | '\u{2068}' // FIRST STRONG ISOLATE
            | '\u{2069}' // POP DIRECTIONAL ISOLATE
            | '\u{FEFF}' // ZERO WIDTH NO-BREAK SPACE
    )
}

fn normalize(pw: &str) -> Result<String> {
    let norm: String = pw.nfc().collect();

    anyhow::ensure!(
        !norm.chars().any(|c| c.is_control()),
        "password contains disallowed control characters"
    );

    anyhow::ensure!(
        !norm.chars().any(is_disallowed_format_char),
        "password contains disallowed invisible characters"
    );

    let bytes = norm.as_bytes();
    anyhow::ensure!(bytes.len() <= MAX_PW_BYTES, "password too long");

    Ok(norm)
}

pub struct NormalizedPassword(Zeroizing<String>);

impl TryFrom<&str> for NormalizedPassword {
    type Error = anyhow::Error;
    fn try_from(pw: &str) -> Result<Self> {
        let norm = normalize(pw)?;
        Ok(NormalizedPassword(Zeroizing::new(norm)))
    }
}

impl TryFrom<&SecretString> for NormalizedPassword {
    type Error = anyhow::Error;
    fn try_from(pw: &SecretString) -> Result<Self> {
        let norm = normalize(pw.expose_secret())?;
        Ok(NormalizedPassword(Zeroizing::new(norm)))
    }
}

impl Deref for NormalizedPassword {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
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

pub fn validate_policy(norm: &NormalizedPassword) -> Result<()> {
    let char_count = norm.graphemes(true).count();
    anyhow::ensure!(
        char_count >= MIN_PW_CHARS,
        "password too short (by characters)"
    );

    let estimate = zxcvbn(norm, &[]);
    anyhow::ensure!(estimate.score() >= Score::Four, "password too weak");

    Ok(())
}

pub fn hash_password(norm: &NormalizedPassword, pepper: &str) -> Result<Vec<u8>> {
    hash_with_argon2(norm.as_bytes(), pepper)
}

pub fn verify_password(pwd: &SecretString, stored_phc: &[u8], pepper: &str) -> Result<bool> {
    let norm = NormalizedPassword::try_from(pwd)?;

    verify_with_argon2(norm.as_bytes(), stored_phc, pepper)
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
    let mut rng = ROSrnd;
    (0..CODE_LEN)
        .map(|_| {
            let idx = (rng.try_next_u32().expect("OS RNG failure") as usize) % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    const TEST_PEPPER: &str = "test-pepper-secret";

    #[test]
    fn test_normalized_password_deref() {
        let norm = NormalizedPassword::try_from("TestPassword").unwrap();
        let s: &str = &norm;
        assert_eq!(s, "TestPassword");
    }

    #[test]
    fn test_normalize_nfc() {
        let pw1 = "café";
        let pw2 = "cafe\u{0301}";
        assert_ne!(pw1, pw2);

        println!("pw1: {}, pw2: {}", pw1, pw2);

        let norm1: String = pw1.nfc().collect();
        let norm2: String = pw2.nfc().collect();
        assert_eq!(norm1, norm2);
    }

    #[test]
    fn test_normalize_with_control_chars() {
        let result = normalize("password\u{0000}test");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("control characters")
        );
    }

    #[test]
    fn test_normalize_with_invisible_chars() {
        let result = normalize("password\u{200B}test");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invisible characters")
        );
    }

    #[test]
    fn test_normalize_too_long() {
        let long_pw = "a".repeat(MAX_PW_BYTES + 1);
        let result = normalize(&long_pw);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[test]
    fn test_normalized_password_from_str() {
        let result = NormalizedPassword::try_from("ValidPassword123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_normalized_password_from_secret_string() {
        let secret = SecretString::new("ValidPassword123".into());
        let result = NormalizedPassword::try_from(&secret);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_verification_code_valid() {
        let result = validate_verification_code("ABCD1234");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"ABCD1234");
    }

    #[test]
    fn test_validate_verification_code_invalid_chars() {
        let result = validate_verification_code("ABC@1234");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_verification_code_wrong_length() {
        let result = validate_verification_code("ABC123");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_verification_code_empty() {
        let result = validate_verification_code("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_policy_too_short() {
        let norm = NormalizedPassword::try_from("short").unwrap();
        let policy_result = validate_policy(&norm);
        assert!(policy_result.is_err());

        // Test multi-byte characters (CJK, Thai, Arabic, etc.)
        // MIN_PW_CHARS is 10. We test 8 and 9 visible characters.
        let short_multibyte_passwords = [
            // Chinese (8 chars)
            "一二三四五六七八",
            // Chinese (9 chars)
            "一二三四五六七八九",
            // Japanese (8 chars)
            "あいうえおかきく",
            // Japanese (9 chars)
            "あいうえおかきくけ",
            // Korean (8 chars)
            "가나다라마바사아",
            // Korean (9 chars)
            "가나다라마바사아자",
            // Thai (8 chars)
            "กขฃคฅฆงจ",
            // Thai (9 chars)
            "กขฃคฅฆงจฉ",
            // Arabic (9 chars)
            "أبجدهوزحط",
            "ăâđêôơưá",
            // Vietnamese (9 chars)
            "ăâđêôơưáà",
        ];

        for pw in short_multibyte_passwords {
            let norm = NormalizedPassword::try_from(pw).unwrap();
            let result = validate_policy(&norm);
            assert!(
                result.is_err(),
                "password '{}' (len: {}) should be too short",
                pw,
                pw.chars().count()
            );
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("password too short")
            );
        }
    }

    #[test]
    fn test_validate_policy_too_weak() {
        let weak_passwords = [
            "一二三四五六七八九十",
            "あいうえおかきくけこ",
            "가나다라마바사아자차",
            "กขฃคฅฆงจฉชซ",
            "1234567890",
            "qwertyuiop",
            "password1!",
            "1111111111",
            "letmein1234",
            "Qawsed123!",
            "qawsed123321A!",
            "michael!2024",
            "sarah1!2024",
            "bob123!2024",
            "alice#1secure",
            "john-doe99x",
            "lee*12secure",
        ];

        for &pw in &weak_passwords {
            let norm = NormalizedPassword::try_from(pw).unwrap();
            let result = validate_policy(&norm);
            assert!(
                result.is_err(),
                "password '{}' should be considered too weak",
                pw
            );
        }
    }

    #[test]
    fn test_validate_policy_strong_password() {
        let strong_passwords = [
            "MyStr0ng!P@ssw0rd2024",
            "Truly$ecureP4ssword2024!",
            "S0l1d-P@ssword_#2024",
            // Chinese + ASCII mix
            "安全Passw0rd!2024中文",
            "很强的密码Secure!2024混合字符",
            // Japanese + ASCII mix
            "安全パスワード!2024Strong",
            "とても安全なP@ssw0rd2024!",
            // Korean + ASCII mix
            "안전비밀번호!2024Strong",
            "매우강력한P@ssw0rd2024!",
            // Arabic + ASCII mix
            "كلمهالسرSecure!2024",
            "أمنقويP@ss2024مزيج",
        ];

        for &pw in &strong_passwords {
            let norm = NormalizedPassword::try_from(pw).unwrap();
            let result = validate_policy(&norm);
            assert!(result.is_ok(), "password '{}' should pass the policy", pw);
        }
    }

    #[test]
    fn test_hash_and_verify_password() {
        let password = SecretString::new("MySecurePassword123!".into());
        let norm = NormalizedPassword::try_from(&password).unwrap();

        let hash = hash_password(&norm, TEST_PEPPER).unwrap();
        let verify_result = verify_password(&password, &hash, TEST_PEPPER).unwrap();

        assert!(verify_result);
    }

    #[test]
    fn test_verify_password_wrong_password() {
        let password = SecretString::new("MySecurePassword123!".into());
        let wrong_password = SecretString::new("WrongPassword123!".into());
        let norm = NormalizedPassword::try_from(&password).unwrap();

        let hash = hash_password(&norm, TEST_PEPPER).unwrap();
        let verify_result = verify_password(&wrong_password, &hash, TEST_PEPPER).unwrap();

        assert!(!verify_result);
    }

    #[test]
    fn test_hash_and_verify_verification_code() {
        let code = "ABCD1234";
        let hash = hash_verification_code(code, TEST_PEPPER).unwrap();
        let verify_result = verify_verification_code(code, &hash, TEST_PEPPER).unwrap();

        assert!(verify_result);
    }

    #[test]
    fn test_verify_verification_code_wrong_code() {
        let code = "ABCD1234";
        let wrong_code = "WXYZ5678";
        let hash = hash_verification_code(code, TEST_PEPPER).unwrap();
        let verify_result = verify_verification_code(wrong_code, &hash, TEST_PEPPER).unwrap();

        assert!(!verify_result);
    }

    #[test]
    fn test_generate_verification_code() {
        let code = generate_verification_code();
        assert_eq!(code.len(), CODE_LEN);
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_generate_verification_code_uniqueness() {
        let code1 = generate_verification_code();
        let code2 = generate_verification_code();
        // Extremely unlikely to be equal
        assert_ne!(code1, code2);
    }
}
