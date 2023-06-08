# Balance By Stake Key

stake key -> ada balance

## Configuration

```toml
[[reducers]]
type = "BalanceByStakeKey"
key_prefix = Option<String>                     # default "balance_by_stake_key"
filter = Option<crosscut::filters::Predicate>   # default none
```

- `key_prefix:` Set custom key prefix
- `filter:`