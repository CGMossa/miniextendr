# LLM-doc regeneration read a different target directory than Cargo wrote

## What was attempted

Run `just llm-docs` after changing `miniextendr-engine` public rustdoc.

## What went wrong

All rustdoc builds completed, but the renderer failed because
`target/doc/miniextendr_api.json` did not exist.

## Root cause

Cargo was configured to write artifacts under `/Users/elea/.cargo-target`, as
confirmed by `cargo metadata`. The generation script hard-codes paths under
the checkout's `target/`, so its producer and consumer disagreed about the
artifact location.

## Fix

For this branch, generation was rerun with the relative
`CARGO_TARGET_DIR=target`. Cargo resolves that relative to each workspace, so
the root JSON landed in `target/doc/` and the standalone cargo-revendor JSON in
`cargo-revendor/target/doc/`, matching both renderer paths. Generation then
completed. Corpus-wide drift unrelated to this branch remains handled by PR
#1400; only the regenerated engine digest and impl inventory were retained.
The durable script fix is tracked in #1413 so it does not expand the
engine-handle PR.
