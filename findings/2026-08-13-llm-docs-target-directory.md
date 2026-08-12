# LLM docs generation assumed Cargo's default target directory

## Finding

`rust-llm-docs/generate-miniextendr-docs.sh` ran `cargo doc` and then
unconditionally opened `target/doc/*.json` (plus
`cargo-revendor/target/doc/*.json`). Cargo permits its target directory to be
overridden by user configuration, workspace configuration, or
`CARGO_TARGET_DIR`, so the build and render steps could point at different
directories.

This was reproduced with the active developer configuration:

```toml
[build]
target-dir = "/Users/elea/.cargo-target"
```

All rustdoc builds succeeded and wrote JSON to `/Users/elea/.cargo-target/doc`,
then rendering failed with `FileNotFoundError` for
`target/doc/miniextendr_api.json`.

## Resolution

Resolve `target_directory` through `cargo metadata --no-deps` from the same
working directory as each `cargo doc` invocation. Root-workspace renderers use
the root target directory; the standalone cargo-revendor renderers use its own
resolved target directory. This honors absolute, relative, global, workspace,
and environment overrides without forcing a cold task-local build. The report
renderers receive separate repository-relative source labels so generated
Markdown remains machine-independent and does not disclose developer paths.
