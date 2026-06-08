# ============================
# RSWS_V1 Dockerfile — Multi-stage build
# ============================

# ---- Builder stage ----
FROM rust:1.92-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Limit parallel compilation to avoid OOM during docker build
ENV CARGO_BUILD_JOBS=2

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

RUN cargo build --release --bin resource-sharing-web-system 2>/dev/null || true

# Copy real source code
COPY . .

RUN touch rsws_api/src/lib.rs rsws_service/src/lib.rs rsws_model/src/lib.rs \
    rsws_db/src/lib.rs rsws_common/src/lib.rs rsws_usdt/src/lib.rs rsws_bin/src/main.rs

RUN cargo build --release --bin resource-sharing-web-system

# ---- Runtime stage ----
FROM debian:bookworm-slim

ARG VERSION=dev
ENV APP_VERSION=$VERSION

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

LABEL org.opencontainers.image.version="${VERSION}"
LABEL org.opencontainers.image.source="https://github.com/<owner>/RSWS_V1"
LABEL org.opencontainers.image.title="Resource Sharing Web System"
LABEL org.opencontainers.image.description="数字内容付费交易平台"

COPY --from=builder /app/target/release/resource-sharing-web-system /app/resource-sharing-web-system
# Config via environment variables (RSWS__DATABASE__URL, RSWS__REDIS__URL, etc.)
# No default config.toml — all settings via env

EXPOSE 5170

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:5170/health || exit 1

CMD ["/app/resource-sharing-web-system"]
