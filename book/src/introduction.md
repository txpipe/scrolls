# Introduction

## Motivation

Since blockchain is a sequence of transactions it is far from trivial to get a fast read only access to data. Scrolls is a tool for building and maintaining read-optimized collections of Cardano's on-chain entities. It crawls the history of the chain and aggregates all data to reflect the current state of affairs. Once the whole history has been processed, Scrolls watches the tip of the chain to keep the collections up-to-date. 

## Under the hood
All the heavy lifting required to communicate with the Cardano node is done by the [Pallas](https://github.com/txpipe/pallas) library, which provides an implementation of the Ouroboros multiplexer and a few of the required mini-protocol state-machines (ChainSync and LocalState in particular).
