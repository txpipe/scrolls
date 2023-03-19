# Usage

Once you have Scrolls and Redis installed in your system (see [Installation](../installation/index.md)) you can start the daemon with the following command:

``` bash
/path/to/scrolls daemon --config /path/to/config.toml
```

If you have no experience using Redis, you can follow the [Redis-cli guide](../guides/redis.md) to learn the very basics needed to fetch data from redis using the command line application provided by Redis. For any real application you would need to use a client library in your language of preference. See our [Python](../guides/python.md) and [NodeJS](../guides/nodejs.md) guides.

## Configuration
Scrolls daemon must be configured using a single `.toml` file. For the purpose of testing out Scrolls you can use the provided configuration located in `testdrive/simple/daemon.toml`. See below for an example config with examplantions.

```toml
# get data from a relay node
[source]
type = "N2N"
address = "relays-new.cardano-mainnet.iohk.io:3001"

# You can optionally enable enrichment (local db with transactions), this is needed for some reducers
[enrich]
type = "Sled"
db_path = "/opt/scrolls/sled_db"

# enable the "UTXO by Address" collection
[[reducers]]
type = "UtxoByAddress"
# you can optionally prefix the keys in the collection
key_prefix = "c1"
# you can optionally only process UTXO from a set of predetermined addresses
filter = ["addr1qy8jecz3nal788f8t2zy6vj2l9ply3trpnkn2xuvv5rgu4m7y853av2nt8wc33agu3kuakvg0kaee0tfqhgelh2eeyyqgxmxw3"]

# enable the "Point by Tx" collection
[[reducers]]
type = "PointByTx"
key_prefix = "c2"

# store the collections in a local Redis
[storage]
type = "Redis"
connection_params = "redis://127.0.0.1:6379"

# start reading from an arbitrary point in the chain
[intersect]
type = "Point"
value = [57867490, "c491c5006192de2c55a95fb3544f60b96bd1665accaf2dfa2ab12fc7191f016b"]

# let Scrolls know that we're working with mainnet
[chain]
type = "Mainnet"
```
### The source section
This section specifies the origin of the data. The special type field must always be present and containing a value matching any of the available built-in sources. The rest of the fields in the section will depend on the selected type. See the [Sources](../sources/index.md) section for a list of available options and their corresponding config values.
### The enrich section
### The reducers section
See the [Reducers](../reducers/index.md) section of the manual.
### The storage section
See the [Storage](../storage/index.md) section of the manual.
### The intersect section
### The chain section
