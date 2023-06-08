# Assets By Address

address -> assets owned by the address and their quantities

## Configuration

```toml
[[reducers]]
type = "AssetsByAddress"
key_prefix = Option<String>                     # default "assets_by_address"
filter = Option<crosscut::filters::Predicate>   # default none
aggr_by = Option<AggrBy>                        # default none, possible values: ["Epoch"]
policy_ids_hex = Option<Vec<String>>            # default none
```

- `key_prefix:` Set custom key prefix
- `filter:` 
- `aggr_by:` Aggregate by epoch
- `policy_ids_hex:` If specified only those policy ids as hex will be taken into account. If not, all policy ids will be indexed