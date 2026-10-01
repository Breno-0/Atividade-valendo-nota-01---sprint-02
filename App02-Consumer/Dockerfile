FROM rust:1-slim-bookworm AS builder

WORKDIR /app

COPY rust-amqp/Cargo.toml rust-amqp/Cargo.lock ./

RUN mkdir -p src/bin \
    && echo "fn main() {}" > src/main.rs \
    && echo "fn main() {}" > src/bin/receive.rs \
    && cargo build --release --bin receive \
    && rm -rf src

COPY rust-amqp/src ./src

RUN touch src/bin/receive.rs && cargo build --release --bin receive

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/receive .

CMD ["./receive"]
