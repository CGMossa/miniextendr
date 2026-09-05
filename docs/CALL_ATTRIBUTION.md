# Call Attribution and `match.call()`

Generated R wrappers pass `.call = match.call()` into every `.Call()` so that errors raised from Rust are attributed to the user's call frame, with formal parameters matched by name. This page shows the difference using a real, runnable fixture.

## The fixture

`rpkg/src/rust/call_attribution_demo.rs` defines two functions that raise the same error message. Only the wrapper differs.

```rust
// Wrapped path. Generated R wrapper passes `.call = match.call()` into the
// C entry; on panic, `Rf_errorcall(call, msg)` shows the user's call frame.
#[miniextendr]
pub fn call_attr_with(_left: i32, _right: i32) -> i32 {
    panic!("left + right is too risky")
}

// Unwrapped path. `extern "C-unwind"` bypasses the wrapper entirely — there is
// no call slot and no `with_r_unwind_protect`. We raise an R error directly
// with `Rf_error`, which carries no call attribution.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_call_attr_without(_left: SEXP, _right: SEXP) -> SEXP {
    unsafe {
        ::miniextendr_api::sys::Rf_error(
            c"%s".as_ptr(),
            c"left + right is too risky".as_ptr(),
        )
    }
}
```

Generated R wrappers (excerpted from `rpkg/R/miniextendr-wrappers.R`):

```r
call_attr_with <- function(left, right) {
  # ... preconditions ...
  .val <- .Call(C_miniextendr_call_attr_with, .call = match.call(), left, right)
  # ... tagged-condition demux ...
}

unsafe_C_call_attr_without <- function(left, right) {
  .val <- .Call(C_call_attr_without, left, right)   # no .call slot
  # ...
}
```

The `extern "C-unwind"` path is registered directly as the `.Call` symbol, so there is nowhere to thread a call SEXP.

## The transcript

These two functions are internal demo fixtures (not exported); the `:::` prefix is intentional.

```r
> library(miniextendr)

> miniextendr:::call_attr_with(1L, 2L)
Error in miniextendr:::call_attr_with(left = 1L, right = 2L) :
  left + right is too risky

> miniextendr:::unsafe_C_call_attr_without(1L, 2L)
Error in miniextendr:::unsafe_C_call_attr_without(1L, 2L) :
  left + right is too risky
```

Wrapped inside another function so the difference is sharper:

```r
> outer_with <- function(x) miniextendr:::call_attr_with(x, x + 1L)
> outer_with(5L)
Error in miniextendr:::call_attr_with(left = x, right = x + 1L) :
  left + right is too risky

> outer_without <- function(x) miniextendr:::unsafe_C_call_attr_without(x, x + 1L)
> outer_without(5L)
Error in miniextendr:::unsafe_C_call_attr_without(x, x + 1L) :
  left + right is too risky
```

Programmatic comparison via `tryCatch`:

```r
> e_with    <- tryCatch(outer_with(5L),    error = identity)
> e_without <- tryCatch(outer_without(5L), error = identity)

> class(e_with)
[1] "rust_error"   "simpleError"  "error"        "condition"
> class(e_without)
[1] "simpleError"  "error"        "condition"

> conditionCall(e_with)
miniextendr:::call_attr_with(left = x, right = x + 1L)
> conditionCall(e_without)
miniextendr:::unsafe_C_call_attr_without(x, x + 1L)
```

## What `.call = match.call()` buys you

1. **Formal parameter names matched.** `call_attr_with(left = 1L, right = 2L)` reads better than `call_attr_with(1L, 2L)` and is robust to positional vs. named call style.
2. **Public function name, not internal symbol.** Errors blame `call_attr_with`, not `miniextendr:::unsafe_C_call_attr_without`. Triple-colon paths in error messages are a leak of internals.
3. **Structured `rust_error` class.** The wrapped path goes through the tagged-condition decoder and returns a condition with class `rust_error`, which downstream handlers can catch specifically. The unwrapped path produces a plain `simpleError`.
4. **Stable across nesting.** Whether the user calls the function directly or from another function, `match.call()` always captures the *immediate* caller's expression, with the call written as the user wrote it.

## How it flows end to end

```text
R user calls:  call_attr_with(1L, 2L)
                       │
                       ▼
R wrapper:     .Call(C_miniextendr_call_attr_with, .call = match.call(), left, right)
                       │
                       ▼
C wrapper:     extern "C-unwind" fn(__miniextendr_call: SEXP, left: SEXP, right: SEXP)
                  │
                  └── with_r_unwind_protect(closure, Some(__miniextendr_call))
                          │
                          ▼
                     Rust panic caught
                          │
                          ▼
                     Rf_errorcall(__miniextendr_call, "left + right is too risky")
                          │
                          ▼
                     R error: "Error in call_attr_with(left = 1L, right = 2L) : ..."
```

For the unwrapped extern `"C-unwind"` path, the call slot does not exist, so the panic-to-error path becomes `Rf_error(msg)` and R falls back to `sys.call()` of the wrapper frame.

## Internal entry points: caller attribution

`noexport` exists for package-internal entry points, and the natural layout is a
hand-written R function that validates its arguments and delegates to the
generated wrapper. With the default attribution the user then sees the bridge:

```text
Error in call_attr_self_impl(x = value) : x must be positive, got -1
```

`#[miniextendr(noexport, call = caller)]` moves the attribution one frame up.
The wrapper resolves its parent frame in its own frame first, then hands that
call to `.Call()` and to the raise fallback:

```r
call_attr_caller_impl <- function(x) {
  .mx_parent <- sys.parent()
  .mx_def <- if (.mx_parent > 0L) sys.function(.mx_parent)
  .mx_pc <- if (.mx_parent > 0L) sys.call(.mx_parent)
  .mx_call <- if (typeof(.mx_def) == "closure") match.call(.mx_def, .mx_pc) else match.call()
  .val <- .Call(C_mypkg_call_attr_caller_impl, .call = .mx_call, x)
  if (inherits(.val, "rust_condition_value") && ...) return(.miniextendr_raise_condition(.val, .mx_call))
  .val
}
```

so the condition carries the hand-written function's call with *its* formals
matched:

```text
Error in call_attr_caller(value = -1L) : x must be positive, got -1
```

The wrapper falls back to its own `match.call()` in two cases: `sys.parent()`
is `0` (called from top level, also under `tryCatch()` there), or the parent
frame's function is not a closure. The second case covers `eval()`'d code (a
testthat block, `source()`, `local()`): R gives such a frame the `eval`
primitive as its function, and `match.call()` rejects a non-closure
`definition`. Every `sys.*` lookup is a plain statement in the wrapper's own
frame on purpose: inside a promise forced by `match.call()`, `sys.call(0)`
resolves to `match.call`'s own frame, not the wrapper's. The option requires
`noexport` or `internal` (an exported function's caller is arbitrary user
code), cannot combine with `no_call_attribution` / `fast` (no call slot to
redirect), and applies to standalone functions only; class methods keep their
own attribution. The frame is the *calling* frame, so an entry point reached
through `lapply()` / `do.call()` reports that frame's call, the same way
`sys.call(-1)` would. Fixture pair:
`call_attr_caller_impl` / `call_attr_self_impl` in
`rpkg/src/rust/call_attribution_demo.rs` with the delegates in
`rpkg/R/call_attribution.R`, verified by `test-call-attribution.R`.

## Where this is emitted

Every `.Call()` inside generated R wrappers goes through one source of truth: `DotCallBuilder` in `miniextendr-macros/src/r_wrapper_builder.rs`, which always prepends `.call = match.call()`. The C wrapper builder in `miniextendr-macros/src/c_wrapper_builder.rs` always declares `__miniextendr_call: SEXP` as the first parameter, so the convention is symmetric.

It applies uniformly to:

- Standalone `#[miniextendr]` functions
- All six class systems (R6, S3, S4, S7, Env, Vctrs) — constructors, instance methods, static methods, active bindings, finalizers, `deep_clone`
- All trait implementations across all class systems
- `match_arg` choices helper calls
- Sidecar `Type_get_field` / `Type_set_field` accessors generated by `#[derive(ExternalPtr)]`

## Where it is intentionally absent

- **`extern "C-unwind"` functions** registered directly with `#[miniextendr]`. The function *is* the C entry point — there is no generated wrapper and no call slot. This is the demo above. Use only for low-level fixtures and tests where you control the error path manually.
- **`vctrs_derive` boilerplate** — `format.<class>`, `vec_ptype2.<class>.<class>`, etc. — pure R, no `.Call()`.

## Where `.call = NULL` is used instead of `match.call()`

Five lambda dispatch sites cannot use `match.call()` because the lambda is invoked by R6/S7 dispatch machinery, not by user code. `match.call()` inside those lambdas would capture the dispatch frame (e.g., `R6$finalize()`, `S7::prop_get()`), not the user's `obj$field` access. The generated `.Call()` instead passes `.call = NULL`. The `%||% sys.call()` fallback in `condition_check_lines` then surfaces the nearest meaningful frame.

The five sites are:

1. **R6 finalizer** — `finalize = function() .Call(C_mypkg_Type__finalize, .call = NULL, private$.ptr)`
2. **R6 `deep_clone`** — `deep_clone = function(name, value) .Call(C_mypkg_Type__deep_clone, .call = NULL, private$.ptr, name, value)`
3. **S7 property validator** — `validator = function(value) .Call(C_mypkg_Type__validate_prop, .call = NULL, value)`
4. **S7 property getter** — `getter = function(self) .Call(C_mypkg_Type__get_prop, .call = NULL, self@.ptr)`
5. **S7 property setter** — `setter = function(self, value) { .Call(C_mypkg_Type__set_prop, .call = NULL, self@.ptr, value); self }`

This is implemented via `DotCallBuilder::null_call_attribution()` in `miniextendr-macros/src/r_wrapper_builder.rs`. The C wrapper still receives `__miniextendr_call: SEXP` (it always does) and gets `R_NilValue`; `make_rust_condition_value` stores it and the R-side `%||% sys.call()` recovers the user's frame.

## Reproducing the transcript

```bash
just configure
just rcmdinstall
Rscript -e '
library(miniextendr)
try(miniextendr:::call_attr_with(1L, 2L))
try(miniextendr:::unsafe_C_call_attr_without(1L, 2L))
'
```

The fixture lives at `rpkg/src/rust/call_attribution_demo.rs`.
