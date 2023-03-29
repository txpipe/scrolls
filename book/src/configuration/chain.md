# Chain

Specify which network to fetch data from.

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

## Examples

Using mainnet
``` toml
[chain]
type = "Mainnet"
```

Using mainnet (custom values): 
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

Using preview (custom values):
``` toml
magic = 2
byron_epoch_length = 432000
byron_slot_length = 20
byron_known_slot = 0
byron_known_hash = ""
byron_known_time = 1660003200
shelley_epoch_length = 432000
shelley_slot_length = 1
shelley_known_slot = 25260
shelley_known_hash = "cac921895ef5f2e85f7e6e6b51b663ab81b3605cd47d6b6d66e8e785e5c65011"
shelley_known_time = 1660003200
address_network_id = 0
adahandle_policy = ""
```

Using preprod (custom values):
``` toml
[chain]
type = "Custom"
magic = 1
byron_epoch_length = 432000
byron_slot_length = 20
byron_known_slot = 0
byron_known_hash = "9ad7ff320c9cf74e0f5ee78d22a85ce42bb0a487d0506bf60cfb5a91ea4497d2"
byron_known_time = 1654041600
shelley_epoch_length = 432000
shelley_slot_length = 1
shelley_known_slot = 86400
shelley_known_hash = "c4a1595c5cc7a31eda9e544986fe9387af4e3491afe0ca9a80714f01951bbd5c"
shelley_known_time = 1654041600
address_network_id = 0
adahandle_policy = ""
```
