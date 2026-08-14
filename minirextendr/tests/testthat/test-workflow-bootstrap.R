# Tests for miniextendr_build()'s fresh-package bootstrap helpers (#822).
# These exercise the pure-R guards without invoking a compile/build.

make_pkg_root <- function() {
  tmp <- tempfile("workflow-bootstrap-")
  dir.create(tmp)
  writeLines(
    "Package: testpkg\nTitle: Test\nVersion: 0.1.0\n",
    file.path(tmp, "DESCRIPTION")
  )
  writeLines("", file.path(tmp, "NAMESPACE"))
  dir.create(file.path(tmp, "R"))
  tmp
}

test_that("wrappers_file_exists detects a generated *-wrappers.R", {
  tmp <- make_pkg_root()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # Fresh package: no wrappers file yet.
  expect_false(minirextendr:::wrappers_file_exists(tmp))

  # Once a *-wrappers.R lands, it is detected regardless of the package stem.
  writeLines("# generated", file.path(tmp, "R", "testpkg-wrappers.R"))
  expect_true(minirextendr:::wrappers_file_exists(tmp))
})

test_that("wrappers_file_exists is FALSE when the R/ directory is missing", {
  tmp <- tempfile("workflow-bootstrap-nodir-")
  dir.create(tmp)
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  expect_false(minirextendr:::wrappers_file_exists(tmp))
})

test_that("clear_install_mode_latch removes tarball, vendor/, and .cargo/", {
  tmp <- make_pkg_root()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  inst_dir <- file.path(tmp, "inst")
  dir.create(inst_dir)
  tarball <- file.path(inst_dir, "vendor.tar.xz")
  writeLines("fake tarball", tarball)

  vendor_dir <- file.path(tmp, "vendor")
  dir.create(vendor_dir)
  writeLines("x", file.path(vendor_dir, "marker"))

  cargo_dir <- file.path(tmp, "src", "rust", ".cargo")
  dir.create(cargo_dir, recursive = TRUE)
  writeLines("[build]", file.path(cargo_dir, "config.toml"))

  expect_true(minirextendr:::clear_install_mode_latch(tmp))

  expect_false(file.exists(tarball))
  expect_false(dir.exists(vendor_dir))
  expect_false(dir.exists(cargo_dir))
})

test_that("clear_install_mode_latch is a no-op when nothing is present", {
  tmp <- make_pkg_root()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  expect_true(minirextendr:::clear_install_mode_latch(tmp))
})

test_that("bootstrap_fresh_wrappers sets MINIEXTENDR_FORCE_WRAPPER_GEN=1 during installs", {
  # bootstrap_fresh_wrappers() keeps the override around its
  # devtools::install() calls so stale generated tarball-mode configuration
  # cannot suppress the cdylib wrapper-gen pass.
  #
  # We mock devtools::install and miniextendr_configure to avoid a real compile.
  # devtools::document is also mocked to avoid roxygenising a stub package.
  # wrappers_file_exists is mocked to return TRUE after the first install so
  # the function progresses to the re-install pass, exercising both call sites.
  install_call_count <- 0L

  testthat::local_mocked_bindings(
    install = function(...) {
      install_call_count <<- install_call_count + 1L
      expect_equal(
        Sys.getenv("MINIEXTENDR_FORCE_WRAPPER_GEN"),
        "1",
        info = paste("install call", install_call_count)
      )
      invisible(NULL)
    },
    .package = "devtools"
  )

  testthat::local_mocked_bindings(
    document = function(...) invisible(NULL),
    .package = "devtools"
  )

  # Stub out internal helpers so bootstrap_fresh_wrappers() can run to
  # completion without a real package tree or configure script.
  testthat::local_mocked_bindings(
    miniextendr_configure = function(...) invisible(TRUE),
    clear_install_mode_latch = function(...) invisible(TRUE),
    wrappers_file_exists = function(...) TRUE,
    .package = "minirextendr"
  )

  tmp <- make_pkg_root()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  old_force <- Sys.getenv("MINIEXTENDR_FORCE_WRAPPER_GEN", unset = NA)
  on.exit(
    if (is.na(old_force)) Sys.unsetenv("MINIEXTENDR_FORCE_WRAPPER_GEN")
    else Sys.setenv(MINIEXTENDR_FORCE_WRAPPER_GEN = old_force),
    add = TRUE
  )
  # Ensure the var is absent before the call so we test that bootstrap sets it.
  Sys.unsetenv("MINIEXTENDR_FORCE_WRAPPER_GEN")

  minirextendr:::bootstrap_fresh_wrappers(tmp)

  # Both install calls (initial wrapper-gen + re-install after document) must
  # have been reached.
  expect_equal(install_call_count, 2L)

  # The env var must be restored (unset) after bootstrap_fresh_wrappers returns.
  expect_equal(Sys.getenv("MINIEXTENDR_FORCE_WRAPPER_GEN", unset = ""), "")
})
