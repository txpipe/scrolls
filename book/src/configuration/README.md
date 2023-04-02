# Configuration
For the purpose of testing out Scrolls you can use the provided configuration located in `testdrive/simple/daemon.toml`. See below for another example with explanations and check the following sections of this book to understand in detail each section of the configuration file.

## Format
Scrolls daemon supports `.toml` and `.json` configuration files. Unlike json, toml supports comments which are very handy to turn declarations on and off, specially during early stages of development, debugging, learning, etc. On the other hand, deeply nested filters can be difficult to understand using toml syntax, so the user can choose to declare the whole configuration with json, or instead to rely on tools like [toml2json](https://github.com/woodruffw/toml2json) and [remarshal](https://github.com/remarshal-project/remarshal) to translate small chunks of json (such as complex deeply nested filters) to be used in toml configuration files.

When working with toml configuration files, sometimes it also helps to translate the whole configuration to json, and use [jq](https://stedolan.github.io/jq/)/[bat](https://github.com/sharkdp/bat) to make the json human friendly. This often helps to understand the structure of the filters. Example: `toml2json ./configuration.toml | jq | bat -l json` 

## Configuration Example
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

