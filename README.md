service-gatekeeper
==================
[<img src="https://img.shields.io/docker/cloud/build/paperwork/service-gatekeeper.svg?style=for-the-badge"/>](https://hub.docker.com/r/paperwork/service-gatekeeper)

Gatekeeper Service

## Building

```bash
cargo build
```

## Running

### Locally

```bash
CONFIG_JSON=$(cat ./config.json | jq -c -r) ./target/debug/gatekeeper
```

### Docker

```bash
docker run -it --rm --name service-gateway --env $(cat ./config.json | jq -c -r) paperwork/service-gateway
```
