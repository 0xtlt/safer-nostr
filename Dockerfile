# 1. This tells docker to use the Rust official image
FROM rust:1-slim-buster as build

# Install openssl
RUN apt-get update && apt-get install -y openssl libssl-dev pkg-config

# Copy only cargo.toml and cargo.lock to cache dependencies
RUN USER=root cargo new --bin safer-nostr
WORKDIR /safer-nostr

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build --release
RUN rm src/*.rs

# 2. Copy the files in your machine to the Docker image
COPY ./src ./src

# Build your program for release
RUN cargo build --release

FROM rust:1-slim-buster
COPY --from=build /safer-nostr/target/release/safer-nostr ./safer-nostr

CMD ["./safer-nostr"]