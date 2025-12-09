# Multi-stage Dockerfile for RiceCoder
# Stage 1: Builder - Compile the application
FROM rust:1.75-bookworm as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# Add musl target for static linking
RUN rustup target add x86_64-unknown-linux-musl

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the application with static linking
RUN RUSTFLAGS='-C target-feature=+crt-static' \
    cargo build --release \
    --target x86_64-unknown-linux-musl \
    -p ricecoder-cli

# Stage 2: Runtime - Minimal Alpine image
FROM alpine:3.18

WORKDIR /app

# Install runtime dependencies (minimal)
RUN apk add --no-cache \
    ca-certificates \
    tini

# Copy binary from builder
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/ricecoder /usr/local/bin/ricecoder

# Create non-root user for security
RUN addgroup -g 1000 ricecoder && \
    adduser -D -u 1000 -G ricecoder ricecoder

# Set working directory for user
WORKDIR /workspace

# Change ownership to non-root user
RUN chown -R ricecoder:ricecoder /workspace

# Switch to non-root user
USER ricecoder

# Use tini as entrypoint to handle signals properly
ENTRYPOINT ["/sbin/tini", "--"]

# Default command
CMD ["ricecoder", "--help"]

# Labels for metadata
LABEL org.opencontainers.image.title="RiceCoder"
LABEL org.opencontainers.image.description="AI-powered code generation and refactoring tool"
LABEL org.opencontainers.image.authors="RiceCoder Contributors"
LABEL org.opencontainers.image.source="https://github.com/moabualruz/ricecoder"
