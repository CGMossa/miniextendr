# Tests for zero-copy conversions (Cow, Arrow pointer recovery, ProtectedStrVec)

# region: Cow<[T]> round-trip (always copies — #880)

# The Cow<[T]> IntoR path no longer attempts speculative SEXP pointer recovery
# (#880): a borrowed sub-slice carries no provenance to prove it points at an
# R vector start, so the probe could read off into unrelated memory. The
# round-trip now always copies — we verify the values survive, not identity.

test_that("Cow<[f64]> round-trip preserves values", {
  x <- c(1.0, 2.0, 3.0)
  expect_equal(zero_copy_cow_f64_roundtrip(x), x)
})

test_that("Cow<[i32]> round-trip preserves values", {
  x <- c(1L, 2L, 3L, 4L, 5L)
  expect_equal(zero_copy_cow_i32_roundtrip(x), x)
})

test_that("Cow<[f64]> round-trip with NAs preserves values", {
  x <- c(1.0, NA, 3.0)
  expect_equal(zero_copy_cow_f64_roundtrip(x), x)
})

test_that("Cow<[i32]> round-trip with NAs preserves values", {
  x <- c(1L, NA, 3L)
  expect_equal(zero_copy_cow_i32_roundtrip(x), x)
})

# endregion

# region: RCow<T> round-trip (safe zero-copy — #880)

# RCow is the safe zero-copy replacement for the removed Cow<[T]> recovery: its
# borrowed arm carries the source SEXP, so the round-trip returns the *same*
# R object with no speculative pointer probe.

test_that("RCow<f64> round-trip returns same R object (zero-copy)", {
  x <- c(1.0, 2.0, 3.0)
  expect_true(zero_copy_rcow_f64_identity(x))
})

test_that("RCow<i32> round-trip returns same R object (zero-copy)", {
  x <- c(1L, 2L, 3L, 4L, 5L)
  expect_true(zero_copy_rcow_i32_identity(x))
})

test_that("RCow<f64> round-trip with NAs returns same object", {
  x <- c(1.0, NA, 3.0)
  expect_true(zero_copy_rcow_f64_identity(x))
})

test_that("RCow copy-on-write yields a fresh, value-correct object", {
  x <- c(1.0, 2.0, 3.0)
  y <- zero_copy_rcow_f64_mutated_is_copy(x)
  expect_equal(y, c(2.0, 4.0, 6.0))
  expect_equal(x, c(1.0, 2.0, 3.0)) # input untouched (copy-on-write)
})

# endregion

# region: Cow<str> scalar

test_that("Cow<str> from R is zero-copy (Borrowed)", {
  expect_true(zero_copy_cow_str_is_borrowed("hello"))
  expect_true(zero_copy_cow_str_is_borrowed(""))
  expect_true(zero_copy_cow_str_is_borrowed("unicode: \u00e9\u00e0\u00fc"))
})

# endregion

# region: Vec<Cow<str>>

test_that("Vec<Cow<str>> elements are all zero-copy (Borrowed)", {
  expect_true(zero_copy_vec_cow_str_all_borrowed(c("a", "b", "c")))
  expect_true(zero_copy_vec_cow_str_all_borrowed(c("hello", "world")))
  # NA maps to Cow::Borrowed("") in non-Option variant
  expect_true(zero_copy_vec_cow_str_all_borrowed(c("a", NA, "c")))
})

# endregion

# region: Arrow array identity (pointer recovery)

test_that("Float64Array round-trip returns same R object (zero-copy)", {
  x <- c(1.0, 2.0, 3.0)
  expect_true(zero_copy_arrow_f64_identity(x))
})

test_that("Float64Array with NAs round-trip returns same object", {
  x <- c(1.0, NA, 3.0)
  expect_true(zero_copy_arrow_f64_identity(x))
})

test_that("Int32Array round-trip returns same R object (zero-copy)", {
  x <- c(1L, 2L, 3L, 4L, 5L)
  expect_true(zero_copy_arrow_i32_identity(x))
})

test_that("Int32Array with NAs round-trip returns same object", {
  x <- c(1L, NA, 3L)
  expect_true(zero_copy_arrow_i32_identity(x))
})

test_that("Float64Array round-trip returns the values unchanged", {
  x <- c(1.5, NA, 3.5)
  expect_equal(zero_copy_arrow_f64_roundtrip(x), x)
})

test_that("Int32Array round-trip returns the values unchanged", {
  x <- c(1L, NA, 3L)
  expect_equal(zero_copy_arrow_i32_roundtrip(x), x)
})

test_that("ALTREP compact integer (1:n) correctly falls through to copy", {
  # ALTREP may share another object's storage, so it cannot establish unique
  # R buffer identity. Arrow conversion must return a copy.
  x <- 1:5
  expect_false(zero_copy_arrow_i32_identity(x))
  # But values are preserved correctly
  expect_equal(miniextendr:::arrow_i32_roundtrip(x), c(1L, 2L, 3L, 4L, 5L))
  # The Cow path always copies now (#880); values still round-trip correctly.
  expect_equal(zero_copy_cow_i32_roundtrip(x), 1:5)
})

test_that("UInt8Array round-trip returns same R object (zero-copy)", {
  x <- as.raw(c(1, 2, 3))
  expect_true(zero_copy_arrow_u8_identity(x))
})

test_that("Computed Arrow array is NOT the same object (different memory)", {
  x <- c(1.0, 2.0, 3.0)
  expect_false(zero_copy_arrow_f64_computed_is_different(x))
})

# endregion

# region: ProtectedStrVec

test_that("ProtectedStrVec counts unique strings", {
  expect_equal(zero_copy_protected_strvec_unique(c("a", "b", "c")), 3L)
  expect_equal(zero_copy_protected_strvec_unique(c("a", "a", "b")), 2L)
})

test_that("ProtectedStrVec handles NA", {
  expect_equal(zero_copy_protected_strvec_unique(c("a", NA_character_, "b")), 2L)
  expect_equal(zero_copy_protected_strvec_unique(c(NA_character_, NA_character_)), 0L)
})

# endregion

# region: Serialization — ALTREP objects with Rust-owned data

# These test the HARD case: data lives in Rust memory (ExternalPtr),
# NOT in R's heap. saveRDS must materialize the ALTREP.
#
# BUG FOUND: ALTREP readRDS in a fresh session returns empty vectors —
# even with library(miniextendr) loaded. The ALTREP class registration
# during R_init doesn't survive cross-session serialization. This is a
# known issue with R's ALTREP unserialize mechanism — the class must be
# registered before readRDS is called, and the package name in the
# serialized stream must match exactly.
#
# Same-session saveRDS/readRDS works correctly.

test_that("ALTREP values are correct before serialization", {
  x <- c(1.0, 2.0, 3.0)
  altrep_result <- zero_copy_arrow_f64_altrep(x)
  expect_equal(altrep_result, c(10.0, 20.0, 30.0))

  y <- c(1L, 2L, 3L)
  altrep_i32 <- zero_copy_arrow_i32_altrep(y)
  expect_equal(altrep_i32, c(101L, 102L, 103L))

  altrep_vec <- zero_copy_vec_f64_altrep(5L)
  expect_equal(altrep_vec, c(0.0, 1.5, 3.0, 4.5, 6.0))
})

test_that("ALTREP saveRDS does not crash (no longer segfaults)", {
  x <- c(1.0, 2.0, 3.0)
  altrep_result <- zero_copy_arrow_f64_altrep(x)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  expect_no_error(saveRDS(altrep_result, tmp))
  expect_true(file.size(tmp) > 0)
})

test_that("ALTREP same-session readRDS works", {
  x <- c(1.0, 2.0, 3.0)
  altrep_result <- zero_copy_arrow_f64_altrep(x)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep_result, tmp)
  loaded <- readRDS(tmp)
  expect_equal(loaded, c(10.0, 20.0, 30.0))
})

test_that("ALTREP with NAs: same-session readRDS preserves NAs", {
  x <- c(1.0, NA, 3.0)
  altrep_result <- zero_copy_arrow_f64_altrep(x)
  expect_true(is.na(altrep_result[2]))
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep_result, tmp)
  loaded <- readRDS(tmp)
  expect_equal(loaded[1], 10.0)
  expect_true(is.na(loaded[2]))
  expect_equal(loaded[3], 30.0)
})

# Skip cross-session (callr) tests on Windows: callr/processx leaves orphan
# Rterm processes that hold stdout pipe handles open, hanging R CMD check.
# Cross-session behavior is platform-independent and tested on Linux/macOS.

test_that("ALTREP cross-session readRDS works (classes registered at init)", {
  skip_on_os("windows")
  x <- c(1.0, 2.0, 3.0)
  altrep_result <- zero_copy_arrow_f64_altrep(x)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep_result, tmp)

  # Fresh R session — library(miniextendr) registers ALTREP classes at init
  loaded <- callr::r(function(path) {
    library(miniextendr)
    readRDS(path)
  }, args = list(path = tmp))

  expect_equal(loaded, c(10.0, 20.0, 30.0))
})

test_that("ALTREP cross-session readRDS WITHOUT package returns plain vector", {
  skip_on_os("windows")
  x <- c(1.0, 2.0, 3.0)
  altrep_result <- zero_copy_arrow_f64_altrep(x)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep_result, tmp)

  # Fresh session WITHOUT miniextendr — R should fall back to serialized state
  loaded <- callr::r(function(path) {
    readRDS(path)
  }, args = list(path = tmp))

  # R reconstructs from the serialized state (a plain numeric vector)
  expect_equal(loaded, c(10.0, 20.0, 30.0))
})

test_that("Vec<f64> ALTREP cross-session readRDS", {
  skip_on_os("windows")
  altrep_result <- zero_copy_vec_f64_altrep(4L)
  expect_equal(altrep_result, c(0.0, 1.5, 3.0, 4.5))

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep_result, tmp)

  loaded <- callr::r(function(path) {
    library(miniextendr)
    readRDS(path)
  }, args = list(path = tmp))
  expect_equal(loaded, c(0.0, 1.5, 3.0, 4.5))

  # Also without package
  loaded2 <- callr::r(function(path) readRDS(path), args = list(path = tmp))
  expect_equal(loaded2, c(0.0, 1.5, 3.0, 4.5))
})

test_that("Int32 ALTREP cross-session readRDS", {
  skip_on_os("windows")
  altrep_result <- zero_copy_arrow_i32_altrep(c(1L, 2L, 3L))
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep_result, tmp)

  loaded <- callr::r(function(path) {
    library(miniextendr)
    readRDS(path)
  }, args = list(path = tmp))
  expect_equal(loaded, c(101L, 102L, 103L))
})

test_that("Arrow ALTREP with NAs cross-session readRDS", {
  skip_on_os("windows")
  x <- c(1.0, NA, 3.0)
  altrep_result <- zero_copy_arrow_f64_altrep(x)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep_result, tmp)

  loaded <- callr::r(function(path) {
    library(miniextendr)
    readRDS(path)
  }, args = list(path = tmp))
  expect_equal(loaded[1], 10.0)
  expect_true(is.na(loaded[2]))
  expect_equal(loaded[3], 30.0)
})

test_that("Double round-trip: saveRDS → readRDS → saveRDS → readRDS", {
  skip_on_os("windows")
  altrep_result <- zero_copy_vec_f64_altrep(3L)
  expected <- c(0.0, 1.5, 3.0)

  tmp1 <- tempfile(fileext = ".rds")
  tmp2 <- tempfile(fileext = ".rds")
  on.exit(unlink(c(tmp1, tmp2)), add = TRUE)

  saveRDS(altrep_result, tmp1)
  loaded1 <- readRDS(tmp1)
  expect_equal(loaded1, expected)

  saveRDS(loaded1, tmp2)
  loaded2 <- callr::r(function(path) readRDS(path), args = list(path = tmp2))
  expect_equal(loaded2, expected)
})

test_that("Materialized ALTREP serializes correctly", {
  skip_on_os("windows")
  altrep_result <- zero_copy_arrow_f64_altrep(c(1.0, 2.0, 3.0))
  # Force materialization by accessing all elements
  dummy <- sum(altrep_result)
  expect_equal(dummy, 60.0)

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep_result, tmp)

  loaded <- callr::r(function(path) readRDS(path), args = list(path = tmp))
  expect_equal(loaded, c(10.0, 20.0, 30.0))
})

# endregion

# region: alloc_r_backed_buffer round-trip

test_that("alloc_r_backed_buffer creates R-backed Arrow that returns same SEXP", {
  result <- zero_copy_alloc_r_backed(3L)
  expect_equal(result, c(100.0, 200.0, 300.0))
})

# endregion

# region: Sliced buffer fallback

test_that("Sliced Arrow array does NOT return original SEXP (different pointer)", {
  x <- c(10.0, 20.0, 30.0, 40.0)
  expect_true(zero_copy_arrow_f64_sliced(x))
})

# endregion
