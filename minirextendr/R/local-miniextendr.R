# Local miniextendr checkout override helpers (#908)
#
# Developers who work on both miniextendr and a consumer package can wire the
# consumer's configure.ac to resolve the framework crates from a local source
# tree instead of the published git URL.  The mechanism is a one-line marker
# file (.miniextendr-local) at the package root; configure.ac reads it with
# plain shell (no minirextendr::*, per repo rule).  The file is gitignored and
# Rbuildignored, so it can never ship in a tarball.

#' Wire a local miniextendr framework checkout (dev-only)
#'
#' Records an absolute path to a local miniextendr checkout in a one-line
#' `.miniextendr-local` marker file at the R package root.  When
#' `bash ./configure` runs, it reads this file and sets `MONOREPO_ROOT`
#' to the recorded path so the `[patch."https://github.com/A2-ai/miniextendr"]`
#' block in `.cargo/config.toml` resolves the framework crates from the local
#' tree.  Tarball mode (`inst/vendor.tar.xz` present) always takes precedence.
#'
#' The marker file is added to `.gitignore` and `.Rbuildignore` automatically,
#' so it can never appear in a distributed tarball.
#'
#' @param miniextendr_path Path to a local miniextendr repository checkout.
#'   Canonicalized to an absolute POSIX path; the directory must contain
#'   `miniextendr-api/Cargo.toml`.
#' @param path Path to the R package root, or `"."` to use the current
#'   directory (house convention, matching e.g. `use_vendor_lib()`).
#' @return Invisibly returns `TRUE`.
#' @seealso [unuse_local_miniextendr()]
#' @export
use_local_miniextendr <- function(miniextendr_path, path = ".") {
  with_project(path)
  if (!is_miniextendr_package()) {
    cli::cli_abort("Not a miniextendr package (configure.ac with CARGO_FEATURES not found)")
  }

  # Canonicalize to absolute path; reject non-existent paths.
  abs <- normalizePath(miniextendr_path, winslash = "/", mustWork = TRUE)
  # Strip \\?\ UNC prefix on Windows (CLAUDE.md: Windows TOML paths rule).
  abs <- sub("^\\\\\\\\\\?\\\\", "", abs)
  abs <- sub("^//\\?/", "", abs)
  # Ensure forward slashes throughout (TOML/cargo requirement on Windows).
  abs <- gsub("\\\\", "/", abs)

  if (!file.exists(file.path(abs, "miniextendr-api", "Cargo.toml"))) {
    cli::cli_abort(c(
      "{.path {abs}} does not look like a miniextendr checkout",
      "x" = "{.path miniextendr-api/Cargo.toml} not found under it"
    ))
  }

  # Tarball latch present -> the override would be inert; warn loudly.
  if (file.exists(usethis::proj_path("inst", "vendor.tar.xz"))) {
    cli::cli_alert_warning(
      "{.path inst/vendor.tar.xz} is present: tarball mode wins over the local \\
override. The marker is recorded but inert until the latch is cleared \\
({.code miniextendr_clean_vendor_leak()})."
    )
  }

  # configure.ac support probe: older scaffolds predate the marker branch.
  cac <- readLines(usethis::proj_path("configure.ac"), warn = FALSE)
  if (!any(grepl(".miniextendr-local", cac, fixed = TRUE))) {
    cli::cli_alert_warning(
      "This package's {.path configure.ac} predates local-checkout support. \\
Run {.code upgrade_miniextendr_package()} then {.code miniextendr_autoconf()} \\
to pick up the marker-file branch."
    )
  }

  writeLines(abs, usethis::proj_path(".miniextendr-local"))
  clean_local_build_ignore()
  usethis::use_build_ignore(".miniextendr-local")
  usethis::use_git_ignore(".miniextendr-local")
  cli::cli_alert_success("Recorded local miniextendr checkout: {.path {abs}}")
  cli::cli_alert_info(
    "Run {.code miniextendr_configure()} to regenerate \\
{.path src/rust/.cargo/config.toml} with the local path override."
  )
  invisible(TRUE)
}

#' Remove a local miniextendr framework override
#'
#' Deletes the `.miniextendr-local` marker file written by
#' [use_local_miniextendr()].  Safe to call even when no marker exists
#' (idempotent).
#'
#' @param path Path to the R package root, or `"."` to use the current
#'   directory.
#' @return Invisibly returns `TRUE` if the marker was removed, `FALSE` if no
#'   marker was present.
#' @seealso [use_local_miniextendr()]
#' @export
unuse_local_miniextendr <- function(path = ".") {
  with_project(path)
  clean_local_build_ignore()
  marker <- usethis::proj_path(".miniextendr-local")
  if (!fs::file_exists(marker)) {
    cli::cli_alert_info("No {.path .miniextendr-local} marker present \u2014 nothing to remove.")
    return(invisible(FALSE))
  }
  fs::file_delete(marker)
  cli::cli_alert_success("Removed {.path .miniextendr-local} local miniextendr override.")
  cli::cli_alert_info(
    "Run {.code miniextendr_configure()} to rewrite \\
{.path src/rust/.cargo/config.toml} without the local path override."
  )
  invisible(TRUE)
}

# Remove the double-escaped entry written before #1405. Keep unrelated rules
# and the correct ignore entry, which still applies if the override is reused.
clean_local_build_ignore <- function() {
  path <- usethis::proj_path(".Rbuildignore")
  if (!fs::file_exists(path)) return(invisible())

  lines <- readLines(path, warn = FALSE)
  obsolete <- lines == "^\\\\.miniextendr-local$$"
  if (any(obsolete)) {
    writeLines(lines[!obsolete], path)
  }
  invisible()
}
