# Build stage
FROM rust:alpine3.20 as builder

# Add build dependencies
RUN apk update && \
    apk add --no-cache build-base pkgconf openssl-dev musl-dev cmake make perl clang16 curl

# Set environment variables for static linking
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr

# Copy the source code into /app
WORKDIR /app
COPY . .

# Build the project with static linking
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
# Use scratch, an empty image, for a minimal final image
FROM scratch

# Copy the static binary from the builder
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/exchange-rate-bot /app/exchange-rate-bot

# Set the working directory and start the application
WORKDIR /app
CMD ["./exchange-rate-bot"]
