FROM rust:1.87 AS builder
WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./
COPY api/    api/
COPY shared/ shared/

# Stub geo-service so the workspace resolves without building it
RUN mkdir -p geo-service/src && \
    printf '[package]\nname="geo-service"\nversion="0.1.0"\nedition="2024"\n' > geo-service/Cargo.toml && \
    echo 'fn main(){}' > geo-service/src/main.rs

ENV SQLX_OFFLINE=true
RUN cargo build --release -p rust-rmce-api

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /workspace/target/release/rust-rmce-api /usr/local/bin/
CMD ["rust-rmce-api"]
