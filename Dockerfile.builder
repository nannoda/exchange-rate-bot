# Build stage
FROM rust:alpine3.21 AS builder

ARG APP_VERSION=DOCKER_UNKNOWN
ENV APP_VERSION=${APP_VERSION}

# Add dependencies in a single RUN command to reduce layers
RUN apk update && apk add --no-cache \
  alpine-sdk \
  build-base \
  pkgconf \
  musl-dev \
  cmake make \
  perl \
  clang18 \
  curl \
  strace \
  sqlite-dev sqlite-static \
  fontconfig-dev fontconfig-static \
  freetype-dev freetype-static \
  libpng-dev libpng-static \
  zlib-dev zlib-static \
  brotli-dev brotli-static \
  bzip2-dev bzip2-static \
  libxml2-dev libxml2-static \
  expat-dev expat-static \
  libjpeg-turbo-dev libjpeg-turbo-static \
  unzip

# Set environment variables for static linking
# ENV OPENSSL_STATIC=1
# ENV OPENSSL_DIR=/usr
ENV PKG_CONFIG_ALLOW_SYSTEM_LIBS=1
ENV PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=1
ENV RUSTFLAGS="-C target-feature=-crt-static -L /usr/lib -lxml2 -lpng -lbz2 -lz -lbrotlidec -lbrotlienc -lfreetype -lfontconfig"
ENV PKG_CONFIG_PATH=/usr/lib/pkgconfig:/usr/share/pkgconfig

# Verify the presence of static libraries
RUN ls -al /usr/lib | grep -E 'libfontconfig|libfreetype|libpng|libz|libbz2|libbrotli|libxml2'

# Verify fontconfig and freetype installation
RUN pkg-config --modversion fontconfig freetype2 && \
  ls -al /usr/lib | grep libfontconfig && \
  ls -al /usr/lib | grep libfreetype

# Set the working directory and copy Cargo.toml separately for caching dependencies
WORKDIR /app

RUN echo "fn main() {}" >> dummy.rs
COPY Cargo.toml .
COPY Cargo.lock .

RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
# Copy the source code only after dependencies are fetched

RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml
