# Use the official Rust image as the base image for building
FROM rust:1.90-slim AS builder

# Install required packages for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy openapi.yaml before building
COPY openapi.yaml ./openapi.yaml

# Copy the source code
COPY src ./src
COPY migrations ./migrations
COPY configuration ./configuration
COPY templates ./templates
COPY static ./static

# Build the application in release mode
RUN cargo build --release

# Use a minimal base image for the runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd --create-home --shell /bin/bash app

# Set the working directory
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/url-shortener-ztm /app/url-shortener-ztm

# Copy configuration files
COPY configuration ./configuration
COPY templates ./templates
COPY static ./static

# Create data directory for SQLite database
RUN mkdir -p /app/data

# Change ownership of the app directory to the app user
RUN chown -R app:app /app

# Switch to the non-root user
USER app

# Set environment to production (uses configuration/production.yml)
ENV APP_ENVIRONMENT=production

# Expose the port the app runs on
EXPOSE 8000

# Set the default command to run the application
CMD ["./url-shortener-ztm"]