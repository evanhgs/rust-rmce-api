use dotenvy::dotenv;
use tracing_subscriber;

use rust_rmce_api::{db, routes};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    tracing_subscriber::fmt().init();

    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::create_pool(&url).await?;

    let app = routes::create_app(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
