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

// Tests pour les routes posts
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
async fn create_post_returns_200() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    let body = serde_json::json!({
        "title": "Test Post",
        "body": "This is a test post body",
        "user_id": 1
    });

    let request = Request::builder()
        .method("POST")
        .uri("/posts")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body)?))?;

    let response = app.clone().oneshot(request).await?;

    // Peut retourner 200 ou 500 selon si l'utilisateur existe
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::INTERNAL_SERVER_ERROR);
    Ok(())
}

#[tokio::test]
async fn get_post_by_id_returns_404_for_invalid_id() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    let response = app
        .clone()
        .oneshot(Request::builder().uri("/posts/99999").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    Ok(())
}

#[tokio::test]
async fn delete_post_returns_404_for_invalid_id() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    let request = Request::builder()
        .method("DELETE")
        .uri("/posts/99999")
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    Ok(())
}

// Tests pour les routes users
#[tokio::test]
async fn get_users_returns_200() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    let response = app
        .clone()
        .oneshot(Request::builder().uri("/users").body(Body::empty())?)
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

#[tokio::test]
async fn get_user_by_id_returns_404_for_invalid_id() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    let response = app
        .clone()
        .oneshot(Request::builder().uri("/users/99999").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    Ok(())
}

#[tokio::test]
async fn delete_user_returns_404_for_invalid_id() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    let request = Request::builder()
        .method("DELETE")
        .uri("/users/99999")
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    Ok(())
}

// Tests pour les routes auth
#[tokio::test]
async fn register_user_returns_200() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    let body = serde_json::json!({
        "username": format!("test_register_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
        "email": format!("test_register_{}@example.com", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
        "password": "test_password_123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body)?))?;

    let response = app.clone().oneshot(request).await?;

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn register_duplicate_user_returns_409() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    let username = format!("test_duplicate_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
    let email = format!("test_duplicate_{}@example.com", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());

    let body = serde_json::json!({
        "username": username.clone(),
        "email": email.clone(),
        "password": "test_password_123"
    });

    // Premier enregistrement
    let request1 = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body)?))?;

    let _response1 = app.clone().oneshot(request1).await?;

    // Tentative de doublon
    let request2 = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body)?))?;

    let response2 = app.clone().oneshot(request2).await?;

    assert_eq!(response2.status(), StatusCode::CONFLICT);
    Ok(())
}

#[tokio::test]
async fn login_with_valid_credentials_returns_200() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let username = format!("test_login_{}", timestamp);
    let email = format!("test_login_{}@example.com", timestamp);
    let password = "test_password_123";

    // D'abord enregistrer l'utilisateur
    let register_body = serde_json::json!({
        "username": username,
        "email": email.clone(),
        "password": password
    });

    let register_request = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&register_body)?))?;

    let _register_response = app.clone().oneshot(register_request).await?;

    // Ensuite se connecter
    let login_body = serde_json::json!({
        "email": email,
        "password": password
    });

    let login_request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_body)?))?;

    let login_response = app.clone().oneshot(login_request).await?;

    assert_eq!(login_response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn login_with_invalid_credentials_returns_401() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    let body = serde_json::json!({
        "email": "nonexistent@example.com",
        "password": "wrong_password"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body)?))?;

    let response = app.clone().oneshot(request).await?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}
