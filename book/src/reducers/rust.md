# Using Rust to build custom reducers

To build reducer using Rust, we need to code our custom logic as any other Rust-based program and compile it with a WASM target. The resulting file (.wasm) can then be referenced by Scrolls configuration so that it's loaded dynamically at the moment of execution.

## Configuration

Example of a configuration

```toml
[reducer]
type = "Wasm"
main_module = "./examples/wasm/enrich.wasm"
```

### Section: `reducer`

- `type`: the literal value `Wasm`.
- `main_module`: the wasm file containing the reducer logic


## Run code

To run a custom WASM reducer, it will be necessary to trigger scrolls enabling the  `wasm` feature

```sh
cargo run --features=wasm -- daemon --config ./examples/wasm/daemon.toml
```
