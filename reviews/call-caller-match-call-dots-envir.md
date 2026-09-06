# `call = caller`: `match.call()` failed when the caller was invoked with `...` (#1462)

## What was attempted

A downstream test helper forwarded its dots into a hand-written R function that
delegates to a `#[miniextendr(noexport, call = caller)]` entry point:

```r
via <- function(...) call_attr_caller(...)
via(3L)
```

## What went wrong

Every call through the helper failed, on the success path too:

```
Error in match.call(.mx_def, .mx_pc) : ... used in a situation where it does not exist
```

The same failure hit `lapply(xs, call_attr_caller)`, whose call is
`FUN(X[[i]], ...)`, even though `docs/CALL_ATTRIBUTION.md` said the `lapply()`
path reported that frame's call. Nothing in the suite exercised either shape:
the `call = caller` tests called the hand-written function directly.

## Root cause

The generated prelude matched the caller's call as
`match.call(.mx_def, .mx_pc)`. When that call contains a literal `...`,
`match.call()` expands it by looking `...` up in `envir`, whose default
`parent.frame(2L)` is a promise evaluated inside `match.call()`'s own frame.
From there, two frames up is the caller (the hand-written function), which has
no `...`. The dots are bound one frame further up, in the helper (or in
`lapply()`'s frame). The matched call is built before `.Call()`, so the error
was not limited to the error-attribution path.

## Fix

Pass the frame the caller's call was evaluated in, spelled from the wrapper:
`match.call(.mx_def, .mx_pc, envir = parent.frame(2L))`. `parent.frame()` anchors
on the env it is evaluated from (the promise env is the wrapper's frame), so the
result is the caller's caller whether the expression is a plain statement or an
argument promise. `envir` is only consulted for `...` expansion, so every call
without dots is unchanged; the fallback branch (`sys.parent() == 0`, non-closure
parent) is untouched. One R convention to keep in mind when writing expectations:
`match.call()` inlines constants forwarded through `...` but renders symbols and
calls as `..1`, `..2`, and `-1L` is a call (unary minus), so the first draft of the
test expected `value = -1L` and got `value = ..1`. Tests in `rpkg/tests/testthat/test-call-attribution.R` now
pin a `function(...)` helper and an `lapply()` traversal; the plain-R comparison
matrix that confirmed the fix is the shape to rerun when touching the prelude
(`function(...)` forwarding, named args through dots, nested helpers, `lapply`,
`sapply`, `Map`, `do.call` by object and by name, `eval`, `local`, `tryCatch`,
top level).
