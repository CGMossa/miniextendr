# Renderer fixtures could not use the read-only sandbox

## What was attempted

The renderer unit suite was run after adding subprocess tests that write small
rustdoc JSON fixtures through `tempfile.TemporaryDirectory`.

## What went wrong

Both new tests failed before invoking a renderer because Python found no
writable temporary directory in the read-only sandbox.

## Root cause

The command was run without the project-required full-access mode for tests
that create files. The repository code and test fixtures were not at fault.

## Fix

Rerun the same unit suite with full filesystem access. All nine tests passed.
