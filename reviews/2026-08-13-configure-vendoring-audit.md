# Configure-time vendoring scaffold audit

**Date:** 2026-08-13
**Status:** Root cause confirmed and fixed.

## What was attempted

Dogfood a freshly generated, non-Git miniextendr package in both ordinary
source mode and with `use_local_miniextendr()`, then compare it with the same
package initialized as a Git checkout.

## What went wrong

The non-Git package ran `cargo revendor` during `bash ./configure`, created
`inst/vendor.tar.xz`, and selected tarball mode. The Git package did not. In
the local-override case this also made the newly written source override inert.

The first audit harness attempts had three setup errors: an environment
assignment was placed after `Rscript` rather than before the command, autoconf
was invoked from the repository root instead of the scaffold directory, and an
accidental `/tmp/.git` changed every `/tmp` fixture's ancestry. The accidental
repository was moved to Trash (recoverable), then the reproduction was rerun
with explicit working directories and per-command environment setup.

## Root cause

Both scaffold `configure.ac` files contained an install-time “self-repair”
block keyed on the absence of a `.git` ancestor. It invoked `cargo revendor`
before the normal file-existence mode check. This made source-tree location a
hidden mode flag and let configure mutate the package into a vendored build.
Several tests created fake `.git` directories specifically to suppress that
behavior, so the suite encoded the workaround instead of the contract.

## Fix

Delete configure-time vendoring. Configure now only consumes an existing
`inst/vendor.tar.xz`; explicit tarball-building workflows remain responsible
for producing it. Regression tests exercise true non-Git scaffolds and fail if
the fake `cargo-revendor` sentinel is touched or a vendor tarball appears.

## Verification hiccup

The first `just minirextendr-document` attempt failed because the worktree's R
library had not been populated for the newly selected R 4.6 installation, so R
could not load `devtools`. Re-running the mandated setup—`rig default 4.6`,
verifying `R 4.6.0`, then `just worktree-sync`—restored the isolated dependency
library. Documentation generation then completed successfully.

The first full `just minirextendr-test` run also appeared idle after entering
the template end-to-end cases. This was not a test deadlock: locally,
`skip_e2e()` deliberately permits cold Rust compilation and network-dependent
round trips; only CI skips them. An accidental duplicate run was stopped. The
per-PR shape was rerun with `CI=true` and completed with 797 passes, 11 expected
skips, no failures, and no warnings. The affected standalone workflow was then
run separately outside CI via `test-scaffold-smoke.R`; all six real vendoring
and offline-check expectations passed.

The first `just minirextendr-check` attempt likewise inherited the local-only
E2E behavior and was interrupted after a long silent build. Re-running with
`CI=true` completed in 56 seconds with zero errors and zero warnings. It exposed
two pre-existing R CMD check notes—`AGENTS.md` at package top level and an
unknown self-package Rd cross-reference—which are separate audit findings, not
vendoring changes. They are tracked as #1409 and #1410, respectively. All R
commands used the `rproject.toml` pin (`4.6`, resolved as R 4.6.0); the root
instructions now explicitly make that file, rather than a version copied into
prose, authoritative.
