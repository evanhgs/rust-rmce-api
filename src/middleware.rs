use axum::{
    middleware::Next,
    http::Request,
    body::Body,
    response::Response,
};
use tracing::error;
use crate::routes::auth::verify_jwt;

/// Middleware pour extraire et vérifier le JWT token
pub async fn auth_middleware(
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| {
            error!("Missing or invalid Authorization header");
            axum::http::StatusCode::UNAUTHORIZED
        })?;

    let claims = verify_jwt(token)
        .map_err(|e| {
            error!("Invalid JWT token: {}", e);
            axum::http::StatusCode::UNAUTHORIZED
        })?;

    // Insérer les claims dans les extensions pour les utiliser dans les routes
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

