# Transaction Count By Stake Key

stake key -> tx count

## Configuration

```toml
[[reducers]]
type = "TxCountByStakeKey"
key_prefix = Option<String>                     # default "tx_count_by_stake_key"
filter = Option<crosscut::filters::Predicate>   # default none
```

- `key_prefix:` Set custom key prefix
- `filter:` 