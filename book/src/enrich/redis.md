# Redis

Stores the current UTxO set in Redis.

## Configuration

```toml
[enrich]
type = "Redis"
redis_url = "redis://localhost:6666"
```

- `type` (Redis): this field must be set to the literal value `Redis`
- `redis_url` (string): url connection parameter to your Redis instance