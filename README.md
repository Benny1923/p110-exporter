# P110 exporter

tapo powerplug metrics for prometheus

## Deploy

docker-compose.yml (recommend)
```yaml
services:
  p110-exporter:
    container_name: p110-exporter
    image: ghcr.io/benny1923/p110-exporter:latest
    ports:
      - 9200:9200
    volumes:
      - /path/to/config.yml:/config.yml
    restart: unless-stopped
```


## Example Config

see [config-example.yml](config-example.yml)

## Metrics

- tapo_energy_usage
- tapo_device_on
- tapo_request_fail