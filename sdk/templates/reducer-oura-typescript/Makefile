build:
	deno bundle reducer.ts reducer.js

run: build
	oura daemon --config ./oura.toml

.PHONY: build run
