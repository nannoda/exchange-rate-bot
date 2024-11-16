# Build stage
FROM rust:alpine3.20 AS builder

ARG APP_VERSION=DOCKER_UNKNOWN
ENV APP_VERSION=${APP_VERSION}

# Add dependencies in a single RUN command to reduce layers
RUN apk update && apk add --no-cache \
    alpine-sdk\
    build-base \
    pkgconf \
    openssl-dev openssl-libs-static \
    musl-dev \
    cmake make \
    perl \
    clang18 \
    curl \
    strace \
    fontconfig-dev \
    freetype-dev \
    libstdc++ \
    zlib-dev

    RUN pkg-config --modversion fontconfig freetype && \
    ls -al /usr/lib | grep libfontconfig && \
    ls -al /usr/lib | grep libfreetype

# Add fonts
RUN apk add font-terminus font-inconsolata font-dejavu font-noto font-noto-cjk font-awesome font-noto-extra

# Set environment variables for static linking
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr
ENV PKG_CONFIG_ALLOW_SYSTEM_LIBS=1
ENV PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=1
ENV PKG_CONFIG_PATH=/usr/lib/pkgconfig:/usr/share/pkgconfig

# Set the working directory and copy Cargo.toml separately for caching dependencies
WORKDIR /app

# Copy the source code only after dependencies are fetched
COPY . .

# Build the project with static linking
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:latest
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/exchange-rate-bot /app/exchange-rate-bot
WORKDIR /app
CMD ["./exchange-rate-bot"]
