# Usage Instructions for Personal Ledger Backend Docker Image
#
# 1. Build the Docker image:
#    docker build -t personal-ledger-backend .
#
# 2. Run the container (exposing the gRPC port):
#    docker run -p 50065:50065 personal-ledger-backend
#
#    For development with mounted config and data volumes:
#    docker run -p 50065:50065 -v $(pwd)/config:/etc/personal_ledger_backend -v $(pwd)/data:/var/lib/personal_ledger_backend personal-ledger-backend
#
# 3. Exec into the running container to confirm:
#    docker exec -it <container_id> /bin/bash
#    Inside the container, check the process: ps aux | grep personal_ledger_backend
#    Or view logs: tail -f /var/log/personal_ledger_backend.log (if logging to file)
#    Or test gRPC connectivity from host: grpcurl -plaintext localhost:50065 list
#
# Note: Replace <container_id> with the actual container ID from 'docker ps'.

# Dockerfile for Personal Ledger Backend
# This Dockerfile uses a multi-stage build to create an efficient, secure container image
# for the Rust-based gRPC backend service. It leverages cargo-chef for optimized dependency
# caching and follows best practices for minimal image size and security.


# ------------------------------[ CARGO CHEF ]----------------------------------
# First stage: Set up cargo-chef for dependency caching
# cargo-chef analyzes Cargo.toml files and creates a recipe for dependencies,
# allowing us to cache dependency compilation separately from source code changes.

# Base image with Rust toolchain
FROM rust:1.91 AS chef

# Install cargo-chef tool for dependency management
RUN cargo install cargo-chef
# Set working directory for this stage
WORKDIR /app


# ------------------------------[ PLANNER ]----------------------------------
# Second stage: Analyze dependencies and create a recipe

FROM chef AS planner
# Copy all source files to analyze dependencies
COPY . .
# Generate a recipe.json file with dependency information
RUN cargo chef prepare --recipe-path recipe.json


# -------------------------------[ BUILDER ]------------------------------------
# Third stage: Build the application binary

FROM planner AS builder

# Copy the dependency recipe from planner stage
COPY --from=planner /app/recipe.json recipe.json

# Install system dependencies required for building the Rust application
# These include SSL libraries, protobuf compiler for gRPC code generation, etc.
RUN cargo chef cook --release --recipe-path recipe.json \
    && apt-get update -qq \
    && apt-get install -y -qq --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    libprotobuf-dev \
    # Clean up apt cache to reduce image size
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy the full source code into the builder
COPY . .

# Enable SQLx offline mode to use pre-compiled query metadata instead of connecting to a database
ENV SQLX_OFFLINE=true

# Compile the application in release mode for optimal performance
RUN cargo build --release --bin personal_ledger_backend

# ------------------------------[ RUNTIME IMAGE ]-------------------------------
# Final stage: Create the minimal runtime image
# Use a slim Debian base for security and size, containing only runtime dependencies

FROM debian:12-slim

# Add OCI-compliant metadata labels for the image
LABEL maintainer="Ian Teda <ian@teda.id.au>" \
      description="Personal Ledger Backend - A gRPC-based financial tracking service built with Rust and Tonic" \
      version="0.1.0" \
      repository="https://github.com/IanTeda/personal-ledger-backend" \
      license="GPL-3.0" \
      org.opencontainers.image.source="https://github.com/IanTeda/personal-ledger-backend" \
      org.opencontainers.image.description="Personal Ledger Backend - gRPC service for financial data management" \
      org.opencontainers.image.licenses="GPL-3.0"

# Install minimal runtime dependencies required by the application
# ca-certificates for HTTPS, libssl3 for TLS encryption, sqlite3 for database CLI if needed
RUN apt-get update -qq && apt-get install -y -qq \
    ca-certificates \
    libssl3 \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security (principle of least privilege)
RUN useradd -r -s /bin/false personal_ledger_user

# Set the working directory for the application data
WORKDIR /var/lib/personal_ledger_backend

# Copy the compiled binary from the builder stage to the runtime image
COPY --from=builder /app/target/release/personal_ledger_backend /opt/personal_ledger_backend/personal_ledger_backend

# Ensure the working directory is owned by the non-root user
RUN chown -R personal_ledger_user:personal_ledger_user /var/lib/personal_ledger_backend

# Set environment variables for configuration and database paths
ENV CONFIG_DIR=/etc/personal_ledger_backend
ENV DATABASE_URL=file:/var/lib/personal_ledger_backend/personal_ledger.db

# Define volumes for persistent data and configuration
VOLUME ["/etc/personal_ledger_backend", "/var/lib/personal_ledger_backend"]

# Switch to the non-root user for running the application
USER personal_ledger_user

# Expose the gRPC service port
EXPOSE 50065

# Define the default command to start the application
CMD ["/opt/personal_ledger_backend/personal_ledger_backend"]