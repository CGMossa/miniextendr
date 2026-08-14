# `configure` created vendor tarballs outside tarball builds

**Date:** 2026-08-13
**Status:** Confirmed; fixed on `fix/configure-no-auto-vendor`.

## Finding

A freshly scaffolded package outside a Git checkout ran `cargo revendor` from
`bash ./configure` whenever `cargo-revendor` was on `PATH`. Configure created
`inst/vendor.tar.xz`, immediately selected tarball mode, and could ignore a
`.miniextendr-local` source override.

That violates the install contract: configure chooses between source mode and
an existing vendored-tarball mode. It must never produce the mode signal.
Vendoring is reserved for workflows that are deliberately creating a package
tarball (`just vendor`, `miniextendr_vendor()`, or `bootstrap.R` before
`R CMD build`).

## Reproduction

1. Scaffold a standalone package outside any `.git` ancestor.
2. Put `cargo-revendor` on `PATH`.
3. Run `bash ./configure` with no `inst/vendor.tar.xz` present.
4. Observe `cargo revendor` run, `inst/vendor.tar.xz` appear, and configure
   report tarball/offline mode.

The same package placed under Git stayed in source mode, showing that Git
ancestry—not the presence of a build artifact—had accidentally become a mode
decision.

## Fix

- Removed all `cargo revendor` execution from the rpkg and standalone scaffold
  `configure.ac` templates.
- Kept `[ -f inst/vendor.tar.xz ]` as the sole install-mode signal.
- Removed test-only `.git` directories that had hidden the defect.
- Added behavioral tests with a fake `cargo-revendor` sentinel proving that
  non-Git scaffolds remain in source mode, both with and without a local
  miniextendr override.
- Corrected project, release, debugging, scaffold, and CRAN documentation to
  separate tarball production from configure-time consumption.

