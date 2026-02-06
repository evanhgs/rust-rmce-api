use axum::{
    Json, Router,
    extract::{Extension, Path},
    http::{StatusCode},
    routing::post
};
use tracing::callsite::register;
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
    let login = sqlx::query!(
        User,
        ""
    );
}

async fn register(
    Extension(pool): Extension<DbPool>,
    Json(register_req): Json<Register>,
) -> Result<Json<User>, StatusCode> {
    
}