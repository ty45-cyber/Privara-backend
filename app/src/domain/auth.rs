use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub org: String, // org id
    pub role: String,
    pub exp: i64,
}

fn secret() -> String {
    std::env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

pub fn issue_token(user_id: Uuid, org_id: Uuid, role: &str) -> AppResult<String> {
    let exp = (Utc::now() + Duration::hours(12)).timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        org: org_id.to_string(),
        role: role.to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret().as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.into()))
}

pub fn verify_token(token: &str) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret(secret().as_bytes())),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|_| AppError::Unauthorized)
}