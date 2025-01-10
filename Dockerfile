# Use a build argument to specify the base image
ARG BASE_IMAGE=exchange-rate-bot:builder-latest

# Build stage
FROM ${BASE_IMAGE} AS builder

ARG APP_VERSION=DOCKER_UNKNOWN
ENV APP_VERSION=${APP_VERSION}

# Set the working directory and copy Cargo.toml separately for caching dependencies
WORKDIR /app

# Copy the source code only after dependencies are fetched
COPY . .

RUN cargo build --release --verbose

# Runtime stage
FROM alpine:3.21

# Install runtime dependencies
RUN apk update && apk add --no-cache fontconfig freetype libgcc \
  font-terminus font-inconsolata font-noto

# Copy the Red Hat Display font from the builder
# COPY --from=builder /usr/share/fonts/red-hat-display /usr/share/fonts/red-hat-display

# Update font cache
RUN fc-cache -f -v

# Copy the built application from the builder stage
COPY --from=builder /app/target/release/exchange-rate-bot /app/exchange-rate-bot

# Set the working directory
WORKDIR /app

# Command to run the application
CMD ["./exchange-rate-bot"]
