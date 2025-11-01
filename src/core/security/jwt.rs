use argon2::password_hash::rand_core::{OsRng, RngCore};
use base64::Engine;
use chrono::{Duration, Utc};
use hmac::Mac;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::security::HmacSha256;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub ver: u32,
    pub exp: i64,
}

#[derive(Clone)]
pub struct JwtKeys {
    enc: EncodingKey,
    dec: DecodingKey,
    validation: Validation,
}

impl JwtKeys {
    pub fn new(secret: &[u8]) -> Self {
        let mut val = Validation::new(Algorithm::HS256);
        val.leeway = 60; // allow 60 seconds of clock skew
        val.validate_exp = true; // ensure exp is validated
        val.validate_nbf = false; // not using nbf

        Self {
            enc: EncodingKey::from_secret(secret),
            dec: DecodingKey::from_secret(secret),
            validation: val,
        }
    }

    pub fn sign(&self, sub: Uuid, ver: u32, ttl: Duration) -> anyhow::Result<String> {
        let claims = Claims {
            sub,
            ver,
            exp: (Utc::now() + ttl).timestamp(),
        };

        Ok(encode(&Header::default(), &claims, &self.enc)?)
    }

    pub fn verify(&self, token: &str) -> anyhow::Result<Claims> {
        let token_data = decode::<Claims>(token, &self.dec, &self.validation)?;

        Ok(token_data.claims)
    }
}

pub fn gen_refresh_token() -> String {
    let mut buf = [0u8; 48];
    OsRng.fill_bytes(&mut buf);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}

pub fn hash_refresh_token(token: &str, pepper: &str) -> anyhow::Result<Vec<u8>> {
    let mac = HmacSha256::new_from_slice(pepper.as_bytes());
    match mac {
        Ok(mut m) => {
            m.update(token.as_bytes());
            Ok(m.finalize().into_bytes().to_vec())
        }
        Err(e) => Err(anyhow::anyhow!("HMAC error: {}", e)),
    }
}
