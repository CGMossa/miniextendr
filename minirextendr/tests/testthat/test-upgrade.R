# Tests for upgrade functionality

test_that("upgrade_gitignore removes obsolete entries", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)

  # Create a .gitignore with some current and obsolete entries
  gitignore <- file.path(tmp, ".gitignore")
  writeLines(c(
    "src/rust/target/",
    "src/entrypoint.c",
    "src/mx_abi.c",
    "src/rust/document.rs",
    ".cargo/config.toml",
    "vendor/"
  ), gitignore)

  # Mock the gitignore template lookup to avoid package dependency
  local_mocked_bindings(
    use_miniextendr_gitignore = function(...) invisible(),
    .package = "minirextendr"
  )

  minirextendr:::upgrade_gitignore()

  result <- readLines(gitignore)
  expect_true("src/rust/target/" %in% result)
  expect_true("vendor/" %in% result)
  expect_false("src/entrypoint.c" %in% result)
  expect_false("src/mx_abi.c" %in% result)
  expect_false("src/rust/document.rs" %in% result)
  # Superseded mis-anchored cargo-config pattern is dropped (#1226)
  expect_false(".cargo/config.toml" %in% result)
})

test_that("check_configure_ac_drift detects drift despite all legacy markers", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  writeLines("Package: driftcheck", file.path(tmp, "DESCRIPTION"))

  # All three markers used by the old heuristic are present, but the local
  # override and other current build-system logic are absent.
  outdated <- c(
    "AC_INIT([driftcheck], [0.1.0])",
    "AC_CONFIG_AUX_DIR([tools])",
    "CARGO_STATICLIB_NAME=driftcheck",
    "CARGO_TARGET_DIR=target",
    "AC_OUTPUT"
  )
  configure <- file.path(tmp, "configure.ac")
  writeLines(outdated, configure)

  expect_warning(
    minirextendr:::check_configure_ac_drift(),
    "configure.ac differs from the current template"
  )
  expect_identical(readLines(configure), outdated)
})

test_that("check_configure_ac_drift accepts the freshly rendered template", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  writeLines("Package: driftcheck", file.path(tmp, "DESCRIPTION"))
  suppressMessages(minirextendr:::use_miniextendr_configure())

  expect_no_warning(minirextendr:::check_configure_ac_drift())
})

test_that("check_configure_ac_drift reports arbitrary edits without replacing them", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  writeLines("Package: driftcheck", file.path(tmp, "DESCRIPTION"))
  suppressMessages(minirextendr:::use_miniextendr_configure())

  configure <- file.path(tmp, "configure.ac")
  customized <- c(readLines(configure), "CARGO_FEATURES='serde'")
  writeLines(customized, configure)

  expect_warning(
    minirextendr:::check_configure_ac_drift(),
    "configure.ac differs from the current template"
  )
  expect_identical(readLines(configure), customized)
})

test_that("configure drift uses the package layout, not prior scaffold state", {
  tmp <- withr::local_tempdir()
  usethis::local_project(tmp, force = TRUE, setwd = FALSE)
  writeLines("Package: driftcheck", file.path(tmp, "DESCRIPTION"))
  old_type <- minirextendr:::get_template_type()
  withr::defer(minirextendr:::set_template_type(old_type))
  minirextendr:::set_template_type("rpkg")
  suppressMessages(minirextendr:::use_miniextendr_configure())

  minirextendr:::set_template_type("monorepo")
  expect_no_warning(minirextendr:::check_configure_ac_drift())
  expect_identical(minirextendr:::get_template_type(), "monorepo")
})

test_that("configure drift accepts the current monorepo package template", {
  tmp <- withr::local_tempdir()
  writeLines("[workspace]", file.path(tmp, "Cargo.toml"))
  pkg <- file.path(tmp, "rpkg")
  dir.create(pkg)
  writeLines("Package: driftcheck", file.path(pkg, "DESCRIPTION"))
  usethis::local_project(pkg, force = TRUE, setwd = FALSE)
  old_type <- minirextendr:::get_template_type()
  withr::defer(minirextendr:::set_template_type(old_type))
  minirextendr:::set_template_type("monorepo")
  suppressMessages(minirextendr:::use_miniextendr_configure(subdir = "rpkg"))

  minirextendr:::set_template_type("rpkg")
  expect_no_warning(minirextendr:::check_configure_ac_drift())
  expect_identical(minirextendr:::get_template_type(), "rpkg")
})
