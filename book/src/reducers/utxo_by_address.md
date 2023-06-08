# UTxOs By Address

address -> utxo references

## Configuration

```toml
[[reducers]]
type = "UtxoByAddress"
key_prefix = Option<String>                     # default none
filter = Option<crosscut::filters::Predicate>   # default none
```

- `key_prefix:` Set custom key prefix
- `filter:` 