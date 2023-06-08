# Point By Transaction

tx hash -> (absolute slot, block hash)

## Configuration

```toml
[[reducers]]
type = "PointByTx"
key_prefix = Option<String> # default none
```

- `key_prefix:` Set custom key prefix