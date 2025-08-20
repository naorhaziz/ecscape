# Stage 1: Build the Rust application in a builder container
FROM rust:alpine3.22 AS builder

# Install build dependencies for musl target
RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy dependency manifests to leverage Docker cache
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs to cache dependencies layer
RUN mkdir src && echo "fn main() {}" > src/main.rs
# Build only dependencies to cache them
RUN cargo build --release
# Clean up dummy files
RUN rm -rf src target/release/deps/ecscape*

# Copy the application source code
COPY ./src ./src

# Build the application for release
RUN cargo build --release

# Stage 2: Create a small production image
FROM alpine:latest

# Set environment variables
ENV RUST_BACKTRACE=full
# anyhow creates a backtrace for every error, which can be quite taxing, turning it off for now
ENV RUST_LIB_BACKTRACE=0
ENV RUST_LOG=info

WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/ecscape ./ecscape

# Change CMD to run the application binary instead of sleeping
CMD ["./ecscape"]
