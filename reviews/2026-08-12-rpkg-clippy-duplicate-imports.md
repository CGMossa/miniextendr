# rpkg clippy gate exposed duplicate imports

## What was attempted

Ran the proposed `rpkg/src/rust` CI gate locally with its full feature set and
`-D warnings`:

```text
just clippy-rpkg --features full -- -D warnings
```

## What went wrong

The fixture crate failed with E0252 and `unused_imports` because
`ColumnarFrame` was imported twice in each of two test modules.

## Root cause

Commit `d557ac39` (#1380) intended to move the imports under `#[cfg(test)]`, but
the target modules already contained the imports. The change added a second
copy instead of relocating the top-level copy, leaving main unable to pass the
exact command that #1380 was meant to make warning-clean.

## Fix

Removed the two duplicate imports and reran the full-feature clippy gate. The
new CI step now prevents the standalone fixture workspace from regressing
silently again.
