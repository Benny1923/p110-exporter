FROM ubuntu

ARG BIN_PATH=target/release/p110-exporter

WORKDIR /app

COPY $BIN_PATH /app/p110-exporter

CMD ["sh", "-c", "/app/p110-exporter"]