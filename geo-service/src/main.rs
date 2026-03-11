use dotenvy::dotenv;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod friendship;
mod redis_client;
mod ws;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://redis:6379".to_string());

    let api_base_url = std::env::var("API_BASE_URL")
        .unwrap_or_else(|_| "http://api:5000".to_string());

    let redis = redis_client::connect(&redis_url)
        .await
        .expect("Failed to connect to Redis");

    let app = ws::create_app(redis, api_base_url);

    let addr = "0.0.0.0:8080";
    info!("geo-service WebSocket server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.unwrap();
}
