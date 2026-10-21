# Deno

The Deno reducer, which ran custom reducer logic written in JavaScript, has
been removed together with the `deno` build feature, pending an upgrade of its
Deno runtime: the `deno_runtime` version it used pinned a TLS stack that no
longer receives fixes.
