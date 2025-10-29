# -------------------------------[ BUILDER ]------------------------------------

# 0.3 Build the binary.

FROM rust:1.90-slim AS builder

# Install required dependencies for building
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    libprotobuf-dev \
    && rm -rf /var/lib/apt/lists/*

# Set a dummy DATABASE_URL for SQLx compile-time checks
ENV DATABASE_URL="sqlite:///tmp/db.sqlite"

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the proto files (submodule is already available in build context)
COPY proto ./proto

# Copy the migrations and run them to set up the database schema
COPY migrations ./migrations
RUN mkdir -p /tmp && touch /tmp/db.sqlite
RUN cargo install sqlx-cli --no-default-features --features sqlite
RUN sqlx migrate run

# Copy the source code
COPY src ./src
COPY build.rs ./

# Build the application in release mode
RUN cargo build --release --bin personal_ledger_backend


# ------------------------------[ RUNTIME IMAGE ]-------------------------------

# Copy the binary to the runtime image

# Use a minimal base image for the runtime
FROM debian:bookworm-slim

# Add metadata labels
LABEL maintainer="Ian Teda <ian@teda.id.au>" \
      description="Personal Ledger Backend - A gRPC-based financial tracking service built with Rust and Tonic" \
      version="0.1.0" \
      repository="https://github.com/IanTeda/personal-ledger-backend" \
      license="GPL-3.0" \
      org.opencontainers.image.source="https://github.com/IanTeda/personal-ledger-backend" \
      org.opencontainers.image.description="Personal Ledger Backend - gRPC service for financial data management" \
      org.opencontainers.image.licenses="GPL-3.0"

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false appuser

# Set the working directory
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/personal_ledger_backend /app/personal_ledger_backend

# Copy configuration files if needed
COPY config ./config

# Change ownership to the non-root user
RUN chown -R appuser:appuser /app

# Create volume directories
VOLUME ["/config", "/data"]

# Switch to the non-root user
USER appuser

# Set environment variable for data directory
ENV DATA_DIR=/data

# Expose the port the app runs on (adjust if different)
EXPOSE 50065

# Set the default command to run the application
CMD ["./personal_ledger_backend"]