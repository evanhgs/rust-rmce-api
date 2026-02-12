use axum::{
    Json, Router,
    extract::Extension,
    http::StatusCode,
    routing::post
};
use bcrypt::{hash, verify, DEFAULT_COST};
use tracing::{info, warn, error};
use crate::{
    db::DbPool,
    models::auth::{Login, Register},
    models::user::{User}
};

pub fn router() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}

async fn login(
    Extension(pool): Extension<DbPool>,
    Json(login_req): Json<Login>,
) -> Result<Json<String>, StatusCode> {
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
                    info!("Connexion réussie pour l'utilisateur: {}", u.username);
                    Ok(Json(format!("Login successful for user: {}", u.username)))
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