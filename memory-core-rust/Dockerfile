FROM rust:1.82-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY proto ../proto/
COPY build.rs .
COPY src/ src/

RUN apt-get update && apt-get install -y protobuf-compiler && \
    cargo build --release && \
    rm -rf /var/lib/apt/lists/*

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/memory-server /usr/local/bin/memory-server
CMD ["memory-server"]
