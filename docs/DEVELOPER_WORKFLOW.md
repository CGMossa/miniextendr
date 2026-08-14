# Developer Workflow

Quick reference for common development tasks. See [CLAUDE.md](../CLAUDE.md) for complete build system documentation.

## Prerequisites

- Rust toolchain (stable)
- R 4.2+ with development headers
- [just](https://github.com/casey/just) command runner (optional if using the CLI)
- autoconf (for regenerating `configure` from `configure.ac`)

> **Tip:** The `miniextendr` CLI (`miniextendr-cli/`) can replace most `just` recipes. Install with [`just cli-install`](https://github.com/A2-ai/miniextendr/blob/main/justfile) or `cargo install --path miniextendr-cli --features dev`. See the [CLI README](../miniextendr-cli/README.md) for full usage.

### One-shot dev bootstrap

```bash
just dev-tools-install
```

Installs the helpers expected by the other recipes:

- `cargo-revendor` from the in-tree `cargo-revendor/` (required by [`just vendor`](https://github.com/A2-ai/miniextendr/blob/main/justfile) and tarball-producing bootstrap workflows; configure itself never vendors).
- `cargo-limit`, which provides the `cargo lcheck` / `lclippy` / `ltest` / `lbuild` aliases that truncate output to the first few errors, recommended for interactive iteration. CI and `just` recipes keep plain `cargo` so `-D warnings` output stays complete.

## Quick Start

```bash
just configure          # Generate build config (REQUIRED first step)
just rcmdinstall        # Build and install the example R package (`rpkg/`)
just devtools-test      # Run R tests
```

## Common Workflows

### Rust-only development (fastest iteration)

```bash
just check              # Type-check all crates
just check-features     # Verify feature combinations compile
just test               # Run Rust unit tests
just clippy             # Lint
just fmt                # Format
```

### R package development

```bash
just configure          # 1. Generate Makevars, config files
just rcmdinstall        # 2. Compile Rust + install R package
just devtools-test      # 3. Run R tests
```

### After changing proc macros or `#[miniextendr]` attributes

```bash
just configure && just rcmdinstall && just force-document
```

Use `just force-document` (not `just devtools-document`) here — it bypasses
`roxygen2::needs_roxygenize()`'s mtime cache, which may not catch macro-layer
output changes made via `rcmdinstall`. Use `just devtools-document` only for
pure R/roxygen changes where no proc-macro output changed.

### Adding a new `#[miniextendr]` function

1. Add `#[miniextendr]` to your `pub` function in a `.rs` file
2. Run the macro-change workflow above

Registration is automatic via `#[miniextendr]` -- no manual module declarations needed.

### Running R CMD check

```bash
just configure          # 1. Configure
just r-cmd-build        # 2. Build tarball
just r-cmd-check        # 3. Check the tarball
```

### Cross-package tests

```bash
just cross-install      # Build and install producer.pkg + consumer.pkg
just cross-test         # Run cross-package trait ABI tests
```

### CRAN release prep

```bash
just vendor             # Regen Cargo.lock in tarball-shape, vendor deps,
                        # compress to inst/vendor.tar.xz.
just r-cmd-build        # Build tarball (depends on `just vendor`).
just r-cmd-check        # Check tarball (always check the tarball, not source dir).
```

The single signal `[ -f inst/vendor.tar.xz ]` triggers tarball-mode install
automatically; no env var to set. See [docs/CRAN_COMPATIBILITY.md](CRAN_COMPATIBILITY.md).

## Troubleshooting

### "configure: command not found"

Always invoke configure with `bash`:

```bash
cd rpkg && bash ./configure     # Correct
cd rpkg && ./configure          # May fail on some systems
```

Or regenerate from `configure.ac`:

```bash
cd rpkg && autoconf && bash ./configure
```

### "Could not find tools necessary to compile a package"

If running in a sandboxed environment, compilation commands need the sandbox disabled.

### R tests fail with "could not find function"

The function exists in Rust but isn't callable from R. Check:

1. Function is `pub`
2. Function has the `#[miniextendr]` attribute
3. Run [`just force-document`](https://github.com/A2-ai/miniextendr/blob/main/justfile) to regenerate NAMESPACE

Quick fix:

```bash
just configure && just rcmdinstall && just force-document
```

### Stale R wrappers after macro changes

Same fix:

```bash
just configure && just rcmdinstall && just force-document
```

### Permission errors during install

Use a local library path:

```bash
R_LIBS=/tmp/R_lib just rcmdinstall
```

### Lint warnings

Run [`just lint`](https://github.com/A2-ai/miniextendr/blob/main/justfile) to check source-level attributes for consistency. See [MACRO_ERRORS.md](MACRO_ERRORS.md) for interpreting lint output.

### Transient test failures in parallel runs

Running all test suites in parallel can cause races. Re-run failed suites individually:

```bash
just test               # If this fails, run specific crate tests
just clippy             # May fail during parallel configure
```

## Install modes

| Mode | When | Behavior |
|---|---|---|
| Source | `inst/vendor.tar.xz` absent | Cargo resolves through `[patch."git+url"]` to monorepo siblings, or fetches the git URL when no siblings are detected. |
| Tarball | `inst/vendor.tar.xz` present | Configure unpacks the tarball, writes `[source]` replacement to `vendored-sources`, build runs `--offline`. |

See [docs/CRAN_COMPATIBILITY.md](CRAN_COMPATIBILITY.md) for the full table.

## Sync Checks

```bash
just vendor-sync-check  # Verify vendored crates match workspace
just templates-check    # Verify templates haven't drifted from rpkg
just templates-approve  # Accept current template delta (after intentional changes)
```
