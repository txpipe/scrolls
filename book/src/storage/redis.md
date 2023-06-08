# Redis

Storage stage that collects metrics into a Redis database.

## Installation

You can quickly bring up an instance of Redis using docker.

### Docker cli

```sh
docker run -p 6379:6379 -v ./data/redis:/data redis
```

### Docker compose

```yaml
version: "3.7"

services:
  redis:
    image: redis
    volumes:
      - ./data/redis:/data
    ports:
      - "6379:6379"
```

```sh
docker compose up -d redis
```

### Other

For more installation options please see the [official documentation](https://redis.io/docs/getting-started/installation/).

## Configuration

```toml
[storage]
type = "Redis"
connection_params = "redis://localhost:6379"
cursor_key = "_cursor"
```

- `type` (Redis): the literal value `Redis`.
- `connection_params` (string): the connection path to your Redis instance
- `cursor_key` (string): the name of the Redis key containing the value of the _Scrolls_ cursor. Defaults to `_cursor`.

## Querying Redis

A quick and easy way to query Redis for testing and debugging purposes is through the `redis-cli`. This command line utility comes package with a [local install](https://redis.io/docs/getting-started/installation/) of Redis.

Test Redis is running:

```sh
redis-cli ping
```

Query all keys in the database:

```sh
redis-cli keys
```

Query a _Scrolls_ key-value pair:

```sh
redis-cli get _cursor
```

Query a _Scrolls_ sorted set:

```sh
redis-cli zrange addresses_by_asset.addr1wypty6mmh9ys7hn7k4pmy5exv8qj5u0lvr7gt7e6sw76cus8q85le 0 10 WITHSCORES
```

# Kvrocks

For certain _Scrolls_ reducers, the memory needs can quickly become prohibitive when using Redis. To reduce the memory requirements of _Scrolls_ (at the cost of read latency), it is possible to use [Kvrocks](https://github.com/apache/incubator-kvrocks) instead. 

## On-disk vs in-memory

Kvrocks maintains a key value store on-disk rather than in memory. This allows _Scrolls_ to scale to hundreds of gigabytes of storage capacity while still being fast to query.

From the Kvrocks repo:

"Apache Kvrocks is a distributed key value NoSQL database that uses RocksDB as storage engine and is compatible with Redis protocol. Kvrocks intends to decrease the cost of memory and increase the capacity as compared to Redis."

## Installation

You can quickly bring up an instance of Kvrocks using docker.

### Docker command

```sh
docker run -p 6666:6666 -v ./data/kvrocks:/var/lib/kvrocks apache/kvrocks
```

### Docker compose

```yaml
version: "3.7"

services:
  kvrocks:
    image: apache/kvrocks
    volumes:
      - ./data/kvrocks:/var/lib/kvrocks
    ports:
      - 6666:6666
```

```sh
docker compose up -d kvrocks
```
### Other

For more installation options please see the [official documentation](https://kvrocks.apache.org/docs/getting-started/).

## Configuration

```toml
[storage]
type = "Redis"
connection_params = "redis://localhost:6666"
cursor_key = "_cursor"
```