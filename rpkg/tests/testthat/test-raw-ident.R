# Raw identifiers (`r#keyword`) on the Rust side must surface as plain R names.
# Fixture: `rpkg/src/rust/raw_ident_tests.rs`.

test_that("keyword function and parameter names lose the r# prefix", {
  expect_equal(where(21L), 42L)
  expect_equal(names(formals(raw_ident_args)), c("where", "type", "ref"))
  expect_equal(raw_ident_args(where = "w", type = 2L), "w-2-1")
  expect_equal(raw_ident_args("w", 2L, ref = 5L), "w-2-5")
})

test_that("match.arg and named dots work on keyword parameters", {
  expect_equal(names(formals(raw_ident_choice)), "type")
  expect_equal(raw_ident_choice(), "fast")
  expect_equal(raw_ident_choice(type = "slow"), "slow")
  expect_error(raw_ident_choice("nope"), "should be one of")

  expect_equal(names(formals(raw_ident_match_arg)), "type")
  expect_equal(raw_ident_match_arg(), "fast")
  expect_equal(raw_ident_match_arg(type = "Slow"), "slow")
  expect_error(raw_ident_match_arg("nope"), "should be one of")
  expect_equal(names(formals(raw_ident_coerce)), "type")
  expect_equal(raw_ident_coerce(100L), 100L)
  expect_error(raw_ident_coerce(-1L))

  expect_equal(names(formals(raw_ident_dots)), c("type", "..."))
  expect_equal(raw_ident_dots(type = 1L, "a", "b"), 3L)
})

test_that("R6 keyword methods, active binding, and parameters", {
  obj <- RawIdentR6$new(type = 5L)
  expect_equal(obj$type, 5L)
  expect_equal(names(formals(obj$move)), "where")
  expect_equal(obj$move(where = 2L), 7L)
  expect_equal(obj$type, 7L)
  expect_equal(obj$use(mod = 4L), 3L)
})

test_that("env class keyword methods and keyword sidecar accessors", {
  obj <- RawIdentEnv$new(type = 5L, base = 10L)
  expect_equal(obj$loop(where = 1L), 11L)
  expect_equal(RawIdentEnv_get_type(obj), 5L)
  RawIdentEnv_set_type(obj, 9L)
  expect_equal(RawIdentEnv_get_type(obj), 9L)
})

test_that("keyword struct fields become list names and data-frame columns", {
  df <- raw_ident_df()
  expect_s3_class(df, "data.frame")
  expect_equal(names(df), c("type", "where"))
  expect_equal(df$type, c("a", "b"))
  expect_equal(df$where, c(1L, 2L))

  l <- raw_ident_list()
  expect_equal(names(l), c("type", "where"))
  expect_equal(l$type, "a")
  expect_equal(l$where, 1L)
})
