use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use sqlx::Error as SqlxError;

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
    let posts = sqlx::query_as!(Post, "SELECT id, user_id, title, body FROM posts")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(posts))
}

async fn get_post(
    Extension(pool): Extension<DbPool>,
    Path(id): Path<i32>,
) -> Result<Json<Post>, StatusCode> {
    let opt = sqlx::query_as!(
        Post,
        "SELECT id, user_id, title, body FROM posts WHERE id = $1",
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match opt {
        Some(post) => Ok(Json(post)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_post(
    Extension(pool): Extension<DbPool>,
    Json(new_post): Json<CreatePost>,
) -> Result<Json<Post>, StatusCode> {
    let post = sqlx::query_as!(
        Post,
        "INSERT INTO posts (user_id, title, body) VALUES ($1, $2, $3) RETURNING id, user_id, title, body",
        new_post.user_id,
        new_post.title,
        new_post.body
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(post))
}

async fn update_post(
    Extension(pool): Extension<DbPool>,
    Path(id): Path<i32>,
    Json(updated_post): Json<UpdatePost>,
) -> Result<Json<Post>, StatusCode> {
    let res = sqlx::query_as!(
        Post,
        "UPDATE posts SET title = $1, body = $2, user_id = $3 WHERE id = $4 RETURNING id, user_id, title, body",
        updated_post.title,
        updated_post.body,
        updated_post.user_id,
        id
    )
    .fetch_one(&pool)
    .await;

    match res {
        Ok(post) => Ok(Json(post)),
        Err(SqlxError::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_post(
    Extension(pool): Extension<DbPool>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query!("DELETE FROM posts WHERE id = $1", id)
        .execute(&pool)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => Ok(Json(serde_json::json!({
            "message": "Post deleted successfully"
        }))),
        Ok(_) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

