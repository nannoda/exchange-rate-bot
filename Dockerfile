# Build stage
FROM rust:alpine3.20 AS builder

# Add build dependencies
RUN apk update
RUN apk add build-base
RUN apk add pkgconf openssl-dev musl-dev cmake make perl clang16 curl strace
RUN apk add musl-dev
RUN apk add openssl-dev
RUN apk add openssl-libs-static

# Set environment variables for static linking
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr

# Copy the source code into /app
WORKDIR /app
COPY . .

# Build the project with static linking
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/exchange-rate-bot /app/exchange-rate-bot
WORKDIR /app
CMD ["./exchange-rate-bot"]
