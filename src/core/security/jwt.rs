use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
        let mut val = Validation::default();
        val.algorithms = vec![Algorithm::HS256];
        val.leeway = 60;

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
        let claims = token_data.claims;

        Ok(claims)
    }
}
