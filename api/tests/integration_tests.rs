use std::env;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use dotenvy::dotenv;
use rust_rmce_api::{db, routes};
use serde_json::json;
use tower::ServiceExt;

async fn build_app() -> Result<Option<axum::Router>, Box<dyn std::error::Error>> {
    dotenv().ok();

    let url = match env::var("DATABASE_URL") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("DATABASE_URL non définie, les tests sont ignorés.");
            return Ok(None);
        }
    };

    let pool = db::create_pool(&url).await?;
    let app = routes::create_app(pool);

    Ok(Some(app))
}

// Generate unique username with timestamp
fn unique_username(base: &str) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{}_{}", base, now)
}

// ============ USER STORIES & INTEGRATION TESTS ============

// USER STORY 1: Authentification et gestion utilisateur
#[tokio::test]
async fn user_story_01_registration_and_login() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    // Story: Un nouvel utilisateur s'inscrit
    let username = unique_username("user_story_1");
    let register_body = json!({
        "username": username,
        "email": format!("{}@test.com", username),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&register_body)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK, "Registration should succeed");

    // Story: L'utilisateur se connecte avec ses identifiants
    let login_body = json!({
        "email": format!("{}@test.com", username),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_body)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK, "Login should succeed");

    // Extraire le token JWT de la réponse
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let login_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let token = login_response["token"].as_str().expect("Token should exist");

    assert!(!token.is_empty(), "Token should not be empty");
    println!("✅ US1: User registration and login successful");
    println!("   Token received: {}...", &token[..20]);

    Ok(())
}

// USER STORY 2: Création et gestion de parcours
#[tokio::test]
async fn user_story_02_create_and_manage_routes() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    // D'abord, enregistrer et se connecter pour obtenir un token
    let username = unique_username("runner_story_2");
    let register_body = json!({
        "username": username,
        "email": format!("{}@test.com", username),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&register_body)?))?;

    let _ = app.clone().oneshot(request).await?;

    let login_body = json!({
        "email": format!("{}@test.com", username),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_body)?))?;

    let response = app.clone().oneshot(request).await?;
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let login_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let token = login_response["token"].as_str().unwrap().to_string();

    // Story: L'utilisateur crée un nouveau parcours
    let create_route_body = json!({
        "name": "Parc Central",
        "description": "5km autour du parc",
        "is_public": true,
        "path_data": {
            "type": "LineString",
            "coordinates": [[2.3522, 48.8566], [2.3523, 48.8567]]
        },
        "distance_meters": 5000.0
    });

    let request = Request::builder()
        .method("POST")
        .uri("/routes")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&create_route_body)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK, "Create route should succeed");

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let route_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let route_id = route_response["id"].as_i64().expect("Route ID should exist") as i32;

    // Story: L'utilisateur visualise le parcours créé
    let request = Request::builder()
        .method("GET")
        .uri(&format!("/routes/{}", route_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK, "Get route should succeed");

    // Story: L'utilisateur met à jour le parcours
    let update_route_body = json!({
        "name": "Parc Central - Updated",
        "distance_meters": 5500.0
    });

    let request = Request::builder()
        .method("PUT")
        .uri(&format!("/routes/{}", route_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&update_route_body)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK, "Update route should succeed");

    println!("✅ US2: Route creation and management successful");
    println!("   Route created with ID: {}", route_id);

    Ok(())
}

// USER STORY 3: Soumettre un score après une course
#[tokio::test]
async fn user_story_03_submit_score_after_run() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    // Setup: Créer utilisateur et parcours
    let username = unique_username("athlete_story_3");
    let register_body = json!({
        "username": username,
        "email": format!("{}@test.com", username),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&register_body)?))?;

    let _ = app.clone().oneshot(request).await?;

    let login_body = json!({
        "email": format!("{}@test.com", username),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_body)?))?;

    let response = app.clone().oneshot(request).await?;
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let login_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let token = login_response["token"].as_str().unwrap().to_string();

    // Créer un parcours
    let create_route_body = json!({
        "name": "Course marathon",
        "description": "Un parcours de 10km",
        "is_public": true,
        "path_data": {"type": "LineString", "coordinates": [[0.0, 0.0], [1.0, 1.0]]},
        "distance_meters": 10000.0
    });

    let request = Request::builder()
        .method("POST")
        .uri("/routes")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&create_route_body)?))?;

    let response = app.clone().oneshot(request).await?;
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let route_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let route_id = route_response["id"].as_i64().unwrap() as i32;

    // Story: L'utilisateur soumet son temps après avoir complété le parcours
    let submit_score_body = json!({
        "time_seconds": 2400.5,
        "max_speed_kmh": 18.5,
        "avg_speed_kmh": 15.0,
        "max_g_force": 1.2,
        "max_inclination_degrees": 8.5,
        "max_sound_db": 85.0
    });

    let request = Request::builder()
        .method("POST")
        .uri(&format!("/routes/{}/score", route_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&submit_score_body)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK, "Submit score should succeed");

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let score_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let score_id = score_response["id"].as_i64().unwrap() as i32;

    println!("✅ US3: Score submission successful");
    println!("   Score ID: {}, Time: {}s, Max Speed: {} km/h",
        score_id,
        score_response["time_seconds"].as_f64().unwrap(),
        score_response["max_speed_kmh"].as_f64().unwrap()
    );

    Ok(())
}

// USER STORY 4: Ajouter un ami et gérer les demandes d'amitié
#[tokio::test]
async fn user_story_04_add_friend_and_manage_requests() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    // Créer deux utilisateurs
    let user1_name = unique_username("alice_story_4");
    let user1_body = json!({
        "username": user1_name,
        "email": format!("{}@test.com", user1_name),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&user1_body)?))?;

    let response = app.clone().oneshot(request).await?;
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let user1_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let user1_id = user1_response["id"].as_i64().unwrap() as i32;

    let user2_name = unique_username("bob_story_4");
    let user2_body = json!({
        "username": user2_name,
        "email": format!("{}@test.com", user2_name),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&user2_body)?))?;

    let response = app.clone().oneshot(request).await?;
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let user2_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let user2_id = user2_response["id"].as_i64().unwrap() as i32;

    // Login user1
    let login_body = json!({
        "email": format!("{}@test.com", user1_name),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_body)?))?;

    let response = app.clone().oneshot(request).await?;
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let login_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let token1 = login_response["token"].as_str().unwrap().to_string();

    // Story: Alice ajoute Bob comme ami
    let request = Request::builder()
        .method("POST")
        .uri(&format!("/friends/add/{}", user2_id))
        .header("Authorization", format!("Bearer {}", token1))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK, "Add friend request should succeed");

    println!("✅ US4: Friend management successful");
    println!("   Alice (ID: {}) sent friend request to Bob (ID: {})", user1_id, user2_id);

    Ok(())
}

// USER STORY 5: Consulter le classement d'un parcours
#[tokio::test]
async fn user_story_05_view_leaderboard() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    // Setup: Créer utilisateur et soumettre des scores
    let username = unique_username("leaderboard_user_5");
    let register_body = json!({
        "username": username,
        "email": format!("{}@test.com", username),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&register_body)?))?;

    let _ = app.clone().oneshot(request).await?;

    let login_body = json!({
        "email": format!("{}@test.com", username),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_body)?))?;

    let response = app.clone().oneshot(request).await?;
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let login_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let token = login_response["token"].as_str().unwrap().to_string();

    // Créer parcours et soumettre score
    let create_route_body = json!({
        "name": "10k race",
        "description": "10km",
        "is_public": true,
        "path_data": {"type": "LineString", "coordinates": [[0.0, 0.0], [1.0, 1.0]]},
        "distance_meters": 10000.0
    });

    let request = Request::builder()
        .method("POST")
        .uri("/routes")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&create_route_body)?))?;

    let response = app.clone().oneshot(request).await?;
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let route_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let route_id = route_response["id"].as_i64().unwrap() as i32;

    let submit_score_body = json!({
        "time_seconds": 2100.0,
        "max_speed_kmh": 20.0,
        "avg_speed_kmh": 17.1,
        "max_g_force": 1.1,
        "max_inclination_degrees": 5.0,
        "max_sound_db": 80.0
    });

    let request = Request::builder()
        .method("POST")
        .uri(&format!("/routes/{}/score", route_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&submit_score_body)?))?;

    let _ = app.clone().oneshot(request).await?;

    // Story: L'utilisateur consulte le classement du parcours
    let request = Request::builder()
        .method("GET")
        .uri(&format!("/api/leaderboard/route/{}", route_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK, "Leaderboard should be accessible");

    println!("✅ US5: View leaderboard successful");
    println!("   Leaderboard retrieved for route ID: {}", route_id);

    Ok(())
}

// USER STORY 6: Télécharger les données de capteurs
#[tokio::test]
async fn user_story_06_upload_sensor_data() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    // Setup
    let username = unique_username("sensor_user_6");
    let register_body = json!({
        "username": username,
        "email": format!("{}@test.com", username),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&register_body)?))?;

    let _ = app.clone().oneshot(request).await?;

    let login_body = json!({
        "email": format!("{}@test.com", username),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_body)?))?;

    let response = app.clone().oneshot(request).await?;
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let login_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let token = login_response["token"].as_str().unwrap().to_string();

    // Create route and score
    let create_route_body = json!({
        "name": "sensor test route",
        "description": "test",
        "is_public": true,
        "path_data": {"type": "LineString", "coordinates": [[0.0, 0.0], [1.0, 1.0]]},
        "distance_meters": 5000.0
    });

    let request = Request::builder()
        .method("POST")
        .uri("/routes")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&create_route_body)?))?;

    let response = app.clone().oneshot(request).await?;
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let route_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let route_id = route_response["id"].as_i64().unwrap() as i32;

    let submit_score_body = json!({
        "time_seconds": 1200.0,
        "max_speed_kmh": 15.0,
        "avg_speed_kmh": 12.0,
        "max_g_force": 1.0,
        "max_inclination_degrees": 3.0,
        "max_sound_db": 75.0
    });

    let request = Request::builder()
        .method("POST")
        .uri(&format!("/routes/{}/score", route_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&submit_score_body)?))?;

    let response = app.clone().oneshot(request).await?;
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let score_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let score_id = score_response["id"].as_i64().unwrap() as i32;

    // Story: L'utilisateur envoie les données de capteurs collectées pendant la course
    let bulk_sensor_data = json!({
        "score_id": score_id,
        "data": [
            {
                "timestamp_offset_ms": 0,
                "accel_x": 0.1,
                "accel_y": 0.2,
                "accel_z": 9.8,
                "gyro_x": 0.01,
                "gyro_y": 0.01,
                "gyro_z": 0.0,
                "orientation_azimuth": 180.0,
                "orientation_pitch": 5.0,
                "orientation_roll": 0.0,
                "speed_kmh": 0.0,
                "g_force": 1.0,
                "inclination_degrees": 0.0,
                "sound_db": 70.0,
                "nearby_devices": 0,
                "latitude": 48.8566,
                "longitude": 2.3522,
                "altitude": 35.0
            },
            {
                "timestamp_offset_ms": 1000,
                "accel_x": 0.5,
                "accel_y": 0.3,
                "accel_z": 9.6,
                "gyro_x": 0.05,
                "gyro_y": 0.02,
                "gyro_z": 0.01,
                "orientation_azimuth": 180.5,
                "orientation_pitch": 5.5,
                "orientation_roll": 0.2,
                "speed_kmh": 5.0,
                "g_force": 1.05,
                "inclination_degrees": 2.0,
                "sound_db": 75.0,
                "nearby_devices": 1,
                "latitude": 48.8567,
                "longitude": 2.3523,
                "altitude": 36.0
            }
        ]
    });

    let request = Request::builder()
        .method("POST")
        .uri("/sensor-data/bulk")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&bulk_sensor_data)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK, "Sensor data upload should succeed");

    println!("✅ US6: Sensor data upload successful");
    println!("   Uploaded {} sensor data points for score ID: {}", 2, score_id);

    Ok(())
}

// ============ SECURITY TESTS ============

#[tokio::test]
async fn security_test_unauthorized_access_without_token() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    // Story: Quelqu'un essaie d'accéder à une route protégée sans token
    let request = Request::builder()
        .method("GET")
        .uri("/routes")
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "Should reject request without token");

    println!("✅ SECURITY: Unauthorized access blocked");

    Ok(())
}

#[tokio::test]
async fn security_test_invalid_token() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    // Story: Quelqu'un essaie d'utiliser un faux token
    let request = Request::builder()
        .method("GET")
        .uri("/routes")
        .header("Authorization", "Bearer invalid.token.here")
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "Should reject invalid token");

    println!("✅ SECURITY: Invalid token rejected");

    Ok(())
}

#[tokio::test]
async fn security_test_user_cannot_modify_others_route() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_app = build_app().await?;
    let app = if let Some(app) = maybe_app {
        app
    } else {
        return Ok(());
    };

    // Create user 1
    let user1_name = unique_username("owner_user");
    let user1_body = json!({
        "username": user1_name,
        "email": format!("{}@test.com", user1_name),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&user1_body)?))?;

    let _ = app.clone().oneshot(request).await?;

    let login_body = json!({
        "email": format!("{}@test.com", user1_name),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_body)?))?;

    let response = app.clone().oneshot(request).await?;
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let login_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let token1 = login_response["token"].as_str().unwrap().to_string();

    // Create route
    let create_route_body = json!({
        "name": "My secret route",
        "description": "Only mine",
        "is_public": false,
        "path_data": {"type": "LineString", "coordinates": [[0.0, 0.0]]},
        "distance_meters": 5000.0
    });

    let request = Request::builder()
        .method("POST")
        .uri("/routes")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token1))
        .body(Body::from(serde_json::to_vec(&create_route_body)?))?;

    let response = app.clone().oneshot(request).await?;
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let route_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let route_id = route_response["id"].as_i64().unwrap() as i32;

    // Create user 2
    let user2_name = unique_username("hacker_user");
    let user2_body = json!({
        "username": user2_name,
        "email": format!("{}@test.com", user2_name),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&user2_body)?))?;

    let _ = app.clone().oneshot(request).await?;

    let login_body = json!({
        "email": format!("{}@test.com", user2_name),
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_body)?))?;

    let response = app.clone().oneshot(request).await?;
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let login_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    let token2 = login_response["token"].as_str().unwrap().to_string();

    // Try to modify user 1's route
    let update_route_body = json!({
        "name": "Hacked route"
    });

    let request = Request::builder()
        .method("PUT")
        .uri(&format!("/routes/{}", route_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token2))
        .body(Body::from(serde_json::to_vec(&update_route_body)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::FORBIDDEN, "Should prevent unauthorized modification");

    println!("✅ SECURITY: Route ownership enforced");

    Ok(())
}

