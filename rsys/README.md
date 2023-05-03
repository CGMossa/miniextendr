# `rsys`

Generates rust bindings for R through C-headers that come with R installation.

## Configuration

- `allowlist.txt` may be generated using `cargo xtask allowlist`.

## Internals

* `clang` is used to ensure only symbols from the headers are exported.
Meaning if you need C-specific functionality, these have to be imported, and compiled separately from this crate.
