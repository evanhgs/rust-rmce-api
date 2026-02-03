command used: 

```bash
docker run --name rust-postgres-db \
    -e POSTGRES_PASSWORD=password \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_DB=rust-rmce-api \
    -p 5432:5432 \
    -d postgres
```
cargo add sqlx --features runtime-tokio,tls-native-tls,postgres
cargo install sqlx-cli --no-default-features --features native-tls,postgres
cp .env.example .env
sqlx database create
sqlx migrate add create_users_table
sqlx migrate add create_posts_table
sqlx migrate run
cargo add tokio -F full
cargo add dotenvy
cargo add axum serde tracing tracing_subscriber --features serde/derive
cargo add serde_json