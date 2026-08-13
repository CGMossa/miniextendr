# Row-name branch aggregate check reached the known rpkg blocker

## What was attempted

Run the repository-wide `just check` and `just test` gates after the row-name
getter fix passed all three exact CI clippy configurations.

## What went wrong

For `just check`, the root workspace and both cross-package workspaces passed.
For `just test`, the root suite, the ndarray feature legs, and both
cross-package suites passed. The final standalone rpkg workspace failed in both
commands because `ColumnarFrame` is imported twice in
`dataframe_enum_payload_matrix.rs` and `dataframe_collections_test.rs`.

## Root cause

The duplicate generated imports are present on the base branch and are already
isolated in PR #1389. They are unrelated to the row-name getter implementation
or fixtures.

## Fix

Keep the generated-import repair in PR #1389 instead of duplicating it here.
Verify this branch with the passing earlier aggregate legs, all exact clippy
gates, focused public R tests, and gctorture coverage.
