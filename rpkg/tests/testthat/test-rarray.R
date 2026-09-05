test_that("RMatrix dims returns nrow and ncol", {
  m <- matrix(1:6, nrow = 2, ncol = 3)
  expect_equal(rarray_matrix_dims(m * 1.0), c(2L, 3L))
})

test_that("RMatrix len returns total elements", {
  m <- matrix(1:12, nrow = 3, ncol = 4)
  expect_equal(rarray_matrix_len(m * 1.0), 12L)
})

test_that("RVector sum returns correct total", {
  expect_equal(rarray_vector_sum(c(1.0, 2.0, 3.0, 4.0)), 10.0)
  expect_equal(rarray_vector_sum(numeric(0)), 0.0)
})

test_that("RMatrix column extraction works", {
  m <- matrix(c(1, 2, 3, 4, 5, 6), nrow = 2, ncol = 3)
  # 1-based access
  expect_equal(rarray_matrix_column(m, 1L), c(1, 2))
  expect_equal(rarray_matrix_column(m, 2L), c(3, 4))
  expect_equal(rarray_matrix_column(m, 3L), c(5, 6))

  # Out of bounds / non-positive index errors
  expect_error(rarray_matrix_column(m, 0L), "positive 1-based")
  expect_error(rarray_matrix_column(m, 4L), "out of bounds")
})

test_that("RMatrix attribute getters return both dimname axes", {
  m <- matrix(as.double(1:6), nrow = 2L, dimnames = list(
    c("row-a", "row-b"),
    c("col-a", "col-b", "col-c")
  ))

  expect_identical(miniextendr:::rarray_matrix_rownames(m), rownames(m))
  expect_identical(miniextendr:::rarray_matrix_colnames(m), colnames(m))
})

test_that("List attribute getters return both dimname axes", {
  m <- matrix(as.list(1:6), nrow = 2L, dimnames = list(
    c("row-a", "row-b"),
    c("col-a", "col-b", "col-c")
  ))

  expect_identical(miniextendr:::list_matrix_rownames(m), rownames(m))
  expect_identical(miniextendr:::list_matrix_colnames(m), colnames(m))
})

test_that("row-name getters report absent row names", {
  numeric_matrix <- matrix(
    as.double(1:6),
    nrow = 2L,
    dimnames = list(NULL, c("col-a", "col-b", "col-c"))
  )
  list_matrix <- matrix(
    as.list(1:6),
    nrow = 2L,
    dimnames = list(NULL, c("col-a", "col-b", "col-c"))
  )

  expect_error(miniextendr:::rarray_matrix_rownames(numeric_matrix), "expected row names")
  expect_error(miniextendr:::list_matrix_rownames(list_matrix), "expected row names")
})
