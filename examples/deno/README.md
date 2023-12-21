## Transpile and bundle reducer code
```bash
deno run --allow-read --allow-net --allow-env --allow-run --allow-write  build.ts
```

## Setup Redis DB
```bash
docker compose up -d
```

## Run scrolls
```bash
RUST_BACKTRACE=1 cargo run --features deno --bin scrolls -- daemon --config ./examples/deno/daemon.toml
```