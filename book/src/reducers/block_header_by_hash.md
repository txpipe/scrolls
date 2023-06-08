# Block Header By Hash

block hash -> block header

## Configuration

```toml
[[reducers]]
type = "BlockHeaderByHash"
key_prefix = Option<String>                     # default none
filter = Option<crosscut::filters::Predicate>   # default none
```

- `key_prefix:` Set custom key prefix
- `filter:`