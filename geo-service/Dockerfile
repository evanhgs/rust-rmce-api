FROM rust:1.88 AS builder
WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./
COPY api/         api/
COPY shared/      shared/
COPY geo-service/ geo-service/

ENV SQLX_OFFLINE=true
RUN cargo build --release -p geo-service

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /workspace/target/release/geo-service /usr/local/bin/
CMD ["geo-service"]
