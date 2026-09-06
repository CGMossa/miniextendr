# Standalone S3 methods on operator generics (#1475) and @describeIn from Rust
# doc comments (#1476). Fixtures: src/rust/s3_nonsyntactic_tests.rs.

test_that("s3(generic = \"[\") registers a working method (#1475)", {
  bag <- structure(c(1, 2, 3), class = "mx_bag")
  expect_true(is.function(getS3method("[", "mx_bag")))
  expect_equal(bag[2:3], c(2, 3))
  expect_equal(bag[c(1L, 3L)], c(1, 3))
  expect_error(bag[5L], "out of range")
})

test_that("s3(generic = \"$\") registers a working method (#1475)", {
  bag <- structure(c(1, 2, 3), class = "mx_bag")
  expect_true(is.function(getS3method("$", "mx_bag")))
  expect_equal(bag$n, 3)
  expect_equal(bag$sum, 6)
  expect_error(bag$nope, "unknown field")
})

test_that("impl-block generic override to `[[` produces a working method (#1475)", {
  h <- new_mxbaghandle(c(10, 20))
  expect_true(is.function(getS3method("[[", "MxBagHandle")))
  expect_equal(h[[2L]], 20)
  expect_equal(size(h), 2L)
  expect_error(h[[5L]], "out of range")
})

test_that("@describeIn keeps its continuation lines and lands on the destination page (#1476)", {
  rd_db <- tryCatch(tools::Rd_db("miniextendr"), error = function(e) NULL)
  skip_if(is.null(rd_db), "tools::Rd_db('miniextendr') unavailable — package not installed")
  rd <- rd_db[["mx_bag_sum.Rd"]]
  skip_if(is.null(rd), "mx_bag_sum.Rd not found — package not documented")
  rd_text <- paste(capture.output(print(rd)), collapse = "\n")

  # The @describeIn block was merged onto mx_bag_sum's page, so no separate
  # page exists for mx_bag_len and its usage/alias live here, under the
  # destination's page (titled by its wrapper name, #1054).
  expect_null(rd_db[["mx_bag_len.Rd"]])
  expect_match(rd_text, "title\\{mx_bag_sum\\}")
  expect_match(rd_text, "Sum of a bag")
  expect_match(rd_text, "mx_bag_sum\\(x\\)")
  expect_match(rd_text, "alias\\{mx_bag_len\\}", fixed = FALSE)
  expect_match(rd_text, "mx_bag_len\\(x\\)")
  # The wrapped continuation of the @describeIn description survived.
  expect_match(rd_text, "as an integer scalar")
  expect_match(rd_text, "keeps its continuation lines")
  # No file-stem @rdname was injected next to @describeIn (roxygen2 rejects it).
  expect_null(rd_db[["s3_nonsyntactic_tests.Rd"]][["mx_bag_len"]])
})
