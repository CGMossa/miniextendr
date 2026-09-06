test_that("CLI feature rules interoperate with R and drive configure detection", {
  cli <- Sys.getenv("MINIEXTENDR_CLI", "")
  skip_if(!nzchar(cli), "set MINIEXTENDR_CLI to the built CLI binary")
  expect_true(file.exists(cli))
  skip_if(!nzchar(Sys.which("autoconf")), "autoconf not available")
  withr::local_envvar(c(
    R_LIBS = paste(.libPaths(), collapse = .Platform$path.sep),
    CARGO_NET_OFFLINE = "true"
  ))

  pkg <- withr::local_tempdir(pattern = "cli feature package ")
  dir.create(file.path(pkg, "src", "rust"), recursive = TRUE)
  writeLines(c("Package: clifeatures", "Version: 0.1.0"),
             file.path(pkg, "DESCRIPTION"))
  writeLines(c(
    "AC_INIT([clifeatures], [0.1.0])",
    'if test -z "${CARGO_FEATURES+x}"; then',
    "  dnl CARGO_FEATURES not set - use empty (no extra features)",
    '  CARGO_FEATURES=""',
    "fi", "AC_OUTPUT"
  ), file.path(pkg, "configure.ac"))
  writeLines("# old Makevars template", file.path(pkg, "src", "Makevars.in"))
  writeLines(character(), file.path(pkg, "NAMESPACE"))
  manifest <- file.path(pkg, "src", "rust", "Cargo.toml")
  writeLines(c(
    "[package]", 'name = "clifeatures"', 'version = "0.1.0"',
    "[lib]", 'path = "lib.rs"', "[features]", "alpha = []", "beta = []"
  ), manifest)
  writeLines("pub fn example() {}", file.path(pkg, "src", "rust", "lib.rs"))

  run_cli <- function(args) {
    out <- system2(cli, shQuote(c("--path", pkg, args)), stdout = TRUE, stderr = TRUE)
    expect_null(attr(out, "status"), info = paste(out, collapse = "\n"))
    out
  }
  detect <- function() {
    withr::with_dir(pkg, system2(file.path(R.home("bin"), "Rscript"),
                                "tools/detect-features.R", stdout = TRUE))
  }

  run_cli(c("feature", "detect", "init"))
  script <- file.path(pkg, "tools", "detect-features.R")
  expect_true(any(grepl("^## BEGIN RULES", readLines(script))))
  expect_true(file.exists(file.path(pkg, "configure")))
  expect_true(any(grepl("${srcdir}/tools/detect-features.R",
                       readLines(file.path(pkg, "configure.ac")), fixed = TRUE)))
  expect_equal(detect(), "alpha,beta")

  expr <- 'identical("quoted \\\"value\\\"", "different")'
  run_cli(c("feature", "rule", "add", "beta", expr))
  expect_equal(list_feature_rules(path = pkg), list(beta = expr))
  expect_equal(detect(), "alpha")

  # R-written rules must be visible in CLI output, including JSON values.
  add_feature_rule("alpha", "FALSE", path = pkg)
  listed <- run_cli(c("feature", "rule", "list", "--json"))
  expect_equal(jsonlite::fromJSON(paste(listed, collapse = "\n")),
               list(alpha = "FALSE", beta = expr))
  run_cli(c("feature", "rule", "remove", "alpha"))
  expect_equal(list_feature_rules(path = pkg), list(beta = expr))

  # Repeated setup and add preserve existing predicates.
  before <- readLines(script)
  run_cli(c("feature", "detect", "init"))
  run_cli(c("feature", "rule", "add", "beta", "TRUE"))
  expect_identical(readLines(script), before)
  remove_feature_rule("beta", path = pkg)
  expect_equal(detect(), "alpha,beta")

  run_cli(c("feature", "rule", "add", "extra", "TRUE", "--cargo-spec", "alpha"))
  expect_true(any(grepl('extra = ["alpha"]', readLines(manifest), fixed = TRUE)))
  run_cli(c("feature", "rule", "add", "serde", "TRUE", "--optional-dep"))
  expect_true(any(grepl("serde.*optional = true", readLines(manifest))))
  expect_equal(list_feature_rules(path = pkg), list(extra = "TRUE", serde = "TRUE"))

  # Setup warns about legacy scripts, preserving user content for review.
  legacy <- c("detect_features <- function() character()", "# custom rule")
  writeLines(legacy, script)
  warning <- run_cli(c("feature", "detect", "init"))
  expect_true(any(grepl("no.*BEGIN RULES.*marker", warning)))
  expect_identical(readLines(script), legacy)
  manifest_before <- readLines(manifest)
  rejected <- suppressWarnings(system2(
    cli, shQuote(c("--path", pkg, "feature", "rule", "add", "invalid", "TRUE",
                   "--optional-dep")), stdout = TRUE, stderr = TRUE
  ))
  expect_identical(attr(rejected, "status"), 1L)
  expect_identical(readLines(script), legacy)
  expect_identical(readLines(manifest), manifest_before)

  # CLI upgrade refreshes templates through the R helper instead of merely
  # running configure against the unchanged, stale scaffold.
  rust_before <- readLines(file.path(pkg, "src", "rust", "lib.rs"))
  cargo_before <- readLines(manifest)
  run_cli(c("workflow", "upgrade"))
  expect_true(file.exists(file.path(pkg, "bootstrap.R")))
  expect_identical(readLines(file.path(pkg, "src", "Makevars.in")), readLines(
    system.file("templates", "rpkg", "Makevars.in", package = "minirextendr")
  ))
  expect_identical(readLines(manifest), cargo_before)
  expect_identical(readLines(file.path(pkg, "src", "rust", "lib.rs")), rust_before)
})
