# syntax=docker/dockerfile:1

# -------- Builder stage --------
FROM rust:latest AS builder
WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs && \
    cargo build --release || true

# Copy source and build
COPY . .
RUN cargo build --release

# -------- Runtime stage --------
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime libs (SSL, CA certs) for sqlx native-tls
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/rust-fintrack-backend /app/rust-fintrack-backend

# Expose port (default 3000)
EXPOSE 3000

# Use env file via compose; here just set a sensible default log level
ENV RUST_LOG=info

ENTRYPOINT ["/app/rust-fintrack-backend"]