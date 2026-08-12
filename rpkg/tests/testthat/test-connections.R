# Tests for custom R connections backed by Rust types.
#
# Requires the `connections` feature.

skip_if_missing_feature("connections")

# =============================================================================
# String input connection
# =============================================================================

test_that("string_input_connection reads multi-line content", {
  con <- string_input_connection("line1\nline2\nline3")
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_length(lines, 3)
  expect_equal(lines, c("line1", "line2", "line3"))
})

test_that("string_input_connection reads single line", {
  con <- string_input_connection("hello")
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_length(lines, 1)
  expect_equal(lines, "hello")
})

test_that("string_input_connection reads empty string", {
  con <- string_input_connection("")
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_length(lines, 0)
})

# =============================================================================
# Counter connection
# =============================================================================

test_that("counter_connection generates sequential integers", {
  con <- counter_connection(1L, 5L)
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_equal(lines, as.character(1:5))
})

test_that("counter_connection with single value", {
  con <- counter_connection(42L, 42L)
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_equal(lines, "42")
})

# =============================================================================
# Memory connection (read + write + seek)
# =============================================================================

test_that("memory_connection write then read roundtrip", {
  con <- memory_connection()
  on.exit(close(con))
  writeLines("Hello, World!", con)
  seek(con, 0)
  lines <- readLines(con, warn = FALSE)
  expect_equal(lines, "Hello, World!")
})

test_that("memory_connection write multiple lines", {
  con <- memory_connection()
  on.exit(close(con))
  writeLines(c("foo", "bar", "baz"), con)
  seek(con, 0)
  lines <- readLines(con, warn = FALSE)
  expect_equal(lines, c("foo", "bar", "baz"))
})

test_that("memory_connection preserves long formatted writes", {
  con <- memory_connection()
  on.exit(close(con))
  expected <- strrep("x", 12000L)
  writeLines(expected, con)
  seek(con, 0)
  actual <- readLines(con, warn = FALSE)
  expect_identical(actual, expected)
  expect_equal(nchar(actual, type = "bytes"), 12000L)
})

# =============================================================================
# Uppercase transform connection
# =============================================================================

test_that("uppercase_connection transforms to uppercase", {
  con <- uppercase_connection("hello world")
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_equal(lines, "HELLO WORLD")
})

test_that("uppercase_connection preserves non-alpha characters", {
  con <- uppercase_connection("abc 123 !@#")
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_equal(lines, "ABC 123 !@#")
})

# =============================================================================
# ROT13 connection
# =============================================================================

test_that("rot13_connection encodes text", {
  con <- rot13_connection("hello")
  on.exit(close(con))
  lines <- readLines(con, warn = FALSE)
  expect_equal(lines, "uryyb")
})

test_that("rot13 double-application returns original", {
  # ROT13 is its own inverse
  con1 <- rot13_connection("hello world")
  on.exit(close(con1), add = TRUE)
  encoded <- readLines(con1, warn = FALSE)

  con2 <- rot13_connection(encoded)
  on.exit(close(con2), add = TRUE)
  decoded <- readLines(con2, warn = FALSE)

  expect_equal(decoded, "hello world")
})

# =============================================================================
# Cursor connection (binary read/write/seek)
# =============================================================================

test_that("empty_cursor_connection binary roundtrip", {
  con <- empty_cursor_connection()
  on.exit(close(con))
  data <- as.raw(1:10)
  writeBin(data, con)
  seek(con, 0)
  result <- readBin(con, "raw", 10)
  expect_equal(result, data)
})

test_that("cursor_connection reads pre-filled data", {
  data <- charToRaw("Hello")
  con <- cursor_connection(data)
  on.exit(close(con))
  result <- readBin(con, "raw", 5)
  expect_equal(rawToChar(result), "Hello")
})

# =============================================================================
# Standard stream connections — issue #175
# =============================================================================

test_that("rust_get_stdout returns R's stdout connection", {
  con <- rust_get_stdout()
  expect_true(inherits(con, "terminal"))
  expect_true(inherits(con, "connection"))
  expect_equal(summary(con)$description, "stdout")
})

test_that("rust_get_stderr returns R's stderr connection", {
  con <- rust_get_stderr()
  expect_true(inherits(con, "terminal"))
  expect_true(inherits(con, "connection"))
  expect_equal(summary(con)$description, "stderr")
})

test_that("rust_write_to_stderr captured by capture.output(type='message')", {
  out <- capture.output(rust_write_to_stderr("hello-from-rust"), type = "message")
  expect_true(any(grepl("hello-from-rust", out)))
})

test_that("rust_write_to_stderr captured by sink(type='message')", {
  buf <- character(0)
  tc <- textConnection("buf", "w", local = TRUE)
  sink(tc, type = "message")
  rust_write_to_stderr("sink-captured")
  sink(NULL, type = "message")
  close(tc)
  expect_true(any(grepl("sink-captured", buf)))
})

# =============================================================================
# Null connection — issue #176
# =============================================================================

test_that("rust_get_null_connection returns an open write-capable connection", {
  con <- rust_get_null_connection()
  on.exit(close(con))
  caps <- summary(con)
  expect_equal(caps$mode, "w")
  expect_equal(caps$opened, "opened")
  expect_equal(caps[["can read"]], "no")
  expect_equal(caps[["can write"]], "yes")
})

test_that("rust_write_to_null succeeds silently (no error)", {
  expect_no_error(rust_write_to_null("discarded message"))
})

test_that("close(rust_get_null_connection()) does not error", {
  con <- rust_get_null_connection()
  expect_no_error(close(con))
})

test_that("rust_get_null_connection double-close is safe", {
  con <- rust_get_null_connection()
  close(con)
  rm(con)
  gc()
  succeed()
})

# =============================================================================
# txtProgressBar (PR B / #177)
# =============================================================================

test_that("rust_run_progress emits a style-3 progress bar", {
  out <- capture.output(rust_run_progress(5L))
  # style 3 looks like: |====      | 80% or |==========| 100%
  expect_true(any(grepl("\\|.+\\|.+%", out)))
})

test_that("rust_run_progress_to_stderr captures via sink(message)", {
  buf <- character(0)
  con <- textConnection("buf", "w", local = TRUE)
  sink(con, type = "message")
  rust_run_progress_to_stderr(5L)
  sink(NULL, type = "message")
  close(con)
  expect_true(any(grepl("%", buf)))
})

test_that("RTxtProgressBar Drop runs cleanly", {
  # Exercises the auto-close Drop path end-to-end without error.
  expect_no_error(rust_run_progress(3L))
})

test_that("RTxtProgressBar explicit close() succeeds and Drop is no-op", {
  expect_true(rust_run_progress_explicit_close(5L))
})

test_that("gc_stress_txt_progress_bar does not segfault", {
  expect_no_error(miniextendr:::gc_stress_txt_progress_bar())
})
