# Stateful Cursor

The _cursor_ feature provides a mechanism to persist the "position" of the processing pipeline to make it resilient to restarts.

## Context

Building a stateful view of the chain requires us to process every block up to the tip of the chain. Missing any blocks will result in a corrupted state.

## Feature

Scrolls implements a stateful cursor that receives notifications from the sink stage of the pipeline to continuously track the current position of the chain. After every block, the position is persisted into storage.

Assuming that a restart occurs, the process will locate and load the persisted value and instruct the source stage to begin reading chain data from the last known position. 

## Configuration

The name of the _cursor_ key can be configured in the storage section:

```toml
[storage]
type = "Redis"
connection_params = "redis://localhost:6379"
cursor_key = "_cursor"
```