use crate::models::User;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: i32,
    exp: usize,
}

#[allow(dead_code)]
pub fn create_token(user: &User, secret: &str) -> anyhow::Result<String> {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH)?.as_secs();
    let claims = Claims { sub: user.id, exp: (since_epoch + 60 * 60 * 24) as usize };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))?;
    Ok(token)
}

#[allow(dead_code)]
pub fn decode_token(token: &str, secret: &str) -> Result<Claims, JwtError> {
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())?;
    Ok(token_data.claims)
}
