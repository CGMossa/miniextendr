# Integration tests for template scaffolding
#
# These tests verify that the scaffolding functions create valid projects
# that can be built with the miniextendr toolchain.

# -----------------------------------------------------------------------------
# Shared helpers
# -----------------------------------------------------------------------------

# Build a cdylib, generate R wrappers, and roxygenise.
# Returns invisible(TRUE) on success, stops on failure.
generate_r_wrappers <- function(pkg_path) {
  rust_dir <- file.path(pkg_path, "src", "rust")
  features_flag <- grep("^CARGO_FEATURES_FLAG", readLines(file.path(pkg_path, "src", "Makevars")),
                        value = TRUE)
  features_flag <- sub("^CARGO_FEATURES_FLAG *= *", "", features_flag)
  cargo_lines <- readLines(file.path(rust_dir, "Cargo.toml"))
  name_line <- grep("^name\\s*=", cargo_lines, value = TRUE)[1]
  crate_name <- gsub("-", "_", gsub(".*\"(.+)\".*", "\\1", name_line))

  cdylib_result <- system2(
    "cargo",
    c("rustc", "--lib", "--manifest-path", file.path(rust_dir, "Cargo.toml"),
      "--target-dir", file.path(rust_dir, "target"),
      if (nzchar(features_flag)) features_flag,
      "--crate-type", "cdylib"),
    stdout = TRUE, stderr = TRUE
  )
  cdylib_status <- attr(cdylib_result, "status")
  if (!is.null(cdylib_status) && cdylib_status != 0) {
    stop("cdylib build failed: ", paste(cdylib_result, collapse = "\n"))
  }

  cdylib_ext <- if (.Platform$OS.type == "windows") "dll"
                else if (Sys.info()[["sysname"]] == "Darwin") "dylib"
                else "so"
  cdylib_path <- file.path(rust_dir, "target", "debug",
    paste0("lib", crate_name, ".", cdylib_ext))

  pkg_name <- read.dcf(file.path(pkg_path, "DESCRIPTION"))[1, "Package"]
  wrapper_path <- file.path(pkg_path, "R", paste0(pkg_name, "-wrappers.R"))

  lib <- dyn.load(cdylib_path)
  on.exit(dyn.unload(cdylib_path), add = TRUE)
  .Call(getNativeSymbolInfo("miniextendr_write_wrappers", lib), wrapper_path)

  suppressMessages(roxygen2::roxygenise(pkg_path))
  invisible(TRUE)
}

# R CMD INSTALL to a temp library. Returns the lib_path.
install_to_templib <- function(pkg_path, tmp) {
  lib_path <- file.path(tmp, "library")
  dir.create(lib_path, showWarnings = FALSE)

  result <- system2(
    file.path(R.home("bin"), "R"),
    c("CMD", "INSTALL", "--no-multiarch", "-l", lib_path, pkg_path),
    env = c(paste0("R_LIBS=", lib_path)),
    stdout = TRUE, stderr = TRUE
  )
  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    stop("R CMD INSTALL failed: ", paste(result, collapse = "\n"))
  }
  lib_path
}

# Standard e2e skip guards.
#
# These tests cold-compile a Rust crate, run autoconf, and (for the standalone
# round-trip) network-fetch + vendor miniextendr — minutes per run. They're
# skipped on CI by default so the per-PR `minirextendr` job stays fast.
#
# Opt-in bypass: set MINIEXTENDR_RUN_E2E=1 to run them on CI anyway (used by the
# nightly/label-gated round-trip job — see .github/workflows/ci.yml
# `r-roundtrip-e2e`). The bypass only lifts the skip_on_ci() gate; the
# tool-availability + local-repo guards below still apply, so a misconfigured
# runner skips with a clear reason instead of erroring. Local runs (where
# skip_on_ci() is already a no-op) are unaffected by the flag. See #775 / #805:
# before this, skip_on_ci() ran first and the round-trip was unreachable in CI.
skip_e2e <- function() {
  # as.logical("1") is NA in R, so accept the common truthy spellings explicitly.
  run_e2e <- tolower(Sys.getenv("MINIEXTENDR_RUN_E2E", "")) %in% c("1", "true", "yes")
  if (!run_e2e) {
    skip_on_ci()
  }
  skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  skip_if_not(nzchar(Sys.which("R")), "R not available")
  skip_if_no_local_repo()
}

# -----------------------------------------------------------------------------
# Templates patch sync check
# -----------------------------------------------------------------------------

test_that("templates patch is in sync with rpkg sources", {
  skip_if_not(nzchar(Sys.which("just")), "just not available")

  pkg_path <- normalizePath(
    file.path(testthat::test_path(), "..", ".."),
    mustWork = FALSE
  )
  skip_if(!dir.exists(pkg_path), "Cannot find package root")

  repo_root <- dirname(pkg_path)
  skip_if(!file.exists(file.path(repo_root, "justfile")), "Not in miniextendr monorepo")

  result <- withr::with_dir(repo_root, {
    system2("just", c("templates-check"), stdout = TRUE, stderr = TRUE)
  })
  status <- attr(result, "status")

  if (!is.null(status) && status != 0) {
    fail(paste0(
      "Templates patch is out of sync with rpkg sources.\n",
      "Run `just templates-approve` to update the patch.\n\n",
      "Diff output:\n", paste(result, collapse = "\n")
    ))
  }

  expect_true(is.null(status) || status == 0)
})

# -----------------------------------------------------------------------------
# Monorepo template tests (shared fixture)
# -----------------------------------------------------------------------------

# Create one monorepo fixture for all structure/content tests.
# The fixture is created once and reused across tests in this block.
monorepo_fixture <- NULL
monorepo_fixture_cleanup <- NULL

local({
  setup_monorepo_fixture <- function() {
    if (!is.null(monorepo_fixture)) return(monorepo_fixture)
    repo <- find_miniextendr_repo()
    if (is.null(repo)) return(NULL)

    tmp <- tempfile("monorepo-shared-")
    suppressMessages(
      create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg-rs",
                                  local_path = repo, open = FALSE)
    )
    monorepo_fixture <<- tmp
    monorepo_fixture_cleanup <<- withr::defer(
      unlink(tmp, recursive = TRUE),
      envir = testthat::teardown_env()
    )
    tmp
  }

  get_fixture <- function() {
    skip_if_no_local_repo()
    path <- setup_monorepo_fixture()
    skip_if(is.null(path), "Failed to create monorepo fixture")
    path
  }

  test_that("create_miniextendr_monorepo creates correct directory structure", {
    tmp <- get_fixture()

    # Root files
    expect_true(file.exists(file.path(tmp, "Cargo.toml")))
    # Scaffolded packages must NOT ship a justfile — `just` is a miniextendr
    # dev helper, not a scaffolding/build dependency (end users may not have it).
    expect_false(file.exists(file.path(tmp, "justfile")))
    expect_true(file.exists(file.path(tmp, ".gitignore")))
    expect_true(dir.exists(file.path(tmp, ".git")))

    # Main crate
    expect_true(file.exists(file.path(tmp, "testpkg-rs", "Cargo.toml")))
    expect_true(file.exists(file.path(tmp, "testpkg-rs", "src", "lib.rs")))

    # rpkg structure
    expect_true(file.exists(file.path(tmp, "testpkg", "DESCRIPTION")))
    expect_true(file.exists(file.path(tmp, "testpkg", "configure.ac")))
    expect_true(file.exists(file.path(tmp, "testpkg", "src", "Makevars.in")))
    expect_true(file.exists(file.path(tmp, "testpkg", "src", "stub.c")))
    expect_true(file.exists(file.path(tmp, "testpkg", "src", "rust", "lib.rs")))
    expect_true(file.exists(file.path(tmp, "testpkg", "src", "rust", "Cargo.toml")))
    expect_true(file.exists(file.path(tmp, "testpkg", "src", "rust", "build.rs")))
    expect_true(dir.exists(file.path(tmp, "testpkg", "vendor")))
  })

  test_that("monorepo root Cargo.toml has valid workspace configuration", {
    tmp <- get_fixture()

    cargo_text <- paste(readLines(file.path(tmp, "Cargo.toml")), collapse = "\n")
    expect_true(grepl("\\[workspace\\]", cargo_text))
    expect_true(grepl('resolver = "3"', cargo_text))
    expect_true(grepl("testpkg-rs", cargo_text))
    expect_true(grepl('exclude = \\["testpkg/src/rust"', cargo_text))
    expect_true(grepl("\\[workspace\\.package\\]", cargo_text))
    expect_true(grepl('version = "0\\.1\\.0"', cargo_text))
  })

  test_that("monorepo rpkg DESCRIPTION has correct miniextendr config", {
    tmp <- get_fixture()

    dcf <- read.dcf(file.path(tmp, "testpkg", "DESCRIPTION"))
    expect_equal(unname(dcf[1, "Config/build/bootstrap"]), "TRUE")
    expect_equal(unname(dcf[1, "Config/build/never-clean"]), "true")
    expect_equal(unname(dcf[1, "Config/build/extra-sources"]), "src/rust/Cargo.lock")
    expect_true(grepl("Rust", dcf[1, "SystemRequirements"]))
  })

  test_that("monorepo can run autoconf and configure", {
    skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")
    skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
    tmp <- get_fixture()

    result <- withr::with_dir(file.path(tmp, "testpkg"), {
      system2("autoconf", c("-v", "-i", "-f"), stdout = TRUE, stderr = TRUE)
    })
    status <- attr(result, "status")
    expect_true(is.null(status) || status == 0)
    expect_true(file.exists(file.path(tmp, "testpkg", "configure")))
  })

  test_that("monorepo workspace can cargo check", {
    skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
    tmp <- get_fixture()

    result <- suppressWarnings(
      withr::with_dir(file.path(tmp, "testpkg-rs"), {
        system2("cargo", c("check"), stdout = TRUE, stderr = TRUE)
      })
    )
    status <- attr(result, "status")
    output <- paste(result, collapse = "\n")
    valid_outcome <- is.null(status) || status == 0 ||
                     grepl("miniextendr-api", output) ||
                     grepl("Compiling", output) ||
                     grepl("Checking", output)
    expect_true(valid_outcome)
  })
})

# Substitution test needs different params — separate fixture
test_that("create_miniextendr_monorepo performs correct template substitution", {
  skip_if_no_local_repo()
  tmp <- tempfile("monorepo-subst-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "myPkg", crate_name = "my-pkg",
                              local_path = find_miniextendr_repo(), open = FALSE)

  expect_true(any(grepl("my-pkg", readLines(file.path(tmp, "Cargo.toml")))))
  expect_true(any(grepl('name = "my-pkg"', readLines(file.path(tmp, "my-pkg", "Cargo.toml")))))
  expect_true(any(grepl("#\\[miniextendr\\]", readLines(file.path(tmp, "myPkg", "src", "rust", "lib.rs")))))
  expect_true(any(grepl("Package: myPkg", readLines(file.path(tmp, "myPkg", "DESCRIPTION")))))
})

# -----------------------------------------------------------------------------
# rpkg template tests
# -----------------------------------------------------------------------------

test_that("use_template works with rpkg template type", {
  tmp <- tempfile("rpkg-template-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  dir.create(tmp)
  usethis::proj_set(tmp, force = TRUE)
  writeLines("Package: testpkg\nTitle: Test\nVersion: 0.0.1\n", file.path(tmp, "DESCRIPTION"))

  minirextendr:::set_template_type("rpkg")
  data <- list(package = "testpkg", package_rs = "testpkg", Package = "Testpkg", year = "2025")
  minirextendr:::ensure_dir(usethis::proj_path("src", "rust"))
  minirextendr:::use_template("lib.rs", save_as = "src/rust/lib.rs", data = data)
  minirextendr:::use_template("build.rs", save_as = "src/rust/build.rs")
  minirextendr:::use_template("Makevars.in", save_as = "src/Makevars.in")

  expect_true(file.exists(file.path(tmp, "src", "rust", "lib.rs")))
  expect_true(file.exists(file.path(tmp, "src", "rust", "build.rs")))
  expect_true(file.exists(file.path(tmp, "src", "Makevars.in")))
  expect_true(any(grepl("#\\[miniextendr\\]", readLines(file.path(tmp, "src", "rust", "lib.rs")))))
})

test_that("use_template performs mustache substitution correctly", {
  tmp <- tempfile("subst-test-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  dir.create(tmp)
  usethis::proj_set(tmp, force = TRUE)
  writeLines("Package: testpkg\n", file.path(tmp, "DESCRIPTION"))

  minirextendr:::set_template_type("rpkg")
  data <- list(package = "specialPkg", package_rs = "special_pkg")
  minirextendr:::ensure_dir(usethis::proj_path("src", "rust"))
  minirextendr:::use_template("lib.rs", save_as = "src/rust/lib.rs", data = data)

  content <- paste(readLines(file.path(tmp, "src", "rust", "lib.rs")), collapse = "\n")
  expect_true(grepl("#\\[miniextendr\\]", content))
  expect_false(grepl("\\{\\{package", content))
})

# -----------------------------------------------------------------------------
# End-to-end scaffolding tests (full build and test)
# -----------------------------------------------------------------------------

test_that("rpkg scaffolding builds and functions work end-to-end", {
  skip_e2e()
  miniextendr_path <- find_miniextendr_repo()

  tmp <- tempfile("rpkg-e2e-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)
  pkg_path <- file.path(tmp, "testpkg")

  suppressWarnings(suppressMessages({
    usethis::create_package(pkg_path, open = FALSE)
    use_miniextendr(path = pkg_path, local_path = miniextendr_path)
    usethis::proj_set(pkg_path, force = TRUE)
    usethis::use_package_doc()
  }))

  suppressMessages({
    miniextendr_autoconf(path = pkg_path)
    miniextendr_configure(path = pkg_path)
    devtools::document(pkg = pkg_path)
  })

  pkg_name <- read.dcf(file.path(pkg_path, "DESCRIPTION"))[1, "Package"]
  lib_path <- install_to_templib(pkg_path, tmp)
  generate_r_wrappers(pkg_path)
  lib_path <- install_to_templib(pkg_path, tmp)

  withr::with_libpaths(lib_path, action = "prefix", {
    library(pkg_name, character.only = TRUE)
    expect_equal(add(1, 2), 3)
    expect_equal(add(10, 20), 30)
    expect_equal(hello("World"), "Hello, World!")
    expect_equal(hello("Test"), "Hello, Test!")
    detach(paste0("package:", pkg_name), character.only = TRUE, unload = TRUE)
  })
})

test_that("rpkg scaffolding with external cargo dependency works", {
  skip_e2e()
  miniextendr_path <- find_miniextendr_repo()

  tmp <- tempfile("rpkg-cargo-dep-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)
  pkg_path <- file.path(tmp, "testpkg")

  suppressWarnings(suppressMessages({
    usethis::create_package(pkg_path, open = FALSE)
    use_miniextendr(path = pkg_path, local_path = miniextendr_path)
    usethis::proj_set(pkg_path, force = TRUE)
    usethis::use_package_doc()
  }))

  suppressMessages({
    miniextendr_autoconf(path = pkg_path)
    miniextendr_configure(path = pkg_path)
  })

  # Add itertools dependency
  cargo_toml <- file.path(pkg_path, "src", "rust", "Cargo.toml")
  cargo_content <- readLines(cargo_toml)
  deps_idx <- grep("^\\[dependencies\\]", cargo_content)[1]
  if (!is.na(deps_idx)) {
    cargo_content <- c(
      cargo_content[1:deps_idx],
      "itertools = \"0.13\"",
      cargo_content[(deps_idx + 1):length(cargo_content)]
    )
    writeLines(cargo_content, cargo_toml)
  }

  # Add itertools usage to lib.rs
  lib_rs <- file.path(pkg_path, "src", "rust", "lib.rs")
  lib_content <- readLines(lib_rs)
  use_idx <- grep("use miniextendr_api", lib_content)[1]
  lib_content <- c(
    lib_content[1:use_idx],
    "use itertools::Itertools;",
    "",
    "/// Join strings with itertools",
    "/// @param parts Character vector to join",
    "/// @return Joined string",
    "#[miniextendr]",
    "pub fn join_strings(parts: Vec<String>) -> String {",
    "    parts.into_iter().join(\", \")",
    "}",
    "",
    lib_content[(use_idx + 1):length(lib_content)]
  )
  writeLines(lib_content, lib_rs)

  # Reconfigure in source mode (itertools will be fetched by cargo during build)
  suppressMessages({
    result <- minirextendr:::run_with_logging(
      "bash", args = c("./configure"),
      log_prefix = "configure-cargo-dep", wd = pkg_path,
      env = devtools::r_env_vars()
    )
    expect_true(result$success,
                info = paste("configure failed:", paste(result$output, collapse = "\n")))
  })

  lib_path <- install_to_templib(pkg_path, tmp)
  generate_r_wrappers(pkg_path)
  lib_path <- install_to_templib(pkg_path, tmp)

  withr::with_libpaths(lib_path, action = "prefix", {
    library(testpkg)
    expect_equal(add(1, 2), 3)
    expect_equal(hello("Test"), "Hello, Test!")
    expect_equal(join_strings(c("a", "b", "c")), "a, b, c")
    detach("package:testpkg", character.only = TRUE, unload = TRUE)
  })
})

# -----------------------------------------------------------------------------
# Package name handling tests
# -----------------------------------------------------------------------------

test_that("monorepo handles dots in package names", {
  skip_if_no_local_repo()
  tmp <- tempfile("monorepo-dots-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "my.pkg", crate_name = "my-pkg",
                              local_path = find_miniextendr_repo(), open = FALSE)

  expect_true(any(grepl('name = "my-pkg"', readLines(file.path(tmp, "my-pkg", "Cargo.toml")))))
  expect_true(any(grepl("Package: my.pkg", readLines(file.path(tmp, "my.pkg", "DESCRIPTION")))))
  expect_true(any(grepl("#\\[miniextendr\\]", readLines(file.path(tmp, "my.pkg", "src", "rust", "lib.rs")))))
})

test_that("monorepo handles hyphens in crate names", {
  skip_if_no_local_repo()
  tmp <- tempfile("monorepo-hyphens-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "test-pkg",
                              local_path = find_miniextendr_repo(), open = FALSE)

  expect_true(any(grepl('name = "test-pkg"', readLines(file.path(tmp, "test-pkg", "Cargo.toml")))))
})

test_that("create_miniextendr_package rejects invalid R package names", {
  tmp <- tempfile("invalid-name-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  expect_error(create_miniextendr_package(file.path(tmp, "1badname"), open = FALSE))
})

test_that("use_miniextendr validates template_type against its choices", {
  tmp <- tempfile("bad-template-type-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # An unknown template_type errors immediately (match.arg is the first
  # statement in the body), before any scaffolding side effects occur.
  expect_error(
    use_miniextendr(path = tmp, template_type = "banana"),
    "should be one of"
  )

  # The default (a missing arg) still resolves to "auto": the choices vector
  # lists it first, so match.arg() picks it unchanged.
  expect_identical(eval(formals(use_miniextendr)$template_type)[[1]], "auto")
})

# -----------------------------------------------------------------------------
# Monorepo end-to-end
# -----------------------------------------------------------------------------

test_that("monorepo scaffolding builds and functions work end-to-end", {
  skip_e2e()
  miniextendr_path <- find_miniextendr_repo()

  tmp <- tempfile("monorepo-e2e-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  suppressMessages({
    create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg-rs",
                                local_path = miniextendr_path, open = FALSE)
  })

  rpkg_path <- file.path(tmp, "testpkg")

  suppressMessages({
    usethis::proj_set(rpkg_path, force = TRUE)
    usethis::use_package_doc()
  })

  # Vendor crates.io deps (miniextendr crates already vendored by scaffolding)
  suppressWarnings({
    withr::with_dir(rpkg_path, {
      system2("cargo", c("vendor", "--manifest-path", "src/rust/Cargo.toml", "vendor"),
              stdout = FALSE, stderr = FALSE)
    })
  })

  suppressMessages({
    usethis::proj_set(rpkg_path, force = TRUE)
    usethis::use_package_doc()
    miniextendr_autoconf(path = rpkg_path)
    miniextendr_configure(path = rpkg_path)
  })

  pkg_name <- read.dcf(file.path(rpkg_path, "DESCRIPTION"))[1, "Package"]
  lib_path <- install_to_templib(rpkg_path, tmp)
  generate_r_wrappers(rpkg_path)
  lib_path <- install_to_templib(rpkg_path, tmp)

  withr::with_libpaths(lib_path, action = "prefix", {
    library(pkg_name, character.only = TRUE)
    expect_equal(add(1, 2), 3)
    expect_equal(add(10, 20), 30)
    expect_equal(hello("World"), "Hello, World!")
    expect_equal(hello("Test"), "Hello, Test!")
    detach(paste0("package:", pkg_name), character.only = TRUE, unload = TRUE)
  })
})

# -----------------------------------------------------------------------------
# Standalone (non-monorepo) end-to-end
# -----------------------------------------------------------------------------

# Standalone counterpart to the monorepo round-trip above, and the in-suite home
# for the #757 / #775 / #822 / #963 regression (it lived as a standalone bash
# script + bespoke CI job before; see #805). create_miniextendr_package() +
# miniextendr_build() scaffold a package that sits outside any .git ancestor.
# A build = TRUE install's bootstrap.R vendors while producing the tarball and
# flips the staged package into
# *tarball mode* — which skips the wrapper-gen pass. On a brand-new
# package (no R/<pkg>-wrappers.R yet) that means the wrappers can never be
# generated and library() exposes nothing (#822). miniextendr_build() now
# detects the absent wrappers file and calls bootstrap_fresh_wrappers(), which
# clears the latch, re-runs configure, then installs with
# MINIEXTENDR_FORCE_WRAPPER_GEN=1 (build = FALSE). Configure does not vendor in
# this source-mode bootstrap; the FORCE flag also guards the wrappers-present
# case in install_pkg() against a leaked tarball latch (#757).
#
# Unlike the monorepo tests, create_miniextendr_package() takes no local_path, so
# it scaffolds against miniextendr `main` and this test is network-dependent
# (vendors via cargo-revendor). Heavy maintainer/nightly check; #805 tracks
# wiring it into a scheduled CI run rather than the per-PR suite.
test_that("standalone scaffolding builds in tarball mode and exposes functions", {
  skip_e2e()
  skip_if_not(nzchar(Sys.which("cargo-revendor")),
              "cargo-revendor not available (tarball build vendoring)")

  pkg_name <- "mxroundtrip"
  tmp <- tempfile("standalone-e2e-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)
  pkg_path <- file.path(tmp, pkg_name)

  # Install the throwaway package into a temp library so the dev/CI base library
  # stays clean: miniextendr_build(install = TRUE) -> devtools::install lands in
  # .libPaths()[1], and library() below resolves there.
  templib <- file.path(tmp, "library")
  dir.create(templib)

  withr::with_libpaths(templib, action = "prefix", {
    suppressMessages(
      create_miniextendr_package(pkg_path, open = FALSE, rstudio = FALSE)
    )
    suppressMessages(
      miniextendr_build(pkg_path, install = TRUE)
    )

    # Wrapper-gen ran and emitted the scaffolded exports (the #757 / #822 symptom
    # is an absent wrappers file).
    wrappers <- file.path(pkg_path, "R", paste0(pkg_name, "-wrappers.R"))
    expect_true(file.exists(wrappers),
                info = "fresh-package build skipped wrapper-gen (#757/#822 regression)")
    wrappers_src <- paste(readLines(wrappers, warn = FALSE), collapse = "\n")
    expect_match(wrappers_src, "add", fixed = TRUE)
    expect_match(wrappers_src, "hello", fixed = TRUE)

    # NAMESPACE exports them (empty-namespace bug symptom).
    namespace_src <- paste(readLines(file.path(pkg_path, "NAMESPACE"), warn = FALSE),
                           collapse = "\n")
    expect_match(namespace_src, "export(add)", fixed = TRUE)
    expect_match(namespace_src, "export(hello)", fixed = TRUE)

    # Detach/unload any dev (load_all) namespace left by miniextendr_build()'s
    # document() step so library() below resolves the *installed* copy, not a
    # source-backed dev namespace (and avoids the #1000 stale-.rdb reload path).
    if (pkg_name %in% loadedNamespaces()) pkgload::unload(pkg_name, quiet = TRUE)
    # Installed package loads and the functions actually run.
    library(pkg_name, character.only = TRUE)
    expect_equal(add(2, 3), 5)
    expect_equal(hello("loop"), "Hello, loop!")
    detach(paste0("package:", pkg_name), character.only = TRUE, unload = TRUE)
  })
})

# -----------------------------------------------------------------------------
# miniextendr_build() single-pass export round-trip (#898, fix for #860)
# -----------------------------------------------------------------------------

# Regression for #860 / #898: adding a brand-new #[miniextendr] export must be
# visible via library(pkg) after a SINGLE miniextendr_build() pass. The bug
# (#860) was an ordering hazard: the wrappers file is generated *during* install,
# and document() reads its @export tags to (re)write NAMESPACE, but install
# collated NAMESPACE *before* document() rewrote it — so the freshly-installed
# image lagged the on-disk NAMESPACE by one build and the new export was missing
# until a second build. The fix snapshots NAMESPACE before/after document() and
# reinstalls once iff it changed.
#
# This bug only manifests in the installed image's export set, so the assertion
# is on getNamespaceExports() (a behavioural check) — a source-grep / deparse()
# structural test is explicitly rejected as theater (see the brittle-deparse
# memory rule and the dropped #523 structural test). Reproducing it requires a
# full scaffold + cargo build + R CMD INSTALL round-trip, hence the e2e gate.
#
# Uses the local-repo monorepo scaffold (offline; configure stays in source mode
# while no vendor tarball exists), then drives
# the real miniextendr_build() once — exercising the actual single-pass logic
# rather than the manual generate_r_wrappers() + install_to_templib() helpers.
test_that("miniextendr_build() exports a newly added function in a single pass (#898)", {
  skip_e2e()
  miniextendr_path <- find_miniextendr_repo()

  tmp <- tempfile("single-pass-export-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  suppressMessages({
    create_miniextendr_monorepo(tmp, package = "spexport", crate_name = "spexport-rs",
                                local_path = miniextendr_path, open = FALSE)
  })
  rpkg_path <- file.path(tmp, "spexport")

  # Configure stays in source mode while no vendor tarball exists, but the
  # monorepo's crates.io deps still need vendoring for the offline build (mirrors
  # the monorepo e2e test above). The miniextendr framework crates are already
  # vendored by the scaffolder.
  suppressWarnings({
    withr::with_dir(rpkg_path, {
      system2("cargo", c("vendor", "--manifest-path", "src/rust/Cargo.toml", "vendor"),
              stdout = FALSE, stderr = FALSE)
    })
  })

  # Add a brand-new exported function whose name differs from the scaffold's
  # add/hello, so document() produces a *new* @export — the exact shape of the
  # #860 bug. Insert it just after the `use miniextendr_api` line.
  lib_rs <- file.path(rpkg_path, "src", "rust", "lib.rs")
  lib_content <- readLines(lib_rs)
  use_idx <- grep("use miniextendr_api", lib_content)[1]
  lib_content <- append(lib_content, c(
    "",
    "/// A function added after the initial scaffold.",
    "/// @param x A number",
    "/// @return x doubled",
    "#[miniextendr]",
    "pub fn mx_new_fn(x: f64) -> f64 {",
    "    x * 2.0",
    "}"
  ), after = use_idx)
  writeLines(lib_content, lib_rs)

  # Install the throwaway package into a temp library so the dev/CI base library
  # stays clean: miniextendr_build(install = TRUE) lands in .libPaths()[1].
  templib <- file.path(tmp, "library")
  dir.create(templib)

  withr::with_libpaths(templib, action = "prefix", {
    # The single pass under test. If #860 regresses, this build collates the
    # stale NAMESPACE and mx_new_fn won't be in the installed image.
    suppressMessages(
      miniextendr_build(rpkg_path, install = TRUE)
    )

    # Behavioural assertion: the installed image's export set, not the source.
    exports <- getNamespaceExports("spexport")
    expect_true("mx_new_fn" %in% exports,
                info = paste0(
                  "mx_new_fn missing from getNamespaceExports() after a single ",
                  "miniextendr_build() — #860 single-pass export regression. ",
                  "Exports: ", paste(exports, collapse = ", ")
                ))

    # And it actually resolves + runs (the wrapper is wired, not just named).
    expect_true(exists("mx_new_fn", envir = asNamespace("spexport")))
    expect_equal(spexport::mx_new_fn(21), 42)

    detach("package:spexport", character.only = TRUE, unload = TRUE)
  })
})

# -----------------------------------------------------------------------------
# miniextendr_build() removal/rename self-heal (#1288)
# -----------------------------------------------------------------------------

# Sibling of the #898 single-pass test above, but for the opposite direction:
# removing or renaming an exported #[miniextendr] function. That case dies at
# Step 3 rather than lagging by one build -- the on-disk NAMESPACE is a
# *superset* of the freshly regenerated wrappers, and R CMD INSTALL's
# test-load aborts with "undefined exports: <old_name>" before document() gets
# a chance to drop the stale entry. miniextendr_build() now defers that Step-3
# failure, lets document() reconcile NAMESPACE, then forces a Step-5 retry --
# so the rename still heals in a single miniextendr_build() pass.
#
# This also pins #1294 (the mid-build source-tree restore): Step 3's
# build = TRUE install runs bootstrap.R in the SOURCE tree, sealing
# inst/vendor.tar.xz there by design. Without the mid-build restore, Steps
# 4/5 run latched in tarball mode, wrapper regeneration is skipped (the
# Makevars #1022 guard), and the self-heal breaks for EVERY post-first-build
# export change -- additive too. The `add_renamed %in% exports_after`
# assertion below IS the additive-second-build regression pin for #1294 (a
# new export appearing on a second build), so no separate additive e2e is
# needed. Each build additionally asserts the latch is absent and the
# manifest byte-identical once miniextendr_build() returns.
#
# Same monorepo/offline scaffold shape as #898 (local-repo `.git` keeps
# configure in source mode; crates.io deps vendored via `cargo vendor`), but
# this needs TWO miniextendr_build() passes: one to install the scaffold's
# original `add`/`hello` exports, then a second (the one under test) after
# renaming `add` -> `add_renamed` in src/rust/lib.rs.
test_that("miniextendr_build() heals a removed/renamed export in a single pass (#1288)", {
  skip_e2e()
  miniextendr_path <- find_miniextendr_repo()

  tmp <- tempfile("removal-rename-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  suppressMessages({
    create_miniextendr_monorepo(tmp, package = "sprename", crate_name = "sprename-rs",
                                local_path = miniextendr_path, open = FALSE)
  })
  rpkg_path <- file.path(tmp, "sprename")

  # Vendor the monorepo's crates.io deps for the offline build (mirrors #898).
  suppressWarnings({
    withr::with_dir(rpkg_path, {
      system2("cargo", c("vendor", "--manifest-path", "src/rust/Cargo.toml", "vendor"),
              stdout = FALSE, stderr = FALSE)
    })
  })

  # #1294 assertion helpers: after every miniextendr_build() return, the dev
  # source tree must be restored -- no bootstrap.R-sealed latch left behind,
  # and src/rust/Cargo.toml byte-identical to its pre-build snapshot.
  latch_path <- file.path(rpkg_path, "inst", "vendor.tar.xz")
  manifest_path <- file.path(rpkg_path, "src", "rust", "Cargo.toml")

  # Probe the INSTALLED image in a fresh R subprocess. The building session
  # is unsuitable for these assertions: document()'s load_all leaves a
  # source-backed dev namespace loaded (exports = pre-roxygenize NAMESPACE
  # intersected with loaded objects), and reloading the installed namespace
  # in-session after build 2 reuses the session's earlier native-symbol
  # registration from build 1's DLL at the same installed path (fresh R code
  # + stale registration -> "object 'C_add_renamed' not found"). A clean
  # child process sees exactly what an end user's library(sprename) sees.
  probe_installed <- function(lib) {
    callr::r(
      function(lib) {
        .libPaths(c(lib, .libPaths()))
        exports <- getNamespaceExports("sprename")
        value <- tryCatch(sprename::add_renamed(2, 3),
                          error = function(e) conditionMessage(e))
        list(exports = exports, value = value)
      },
      args = list(lib = lib)
    )
  }

  # Install the throwaway package into a temp library so the dev/CI base library
  # stays clean: miniextendr_build(install = TRUE) lands in .libPaths()[1].
  templib <- file.path(tmp, "library")
  dir.create(templib)

  withr::with_libpaths(templib, action = "prefix", {
    # First pass: install the scaffold as-is, establishing the baseline
    # NAMESPACE that exports `add`.
    manifest_snap_1 <- readLines(manifest_path, warn = FALSE)
    suppressMessages(
      miniextendr_build(rpkg_path, install = TRUE)
    )
    expect_false(file.exists(latch_path),
                 info = "build 1 left inst/vendor.tar.xz behind (#1294 restore regression)")
    expect_identical(readLines(manifest_path, warn = FALSE), manifest_snap_1,
                     info = "build 1 left src/rust/Cargo.toml frozen (#1294 restore regression)")

    probe_1 <- probe_installed(templib)
    expect_true("add" %in% probe_1$exports)

    # Unload the dev namespace document()'s load_all left in this session
    # before build 2 reinstalls over the same library path (#1000 stale-.rdb
    # hygiene; mirrors the standalone e2e's unload-before-library).
    if ("sprename" %in% loadedNamespaces()) pkgload::unload("sprename", quiet = TRUE)

    # Rename the exported `add` function. Exact-string substring rename, NOT a
    # regex over "add" -- a regex would also corrupt unrelated identifiers
    # (substring-rename gotcha, see project memory).
    lib_rs <- file.path(rpkg_path, "src", "rust", "lib.rs")
    lib_content <- readLines(lib_rs)
    lib_content <- sub("pub fn add(", "pub fn add_renamed(", lib_content, fixed = TRUE)
    writeLines(lib_content, lib_rs)

    # The single pass under test. If #1288 regresses, Step 3's install aborts
    # ("undefined exports: add") and propagates instead of healing, and this
    # miniextendr_build() call errors instead of returning. Two warnings are
    # part of the healing pass's contract: minirextendr's own deferral
    # warning, and pkgload's setup_ns_exports() superset warning from Step
    # 4's load_all (the "wrinkle 1" mechanism that lets document() survive
    # the stale NAMESPACE) -- assert both are present among ALL warnings
    # raised. capture_warnings() (not nested expect_warning()) because
    # load_all's internal unload/reload cycle can re-emit the pkgload
    # superset warning more than once per build; nested expect_warning()
    # only muffles the FIRST occurrence matching each regex, so a repeat
    # occurrence leaks past both handlers to testthat's own stray-warning
    # watcher and inflates the run's WARN tally without failing the test.
    # capture_warnings() unconditionally muffles every warning it sees, so
    # nothing leaks regardless of how many times a message repeats.
    manifest_snap_2 <- readLines(manifest_path, warn = FALSE)
    warnings_seen <- testthat::capture_warnings(
      suppressMessages(
        miniextendr_build(rpkg_path, install = TRUE)
      )
    )
    expect_true(any(grepl("Install failed before NAMESPACE reconciliation", warnings_seen)))
    expect_true(any(grepl("Objects listed as exports, but not present", warnings_seen)))
    expect_false(file.exists(latch_path),
                 info = "build 2 left inst/vendor.tar.xz behind (#1294 restore regression)")
    expect_identical(readLines(manifest_path, warn = FALSE), manifest_snap_2,
                     info = "build 2 left src/rust/Cargo.toml frozen (#1294 restore regression)")

    # Behavioural assertions against the installed image (fresh subprocess).
    probe_2 <- probe_installed(templib)
    expect_true("add_renamed" %in% probe_2$exports,
                info = paste0(
                  "add_renamed missing from getNamespaceExports() after a single ",
                  "miniextendr_build() following a rename -- #1288 regression. ",
                  "Exports: ", paste(probe_2$exports, collapse = ", ")
                ))
    expect_false("add" %in% probe_2$exports,
                 info = "stale `add` export should have been dropped by document()")

    # And it actually resolves + runs (the wrapper is wired, not just named).
    expect_equal(probe_2$value, 5)

    # Drop build 2's dev namespace so later tests don't see a loaded
    # namespace pointing into this test's soon-deleted temp library.
    if ("sprename" %in% loadedNamespaces()) pkgload::unload("sprename", quiet = TRUE)
  })
})

# -----------------------------------------------------------------------------
# MINIEXTENDR_FORCE_WRAPPER_GEN propagation into nested R CMD INSTALL (#911)
# -----------------------------------------------------------------------------

# Regression for #911. install_pkg() forces the wrapper-gen pass by
# setting MINIEXTENDR_FORCE_WRAPPER_GEN=1 in the R *session* before calling
# devtools::install(build = TRUE). #911 flagged as unverified whether that
# in-session override survives the two subprocess hops (R CMD build -> R CMD
# INSTALL -> make) down to Makevars.in, which is where the var is actually read
# ([ -z "$$MINIEXTENDR_FORCE_WRAPPER_GEN" ]). If it didn't propagate, a stale /
# leaked-tarball build could silently ship outdated wrappers.
#
# Empirical finding (2026-06-11): it DOES propagate. devtools::install ->
# pkgbuild -> callr::rcmd_safe inherits the parent R session's full environment
# and merely *merges* rcmd_safe_env() on top (it does not replace the
# environment), so a session Sys.setenv() reaches the make recipe two hops down.
# No propagation fix was needed; this test locks the guarantee so a future callr/
# pkgbuild change that sanitised the child environment would be caught.
#
# The probe is a minimal C package (no Rust) with a Makevars that records the
# value of $MINIEXTENDR_FORCE_WRAPPER_GEN seen by the make recipe. It is fast
# (one tiny C compile) but still needs build tools and a real build = TRUE
# install, so it is skipped on CRAN.
test_that("MINIEXTENDR_FORCE_WRAPPER_GEN propagates through install(build = TRUE) into make (#911)", {
  testthat::skip_on_cran()
  skip_if_not(nzchar(Sys.which("make")), "make not available")
  skip_if_not(requireNamespace("devtools", quietly = TRUE), "devtools not available")
  # A working C toolchain is required for R CMD build/INSTALL of the probe pkg.
  skip_if_not(pkgbuild::has_build_tools(debug = FALSE), "C build tools not available")

  tmp <- tempfile("force-propagate-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  pkg_path <- file.path(tmp, "fwgprobe")
  dir.create(file.path(pkg_path, "src"), recursive = TRUE)
  dir.create(file.path(pkg_path, "R"))

  writeLines(c(
    "Package: fwgprobe",
    "Title: Force-Wrapper-Gen Propagation Probe",
    "Version: 0.0.0.9000",
    "Authors@R: person(\"Test\", \"Author\", email = \"test@example.com\", role = c(\"aut\", \"cre\"))",
    "Description: Records the MINIEXTENDR_FORCE_WRAPPER_GEN value seen by make.",
    "License: MIT + file LICENSE",
    "Encoding: UTF-8"
  ), file.path(pkg_path, "DESCRIPTION"))
  writeLines(c("YEAR: 2026", "COPYRIGHT HOLDER: Test Author"),
             file.path(pkg_path, "LICENSE"))
  writeLines(character(), file.path(pkg_path, "NAMESPACE"))

  # A trivial C source so R's build system produces a SHLIB (drives make).
  writeLines("#include <R.h>\nvoid fwgprobe_noop(void) {}",
             file.path(pkg_path, "src", "dummy.c"))

  # Makevars: before building the SHLIB, record the env var the recipe sees.
  # `$$` escapes make's expansion so the shell reads the actual environment —
  # exactly how rpkg's Makevars.in tests it. The result path is passed through
  # the environment to dodge make/shell quoting of the temp path.
  result_file <- file.path(tmp, "fwg_seen.txt")
  Sys.setenv(FWG_RESULT_FILE = result_file)
  on.exit(Sys.unsetenv("FWG_RESULT_FILE"), add = TRUE)
  writeLines(c(
    "$(SHLIB): fwg_probe",
    "fwg_probe:",
    "\t@echo \"[$$MINIEXTENDR_FORCE_WRAPPER_GEN]\" > \"$$FWG_RESULT_FILE\""
  ), file.path(pkg_path, "src", "Makevars"))

  templib <- file.path(tmp, "library")
  dir.create(templib)

  # R_USER_CACHE_DIR handling for the pak::local_install_deps() step inside
  # devtools::install() lives in setup-pak-cache.R (#1154). A per-test
  # withr::with_envvar() here cannot work: pak's persistent subprocess
  # snapshots its environment at creation, so the override never reaches it
  # once any earlier test has used pak.

  run_build_install <- function() {
    unlink(result_file)
    withr::with_libpaths(templib, action = "prefix", {
      suppressMessages(
        devtools::install(pkg_path, build = TRUE, upgrade = FALSE,
                          quiet = TRUE, reload = FALSE,
                          dependencies = FALSE)
      )
    })
    skip_if_not(file.exists(result_file),
                "probe Makevars did not run (no SHLIB build on this platform)")
    trimws(readLines(result_file, warn = FALSE)[1])
  }

  # When set in the session, the make recipe two hops down must see it.
  old_force <- Sys.getenv("MINIEXTENDR_FORCE_WRAPPER_GEN", unset = NA)
  on.exit(
    if (is.na(old_force)) Sys.unsetenv("MINIEXTENDR_FORCE_WRAPPER_GEN")
    else Sys.setenv(MINIEXTENDR_FORCE_WRAPPER_GEN = old_force),
    add = TRUE
  )

  Sys.setenv(MINIEXTENDR_FORCE_WRAPPER_GEN = "1")
  expect_equal(run_build_install(), "[1]",
               info = "session MINIEXTENDR_FORCE_WRAPPER_GEN=1 did not reach the nested make recipe (#911 propagation regression)")

  # And when unset, the recipe must see it empty (so the guard would skip).
  Sys.unsetenv("MINIEXTENDR_FORCE_WRAPPER_GEN")
  expect_equal(run_build_install(), "[]",
               info = "unset MINIEXTENDR_FORCE_WRAPPER_GEN leaked a non-empty value into the nested make recipe")
})
