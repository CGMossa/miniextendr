# Tests for system command execution helpers (R/system.R)

# Resolve the Rscript that ships with the running R, or "" if absent.
# Use R.home("bin") rather than Sys.which("Rscript"): under `R CMD check` the
# bare name resolves to a wrapper on PATH that exits non-zero with "'Rscript'
# should not be used without a path -- see par. 1.6 of the manual". Addressing
# it by its home path (the method Writing R Extensions sec. 1.6 prescribes)
# invokes the real interpreter and sidesteps the wrapper.
find_rscript <- function() {
  exe <- if (.Platform$OS.type == "windows") "Rscript.exe" else "Rscript"
  cand <- file.path(R.home("bin"), exe)
  if (file.exists(cand)) cand else ""
}

test_that("run_with_logging captures a non-zero exit without leaking a warning", {
  rscript <- find_rscript()
  skip_if_not(nzchar(rscript), "Rscript not found")

  # Point Rscript at a file that does not exist: deterministic non-zero exit,
  # no shell metacharacters in the argument. base R raises a
  # "running command ... had status N" warning on non-zero exit when output is
  # captured; run_with_logging() suppresses it because the status is reported
  # through $status/$success instead. Regression guard for #798.
  missing <- file.path(tempdir(), "minirextendr-no-such-script-9f3c.R")

  result <- expect_no_warning(
    minirextendr:::run_with_logging(rscript, missing, log_prefix = "test-nonzero")
  )

  # Status is still reported correctly despite the silenced warning.
  expect_false(result$success)
  expect_false(is.null(result$status))
  expect_gt(result$status, 0L)
})

test_that("run_with_logging reports success for a zero-exit command", {
  rscript <- find_rscript()
  skip_if_not(nzchar(rscript), "Rscript not found")

  # Run an empty R script: a no-op program that exits 0.
  empty <- file.path(tempdir(), "minirextendr-empty-9f3c.R")
  file.create(empty)
  on.exit(unlink(empty), add = TRUE)

  result <- minirextendr:::run_with_logging(rscript, empty, log_prefix = "test-zero")

  expect_true(result$success)
  expect_null(result$status)
})

# An *empty* R script: a metacharacter-free, zero-exit no-op used by the
# env-restore tests below. We only care that run_with_logging() runs *some*
# command (so the env-apply/on.exit-restore path executes), not its output.
# Returns the path; caller is responsible for unlink() via its own on.exit.
make_empty_rscript <- function() {
  empty <- file.path(tempdir(), "minirextendr-empty-env-9f3c.R")
  file.create(empty)
  empty
}

test_that("run_with_logging restores a single already-set env var (#804)", {
  rscript <- find_rscript()
  skip_if_not(nzchar(rscript), "Rscript not found")

  empty <- make_empty_rscript()
  on.exit(unlink(empty), add = TRUE)

  # Pre-set a sentinel value and arrange to restore the prior state ourselves
  # (base Sys.setenv/Sys.unsetenv; withr is only needed for the empty-script
  # defer above, which is part of the test scaffolding, not the code path).
  var <- "MINIREXTENDR_TEST_ENV_804"
  prior <- Sys.getenv(var, unset = NA)
  Sys.setenv(MINIREXTENDR_TEST_ENV_804 = "sentinel")
  on.exit(
    if (is.na(prior)) Sys.unsetenv(var) else do.call(Sys.setenv, stats::setNames(list(prior), var)),
    add = TRUE
  )

  # Single-variable env: this is the case that tripped "all arguments must be
  # named" before names = TRUE was added to the Sys.getenv() capture.
  expect_no_error(
    minirextendr:::run_with_logging(
      rscript, empty,
      log_prefix = "test-env-single",
      env = c(MINIREXTENDR_TEST_ENV_804 = "temp")
    )
  )

  # The sentinel value must be restored once run_with_logging() returns.
  expect_identical(Sys.getenv(var, unset = NA), "sentinel")
})

test_that("run_with_logging restores multiple env vars and unsets previously-unset ones", {
  rscript <- find_rscript()
  skip_if_not(nzchar(rscript), "Rscript not found")

  empty <- make_empty_rscript()
  on.exit(unlink(empty), add = TRUE)

  set_var <- "MINIREXTENDR_TEST_ENV_804_A"   # already set -> restore to sentinel
  unset_var <- "MINIREXTENDR_TEST_ENV_804_B" # previously unset -> must end unset

  prior_set <- Sys.getenv(set_var, unset = NA)
  prior_unset <- Sys.getenv(unset_var, unset = NA)
  Sys.setenv(MINIREXTENDR_TEST_ENV_804_A = "sentinel-a")
  Sys.unsetenv(unset_var)
  on.exit(
    {
      if (is.na(prior_set)) Sys.unsetenv(set_var) else do.call(Sys.setenv, stats::setNames(list(prior_set), set_var))
      if (is.na(prior_unset)) Sys.unsetenv(unset_var) else do.call(Sys.setenv, stats::setNames(list(prior_unset), unset_var))
    },
    add = TRUE
  )

  expect_no_error(
    minirextendr:::run_with_logging(
      rscript, empty,
      log_prefix = "test-env-multi",
      env = c(
        MINIREXTENDR_TEST_ENV_804_A = "temp-a",
        MINIREXTENDR_TEST_ENV_804_B = "temp-b"
      )
    )
  )

  # The set var is restored to its sentinel; the previously-unset var is unset
  # again (Sys.getenv returns "" for an unset name without `names`/`unset`).
  expect_identical(Sys.getenv(set_var, unset = NA), "sentinel-a")
  expect_identical(Sys.getenv(unset_var, unset = NA), NA_character_)
})

test_that("run_with_logging preserves argument boundaries and literal characters", {
  rscript <- find_rscript()
  skip_if_not(nzchar(rscript), "Rscript not found")
  tmp <- withr::local_tempdir(pattern = "system arguments ")
  script <- file.path(tmp, "echo arguments.R")
  writeLines("writeLines(commandArgs(trailingOnly = TRUE))", script)
  args <- c("two words", 'double "quoted"', "back\\slash", "$(printf wrong)")

  result <- minirextendr:::run_with_logging(rscript, c(script, args))
  expect_true(result$success)
  expect_identical(result$output, args)
})
