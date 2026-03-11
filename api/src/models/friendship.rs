use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Friendship {
    pub id: i32,
    pub user_id: i32,
    pub friend_id: i32,
    pub status: String,
    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct FriendInfo {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub status: String,
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

