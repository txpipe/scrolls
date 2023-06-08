# Address By Asset

asset -> last address the asset was transferred to (intended to track Ada handles)

## Configuration

```toml
[[reducers]]
type = "AddressByAsset"
key_prefix = Option<String> # default none
filter = Option<Vec<String>> # default none
policy_id_hex = String
convert_to_ascii = Option<bool> # default true
```

- `key_prefix:` Set custom key prefix
- `filter:` 
- `policy_id_hex:` Filter based on specific policy id
- `convert_to_ascii:` For the key suffix, encode asset name as ascii; otherwise encode as hex.
