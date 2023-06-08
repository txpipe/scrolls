# Transaction By Hash

tx hash -> {tx cbor, absolute slot, tx time}

## Configuration

```toml
[[reducers]]
type = "TxByHash"
key_prefix = Option<String>                     # default none
filter = Option<crosscut::filters::Predicate>   # default none
projection = Option<Projection>                 # default none, possible values: ["Cbor", "Json"]
```

- `key_prefix:` Set custom key prefix
- `filter:` 
- `projection:`