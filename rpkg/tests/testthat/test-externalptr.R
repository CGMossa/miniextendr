test_that("Counter ExternalPtr works correctly", {
  # Create a counter with initial value 0
  counter <- extptr_counter_new(0L)
  expect_true(inherits(counter, "externalptr"))

  # Get the initial value
  expect_equal(miniextendr:::unsafe_C_extptr_counter_get(counter), 0L)

  # Increment and check
  miniextendr:::unsafe_C_extptr_counter_increment(counter)
  expect_equal(miniextendr:::unsafe_C_extptr_counter_get(counter), 1L)

  # Increment again
  miniextendr:::unsafe_C_extptr_counter_increment(counter)
  expect_equal(miniextendr:::unsafe_C_extptr_counter_get(counter), 2L)
})

test_that("Counter ExternalPtr works with non-zero initial value", {
  counter <- extptr_counter_new(42L)
  expect_equal(miniextendr:::unsafe_C_extptr_counter_get(counter), 42L)

  miniextendr:::unsafe_C_extptr_counter_increment(counter)
  expect_equal(miniextendr:::unsafe_C_extptr_counter_get(counter), 43L)
})

test_that("Point ExternalPtr works correctly", {
  # Create a point at (3, 4)
  point <- extptr_point_new(3.0, 4.0)
  expect_true(inherits(point, "externalptr"))

  # Get coordinates
  expect_equal(miniextendr:::unsafe_C_extptr_point_get_x(point), 3.0)
  expect_equal(miniextendr:::unsafe_C_extptr_point_get_y(point), 4.0)
})

test_that("Point ExternalPtr works with various coordinates", {
  # Negative coordinates
  point1 <- extptr_point_new(-1.5, -2.5)
  expect_equal(miniextendr:::unsafe_C_extptr_point_get_x(point1), -1.5)
  expect_equal(miniextendr:::unsafe_C_extptr_point_get_y(point1), -2.5)

  # Origin
  point2 <- extptr_point_new(0.0, 0.0)
  expect_equal(miniextendr:::unsafe_C_extptr_point_get_x(point2), 0.0)
  expect_equal(miniextendr:::unsafe_C_extptr_point_get_y(point2), 0.0)
})

test_that("ErasedExternalPtr type checking works", {
  counter <- extptr_counter_new(10L)
  point <- extptr_point_new(1.0, 2.0)

  # Counter should be identified as Counter, not Point
  expect_equal(miniextendr:::unsafe_C_extptr_is_counter(counter), 1L)
  expect_equal(miniextendr:::unsafe_C_extptr_is_point(counter), 0L)

  # Point should be identified as Point, not Counter
  expect_equal(miniextendr:::unsafe_C_extptr_is_counter(point), 0L)
  expect_equal(miniextendr:::unsafe_C_extptr_is_point(point), 1L)
})

test_that("new('externalptr') null pointer is handled correctly", {
  # R's new("externalptr") creates a null external pointer
  null_ptr <- new("externalptr")
  expect_true(inherits(null_ptr, "externalptr"))

  # null_test should return 0 (indicating null was detected)
  expect_equal(miniextendr:::unsafe_C_extptr_null_test(null_ptr), 0L)

  # Type checks should return 0 for null pointers
  expect_equal(miniextendr:::unsafe_C_extptr_is_counter(null_ptr), 0L)
  expect_equal(miniextendr:::unsafe_C_extptr_is_point(null_ptr), 0L)
})

test_that("type mismatch returns error/0", {
  # Create a Point and try to use it as a Counter
  point <- extptr_point_new(1.0, 2.0)

  # type_mismatch_test passes a Point to a Counter-expecting function
  # It should return 0 (type mismatch detected, not an error)
  expect_equal(miniextendr:::unsafe_C_extptr_type_mismatch_test(point), 0L)
})

test_that("multiple Counter instances are independent", {
  counter1 <- extptr_counter_new(0L)
  counter2 <- extptr_counter_new(100L)

  # Increment counter1
  miniextendr:::unsafe_C_extptr_counter_increment(counter1)
  miniextendr:::unsafe_C_extptr_counter_increment(counter1)

  # counter1 should be 2, counter2 should still be 100
  expect_equal(miniextendr:::unsafe_C_extptr_counter_get(counter1), 2L)
  expect_equal(miniextendr:::unsafe_C_extptr_counter_get(counter2), 100L)
})

test_that("ExternalPtr pointee is readable after transfer to the worker", {
  counter <- extptr_counter_new(37L)

  expect_equal(miniextendr:::extptr_worker_value(counter), 37L)
  expect_equal(miniextendr:::unsafe_C_extptr_counter_get(counter), 37L)
})

test_that("ExternalPtr worker round-trip preserves R identity", {
  counter <- extptr_counter_new(41L)

  returned <- miniextendr:::extptr_worker_identity(counter)

  expect_identical(returned, counter)
  expect_equal(miniextendr:::unsafe_C_extptr_counter_get(returned), 41L)
})

test_that("ExternalPtr into_inner on worker clears the R handle", {
  counter <- extptr_counter_new(73L)

  expect_equal(miniextendr:::extptr_worker_into_inner(counter), 73L)
  expect_error(
    miniextendr:::extptr_worker_value(counter),
    "external pointer is null"
  )
})
