# Enrich
Store UTXOs information in a local DB, this is needed for some reducers to work. Currently, only [Sled](https://github.com/spacejam/sled) supported.

## Fields
- type: `"Sled" | "Skip"`
- db_path (*): `"<dirpath>"`

(*) Use only with `type = "Sled"`

## Example

``` toml
[enrich]
type = "Sled"
db_path = "/opt/scrolls/sled_db"
```
