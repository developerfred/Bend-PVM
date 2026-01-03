# Build stage
FROM rust:1-alpine AS builder

WORKDIR /build

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo fetch && cargo build --release --target x86_64-unknown-linux-musl

# Copy source code
COPY . .

# Build the project
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:3.19 AS runtime

# Install runtime dependencies
RUN apk add --no-cache openssl ca-certificates

# Create non-root user
RUN addgroup -g 1000 app && \
    adduser -u 1000 -G app -s /bin/sh -D app

# Copy binary from builder
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/bend-pvm /usr/local/bin/

# Use non-root user
USER app

# Set environment variables
ENV RUST_LOG=info

# Default command
CMD ["bend-pvm", "--help"]

# Labels for container metadata
LABEL org.opencontainers.image.title="Bend-PVM" \
      org.opencontainers.image.description="Bend Programming Language Virtual Machine" \
      org.opencontainers.image.version="latest" \
      org.opencontainers.image.source="https://github.com/developerfred/Bend-PVM" \
      org.opencontainers.image.maintainer="Bend-PVM Team"
