FROM ubuntu

WORKDIR /app

COPY target/release/p110-exporter /app/p110-exporter

CMD ["sh", "-c", "/app/p110-exporter"]