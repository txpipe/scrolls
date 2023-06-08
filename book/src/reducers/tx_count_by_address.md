# Transaction Count By Address

address -> tx count

## Configuration

```toml
[[reducers]]
type = "TxCountByAddress"
key_prefix = Option<String>                     # default "txcount_by_address"
filter = Option<crosscut::filters::Predicate>   # default none
```

- `key_prefix:` Set custom key prefix
- `filter:` 
