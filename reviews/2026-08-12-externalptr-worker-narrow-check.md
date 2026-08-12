# ExternalPtr worker fixture narrow-check retries

## Attempt

Compile the new rpkg fixture early with `cargo lcheck --lib --locked` from the
configured `rpkg/src/rust` workspace.

## What went wrong

The first attempt stopped because the tracked tarball-shape `Cargo.lock` cannot
be updated under `--locked` after source-mode configure activates path patches.
Removing `--locked` exposed a second setup omission: rpkg has no default Cargo
features, while explicit `#[miniextendr(worker)]` routines require its
`worker-thread` feature.

## Root cause

The narrow command did not reproduce the feature and lock-shape setup supplied
by the package build. Neither failure came from the new fixture implementation.

## Fix

Run `cargo lcheck --lib --features worker-thread` for the early narrow compile,
then restore the temporary `rpkg/src/rust/Cargo.lock` reshaping. The package
install remains the authoritative end-to-end check because configure supplies
the complete detected feature list.
