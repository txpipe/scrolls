# Chain

## Fields
- type: `"Mainnet" | "Testnet" | "PreProd" | "Preview" | "Custom"`
- magic (*): u64,
- byron_epoch_length (*): u32,
- byron_slot_length (*): u32,
- byron_known_slot (*): u64,
- byron_known_hash (*): String,
- byron_known_time (*): u64,
- shelley_epoch_length (*): u32,
- shelley_slot_length (*): u32,
- shelley_known_slot (*): u64,
- shelley_known_hash (*): String,
- shelley_known_time (*): u64,
- address_network_id (*): u8,
- adahandle_policy (*): String,


(*) Use only with `type = "Custom"`

## Example

``` toml
[chain]
type = "Mainnet"
```
