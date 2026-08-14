# Scaffold smoke tests for CI
#
# Validate CRAN prep workflows end-to-end on a freshly scaffolded package.

# =============================================================================
# Standalone CRAN prep smoke test (requires cargo + autoconf)
# =============================================================================

test_that("standalone scaffold can vendor for CRAN prep", {
  skip_on_ci()
  skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  skip_if_no_local_repo()

  miniextendr_path <- find_miniextendr_repo()

  tmp <- tempfile("cran-standalone-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  pkg_path <- file.path(tmp, "testpkg")

  # Create package
  suppressWarnings(suppressMessages({
    usethis::create_package(pkg_path, open = FALSE)
    use_miniextendr(path = pkg_path, local_path = miniextendr_path)
  }))

  # Autoconf + configure
  suppressMessages({
    miniextendr_autoconf(path = pkg_path)
    miniextendr_configure(path = pkg_path)
  })
  tarball <- file.path(pkg_path, "inst", "vendor.tar.xz")
  expect_false(
    file.exists(tarball),
    info = "configure must not create a vendor tarball"
  )

  # Vendor for CRAN
  suppressMessages({
    miniextendr_vendor(path = pkg_path)
  })

  # Verify vendor tarball was created
  expect_true(file.exists(tarball), info = "vendor.tar.xz should exist")
  expect_true(file.size(tarball) > 0, info = "vendor.tar.xz should be non-empty")

  # Configure in tarball mode (inst/vendor.tar.xz present — configure auto-detects)
  suppressMessages({
    miniextendr_configure(path = pkg_path)
  })

  # cargo check --offline should work with vendored deps
  result <- withr::with_dir(file.path(pkg_path, "src", "rust"), {
    system2("cargo", c("check", "--offline"), stdout = TRUE, stderr = TRUE)
  })
  status <- attr(result, "status")
  expect_true(is.null(status) || status == 0,
              info = paste("cargo check --offline failed:", paste(result, collapse = "\n")))
})

# =============================================================================
# Monorepo CRAN prep smoke test (requires cargo + autoconf)
# =============================================================================

test_that("monorepo scaffold can vendor for CRAN prep", {
  skip_on_ci()
  skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  skip_if_no_local_repo()

  miniextendr_path <- find_miniextendr_repo()

  tmp <- tempfile("cran-monorepo-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # Create monorepo
  suppressMessages({
    create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg-rs",
                                local_path = miniextendr_path, open = FALSE)
  })

  rpkg_path <- file.path(tmp, "testpkg")

  # Autoconf + configure
  suppressMessages({
    miniextendr_autoconf(path = rpkg_path)
    miniextendr_configure(path = rpkg_path)
  })

  # Vendor for CRAN
  suppressMessages({
    miniextendr_vendor(path = rpkg_path)
  })

  # Verify vendor tarball was created
  tarball <- file.path(rpkg_path, "inst", "vendor.tar.xz")
  expect_true(file.exists(tarball), info = "vendor.tar.xz should exist")
  expect_true(file.size(tarball) > 0, info = "vendor.tar.xz should be non-empty")
})
