# ===========================
# RSWS_V1 Dockerfile — Multi-stage build with low memory
# ===========================

# ---- Builder stage ----
FROM rust:1.96-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Configure cargo for low-memory build
ENV CARGO_BUILD_JOBS=1
ENV CARGO_INCREMENTAL=0

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY rsws_api/Cargo.toml ./rsws_api/
COPY rsws_service/Cargo.toml ./rsws_service/
COPY rsws_db/Cargo.toml ./rsws_db/
COPY rsws_model/Cargo.toml ./rsws_model/
COPY rsws_common/Cargo.toml ./rsws_common/
COPY rsws_usdt/Cargo.toml ./rsws_usdt/
COPY rsws_bin/Cargo.toml ./rsws_bin/

# Dummy build to cache dependencies
RUN mkdir -p rsws_api/src rsws_service/src rsws_db/src rsws_model/src rsws_common/src rsws_usdt/src rsws_bin/src \
    && echo "fn main() {}" > rsws_api/src/lib.rs \
    && echo "fn main() {}" > rsws_service/src/lib.rs \
    && echo "fn main() {}" > rsws_db/src/lib.rs \
    && echo "fn main() {}" > rsws_model/src/lib.rs \
    && echo "fn main() {}" > rsws_common/src/lib.rs \
    && echo "fn main() {}" > rsws_usdt/src/lib.rs \
    && echo "fn main() {}" > rsws_bin/src/main.rs \
    && cargo build --release --bin resource-sharing-web-system \
    && rm -rf rsws_*/src

# Copy actual source and build
COPY rsws_api/src ./rsws_api/src
COPY rsws_service/src ./rsws_service/src
COPY rsws_db/src ./rsws_db/src
COPY rsws_model/src ./rsws_model/src
COPY rsws_common/src ./rsws_common/src
COPY rsws_usdt/src ./rsws_usdt/src
COPY rsws_bin/src ./rsws_bin/src

# Build the binary
RUN cargo build --release --bin resource-sharing-web-system

# ---- Runtime stage ----
FROM debian:bookworm-slim

ARG VERSION=dev
ENV APP_VERSION=$VERSION

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

LABEL org.opencontainers.image.version="${VERSION}"
LABEL org.opencontainers.image.source="https://github.com/<owner>/RSWS_V1"
LABEL org.opencontainers.image.title="Resource Sharing Web System"
LABEL org.opencontainers.image.description="数字内容付费交易平台"

COPY --from=builder /app/target/release/resource-sharing-web-system /app/resource-sharing-web-system
RUN chmod +x /app/resource-sharing-web-system

EXPOSE 5170

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:5170/health || exit 1

CMD ["/app/resource-sharing-web-system"]
