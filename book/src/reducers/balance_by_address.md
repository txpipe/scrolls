# Balance By Address

address -> ada balance

## Configuration

```toml
[[reducers]]
type = "BalanceByAddress"
key_prefix = Option<String>                     # default "balance_by_address"
filter = Option<crosscut::filters::Predicate>   # default none
```

- `key_prefix:` Set custom key prefix
- `filter:`