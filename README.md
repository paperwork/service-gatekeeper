service-gatekeeper
==================
[<img src="https://img.shields.io/docker/cloud/build/paperwork/service-gatekeeper.svg?style=for-the-badge"/>](https://hub.docker.com/r/paperwork/service-gatekeeper)

Gatekeeper Service

## Prerequisites

### Docker

Get [Docker Desktop](https://www.docker.com/products/docker-desktop).

### Rust

On MacOS using [brew](https://brew.sh):

```bash
% brew install rust
```

### Paperwork local development environment

Please refer to the [documentation](https://github.com/paperwork/paperwork/#local-development-environment).

## Building

Fetching all dependencies and compiling:

```bash
% make local-build-develop
```

## Running

**Note:** Before starting this service the local development environment needs to be running!

```bash
% make local-run-develop
```
