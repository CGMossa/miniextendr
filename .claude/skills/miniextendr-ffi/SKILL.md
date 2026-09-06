---
name: miniextendr-ffi
description: "Use when the user asks about the FFI safety layer in miniextendr: #[r_ffi_checked] proc macro, checked vs _unchecked FFI variants, when _unchecked is safe, with_r_unwind_protect / with_r_unwind_protect_or_raise, GC protection (OwnedProtect, ProtectScope), the R longjmp leak in the tagged-condition path, MXL300 / MXL301 lint rules, the nonapi feature gate, or the continuation token."
---

# miniextendr FFI Layer

The FFI layer is the boundary where Rust code calls into R's C API. R uses `longjmp` for error handling (which skips Rust destructors), has a single-threaded GC, and has no knowledge of Rust panics. miniextendr provides three interlocking safety layers: thread-checking FFI wrappers, unwind protection, and GC protection.

## When to use this skill

- "What does `#[r_ffi_checked]` do?"
- "When can I use `Rf_allocVector_unchecked` instead of `Rf_allocVector`?"
- "What is `with_r_unwind_protect` and when do I use it?"
- "What is the ~8 byte leak on the R longjmp path?"
- "What is the difference between `with_r_unwind_protect` (default) and `with_r_unwind_protect_or_raise` (legacy)?"
- "What are `OwnedProtect` and `ProtectScope`?"
- "Why does MXL300 flag my `Rf_error` call?"
- "What is the `nonapi` feature?"

## Key concepts

### `#[r_ffi_checked]` — thread-checked FFI wrappers

`#[r_ffi_checked]` is a proc macro applied to `unsafe extern "C-unwind"` blocks in `miniextendr-api/src/sys.rs`. For every function declared in the block, it generates two variants:

- `fn_name(args)` — the checked variant. Calls `with_r_thread(|| …)` first (a debug assertion that the current thread is R's main thread), then delegates to the underlying C symbol.
- `fn_name_unchecked(args)` — the raw variant. No thread check. Same as calling the C symbol directly.

Static symbols (e.g., `R_NilValue`, `R_NaString`) pass through unchanged — no checked/unchecked split, because reading a static is always safe.

The check is a debug assertion only (fires in debug builds and in CI). In release builds the overhead is zero: the thread-ID comparison compiles away.

When to use `_unchecked`:

- Inside ALTREP callbacks (already on main thread by R's own dispatch).
- Inside `with_r_unwind_protect` closures (the closure runs on the main thread).
- Inside `with_r_thread` closures (same).
- Any site where you have already established the main-thread invariant.

MXL301 enforces this: using `_unchecked` outside these known-safe contexts is a lint error. See `miniextendr-lint`.

The `^nonapi^` annotation in `sys.rs` marks functions that require `#[cfg(feature = "nonapi")]`. These are R internals not part of the stable public API.

### `with_r_unwind_protect` — the default for `#[miniextendr]` functions

This is the **only** transport for all `#[miniextendr]` functions and methods. Instead of longjmping on panic, it returns a tagged-SEXP condition value. The generated R wrapper inspects this SEXP and calls `stop(structure(…, class = c("rust_error", "simpleError", "error", "condition")))` on the R side. This gives full `rust_*` class layering accessible to `tryCatch`.

Mechanics (`miniextendr-api/src/unwind_protect.rs`):

1. Runs the closure inside a `catch_unwind` trampoline.
2. If the closure panics, builds a tagged-condition SEXP via `make_rust_condition_value`; the R-side wrapper raises a structured `rust_*` condition.
3. If R longjmps inside the closure, the cleanup handler fires and `R_ContinueUnwind` re-propagates the longjmp.
4. Rust destructors run via `drop(data)` before all diverging paths.

### `with_r_unwind_protect_or_raise` — legacy panics-as-R-error variant

Kept for explicit framework callers (test fixtures, benchmarks, trait-ABI vtable shims) that need panics converted directly to an R error via `raise_rust_condition_via_stop` (Approach 3: `Rf_eval(stop(structure(…)))`) — diverges via longjmp. **Not used by `#[miniextendr]` codegen.**

### The ~8 byte longjmp-path leak

`with_r_unwind_protect` leaks approximately 8 bytes (an `RErrorMarker` marker struct + a `Box` header) on the R longjmp path through `R_ContinueUnwind`. This is because `R_ContinueUnwind` longjmps and Rust cannot reclaim the box through normal drop. Regular Rust panics do not leak.

This is a known, accepted trade-off. The MXL300 lint rule discourages calling `Rf_error`/`Rf_errorcall` directly (use `panic!()` instead) precisely to avoid bypassing the framework's PROTECT-discipline and introducing additional leak sites.

### Continuation token

`get_continuation_token()` returns a globally shared SEXP created once via `R_MakeUnwindCont()` and permanently preserved with `R_PreserveObject`. Using a single global token avoids leaking one token per thread. The function is idempotent (`OnceLock`). Declared in `miniextendr-api/src/unwind_protect.rs`.

### GC protection — `OwnedProtect` and `ProtectScope`

R's GC can run at any R API call that allocates. Fresh SEXPs returned by `IntoR`, `SEXP::scalar_string`, `Rf_allocVector`, etc. are unprotected until explicitly protected. If a GC runs before you protect a SEXP, the pointer becomes dangling.

Two RAII types from `miniextendr-api/src/gc_protect.rs`:

- **`OwnedProtect`**: wraps a single SEXP. On creation, calls `Rf_protect`. On drop, calls `Rf_unprotect(1)`. Use for a single transient SEXP:

  ```rust
  let _guard = unsafe { OwnedProtect::new(my_sexp) };
  // my_sexp is protected until _guard is dropped
  ```

- **`ProtectScope`**: RAII scope for batch protect/unprotect. Each call to `scope.protect(x)` calls `Rf_protect` and returns a `Root<'_>` lifetime-tied to the scope. On scope drop, calls `Rf_unprotect(n)` for all protected values:

  ```rust
  let scope = unsafe { ProtectScope::new() };
  let a = unsafe { scope.protect(sexp_a) };
  let b = unsafe { scope.protect(sexp_b) };
  // a and b are valid until scope drops
  ```

For long-lived SEXPs that outlive a single `.Call` frame, use `R_PreserveObject` / `R_ReleaseObject` (wrapped by the `Root` handle in `miniextendr-api/src/gc_protect.rs`).

### Panic telemetry

Every site that converts a panic to an R error fires `crate::panic_telemetry::fire(message, source)`. The four sources are `PanicSource::Worker`, `PanicSource::Altrep`, `PanicSource::UnwindProtect`, and `PanicSource::Connection`. Telemetry hooks can be registered via `set_panic_telemetry_hook` for diagnostics. See `miniextendr-api/src/panic_telemetry.rs`.

## How it works

### The GC safety discipline

The common pattern for allocating multiple transient SEXPs:

1. Allocate SEXP A.
2. `Rf_protect(A)`.
3. Allocate SEXP B.
4. `Rf_protect(B)`.
5. Use A and B.
6. `Rf_unprotect(2)`.

Or with `OwnedProtect`:

1. `let a = Rf_allocVector(…)`.
2. `let _guard_a = OwnedProtect::new(a)`.
3. `let b = Rf_allocVector(…)`.
4. `let _guard_b = OwnedProtect::new(b)`.
5. Use a and b.
6. Guards drop in reverse order → `Rf_unprotect(1)` × 2.

The critical rule: any R API call between allocating a SEXP and using it must be guarded. R-devel's GC is more aggressive than R-release and will trigger inside that window. R-release passing is NOT proof of safety.

### `_unchecked` inside ALTREP callbacks

ALTREP callbacks receive raw SEXP arguments from R's runtime — they are always on the main thread. Using the checked `Rf_mkCharCE` variant inside an ALTREP `elt` callback would correctly pass the thread check, but adds a debug assertion that is redundant at that call site. In performance-sensitive callbacks, use `Rf_mkCharCE_unchecked` (MXL301 will not fire because you are inside an ALTREP callback).

## Decision trees

### I'm calling R from Rust — which variant do I use?

- Am I on R's main thread?
  - Yes, and I know it (inside `with_r_thread`, ALTREP callback, `with_r_unwind_protect` trampoline): use `fn_name_unchecked(…)` — no redundant check, no MXL301 warning.
  - Yes, but uncertain: use `fn_name(…)` — debug assertion catches mistakes in development.
  - No (worker thread): use `with_r_thread(|| fn_name_unchecked(…))` — routes the call to the main thread.

### I'm in an ALTREP callback — which FFI?

Use `_unchecked` variants. The callback is guaranteed on the main thread by R's dispatch. Guard mode (`r_unwind` / `rust_unwind` / `unsafe`) controls whether panics are caught; it does not affect which FFI variant you call.

### I'm raising an R error — which mechanism?

- Inside `#[miniextendr]` function (default path): `panic!("message")` — the framework converts via `with_r_unwind_protect` to a tagged SEXP → R wrapper raises `stop(structure(…))`. Gives full `rust_*` class layering.
- Inside ALTREP callback: `panic!("message")` — `with_r_unwind_protect_sourced` intercepts (if guard is `r_unwind` or `rust_unwind`) and calls `raise_rust_condition_via_stop` (Approach 3). Same class layering.
- Explicit condition: use `miniextendr_api::error!("message")` or `miniextendr_api::warning!(…)` — emits a `RCondition` payload recognised by the error transport.
- Never call `Rf_error` / `Rf_errorcall` directly — MXL300 flags this. The framework's transport is safer (correct PROTECT discipline, class layering) and avoids the ~8 byte leak.

### I need to protect a SEXP across an R API call — which tool?

- Single short-lived SEXP, drops at end of current block: `OwnedProtect`.
- Multiple SEXPs with the same lifetime, all drop together: `ProtectScope`.
- SEXP outlives the current `.Call` frame (e.g., stored in an R6 object or ExternalPtr): `R_PreserveObject` / `R_ReleaseObject` via the `Root` handle (see `miniextendr-api/src/gc_protect.rs`).

## Key files

- `miniextendr-api/src/sys.rs` — all R API declarations under `#[r_ffi_checked]`. Blocks at lines 248, 787, 956, 1126, 1509, 1572, 1681, 1955.
- `miniextendr-api/src/unwind_protect.rs` — `with_r_unwind_protect` (default), `with_r_unwind_protect_or_raise` (legacy), `with_r_unwind_protect_shim`, `with_r_unwind_protect_sourced`, `raise_rust_condition_via_stop`, `get_continuation_token`.
- `miniextendr-api/src/ffi_guard.rs` — `GuardMode`, `guarded_ffi_call`, `guarded_ffi_call_with_fallback`.
- `miniextendr-api/src/gc_protect.rs` — `OwnedProtect`, `ProtectScope`, `Root`, `ReprotectSlot`, `tls` convenience module.
- `miniextendr-api/src/panic_telemetry.rs` — `PanicSource`, `fire()`, telemetry hook registration.
- `miniextendr-api/src/error_value.rs` — `make_rust_condition_value`, tagged-SEXP format.

## Common pitfalls

- **Direct `Rf_error` call (MXL300)**: bypasses the tagged-SEXP transport, breaks `rust_*` class layering, and can introduce protect-discipline issues. Use `panic!()` inside `#[miniextendr]` fns; use `raise_rust_condition_via_stop` only from ALTREP callbacks (already done automatically by the guard mode machinery).

- **`_unchecked` outside safe context (MXL301)**: using `fn_unchecked` in a regular `#[miniextendr]` function body (not inside `with_r_thread` or an ALTREP callback) is flagged. The main-thread invariant is not statically established at that site.

- **Unprotected SEXP across R API call**: allocating a SEXP and then calling any R API that allocates before protecting the first one is a GC hazard. R-devel will segfault; R-release may silently pass. Always protect immediately after allocation.

- **Matching `Rf_protect` / `Rf_unprotect` counts**: R's protect stack is a raw count. Mismatching protect/unprotect counts corrupts the stack. Prefer RAII (`OwnedProtect`, `ProtectScope`) to manual counting.

- **`catch_unwind` does not catch R longjmps**: wrapping in `catch_unwind` alone is insufficient if the closure calls R API. Use `with_r_unwind_protect` instead, which installs an `R_UnwindProtect` cleanup handler.

- **`nonapi` feature gate**: some R internal functions (not in the stable public API) are declared with `^nonapi^` in `sys.rs`. Calling them requires `features = ["nonapi"]` in `Cargo.toml`. Without it, the checked wrapper bodies are absent (`cfg(not(feature = "nonapi"))`).

## Related skills

- `miniextendr-worker` — `with_r_thread`, `run_on_worker`, `Sendable<T>`, how closures cross thread boundaries.
- `miniextendr-altrep` — guard modes that route through the same unwind-protect machinery.
- `miniextendr-macros` — codegen for `#[miniextendr]` functions; how the generated R wrapper inspects the tagged-SEXP.
- `miniextendr-lint` — MXL300 (direct Rf_error) and MXL301 (unchecked FFI outside safe context) rules.
