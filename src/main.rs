use dotenvy::dotenv;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use axum::{Extension, Router, http::StatusCode, routing::{get, post, put, delete}, Json, extract::Path};
use tracing::info;
use tracing_subscriber;
use serde::{Serialize, Deserialize};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
        dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new().connect(&url).await?;

        tracing_subscriber::fmt().init();

        let app = Router::new()
            .route("/posts", get(get_posts).post(create_post))
            .route("/posts/{id}", get(get_post).put(update_post).delete(delete_post))
            .route("/users", post(create_user))
            .layer(Extension(pool));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
        axum::serve(listener, app).await.unwrap();

        Ok(())
}


async fn root() -> &'static str {
    "Hello world"
}

async fn get_posts(
    Extension(pool): Extension<Pool<Postgres>>
) -> std::result::Result<Json<Vec<Post>>, StatusCode> {
    let posts = sqlx::query_as!(Post, "SELECT id, user_id, title, body FROM posts")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(posts))
}

async fn get_post(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> std::result::Result<Json<Post>, StatusCode> {
    let opt = sqlx::query_as!(
        Post,
        "SELECT id, user_id, title, body FROM posts WHERE id = $1",
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match opt {
        Some(post) => Ok(Json(post)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_post(
    Extension(pool): Extension<Pool<Postgres>>,
    Json(new_post): Json<CreatePost>
) -> std::result::Result<Json<Post>, StatusCode> {
    let post = sqlx::query_as!(
        Post,
        "INSERT INTO posts (user_id, title, body) VALUES ($1, $2, $3) RETURNING id, user_id, title, body",
        new_post.user_id,
        new_post.title,
        new_post.body
    ).fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(post))
}

async fn update_post(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
    Json(updated_post): Json<UpdatePost>,
) -> std::result::Result<Json<Post>, StatusCode> {
    let res = sqlx::query_as!(
        Post,
        "UPDATE posts SET title = $1, body = $2, user_id = $3 WHERE id = $4 RETURNING id, user_id, title, body",
        updated_post.title,
        updated_post.body,
        updated_post.user_id,
        id
    )
    .fetch_one(&pool)
    .await;

    match res {
        Ok(post) => Ok(Json(post)),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_post(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query!("DELETE FROM posts WHERE id = $1", id)
        .execute(&pool)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => Ok(Json(serde_json::json! ({
            "message": "Post deleted successfully"
        }))),
        Ok(_) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_user (
    Extension(pool): Extension<Pool<Postgres>>,
    Json(new_user): Json<CreateUser>
) -> std::result::Result<Json<User>, StatusCode> {
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id, username, email",
        new_user.username,
        new_user.email
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}

#[derive(Serialize, Deserialize)]
struct Post {
    id: i32,
    user_id: Option<i32>,
    title: String,
    body: String,
}

#[derive(Serialize, Deserialize)]
struct CreatePost {
    title: String,
    body: String,
    user_id: Option<i32>,
}

#[derive(Serialize, Deserialize)]
struct UpdatePost {
    title: String,
    body: String,
    user_id: Option<i32>,
}

#[derive(Serialize)]
struct Message {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct CreateUser {
    username: String,
    email: String,
}
 
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    email: String,
}
