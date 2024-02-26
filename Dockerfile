# Use the official Rust image as a base
FROM rust:slim-bookworm AS builder

# Create a new empty project
RUN USER=root cargo new --bin kasuki
WORKDIR /kasuki

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy over your manifests
COPY ./Cargo.toml ./Cargo.toml

# This dummy build helps to cache your dependencies
RUN mkdir -p ./src && \
    echo 'fn main() { println!("dummy") }' > ./src/main.rs && \
    cargo build --release && \
    rm -rf ./src

# Now copy your actual source code
COPY ./src ./src

# Build for release. Dependencies will be reused from the previous build
RUN cargo build --release

# Start a new stage
FROM debian:trixie-slim AS bot

# Set labels
LABEL maintainer="valgul"
LABEL author="valgul"

# Set the working directory
WORKDIR /kasuki/

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy over the built binary file from the builder stage
COPY --from=builder /kasuki/target/release/kasuki /kasuki/

# Copy other necessary files
COPY json /kasuki/json
COPY server_image /kasuki/server_image

# Set the command to run your application
CMD ["./kasuki"]