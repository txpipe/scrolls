<div align="center">
    <img src="./assets/logo-dark.png?sanitize=true#gh-dark-mode-only" alt="Scrolls Logo" width="500">
    <img src="./assets/logo-light.png?sanitize=true#gh-light-mode-only" alt="Scrolls Logo" width="500">
    <hr />
        <h3 align="center" style="border-bottom: none">Read-optimized cache of Cardano on-chain entities</h3>
        <img alt="GitHub" src="https://img.shields.io/github/license/txpipe/scrolls" />
        <img alt="GitHub Workflow Status" src="https://img.shields.io/github/workflow/status/txpipe/scrolls/Validate" />
    <hr/>
</div>

## Intro

_Scrolls_ is a tool for building and maintaining read-optimized collections of Cardano's on-chain entities. It crawls the history of the chain and aggregates all data to reflect the current state of affairs. Once the whole history has been processed, _Scrolls_ watches tip of the chain to keep the collections up-to-date.

Examples of collections are: "utxo by address", "chain parameters by epoch", "pool metadata by pool id", "tx cbor by hash", etc.

> In other words, _Scrolls_ is just a map-reduce algorithm that aggregates the history of the chain into use-case-specific, key-value dictionaries.

:warning: this tool is under heavy development. Library API, configuration schema and storage structure may vary drastrically. Use at your own peril.

## Storage

Storage backends are "pluggable", any key-value storage mechanism is a potential candidate. Our backend of preference is Redis (and TBH, the only one implemented so far). It provides very high "read" throughput, it can be shared across the network by multiple clients and can be used in cluster-mode for horizontal scaling.

We also understand that a memory db like Redis may be prohibitive for some use-cases where storage optimization is more important than read-latency. The goal is to provide other backend options within the realm of NoSQL databases better suited for the later scenarios.

## About CRDTs

The persistence data model does heavy use of CRDTs (Conflict-free replicated data types) and idempotent calls, which provide benefits for write concurrency and rollback procedures.

For example, CRDTs allows us to re-build the indexes by spawning several history readers that crawl on-chain data concurrently from different start positions. This provides a sensible benefit on bootstrap procedures.

TODO: explain future plan to leverage CRDTs for rollback checkpoints.

## Accesing the Data

_Scrolls_ doesn't provide any custom client for accesing the data, it relies on the fact that the canonical clients of the selected backends are ubiquotos, battle-tested and relatively easy to use. By knowing the structure of the stored keys/values, a developer should be able to query the data directly from Redis.

TODO: reference specs for each key/value

## Filtering

Not all data is important in every scenario. Every collection in _Scrolls_ is disabled by default and needs to be opted-in via configuration for it to be processed and stored. Some collections are more storage-hungry than others (eg: "Block CBOR by Hash"), plan ahead to avoid resource exhaustion.

Within the scope of a particular collection, further filtering can be specified depending on the nature of the data being aggregated. For example, the "UTXOs by Address" collection can be filtered to only process UTXO from a set of predetermined addresses.

TODO: Document filtering options per collection

## Feature Status

- [ ] Collections
  - [x] UTXOs by Address
  - [ ] Address by UTXO
  - [ ] Tx CBOR by UTXO
  - [ ] Tx CBOR by Address
  - [ ] Pool Id by Stake Address
  - [ ] Pool Metadata by Pool Id
  - [ ] Chain Parameters by Epoch
  - [ ] UTXOs by Asset
  - [x] Block Hash by Tx Hash
  - [ ] Block Hashes by Epoch
  - [ ] Block Header by Block Hash
  - [ ] Tx Hashes by Block Hash
  - [ ] Ada Handle by Address
  - [ ] Address by Ada Handle
  - [ ] Block CBOR by Hash
  - [ ] Tx CBOR by Hash
  - [ ] Feature requests open
- [ ] Data Sources
  - [x] Node-to-Node ChainSync + Blockfetch
  - [ ] Node-to-Client ChainSync
  - [ ] Oura Kafka Topic
  - [ ] Raw-CBOR Block files
- [ ] Storage Backend
  - [x] Redis
  - [ ] MongoDB
  - [ ] AWS DynamoDB
  - [ ] GCP BigQuery
  - [ ] Firestore
  - [ ] Azure CosmoDB
  - [ ] Feature requests open

## Configuration

This is an example configuration file:

```toml
# get data from a relay node
[source]
type = "N2N"
address = "relays-new.cardano-mainnet.iohk.io:3001"

# enable the "UTXO by Address" collection
[[reducers]]
type = "UtxoByAddress"
# you can optionally prefix the keys in the collection
key_prefix = "c0"

# enable the "Point by Tx" collection
[[reducers]]
type = "PointByTx"
key_prefix = "a1"

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

## Testdrive

In the `testdrive` folder you'll find a minimal example that uses docker-compose to spin up a local Redis instance and a Scrolls daemon. You'll need Docker and docker-compose installed in you local machine. Run the following commands to start it:

```sh
cd testdrive
docker-compose up
```

You should see the logs of both _Redis_ and _Scrolls_ crawling the chain from a remote relay node. If you're familiar with Redis CLI, you can run the following commands to see the data being cached:

TODO

## FAQ

> Don't we have tools for this already?

Yes, we do. We have excelent tools like Kupo, db-sync, dcSpark's oura-db-sync. Even the Cardano node itself might work as a source for some of the collections. Every tool is architected with a set of well-understood trade-offs. We believe _Scrolls_ makes sense as an addition to the list because assumes a particular set of trade-offs:

- network storage over local storage: _Scrolls_ makes sense if you have multiple distributed clients working in a private network that want to connect to the same data instance.
- read latency over data normalization: _Scrolls_ works well when you need to answer simple questions, like a lookup table. It won't work if you need to create joins or complex relational queries.
- data cache over data source: _Scrolls_ aims at being a "cache" of data, not the actual source of data. It has emphasis on easy and fast reconstruction of the collections. It promotes workflows where the data is wiped and rebuilt from scratch whenever the use-case requires (such as adding / removing filters).
- Rust over Haskell: this is not a statement about the languages, both are great languages, each one with it's own set of trade-offs. Since most of the Cardano ecosystem is written in Haskell, we opt for Rust as a way to broaden the reach to a different community of Rust developers (such as the authors of this tool). _Scrolls_ is extensible, it can be used as a library in Rust projects to create custom cache collections.
- bring your own db: storage mechanism in _Scrolls_ are pluggable, our goal is to provide a tool that plays nice with existing infrastructure. The trade-off is that you end up having more moving parts.

> How do I read the data using Python?

TODO

> How do I read the data using NodeJS?

TODO