# Quill

[Quill](https://github.com/alethea-io/quill) is a library of reducers written in Javascript that are compatible with Scrolls.

## Setup Redis DB
```bash
docker compose up -d
```

## Run scrolls
```bash
RUST_BACKTRACE=1 cargo run --features deno --bin scrolls -- daemon --config ./examples/quill/daemon.toml
```