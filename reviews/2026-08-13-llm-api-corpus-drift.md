# Factor documentation regeneration exposed stale API corpus

## What was attempted

The Rust API corpus was regenerated after removing two unsound factor view
types, as required for public API changes.

## What went wrong

Instead of a factor-only documentation delta, generation changed more than two
thousand lines across the API, macro, CLI, and implementation-inventory
reports.

## Root cause

Several merged public API changes after the last corpus refresh did not commit
their regenerated `rust-llm-docs/generated/` output. The factor change merely
made that accumulated drift visible.

## Fix

Refresh the corpus from unmodified current `origin/main` in a dedicated branch,
verify a second generation is byte-for-byte stable, and stack the factor API
change on that clean generated baseline rather than mixing unrelated drift into
its PR.
