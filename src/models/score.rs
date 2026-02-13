use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Score {
    pub id: i32,
    pub route_id: i32,
    pub user_id: i32,
    pub time_seconds: f32,
    pub max_speed_kmh: Option<f32>,
    pub avg_speed_kmh: Option<f32>,
    pub max_g_force: Option<f32>,
    pub max_inclination_degrees: Option<f32>,
    pub max_sound_db: Option<f32>,
    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateScore {
    // route_id comes from the URL path for /api/routes/:id/score
    pub time_seconds: f32,
    pub max_speed_kmh: Option<f32>,
    pub avg_speed_kmh: Option<f32>,
    pub max_g_force: Option<f32>,
    pub max_inclination_degrees: Option<f32>,
    pub max_sound_db: Option<f32>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct LeaderboardEntry {
    pub user_id: i32,
    pub username: String,
    pub time_seconds: f32,
    pub max_speed_kmh: Option<f32>,
    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: Option<chrono::NaiveDateTime>,
}

fn serialize_datetime<S>(date: &Option<chrono::NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match date {
        Some(d) => serializer.serialize_str(&d.to_string()),
        None => serializer.serialize_none(),
    }
}

