use axum::{routing::get, Extension, Router};

use crate::db::DbPool;

pub mod posts;
pub mod users;
pub mod auth;

pub fn create_app(pool: DbPool) -> Router {
    Router::new()
        .route("/", get(|| async { "OK" }))
        .nest("/posts", posts::router())
        .nest("/users", users::router())
        .nest("/auth", auth::router())
        .layer(Extension(pool))
}

