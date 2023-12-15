# Deno

With the deno reducer is possible to create a custom reducer for logics that the builtin reducers don't have support. This reducer is only enable with the deno feature on build.

## Configuration

Example of a configuration

```toml
[reducer]
type = "Deno"
main_module = "./examples/deno/enrich.js"
use_async = true
```

### Section: `reducer`

- `type`: the literal value `Deno`.
- `main_module`: the js file with the reducer logic
- `use_async`: run the js in async mode


## Run code

To run the code with the deno will be necessary to use deno feature

```sh
cargo run --features=deno -- daemon --config ./examples/deno/enrich.js
```
