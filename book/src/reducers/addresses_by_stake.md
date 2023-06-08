# Addresses By Stake

stake key -> addresses belonging to the stake key

## Configuration

```toml
[[reducers]]
type = "AddressesByStake"
key_prefix = Option<String>                     # default none
filter = Option<crosscut::filters::Predicate>   # default none
```

- `key_prefix:` Set custom key prefix
- `filter:` 