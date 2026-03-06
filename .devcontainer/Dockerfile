FROM rust:bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates pkg-config \
    sqlite3 curl zsh vim \
    git gnupg2 jq sudo \
    build-essential openssl \
    protobuf-compiler \
    libopus-dev \
    cmake clang \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# Use nightly to match the production build
RUN rustup default nightly && rustup update
RUN rustup component add rustfmt clippy

WORKDIR /home/

COPY setup.sh .

RUN bash ./setup.sh
