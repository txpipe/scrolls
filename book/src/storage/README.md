# Storage

Storage backends are "pluggable", any key-value storage mechanism is a potential candidate. Our backend of preference is Redis (and TBH, the only one implemented so far). It provides a very high "read" throughput, it can be shared across the network by multiple clients and can be used in cluster-mode for horizontal scaling.


## Available backends:

- [Redis](./redis.md)


