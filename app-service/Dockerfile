# Start with image that has the Rust toolchain installed
FROM rust:1.85-alpine AS chef
USER root
# Add cargo-chef to cache dependencies
RUN apk add --no-cache musl-dev && cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
# Capture info needed to build dependencies
RUN cargo chef prepare --recipe-path recipe.json --bin app-service

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json --bin app-service
# Build application
COPY . .
RUN cargo build --release --bin app-service

# We do not need the Rust toolchain to run the binary!
# Start with a minimal image and copy over the binary and assets folder.
FROM debian:buster-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/app-service /usr/local/bin
# Fix the assets path - it should be relative to the workspace root
COPY --from=builder /app/app-service/assets /app/assets
ENTRYPOINT ["/usr/local/bin/app-service"]