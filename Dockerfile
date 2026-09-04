# 1. Builder Stage
FROM rust:1.80-slim-bullseye AS builder

WORKDIR /usr/src/app

# Install required build dependencies (OpenSSL, pkg-config, etc.)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the Cargo workspace configs
COPY Cargo.toml Cargo.lock ./
COPY libs/rust-yalc libs/rust-yalc

# Create dummy main.rs to build dependencies and cache them
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Now copy the actual source code
COPY . .
# Touch main.rs to force Cargo to rebuild it (otherwise it thinks the dummy is still valid)
RUN touch src/main.rs

# Build the final release binary
RUN cargo build --release

# 2. Runtime Stage
FROM debian:bullseye-slim

# Install runtime dependencies (e.g. CA certificates for HTTPS requests, OpenSSL)
RUN apt-get update && apt-get install -y ca-certificates libssl1.1 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/rust-boilerplate /usr/local/bin/rust-boilerplate

# Create a non-root user for security
RUN useradd -m -s /bin/bash appuser
USER appuser

# Expose ports: 3000 for Axum, 9000 for Prometheus metrics
EXPOSE 3000
EXPOSE 9000

# Set environment to production (can be overridden by k8s)
ENV RUST_ENV=production

# Run the binary
CMD ["rust-boilerplate"]
