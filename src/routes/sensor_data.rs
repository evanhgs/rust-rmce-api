use axum::{
    Json, Router,
    extract::{Extension, Path},
    http::StatusCode,
    routing::{get, post}
};
use tracing::{info, error};

use crate::{
    db::DbPool,
    models::sensor_data::{BulkSensorData, CreateSensorData, SensorData},
};

pub fn router() -> Router {
    Router::new()
        .route("/", post(upload_sensor_data))
        .route("/bulk", post(upload_bulk_sensor_data))
        .route("/score/{score_id}", get(get_sensor_data))
}

async fn upload_sensor_data(
    Extension(pool): Extension<DbPool>,
    Path(score_id): Path<i32>,
    Json(data): Json<CreateSensorData>,
) -> Result<Json<SensorData>, StatusCode> {
    info!("Upload de données de capteur pour le score {}", score_id);
    
    let sensor_data = sqlx::query_as::<_, SensorData>(
        "INSERT INTO sensor_data (
            score_id, timestamp_offset_ms, accel_x, accel_y, accel_z,
            gyro_x, gyro_y, gyro_z, orientation_azimuth, orientation_pitch, orientation_roll,
            speed_kmh, g_force, inclination_degrees, sound_db, nearby_devices,
            latitude, longitude, altitude
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
        RETURNING id, score_id, timestamp_offset_ms, accel_x, accel_y, accel_z,
                  gyro_x, gyro_y, gyro_z, orientation_azimuth, orientation_pitch, orientation_roll,
                  speed_kmh, g_force, inclination_degrees, sound_db, nearby_devices,
                  latitude, longitude, altitude"
    )
    .bind(score_id)
    .bind(data.timestamp_offset_ms)
    .bind(data.accel_x)
    .bind(data.accel_y)
    .bind(data.accel_z)
    .bind(data.gyro_x)
    .bind(data.gyro_y)
    .bind(data.gyro_z)
    .bind(data.orientation_azimuth)
    .bind(data.orientation_pitch)
    .bind(data.orientation_roll)
    .bind(data.speed_kmh)
    .bind(data.g_force)
    .bind(data.inclination_degrees)
    .bind(data.sound_db)
    .bind(data.nearby_devices)
    .bind(data.latitude)
    .bind(data.longitude)
    .bind(data.altitude)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de l'upload des données de capteur: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Données de capteur uploadées avec succès (ID: {})", sensor_data.id);
    Ok(Json(sensor_data))
}

async fn upload_bulk_sensor_data(
    Extension(pool): Extension<DbPool>,
    Json(bulk_data): Json<BulkSensorData>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Upload en masse de {} points de données pour le score {}", bulk_data.data.len(), bulk_data.score_id);
    
    // Verify score exists
    let score_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM scores WHERE id = $1)"
    )
    .bind(bulk_data.score_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la vérification du score: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if !score_exists {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // Insert all sensor data in a transaction
    let mut tx = pool.begin().await.map_err(|e| {
        error!("Erreur lors du démarrage de la transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let mut inserted_count = 0;
    
    for data in bulk_data.data {
        let result = sqlx::query(
            "INSERT INTO sensor_data (
                score_id, timestamp_offset_ms, accel_x, accel_y, accel_z,
                gyro_x, gyro_y, gyro_z, orientation_azimuth, orientation_pitch, orientation_roll,
                speed_kmh, g_force, inclination_degrees, sound_db, nearby_devices,
                latitude, longitude, altitude
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)"
        )
        .bind(bulk_data.score_id)
        .bind(data.timestamp_offset_ms)
        .bind(data.accel_x)
        .bind(data.accel_y)
        .bind(data.accel_z)
        .bind(data.gyro_x)
        .bind(data.gyro_y)
        .bind(data.gyro_z)
        .bind(data.orientation_azimuth)
        .bind(data.orientation_pitch)
        .bind(data.orientation_roll)
        .bind(data.speed_kmh)
        .bind(data.g_force)
        .bind(data.inclination_degrees)
        .bind(data.sound_db)
        .bind(data.nearby_devices)
        .bind(data.latitude)
        .bind(data.longitude)
        .bind(data.altitude)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Erreur lors de l'insertion des données: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        inserted_count += result.rows_affected();
    }
    
    tx.commit().await.map_err(|e| {
        error!("Erreur lors de la validation de la transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    info!("{} points de données insérés avec succès", inserted_count);
    Ok(Json(serde_json::json!({
        "message": "Sensor data uploaded successfully",
        "inserted_count": inserted_count
    })))
}

async fn get_sensor_data(
    Extension(pool): Extension<DbPool>,
    Path(score_id): Path<i32>,
) -> Result<Json<Vec<SensorData>>, StatusCode> {
    info!("Récupération des données de capteur pour le score {}", score_id);
    
    let sensor_data = sqlx::query_as::<_, SensorData>(
        "SELECT id, score_id, timestamp_offset_ms, accel_x, accel_y, accel_z,
                gyro_x, gyro_y, gyro_z, orientation_azimuth, orientation_pitch, orientation_roll,
                speed_kmh, g_force, inclination_degrees, sound_db, nearby_devices,
                latitude, longitude, altitude
         FROM sensor_data
         WHERE score_id = $1
         ORDER BY timestamp_offset_ms"
    )
    .bind(score_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("Erreur lors de la récupération des données de capteur: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("{} points de données récupérés pour le score {}", sensor_data.len(), score_id);
    Ok(Json(sensor_data))
}

