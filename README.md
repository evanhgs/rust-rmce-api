command used: 
cargo add sqlx --features runtime-tokio,tls-native-tls,postgres
cargo install sqlx-cli --no-default-features --features native-tls,postgres
echo "DATABASE_URL=postgres://postgres:password@localhost:5432/rust-axum-rest-api" >> .env
sqlx database create
sqlx migrate add create_users_table
sqlx migrate add create_posts_table
sqlx migrate run
cargo add tokio -F full
cargo add dotenvy
cargo add axum serde tracing tracing_subscriber --features serde/derive
cargo add serde_json
