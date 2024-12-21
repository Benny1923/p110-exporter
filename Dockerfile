FROM rust:1.83 AS build

WORKDIR /usr/src/p110-exporter
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo build --release

FROM ubuntu

COPY --from=build /usr/src/p110-exporter/target/release/p110-exporter /usr/bin/p110-exporter

CMD ["sh", "-c", "p110-exporter"]
