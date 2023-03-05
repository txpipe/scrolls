# Liquidity by Token Pair Reducer

## Introduction

This reducer intends to aggregate changes across different AMM DEXs (decentralized exchanges). It currently supports the most popular ones which includes:

- MinSwap
- SundaeSwap
- Wingriders

Note, that Muesliswap is not a pure AMM DEX and therefore, its liquidity for different token pairs cannot be in a similar way.

## Configuration

- `pool_currency_symbol` required hex-encoded currency symbol of the tokent that marks valid liquidity pool unspent transaction outputs (UTxOs)
- `pool_prefix` optional prefix for Redis key
- `dex_prefix` optional prefix for Redis members (usually used to prefix different liquidity sources by unique dex prefix)
- `json_value` an optional flag whether Redis members should be encoded in json or colon-notation (less disk space usage). Defaults to `false`.

## How it works

The reducer was implemented to be used for Redis. Hence, it produces key/value pairs for different liquidity sources. Thereby, a redis key represents a token pair (coin a, coin b) for which one or more liquidity pools exist from different DEXs.

### Redis Key Schema

A Redis key for a token pair for which at least one liquidity pool exists follows the following schema:

(`<pool_prefix>:`)?({`<currency_symbol_coin_a>.<coin_a_asset_name>` | `<empty>`}):(`<currency_symbol_coin_b>.<coin_b_asset_name>`)

Any key may have a prefix that can be optionally defined via the `pool_prefix` configuration (see section below). A single token is identified by its `currency_symbol` plus `asset_name`, separated by a period `.` and encoded in `hex`.
Any two tokens that make up a Redis key are sorted alphanumerically so that the smaller key comes first. This is required to have consistency across all liquidity pairs from different DEXs and also affects the member for a Redis key which hold liquidity ratios of each coin. But more on that later.
Liquidity pools that provide liquidity for `ADA` and some native asset always result in a Redis key that has the following format:

(`<pool_prefix>:`)?(`<currency_symbol_coin_b>.<coin_b_asset_name>`)

Since ADA has an empty currency symbol and asset name.

Example ADA/WRT liquidity key with `pool` prefix:
https://preprod.cardanoscan.io/token/659ab0b5658687c2e74cd10dba8244015b713bf503b90557769d77a757696e67526964657273

`pool.659ab0b5658687c2e74cd10dba8244015b713bf503b90557769d77a.757696e67526964657273`

### Redis Value Schema

The reducer's value is a set. Each entry is a single liquidity source can either be json encoded or a colon separated string of values. A single member can contains up to four fields:

- dex specific prefix to identift the origin of the liquidity source
- big integer amount of coin a
- big integer amount of coin b
- a decimal number defining the fee of the liquidity source that's paid to liquidity providers

Below you can find the general schema for a colon separated member:
(`<dex_prefix>:`)?(`<coin_a_amount>:<coin_b_amount>`)(`:<dex_liquidity_provider_fee>`)?

Example ADA/MIN liquidity source from MinSwap DEX:
`min:31249392392:1323123231221:0.003`

Below you can find the general schema for a JSON encoded member:

```
{
  "coin_a": number,
  "coin_b": number,
  "dex": string,
  "fee": numer
}
```

Example ADA/MIN liquidity source from MinSwap DEX:

```
{
  "coin_a": 31249392392,
  "coin_b": 1323123231221,
  "dex": "min",
  "fee": 0.003
}
```

### How to run

An example can be found in the test_drive directory for preprod and mainnet. It also comes with a `docker-compose` file.
