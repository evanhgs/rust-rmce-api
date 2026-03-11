use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Challenge {
    pub id: i32,
    pub route_id: i32,
    pub challenger_id: i32,
    pub challenged_id: Option<i32>,
    pub status: String,
    pub challenger_time: Option<f32>,
    pub challenged_time: Option<f32>,
    pub winner_id: Option<i32>,
    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: Option<chrono::NaiveDateTime>,
    #[serde(serialize_with = "serialize_datetime")]
    pub completed_at: Option<chrono::NaiveDateTime>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateChallenge {
    pub route_id: i32,
    pub challenged_id: Option<i32>, // None for open challenges
}

#[derive(Serialize, Deserialize)]
pub struct UpdateChallenge {
    pub status: Option<String>,
    pub challenger_time: Option<f32>,
    pub challenged_time: Option<f32>,
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

