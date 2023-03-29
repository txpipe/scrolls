# Reducers


- [address_by_asset](#address_by_asset)
- [address_by_txo](#address_by_txo)
- [addresses_by_stake](#addresses_by_stake)
- [asset_holders_by_asset_id](#asset_holders_by_asset_id)
- [balance_by_address](#balance_by_address)
- [block_header_by_hash](#block_header_by_hash)
- [last_block_parameters](#last_block_parameters)
- [point_by_tx](#point_by_tx)
- [pool_by_stake](#pool_by_stake)
- [supply_by_asset](#supply_by_asset)
- [tx_by_hash](#tx_by_hash)
- [tx_count_by_address](#tx_count_by_address)
- [tx_count_by_native_token_policy_id](#tx_count_by_native_token_policy_id)
- [utxo_by_address](#utxo_by_address)
- [utxo_by_stake](#utxo_by_stake)
- [utxos_by_asset](#utxos_by_asset)


<br />
<br />
<hr />

## address_by_asset

### Config

- key_prefix: `Option<String>`
- filter: `Option<Vec<String>>`
- policy_id_hex: `String`
- convert_to_ascii(*): `Option<bool>`

(*) default is true.

### Example

### Output Format

<br />
<br />
<hr />

## address_by_txo

### Config

- key_prefix: `Option<String>`
- filter: `Option<Predicate>`

### Example
### Output Format

<br />
<br />
<hr />

## addresses_by_stake

### Config

- key_prefix: `Option<String>`
- filter: `Option<Vec<String>>`

### Example
### Output Format

<br />
<br />
<hr />

## asset_holders_by_asset_id

### Config

- key_prefix: `Option<String>`
- filter: `Option<Predicate>`
- aggr_by: `Option<AggrType>`

    /// Policies to match
    ///
    /// If specified only those policy ids as hex will be taken into account, if
    /// not all policy ids will be indexed.
- policy_ids_hex: `Option<Vec<String>>`
### Example
### Output Format

<br />
<br />
<hr />

## balance_by_address

### Config
- key_prefix: `Option<String>`
- filter: `Option<Predicate>`
### Example
### Output Format

<br />
<br />
<hr />

## block_header_by_hash

### Config
- key_prefix: `Option<String>`
- filter: `Option<Predicate>`

### Example
### Output Format

<br />
<br />
<hr />

## last_block_parameters

### Config
- key_prefix: `Option<String>`

### Example
### Output Format

<br />
<br />
<hr />

## point_by_tx

### Config
- key_prefix: `Option<String>`

### Example
### Output Format

<br />
<br />
<hr />

## pool_by_stake

### Config
- key_prefix: `Option<String>`

### Example
### Output Format

<br />
<br />
<hr />

## supply_by_asset

### Config
- key_prefix: `Option<String>`
- policy_ids_hex: `Option<Vec<String>>`

### Example
### Output Format

<br />
<br />
<hr />

## tx_by_hash

### Config
- key_prefix: `Option<String>`
- filter: `Option<Predicate>`
- projection: `"Cbor" | "Json"`

### Example

``` toml
[[reducers]]
type: "TxByHash"
key_prefix: "c1"
```

### Output Format

<br />
<br />
<hr />

## tx_count_by_address

### Config
- key_prefix: `Option<String>`
- filter: `Option<Predicate>`

### Example
### Output Format

<br />
<br />
<hr />

## tx_count_by_native_token_policy_id

### Config
- key_prefix: `Option<String>`
- aggr_by: `Option<AggrType>`

### Example
### Output Format

<br />
<br />
<hr />

## utxo_by_address

### Config
- key_prefix: `Option<String>`
- filter: `Option<Vec<String>>`

### Example
### Output Format

<br />
<br />
<hr />

## utxo_by_stake

### Config
- key_prefix: `Option<String>`
- filter: `Option<Vec<String>>`

### Example
### Output Format

<br />
<br />
<hr />

## utxos_by_asset

### Config
- key_prefix: `Option<String>`
- policy_ids_hex: `Option<Vec<String>>`

### Example
### Output Format
