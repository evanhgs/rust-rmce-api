use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::error;

lazy_static! {
    static ref JWT_SECRET: Vec<u8> = std::env::var("JWT_SECRET")
        .map(|s| s.into_bytes())
        .unwrap_or_else(|_| {
            error!("JWT_SECRET n'est pas définie dans les variables d'environnement");
            error!("Créez un fichier .env avec: JWT_SECRET=your_jwt_secret_key");
            panic!("JWT_SECRET must be set");
        });
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub exp: u64,
    pub iat: u64,
}

pub fn create_jwt(
    user_id: i32,
    username: String,
    email: String,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = Claims {
        user_id,
        username,
        email,
        exp: now + 86400 * 7, // 7 jours
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&JWT_SECRET),
    )
}

pub fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(&JWT_SECRET),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
