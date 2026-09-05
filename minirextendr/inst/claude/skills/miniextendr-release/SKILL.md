---
name: miniextendr-release
description: Use when building, checking, or releasing a miniextendr-backed R package — producing the CRAN tarball, vendoring Rust dependencies (inst/vendor.tar.xz), offline installs, R CMD check --as-cran, a leftover vendor tarball breaking dev builds, Cargo.lock complaints, release CI for binaries, or version bumps.
---

# Releasing a miniextendr package

A CRAN (or offline) install cannot run `cargo` against the network, so the
release tarball must carry every Rust dependency inside it. The entire
release story revolves around one file:

## `inst/vendor.tar.xz` — the latch

| State | Meaning |
|---|---|
| absent | **source mode** — dev builds; cargo fetches deps from the network |
| present | **tarball mode** — offline build from the vendored sources inside it |

`./configure` checks for the file and flips modes automatically. It is
generated at build time and **must stay gitignored** (the scaffold's
`.gitignore` already covers it).

Two tarball-producing mechanisms create it:

1. **`bootstrap.R`** (in your package root, run by build frontends such as
   `devtools::build()` / `pkgbuild::build()` / `rcmdcheck` via
   `Config/build/bootstrap: TRUE`) — runs configure, then invokes
   `cargo-revendor` before the frontend calls `R CMD build`. Base
   `R CMD build` does not run this pkgbuild extension.
2. **Explicit**: `minirextendr::miniextendr_vendor()`.

`./configure` never creates the vendor tarball. A package tarball that lacks it
remains in source mode and is not suitable for CRAN's offline build farm.

Prerequisite once per machine:

```sh
cargo install --git https://github.com/A2-ai/miniextendr cargo-revendor --locked
```

Without it, `bootstrap.R` warns and builds a source-mode tarball (fine for
git installs; **not** CRAN-ready — CRAN's offline farm has no cargo-revendor
and will fail loudly, which is the intended canary).

## The release checklist

```r
# 1. clean tree, tests green
minirextendr::miniextendr_doctor()          # no leaks, toolchain ok
devtools::test()

# 2. build the tarball (bootstrap.R vendors automatically)
devtools::build()                            # → ../mypkg_X.Y.Z.tar.gz
```

```sh
# 3. ALWAYS check the built tarball, not the source directory —
#    source-dir checks skip Authors@R conversion and miss real CRAN failures
R CMD check --as-cran mypkg_X.Y.Z.tar.gz
```

```sh
# 4. prove the tarball installs offline (what CRAN effectively does)
R CMD INSTALL --library=$(mktemp -d) mypkg_X.Y.Z.tar.gz
```

DESCRIPTION fields the scaffold set up — keep them:
`SystemRequirements: Rust (>= 1.85)`, `Config/build/bootstrap: TRUE`,
`Config/build/extra-sources: src/rust/Cargo.lock`.

## The classic failure: a leaked tarball in the dev tree

If a build aborts partway, `inst/vendor.tar.xz` can linger in your source
tree. Every subsequent dev build then silently runs in tarball mode:
**your Rust edits are ignored** (the vendored copies build instead), or you
get `Cargo.lock` mismatch errors, or `library()` misses new functions.

```r
minirextendr::miniextendr_clean_vendor_leak()   # remove the stale latch
minirextendr::miniextendr_build()               # normal dev build again
```

`miniextendr_doctor()` detects this state. Note that
`minirextendr::miniextendr_build()` snapshots and restores the manifest +
tarball around its own install, so the *supported* dev loop never leaks.

## Cargo.lock

- Commit `src/rust/Cargo.lock` — reproducible dependency resolution is a
  CRAN expectation and the ship-what-you-tested guarantee.
- Vendoring normalizes the lockfile into "tarball shape" (source-replacement
  attribution). If configure complains about the lock shape after manual
  cargo operations: `minirextendr::miniextendr_repair_lock()`.
- Dependency updates are a deliberate act: `cargo update` in `src/rust/`,
  re-run tests, commit the lock.

## Binary release CI (GitHub Actions)

```r
minirextendr::use_release_workflow()
```

scaffolds a known-good workflow that builds binaries on Linux
(AlmaLinux 8, glibc-portable) and macOS arm64. Use the template rather than
writing your own — it encodes several platform traps: AlmaLinux's `C` locale
(breaks cargo output parsing) vs macOS rejecting `C.UTF-8`, git-CLI auth for
private crate fetches (`CARGO_NET_GIT_FETCH_WITH_CLI`), and macOS
SDK/deployment-target pinning to match CRAN's toolchain. If you must edit
it, keep env pins scoped per-job (Linux-only settings break macOS jobs and
vice versa).

## Version bumps

Keep `DESCRIPTION` `Version:` and `src/rust/Cargo.toml` `version` in
lockstep; the scaffold ships a helper:

```sh
Rscript tools/bump-version.R --sync
```

## webR / wasm note

wasm32 builds cannot generate wrappers (the wasm module can't be loaded by
host R at build time), so they consume the pre-generated
`R/<pkg>-wrappers.R` + `src/rust/wasm_registry.rs` left on disk by your last
native build (both are gitignored in scaffolded packages — they travel with
the source tree, not through git). Always run a native `miniextendr_build()`
before producing a wasm artifact, so those files are current.
`minirextendr::miniextendr_webr_import_lint()` flags namespace-level imports
of compiled packages that break under webR.

## Pitfalls

- **Checking the source dir instead of the tarball** — hides CRAN failures;
  always check the built `.tar.gz`.
- **Tracking `inst/vendor.tar.xz` in git** — 20+ MB binary churn per commit
  and stale-after-merge drift. It's build output; regenerate per release.
- **Hand-editing the tarball's contents** — regenerate via a clean
  `devtools::build()` instead; the vendor tree, lockfile, and manifest must
  agree.
- **Assuming `--as-cran` covers the offline story** — it doesn't fully; do
  the temp-library install (step 4) at least once per release.

Full manual (CRAN compatibility chapter): https://a2-ai.github.io/miniextendr
