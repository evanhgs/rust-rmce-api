use redis::{aio::ConnectionManager, Client};

pub struct RedisHandle {
    /// For commands: PUBLISH, SET EX. Clone-able and concurrent-safe.
    pub mgr: ConnectionManager,
    /// For spawning per-session pub/sub connections.
    pub client: Client,
}

pub async fn connect(url: &str) -> Result<RedisHandle, redis::RedisError> {
    let client = Client::open(url)?;
    let mgr = client.get_connection_manager().await?;
    Ok(RedisHandle { mgr, client })
}
