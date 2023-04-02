# Predicates 

Following `Predicate`s are available to be used as filters by some reducers.

- [all_of](#all_of-vecpredicate) `Vec<Predicate>`
- [any_of](#any_of-vecpredicate) `Vec<Predicate>`
- [not](#not-predicate) `Predicate`
- [block](#block-blockpattern) `BlockPattern`
- [transaction](#transaction-transactionpattern) `TransactionPattern`
- [input_address](#input_address-addresspattern) `AddressPattern`
- [output_address](#output_address-addresspattern) `AddressPattern`
- [withdrawal_address](#withdrawal_address-addresspattern) `AddressPattern`
- [collateral_address](#collateral_address-addresspattern) `AddressPattern`
- [address](#address-addresspattern) `AddressPattern`


##### BlockPattern:
- slot_before: `Option<u64>`
- slot_after: `Option<u64>`

##### TransactionPattern:
- is_valid: `Option<bool>`

##### AddressPattern: 
- exact_hex: `Option<String>`
- exact_bech32: `Option<String>`
- payment_hex: `Option<String>`
- payment_bech32: `Option<String>`
- stake_hex: `Option<String>`
- stake_bech32: `Option<String>`
- is_script: `Option<bool>`

<br />
<br />
<hr />

## `all_of (Vec<Predicate>)`
 This predicate will yield true when _all_ of the predicates in the argument yields true.

#### Example (toml)

```toml
[[reducers]]
type = "AddressByTxo"

[[reducers.filter.all_of]]
[reducers.filter.all_of.output_address]
exact = "<address>"

[[reducers.filter.all_of]]
[reducers.filter.all_of.input_address]
exact = "<address>"
```

#### Example (json)

```json
{
  "reducers": [
    {
      "filter": {
        "all_of": [
          {
            "output_address": {
              "exact": "<address>"
            }
          },
          {
            "input_address": {
              "exact": "<address>"
            }
          }
        ]
      },
      "type": "AddressByTxo"
    }
  ]
}
```

<br />
<br />
<hr />


## `any_of (Vec<Predicate>)`
 This predicate will yield true when _any_ of the predicates in the argument yields true.

#### Example (toml)

```toml
[[reducers]]
type = "AddressByTxo"

[[reducers.filter.any_of]]
[reducers.filter.any_of.output_address]
exact = "<address>"

[[reducers.filter.any_of]]
[reducers.filter.any_of.input_address]
exact = "<address>"
```

#### Example (json)

```json
{
  "reducers": [
    {
      "filter": {
        "any_of": [
          {
            "output_address": {
              "exact": "<address>"
            }
          },
          {
            "input_address": {
              "exact": "<address>"
            }
          }
        ]
      },
      "type": "AddressByTxo"
    }
  ]
}
```

<br />
<br />
<hr />

## not (Predicate)
 This predicate will yield true when the predicate in the arguments yields false.


#### Example (toml)

```toml
[[reducers]]
type = "AddressByTxo"

[[reducers.filter.not]]
[reducers.filter.not.output_address]
exact = "<hex>"

```

#### Example (json)

```json
{
  "reducers": [
    {
      "type": "AddressByTxo",
      "filter": {
        "not": {
          "output_address": {
            "exact": "<hex>"
          }
        }
      }
    }
  ]
}
```

<br />
<br />
<hr />

## `block (BlockPattern)`

#### Example (toml)

```toml
[[reducers]]
type = "AddresByTxo"

[[reducers.filter.block]]
slot_before = 83548786
```

#### Example (json)

```json
{
  "reducers": [
    {
      "type": "AddresByTxo",
      "filter": {
        "block": [
          {
            "slot_before": 83548786
          }
        ]
      }
    }
  ]
}
```

<br />
<br />
<hr />

## `transaction (TransactionPattern)`

#### Example (toml)

```toml
[[reducers]]
type = "AddressByTxo"

[reducers.filter.transaction]
is_valid = false

```

#### Example (json)

```json
{
  "reducers": [
    {
      "filter": {
        "transaction": {
          "is_valid": false
        }
      },
      "type": "AddressByTxo"
    }
  ]
}
```

<br />
<br />
<hr />

## `input_address (AddressPattern)`

#### Example (toml)

```toml
[[reducers]]
type = "AddressByTxo"

[reducers.filter.input_address]
exact_hex = "<hex>"
```

#### Example (json)

```json
{
  "reducers": [
    {
      "filter": {
        "input_address": {
          "exact_hex": "<hex>"
        }
      },
      "type": "AddressByTxo"
    }
  ]
}
```

<br />
<br />
<hr />

## `output_address (AddressPattern)`

#### Example (toml)

```toml
[[reducers]]
type = "AddressByTxo"

[reducers.filter.output_address]
exact_hex = "<hex>"
```

#### Example (json)

```json
{
  "reducers": [
    {
      "filter": {
        "output_address": {
          "exact_hex": "<hex>"
        }
      },
      "type": "AddressByTxo"
    }
  ]
}
```

<br />
<br />
<hr />

## `withdrawal_address (AddressPattern)`

#### Example (toml)

```toml
[[reducers]]
type = "AddressByTxo"

[reducers.filter.withdrawal_address]
exact_hex = "<hex>"
```

#### Example (json)

```json
{
  "reducers": [
    {
      "filter": {
        "withdrawal_address": {
          "exact_hex": "<hex>"
        }
      },
      "type": "AddressByTxo"
    }
  ]
}
```

<br />
<br />
<hr />

## `collateral_address (AddressPattern)`

#### Example (toml)

```toml
[[reducers]]
type = "AddressByTxo"

[reducers.filter.collateral_address]
exact_hex = "<hex>"
```

#### Example (json)

```json
{
  "reducers": [
    {
      "filter": {
        "collateral_address": {
          "exact_hex": "<hex>"
        }
      },
      "type": "AddressByTxo"
    }
  ]
}
```

<br />
<br />
<hr />

## `address (AddressPattern)`

#### Example (toml)

```toml
[[reducers]]
type = "AddressByTxo"

[reducers.filter.address]
exact_hex = "<hex>"
```

#### Example (json)

```json
{
  "reducers": [
    {
      "filter": {
        "address": {
          "exact_hex": "<hex>"
        }
      },
      "type": "AddressByTxo"
    }
  ]
}
```
