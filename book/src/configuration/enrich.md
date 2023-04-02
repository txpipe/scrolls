# Enrich
Store utxo information in a local DB, this is needed for some reducers to work. Currently, only [Sled](https://github.com/spacejam/sled) databases are supported.

## Fields
- type: `"Sled" | "Skip"`
- db_path (*): `String`

(*) Use only with `type = "Sled"`

## Example

``` toml
[enrich]
type = "Sled"
db_path = "/opt/scrolls/sled_db"
```
