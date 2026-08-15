# Runtime boundary of the serde -> R lowering, pinned empirically
# (rpkg/src/rust/serde_hostile_probe_tests.rs).
#
# The compile-time boundary cannot be pinned here: a struct field holding an
# R handle (SEXP, ExternalPtr<T>) fails #[derive(Serialize)] with E0277 —
# neither type implements Serialize/Deserialize, so there is nothing to lower.

test_that("u128 fields cannot be serialized to R data", {
  expect_match(probe_serde_u128(), "u128 is not supported")
})

test_that("non-string map keys cannot be serialized to R data", {
  expect_match(probe_serde_int_keys(), "map keys must be strings")
})

test_that("u64 above 2^53 lowers and re-reads silently corrupted", {
  d <- probe_serde_big_u64_to_r()
  expect_identical(typeof(d$id), "double")
  # The Rust value was 2^53 + 1; the nearest double is 2^53. No error is
  # raised at serialize or deserialize time — the value just changes.
  expect_identical(probe_serde_big_u64_read(d), "9007199254740992")
})

test_that("u64 corruption persists across sessions without error", {
  skip_if_not_installed("callr")
  skip_on_os("windows")  # same callr rationale as the other cross-session suites
  d <- probe_serde_big_u64_to_r()
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))
  saveRDS(d, tmp)
  res <- callr::r(function(p) {
    library(miniextendr)
    probe_serde_big_u64_read(readRDS(p))
  }, args = list(p = tmp))
  expect_identical(res, "9007199254740992")
})

test_that("serde None output is always NULL, never typed NA", {
  d <- probe_serde_none_to_r()
  expect_null(d$note)
  expect_null(d$n)
  expect_identical(names(d), c("note", "n"))
})

test_that("serde deserialization accepts NULL and typed NA as None (#1166)", {
  expect_identical(
    probe_serde_none_read(list(note = NULL, n = NULL)),
    "note=None n=None"
  )
  expect_identical(
    probe_serde_none_read(list(note = NA_character_, n = NA_integer_)),
    "note=None n=None"
  )
})

test_that("RECTIFIED: ExternalPtr<T: Serialize> fields lower by value", {
  d <- probe_serde_handle_to_r()
  expect_identical(d$label, "holder")
  # The handle entry is the pointee's named list — plain R data, no pointer.
  expect_identical(d$handle$name, "inner")
  expect_identical(d$handle$score, 2.5)
  # Re-reading rebuilds a fresh live handle around the reconstructed pointee.
  expect_identical(probe_serde_handle_read(d), "label=holder name=inner score=2.5")
})

test_that("RECTIFIED: handle-holding struct survives a new session via serde data", {
  skip_if_not_installed("callr")
  skip_on_os("windows")
  d <- probe_serde_handle_to_r()
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))
  saveRDS(d, tmp)
  res <- callr::r(function(p) {
    library(miniextendr)
    probe_serde_handle_read(readRDS(p))
  }, args = list(p = tmp))
  expect_identical(res, "label=holder name=inner score=2.5")
})

test_that("i32::MIN silently aliases NA_integer_ on every output path", {
  d <- probe_serde_i32_min_to_r()
  # Serialization reports success, but the value is already NA in R.
  expect_identical(typeof(d$x), "integer")
  expect_true(is.na(d$x))
  # Required-field re-read: the value is unrecoverable (loud, at least).
  expect_match(probe_serde_i32_min_read(d), "unexpected NA")
  # Option re-read: Some(i32::MIN) has silently become None.
  expect_identical(probe_serde_i32_min_read_opt(d), "Ok: x=None")
  # The macro output path aliases identically — IntoR-level, not serde-specific.
  expect_true(is.na(probe_macro_i32_min()))
})

test_that("f64 with NA_real_'s exact payload aliases NA; plain NaN survives", {
  expect_identical(
    probe_serde_na_real_payload(),
    "na_payload=Ok(None) plain_nan=Ok(Some(NaN))"
  )
})

test_that("JSON path preserves u64 beyond 2^53 exactly (native path corrupts)", {
  skip_if_not(
    "probe_json_big_u64_to_r" %in% getNamespaceExports("miniextendr"),
    "serde_json feature not compiled in"
  )
  j <- probe_json_big_u64_to_r()
  expect_true(is.character(j))
  expect_match(j, "9007199254740993", fixed = TRUE)
  expect_identical(probe_json_big_u64_read(j), "9007199254740993")
})
