# Configure-related scaffolding functions

#' Add configure.ac to package
#'
#' Creates the autoconf configure.ac template that handles Rust toolchain
#' discovery, feature flags, and generates build files.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param subdir Optional template subdirectory (passed through to
#'   `use_template()`) — set to `"rpkg"` when scaffolding the R package
#'   subdirectory of a monorepo.
#' @return Invisibly returns TRUE if file was created
#' @keywords internal
use_miniextendr_configure <- function(path = ".", subdir = NULL) {
  with_project(path)
  use_template("configure.ac", subdir = subdir, data = template_data())
  cli::cli_alert_info("Run {.code minirextendr::miniextendr_autoconf()} to generate configure script")
  invisible(TRUE)
}

#' Add bootstrap.R to package
#'
#' Creates bootstrap.R which runs during package build when
#' `Config/build/bootstrap: TRUE` is set in DESCRIPTION.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param subdir Optional template subdirectory (passed through to
#'   `use_template()`) — set to `"rpkg"` when scaffolding the R package
#'   subdirectory of a monorepo.
#' @return Invisibly returns TRUE if file was created
#' @keywords internal
use_miniextendr_bootstrap <- function(path = ".", subdir = NULL) {
  with_project(path)
  use_template("bootstrap.R", subdir = subdir)
  invisible(TRUE)
}

#' Add cleanup scripts to package
#'
#' Creates cleanup, cleanup.win, and cleanup.ucrt scripts that clean
#' build artifacts after installation (used on CRAN).
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param subdir Optional template subdirectory (passed through to
#'   `use_template()`) — set to `"rpkg"` when scaffolding the R package
#'   subdirectory of a monorepo.
#' @return Invisibly returns TRUE if files were created
#' @keywords internal
use_miniextendr_cleanup <- function(path = ".", subdir = NULL) {
  with_project(path)
  use_template("cleanup", subdir = subdir)
  use_template("cleanup.win", subdir = subdir)
  use_template("cleanup.ucrt", subdir = subdir)

  # Ensure cleanup scripts are executable (R CMD build warns otherwise)
  for (script in c("cleanup", "cleanup.win", "cleanup.ucrt")) {
    script_path <- usethis::proj_path(script)
    if (fs::file_exists(script_path)) {
      fs::file_chmod(script_path, "755")
    }
  }

  invisible(TRUE)
}

#' Add Windows configure wrappers
#'
#' Creates configure.win and configure.ucrt that delegate to the main
#' POSIX configure script.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param subdir Optional template subdirectory (passed through to
#'   `use_template()`) — set to `"rpkg"` when scaffolding the R package
#'   subdirectory of a monorepo.
#' @return Invisibly returns TRUE if files were created
#' @keywords internal
use_miniextendr_configure_win <- function(path = ".", subdir = NULL) {
  with_project(path)
  use_template("configure.win", subdir = subdir)
  use_template("configure.ucrt", subdir = subdir)

  # R expects configure.win / configure.ucrt to be executable; use_template
  # writes with default mode 644 so we restore 755 here (same pattern as cleanup).
  for (script in c("configure.win", "configure.ucrt")) {
    script_path <- usethis::proj_path(script)
    if (fs::file_exists(script_path)) {
      fs::file_chmod(script_path, "755")
    }
  }

  invisible(TRUE)
}

#' Copy config.guess and config.sub scripts
#'
#' Copies the bundled GNU autoconf helper scripts config.guess and config.sub
#' to the tools/ directory. These are required for cross-compilation support
#' and are referenced by AC_CONFIG_AUX_DIR([tools]) in configure.ac.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param subdir Optional template subdirectory (passed through to
#'   `template_path()`) — set to `"rpkg"` when scaffolding the R package
#'   subdirectory of a monorepo, where `tools/lock-shape-check.R` lives one
#'   level deeper under the active "monorepo" template type.
#' @return Invisibly returns TRUE if files were copied
#' @keywords internal
use_miniextendr_config_scripts <- function(path = ".", subdir = NULL) {
  with_project(path)
  # These go in tools/ directory per AC_CONFIG_AUX_DIR([tools]) in configure.ac.
  # config.guess/config.sub are bundled scripts (not templates), shared with
  # the monorepo and inline scaffold paths via copy_config_scripts().
  ensure_dir(usethis::proj_path("tools"))
  copy_config_scripts(usethis::proj_path("tools"), display_prefix = "tools")

  # tools/lock-shape-check.R is referenced by configure.ac's
  # AC_CONFIG_COMMANDS([lock-shape-check]) block; without it, configure
  # fails in tarball mode with "cannot open file 'tools/lock-shape-check.R'".
  tools_subdir <- if (is.null(subdir)) "tools" else file.path(subdir, "tools")
  lock_check_src <- template_path("lock-shape-check.R", subdir = tools_subdir)
  lock_check_dest <- usethis::proj_path("tools", "lock-shape-check.R")
  fs::file_copy(lock_check_src, lock_check_dest, overwrite = TRUE)
  bullet_created(file.path("tools", "lock-shape-check.R"), "Copied")

  invisible(TRUE)
}

#' Add package makefiles
#'
#' Creates src/Makevars.in which is processed by configure to generate
#' the actual Makevars used during package build. Also writes the static
#' src/Makevars.win with the Windows system libraries needed by Rust.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param subdir Optional template subdirectory (passed through to
#'   `use_template()`) — set to `"rpkg"` when scaffolding the R package
#'   subdirectory of a monorepo.
#' @return Invisibly returns TRUE if file was created
#' @keywords internal
use_miniextendr_makevars <- function(path = ".", subdir = NULL) {
  with_project(path)
  ensure_dir(usethis::proj_path("src"))
  use_template("Makevars.in", save_as = "src/Makevars.in", subdir = subdir)
  use_template("Makevars.win", save_as = "src/Makevars.win", subdir = subdir)
  use_template("win.def.in", save_as = "src/win.def.in", subdir = subdir)

  invisible(TRUE)
}
