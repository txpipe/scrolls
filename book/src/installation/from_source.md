# From Source


To compile from source, you'll need to have the Rust toolchain available in your development box. Execute the following commands to clone and build the project:

```sh
git clone https://github.com/txpipe/scrolls.git
cd scrolls
cargo build
```

Tell cargo to use git to fetch dependecies if it fails with `no authentication available` error:
```
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build
```

Execute scrolls with:

```
./target/debug/scrolls
```



TODO: `cargo install --path .`
