# ─── Stage 1: Cache dependencies ─────────────────────────────
FROM rust:slim-bookworm AS deps

RUN apt-get update && apt-get install -y pkg-config libssl-dev protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY protos protos

# All workspace member Cargo.toml files (in proper subdirectories)
COPY packages/common/Cargo.toml packages/common/Cargo.toml
COPY packages/proto/Cargo.toml packages/proto/Cargo.toml
COPY packages/gateway/Cargo.toml packages/gateway/Cargo.toml
COPY packages/frontend/Cargo.toml packages/frontend/Cargo.toml
COPY packages/services/identity/Cargo.toml packages/services/identity/Cargo.toml
COPY packages/services/sis/Cargo.toml packages/services/sis/Cargo.toml
COPY packages/services/academic/Cargo.toml packages/services/academic/Cargo.toml
COPY packages/services/attendance/Cargo.toml packages/services/attendance/Cargo.toml
COPY packages/services/notifications/Cargo.toml packages/services/notifications/Cargo.toml
COPY packages/services/finance/Cargo.toml packages/services/finance/Cargo.toml
COPY packages/services/reporting/Cargo.toml packages/services/reporting/Cargo.toml
COPY packages/services/portal/Cargo.toml packages/services/portal/Cargo.toml
COPY packages/services/curriculum/Cargo.toml packages/services/curriculum/Cargo.toml
COPY packages/services/crm/Cargo.toml packages/services/crm/Cargo.toml

# Dummy sources for cargo metadata resolution
RUN mkdir -p packages/common/src packages/proto/src && \
    touch packages/common/src/lib.rs packages/proto/src/lib.rs && \
    for pkg in gateway frontend; do \
      mkdir -p packages/$pkg/src && echo "fn main() {}" > packages/$pkg/src/main.rs; \
    done && \
    for pkg in identity sis academic attendance notifications finance reporting portal curriculum crm; do \
      mkdir -p packages/services/$pkg/src && echo "fn main() {}" > packages/services/$pkg/src/main.rs; \
    done

RUN cargo fetch

# ─── Stage 2: Compile ────────────────────────────────────────
FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=deps /usr/local/cargo /usr/local/cargo
COPY Cargo.toml Cargo.lock ./
COPY protos protos
COPY packages packages

RUN cargo build --release --workspace --exclude schoolccb-frontend

# ─── Stage 3: Runtime ────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates wget && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/schoolccb-gateway /usr/local/bin/
COPY --from=builder /app/target/release/schoolccb-identity /usr/local/bin/
COPY --from=builder /app/target/release/schoolccb-sis /usr/local/bin/
COPY --from=builder /app/target/release/schoolccb-academic /usr/local/bin/
COPY --from=builder /app/target/release/schoolccb-attendance /usr/local/bin/
COPY --from=builder /app/target/release/schoolccb-notifications /usr/local/bin/
COPY --from=builder /app/target/release/schoolccb-finance /usr/local/bin/
COPY --from=builder /app/target/release/schoolccb-reporting /usr/local/bin/
COPY --from=builder /app/target/release/schoolccb-portal /usr/local/bin/
COPY --from=builder /app/target/release/schoolccb-curriculum /usr/local/bin/
COPY --from=builder /app/target/release/schoolccb-crm /usr/local/bin/
