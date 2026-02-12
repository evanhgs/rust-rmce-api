use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use sqlx::Error as SqlxError;
use tracing::{info, warn, error};

use crate::{
    db::DbPool,
    models::post::{CreatePost, Post, UpdatePost},
};

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_posts).post(create_post))
        .route("/{id}",get(get_post).put(update_post).delete(delete_post))
}

async fn get_posts(Extension(pool): Extension<DbPool>) -> Result<Json<Vec<Post>>, StatusCode> {
    info!("Récupération de tous les posts");
    let posts = sqlx::query_as::<_, Post>("SELECT id, user_id, title, body FROM posts")
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!("Erreur lors de la récupération des posts: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("{} posts récupérés", posts.len());
    Ok(Json(posts))
}

async fn get_post(
    Extension(pool): Extension<DbPool>,
    Path(id): Path<i32>,
) -> Result<Json<Post>, StatusCode> {
    info!("Récupération du post avec ID: {}", id);
    let opt = sqlx::query_as::<_, Post>(
        "SELECT id, user_id, title, body FROM posts WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la récupération du post {}: {}", id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match opt {
        Some(post) => {
            info!("Post {} trouvé: {}", id, post.title);
            Ok(Json(post))
        }
        None => {
            warn!("Post {} non trouvé", id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn create_post(
    Extension(pool): Extension<DbPool>,
    Json(new_post): Json<CreatePost>,
) -> Result<Json<Post>, StatusCode> {
    info!("Création d'un nouveau post: '{}' (user_id: {:?})", new_post.title, new_post.user_id);
    let post = sqlx::query_as::<_, Post>(
        "INSERT INTO posts (user_id, title, body) VALUES ($1, $2, $3) RETURNING id, user_id, title, body"
    )
    .bind(new_post.user_id)
    .bind(&new_post.title)
    .bind(&new_post.body)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la création du post: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Post créé avec succès: ID {} - '{}'", post.id, post.title);
    Ok(Json(post))
}

async fn update_post(
    Extension(pool): Extension<DbPool>,
    Path(id): Path<i32>,
    Json(updated_post): Json<UpdatePost>,
) -> Result<Json<Post>, StatusCode> {
    info!("Mise à jour du post ID: {} - '{}'", id, updated_post.title);
    let res = sqlx::query_as::<_, Post>(
        "UPDATE posts SET title = $1, body = $2, user_id = $3 WHERE id = $4 RETURNING id, user_id, title, body"
    )
    .bind(&updated_post.title)
    .bind(&updated_post.body)
    .bind(updated_post.user_id)
    .bind(id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(post) => {
            info!("Post {} mis à jour avec succès", id);
            Ok(Json(post))
        }
        Err(SqlxError::RowNotFound) => {
            warn!("Post {} non trouvé pour la mise à jour", id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Erreur lors de la mise à jour du post {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_post(
    Extension(pool): Extension<DbPool>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Suppression du post ID: {}", id);
    let result = sqlx::query("DELETE FROM posts WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            info!("Post {} supprimé avec succès", id);
            Ok(Json(serde_json::json!({
                "message": "Post deleted successfully"
            })))
        }
        Ok(_) => {
            warn!("Post {} non trouvé pour la suppression", id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Erreur lors de la suppression du post {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

