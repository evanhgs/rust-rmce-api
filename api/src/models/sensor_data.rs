use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct SensorData {
    pub id: i32,
    pub score_id: i32,
    pub timestamp_offset_ms: i32,
    pub accel_x: Option<f32>,
    pub accel_y: Option<f32>,
    pub accel_z: Option<f32>,
    pub gyro_x: Option<f32>,
    pub gyro_y: Option<f32>,
    pub gyro_z: Option<f32>,
    pub orientation_azimuth: Option<f32>,
    pub orientation_pitch: Option<f32>,
    pub orientation_roll: Option<f32>,
    pub speed_kmh: Option<f32>,
    pub g_force: Option<f32>,
    pub inclination_degrees: Option<f32>,
    pub sound_db: Option<f32>,
    pub nearby_devices: Option<i32>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub altitude: Option<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateSensorData {
    pub timestamp_offset_ms: i32,
    pub accel_x: Option<f32>,
    pub accel_y: Option<f32>,
    pub accel_z: Option<f32>,
    pub gyro_x: Option<f32>,
    pub gyro_y: Option<f32>,
    pub gyro_z: Option<f32>,
    pub orientation_azimuth: Option<f32>,
    pub orientation_pitch: Option<f32>,
    pub orientation_roll: Option<f32>,
    pub speed_kmh: Option<f32>,
    pub g_force: Option<f32>,
    pub inclination_degrees: Option<f32>,
    pub sound_db: Option<f32>,
    pub nearby_devices: Option<i32>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub altitude: Option<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct BulkSensorData {
    pub score_id: i32,
    pub data: Vec<CreateSensorData>,
}

