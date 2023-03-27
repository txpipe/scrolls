# Intersect

Scrolls provides 4 different strategies for finding the intersection point within the chain sync process.

- `Origin`: Scrolls will start reading from the beginning of the chain.
- `Tip`: Scrolls will start reading from the current tip of the chain.
- `Point`: Scrolls will start reading from a particular point (slot, hash) in the chain. If the point is not found, the process will be terminated with a non-zero exit code.
- `Fallbacks`: Scrolls will start reading the first valid point within a set of alternative positions. If point is not valid, the process will fallback into the next available point in the list of options. If none of the points are valid, the process will be terminated with a non-zero exit code.


## Fields
- type: `"Tip" | "Origin" | "Point" | "Fallbacks"`
- value **(*)**: `(u64, String) | Vec<(u64, String)>`

**(*)** Use `(u64, String)` with `type = "Point"` and `Vec<(u64, String)>` with `type = "Fallbacks"`
