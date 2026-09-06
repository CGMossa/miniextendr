# Tests for use_local_miniextendr() / unuse_local_miniextendr() (#908)
#
# All tests use tempdir fixtures — no compilation, no autoconf.

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

make_mx_pkg <- function() {
  # Minimal miniextendr-consumer package (configure.ac with CARGO_FEATURES
  # and src/rust/Cargo.toml + src/Makevars.in present).
  tmp <- tempfile("mx-local-pkg-")
  dir.create(tmp, recursive = TRUE)
  writeLines(
    c("Package: testpkg", "Title: Test", "Version: 0.1.0", ""),
    file.path(tmp, "DESCRIPTION")
  )
  writeLines("", file.path(tmp, "NAMESPACE"))
  writeLines(
    c(
      "AC_INIT([testpkg], [0.1.0])",
      "CARGO_FEATURES=''",
      "AC_OUTPUT"
    ),
    file.path(tmp, "configure.ac")
  )
  dir.create(file.path(tmp, "src", "rust"), recursive = TRUE)
  writeLines(
    c("[package]", 'name = "testpkg"'),
    file.path(tmp, "src", "rust", "Cargo.toml")
  )
  writeLines("", file.path(tmp, "src", "Makevars.in"))
  tmp
}

make_fake_mx_repo <- function() {
  # Fake miniextendr checkout: has miniextendr-api/Cargo.toml.
  repo <- tempfile("fake-mx-repo-")
  dir.create(file.path(repo, "miniextendr-api"), recursive = TRUE)
  writeLines(
    c("[package]", 'name = "miniextendr-api"'),
    file.path(repo, "miniextendr-api", "Cargo.toml")
  )
  repo
}

# ---------------------------------------------------------------------------
# use_local_miniextendr() — basic write / normalize / validate
# ---------------------------------------------------------------------------

test_that("use_local_miniextendr() writes .miniextendr-local with absolute path", {
  pkg <- make_mx_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  repo <- make_fake_mx_repo()
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)

  withr::local_options(usethis.quiet = TRUE)
  use_local_miniextendr(repo, path = pkg)

  marker <- file.path(pkg, ".miniextendr-local")
  expect_true(file.exists(marker))
  recorded <- trimws(readLines(marker, warn = FALSE))
  # Must be an absolute path matching the canonical form of repo.
  abs_repo <- normalizePath(repo, winslash = "/", mustWork = TRUE)
  abs_repo <- sub("^\\\\\\\\\\?\\\\", "", abs_repo)
  abs_repo <- sub("^//\\?/", "", abs_repo)
  abs_repo <- gsub("\\\\", "/", abs_repo)
  expect_equal(recorded, abs_repo)
})

test_that("use_local_miniextendr() adds .miniextendr-local to .gitignore and .Rbuildignore", {
  pkg <- make_mx_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  repo <- make_fake_mx_repo()
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)

  withr::local_options(usethis.quiet = TRUE)
  use_local_miniextendr(repo, path = pkg)

  gi_lines <- readLines(file.path(pkg, ".gitignore"), warn = FALSE)
  expect_true(".miniextendr-local" %in% gi_lines)

  rbi_lines <- readLines(file.path(pkg, ".Rbuildignore"), warn = FALSE)
  ignored <- function(path) {
    any(vapply(rbi_lines, grepl, logical(1), x = path, perl = TRUE))
  }
  expect_true(ignored(".miniextendr-local"))
  expect_false(ignored("x.miniextendr-local"))
  expect_false(ignored(".miniextendr-local.backup"))
})

test_that("use_local_miniextendr() replaces the malformed build-ignore entry", {
  pkg <- make_mx_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  repo <- make_fake_mx_repo()
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  rbi <- file.path(pkg, ".Rbuildignore")
  writeLines(c("^\\\\.miniextendr-local$$", "^keep-me$"), rbi)

  withr::local_options(usethis.quiet = TRUE)
  use_local_miniextendr(repo, path = pkg)
  use_local_miniextendr(repo, path = pkg)

  expect_equal(readLines(rbi), c("^keep-me$", "^\\.miniextendr-local$"))
})

test_that("use_local_miniextendr() rejects a path without miniextendr-api/Cargo.toml", {
  pkg <- make_mx_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  bad_repo <- tempfile("not-a-repo-")
  dir.create(bad_repo)
  on.exit(unlink(bad_repo, recursive = TRUE), add = TRUE)

  withr::local_options(usethis.quiet = TRUE)
  expect_error(
    use_local_miniextendr(bad_repo, path = pkg),
    "does not look like a miniextendr checkout"
  )
})

test_that("use_local_miniextendr() rejects a non-existent path", {
  pkg <- make_mx_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  withr::local_options(usethis.quiet = TRUE)
  expect_error(
    use_local_miniextendr("/no/such/path/here", path = pkg),
    class = "error"
  )
})

# ---------------------------------------------------------------------------
# unuse_local_miniextendr() — idempotence
# ---------------------------------------------------------------------------

test_that("unuse_local_miniextendr() removes the marker and returns TRUE", {
  pkg <- make_mx_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  repo <- make_fake_mx_repo()
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)

  withr::local_options(usethis.quiet = TRUE)
  use_local_miniextendr(repo, path = pkg)
  expect_true(file.exists(file.path(pkg, ".miniextendr-local")))

  rbi <- file.path(pkg, ".Rbuildignore")
  writeLines(c("^\\\\.miniextendr-local$$", "^keep-me$"), rbi)

  result <- unuse_local_miniextendr(path = pkg)
  expect_true(isTRUE(result))
  expect_false(file.exists(file.path(pkg, ".miniextendr-local")))
  expect_equal(readLines(rbi), "^keep-me$")
})

test_that("unuse_local_miniextendr() is idempotent when no marker exists", {
  pkg <- make_mx_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  rbi <- file.path(pkg, ".Rbuildignore")
  writeLines(c("^\\\\.miniextendr-local$$", "^keep-me$"), rbi)

  withr::local_options(usethis.quiet = TRUE)
  result <- unuse_local_miniextendr(path = pkg)
  expect_true(isFALSE(result))
  expect_equal(readLines(rbi), "^keep-me$")
})

# ---------------------------------------------------------------------------
# Tarball-latch warning
# ---------------------------------------------------------------------------

test_that("use_local_miniextendr() warns when inst/vendor.tar.xz is present", {
  pkg <- make_mx_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  repo <- make_fake_mx_repo()
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)

  # Plant a fake tarball latch.
  inst_dir <- file.path(pkg, "inst")
  dir.create(inst_dir, showWarnings = FALSE)
  writeLines("fake", file.path(inst_dir, "vendor.tar.xz"))

  withr::local_options(usethis.quiet = TRUE)
  expect_message(
    use_local_miniextendr(repo, path = pkg),
    regexp = "tarball mode wins|vendor.tar.xz|latch",
    all = FALSE
  )
})

# ---------------------------------------------------------------------------
# miniextendr_doctor() — marker flag
# ---------------------------------------------------------------------------

test_that("miniextendr_doctor() warns about an active .miniextendr-local marker", {
  pkg <- make_mx_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  # Write a fake marker directly (bypass use_local_miniextendr validation).
  writeLines("/some/fake/local/path", file.path(pkg, ".miniextendr-local"))

  withr::local_options(usethis.quiet = TRUE)
  result <- miniextendr_doctor(path = pkg)
  all_warns <- result$warn
  expect_true(
    any(grepl("miniextendr-local", all_warns, fixed = TRUE)),
    info = paste("warn entries:", paste(all_warns, collapse = ", "))
  )
})

test_that("miniextendr_doctor() reports pass when no .miniextendr-local marker", {
  pkg <- make_mx_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  withr::local_options(usethis.quiet = TRUE)
  result <- miniextendr_doctor(path = pkg)
  all_warns <- result$warn
  expect_false(
    any(grepl("miniextendr-local", all_warns, fixed = TRUE)),
    info = paste("unexpected warn entries:", paste(all_warns, collapse = ", "))
  )
})

test_that("miniextendr_doctor() downgrades to info when tarball also present with marker", {
  pkg <- make_mx_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  # Both marker and tarball present: warn, but message says tarball wins.
  writeLines("/some/fake/local/path", file.path(pkg, ".miniextendr-local"))
  inst_dir <- file.path(pkg, "inst")
  dir.create(inst_dir, showWarnings = FALSE)
  writeLines("fake", file.path(inst_dir, "vendor.tar.xz"))

  withr::local_options(usethis.quiet = TRUE)
  result <- miniextendr_doctor(path = pkg)
  # Still in warn (not fail) when both are present.
  all_warns <- result$warn
  expect_true(
    any(grepl("miniextendr-local", all_warns, fixed = TRUE)),
    info = paste("warn entries:", paste(all_warns, collapse = ", "))
  )
})

test_that("miniextendr_doctor() warns about hand-rolled [patch] block in Cargo.toml", {
  pkg <- make_mx_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  cargo_toml <- file.path(pkg, "src", "rust", "Cargo.toml")
  writeLines(
    c(
      "[package]",
      'name = "testpkg"',
      "",
      "[dependencies]",
      'miniextendr-api = "*"',
      "",
      '[patch."https://github.com/A2-ai/miniextendr"]',
      'miniextendr-api = { path = "/home/dev/miniextendr/miniextendr-api" }'
    ),
    cargo_toml
  )

  withr::local_options(usethis.quiet = TRUE)
  result <- miniextendr_doctor(path = pkg)
  all_warns <- result$warn
  expect_true(
    any(grepl("hand-rolled|use_local_miniextendr", all_warns)),
    info = paste("warn entries:", paste(all_warns, collapse = ", "))
  )
})
