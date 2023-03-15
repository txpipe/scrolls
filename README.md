<div align="center">
    <img src="./assets/logo-dark.png?sanitize=true#gh-dark-mode-only" alt="Scrolls Logo" width="500">
    <img src="./assets/logo-light.png?sanitize=true#gh-light-mode-only" alt="Scrolls Logo" width="500">
    <hr />
        <h3 align="center" style="border-bottom: none">Read-optimized cache of Cardano on-chain entities</h3>
        <img alt="GitHub" src="https://img.shields.io/github/license/txpipe/scrolls" />
        <img alt="GitHub Workflow Status" src="https://img.shields.io/github/workflow/status/txpipe/scrolls/Release" />
    <hr/>
</div>

## Introduction

_Scrolls_ is a tool for building and maintaining read-optimized collections of Cardano's on-chain entities. It crawls the history of the chain and aggregates all data to reflect the current state of affairs. Once the whole history has been processed, _Scrolls_ watches the tip of the chain to keep the collections up-to-date.

Examples of collections are: "utxo by address", "chain parameters by epoch", "pool metadata by pool id", "tx cbor by hash", etc.

> In other words, _Scrolls_ is just a map-reduce algorithm that aggregates the history of the chain into use-case-specific, key-value dictionaries.

Check our [documentation](https://txpipe.github.io/scrolls) for detailed information on how to start working with Scrolls.

:warning: this tool is under heavy development. Library API, configuration schema and storage structure may vary drastically. Several important features are still missing. Use at your own peril.

## Storage

Storage backends are "pluggable", any key-value storage mechanism is a potential candidate. Our backend of preference is Redis (and TBH, the only one implemented so far). It provides a very high "read" throughput, it can be shared across the network by multiple clients and can be used in cluster-mode for horizontal scaling.

We also understand that a memory db like Redis may be prohibitive for some use-cases where storage optimization is more important than read-latency. The goal is to provide other backend options within the realm of NoSQL databases better suited for the later scenarios.

## About CRDTs

The persistence data model does heavy use of [CRDTs](https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type) (Conflict-free replicated data types) and idempotent calls, which provide benefits for write concurrency and rollback procedures.

For example, CRDTs allow us to re-build the indexes by spawning several history readers that crawl on-chain data concurrently from different start positions. This provides a sensible benefit on collection-building time. We call this approach "swarm mode".

TODO: explain future plan to leverage CRDTs for rollback checkpoints.

## Accessing the Data

_Scrolls_ doesn't provide any custom client for accessing the data, it relies on the fact that the canonical clients of the selected backends are ubiquitous, battle-tested and relatively easy to use. By knowing the structure of the stored keys/values, a developer should be able to query the data directly from Redis.

TODO: reference specs for each key/value

## Filtering

Not all data is important in every scenario. Every collection in _Scrolls_ is disabled by default and needs to be opted-in via configuration for it to be processed and stored. Some collections are more storage-hungry than others (eg: "Block CBOR by Hash"), plan ahead to avoid resource exhaustion.

Within the scope of a particular collection, further filtering can be specified depending on the nature of the data being aggregated. For example, the "UTXOs by Address" collection can be filtered to only process UTXO from a set of predetermined addresses.

TODO: Document filtering options per collection

## How it Works

Scrolls is a pipeline that takes block data as input and outputs DB update commands. The stages involved in the pipeline are the following:

- Source Stages: are in charge of pulling data block data. It might be directly from a Cardano node (local or remote) or from some other source. The requirement is to have the raw CBOR as part of the payload.
- Reducer Stages: are in charge of applying the map-reduce algorithm. They turn block data into CRDTs commands that can be later merged with existing data. The map-reduce logic will depend on the type of collection being built. Each reducer stage handles a single collection. Reducers can be enabled / disabled via configuration.
- Storage Stages: receive the generic CRDT commands and turns them into DB-specific instructions that are then executed by the corresponding engine.

![diagram](./assets/diagram.svg)

## Feature Status

- [ ] Collections
  - [x] UTXOs by Address
  - [x] Address by Tx Output
  - [x] Tx CBOR by Hash
  - [x] Tx Count by Address
  - [x] Chain Point by Tx Hash
  - [x] Balance by Address
  - [x] Pool Id by Stake Address
  - [ ] Pool Metadata by Pool Id
  - [ ] Chain Parameters by Epoch
  - [ ] UTXOs by Asset
  - [ ] Block Hash by Tx Hash
  - [ ] Block Hashes by Epoch
  - [ ] Block Header by Block Hash
  - [ ] Tx Hashes by Block Hash
  - [ ] Ada Handle by Address
  - [ ] Address by Ada Handle
  - [ ] Block CBOR by Hash
  - [ ] Metadata by Tx Hash
  - [ ] Feature requests open
- [ ] Data Sources
  - [x] Node-to-Node ChainSync + Blockfetch
  - [ ] Node-to-Client ChainSync
  - [ ] Oura Kafka Topic
  - [ ] Raw-CBOR Block files
- [ ] Storage Backend
  - [x] Redis
  - [ ] MongoDB
  - [ ] Cassandra
  - [ ] AWS DynamoDB
  - [ ] GCP BigQuery
  - [ ] Firestore
  - [ ] Azure CosmoDB
  - [ ] Feature requests open
- [ ] Filtering Options
  - [x] By Input / Output Address
  - [x] By Withdrawal Address
  - [x] By Collateral Address
  - [x] By Block Slot Bounds
  - [ ] By Metadata Label
  - [ ] By Mint Policy / Asset
  - [ ] By Pool

## FAQ

### Don't we have tools for this already?

Yes, we do. We have excellent tools such as: [Kupo](https://github.com/CardanoSolutions/kupo), dcSpark's [Carp](https://github.com/dcSpark/carp) or [Db-Sync](https://github.com/input-output-hk/cardano-db-sync). Even the Cardano node itself might work as a source for some of the collections. Every tool is architected with a set of well-understood trade-offs. We believe _Scrolls_ makes sense as an addition to the list because assumes a particular set of trade-offs:

- network storage over local storage: _Scrolls_ makes sense if you have multiple distributed clients working in a private network that want to connect to the same data instance.
- read latency over data normalization: _Scrolls_ works well when you need to answer simple questions, like a lookup table. It won't work if you need to create joins or complex relational queries.
- data cache over data source: _Scrolls_ aims at being a "cache" of data, not the actual source of data. It has an emphasis on easy and fast reconstruction of the collections. It promotes workflows where the data is wiped and rebuilt from scratch whenever the use-case requires (such as adding / removing filters).
- Rust over Haskell: this is not a statement about the languages, both are great languages, each one with its own set of trade-offs. Since most of the Cardano ecosystem is written in Haskell, we opt for Rust as a way to broaden the reach to a different community of Rust developers (such as the authors of this tool). _Scrolls_ is extensible, it can be used as a library in Rust projects to create custom cache collections.
- bring your own db: storage mechanism in _Scrolls_ are pluggable, our goal is to provide a tool that plays nice with existing infrastructure. The trade-off is that you end up having more moving parts.

### How does this tool compare to _Oura_?

There's some overlap between _Oura_ and _Scrolls_. Both tools read on-chain data and output some data results. The main difference is that Oura is meant to **_react_** to events, to watch the chain and actuate upon certain patterns. In contrast, _Scrolls_ is meant to provide a snapshot of the current state of the chain by aggregating the whole history.

They were built to work well together. For example, let's say that you're building an app that uses Oura to process transaction data, you could then integrate _Scrolls_ as a way to lookup the source address of the transaction's input.
### What is "swarm mode"?

Swarm mode is a way to speed up the process of rebuilding collection from scratch by splitting the tasks into concurrent instances of the _Scrolls_ daemon by partitioning the history of the chain into smaller fragments.

![swarm mode diagram](assets/swarm.svg)
