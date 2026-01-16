# Multi-stage Docker build for Gluetun Monitor
# Optimized for minimal size and security

# Stage 1: Build the binary
FROM rust:1.83-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache \
    musl-dev

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src
COPY tests ./tests

# Build release binary with optimizations and strip symbols
RUN touch src/main.rs && \
    cargo build --release && \
    strip /app/target/release/gluetun-monitor

# Stage 2: Runtime image (Alpine for minimal size)
FROM alpine:3.19

# Install only runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc

# Create non-root user
RUN adduser -D -u 1000 monitor

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/gluetun-monitor /usr/local/bin/gluetun-monitor

# Switch to non-root user
USER monitor

# Expose API port
EXPOSE 3010

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --quiet --tries=1 --spider http://localhost:3010/status || exit 1

# Run the monitor
ENTRYPOINT ["/usr/local/bin/gluetun-monitor"]
