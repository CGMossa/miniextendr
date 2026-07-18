# Tests for ColumnarDataFrame all-None column downgrade (#307), schema-upgrade
# ordering, and recursive Compound union (#1370).
#
# When every row has None for an Option<T> field the column lands as a logical
# NA vector (is.logical() && all(is.na())), not list(NULL, NULL, ...).

# region: All-None → logical NA column

test_that("Option<u64> all-None single row lands as logical NA", {
  df <- miniextendr:::test_columnar_opt_u64_all_none_single()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 1L)
  expect_true(is.logical(df$stored))
  expect_true(all(is.na(df$stored)))
  # name column is unaffected
  expect_equal(df$name, "a")
})

test_that("Option<u64> all-None multi-row lands as logical NA", {
  df <- miniextendr:::test_columnar_opt_u64_all_none_multi()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3L)
  expect_true(is.logical(df$stored))
  expect_true(all(is.na(df$stored)))
})

test_that("Option<String> all-None lands as logical NA", {
  df <- miniextendr:::test_columnar_opt_string_all_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.logical(df$label))
  expect_true(all(is.na(df$label)))
})

test_that("Option<bool> all-None lands as logical NA", {
  df <- miniextendr:::test_columnar_opt_bool_all_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.logical(df$flag))
  expect_true(all(is.na(df$flag)))
})

test_that("Option<UserStruct> all-None: column lands as single logical NA column", {
  df <- miniextendr:::test_columnar_opt_user_struct_all_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  # When every row is None for an Option<Struct>, the probe never sees any
  # fields, so struct expansion never fires.  The whole field becomes a single
  # logical NA column under the field name ("point"), not per-subfield columns.
  expect_true("point" %in% names(df))
  expect_true(is.logical(df$point))
  expect_true(all(is.na(df$point)))
})

test_that("Option<HashMap> all-None lands as logical NA", {
  df <- miniextendr:::test_columnar_opt_hashmap_all_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.logical(df$attrs))
  expect_true(all(is.na(df$attrs)))
})

test_that("Option<Vec<u8>> all-None lands as logical NA", {
  df <- miniextendr:::test_columnar_opt_bytes_all_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.logical(df$data))
  expect_true(all(is.na(df$data)))
})

# endregion

# region: Mixed Some/None — no downgrade

test_that("Option<u64> mixed Some/None: numeric column with NA", {
  df <- miniextendr:::test_columnar_opt_u64_mixed()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3L)
  expect_true(is.numeric(df$stored))
  expect_equal(df$stored, c(42, NA, 99))
})

test_that("Option<String> mixed Some/None: character column with NA", {
  df <- miniextendr:::test_columnar_opt_string_mixed()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.character(df$label))
  expect_equal(df$label, c("hello", NA_character_))
})

# endregion

# region: Vec<u8> with values stays a list column

test_that("Vec<u8> with values is still a list column", {
  df <- miniextendr:::test_columnar_bytes_with_values()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.list(df$data))
  # Each element is a raw or integer vector of the right length
  expect_equal(length(df$data[[1]]), 3L)
  expect_equal(length(df$data[[2]]), 2L)
})

# endregion

# region: Mixed columns — bytes stays list, opt-none downgrades

test_that("Vec<u8> column stays list; adjacent all-None Option<u64> downgrades", {
  df <- miniextendr:::test_columnar_bytes_and_opt_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  # raw bytes column: still a list
  expect_true(is.list(df$raw))
  # optional stored column: downgraded to logical NA
  expect_true(is.logical(df$stored))
  expect_true(all(is.na(df$stored)))
})

# endregion

# region: Enum union

test_that("enum all-variant-A with x=None: x column is logical NA", {
  df <- miniextendr:::test_columnar_enum_all_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.logical(df$x))
  expect_true(all(is.na(df$x)))
})

test_that("enum with one variant-B x=Some(42): x column is numeric with NA in row 1", {
  # Two-phase discovery: the probe scans ALL rows before resolving the schema.
  # Variant-B's x=Some(42u64) contributes Scalar(Real), which beats Scalar(Generic)
  # from variant-A's x=None.  The column is numeric, not a list.
  df <- miniextendr:::test_columnar_enum_some_flips_type()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.numeric(df$x))
  expect_true(is.na(df$x[[1]]))
  expect_equal(df$x[[2]], 42)
})

# endregion

# region: Flatten with all-None inner field

test_that("serde(flatten) with all-None inner Option field: column is logical NA", {
  df <- miniextendr:::test_columnar_flatten_all_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_equal(df$id, c(1L, 2L))
  # The flattened 'size' field is always None and is NOT skip_serializing_if,
  # so it appears in the schema and lands as logical NA.
  expect_true("size" %in% names(df))
  expect_true(is.logical(df$size))
  expect_true(all(is.na(df$size)))
  # The 'name' field should be character
  expect_equal(df$name, c("a", "b"))
})

# endregion

# region: R coercion smoke tests

test_that("logical NA column coerces to numeric when combined with numeric values", {
  df_none <- miniextendr:::test_columnar_opt_u64_all_none_multi()
  df_some <- miniextendr:::test_columnar_opt_u64_mixed()

  # bind_rows coerces the logical NA column to the numeric column type
  if (requireNamespace("dplyr", quietly = TRUE)) {
    combined <- dplyr::bind_rows(df_none, df_some)
    expect_true(is.numeric(combined$stored))
    # The three all-NA rows and one more NA from the mixed set
    expect_equal(sum(is.na(combined$stored)), 4L)
  } else {
    # base R fallback: c() coerces logical to numeric
    coerced <- c(df_none$stored, df_some$stored)
    expect_true(is.numeric(coerced))
    expect_equal(sum(is.na(coerced)), 4L)
  }
})

test_that("logical NA column coerces to character when combined with character values", {
  df_none <- miniextendr:::test_columnar_opt_string_all_none()
  df_some <- miniextendr:::test_columnar_opt_string_mixed()

  if (requireNamespace("dplyr", quietly = TRUE)) {
    combined <- dplyr::bind_rows(df_none, df_some)
    expect_true(is.character(combined$label))
    expect_equal(sum(is.na(combined$label)), 3L) # 2 all-None + 1 mixed-None
  } else {
    coerced <- c(df_none$label, df_some$label)
    expect_true(is.character(coerced))
    expect_equal(sum(is.na(coerced)), 3L)
  }
})

# endregion

# region: Schema upgrade — first-row-None then Some

test_that("first-row-None scalar upgrades to numeric when later row has Some(42u64)", {
  df <- miniextendr:::test_columnar_schema_upgrade_scalar()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.numeric(df$x))
  expect_true(is.na(df$x[[1]]))
  expect_equal(df$x[[2]], 42)
})

test_that("first-row-None nested struct: columns point_x and point_y appear, row 1 has NA", {
  df <- miniextendr:::test_columnar_schema_upgrade_nested()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  # Must have per-subfield columns, NOT a single "point" column
  expect_true("point_x" %in% names(df))
  expect_true("point_y" %in% names(df))
  expect_false("point" %in% names(df))
  # Row 1 had point=None, so both subfields are NA
  expect_true(is.na(df$point_x[[1]]))
  expect_true(is.na(df$point_y[[1]]))
  # Row 2 has the actual values
  expect_equal(df$point_x[[2]], 1.0)
  expect_equal(df$point_y[[2]], 2.0)
})

test_that("multiple leading None rows then Some(42): numeric column with NAs at 1,2,4", {
  df <- miniextendr:::test_columnar_schema_upgrade_multi_none_first()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 4L)
  expect_true(is.numeric(df$x))
  expect_true(is.na(df$x[[1]]))
  expect_true(is.na(df$x[[2]]))
  expect_equal(df$x[[3]], 42)
  expect_true(is.na(df$x[[4]]))
})

test_that("compound-vs-compound different shapes: both field sets discovered", {
  # Variant A has only 'value'; variant B has 'value' and 'extra'.
  # Both are distinct top-level keys in the struct (not nested under a single key),
  # so both 'value' and 'extra' are found regardless of order. (The single-key
  # Compound-vs-Compound case — recursive union — is covered in the "Recursive
  # Compound union (#1370)" region below.)
  df <- miniextendr:::test_columnar_compound_different_shapes()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true("value" %in% names(df))
  expect_true("extra" %in% names(df))
  # variant A (row 1) has no 'extra', so it should be NA
  expect_true(is.na(df$extra[[1]]))
  # variant B (row 2) has extra=3.0
  expect_equal(df$extra[[2]], 3.0)
})

# endregion

# region: Recursive Compound union (#1370)

test_that("nested sub-field None-first types as character, not a list column", {
  df <- miniextendr:::test_columnar_nested_subfield_none_first()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true("meta_variant" %in% names(df))
  expect_true("meta_size" %in% names(df))
  expect_true(is.character(df$meta_variant))
  expect_true(is.na(df$meta_variant[[1]]))
  expect_equal(df$meta_variant[[2]], "x")
  expect_equal(df$meta_size, c(4.0, 5.0))
})

test_that("nested sub-field schema is row-order independent", {
  none_first <- miniextendr:::test_columnar_nested_subfield_none_first()
  some_first <- miniextendr:::test_columnar_nested_subfield_some_first()
  expect_equal(names(none_first), names(some_first))
  expect_equal(class(none_first$meta_variant), class(some_first$meta_variant))
  expect_true(is.character(some_first$meta_variant))
})

test_that("Compound union of keys: a key present only in one row's shape is not dropped", {
  df <- miniextendr:::test_columnar_nested_compound_union_of_keys()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  # Row 1's probe of `meta` omitted `extra` (skip_serializing_if); row 2's
  # included it. Both `meta_extra` and `meta_size` must be discovered.
  expect_true("meta_extra" %in% names(df))
  expect_true("meta_size" %in% names(df))
  expect_true(is.na(df$meta_extra[[1]]))
  expect_equal(df$meta_extra[[2]], 3.0)
  expect_equal(df$meta_size, c(1.0, 2.0))
})

# endregion
