use axum::{
    Json, Router,
    extract::{Extension, Path, Query},
    http::StatusCode,
    routing::{get, post}
};
use tracing::{info, warn, error};
use serde::Deserialize;

use crate::{
    db::DbPool,
    models::route::{CreateRoute, Route, UpdateRoute},
    models::score::{CreateScore, Score},
    routes::auth::Claims,
};

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_routes).post(create_route))
        .route("/{id}", get(get_route).put(update_route).delete(delete_route))
        .route("/{id}/score", post(submit_score))
        .route("/user/{user_id}", get(get_user_routes))
        .route("/public", get(get_public_routes))
}

#[derive(Deserialize)]
struct RouteQuery {
    user_id: Option<i32>,
    is_public: Option<bool>,
}

async fn create_route(
    Extension(pool): Extension<DbPool>,
    Extension(claims): Extension<Claims>,
    Json(new_route): Json<CreateRoute>,
) -> Result<Json<Route>, StatusCode> {
    let user_id = claims.user_id;

    info!("Création d'un nouveau parcours: {} par l'utilisateur {}", new_route.name, user_id);

    let route = sqlx::query_as::<_, Route>(
        "INSERT INTO routes (user_id, name, description, is_public, path_data, distance_meters)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, user_id, name, description, is_public, path_data, distance_meters, created_at, updated_at"
    )
    .bind(user_id)
    .bind(&new_route.name)
    .bind(&new_route.description)
    .bind(new_route.is_public)
    .bind(&new_route.path_data)
    .bind(new_route.distance_meters)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la création du parcours: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Parcours créé avec succès: {} (ID: {})", route.name, route.id);
    Ok(Json(route))
}

async fn get_routes(
    Extension(pool): Extension<DbPool>,
    Query(params): Query<RouteQuery>,
) -> Result<Json<Vec<Route>>, StatusCode> {
    info!("Récupération des parcours avec filtres: user_id={:?}, is_public={:?}", params.user_id, params.is_public);

    let mut query = "SELECT id, user_id, name, description, is_public, path_data, distance_meters, created_at, updated_at FROM routes WHERE 1=1".to_string();

    if params.user_id.is_some() {
        query.push_str(" AND user_id = $1");
    }
    if params.is_public.is_some() {
        query.push_str(" AND is_public = $2");
    }
    query.push_str(" ORDER BY created_at DESC");

    let mut db_query = sqlx::query_as::<_, Route>(&query);

    if let Some(uid) = params.user_id {
        db_query = db_query.bind(uid);
    }
    if let Some(public) = params.is_public {
        db_query = db_query.bind(public);
    }

    let routes = db_query
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!("Erreur lors de la récupération des parcours: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("{} parcours récupérés", routes.len());
    Ok(Json(routes))
}

async fn get_route(
    Extension(pool): Extension<DbPool>,
    Path(id): Path<i32>,
) -> Result<Json<Route>, StatusCode> {
    info!("Récupération du parcours avec ID: {}", id);

    let route = sqlx::query_as::<_, Route>(
        "SELECT id, user_id, name, description, is_public, path_data, distance_meters, created_at, updated_at
         FROM routes WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la récupération du parcours {}: {}", id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match route {
        Some(r) => {
            info!("Parcours {} trouvé: {}", id, r.name);
            Ok(Json(r))
        }
        None => {
            warn!("Parcours {} non trouvé", id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn get_user_routes(
    Extension(pool): Extension<DbPool>,
    Path(user_id): Path<i32>,
) -> Result<Json<Vec<Route>>, StatusCode> {
    info!("Récupération des parcours de l'utilisateur {}", user_id);

    let routes = sqlx::query_as::<_, Route>(
        "SELECT id, user_id, name, description, is_public, path_data, distance_meters, created_at, updated_at
         FROM routes WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la récupération des parcours de l'utilisateur {}: {}", user_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("{} parcours récupérés pour l'utilisateur {}", routes.len(), user_id);
    Ok(Json(routes))
}

async fn get_public_routes(
    Extension(pool): Extension<DbPool>,
) -> Result<Json<Vec<Route>>, StatusCode> {
    info!("Récupération des parcours publics");

    let routes = sqlx::query_as::<_, Route>(
        "SELECT id, user_id, name, description, is_public, path_data, distance_meters, created_at, updated_at
         FROM routes WHERE is_public = true ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la récupération des parcours publics: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("{} parcours publics récupérés", routes.len());
    Ok(Json(routes))
}

async fn update_route(
    Extension(pool): Extension<DbPool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
    Json(update): Json<UpdateRoute>,
) -> Result<Json<Route>, StatusCode> {
    info!("Mise à jour du parcours {}", id);

    // Verify user owns this route
    let route_owner: i32 = sqlx::query_scalar("SELECT user_id FROM routes WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            error!("Erreur lors de la vérification du propriétaire: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if route_owner != claims.user_id {
        warn!("Utilisateur {} a tenté de modifier le parcours {} de l'utilisateur {}", claims.user_id, id, route_owner);
        return Err(StatusCode::FORBIDDEN);
    }

    let route = sqlx::query_as::<_, Route>(
        "UPDATE routes
         SET name = COALESCE($1, name),
             description = COALESCE($2, description),
             is_public = COALESCE($3, is_public),
             path_data = COALESCE($4, path_data),
             distance_meters = COALESCE($5, distance_meters),
             updated_at = NOW()
         WHERE id = $6
         RETURNING id, user_id, name, description, is_public, path_data, distance_meters, created_at, updated_at"
    )
    .bind(update.name)
    .bind(update.description)
    .bind(update.is_public)
    .bind(update.path_data)
    .bind(update.distance_meters)
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la mise à jour du parcours {}: {}", id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match route {
        Some(r) => {
            info!("Parcours {} mis à jour avec succès", id);
            Ok(Json(r))
        }
        None => {
            warn!("Parcours {} non trouvé pour la mise à jour", id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn delete_route(
    Extension(pool): Extension<DbPool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Suppression du parcours {}", id);

    // Verify user owns this route
    let route_owner: i32 = sqlx::query_scalar("SELECT user_id FROM routes WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            error!("Erreur lors de la vérification du propriétaire: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if route_owner != claims.user_id {
        warn!("Utilisateur {} a tenté de supprimer le parcours {} de l'utilisateur {}", claims.user_id, id, route_owner);
        return Err(StatusCode::FORBIDDEN);
    }

    let result = sqlx::query("DELETE FROM routes WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            info!("Parcours {} supprimé avec succès", id);
            Ok(Json(serde_json::json!({
                "message": "Route deleted successfully"
            })))
        }
        Ok(_) => {
            warn!("Parcours {} non trouvé pour la suppression", id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Erreur lors de la suppression du parcours {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn submit_score(
    Extension(pool): Extension<DbPool>,
    Extension(claims): Extension<Claims>,
    Path(route_id): Path<i32>,
    Json(new_score): Json<CreateScore>,
) -> Result<Json<Score>, StatusCode> {
    let user_id = claims.user_id;

    info!("Soumission d'un score pour le parcours {} par l'utilisateur {}", route_id, user_id);

    let route_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM routes WHERE id = $1)"
    )
    .bind(route_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la vérification du parcours: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !route_exists {
        warn!("Parcours {} non trouvé", route_id);
        return Err(StatusCode::NOT_FOUND);
    }

    let score = sqlx::query_as::<_, Score>(
        "INSERT INTO scores (route_id, user_id, time_seconds, max_speed_kmh, avg_speed_kmh, max_g_force, max_inclination_degrees, max_sound_db)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, route_id, user_id, time_seconds, max_speed_kmh, avg_speed_kmh, max_g_force, max_inclination_degrees, max_sound_db, created_at"
    )
    .bind(route_id)
    .bind(user_id)
    .bind(new_score.time_seconds)
    .bind(new_score.max_speed_kmh)
    .bind(new_score.avg_speed_kmh)
    .bind(new_score.max_g_force)
    .bind(new_score.max_inclination_degrees)
    .bind(new_score.max_sound_db)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la soumission du score: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Score soumis avec succès: {} secondes (ID: {})", score.time_seconds, score.id);
    Ok(Json(score))
}
