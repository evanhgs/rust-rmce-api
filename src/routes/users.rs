use axum::{
    Json, Router, 
    extract::{Extension, Path}, 
    http::{StatusCode}, 
    routing::get
};

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
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id, username, email",
        new_user.username,
        new_user.email
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}

async fn get_users(
    Extension(pool): Extension<DbPool>
) -> Result<Json<Vec<User>>, StatusCode> {
    let users = sqlx::query_as!(
        User, 
        "SELECT id, username, email FROM users")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(users))
}

async fn get_user(
    Extension(pool): Extension<DbPool>,
    Path(id): Path<i32>,
) -> Result<Json<User>, StatusCode> {
    let opt = sqlx::query_as!(
        User,
        "SELECT id, username, email FROM users WHERE id = $1",
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match opt {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn delete_user(
    Extension(pool): Extension<DbPool>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(&pool)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => Ok(Json(serde_json::json!({
            "message": "User deleted successfully"
        }))),
        Ok(_) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}