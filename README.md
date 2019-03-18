service-gatekeeper
==================
[<img src="https://img.shields.io/docker/cloud/build/paperwork/service-gatekeeper.svg?style=for-the-badge"/>](https://hub.docker.com/r/paperwork/service-gatekeeper)

Gatekeeper Service

## Building

```bash
cargo build
```

## Running

```bash
PORT=1337 JWT_SECRET='ru4XngBQ/uXZX4o/dTjy3KieL7OHkqeKwGH9KhClVnfpEaRcpw+rNvvSiC66dyiY' SERVICE_USERS="http://localhost:8880" ./target/debug/gatekeeper
```
