# Build stage
FROM rust:alpine3.20 AS builder

ARG APP_VERSION=DOCKER_UNKNOWN
ENV APP_VERSION=${APP_VERSION}

# Add dependencies in a single RUN command to reduce layers
RUN apk update && apk add \
    build-base \
    pkgconf \
    openssl-dev openssl-libs-static \
    musl-dev \
    cmake make \
    perl \
    clang18 \
    curl \
    strace

# Set environment variables for static linking
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr

# Set the working directory and copy Cargo.toml separately for caching dependencies
WORKDIR /app

# Copy only the Cargo files to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Pre-fetch cargo dependencies
RUN cargo fetch

# Copy the source code only after dependencies are fetched
COPY . .

# Build the project with static linking
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:latest
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/exchange-rate-bot /app/exchange-rate-bot
WORKDIR /app
CMD ["./exchange-rate-bot"]
