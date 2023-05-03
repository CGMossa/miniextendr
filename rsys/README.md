# `rsys`

Generates rust bindings for R through C-headers that come with R installation.

## Configuration

- `allowlist.txt` may be generated using `cargo xtask allowlist`.

## Internals

- `clang` is used to ensure only symbols from the headers are exported.
Meaning if you need C-specific functionality, these have to be imported, and compiled separately from this crate.

## Troubleshooting

Remember to call `cargo build -vv` to print out both what `cargo` does and what `build.rs` is doing as well.

While using `vscode`'s integrated terminal, you might want to increase the
scrollback limit if the output is overly verbose:

```json
{
    "terminal.integrated.scrollback": 100000,
}
```

## `vscode`-tips

```json
{
    "rust-analyzer.cargo.buildScripts.enable": true,
    //"rust-analyzer.cargo.buildScripts.invocationLocation": "workspace",
    "rust-analyzer.cargo.buildScripts.useRustcWrapper": true,
}
