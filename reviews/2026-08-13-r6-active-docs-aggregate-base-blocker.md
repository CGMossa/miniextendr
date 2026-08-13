# R6 documentation branch aggregate gates reached the known rpkg blocker

## What was attempted

Run the repository-wide `just check` and `just test` gates after all three exact
CI clippy configurations and the focused macro/R tests passed.

## What went wrong

For `just check`, the root workspace and both cross-package workspaces passed.
For `just test`, the root suite, ndarray feature legs, and both cross-package
suites passed. The final standalone rpkg workspace failed in both commands
because `ColumnarFrame` is imported twice in `dataframe_enum_payload_matrix.rs`
and `dataframe_collections_test.rs`.

## Root cause

The duplicate generated imports are present on the base branch and are already
isolated in PR #1389. They are unrelated to the R6 documentation generator.

## Fix

Keep the generated-import repair in PR #1389 instead of duplicating it here.
Verify this branch with the passing earlier aggregate legs, exact clippy gates,
focused macro tests, warning-free forced documentation, and public R6 tests.
