// TODO: JWT Authentication Implementation
// Ce fichier montre comment implémenter JWT pour sécuriser l'API

// Dépendances à ajouter dans Cargo.toml (déjà ajoutées):
// jsonwebtoken = "9"

use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub exp: u64, // Expiration time
    pub iat: u64, // Issued at
}

/// Secret key pour signer les tokens (À STOCKER DANS .env EN PRODUCTION)
const JWT_SECRET: &[u8] = b"your-secret-key-change-in-production";

/// Créer un JWT token
pub fn create_jwt(user_id: i32, username: String, email: String) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = Claims {
        user_id,
        username,
        email,
        exp: now + 86400 * 7, // 7 jours
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}

/// Vérifier et décoder un JWT token
pub fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

// EXEMPLE D'UTILISATION DANS LES ROUTES:

/*
use axum::{
    middleware::{self, Next},
    http::Request,
    response::Response,
};

// Middleware pour extraire et vérifier le JWT
pub async fn auth_middleware<B>(
    mut request: Request<B>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = verify_jwt(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Insérer les claims dans les extensions pour les utiliser dans les routes
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

// Utilisation dans le router:
pub fn create_app(pool: DbPool) -> Router {
    Router::new()
        // Routes publiques
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))

        // Routes protégées
        .nest(
            "/api",
            Router::new()
                .route("/routes", get(get_routes).post(create_route))
                .route("/friends", get(get_friends))
                .layer(middleware::from_fn(auth_middleware))
        )
}

// Dans une route protégée:
async fn create_route(
    Extension(pool): Extension<DbPool>,
    Extension(claims): Extension<Claims>,  // Injecté par le middleware
    Json(new_route): Json<CreateRoute>,
) -> Result<Json<Route>, StatusCode> {
    let user_id = claims.user_id; // Récupérer l'ID utilisateur du token
    // ... reste de la fonction
}
*/

// MISE À JOUR DE LA ROUTE LOGIN:

/*
async fn login(
    Extension(pool): Extension<DbPool>,
    Json(login_req): Json<Login>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Tentative de connexion pour l'email: {}", login_req.email);

    let user = sqlx::query_as::<_, UserWithPassword>(
        "SELECT id, username, email, password FROM users WHERE email = $1"
    )
    .bind(&login_req.email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la requête de connexion: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match user {
        Some(u) => {
            info!("Utilisateur trouvé: {}", u.username);
            match verify(login_req.password, &u.password) {
                Ok(true) => {
                    // Créer le JWT token
                    let token = create_jwt(u.id, u.username.clone(), u.email.clone())
                        .map_err(|e| {
                            error!("Erreur lors de la création du token: {}", e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?;

                    info!("Connexion réussie pour l'utilisateur: {}", u.username);
                    Ok(Json(serde_json::json!({
                        "token": token,
                        "user": {
                            "id": u.id,
                            "username": u.username,
                            "email": u.email
                        }
                    })))
                }
                Ok(false) => {
                    warn!("Mot de passe incorrect pour l'utilisateur: {}", u.username);
                    Err(StatusCode::UNAUTHORIZED)
                }
                Err(e) => {
                    error!("Erreur lors de la vérification du mot de passe: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        None => {
            warn!("Tentative de connexion avec un email inexistant: {}", login_req.email);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify_jwt() {
        let token = create_jwt(1, "test_user".to_string(), "test@example.com".to_string())
            .expect("Failed to create JWT");

        let claims = verify_jwt(&token)
            .expect("Failed to verify JWT");

        assert_eq!(claims.user_id, 1);
        assert_eq!(claims.username, "test_user");
        assert_eq!(claims.email, "test@example.com");
    }

    #[test]
    fn test_invalid_token() {
        let result = verify_jwt("invalid.token.here");
        assert!(result.is_err());
    }
}

