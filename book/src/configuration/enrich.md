# Enrich

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
