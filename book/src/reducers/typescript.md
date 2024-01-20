# Using Typescript to build custom reducers

With the Typescript reducer is possible to create a custom reducer for logics that the builtin reducers don't have support. This reducer is only enable with the deno feature on build.

To build reducer using Typescript, we need to code our custom logic as any other Typescript module and transpile it into JS code using Deno. The resulting file (.js) can then be referenced by Scrolls configuration so that it's loaded dynamically at the moment of execution.

To transpile your Typescript code using Deno, run the following command:

```
deno bundle reducer.ts reducer.js
```

## Configuration

Example of a configuration

```toml
[reducer]
type = "Deno"
main_module = "./examples/deno/reducer.js"
use_async = true
```

### Section: `reducer`

- `type`: the literal value `Deno`.
- `main_module`: the js file with the reducer logic
- `use_async`: run the js in async mode


## Run code

To run the code with the deno will be necessary to use deno feature

```sh
cargo run --features=deno -- daemon --config ./examples/deno/daemon.toml
```
