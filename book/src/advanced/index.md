# Advanced options

- [Stateful Cursor](./stateful_cursor.md): provides a mechanism to persist the "position" of the processing pipeline to make it resilient to restarts
- [Rollback Buffer](./rollback_buffer.md): provides a way to mitigate the impact of chain rollbacks in downstream stages
- [Intersect](./intersect_options.md): options for instructing Scrolls from which point in the chain to start reading from
- [Swarm Mode](./swarm_mode.md): method for speeding up the process of rebuilding collections from scratch by splitting the tasks into concurrent instances of the _Scrolls_ daemon