#!/usr/bin/env Rscript

# Full-suite gctorture2 sweep over rpkg's testthat suite.
#
# Surfaces the long-tail use-after-free class that the fast per-function
# gctorture(TRUE) sweep misses: any SEXP reachable through Rust state but not
# rooted in R's protect mechanism gets collected on the next allocation. The
# strict-glibc R-release runner aborts on these (`malloc(): unsorted double
# linked list corrupted`); other runtimes silently corrupt and "pass".
#
# Honors the harness ordering documented in docs/GCTORTURE_TESTING.md:
#   1. library(miniextendr) FIRST (loading under gctorture crashes on unrelated
#      CRAN-package .onLoad hooks).
#   2. Pre-attach the lazy-loaded helpers test_dir() would otherwise attach
#      mid-run (each .onLoad is a gctorture-amplified hazard — journal
#      2026-05-08-gctorture-uaf-audit.md).
#   3. gctorture2(step = 100, wait = 0, inhibit_release = FALSE).
#   4. testthat::test_dir(...) over rpkg/tests/testthat.
#
# Intended to run on a scheduled CI job (.github/workflows/gctorture-nightly.yml)
# and locally for bisects. Expect 30-90 minutes locally; longer in CI.
#
# Usage:
#   Rscript scripts/gctorture-full-sweep.R [tests_dir] [step]
# Defaults: tests_dir = rpkg/tests/testthat, step = 100.

args <- commandArgs(trailingOnly = TRUE)
tests_dir <- if (length(args) >= 1L && nzchar(args[[1L]])) args[[1L]] else "rpkg/tests/testthat"
step <- if (length(args) >= 2L && nzchar(args[[2L]])) as.integer(args[[2L]]) else 100L

if (!dir.exists(tests_dir)) {
  stop(sprintf("tests dir not found: %s (run from repo root)", tests_dir))
}

# The throwaway library `just gctorture-full` installed rpkg into. Prepended
# INSIDE the session because .libPaths() as set by env vars doesn't survive
# startup in either environment: locally the repo .Rprofile (rv activation)
# replaces the path list wholesale, and in CI an R_LIBS_USER override would
# hide the runner's dependency library (where setup-r-dependencies put R6,
# S7, testthat, ...). Prepending keeps both the fresh install and the deps.
sweep_lib <- Sys.getenv("MINIEXTENDR_GCTORTURE_LIB", "")
if (nzchar(sweep_lib)) {
  .libPaths(c(sweep_lib, .libPaths()))
}

# --- 1. Load the package FIRST, before any gctorture. --------------------------
library(miniextendr)
library(testthat)

# --- 2. Pre-attach lazy helpers test_dir() would otherwise load mid-sweep. -----
# Each .onLoad under gctorture is a torture-amplified hazard in an unrelated
# CRAN package; attach them now so the sweep only stresses our own code paths.
for (pkg in c("rlang", "withr", "lifecycle", "cli", "vctrs", "jsonlite")) {
  suppressWarnings(suppressMessages(
    tryCatch(
      requireNamespace(pkg, quietly = TRUE),
      error = function(e) FALSE
    )
  ))
}

# Don't let R's wall-clock limits abort the (deliberately) slow run.
setTimeLimit(Inf, Inf, transient = FALSE)
options(timeout = max(unlist(options("timeout")), 1e9))

cat(sprintf(
  "gctorture-full-sweep: step=%d over %s (miniextendr %s)\n",
  step, tests_dir, as.character(utils::packageVersion("miniextendr"))
))

# --- 3. Enable full-suite torture. ---------------------------------------------
gctorture2(step = step, wait = 0L, inhibit_release = FALSE)

# --- 4. Run the whole suite; never stop on first failure (collect everything). -
res <- tryCatch(
  testthat::test_dir(
    tests_dir,
    reporter = testthat::ProgressReporter,
    stop_on_failure = FALSE
  ),
  error = function(e) e
)

# Disable torture before we do any reporting allocations.
gctorture2(step = 0L)

if (inherits(res, "condition")) {
  cat("gctorture-full-sweep: test_dir() itself errored:\n")
  cat(conditionMessage(res), "\n")
  quit(status = 1L, save = "no")
}

df <- as.data.frame(res)
failed <- df[df$failed > 0L | df$error, , drop = FALSE]

n_failed <- sum(df$failed)
n_error <- sum(df$error)

cat("\n========================================\n")
cat(sprintf(
  "gctorture-full-sweep summary: %d files, %d failed assertions, %d errored files\n",
  nrow(df), n_failed, sum(df$error)
))
cat("========================================\n")

if (nrow(failed) > 0L) {
  cat("\nOffending test files:\n")
  for (i in seq_len(nrow(failed))) {
    cat(sprintf(
      "  - %s :: %s (failed=%d error=%s)\n",
      failed$file[[i]], failed$context[[i]],
      failed$failed[[i]], as.character(failed$error[[i]])
    ))
  }
}

if (n_failed > 0L || n_error > 0L) {
  quit(status = 1L, save = "no")
}

cat("gctorture-full-sweep: clean — 0 failures, 0 errors.\n")
quit(status = 0L, save = "no")
