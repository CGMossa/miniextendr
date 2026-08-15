# Cross-session persistence of a pointer-backed S7 object (S7SerdePersist,
# rpkg/src/rust/s7_serde_persist_tests.rs).
#
# Hypothesis under test: an S7 class whose Rust struct derives
# Serialize/Deserialize, holding both R-native-representable data (String,
# Vec<f64>, Option<String>) and Rust-native data (u64, BTreeMap), is saved to
# .rds/.rda and read back in a NEW R session.
#
# A new session matters: within one session a serialized pointer could appear
# to work by accident (address reuse, CHARSXP cache); the callr::r()
# subprocess rules that out. The answers these tests pin down:
#
# 1. Saving the S7 OBJECT itself does not survive — R serialization nulls the
#    ExternalPtr address (even same-session), so the class shell loads but
#    the first method call errors.
# 2. The serde bridge survives — lowering to plain R data via
#    miniextendr:::s7_persist_to_r(), saving THAT, and rebuilding with
#    miniextendr:::S7SerdePersist_from_r_data() restores the full Rust state.

# Skip on Windows: callr/processx leaves orphan Rterm processes holding stdout
# pipe handles, hanging R CMD check (same as
# test-altrep-serialization-cross-session.R).
skip_on_os("windows")

make_persist <- function() {
  miniextendr:::S7SerdePersist(
    label = "alpha",
    values = c(1.5, 2.5, 3.5),
    maybe_note = "note-1",
    keys = c("a", "b"),
    counts = c(1L, 2L)
  )
}

test_that("saveRDS nulls the ExternalPtr even within the same session", {
  obj <- make_persist()
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))
  saveRDS(obj, tmp)
  back <- readRDS(tmp)
  # The S7 shell survives, but the pointer inside is dead: state is gone
  # before any new session enters the picture.
  expect_s3_class(back, "miniextendr::S7SerdePersist")
  expect_error(miniextendr:::s7_persist_label(back))
})

test_that("the S7 object itself does not survive saveRDS into a new session", {
  skip_if_not_installed("callr")
  obj <- make_persist()
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))
  saveRDS(obj, tmp)
  res <- callr::r(function(path) {
    library(miniextendr)
    obj <- readRDS(path)
    list(
      klass = class(obj),
      label = tryCatch(
        list(ok = TRUE, val = miniextendr:::s7_persist_label(obj)),
        error = function(e) list(ok = FALSE, msg = conditionMessage(e))
      )
    )
  }, args = list(path = tmp))
  # Class metadata round-trips fine...
  expect_true("miniextendr::S7SerdePersist" %in% res$klass)
  # ...but the Rust state is unreachable: the pointer is null in the new
  # session.
  expect_false(res$label$ok)
})

test_that("the S7 object itself does not survive save()/load() (.rda) either", {
  skip_if_not_installed("callr")
  obj <- make_persist()
  tmp <- tempfile(fileext = ".rda")
  on.exit(unlink(tmp))
  save(obj, file = tmp)
  res <- callr::r(function(path) {
    library(miniextendr)
    load(path)  # restores `obj`
    tryCatch(
      list(ok = TRUE, val = miniextendr:::s7_persist_label(obj)),
      error = function(e) list(ok = FALSE, msg = conditionMessage(e))
    )
  }, args = list(path = tmp))
  expect_false(res$ok)
})

test_that("serde bridge restores full state in a new session (rds)", {
  skip_if_not_installed("callr")
  obj <- make_persist()
  data <- miniextendr:::s7_persist_to_r(obj)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))
  saveRDS(data, tmp)
  res <- callr::r(function(path) {
    library(miniextendr)
    obj2 <- miniextendr:::S7SerdePersist_from_r_data(readRDS(path))
    list(
      klass = class(obj2),
      label = miniextendr:::s7_persist_label(obj2),
      values = miniextendr:::s7_persist_values(obj2),
      note = miniextendr:::s7_persist_note(obj2),
      id = miniextendr:::s7_persist_id(obj2),
      a = miniextendr:::s7_persist_lookup_get(obj2, "a"),
      b = miniextendr:::s7_persist_lookup_get(obj2, "b"),
      missing = tryCatch(
        list(ok = TRUE, val = miniextendr:::s7_persist_lookup_get(obj2, "zzz")),
        error = function(e) list(ok = FALSE, msg = conditionMessage(e))
      ),
      # Re-lower in the new session: must equal what the old session lowered.
      data2 = miniextendr:::s7_persist_to_r(obj2)
    )
  }, args = list(path = tmp))
  expect_true("miniextendr::S7SerdePersist" %in% res$klass)
  # R-native-representable fields.
  expect_equal(res$label, "alpha")
  expect_equal(res$values, c(1.5, 2.5, 3.5))
  expect_equal(res$note, "note-1")
  # Rust-native fields: u64 beyond .Machine$integer.max, BTreeMap lookups.
  expect_equal(res$id, "3735928559")  # 0xDEADBEEF
  expect_equal(res$a, 1L)
  expect_equal(res$b, 2L)
  # Pins current behavior: Option::None from a class METHOD raises NONE_ERR
  # ("returned no value") instead of returning NA as the standalone-fn
  # absence contract documents. When #1415 is fixed, flip this to
  # expect_true(is.na(res$missing$val)).
  expect_false(res$missing$ok)
  expect_match(res$missing$msg, "returned no value")
  # Lower → save → load → rebuild → lower is a fixed point.
  expect_equal(res$data2, data)
})

test_that("serde bridge restores full state in a new session (rda)", {
  skip_if_not_installed("callr")
  obj <- make_persist()
  persist_data <- miniextendr:::s7_persist_to_r(obj)
  tmp <- tempfile(fileext = ".rda")
  on.exit(unlink(tmp))
  save(persist_data, file = tmp)
  res <- callr::r(function(path) {
    library(miniextendr)
    load(path)  # restores `persist_data`
    obj2 <- miniextendr:::S7SerdePersist_from_r_data(persist_data)
    list(
      label = miniextendr:::s7_persist_label(obj2),
      values = miniextendr:::s7_persist_values(obj2),
      id = miniextendr:::s7_persist_id(obj2),
      b = miniextendr:::s7_persist_lookup_get(obj2, "b")
    )
  }, args = list(path = tmp))
  expect_equal(res$label, "alpha")
  expect_equal(res$values, c(1.5, 2.5, 3.5))
  expect_equal(res$id, "3735928559")
  expect_equal(res$b, 2L)
})

test_that("lowered serde data is plain R data, readable without the package", {
  skip_if_not_installed("callr")
  # A session withOUT miniextendr can read the .rds and inspect it; only
  # rebuilding the live object needs the package.
  obj <- make_persist()
  data <- miniextendr:::s7_persist_to_r(obj)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))
  saveRDS(data, tmp)
  res <- callr::r(function(path) {
    d <- readRDS(path)  # no library(miniextendr)
    list(label = d$label, id_class = class(d$id))
  }, args = list(path = tmp))
  expect_equal(res$label, "alpha")
})
