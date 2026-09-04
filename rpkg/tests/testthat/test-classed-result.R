# Classed Result errors (RConditionError / RError), class vectors and the
# condition-data prefix (#1434, #1435, #1440).

test_that("Result<T, E: RConditionError> raises member + family classes with data", {
  e <- tryCatch(classed_result_missing(""), error = function(e) e)
  expect_equal(class(e), c("pkg_error_missing_field", "pkg_error", "rust_error",
                           "simpleError", "error", "condition"))
  expect_equal(conditionMessage(e), "field `id` is missing")
  expect_equal(e$field, "id")
  expect_equal(e$kind, "result_err")

  # Member handler and family handler both dispatch.
  expect_equal(tryCatch(classed_result_missing(""), pkg_error_missing_field = function(e) e$field), "id")
  expect_equal(tryCatch(classed_result_range(150), pkg_error = function(e) e$max), 100)
  expect_equal(tryCatch(classed_result_range(150), pkg_error_out_of_range = function(e) e$value), 150)

  # Ok path untouched.
  expect_equal(classed_result_missing("abc"), 3L)
  expect_equal(classed_result_range(5), 5)
})

test_that("classed Result errors work on the unit-return and impl-method arms", {
  e <- tryCatch(classed_result_unit(500), error = function(e) e)
  expect_s3_class(e, "pkg_error_out_of_range")
  expect_equal(e$value, 500)
  expect_invisible(classed_result_unit(1))

  chk <- new_classedchecker(10)
  expect_equal(check_bound(chk, 3), 3)
  e <- tryCatch(check_bound(chk, 30), error = function(e) e)
  expect_s3_class(e, "pkg_error_out_of_range")
  expect_s3_class(e, "pkg_error")
  expect_equal(e$max, 10)
})

test_that("RError: From<Error> keeps the message, builders add class and data", {
  expect_equal(rerror_parse("42"), 42L)
  e <- tryCatch(rerror_parse("x"), error = function(e) e)
  expect_equal(class(e)[1:3], c("pkg_bad_number", "pkg_error", "rust_error"))
  expect_match(conditionMessage(e), "invalid digit")
  expect_equal(e$input, "x")

  e <- tryCatch(rerror_plain(), error = function(e) e)
  expect_equal(class(e)[1], "rust_error")
  expect_equal(conditionMessage(e), "plain rerror")
  expect_equal(e$kind, "result_err")
})

test_that("RError data_prefix forwards reserved-looking names without clobbering the base slots", {
  e <- tryCatch(rerror_prefixed(), pkg_prefixed = function(e) e)
  expect_equal(e$p_kind, 7L)
  expect_equal(e$p_message, "inner")
  expect_equal(e$p_call, "wrapped::step")
  expect_equal(e$kind, "result_err")
  expect_equal(conditionMessage(e), "prefixed fields")
  expect_true(is.call(e$call) || is.null(e$call))
})

test_that("a computed reserved field name without a prefix is rejected, not spliced", {
  e <- tryCatch(rerror_reserved_runtime("kind"), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e), "reserved")
  expect_match(conditionMessage(e), "data_prefix")
  expect_equal(e$kind, "panic")

  e <- tryCatch(reserved_data_macro_runtime("message"), error = function(e) e)
  expect_match(conditionMessage(e), "reserved")
  expect_equal(e$kind, "panic")

  # Non-reserved computed names still work.
  e <- tryCatch(rerror_reserved_runtime("weight"), error = function(e) e)
  expect_equal(e$weight, 1L)
})

test_that("macro class vectors layer member before family", {
  e <- tryCatch(classed_error_vec("pkg_err_member", "pkg_err_family"), error = function(e) e)
  expect_equal(class(e)[1:3], c("pkg_err_member", "pkg_err_family", "rust_error"))
  expect_equal(tryCatch(classed_error_vec("a_member", "a_family"), a_family = function(e) "family"), "family")

  w <- tryCatch(classed_warning_vec(3L), warning = function(w) w)
  expect_equal(class(w)[1:3], c("pkg_warning_dropped", "pkg_warning", "rust_warning"))
  expect_equal(w$dropped, 3L)

  # `rust_condition!` unwinds the Rust call, so the wrapper returns NULL after
  # signalling; the class vector is what a handler sees.
  seen <- tryCatch(
    classed_condition_vec(c("pkg_cond_member", "pkg_cond_family")),
    pkg_cond_family = function(c) class(c)[1:3]
  )
  expect_equal(seen, c("pkg_cond_member", "pkg_cond_family", "rust_condition"))
})

test_that("data_prefix on the macros leaves message/kind intact", {
  e <- tryCatch(prefixed_data_macro(5L), pkg_prefixed_macro = function(e) e)
  expect_equal(e$p_kind, 5L)
  expect_equal(e$p_message, "inner")
  expect_equal(e$p_call, "wrapped::step")
  expect_equal(e$kind, "error")
  expect_equal(conditionMessage(e), "prefixed macro fields")
})
