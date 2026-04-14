use axum::{
    Router,
    extract::{
        ws::{Message, WebSocket},
        Extension, Query, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use redis::AsyncCommands;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use shared::jwt::verify_jwt;
use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::{friendship::get_friends, redis_client::RedisHandle};

#[derive(Clone)]
pub struct AppState {
    pub redis: Arc<RedisHandle>,
    pub http: Client,
    pub api_base_url: Arc<String>,
}

#[derive(Deserialize)]
struct WsQuery {
    token: String,
}

#[derive(Deserialize)]
struct InboundPosition {
    lat: f64,
    lng: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LocationMessage {
    pub user_id: i32,
    pub lat: f64,
    pub lng: f64,
    pub timestamp: u64,
}

pub fn create_app(redis: RedisHandle, api_base_url: String) -> Router {
    let state = AppState {
        redis: Arc::new(redis),
        http: Client::new(),
        api_base_url: Arc::new(api_base_url),
    };
    Router::new()
        .route("/ws", get(ws_handler))
        .layer(Extension(state))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsQuery>,
    Extension(state): Extension<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, params.token, state))
}

async fn handle_socket(mut socket: WebSocket, token: String, state: AppState) {
    // 1. Verify JWT
    let claims = match verify_jwt(&token) {
        Ok(c) => c,
        Err(e) => {
            warn!("WS auth failed: {}", e);
            let msg = serde_json::json!({"error": "unauthorized", "message": "Invalid JWT"});
            let _ = socket.send(Message::Text(msg.to_string().into())).await;
            return;
        }
    };
    let user_id = claims.user_id;
    info!("WS connected: user_id={}", user_id);

    // 2. Fetch accepted friends from the api service
    let friends = match get_friends(&state.http, &state.api_base_url, &token).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to fetch friends for user {}: {}", user_id, e);
            let msg = serde_json::json!({"error": "internal", "message": "Failed to fetch friends"});
            let _ = socket.send(Message::Text(msg.to_string().into())).await;
            return;
        }
    };
    let friend_ids: Vec<i32> = friends.iter().map(|f| f.id).collect();
    info!("user {} has {} accepted friends", user_id, friend_ids.len());

    // 3. Split socket into sender/receiver halves
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Bridge between the Redis subscriber task and the WS sender task
    let (tx, mut rx) = broadcast::channel::<LocationMessage>(64);

    // Subscriber task: opens a dedicated pub/sub connection, subscribes to
    // location:{friend_id} for each friend, forwards LocationMessages to tx.
    let sub_client = state.redis.client.clone();
    let sub_tx = tx.clone();
    let sub_friend_ids = friend_ids.clone();
    tokio::spawn(async move {
        let mut pubsub = match sub_client.get_async_pubsub().await {
            Ok(ps) => ps,
            Err(e) => {
                error!("Failed to open Redis pub/sub connection: {}", e);
                return;
            }
        };
        for fid in &sub_friend_ids {
            let channel = format!("location:{}", fid);
            if let Err(e) = pubsub.subscribe(&channel).await {
                error!("Failed to subscribe to {}: {}", channel, e);
            }
        }
        let mut stream = pubsub.on_message();
        while let Some(msg) = stream.next().await {
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(e) => {
                    error!("Redis payload error: {}", e);
                    continue;
                }
            };
            if let Ok(loc) = serde_json::from_str::<LocationMessage>(&payload) {
                // If all receivers dropped (socket closed), stop
                if sub_tx.send(loc).is_err() {
                    break;
                }
            }
        }
    });

    // Sender task: drains the broadcast channel, writes JSON frames to the WS client
    tokio::spawn(async move {
        while let Ok(loc) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&loc) {
                if ws_sender.send(Message::Text(json.into())).await.is_err() {
                    break; // client disconnected
                }
            }
        }
    });

    // 4. Main loop: receive position updates from client, publish to Redis
    let mut redis_pub = state.redis.mgr.clone();
    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(pos) = serde_json::from_str::<InboundPosition>(&text) {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    let loc = LocationMessage {
                        user_id,
                        lat: pos.lat,
                        lng: pos.lng,
                        timestamp: now,
                    };

                    if let Ok(payload) = serde_json::to_string(&loc) {
                        let pub_channel = format!("location:{}", user_id);
                        let pos_key = format!("pos:{}", user_id);

                        // Publish to Pub/Sub for friend subscribers
                        if let Err(e) = redis_pub.publish::<_, _, ()>(&pub_channel, &payload).await {
                            error!("Redis PUBLISH failed for user {}: {}", user_id, e);
                        }

                        // Store last known position with 30s TTL
                        if let Err(e) = redis_pub.set_ex::<_, _, ()>(&pos_key, &payload, 30u64).await {
                            error!("Redis SET pos failed for user {}: {}", user_id, e);
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    info!("WS disconnected: user_id={}", user_id);
    // Dropping tx causes rx in the sender task to receive an error and exit cleanly
    drop(tx);
}
