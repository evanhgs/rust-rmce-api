use axum::{
    Json, Router,
    extract::Extension,
    http::StatusCode,
    routing::post
};
use bcrypt::{hash, verify, DEFAULT_COST};
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;

use crate::{
    db::DbPool,
    models::auth::{Login, Register},
    models::user::{User}
};


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

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

pub fn create_jwt(user_id: i32, username: String, email: String) -> Result<String, jsonwebtoken::errors::Error> {
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

pub fn router() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}

async fn login(
    Extension(pool): Extension<DbPool>,
    Json(login_req): Json<Login>,
) -> Result<Json<LoginResponse>, StatusCode> {
    info!("Tentative de connexion pour l'email: {}", login_req.email);
    
    let user = sqlx::query_as::<_, UserWithPassword>(
        "SELECT id, username, email, password FROM users WHERE email = $1"
    )
    .bind(&login_req.email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la requête de connexion: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match user {
        Some(u) => {
            info!("Utilisateur trouvé: {}", u.username);
            match verify(login_req.password, &u.password) {
                Ok(true) => {
                    let token = create_jwt(u.id, u.username.clone(), u.email.clone())
                        .map_err(|e| {
                            error!("Erreur lors de la création du token JWT: {}", e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?;

                    info!("Connexion réussie pour l'utilisateur: {}", u.username);
                    Ok(Json(LoginResponse {
                        token,
                        user: User {
                            id: u.id,
                            username: u.username,
                            email: u.email,
                        },
                    }))
                }
                Ok(false) => {
                    warn!("Mot de passe incorrect pour l'utilisateur: {}", u.username);
                    Err(StatusCode::UNAUTHORIZED)
                }
                Err(e) => {
                    error!("Erreur lors de la vérification du mot de passe: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        None => {
            warn!("Tentative de connexion avec un email inexistant: {}", login_req.email);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

async fn register(
    Extension(pool): Extension<DbPool>,
    Json(register_req): Json<Register>,
) -> Result<Json<User>, StatusCode> {
    info!("Tentative d'enregistrement pour l'utilisateur: {} ({})", register_req.username, register_req.email);
    
    let existing = sqlx::query_as::<_, UserWithPassword>(
        "SELECT id, username, email, password FROM users WHERE email = $1 OR username = $2"
    )
    .bind(&register_req.email)
    .bind(&register_req.username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la vérification de l'utilisateur existant: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(existing_user) = existing {
        warn!("Tentative d'enregistrement avec un email ou username existant: {} / {}", existing_user.email, existing_user.username);
        return Err(StatusCode::CONFLICT);
    }

    info!("Hachage du mot de passe pour l'utilisateur: {}", register_req.username);
    let hashed_password = hash(register_req.password, DEFAULT_COST)
        .map_err(|e| {
            error!("Erreur lors du hachage du mot de passe: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Création de l'utilisateur dans la base de données: {}", register_req.username);
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email, password) VALUES ($1, $2, $3) RETURNING id, username, email"
    )
    .bind(&register_req.username)
    .bind(&register_req.email)
    .bind(&hashed_password)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la création de l'utilisateur: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Utilisateur créé avec succès: {} (ID: {})", user.username, user.id);
    Ok(Json(user))
}

// Structure interne pour les requêtes avec mot de passe
#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct UserWithPassword {
    id: i32,
    username: String,
    email: String,
    password: String,
}