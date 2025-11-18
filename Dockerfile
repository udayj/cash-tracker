# Build stage 1: Create a recipe for dependencies
FROM rust:1.89.0 AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Build stage 2: Analyze dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build stage 3: Build dependencies (cached layer)
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build stage 4: Build application
COPY . .
RUN cargo build --release

# Runtime stage: Create minimal image
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install CA certificates for HTTPS connections
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/cash-tracker /app/cash-tracker

# Copy config file
COPY config.json /app/config.json

# Create a non-root user
RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app
USER appuser

ENTRYPOINT ["/app/cash-tracker"]
