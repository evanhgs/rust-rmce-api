use axum::{
    Json, Router, 
    extract::{Extension, Path}, 
    http::{StatusCode}, 
    routing::{get, post}
};
use tracing::{info, warn, error};

use crate::{
    db::DbPool,
    models::user::{CreateUser, User},
};

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_users).post(create_user))
        .route("/{id}", get(get_user).delete(delete_user))
        .route("/{user_id}/friends/{friend_id}", post(add_friend))
}

async fn create_user(
    Extension(pool): Extension<DbPool>,
    Json(new_user): Json<CreateUser>,
) -> Result<Json<User>, StatusCode> {
    info!("Création d'un nouvel utilisateur: {} ({})", new_user.username, new_user.email);
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id, username, email"
    )
    .bind(&new_user.username)
    .bind(&new_user.email)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la création de l'utilisateur: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Utilisateur créé avec succès: {} (ID: {})", user.username, user.id);
    Ok(Json(user))
}

async fn get_users(
    Extension(pool): Extension<DbPool>
) -> Result<Json<Vec<User>>, StatusCode> {
    info!("Récupération de tous les utilisateurs");
    let users = sqlx::query_as::<_, User>(
        "SELECT id, username, email FROM users"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la récupération des utilisateurs: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("{} utilisateurs récupérés", users.len());
    Ok(Json(users))
}

async fn get_user(
    Extension(pool): Extension<DbPool>,
    Path(id): Path<i32>,
) -> Result<Json<User>, StatusCode> {
    info!("Récupération de l'utilisateur avec ID: {}", id);
    let opt = sqlx::query_as::<_, User>(
        "SELECT id, username, email FROM users WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la récupération de l'utilisateur {}: {}", id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match opt {
        Some(user) => {
            info!("Utilisateur {} trouvé: {}", id, user.username);
            Ok(Json(user))
        }
        None => {
            warn!("Utilisateur {} non trouvé", id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn delete_user(
    Extension(pool): Extension<DbPool>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Suppression de l'utilisateur ID: {}", id);
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            info!("Utilisateur {} supprimé avec succès", id);
            Ok(Json(serde_json::json!({
                "message": "User deleted successfully"
            })))
        }
        Ok(_) => {
            warn!("Utilisateur {} non trouvé pour la suppression", id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Erreur lors de la suppression de l'utilisateur {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn add_friend(
    Extension(pool): Extension<DbPool>,
    Path((user_id, friend_id)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Ajout de l'ami {} à l'utilisateur {}", friend_id, user_id);

    // Check if both users exist
    let user_exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            error!("Erreur lors de la vérification de l'utilisateur: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let friend_exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(friend_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            error!("Erreur lors de la vérification de l'ami: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !user_exists || !friend_exists {
        warn!("Utilisateur ou ami non trouvé: user={}, friend={}", user_id, friend_id);
        return Err(StatusCode::NOT_FOUND);
    }

    // Add friendship
    let result = sqlx::query(
        "INSERT INTO friendships (user_id, friend_id, status) VALUES ($1, $2, 'pending')
         ON CONFLICT (user_id, friend_id) DO NOTHING"
    )
    .bind(user_id)
    .bind(friend_id)
    .execute(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de l'ajout de l'ami: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() > 0 {
        info!("Ami {} ajouté avec succès à l'utilisateur {}", friend_id, user_id);
        Ok(Json(serde_json::json!({
            "message": "Friend request sent successfully"
        })))
    } else {
        info!("Demande d'ami déjà existante");
        Ok(Json(serde_json::json!({
            "message": "Friend request already exists"
        })))
    }
}