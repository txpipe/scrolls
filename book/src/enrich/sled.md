# Redis

Stores the current UTxO set in Sled.

## Configuration

```toml
[enrich]
type = "Sled"
db_path = "./data/sled"
```

- `type` (Sled): this field must be set to the literal value `Sled`
- `db_path` (string): path to folder storing the database files