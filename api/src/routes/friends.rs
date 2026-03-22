use axum::{
    Json, Router,
    extract::{Extension, Path},
    http::StatusCode,
    routing::{get, post, put}
};
use tracing::{info, warn, error};
use shared::jwt::Claims;

use crate::{
    db::DbPool,
    models::friendship::{Friendship, FriendInfo, PendingRequest},
};

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_friends))
        .route("/add/{username}", post(add_friend))
        .route("/accept/{friendship_id}", put(accept_friend))
        .route("/reject/{friendship_id}", put(reject_friend))
        .route("/pending", get(get_pending_requests))
}

async fn get_friends(
    Extension(pool): Extension<DbPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<FriendInfo>>, StatusCode> {
    let user_id = claims.user_id;
    
    info!("Récupération des amis de l'utilisateur {}", user_id);
    
    let friends = sqlx::query_as::<_, FriendInfo>(
        "SELECT u.id, u.username, u.email, f.status
         FROM friendships f
         JOIN users u ON u.id = f.friend_id
         WHERE f.user_id = $1 AND f.status = 'accepted'
         ORDER BY u.username"
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la récupération des amis: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("{} amis récupérés pour l'utilisateur {}", friends.len(), user_id);
    Ok(Json(friends))
}

async fn add_friend(
    Extension(pool): Extension<DbPool>,
    Extension(claims): Extension<Claims>,
    Path(username): Path<String>,
) -> Result<Json<Friendship>, StatusCode> {
    let user_id = claims.user_id;

    info!("Ajout de l'ami '{}' à l'utilisateur {}", username, user_id);

    let friend_id = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM users WHERE username = $1"
    )
    .bind(&username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la recherche de l'utilisateur '{}': {}", username, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or_else(|| {
        warn!("Utilisateur '{}' non trouvé", username);
        StatusCode::NOT_FOUND
    })?;

    let friendship = sqlx::query_as::<_, Friendship>(
        "INSERT INTO friendships (user_id, friend_id, status)
         VALUES ($1, $2, 'pending')
         ON CONFLICT (user_id, friend_id) DO UPDATE SET status = 'pending'
         RETURNING id, user_id, friend_id, status, created_at"
    )
    .bind(user_id)
    .bind(friend_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de l'ajout de l'ami: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Demande d'ami envoyée avec succès à '{}' (id={})", username, friend_id);
    Ok(Json(friendship))
}

async fn accept_friend(
    Extension(pool): Extension<DbPool>,
    Path(friendship_id): Path<i32>,
) -> Result<Json<Friendship>, StatusCode> {
    info!("Acceptation de la demande d'ami {}", friendship_id);
    
    let friendship = sqlx::query_as::<_, Friendship>(
        "UPDATE friendships
         SET status = 'accepted'
         WHERE id = $1
         RETURNING id, user_id, friend_id, status, created_at"
    )
    .bind(friendship_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de l'acceptation de la demande d'ami: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match friendship {
        Some(f) => {
            info!("Demande d'ami {} acceptée", friendship_id);
            Ok(Json(f))
        }
        None => {
            warn!("Demande d'ami {} non trouvée", friendship_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn reject_friend(
    Extension(pool): Extension<DbPool>,
    Path(friendship_id): Path<i32>,
) -> Result<Json<Friendship>, StatusCode> {
    info!("Rejet de la demande d'ami {}", friendship_id);
    
    let friendship = sqlx::query_as::<_, Friendship>(
        "UPDATE friendships
         SET status = 'rejected'
         WHERE id = $1
         RETURNING id, user_id, friend_id, status, created_at"
    )
    .bind(friendship_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors du rejet de la demande d'ami: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match friendship {
        Some(f) => {
            info!("Demande d'ami {} rejetée", friendship_id);
            Ok(Json(f))
        }
        None => {
            warn!("Demande d'ami {} non trouvée", friendship_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn get_pending_requests(
    Extension(pool): Extension<DbPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<PendingRequest>>, StatusCode> {
    let user_id = claims.user_id;

    info!("Récupération des demandes d'ami en attente pour l'utilisateur {}", user_id);

    let requests = sqlx::query_as::<_, PendingRequest>(
        "SELECT f.id AS friendship_id, u.id, u.username, u.email, f.status
         FROM friendships f
         JOIN users u ON u.id = f.user_id
         WHERE f.friend_id = $1 AND f.status = 'pending'
         ORDER BY f.created_at DESC"
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la récupération des demandes d'ami: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("{} demandes d'ami en attente", requests.len());
    Ok(Json(requests))
}

