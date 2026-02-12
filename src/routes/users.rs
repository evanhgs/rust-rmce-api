use axum::{
    Json, Router, 
    extract::{Extension, Path}, 
    http::{StatusCode}, 
    routing::get
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