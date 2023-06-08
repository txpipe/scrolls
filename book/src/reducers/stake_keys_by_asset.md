# Stake Keys By Asset

asset -> stake keys holding the asset

## Configuration

```toml
[[reducers]]
type = "StakeKeysByAsset"
key_prefix = Option<String>                     # default "stake_keys_by_asset"
filter = Option<crosscut::filters::Predicate>   # default none
aggr_by = Option<AggrBy>                        # default none, possible values: ["Epoch"]
policy_ids_hex = Option<Vec<String>>            # default none
```

- `key_prefix:` Set custom key prefix
- `filter:` 
- `aggr_by:` Aggregate by epoch
- `policy_ids_hex:` If specified only those policy ids as hex will be taken into account. If not, all policy ids will be indexed