use serde::{Deserialize, Serialize};

/// Wire-format user (no sqlx dependency — safe to use across services).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
}

/// Friend info as returned by the api /friends endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FriendInfo {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub status: String,
}
