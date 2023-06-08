# Supply By Asset

asset -> current supply of the asset

## Configuration

```toml
[[reducers]]
type = "SupplyByAsset"
key_prefix = Option<String>             # default "supply_by_asset"
policy_ids_hex = Option<Vec<String>>    # default none
```

- `key_prefix:` Set custom key prefix
- `policy_ids_hex:` If specified only those policy ids as hex will be taken into account. If not, all policy ids will be indexed