# Introduction

_Scrolls_ is a tool for building and maintaining read-optimized collections of Cardano's on-chain entities. It crawls the history of the chain and aggregates all data to reflect the current state of the blockchain. Once the whole history has been processed, _Scrolls_ watches the tip of the chain to keep the collections up-to-date.

## How it Works

Scrolls is a pipeline that takes block data as input and outputs DB update commands. The stages involved in the pipeline are the following:

- Source Stages: are in charge of pulling data block data. It might be directly from a Cardano node (local or remote) or from some other source. The requirement is to have the raw CBOR as part of the payload.
- Reducer Stages: are in charge of applying the map-reduce algorithm. They turn block data into CRDTs commands that can be later merged with existing data. The map-reduce logic will depend on the type of collection being built. Each reducer stage handles a single collection. Reducers can be enabled / disabled via configuration.
- Storage Stages: receive the generic CRDT commands and turns them into DB-specific instructions that are then executed by the corresponding engine.

![diagram](./assets/diagram.svg)

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


