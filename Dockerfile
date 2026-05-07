FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY packages/common packages/common
COPY packages/gateway packages/gateway
COPY packages/services packages/services
COPY packages/frontend/Cargo.toml packages/frontend/Cargo.toml
RUN mkdir -p packages/frontend/src && echo "" > packages/frontend/src/lib.rs

RUN cargo build --release --workspace --exclude schoolcbb-frontend

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/schoolcbb-gateway /usr/local/bin/
COPY --from=builder /app/target/release/schoolcbb-identity /usr/local/bin/
COPY --from=builder /app/target/release/schoolcbb-sis /usr/local/bin/
COPY --from=builder /app/target/release/schoolcbb-academic /usr/local/bin/
COPY --from=builder /app/target/release/schoolcbb-attendance /usr/local/bin/
COPY --from=builder /app/target/release/schoolcbb-notifications /usr/local/bin/
