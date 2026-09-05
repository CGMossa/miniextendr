# `#[miniextendr(serde_error)]`: classed Result errors derived from the error
# type's serde output (#1449). No RConditionError impl on the Rust side.

skip_if_not(exists("serde_err_missing"), "rpkg built without the `serde` feature")

test_that("internally tagged struct variant: member + family classes, fields as data", {
  e <- tryCatch(serde_err_missing("id"), error = function(e) e)
  expect_equal(class(e), c("miniextendr_error_missing_field", "miniextendr_error", "rust_error",
                           "simpleError", "error", "condition"))
  expect_equal(conditionMessage(e), "field `id` is missing")
  expect_equal(e$field, "id")
  # The serde tag is consumed as the variant; the framework's own `kind` stays.
  expect_equal(e$kind, "result_err")

  expect_equal(tryCatch(serde_err_missing("x"), miniextendr_error_missing_field = function(e) e$field), "x")
  expect_equal(tryCatch(serde_err_missing("y"), miniextendr_error = function(e) e$field), "y")
})

test_that("numeric fields and a nested unit enum land as scalars", {
  e <- tryCatch(serde_err_range(150), error = function(e) e)
  expect_s3_class(e, "miniextendr_error_out_of_range")
  expect_s3_class(e, "miniextendr_error")
  expect_equal(conditionMessage(e), "150 exceeds the maximum 100")
  expect_equal(e$value, 150)
  expect_equal(e$max, 100)
  expect_equal(e$route, "oral")
  expect_equal(serde_err_range(5), 5)
})

test_that("unit variant: classes only, no data", {
  e <- tryCatch(serde_err_unit_variant(), error = function(e) e)
  expect_equal(class(e), c("miniextendr_error_io", "miniextendr_error", "rust_error",
                           "simpleError", "error", "condition"))
  expect_equal(conditionMessage(e), "I/O failure")
  expect_false(any(c("field", "value", "max", "route") %in% names(e)))
  expect_invisible(tryCatch(serde_err_unit_variant(), error = function(e) invisible(NULL)))
})

test_that("serde_error(prefix = ...) replaces the crate-derived family class", {
  e <- tryCatch(serde_err_prefixed(150), error = function(e) e)
  expect_equal(class(e), c("engine_out_of_range", "engine", "rust_error",
                           "simpleError", "error", "condition"))
  expect_equal(e$max, 100)
  expect_equal(serde_err_prefixed(1), 1)
})

test_that("externally tagged variants: struct fields, newtype payload as `value`, unit", {
  e <- tryCatch(serde_err_external("bad"), error = function(e) e)
  expect_equal(class(e)[1:2], c("miniextendr_error_Bad", "miniextendr_error"))
  expect_equal(e$code, 7L)
  expect_equal(conditionMessage(e), "bad thing 7")

  e <- tryCatch(serde_err_external("plain"), error = function(e) e)
  expect_equal(class(e)[1:2], c("miniextendr_error_Plain", "miniextendr_error"))
  expect_equal(e$value, "boom")
  expect_equal(conditionMessage(e), "boom")

  e <- tryCatch(serde_err_external("unit"), error = function(e) e)
  expect_equal(class(e)[1:2], c("miniextendr_error_Unit", "miniextendr_error"))
  expect_false("value" %in% names(e))
})

test_that("the Ok arm is untouched", {
  expect_equal(serde_err_ok(2.5), 2.5)
})

test_that("serde_error on an S3 method goes through the same path", {
  chk <- new_serdechecker(10)
  expect_equal(check_value(chk, 3), 3)
  e <- tryCatch(check_value(chk, 30), error = function(e) e)
  expect_s3_class(e, "miniextendr_error_out_of_range")
  expect_s3_class(e, "miniextendr_error")
  expect_equal(e$max, 10)
  expect_equal(e$route, "iv")
  expect_equal(tryCatch(check_value(chk, 30), miniextendr_error = function(e) e$value), 30)
})
