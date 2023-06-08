# Transaction Count By Policy Id

policy id -> tx count

## Configuration

```toml
[[reducers]]
type = "TxCountByNativeTokenPolicyId"
key_prefix = Option<String>                     # default "transaction_count_by_native_token_policy"
aggr_by = Option<AggrBy>                        # default none, possible values: ["Epoch"]
```

- `key_prefix:` Set custom key prefix
- `aggr_by:` Aggregate by epoch