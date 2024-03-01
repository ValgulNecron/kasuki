# This Dockerfile is used to build and run the Kasuki bot.

# Use the official Rust image as a base
# This image includes all the necessary tools to compile a Rust project.
FROM rust:slim-bookworm AS builder

# Create a new empty project
# This is done as root to avoid permission issues.
RUN USER=root cargo new --bin kasuki
WORKDIR /kasuki

# Install system dependencies
# These are required for the Kasuki bot to function correctly.
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy over your manifests
# This includes the Cargo.toml file which specifies the Rust dependencies.
COPY ./Cargo.toml ./Cargo.toml

# This dummy build helps to cache your dependencies
# It allows Docker to cache the dependencies separately from the source code.
RUN mkdir -p ./src && \
    echo 'fn main() { println!("dummy") }' > ./src/main.rs && \
    cargo build --release && \
    rm -rf ./src

# Now copy your actual source code
# This is done after the dummy build to take advantage of Docker's layer caching.
COPY ./src ./src

# Build for release. Dependencies will be reused from the previous build
# This compiles the Kasuki bot for release.
RUN cargo build --release

# Start a new stage
# This is a multi-stage build. The previous stage was used to compile the bot.
# This stage is used to create the final image that will be run.
FROM debian:trixie-slim AS bot

# Set labels
# These provide metadata about the image.
LABEL maintainer="valgul"
LABEL author="valgul"

# Set the working directory
# This is where the Kasuki bot will be run from.
WORKDIR /kasuki/

# Install system dependencies
# These are required for the Kasuki bot to function correctly.
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy other necessary files
# These include JSON files and server images used by the Kasuki bot.
COPY json /kasuki/json
COPY server_image /kasuki/server_image

# Copy over the built binary file from the builder stage
# This is the compiled Kasuki bot.
COPY --from=builder /kasuki/target/release/kasuki /kasuki/

# Set the command to run your application
# This is the command that will be run when a container is started from this image.
CMD ["./kasuki"]