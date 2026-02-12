use axum::{routing::get, Extension, Router};
use tower_http::trace::TraceLayer;
use tracing::Level;

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
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::span!(
                        Level::INFO,
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_request(|request: &axum::http::Request<_>, _span: &tracing::Span| {
                    tracing::info!(
                        "→ {} {}",
                        request.method(),
                        request.uri()
                    );
                })
                .on_response(|response: &axum::http::Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::info!(
                        "← {} {}ms",
                        response.status(),
                        latency.as_millis()
                    );
                })
                .on_failure(|error: tower_http::classify::ServerErrorsFailureClass, latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::error!(
                        "✗ {} {}ms",
                        error,
                        latency.as_millis()
                    );
                })
        )
}

