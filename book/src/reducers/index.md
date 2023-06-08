# Reducers

Reducers are plugins that perform data aggregation within _Scrolls_. They describe the specific reduce operation we would like to perform on our blockchain data.

| Reducer Name                                          | Reducer Function                                                            |
|-------------------------------------------------------|-----------------------------------------------------------------------------|
| [AddressByAsset](./address_by_asset.md)               | asset -> last address the asset was transferred to (track Ada handles)      |
| [AddressByTxo](./address_by_txo.md)                   | transaction output reference (tx hash + txo index) -> owning address        |
| [AddressesByAsset](./addresses_by_asset.md)           | asset -> addresses holding the asset                                        |
| [AddressesByStake](./addresses_by_stake.md)           | stake key -> addresses belonging to the stake key                           |
| [AssetsByAddress](./assets_by_address.md)             | address -> assets owned by the address and their quantities                 |
| [AssetsByStakeKey](./assets_by_stake_key.md)          | stake key -> assets owned by the stake key and their quantities             |
| [BalanceByAddress](./balance_by_address.md)           | address -> ada balance                                                      |
| [BalanceByStakeKey](./balance_by_stake_key.md)        | stake key -> ada balance                                                    |
| [BlockHeaderByHash](./block_header_by_hash.md)        | block hash -> block header                                                  |
| [LastBlockParameters](./last_block_parameters.md)     | last block -> block parameters                                              |
| [PointByTx](./point_by_tx.md)                         | tx hash -> (absolute slot, block hash)                                      |
| [PoolByStake](./pool_by_stake.md)                     | stake key -> pool hash                                                      |
| [StakeKeysByAsset](./stake_keys_by_asset.md)          | asset -> stake keys holding the asset                                       |
| [SupplyByAsset](./supply_by_asset.md)                 | asset -> current supply of the asset                                        |
| [TxByHash](./tx_by_hash.md)                           | tx hash -> {tx cbor, absolute slot, tx time}                                |
| [TxCountByAddress](./tx_count_by_address.md)          | address -> tx count                                                         |
| [TxCountByAsset](./tx_count_by_asset.md)              | asset -> tx count                                                           |
| [TxCountByPolicyId](./tx_count_by_policy_id.md)       | policy id -> tx count                                                       |
| [TxCountByStakeKey](./tx_count_by_stake_key.md)       | stake key -> tx count                                                       |
| [UtxoByAddress](./utxo_by_address.md)                 | address -> utxo references                                                  |
| [UtxoByStake](./utxo_by_stake.md)                     | stake key -> utxo references                                                |
| [UtxosByAsset](./utxos_by_asset.md)                   | asset -> utxo references                                                    |

## Filter

## Aggregate By