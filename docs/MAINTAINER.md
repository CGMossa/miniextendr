# Maintainer Guide

This document covers maintenance tasks for the miniextendr project.

## Version Management

Versions are tracked in two places that must stay in sync:

- `Cargo.toml` (`[workspace.package].version`)
- `rpkg/DESCRIPTION` (`Version:`)

Other locations derive their version automatically:

- All Rust crates use `version.workspace = true`
- `rpkg/configure.ac` reads from `Cargo.toml`

### Bumping Version

```bash
./scripts/bump-version.sh 0.2.0
```

This updates both `Cargo.toml` and `rpkg/DESCRIPTION`.

R development versions (e.g., `0.2.0.9000`) are allowed and will match the base version `0.2.0` in CI checks.

## Regenerating Configure Scripts

After modifying `rpkg/configure.ac`, regenerate the configure script:

```bash
cd rpkg
autoreconf -vif
```

Or use the justfile:

```bash
just configure  # runs autoconf + ./configure
```

### When to Regenerate

- After editing `configure.ac`
- After editing `Makevars.in`
- After changing autoconf macros
- Before committing changes to configure.ac

### Dependencies

Requires GNU autotools:

```bash
# macOS
brew install autoconf automake

# Debian/Ubuntu
apt-get install autoconf automake
```

## Development Workflow

### Full Rebuild (after macro changes)

When changing proc-macros, the full sequence is:

```bash
just configure && just rcmdinstall && just force-document
```

Use `just force-document` (not `just devtools-document`) after macro changes —
it bypasses `roxygen2::needs_roxygenize()`'s mtime cache, which may not catch
macro-layer output changes. `just devtools-document` is for pure R/roxygen changes.

### Quick Iteration

For most changes:

```bash
just check              # Fast cargo check
just rcmdinstall        # Build and install
```

### Running Tests

```bash
# Rust tests
just test

# R tests
just devtools-test

# Full R CMD check
just r-cmd-check
```

## Rustdoc Maintenance

Public and internal APIs should stay documented as they evolve.

- Run a targeted doc lint snapshot when touching API-heavy modules:

```bash
RUSTFLAGS='-Wmissing-docs' cargo check -p miniextendr-api --lib
```

- Prefer documenting internals in-place with rustdoc comments (`///`) near trait
  constants, enum variants, and error fields so generated docs remain useful.
- For raw header-mirror FFI declarations, document key types and safety model;
  avoid duplicating full upstream header docs verbatim.

## CI Workflow

The authoritative merge/release classification is the
[README gate table](../README.md#ci-and-release-gates). The normal
`.github/workflows/ci.yml` path is summarized by one stable check named
`CI Success`. It aggregates generated-file and version hygiene, sync checks,
documentation links, Rust lint/tests, Linux R checks/tests/GC-stress shards,
cross-package ABI tests, `minirextendr`, the CRAN-like tarball check, and the
bootstrap-vendor regression test. Path filters may legitimately skip
inapplicable jobs; the aggregate accepts `success` and `skipped`, not failures.

Do not infer platform or feature coverage from a green aggregate. macOS,
feature-runtime, feature-combination, and standalone round-trip jobs are
weekly or manually dispatched and intentionally sit outside `CI Success`.
The webR suite is a separate path-triggered workflow. Windows R validation is
currently disabled.

## Release Checklist

Use [Releasing miniextendr](./RELEASING.md) for versioning, artifact
construction, and tagging. Before pushing the tag:

1. **Push the release commit to a branch and record its SHA.** Every result
   below must exercise that commit, not an older green run on `main`.

2. **Require normal native CI to finish green.** Inspect `CI Success` and its
   constituent jobs:

   ```bash
   gh run list --workflow ci.yml --branch <release-branch> --limit 5
   ```

3. **Validate the release tarball.** `CRAN-like check` must be green, and run
   the maintainer recipe locally so the checked input is the built tarball:

   ```bash
   just r-cmd-check
   ```

4. **Run release-only native coverage.** A manual CI dispatch exercises both
   macOS architectures, the feature-runtime matrix, and the standalone
   round-trip. If feature selection changed, run the scheduled-only feature
   combination recipe locally too:

   ```bash
   just check-features
   gh workflow run ci.yml --ref <release-branch>
   ```

   `CI Success` does not aggregate these release-only jobs; inspect each result
   in the dispatched run.

5. **Run webR when its covered paths changed.** Require every job, including
   the tier-2 `R CMD INSTALL`, to pass:

   ```bash
   gh workflow run webr.yml --ref <release-branch>
   ```

6. **Run the GC safety sweep when runtime-sensitive code changed.** Miri is
   informational, but its latest report should still be reviewed:

   ```bash
   gh workflow run gctorture-nightly.yml --ref <release-branch>
   gh run list --workflow miri-nightly.yml --limit 1
   ```

7. **Record the evidence.** Put the release commit SHA and relevant Actions
   run URLs in the release notes. Until Windows validation is restored, state
   that the release lacks a Windows gate and link #94, #594, and #1335.

Only after this checklist is green should the release commit be tagged and the
tag pushed. There is no crates.io publication step today; distribution is from
the Git tag and the attached R source tarball.

## Vendoring for CRAN

R packages submitted to CRAN must be self-contained. The `vendor` recipe (which
calls `cargo-revendor`) produces the offline bundle:

```bash
just vendor
```

This:

1. Regenerates `Cargo.lock` in tarball-shape (git-URL sources for the
   `miniextendr-{api,lint,macros}` workspace crates)
2. Vendors all crates.io dependencies (proc-macro2, quote, syn, etc.) into `rpkg/vendor/`
3. Compresses the result to `rpkg/inst/vendor.tar.xz`

`just configure` does **no** vendoring in dev mode — it only generates
`Makevars` and `.cargo/config.toml`, and auto-detects tarball mode from the
presence of `inst/vendor.tar.xz`. Once the tarball exists, `configure` unpacks
it and writes the offline `[source]` replacement; delete it with
`just clean-vendor-leak` to return to source-mode dev iteration.

## Useful Commands

```bash
# List all just recipes
just

# Clean build artifacts
just clean

# Check all crates compile
just check

# Format code
just fmt

# Run clippy
just clippy

# Build documentation
just doc

# Expand macros (requires cargo-expand)
just expand

# Run benchmarks
just bench
```

## File Locations

| Purpose | Location |
|---------|----------|
| Workspace config | `Cargo.toml` |
| R package | `rpkg/` |
| Configure script | `rpkg/configure.ac` |
| Makefile template | `rpkg/src/Makevars.in` |
| Vendored crates | `rpkg/vendor/` |
| CI workflow | `.github/workflows/ci.yml` |
| Version bump script | `scripts/bump-version.sh` |
