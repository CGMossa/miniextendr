# `match.arg` choices from an enum the package does not own (#1436): a newtype
# with a hand-written `MatchArg` impl behaves exactly like a derived enum.

test_that("hand-written MatchArg newtype: default, exact and partial matching", {
  expect_equal(foreign_enum_interp(), "Linear")
  expect_equal(foreign_enum_interp("cubic"), "Cubic")
  expect_equal(foreign_enum_interp("n"), "Nearest")
  expect_equal(foreign_enum_interp(factor("cubic")), "Cubic")
})

test_that("hand-written MatchArg newtype: invalid choice errors like match.arg", {
  expect_error(foreign_enum_interp("spline"), "should be one of")
  expect_error(foreign_enum_interp(1), "must be NULL or a character vector")
})

test_that("hand-written MatchArg newtype: choices are spliced into the formal default", {
  expect_equal(formals(foreign_enum_interp)$method, quote(c("linear", "cubic", "nearest")))
  expect_equal(formals(foreign_enum_interps)$methods, quote(c("linear", "cubic", "nearest")))
})

test_that("hand-written MatchArg newtype: several_ok and Vec<T> return", {
  expect_equal(foreign_enum_interps(c("cubic", "linear")), c("cubic", "linear"))
  expect_equal(foreign_enum_interps("near"), "nearest")
  # Omitted argument -> every choice, as match.arg(several.ok = TRUE) does.
  expect_equal(foreign_enum_interps(), c("linear", "cubic", "nearest"))
  # match.arg(several.ok = TRUE) drops unmatched entries when at least one
  # matches, and errors only when none do (base R semantics, not ours).
  expect_equal(foreign_enum_interps(c("cubic", "spline")), "cubic")
  expect_error(foreign_enum_interps(c("spline", "bogus")), "should be one of")
})

test_that("hand-written MatchArg newtype: scalar return renders the choice string", {
  expect_identical(foreign_enum_default(), "cubic")
})
