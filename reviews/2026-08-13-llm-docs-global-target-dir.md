# LLM docs generation ignored Cargo's configured target directory

## What was attempted

`just llm-docs` was run after removing public factor types so the tracked
rustdoc-derived API corpus would be regenerated rather than edited by hand.

## What went wrong

Every `cargo doc` invocation succeeded, but the renderer failed with:

```text
FileNotFoundError: target/doc/miniextendr_api.json
```

## Root cause

Cargo honored the developer's global `build.target-dir` setting and emitted
JSON under `/Users/elea/.cargo-target/doc`. The script ignored Cargo's resolved
configuration and hard-coded paths below each workspace checkout.

## Fix

The generator now asks `cargo metadata` for the root and cargo-revendor target
directories from their respective working directories, then feeds those exact
paths to every renderer. Stable source labels keep absolute developer paths out
of the committed reports.
