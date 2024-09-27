FROM rust:1.81.0 AS builder

WORKDIR /usr/src/moquist

COPY . .

RUN cargo test

RUN cargo install --path .

FROM debian:bookworm-slim

LABEL org.opencontainers.image.source="https://github.com/Netnix-SA/moquist"

RUN apt update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/moquist /usr/local/bin/moquist

ENTRYPOINT ["moquist"]
