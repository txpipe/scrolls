# Address By Transaction Output

transaction output reference (tx hash + txo index) -> owning address

## Configuration

```toml
[[reducers]]
type = "AddressByTxo"
key_prefix = Option<String> # default none
filter = Option<crosscut::filters::Predicate> # default none
```

- `key_prefix:` Set custom key prefix
- `filter:` 