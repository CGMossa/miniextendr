# Factor API docs caused unrelated report churn

## What was attempted

The generated Rust API corpus was refreshed on top of a newly clean baseline
after removing two factor view types.

## What went wrong

The factor-only source change still produced roughly three thousand changed
lines. Unrelated re-exports and conversion impls repeatedly swapped positions,
including entries in files untouched by the factor refactor.

## Root cause

The Python renderers used stable sorts with non-unique keys and therefore
inherited rustdoc JSON object order for ties. Rustdoc emitted a different item
order in the second worktree.

## Fix

Make each report ordering total with semantic secondary keys, add reversed-index
regression tests for all three affected renderers, and refresh the corpus into
the canonical order before stacking the factor change.
