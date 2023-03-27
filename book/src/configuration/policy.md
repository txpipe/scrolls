# Policy

## Fields
- missing_data: `"Skip" | "Warn" | "Default"`
- cbor_errors: `"Skip" | "Warn" | "Default"`
- ledger_errors: `"Skip" | "Warn" | "Default"`
- any_error: `"Skip" | "Warn" | "Default"`


## Example

``` toml
[policy]
cbor_errors = "Skip"
missing_data = "Warn"
```

