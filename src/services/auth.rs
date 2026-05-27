use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm };
use serde::{Deserialize, Serialize};
use chrono::Utc;
use crate::errors::{AppError, AppResult};

const TOKEN_TTL_SECS: i64 = 86_400; // 24h

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub role: String,
    pub exp: usize,
}

/// Issues a signed JWT. 0(1)
pub fn issue_token(user_id: &str, role: &str, secret: &str) -> AppResult<String> {
    let exp = (Utc::now().timestamp() + TOKEN_TTL_SECS) as usize;
    let claims = Claims { sub: user_id.into(), role: role.into(), exp};
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Token encode: {e}")))
}

/// Verifies and decodes a JWT. O(1)
pub fn verify_token(token: &str, secret: &str) -> AppResult<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}
