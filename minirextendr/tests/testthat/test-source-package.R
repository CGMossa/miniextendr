test_that("built source package excludes maintainer files and resolves help links", {
  skip_if_no_local_repo()
  skip_if_not_installed("pkgbuild")

  package <- file.path(find_miniextendr_repo(), "minirextendr")
  destination <- withr::local_tempdir()
  archive <- pkgbuild::build(
    package, dest_path = destination, binary = FALSE,
    vignettes = FALSE, manual = FALSE, quiet = TRUE
  )
  contents <- utils::untar(archive, list = TRUE)
  expect_true(all(c(
    "minirextendr/DESCRIPTION",
    "minirextendr/R/use-release-workflow.R",
    "minirextendr/man/use_release_workflow.Rd"
  ) %in% contents))
  expect_false(any(c("minirextendr/AGENTS.md", "minirextendr/CLAUDE.md") %in% contents))

  utils::untar(archive, exdir = destination)
  # A maintainer's installed miniextendr can hide an undeclared external Rd
  # link. Check declared packages too, using the same resolver as R CMD check.
  withr::local_envvar(`_R_CHECK_XREFS_PKGS_ARE_DECLARED_` = "TRUE")
  # The resolver always scans R's recommended-package help. R CMD check puts
  # dummy packages ahead of .Library to hide undeclared test dependencies;
  # use the real standard library for this metadata-only scan.
  withr::local_libpaths(c(.Library, .libPaths()))
  xrefs <- tools:::.check_Rd_xrefs(dir = file.path(destination, "minirextendr"))
  expect_identical(format(xrefs), character())
})
