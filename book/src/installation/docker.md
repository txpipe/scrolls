# Docker

_Scrolls_ provides pre-built public Docker images through github packages. 

## Pre-requisites

- [Install Docker](https://docs.docker.com/engine/install/)

## Download, install and test

The command below will download the latest docker image and show _Scrolls's_ command-line help message.

```sh
docker run ghcr.io/txpipe/scrolls:latest
```

See the [usage](../usage/index.md) page for running _Scrolls_.

## Versioned Images

Images are also tagged with the corresponding version number. It is highly recommended to use a fixed image version in production environments to avoid the effects of new features being included in each release (please remember Scrolls hasn't reached v1 stability guarantees).

To use a versioned image, replace the `latest` tag by the desired version with the `v` prefix. For example, to use version `0.5.0`, use the following image:

```
ghcr.io/txpipe/scrolls:v0.5.0
```