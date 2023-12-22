## Setup Postgres DB
```bash
docker compose up -d
```

## Run scrolls
```bash
RUST_BACKTRACE=1 cargo run --bin scrolls --features deno -- daemon --config examples/deno-postgres/daemon.toml
```