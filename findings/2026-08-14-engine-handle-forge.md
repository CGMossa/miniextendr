# `REngine` can be forged without initializing R

**Tracked as:** https://github.com/A2-ai/miniextendr/issues/1412

## Summary

`miniextendr-engine` documents `REngine` as a marker proving that embedded R
has been initialized for the process, but declares it as a public unit struct:

```rust
pub struct REngine;
```

That exports the value constructor. Safe downstream code can write
`let engine = miniextendr_engine::REngine;` while
`r_initialized_sentinel()` is still false. The type therefore does not uphold
the invariant its API and consumers assign to it. The benchmark stores the
handle in a process-wide `OnceLock`, so the codebase already treats successful
construction as initialization evidence.

The user-facing embedding example in `docs/ENTRYPOINT.md` exposes the same
contract drift from the opposite direction: it calls
`REngine::build().unwrap()`, although `build()` only returns a builder, and
then claims runtime initialization happened automatically. The real consumer
paths call unsafe `.init()` and separately call
`miniextendr_runtime_init()` before using `miniextendr-api` FFI.

## Reproduction

This compiles without calling R:

```rust
use miniextendr_engine::{REngine, r_initialized_sentinel};

let _forged = REngine;
assert!(!r_initialized_sentinel());
```

The central `docs/ENTRYPOINT.md` example does not compile because
`REngineBuilder` has no `unwrap()` method.

## Suggested fix

- Give `REngine` a private field and construct it only after
  `setup_Rmainloop()` completes.
- Add a compile-fail doctest for direct construction.
- Turn the engine quick-start snippets from ignored examples into compiled
  `no_run` doctests.
- Correct the embedding guide to call unsafe `.init()` and, when using
  `miniextendr-api`, explicitly register the initialized thread with
  `miniextendr_runtime_init()`.

## Overlap audit

Closed issue #974 fixed an older `REngine::new()` rustdoc typo, but did not
cover public construction or the current `docs/ENTRYPOINT.md` example. Open
issue #1352 concerns off-main-thread R API escape hatches; it does not cover
the handle's construction invariant.
