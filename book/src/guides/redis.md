# Redis-cli

If you are not familiar with Redis you might want to play with the [redis-cli](https://redis.io/docs/ui/cli/) command line application before start coding any serious application. This guide gives you the very basic commands needed to fetch information using `redis-cli`. 

See the [Redis](../storage/redis.md) (storage) section of the manual for Redis installation and configuration.


- Get all keys: KEYS *
- Get type of key: TYPE
- Get set members: SMEMBERS
- Get string: GET
- Clear database: FLUSHDB
- Clear all databases: FLUSHALL
