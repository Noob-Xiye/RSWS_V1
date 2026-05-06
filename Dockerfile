# ============================
# RSWS_V1 Dockerfile — Multi-stage build
# ============================

# ---- Builder stage ----
FROM rust:1.85-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY rsws_api/Cargo.toml rsws_api/Cargo.toml
COPY rsws_service/Cargo.toml rsws_service/Cargo.toml
COPY rsws_model/Cargo.toml rsws_model/Cargo.toml
COPY rsws_db/Cargo.toml rsws_db/Cargo.toml
COPY rsws_common/Cargo.toml rsws_common/Cargo.toml
COPY rsws_usdt/Cargo.toml rsws_usdt/Cargo.toml
COPY rsws_bin/Cargo.toml rsws_bin/Cargo.toml

# Create dummy source files to cache deps
RUN mkdir -p rsws_api/src && echo "" > rsws_api/src/lib.rs \
    && mkdir -p rsws_service/src && echo "" > rsws_service/src/lib.rs \
    && mkdir -p rsws_model/src && echo "" > rsws_model/src/lib.rs \
    && mkdir -p rsws_db/src && echo "" > rsws_db/src/lib.rs \
    && mkdir -p rsws_common/src && echo "" > rsws_common/src/lib.rs \
    && mkdir -p rsws_usdt/src && echo "" > rsws_usdt/src/lib.rs \
    && mkdir -p rsws_bin/src && echo "fn main(){}" > rsws_bin/src/main.rs

RUN cargo build --release --bin rsws 2>/dev/null || true

# Copy real source code
COPY . .

RUN touch rsws_api/src/lib.rs rsws_service/src/lib.rs rsws_model/src/lib.rs \
    rsws_db/src/lib.rs rsws_common/src/lib.rs rsws_usdt/src/lib.rs rsws_bin/src/main.rs

RUN cargo build --release --bin rsws

# ---- Runtime stage ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rsws /app/rsws
COPY --from=builder /app/migrations /app/migrations

# Config will be mounted or use environment variables
COPY rsws_bin/config.toml /app/config.toml

EXPOSE 5170

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:5170/health || exit 1

CMD ["/app/rsws"]
