test_that("normal function is exported", {
  # Normal functions are exported and callable
  expect_equal(export_control_normal(), "normal")

  # Check it appears in NAMESPACE
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  expect_true(any(grepl("export_control_normal", ns)))
})

test_that("internal function is callable but not exported", {
  # Internal functions are still callable via :::
  expect_equal(miniextendr:::export_control_internal(), "internal")

  # Check it does NOT appear as export() in NAMESPACE
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  export_lines <- ns[grepl("^export\\(", ns)]
  expect_false(any(grepl("export_control_internal", export_lines)))
})

test_that("noexport function is callable but not exported", {
  # Noexport functions are still callable via :::
  expect_equal(miniextendr:::export_control_noexport(), "noexport")

  # Check it does NOT appear as export() in NAMESPACE
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  export_lines <- ns[grepl("^export\\(", ns)]
  expect_false(any(grepl("export_control_noexport", export_lines)))
})

# region: man-page-level distinction (audit A10)
#
# `internal` and `noexport` both suppress NAMESPACE `export()`, but must
# differ at the documentation level: `internal` stays documented (with an
# alias) under `\keyword{internal}`; `noexport` gets no Rd contribution at
# all (`@noRd`) — no alias, anywhere. Before the fix, `noexport` still
# contributed an `\alias{}` to the shared `export_control_tests.Rd` page.

test_that("internal function has an alias in the rendered Rd", {
  rd_db <- tryCatch(tools::Rd_db("miniextendr"), error = function(e) NULL)
  skip_if(is.null(rd_db), "tools::Rd_db('miniextendr') unavailable — package not installed")
  rd_name <- grep("export_control", names(rd_db), value = TRUE)[1]
  skip_if(is.na(rd_name), "export_control_tests.Rd not found — package not documented")

  rd <- rd_db[[rd_name]]
  rd_text <- paste(capture.output(print(rd)), collapse = "\n")

  expect_true(
    grepl("export_control_internal", rd_text),
    info = paste(
      "`export_control_internal` must have an alias in the rendered Rd",
      "(documented, just unexported). Got:\n", rd_text
    )
  )
})

test_that("noexport function has no alias anywhere in the rendered Rd", {
  rd_db <- tryCatch(tools::Rd_db("miniextendr"), error = function(e) NULL)
  skip_if(is.null(rd_db), "tools::Rd_db('miniextendr') unavailable — package not installed")

  # `export_control_noexport` must not be an alias target on ANY Rd page —
  # not just the shared `export_control_tests.Rd` page it used to leak into.
  has_alias <- vapply(rd_db, function(rd) {
    grepl("export_control_noexport", paste(capture.output(print(rd)), collapse = "\n"))
  }, logical(1))

  expect_false(
    any(has_alias),
    info = paste(
      "`export_control_noexport` must not appear in any rendered Rd page.",
      "Found in:", paste(names(rd_db)[has_alias], collapse = ", ")
    )
  )
})

# The same distinction on the trait-impl codegen path: `ExportControlTraitPoint`
# (rpkg/src/rust/adapter_traits_tests.rs) implements `RDebug` with `internal`
# and `RDisplay` with `noexport`.

test_that("internal trait method has an alias in the rendered Rd", {
  rd_db <- tryCatch(tools::Rd_db("miniextendr"), error = function(e) NULL)
  skip_if(is.null(rd_db), "tools::Rd_db('miniextendr') unavailable — package not installed")
  rd_name <- grep("ExportControlTraitPoint", names(rd_db), value = TRUE)[1]
  skip_if(is.na(rd_name), "ExportControlTraitPoint.Rd not found — package not documented")

  rd <- rd_db[[rd_name]]
  rd_text <- paste(capture.output(print(rd)), collapse = "\n")

  expect_true(
    grepl("RDebug", rd_text, fixed = TRUE),
    info = paste(
      "internal trait impl (RDebug) must have an alias in the rendered Rd.",
      "Got:\n", rd_text
    )
  )
})

test_that("noexport trait method has no alias in the rendered Rd", {
  rd_db <- tryCatch(tools::Rd_db("miniextendr"), error = function(e) NULL)
  skip_if(is.null(rd_db), "tools::Rd_db('miniextendr') unavailable — package not installed")
  rd_name <- grep("ExportControlTraitPoint", names(rd_db), value = TRUE)[1]
  skip_if(is.na(rd_name), "ExportControlTraitPoint.Rd not found — package not documented")

  rd <- rd_db[[rd_name]]
  rd_text <- paste(capture.output(print(rd)), collapse = "\n")

  expect_false(
    grepl("RDisplay", rd_text, fixed = TRUE),
    info = paste(
      "noexport trait impl (RDisplay) must NOT have an alias in the rendered Rd.",
      "Got:\n", rd_text
    )
  )
})

# endregion

test_that("postfix names the internal entry point from the Rust name (#1451)", {
  # Generated wrapper: Rust `export_control_delegate` + postfix `_impl`.
  expect_equal(miniextendr:::export_control_delegate_impl(4L), 8L)
  expect_false("export_control_delegate_impl" %in% getNamespaceExports("miniextendr"))
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  expect_false(any(grepl("export_control_delegate_impl", ns)))

  # Hand-written public surface delegates to it.
  expect_true(any(grepl("^export\\(export_control_delegate\\)$", ns)))
  expect_equal(export_control_delegate(4), 8L)
  expect_error(export_control_delegate("a"), "is.numeric")
})

test_that("postfix renames an impl method", {
  w <- ExportControlWidget$new(5L)
  expect_equal(w$bump_impl(2L), 7L)
  expect_null(w$bump)
})
