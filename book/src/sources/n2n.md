# Node-to-Node

The Node-to-Node (N2N) source uses Ouroboros mini-protocols to connect to a local or remote Cardano node through a tcp socket bearer and fetches block data using the ChainSync mini-protocol instantiated to "headers only" and the BlockFetch mini-protocol for retrieval of the actual block payload.

## Configuration

```toml
[source]
type = "N2N"
address = "<hostname:port>"
min_depth = <int>

[chain]
type = "<chain_name>"

[intersect]
type = "<intersection_type>"
value = [<absolute_slot>, "<block_hash>"]

[finalize]
until_hash = "<block_hash>"
```

### Section `source`:

Options for configuring the connection to a remote cardano node

- `type` (N2N): this field must be set to the literal value `N2N`
- `address` (string): a string describing the location of the tcp endpoint. It must be specified as a string with hostname and port number.
- `min_depth` (int): the minimum number of blocks the indexer should stay behind the tip of the chain. The larger the depth, the higher the data latency but the lower the chance of encountering a corrupting rollback.

### Section `chain`:

Option for selecting the desired chain

- `type` (mainnet | testnet | preprod | preview)

### Section `intersect`:

Advanced options for indicating which point in the chain to start reading from. Read the [intersect options](../advanced/intersect_options.md) documentation for detailed information.

### Section `finalize`:

Option for stopping the indexer at a specified point (i.e. block). If not set, the indexer will run indefinitely. TODO: Is the point inclusive?

- `until_hash` (string): block hash to stop at

## Examples

Connecting to a remote Cardano node in mainnet and starting from beginning of the chain:

```toml
[source]
type = "N2N"
address = "relays-new.cardano-mainnet.iohk.io:3001"
min_depth = 10

[chain]
type = "mainnet"

[intersect]
type = "origin"
```

Connecting to a remote Cardano node in testnet and starting from the tip of the chain:

```toml
[source]
type = "N2N"
address = "relays-new.cardano-mainnet.iohk.io:3001"
min_depth = 10

[chain]
type = "testnet"

[intersect]
type = "tip"
```

Start reading from a particular point in the chain and stop at a different point:

```toml
[source]
type = "N2N"
address = "relays-new.cardano-mainnet.iohk.io:3001"
min_depth = 10

[chain]
type = "mainnet"

[intersect]
type = "Point"
value = [48896539, "5d1f1b6149b9e80e0ff44f442e0cab0b36437bb92eacf987384be479d4282357"]

[finalize]
until_hash = "5c066b37ca345d225f8b80808ddb46c38ee13d59c4b44a28f145f1d6f0c2028e"
```