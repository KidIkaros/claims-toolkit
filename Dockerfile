# Multi-stage build for minimal production image
FROM rust:1.85-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build release binary
RUN cargo build --release -p claims-toolkit-cli

# Runtime stage - minimal Debian image
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/claims-toolkit /usr/local/bin/claims-toolkit

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash claims

# Switch to non-root user
USER claims

# Default command
ENTRYPOINT ["claims-toolkit"]
CMD ["--help"]
