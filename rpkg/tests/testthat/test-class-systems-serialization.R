# saveRDS/readRDS survival matrix across ALL SIX class systems (companion to
# test-s7-serde-cross-session.R, which covers S7 in depth).
#
# Architecture predicts the split:
# - R6 / Env / S3 / S4 / S7 store their Rust state behind an EXTPTRSXP
#   (`private$.ptr` / env `.ptr` / `@ptr` / `@.ptr`). R's serializer writes
#   only EXTPTR_PROT/EXTPTR_TAG and the reader nulls the address
#   (r-svn src/main/serialize.c), so the loaded object's class survives but
#   the first pointer-reading method call errors.
# - Vctrs is the exception BY CONSTRUCTION: its constructor returns the
#   vector payload and the R wrapper wraps plain data with vctrs::new_vctr(),
#   so there is no Rust state to lose — the object fully survives.

skip_on_os("windows")  # callr orphan-process rationale, same as sibling suites

# Save obj, load it in a fresh R session, run the system-specific value
# method there; returns list(ok=, val=|msg=) plus the loaded class vector.
probe_in_new_session <- function(obj, system) {
  skip_if_not_installed("callr")
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))
  saveRDS(obj, tmp)
  callr::r(function(path, system) {
    library(miniextendr)
    ns <- asNamespace("miniextendr")
    obj <- readRDS(path)
    read_value <- switch(system,
      r6 = function(o) o$value(),
      env = function(o) o$value(),
      s3 = function(o) ns$s3_value(o),
      s4 = function(o) ns$s4_value(o),
      s7 = function(o) ns$s7_value(o),
      vctrs = function(o) unclass(o)
    )
    res <- tryCatch(
      list(ok = TRUE, val = read_value(obj)),
      error = function(e) list(ok = FALSE, msg = conditionMessage(e))
    )
    res$klass <- class(obj)
    res
  }, args = list(path = tmp, system = system))
}

test_that("R6 object: class survives saveRDS, pointer state does not", {
  obj <- miniextendr:::R6Counter$new(7L)
  res <- probe_in_new_session(obj, "r6")
  expect_true("R6Counter" %in% res$klass)
  expect_false(res$ok)
})

test_that("Env object: class survives saveRDS, pointer state does not", {
  obj <- miniextendr:::ReceiverCounter$new(7L)
  res <- probe_in_new_session(obj, "env")
  expect_true("ReceiverCounter" %in% res$klass)
  expect_false(res$ok)
})

test_that("S3 object: class survives saveRDS, pointer state does not", {
  obj <- miniextendr:::new_s3counter(7L)
  res <- probe_in_new_session(obj, "s3")
  expect_true("S3Counter" %in% res$klass)
  expect_false(res$ok)
})

test_that("S4 object: class survives saveRDS, pointer state does not", {
  obj <- miniextendr:::S4Counter(7L)
  res <- probe_in_new_session(obj, "s4")
  expect_true("S4Counter" %in% res$klass)
  expect_false(res$ok)
})

test_that("S7 object: class survives saveRDS, pointer state does not", {
  obj <- miniextendr:::S7Counter(7L)
  res <- probe_in_new_session(obj, "s7")
  expect_true("miniextendr::S7Counter" %in% res$klass)
  expect_false(res$ok)
})

test_that("Vctrs object survives saveRDS INTACT (data-backed, no pointer)", {
  obj <- miniextendr:::new_percent(0.5)
  res <- probe_in_new_session(obj, "vctrs")
  expect_true("percent" %in% res$klass)
  expect_true(res$ok)
  expect_identical(res$val, 0.5)
})
