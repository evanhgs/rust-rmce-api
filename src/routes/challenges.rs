use axum::{
    Json, Router,
    extract::{Extension, Path},
    http::StatusCode,
    routing::{get, post}
};
use tracing::{info, warn, error};

use crate::{
    db::DbPool,
    models::{
        challenge::{Challenge, CreateChallenge, UpdateChallenge},
        score::LeaderboardEntry,
    },
};

pub fn router() -> Router {
    Router::new()
        // Challenge routes
        .route("/challenges", post(create_challenge))
        .route("/challenges/{id}", get(get_challenge))
        .route("/challenges/{id}/accept", post(accept_challenge))
        .route("/challenges/{id}/complete", post(complete_challenge))
        .route("/challenges/available", get(get_available_challenges))

        // Leaderboard routes
        .route("/leaderboard/route/{route_id}", get(get_route_leaderboard))
        .route("/leaderboard/global/speed", get(get_global_speed_leaderboard))
}

// ============ Challenge Routes ============

async fn create_challenge(
    Extension(pool): Extension<DbPool>,
    Json(new_challenge): Json<CreateChallenge>,
) -> Result<Json<Challenge>, StatusCode> {
    // TODO: Get user_id from JWT token
    let user_id = 1; // Temporary placeholder

    info!("Création d'un défi sur le parcours {} par l'utilisateur {}", new_challenge.route_id, user_id);

    let challenge = sqlx::query_as::<_, Challenge>(
        "INSERT INTO challenges (route_id, challenger_id, challenged_id, status)
         VALUES ($1, $2, $3, 'pending')
         RETURNING id, route_id, challenger_id, challenged_id, status, challenger_time, challenged_time, winner_id, created_at, completed_at"
    )
    .bind(new_challenge.route_id)
    .bind(user_id)
    .bind(new_challenge.challenged_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la création du défi: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Défi créé avec succès (ID: {})", challenge.id);
    Ok(Json(challenge))
}

async fn get_challenge(
    Extension(pool): Extension<DbPool>,
    Path(id): Path<i32>,
) -> Result<Json<Challenge>, StatusCode> {
    info!("Récupération du défi {}", id);

    let challenge = sqlx::query_as::<_, Challenge>(
        "SELECT id, route_id, challenger_id, challenged_id, status, challenger_time, challenged_time, winner_id, created_at, completed_at
         FROM challenges WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la récupération du défi: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match challenge {
        Some(c) => {
            info!("Défi {} trouvé", id);
            Ok(Json(c))
        }
        None => {
            warn!("Défi {} non trouvé", id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn accept_challenge(
    Extension(pool): Extension<DbPool>,
    Path(id): Path<i32>,
) -> Result<Json<Challenge>, StatusCode> {
    // TODO: Get user_id from JWT token and verify they are the challenged user

    info!("Acceptation du défi {}", id);

    let challenge = sqlx::query_as::<_, Challenge>(
        "UPDATE challenges
         SET status = 'active'
         WHERE id = $1 AND status = 'pending'
         RETURNING id, route_id, challenger_id, challenged_id, status, challenger_time, challenged_time, winner_id, created_at, completed_at"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de l'acceptation du défi: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match challenge {
        Some(c) => {
            info!("Défi {} accepté", id);
            Ok(Json(c))
        }
        None => {
            warn!("Défi {} non trouvé ou déjà accepté", id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn complete_challenge(
    Extension(pool): Extension<DbPool>,
    Path(id): Path<i32>,
    Json(update): Json<UpdateChallenge>,
) -> Result<Json<Challenge>, StatusCode> {
    info!("Complétion du défi {}", id);

    // Determine winner based on times
    let winner_query = "
        SELECT
            CASE
                WHEN $1 IS NOT NULL AND $2 IS NOT NULL THEN
                    CASE WHEN $1 < $2 THEN challenger_id ELSE challenged_id END
                ELSE NULL
            END as winner
        FROM challenges WHERE id = $3";

    let winner_id: Option<i32> = sqlx::query_scalar(winner_query)
        .bind(update.challenger_time)
        .bind(update.challenged_time)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            error!("Erreur lors du calcul du gagnant: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .flatten();

    let challenge = sqlx::query_as::<_, Challenge>(
        "UPDATE challenges
         SET status = COALESCE($1, status),
             challenger_time = COALESCE($2, challenger_time),
             challenged_time = COALESCE($3, challenged_time),
             winner_id = $4,
             completed_at = CASE WHEN $1 = 'completed' THEN NOW() ELSE completed_at END
         WHERE id = $5
         RETURNING id, route_id, challenger_id, challenged_id, status, challenger_time, challenged_time, winner_id, created_at, completed_at"
    )
    .bind(update.status)
    .bind(update.challenger_time)
    .bind(update.challenged_time)
    .bind(winner_id)
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la complétion du défi: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match challenge {
        Some(c) => {
            info!("Défi {} complété", id);
            Ok(Json(c))
        }
        None => {
            warn!("Défi {} non trouvé", id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn get_available_challenges(
    Extension(pool): Extension<DbPool>,
) -> Result<Json<Vec<Challenge>>, StatusCode> {
    info!("Récupération des défis disponibles");

    let challenges = sqlx::query_as::<_, Challenge>(
        "SELECT id, route_id, challenger_id, challenged_id, status, challenger_time, challenged_time, winner_id, created_at, completed_at
         FROM challenges
         WHERE status = 'pending' AND challenged_id IS NULL
         ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la récupération des défis disponibles: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("{} défis disponibles récupérés", challenges.len());
    Ok(Json(challenges))
}

// ============ Leaderboard Routes ============

async fn get_route_leaderboard(
    Extension(pool): Extension<DbPool>,
    Path(route_id): Path<i32>,
) -> Result<Json<Vec<LeaderboardEntry>>, StatusCode> {
    info!("Récupération du classement pour le parcours {}", route_id);

    let leaderboard = sqlx::query_as::<_, LeaderboardEntry>(
        "SELECT DISTINCT ON (s.user_id)
            s.user_id, u.username, s.time_seconds, s.max_speed_kmh, s.created_at
         FROM scores s
         JOIN users u ON u.id = s.user_id
         WHERE s.route_id = $1
         ORDER BY s.user_id, s.time_seconds ASC
         LIMIT 100"
    )
    .bind(route_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la récupération du classement: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("{} entrées récupérées pour le classement du parcours {}", leaderboard.len(), route_id);
    Ok(Json(leaderboard))
}

async fn get_global_speed_leaderboard(
    Extension(pool): Extension<DbPool>,
) -> Result<Json<Vec<LeaderboardEntry>>, StatusCode> {
    info!("Récupération du classement global des vitesses");

    let leaderboard = sqlx::query_as::<_, LeaderboardEntry>(
        "SELECT DISTINCT ON (s.user_id)
            s.user_id, u.username, s.time_seconds, s.max_speed_kmh, s.created_at
         FROM scores s
         JOIN users u ON u.id = s.user_id
         WHERE s.max_speed_kmh IS NOT NULL
         ORDER BY s.user_id, s.max_speed_kmh DESC
         LIMIT 100"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la récupération du classement des vitesses: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("{} entrées récupérées pour le classement global des vitesses", leaderboard.len());
    Ok(Json(leaderboard))
}
