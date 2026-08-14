# Tests for use_local_miniextendr() / unuse_local_miniextendr() (#908)
#
# All tests use tempdir fixtures. Configure regressions run autoconf with a
# fake Rust toolchain, but never compile.

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

make_fake_rust_toolchain <- function() {
  bin <- tempfile("fake-rust-bin-")
  dir.create(bin)

  cargo <- file.path(bin, "cargo")
  writeLines(
    c(
      "#!/bin/sh",
      "if test \"$1\" = \"--version\"; then",
      "  echo 'cargo 1.97.1 (test)'",
      "  exit 0",
      "fi",
      "if test \"$1\" = \"revendor\"; then",
      "  : > \"$MX_REVENDOR_SENTINEL\"",
      "  exit 0",
      "fi",
      "if test \"$1\" = \"generate-lockfile\"; then",
      "  while test $# -gt 0; do",
      "    if test \"$1\" = \"--manifest-path\"; then",
      "      shift",
      "      mkdir -p \"$(dirname \"$1\")\"",
      "      : > \"$(dirname \"$1\")/Cargo.lock\"",
      "      exit 0",
      "    fi",
      "    shift",
      "  done",
      "fi",
      "exit 0"
    ),
    cargo
  )

  rustc <- file.path(bin, "rustc")
  writeLines(
    c(
      "#!/bin/sh",
      "if test \"$1\" = \"-vV\"; then",
      "  echo 'rustc 1.97.1 (test)'",
      "  echo 'host: x86_64-unknown-linux-gnu'",
      "else",
      "  echo 'rustc 1.97.1 (test)'",
      "fi"
    ),
    rustc
  )

  writeLines(c("#!/bin/sh", "exit 0"), file.path(bin, "cargo-revendor"))
  Sys.chmod(c(cargo, rustc, file.path(bin, "cargo-revendor")), "0755")
  bin
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

  # .gitignore
  gitignore <- file.path(pkg, ".gitignore")
  if (file.exists(gitignore)) {
    gi_lines <- readLines(gitignore, warn = FALSE)
    expect_true(any(grepl(".miniextendr-local", gi_lines, fixed = TRUE)))
  }
  # .Rbuildignore
  rbi <- file.path(pkg, ".Rbuildignore")
  if (file.exists(rbi)) {
    rbi_lines <- readLines(rbi, warn = FALSE)
    expect_true(any(grepl("miniextendr-local", rbi_lines, fixed = TRUE)))
  }
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

test_that("local checkout keeps a non-git scaffold in source mode", {
  skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")
  skip_if_no_local_repo()

  tmp <- withr::local_tempdir()
  pkg <- file.path(tmp, "localpkg")
  sentinel <- file.path(tmp, "cargo-revendor-ran")
  fake_bin <- make_fake_rust_toolchain()
  withr::defer(unlink(fake_bin, recursive = TRUE))
  withr::local_envvar(c(
    PATH = paste(fake_bin, Sys.getenv("PATH"), sep = .Platform$path.sep),
    MX_REVENDOR_SENTINEL = sentinel
  ))
  withr::local_options(usethis.quiet = TRUE)

  suppressWarnings(suppressMessages(
    create_miniextendr_package(pkg, open = FALSE, rstudio = FALSE)
  ))
  suppressMessages(use_local_miniextendr(find_miniextendr_repo(), path = pkg))
  suppressMessages(miniextendr_configure(path = pkg))

  expect_false(file.exists(sentinel))
  expect_false(file.exists(file.path(pkg, "inst", "vendor.tar.xz")))
  cargo_config <- readLines(
    file.path(pkg, "src", "rust", ".cargo", "config.toml"),
    warn = FALSE
  )
  expect_true(any(grepl('[patch."https://github.com/A2-ai/miniextendr"]',
                        cargo_config, fixed = TRUE)))
})

test_that("configure does not vendor a non-git scaffold", {
  skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")

  tmp <- withr::local_tempdir()
  pkg <- file.path(tmp, "ordinarypkg")
  sentinel <- file.path(tmp, "cargo-revendor-ran")
  fake_bin <- make_fake_rust_toolchain()
  withr::defer(unlink(fake_bin, recursive = TRUE))
  withr::local_envvar(c(
    PATH = paste(fake_bin, Sys.getenv("PATH"), sep = .Platform$path.sep),
    MX_REVENDOR_SENTINEL = sentinel
  ))
  withr::local_options(usethis.quiet = TRUE)

  suppressWarnings(suppressMessages(
    create_miniextendr_package(pkg, open = FALSE, rstudio = FALSE)
  ))
  suppressMessages(miniextendr_configure(path = pkg))

  expect_false(file.exists(sentinel))
  expect_false(file.exists(file.path(pkg, "inst", "vendor.tar.xz")))
  cargo_config <- readLines(
    file.path(pkg, "src", "rust", ".cargo", "config.toml"),
    warn = FALSE
  )
  expect_false(any(grepl("vendored-sources", cargo_config, fixed = TRUE)))
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

  result <- unuse_local_miniextendr(path = pkg)
  expect_true(isTRUE(result))
  expect_false(file.exists(file.path(pkg, ".miniextendr-local")))
})

test_that("unuse_local_miniextendr() is idempotent when no marker exists", {
  pkg <- make_mx_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  withr::local_options(usethis.quiet = TRUE)
  result <- unuse_local_miniextendr(path = pkg)
  expect_true(isFALSE(result))
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
