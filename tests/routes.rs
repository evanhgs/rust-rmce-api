use std::env;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use dotenvy::dotenv;
use rust_rmce_api::{db, routes};
use tower::ServiceExt; 


async fn build_app() -> Result<Option<axum::Router>, Box<dyn std::error::Error>> {
    dotenv().ok();

    let url = match env::var("DATABASE_URL") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("DATABASE_URL non définie, les tests des routes SQLX sont ignorés.");
            return Ok(None);
        }
    };

    let pool = db::create_pool(&url).await?;
    let app = routes::create_app(pool);

    Ok(Some(app))
}

#[tokio::test]
async fn health_route_root_returns_200() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    let response = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn get_posts_returns_200() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    let response = app
        .clone()
        .oneshot(Request::builder().uri("/posts").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn create_user_returns_200() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    let body = serde_json::json!({
        "username": "test_user_routes",
        "email": "test_user_routes@example.com"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body)?))?;

    let response = app.clone().oneshot(request).await?;

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

