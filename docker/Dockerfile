# ============================================================================
# Enterprise-Grade Multi-Stage Dockerfile for LLM Registry
#
# Features:
# - Multi-stage builds for minimal image size
# - Security hardening with non-root user
# - gRPC/Protobuf support
# - Health checks
# ============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Builder - Build application
# -----------------------------------------------------------------------------
FROM rust:latest AS builder

WORKDIR /app

# Install system dependencies including protobuf compiler for gRPC
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    libprotobuf-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migrations ./migrations
COPY src ./src

# Build application with all optimizations
RUN cargo build -p llm-registry-server --release && \
    strip /app/target/release/llm-registry-server

# -----------------------------------------------------------------------------
# Stage 2: Runtime - Minimal secure runtime image
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies and security updates
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && apt-get upgrade -y \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user with specific UID/GID for security
RUN groupadd -g 10001 appuser && \
    useradd -m -u 10001 -g appuser -s /bin/bash appuser

# Create necessary directories
RUN mkdir -p /app/config /app/data /var/log/llm-registry && \
    chown -R appuser:appuser /app /var/log/llm-registry

WORKDIR /app

# Copy binary from builder
COPY --from=builder --chown=appuser:appuser /app/target/release/llm-registry-server /usr/local/bin/llm-registry-server

# Copy configuration files
COPY --chown=appuser:appuser config ./config

# Add health check script (before switching to non-root user)
RUN printf '#!/bin/bash\nset -eo pipefail\nHTTP_STATUS=$(curl -s -o /dev/null -w "%%{http_code}" http://localhost:${SERVER_PORT:-3000}/health || echo "000")\nif [ "$HTTP_STATUS" = "200" ]; then\n  echo "Health check passed"\n  exit 0\nelse\n  echo "Health check failed with status: $HTTP_STATUS"\n  exit 1\nfi\n' > /usr/local/bin/healthcheck.sh && chmod +x /usr/local/bin/healthcheck.sh

# Switch to non-root user
USER appuser

# Expose ports for HTTP, gRPC, and metrics
EXPOSE 3000 50051 9090

# Set environment variables with secure defaults
ENV RUST_LOG=info \
    RUST_BACKTRACE=1 \
    SERVER_HOST=0.0.0.0 \
    SERVER_PORT=3000 \
    GRPC_HOST=0.0.0.0 \
    GRPC_PORT=50051 \
    ENVIRONMENT=production

# Health check configuration
HEALTHCHECK --interval=30s \
            --timeout=10s \
            --start-period=40s \
            --retries=3 \
    CMD ["/usr/local/bin/healthcheck.sh"]

# Metadata labels
LABEL org.opencontainers.image.title="LLM Registry Server" \
      org.opencontainers.image.description="Enterprise-grade LLM asset registry with gRPC and REST APIs" \
      org.opencontainers.image.vendor="LLM DevOps Team" \
      org.opencontainers.image.version="0.1.0" \
      org.opencontainers.image.licenses="Apache-2.0 OR MIT"

# Run the application
CMD ["llm-registry-server"]
