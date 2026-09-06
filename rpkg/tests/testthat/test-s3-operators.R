# Regression for #1475: operator methods must parse and register under their
# exact names, through both free-function wrapper paths and inherent impls.

test_that("free-function S3 operators dispatch on classed vectors", {
  x <- structure(c(10L, 20L, 30L), class = "mx_s3_vector")
  expect_identical(x[c(3L, 1L)], c(30L, 10L))
  expect_identical(x[[2L]], 20L)
  expect_identical(x[integer()], integer())
  expect_identical(names(formals(getS3method("[", "mx_s3_vector"))), c("x", "i", "..."))
  expect_identical(names(formals(getS3method("[[", "mx_s3_vector"))), c("x", "i", "..."))
})

test_that("inherent S3 operator methods dispatch on external pointers", {
  counter <- new_s3counter(42L)
  expect_identical(counter$value, 42L)
  expect_identical(getS3method("$", "S3Counter")(counter, "value"), 42L)
})
