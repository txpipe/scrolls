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

