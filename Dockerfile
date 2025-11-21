# Build stage
FROM rust:1.89.0-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./

# Copy source code
COPY src ./src
COPY assets ./assets
COPY config.json ./config.json

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/cash-tracker /app/cash-tracker

# Copy config and assets
COPY --from=builder /app/config.json /app/config.json
COPY --from=builder /app/assets /app/assets

# Change ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Run the application
CMD ["./cash-tracker"]
