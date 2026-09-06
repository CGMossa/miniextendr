# Receiver shapes accepted by class-system instance methods (#1469). The
# generated S3 constructor returns a classed external pointer, and the method
# wrappers pass the object itself to `.Call()`. The receiver prelude resolves
# any other shape through the same class-handle unwrap that `ExternalPtr<T>`
# arguments use, so a hand-built S3 object that keeps R-side state next to the
# handle in `.ptr` dispatches into the Rust methods unchanged. Fixture:
# `CounterTraitS3` in src/rust/class_system_matrix.rs (`get_value` is an
# exported generic; `custom_add.CounterTraitS3` is the exported `&mut self`
# method).

test_that("S3 methods accept a list wrapper carrying the handle in `.ptr`", {
  x <- structure(list(.ptr = new_countertraits3(5L), log = character()), class = "CounterTraitS3")
  expect_equal(get_value(x), 5L)
  # `&mut self` mutates the Rust value behind the shared pointer; the wrapper
  # returns the receiver invisibly, as for a bare handle.
  expect_invisible(custom_add.CounterTraitS3(x, 3L))
  expect_equal(get_value(x), 8L)
  # R-side state lives in the list and is untouched by the Rust side.
  x$log <- c(x$log, "added 3")
  expect_equal(x$log, "added 3")
  expect_equal(get_value(x), 8L)
  # `.ptr` is found by name, not position.
  y <- structure(list(log = "first", .ptr = new_countertraits3(6L)), class = "CounterTraitS3")
  expect_equal(get_value(y), 6L)
})

test_that("S3 methods accept a `.ptr` attribute and an environment binding", {
  x_attr <- structure(list(state = 1), class = "CounterTraitS3", .ptr = new_countertraits3(2L))
  expect_equal(get_value(x_attr), 2L)
  e <- new.env(parent = emptyenv())
  e$.ptr <- new_countertraits3(4L)
  class(e) <- "CounterTraitS3"
  expect_equal(get_value(e), 4L)
})

test_that("a receiver without a recoverable handle raises a rust_error naming the class", {
  bare <- structure(list(state = 1), class = "CounterTraitS3")
  e <- tryCatch(get_value(bare), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e), "expected a `CounterTraitS3` object", fixed = TRUE)
  expect_match(conditionMessage(e), "VECSXP", fixed = TRUE)
  # A `.ptr` element that is not a pointer counts as no handle.
  wrong <- structure(list(.ptr = 1L), class = "CounterTraitS3")
  e <- tryCatch(get_value(wrong), error = function(e) e)
  expect_match(conditionMessage(e), "expected a `CounterTraitS3` object", fixed = TRUE)
})

test_that("a handle of another Rust type inside `.ptr` still fails the type check", {
  other <- structure(list(.ptr = new_classedchecker(1)), class = "CounterTraitS3")
  e <- tryCatch(get_value(other), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e), "expected ExternalPtr<CounterTraitS3>", fixed = TRUE)
  expect_match(conditionMessage(e), "ClassedChecker", fixed = TRUE)
})
