# miniextendr CLI

A standalone command-line tool for building R packages with Rust. Covers all scaffolding, workflow, vendor, and cargo operations without requiring R for native commands.

## Install

```bash
cargo install --path miniextendr-cli
```

Or from the workspace:

```bash
just cli-build    # build only
just cli-install  # install to ~/.cargo/bin
```

## Usage

```text
miniextendr [OPTIONS] <COMMAND>

Options:
    --path <PATH>  Project directory (default: current directory)
    -q, --quiet    Suppress output
    --json         Output in JSON format
    -h, --help     Print help
    -V, --version  Print version
```

## Commands

End-user commands for building R packages with Rust. No knowledge of miniextendr internals required.

### `init` — Create or add miniextendr to a project

```bash
miniextendr init package my.package         # New R package with Rust
miniextendr init monorepo my.project        # Rust workspace + embedded R package
miniextendr init use                        # Add miniextendr to existing package
```

`init` renders the canonical `minirextendr` templates (embedded at compile
time from `minirextendr/inst/templates/`), so a CLI-generated package is
identical to one scaffolded by `minirextendr::create_miniextendr_package()` /
`create_miniextendr_monorepo()` / `use_miniextendr()`: same two install modes
(source vs. offline tarball, keyed on `inst/vendor.tar.xz` presence), same
configure-written `.cargo/config.toml`, same wrapper and wasm-registry
generation — and no `just` required to build the result.

`init use` auto-detects the layout: run it inside an R package (has
`DESCRIPTION`) to add Rust scaffolding in place, or inside a Rust workspace
(has `Cargo.toml`) to embed an R package subdirectory
(`--template-type rpkg|monorepo` overrides detection).

### `workflow` — Build, document, check, and manage R package

```bash
miniextendr workflow build                  # Full two-pass build
miniextendr workflow configure              # Generate Makevars and build config
                                            # (install mode auto-detected from
                                            #  inst/vendor.tar.xz presence)
miniextendr workflow document               # Derive NAMESPACE/man from generated wrappers
miniextendr workflow test                   # Run R tests (devtools::test)
miniextendr workflow test --filter vec      # Run filtered tests
miniextendr workflow check                  # R CMD check (devtools::check)
miniextendr workflow doctor                 # Comprehensive project health check
miniextendr workflow sync                   # autoconf + configure + document
miniextendr workflow install --r-cmd        # R CMD INSTALL
miniextendr workflow install                # devtools::install
miniextendr workflow dev-link               # devtools::load_all
miniextendr workflow autoconf               # Run autoconf
miniextendr workflow check-rust             # Validate Rust toolchain
miniextendr workflow upgrade                # Upgrade build templates and metadata
```

### `status` — Check project status (native, no R needed)

```bash
miniextendr status has                      # Check if project has miniextendr
miniextendr status show                     # Show which files are present/missing
miniextendr status validate                 # Validate configuration
miniextendr status has --json               # JSON output
```

### `cargo` — Run cargo commands in project context

Automatically finds `src/rust/Cargo.toml` and passes `--manifest-path`.

```bash
miniextendr cargo build                     # cargo build
miniextendr cargo build --release           # Release build
miniextendr cargo check                     # cargo check
miniextendr cargo test                      # cargo test
miniextendr cargo test -- --nocapture       # Pass args to test binary
miniextendr cargo clippy                    # cargo clippy
miniextendr cargo fmt                       # cargo fmt
miniextendr cargo fmt --check               # Check formatting
miniextendr cargo doc --open                # Build and open docs
miniextendr cargo add serde --features derive  # Add dependency
miniextendr cargo rm serde                  # Remove dependency
miniextendr cargo update                    # Update Cargo.lock
miniextendr cargo deps                      # Dependency tree (depth 1)
miniextendr cargo deps --depth 3            # Deeper tree
miniextendr cargo search tokio              # Search crates.io
miniextendr cargo init                      # Initialize Rust crate in src/rust
miniextendr cargo clean                     # Clean build artifacts
```

### `vendor` — Manage vendored dependencies

```bash
miniextendr vendor pack                     # Create vendor.tar.xz for CRAN
miniextendr vendor sync                     # Sync from workspace source
miniextendr vendor sync-check               # Verify vendor matches workspace
miniextendr vendor sync-diff                # Show diff
miniextendr vendor crates-io                # Vendor external deps
miniextendr vendor miniextendr              # Copy miniextendr crates to vendor/
miniextendr vendor versions                 # List available versions (via gh)
miniextendr vendor cache-info               # Show cache directory
miniextendr vendor cache-clear              # Clear all cached archives
miniextendr vendor use-lib mycrate --dev-path ../mycrate  # Vendor local dep
```

### `feature` — Manage Cargo features and detection rules

```bash
miniextendr feature enable r6               # Enable R6 class system
miniextendr feature enable serde            # Add serde with derive
miniextendr feature enable rayon            # Add rayon parallelism
miniextendr feature list                    # List Cargo features
miniextendr feature list --json             # JSON output
miniextendr feature detect init             # Set up feature detection
miniextendr feature rule add myfeature 'requireNamespace("pkg")'
miniextendr feature rule list               # List detection rules
miniextendr feature rule remove myfeature   # Remove a rule
```

Configure-time feature setup and rule edits require R with `minirextendr`
installed. They call the same R helpers as `use_configure_feature_detection()`,
`add_feature_rule()`, and `remove_feature_rule()`, including `--cargo-spec` and
`--optional-dep`. Setup patches `configure.ac` and reruns autoconf when available.
An existing script without the rules markers is preserved with a diagnostic.

Rule listing is native and does not execute predicates. `feature rule list
--json` returns an object mapping feature names to their R expressions.
`feature detect update` updates **runtime** feature-query wrappers; configure-time
rules are managed by `feature detect init` and `feature rule`.

`workflow upgrade` also requires `minirextendr`: it calls
`upgrade_miniextendr_package()` with the R helper's defaults, including its dirty
file guard and preservation of customized `configure.ac`. To replace that file,
use `minirextendr::upgrade_miniextendr_package(configure_ac = TRUE)` from R.

Optional Git hooks and editor skills remain R-side setup:
`minirextendr::use_miniextendr_git_hooks()` and `minirextendr::use_claude_skills()`.
The CLI does not install those extras during `init`.

### `render` — Rmarkdown/Quarto integration

```bash
miniextendr render knitr-setup              # Print knitr setup instructions
miniextendr render rmarkdown                # Print rmarkdown YAML header
miniextendr render quarto                   # Print Quarto config
miniextendr render quarto-pre               # Run Quarto pre-render (sync)
miniextendr render html                     # Sync + render HTML
miniextendr render pdf                      # Sync + render PDF
miniextendr render word                     # Sync + render Word
```

### `rust` — Dynamic Rust compilation

```bash
miniextendr rust source 'pub fn add(a: f64, b: f64) -> f64 { a + b }'
miniextendr rust function '#[miniextendr] pub fn hello() -> &str { "hi" }'
miniextendr rust clean                      # Clean compilation cache
```

### `config` — Show configuration (native, no R needed)

```bash
miniextendr config show                     # Show miniextendr.yml
miniextendr config defaults                 # Show default values
miniextendr config defaults --json          # JSON output
```

### `lint` — Run miniextendr-lint

```bash
miniextendr lint                            # Check macro/module consistency
```

### `clean` — Clean build artifacts

```bash
miniextendr clean                           # Clean workspace build artifacts
```

### `completions` — Generate shell completions

```bash
miniextendr completions bash                # Bash completions
miniextendr completions zsh                 # Zsh completions
miniextendr completions fish                # Fish completions
miniextendr completions powershell          # PowerShell completions
miniextendr completions elvish              # Elvish completions
```

#### Installation

```bash
# Bash — add to ~/.bashrc
eval "$(miniextendr completions bash)"

# Zsh — add to ~/.zshrc (ensure fpath includes the target dir)
miniextendr completions zsh > "${fpath[1]}/_miniextendr"

# Fish
miniextendr completions fish > ~/.config/fish/completions/miniextendr.fish

# PowerShell — add to $PROFILE
miniextendr completions powershell >> $PROFILE
```

## Developer Commands (`miniextendr dev`)

Commands for developing the miniextendr framework itself. These are **not needed by end users** building R packages — they're for contributors working on the miniextendr codebase.

These commands are behind the `dev` Cargo feature and not included in default builds:

```bash
cargo install --path miniextendr-cli --features dev   # install with dev commands
just cli-install                                       # same (just recipe passes --features dev)
```

### `dev bench` — Run benchmarks

```bash
miniextendr dev bench run                   # All benchmarks
miniextendr dev bench core                  # Core benchmarks only
miniextendr dev bench features              # Feature-gated benchmarks
miniextendr dev bench full                  # Full suite (core + features)
miniextendr dev bench r                     # R-side benchmarks
miniextendr dev bench save                  # Save baseline
miniextendr dev bench compare               # Compare baselines
miniextendr dev bench drift                 # Check for regressions
miniextendr dev bench info                  # List saved baselines
miniextendr dev bench compile               # Macro compile-time perf
miniextendr dev bench lint-bench            # Lint scan performance
miniextendr dev bench check                 # Verify bench crate compiles
```

### `dev cross` — Cross-package trait dispatch testing

```bash
miniextendr dev cross configure             # Configure both test packages
miniextendr dev cross install               # Build and install both
miniextendr dev cross document              # Regenerate docs for both
miniextendr dev cross test                  # Run cross-package tests
miniextendr dev cross check                 # R CMD check both
miniextendr dev cross clean                 # Clean both
```

### `dev templates` — Template drift checking

```bash
miniextendr dev templates check             # Verify templates match approved delta
miniextendr dev templates approve           # Accept current delta
miniextendr dev templates sources           # Show template source mappings
```

## Architecture

Three execution modes:

| Mode | R needed? | Examples |
|------|-----------|---------|
| **Native** | No | `status`, `config`, `cargo`, `lint`, `feature list` |
| **Hybrid** | No (shell only) | `workflow autoconf`, `workflow configure` |
| **R bridge** | Yes (Rscript) | `workflow document`, `workflow test`, `render html` |
| **Dev** | Varies | `dev bench`, `dev cross`, `dev templates` |

The CLI is fully independent of the `minirextendr` R package. When R is needed, it calls `devtools`/`testthat` directly via `Rscript -e`.

## Global options

- `--path <dir>` — Project directory (default: `.`)
- `--quiet` / `-q` — Suppress output
- `--json` — JSON output (supported by `status`, `config`, `vendor`, `feature`)
