RUST_LOG=info cargo run \
    --manifest-path ../Cargo.toml \
    --all-features \
    --bin scrolls -- daemon --console plain --config ./daemon.toml 
