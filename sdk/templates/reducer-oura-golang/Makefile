plugin.wasm:
	tinygo build -o plugin.wasm -target wasi main.go

scrolls.db:
	sqlite3 scrolls.db < init.sql

run: plugin.wasm scrolls.db
	oura daemon --config ./oura.toml

.PHONY: run
