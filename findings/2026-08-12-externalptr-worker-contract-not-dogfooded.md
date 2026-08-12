# ExternalPtr worker transfer was documented but not dogfooded

## Finding

`ExternalPtr<T>` implements `Send` when `T: Send`, and its module-level thread
safety contract says that this exists to move handles through miniextendr's
worker execution path. The R-package fixtures did not exercise that contract.

The tests named "ExternalPtr from worker context" computed plain Rust values on
the worker, returned those values to the R main thread, and only then created
the external pointers. The fixture source reinforced the blind spot with the
stale claim that `ExternalPtr` was `!Send`.

No test covered any of the three distinct worker-sensitive operations:

- reading the cached pointee after an R-owned handle crosses to the worker;
- returning that borrowed handle and preserving the original R SEXP identity;
- consuming the handle with `ExternalPtr::into_inner`, whose checked R API
  calls must route back to the main thread and clear the R pointer exactly once.

## Risk

The worker boundary is where an apparently safe refactor can turn into an
off-main-thread R API call, an unrooted handle, a deep copy that breaks R object
identity, or a double finalization. Main-thread-only tests cannot detect those
regressions.

## Correction

Add real `#[miniextendr(worker)]` fixtures for read, identity round-trip, and
`into_inner`, then invoke them through the installed R package. Also remove the
stale `!Send` comment from the older value-computation fixture.

The tests deliberately use an R-owned pointer as input. This covers the actual
generated-wrapper path: convert on the main thread, move the borrowed handle to
the worker, and convert the result back on the main thread.
