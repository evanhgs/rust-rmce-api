use axum::{routing::get, Extension, Router};

use crate::db::DbPool;

pub mod posts;
pub mod users;

pub fn create_app(pool: DbPool) -> Router {
    Router::new()
        .route("/", get(|| async { "OK" }))
        .nest("/posts", posts::router())
        .nest("/users", users::router())
        .layer(Extension(pool))
}

