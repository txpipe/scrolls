# Storage

Storage backends are "pluggable", any key-value storage mechanism is a potential candidate. Our backend of preference is Redis (and TBH, the only one implemented so far). It provides a very high "read" throughput, it can be shared across the network by multiple clients and can be used in cluster-mode for horizontal scaling.

In case you don't have a redis instance running in your system, you might want to look at our [Testdrive](../guides/testdrive.md) guide for a minimal docker example providing a local redis instance. You can also look at the official Redis [Installation guide](https://redis.io/docs/getting-started/installation/).

See our [Redis-cli basics](../guides/redis.md) guide to get started playing with Redis from the command line.

### Fields

- type: `"Redis" | "Skip"` 
- connection_params (*): `String`
- cursor_key (*): `Option<String>`

(*) Use only with `type = "Redis"`

#### Example

``` toml
[storage]
type = "Redis"
connection_params = "redis://127.0.0.1:6379"
```
