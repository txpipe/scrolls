# Assets By Stake Key

stake key -> assets owned by the stake key and their quantities

## Configuration

```toml
[[reducers]]
type = "AssetsByStakeKey"
key_prefix = Option<String>                     # default "assets_by_stake_key"
filter = Option<crosscut::filters::Predicate>   # default none
aggr_by = Option<AggrBy>                        # default none, possible values: ["Epoch"]
policy_ids_hex = Option<Vec<String>>            # default none
```

- `key_prefix:` Set custom key prefix
- `filter:` 
- `aggr_by:` Aggregate by epoch
- `policy_ids_hex:` If specified only those policy ids as hex will be taken into account. If not, all policy ids will be indexed