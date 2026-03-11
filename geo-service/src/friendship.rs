use reqwest::Client;
use shared::models::FriendInfo;
use tracing::error;

/// Calls the api service to retrieve the accepted friends list for a given user.
/// Passes the original JWT Bearer token so the api's auth middleware accepts it.
pub async fn get_friends(
    http: &Client,
    api_base_url: &str,
    jwt_token: &str,
) -> Result<Vec<FriendInfo>, reqwest::Error> {
    let url = format!("{}/friends", api_base_url);

    let resp = http
        .get(&url)
        .header("Authorization", format!("Bearer {}", jwt_token))
        .send()
        .await
        .map_err(|e| {
            error!("Failed to reach api /friends: {}", e);
            e
        })?
        .error_for_status()
        .map_err(|e| {
            error!("api /friends returned error status: {}", e);
            e
        })?
        .json::<Vec<FriendInfo>>()
        .await?;

    Ok(resp)
}
