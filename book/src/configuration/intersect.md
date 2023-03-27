# Intersect

Scrolls provides 4 different strategies for finding the intersection point within the chain sync process.

- `Origin`: Scrolls will start reading from the beginning of the chain.
- `Tip`: Scrolls will start reading from the current tip of the chain.
- `Point`: Scrolls will start reading from a particular point (slot, hash) in the chain. If the point is not found, the process will be terminated with a non-zero exit code.
- `Fallbacks`: Scrolls will start reading the first valid point within a set of alternative positions. If point is not valid, the process will fallback into the next available point in the list of options. If none of the points are valid, the process will be terminated with a non-zero exit code.


## Fields
- type: `"Tip" | "Origin" | "Point" | "Fallbacks"`
- value (*): `(u64, String) | Vec<(u64, String)>`

(*) Use value of type `(u64, String)` with `type = "Point"` and value of type `Vec<(u64, String)>` with `type = "Fallbacks"`

## Examples

Using **Point**:
``` toml
[intersect]
type = "Point"
value = [57867490, "c491c5006192de2c55a95fb3544f60b96bd1665accaf2dfa2ab12fc7191f016b"]
```

Using **Fallbacks**:
``` toml
[intersect]
type = "Fallbacks"
value = [
      [12345678, "this_is_not_a_valid_hash_ff1b93cdfd997d4ea93e7d930908aa5905d788f"],
      [57867490, "c491c5006192de2c55a95fb3544f60b96bd1665accaf2dfa2ab12fc7191f016b"]
      ]
```
