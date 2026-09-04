# `Option<scalar>` fields in `#[derive(DataFrameRow)]` structs (#1437):
# `None` <-> typed `NA`, in both directions, without changing the column type.

test_that("Option<scalar> fields write as typed NA columns", {
  df <- df_option_scalar_rows()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3L)
  expect_equal(names(df), c("id", "weight", "label", "flag", "count"))

  expect_type(df$weight, "double")
  expect_type(df$label, "character")
  expect_type(df$flag, "logical")
  expect_type(df$count, "integer")

  expect_equal(df$weight, c(1.5, NA, -2.25))
  expect_equal(df$label, c("a", NA, ""))
  expect_equal(df$flag, c(TRUE, NA, FALSE))
  expect_equal(df$count, c(10L, NA, -7L))
})

test_that("Option<scalar> fields read NA back as None and round-trip", {
  df <- df_option_scalar_rows()
  expect_equal(df_option_scalar_roundtrip(df), df)
  expect_equal(df_option_scalar_none_count(df), 4L)

  # Hand-built input with NAs scattered across rows.
  input <- data.frame(
    id = 1:4,
    weight = c(NA, 2, NA, 4),
    label = c("x", NA, NA, "w"),
    flag = c(NA, TRUE, FALSE, NA),
    count = c(1L, NA, 3L, NA),
    stringsAsFactors = FALSE
  )
  expect_equal(df_option_scalar_roundtrip(input), input)
  # weight: rows 1,3; label: rows 2,3; flag: rows 1,4; count: rows 2,4
  expect_equal(df_option_scalar_none_count(input), 8L)
})

test_that("an all-NA Option<scalar> column keeps its type through the round-trip", {
  input <- data.frame(
    id = 1:2,
    weight = c(NA_real_, NA_real_),
    label = c(NA_character_, NA_character_),
    flag = c(NA, NA),
    count = c(NA_integer_, NA_integer_),
    stringsAsFactors = FALSE
  )
  out <- df_option_scalar_roundtrip(input)
  expect_equal(out, input)
  expect_type(out$weight, "double")
  expect_type(out$label, "character")
  expect_type(out$count, "integer")
})
