FROM rust:1-bookworm AS builder

WORKDIR /usr/src/spangen
COPY src src
COPY Cargo.lock Cargo.toml ./

RUN cargo install --path .

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y kafkacat && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/spangen /usr/local/bin/spangen
COPY docker-entrypoint.sh /usr/local/bin/

ENV KAFKA_BROKER=kafka-001:9092
ENV KAFKA_TOPIC=snuba-spans
ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
