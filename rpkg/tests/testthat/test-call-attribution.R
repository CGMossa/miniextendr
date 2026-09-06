# Tests for call attribution behaviour documented in docs/CALL_ATTRIBUTION.md.
# Both fixtures are internal (:::); this file makes the transcript machine-verifiable.

# region: call_attr_with (wrapped path — with call attribution)

test_that("call_attr_with produces rust_error with call attribution", {
  e <- tryCatch(
    miniextendr:::call_attr_with(1L, 2L),
    error = function(e) e
  )
  expect_s3_class(e, "rust_error")
  call_str <- deparse(conditionCall(e))
  expect_match(call_str, "call_attr_with")
})

test_that("call_attr_with conditionCall includes formal parameter names", {
  # match.call() captures named formals: left = ..., right = ...
  e <- tryCatch(
    miniextendr:::call_attr_with(1L, 2L),
    error = function(e) e
  )
  call_str <- deparse(conditionCall(e))
  expect_match(call_str, "left")
  expect_match(call_str, "right")
})

# endregion

# region: unsafe_C_call_attr_without (unwrapped path — no call attribution)

test_that("unsafe_C_call_attr_without produces simpleError without rust_error class", {
  e <- tryCatch(
    miniextendr:::unsafe_C_call_attr_without(1L, 2L),
    error = function(e) e
  )
  expect_false(inherits(e, "rust_error"))
  expect_s3_class(e, "simpleError")
})

test_that("wrapped and unwrapped paths differ in call attribution", {
  # The wrapped path (call_attr_with) goes through error_in_r and match.call(),
  # so conditionCall includes formal names. The unwrapped path (Rf_error) does not.
  e_with <- tryCatch(
    miniextendr:::call_attr_with(1L, 2L),
    error = function(e) e
  )
  e_without <- tryCatch(
    miniextendr:::unsafe_C_call_attr_without(1L, 2L),
    error = function(e) e
  )
  # Only the wrapped path emits rust_error class
  expect_true(inherits(e_with, "rust_error"))
  expect_false(inherits(e_without, "rust_error"))
  # Wrapped path conditionCall includes formal parameter names
  call_with_str <- deparse(conditionCall(e_with))
  expect_match(call_with_str, "left")
  # Unwrapped path's conditionCall is non-NULL and names the function;
  # see docs/CALL_ATTRIBUTION.md:93-94. Rf_error captures the C-level
  # call expression, just without match.call()'s formal-name wiring.
  expect_false(is.null(conditionCall(e_without)))
  call_without_str <- deparse(conditionCall(e_without))
  expect_match(call_without_str, "unsafe_C_call_attr_without")
})

# endregion

# region: export check

test_that("call attribution fixtures are not exported", {
  exports <- getNamespaceExports("miniextendr")
  expect_false("call_attr_with" %in% exports)
  expect_false("unsafe_C_call_attr_without" %in% exports)
})

# endregion

# region: call = caller (internal entry point behind a hand-written function, #1450)

test_that("call = caller attributes the condition to the hand-written caller", {
  e <- tryCatch(miniextendr:::call_attr_caller(-1L), error = function(e) e)
  expect_s3_class(e, "rust_error")
  # `Result<_, String>` renders the Err through Debug (quoted) by default.
  expect_match(conditionMessage(e), "x must be positive, got -1", fixed = TRUE)
  # The caller's call, with the caller's formals matched.
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_caller(value = -1L)))
  expect_equal(miniextendr:::call_attr_caller(3L), 3L)
})

test_that("call = caller expands a literal `...` in the caller's call (#1462)", {
  # The caller's call is `miniextendr:::call_attr_caller(...)`; the dots are
  # bound in the helper's frame, one above the caller. The wrapper hands that
  # frame to `match.call(envir = )`, so the matched call carries the forwarded
  # arguments. Before the fix this failed on the success path too.
  via_dots <- function(...) miniextendr:::call_attr_caller(...)
  expect_equal(via_dots(3L), 3L)
  e <- tryCatch(via_dots(0L), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e), "x must be positive, got 0", fixed = TRUE)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_caller(value = 0L)))
  # A named argument travelling through `...` matches the caller's formal too.
  e <- tryCatch(via_dots(value = 0L), error = function(e) e)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_caller(value = 0L)))
  # R's `match.call()` inlines constants forwarded through `...` (above) but
  # renders symbols and calls as `..1`, `..2`: `-1L` is a call to unary minus.
  e <- tryCatch(via_dots(-1L), error = function(e) e)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_caller(value = ..1)))
})

test_that("call = caller expands lapply()'s `FUN(X[[i]], ...)` call (#1462)", {
  expect_equal(lapply(list(3L, 4L), miniextendr:::call_attr_caller), list(3L, 4L))
  e <- tryCatch(lapply(list(-1L), miniextendr:::call_attr_caller), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_equal(conditionCall(e), quote(FUN(value = X[[i]])))
})

test_that("default noexport attribution still names the wrapper", {
  e <- tryCatch(miniextendr:::call_attr_self(-1L), error = function(e) e)
  expect_equal(conditionCall(e), quote(call_attr_self_impl(x = value)))
})

test_that("call = caller falls back to the wrapper's own call when called directly", {
  e <- tryCatch(miniextendr:::call_attr_caller_impl(-1L), error = function(e) e)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_caller_impl(x = -1L)))
})

test_that("call = caller fixtures are not exported", {
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  expect_false(any(grepl("call_attr_caller", ns)))
  expect_false(any(grepl("call_attr_self", ns)))
})

# endregion
