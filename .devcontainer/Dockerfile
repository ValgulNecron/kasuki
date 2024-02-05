FROM rust:bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates pkg-config \        
    sqlite3 curl zsh vim \
    git gnupg2 jq sudo \
    build-essential openssl \
    && rm -rf /var/lib/apt/lists/*
    
WORKDIR /home/

COPY . .

RUN bash ./setup.sh

ENV PATH="/root/.cargo/bin:$PATH"
