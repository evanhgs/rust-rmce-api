use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use serde_json::Value;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Route {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub path_data: Value,
    pub distance_meters: Option<f32>,
    #[serde(serialize_with = "serialize_datetime")]
    #[sqlx(rename = "created_at")]
    pub created_at: Option<chrono::NaiveDateTime>,
    #[serde(serialize_with = "serialize_datetime")]
    #[sqlx(rename = "updated_at")]
    pub updated_at: Option<chrono::NaiveDateTime>,
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

#[derive(Serialize, Deserialize)]
pub struct CreateRoute {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub path_data: Value,
    pub distance_meters: Option<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateRoute {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub path_data: Option<Value>,
    pub distance_meters: Option<f32>,
}

