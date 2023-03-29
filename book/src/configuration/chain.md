# Chain

Specify which network to fetch data from.

## Fields
- type: `"Mainnet" | "Testnet" | "PreProd" | "Preview" | "Custom"`
- magic (*): `u64`,
- byron_epoch_length (*): `u32`,
- byron_slot_length (*): `u32`,
- byron_known_slot (*): `u64`,
- byron_known_hash (*): `String`,
- byron_known_time (*): `u64`,
- shelley_epoch_length (*): `u32`,
- shelley_slot_length (*): `u32`,
- shelley_known_slot (*): `u64`,
- shelley_known_hash (*): `String`,
- shelley_known_time (*): `u64`,
- address_network_id (*): `u8`,
- adahandle_policy (*): `String`,


(*) Use only with `type = "Custom"`

## Examples

Using mainnet
``` toml
[chain]
type = "Mainnet"
```

Using custom values (mainnet): 
``` toml
[chain]
type = "Custom"
magic = 764824073
byron_epoch_length = 432000
byron_slot_length = 20
byron_known_slot = 0
byron_known_time = 1506203091
byron_known_hash = "f0f7892b5c333cffc4b3c4344de48af4cc63f55e44936196f365a9ef2244134f"
shelley_epoch_length = 432000
shelley_slot_length = 1
shelley_known_slot = 4492800
shelley_known_hash = "aa83acbf5904c0edfe4d79b3689d3d00fcfc553cf360fd2229b98d464c28e9de"
shelley_known_time = 1596059091
address_network_id = 1
adahandle_policy = "f0ff48bbb7bbe9d59a40f1ce90e9e9d0ff5002ec48f232b49ca0fb9a"

```
