#!/bin/sh
set -e

echo "[entrypoint] Running database migrations..."
sqlx migrate run --source /migrations

echo "[entrypoint] Starting API..."
exec rust-rmce-api
