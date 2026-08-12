# Aggregate Rust check stopped at existing rpkg duplicate imports

## What was attempted

`just check` was run after the factor soundness refactor. It checks the root
workspace, both cross-package fixture workspaces, and the standalone
`rpkg/src/rust` workspace.

`just test` was then run to exercise the same multi-workspace boundary with
runtime tests.

## What went wrong

The root workspace and both cross-package workspaces passed. The standalone
rpkg test target then failed with E0252 and unused-import warnings for duplicate
`ColumnarFrame` imports in:

- `dataframe_enum_payload_matrix.rs:983` / `:985`; and
- `dataframe_collections_test.rs:354` / `:356`.

`just test` reached the identical final blocker after the root suite (including
329 miniextendr-api unit tests), integration and doc tests, the dedicated
ndarray legs, and both cross-package workspaces reported zero failures.

## Root cause

These duplicate imports are present on the parent branches and unrelated to
`factor.rs`. Their removal and the missing rpkg clippy gate are already isolated
in PR #1389.

## Fix

Do not duplicate #1389 in the factor PR. Verify this branch through the root
workspace's exact three CI clippy configurations, factor-focused Rust tests,
installed-package factor/Arrow tests, gctorture dogfood, and the aggregate
recipe portions that run before the shared rpkg blocker.
