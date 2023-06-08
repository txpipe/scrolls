# Usage

_Scrolls_ currently offers only a single mode of operation called daemon mode.

## Daemon Mode

In daemon mode, _Scrolls_ will run as a continuous process consuming and reducing new blocks.

Available options:
- `--config` (string): path to a custom toml file. If not specified, _Scrolls_ will look for a `scrolls.toml` file in the current working directory. If that is not found, it will search for a file at `/etc/scrolls/daemon.toml`.
- `--console` (plain | tui): option to set how the terminal output will be displayed. In plain mode, pipeline events are printed as a plain sequence of logs. In TUI mode, events are shown as an aggregated progress bar and series of counters. The default is plain mode.

### Pre-requisites

Before running _Scrolls_ make sure you have created and configured a `daemon.toml` file.

Here is a minimal daemon file as an example:

```toml
[source]
type = "N2N"
address = "relays-new.cardano-mainnet.iohk.io:3001"

[chain]
type = "Mainnet"

[intersect]
type = "Origin"

[storage]
type = "Redis"
connection_params = "redis://redis:6379"

[[reducers]]
type = "PoolByStake"

[policy]
missing_data = "Skip"
```

In addition, make sure you have installed and started a [storage service](../storage/index.md). For the daemon configuration above, we are using Redis which can be brought up quickly via docker:

```sh
docker run -p 6379:6379 -v ./data/redis:/data redis
```

### Run from source installation

One way to run via source install, is through `cargo run`.

Assuming your directory structure is as follows:

```bash
scrolls/
    Cargo.toml
my-app/
    daemon.toml
```

And you are in the `my-app` directory, your command is:

```sh
# my-app

RUST_LOG=info cargo run \
    --manifest-path ../scrolls/Cargo.toml \
    --all-features \
    --bin scrolls -- daemon --console plain --config ./daemon.toml
```

Provide the path to the scrolls `Cargo.toml` file as the `--manifest-path` if running outside the scrolls directory.

### Run from binary installation

If the path to the scrolls binary is in your `$PATH` variable you can use:

```sh
scrolls daemon --console plain --config ./daemon.toml
```

Otherwise you need to specify the path to the binary:

```sh
/path/to/scrolls daemon --console plain --config ./daemon.toml
```

### Run using docker

Run using docker cli:

```sh
docker run -v ./daemon.toml:/etc/scrolls/daemon.toml ghcr.io/txpipe/scrolls:latest daemon --console plain
```

Run using docker compose:

```yaml
version: "3.7"

services:
  scrolls:
    image: ghcr.io/txpipe/scrolls:latest
    command: [ "daemon", "--console", "plain"]
    environment:
      - RUST_LOG=info
    volumes:
      - ./daemon.toml:/etc/scrolls/daemon.toml
```