FROM rust:alpine3.19 as builder

# Add build dependencies
RUN apk update
RUN apk add build-base
RUN apk add pkgconf openssl-dev musl-dev cmake make perl clang16 curl strace
RUN apk add g++ gcc
RUN apk add musl-dev
RUN apk add openssl-dev
RUN apk add openssl-libs-static

# Set environment variables
ENV OPENSSL_DIR=/usr
ENV OPENSSL_STATIC=1

# Copy . to /app
COPY . /app

# Change working directory to /app
WORKDIR /app

# Build the project
RUN cargo build --release

# Create build directory
RUN mkdir -p /build

# Copy the binary to /build
RUN cp target/release/exchange-rate-bot /build

# Create a new image from alpine
FROM alpine:3.19

# RUN apk add bash bash-completion
# RUN apk add util-linux pciutils hwdata-pci usbutils hwdata-usb coreutils binutils findutils grep iproute2

RUN mkdir /app

# Copy the binary from the builder stage to /app
COPY --from=builder /build/exchange-rate-bot /app

# Change working directory to /app
WORKDIR /app

# Run the binary
CMD ["./exchange-rate-bot"]
