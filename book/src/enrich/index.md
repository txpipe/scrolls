# Enrich

For some reducers to work, _Scrolls_ requires an extra pipeline stage referred to as enrichment.

Simply, enrichment is the process of maintaining the current UTxO set and providing these UTxOs (specifically their data) to the reducers when they are eventually consumed.

To elaborate, blocks processed by _Scrolls_ are made up of transactions which themselves are made up of inputs and outputs. While the transaction outputs contain output data such as the destination address, amount of Ada and quantity of native assets, the transaction inputs are simply *references* to previous transaction outputs (specifically the reference is a transaction hash + UTxO index).

In other words, when we process a block's data in isolation we only know the output distribution of assets, not the input distribution. This is problematic for certain reducers that require the input distribution to maintain an accurate record of account. For example, a reducer such as BalanceByAddress needs to not only add Ada value to the destination addresses, but simultaneously subract Ada value from the source addresses. With only a UTxO reference, we don't know the source address or the Ada value emitted by that address.

Enrichment solves this problem by maintaining a key-value store mapping UTxO references to their corresponding data. When a block is ingested into the pipeline, _Scrolls_ will look up the input references assciated with that block and provide the corresponding UTxO as an additional parameter to the reducer. It will also update the UTxO set by deleting consumed outputs and adding the new outputs.

_Scrolls_ offers several options for storing the UTxO set.

- [redis](./redis.md): uses a Redis instance. Convenient if you are already using Redis as your storage service
- [sled](./sled.md): uses an embedded on-disk database written in Rust ([link](https://github.com/spacejam/sled))