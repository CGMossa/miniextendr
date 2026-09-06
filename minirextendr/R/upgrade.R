# Package upgrade functions

#' Upgrade a miniextendr package
#'
#' Comprehensively upgrades an existing miniextendr package to the latest
#' build system templates, vendored crates, and package metadata. This replaces
#' the old `miniextendr_update()` with a more thorough upgrade that covers
#' configure.ac, DESCRIPTION, .gitignore, .Rbuildignore, build.rs, and
#' generated C files.
#'
#' User-authored files (lib.rs, Cargo.toml, R/ code) are never touched.
#'
#' In a monorepo layout (workspace root containing an rpkg subdirectory),
#' `upgrade_miniextendr_package()` automatically detects the rpkg subdir and
#' operates on it rather than the workspace root. The rpkg subdir is the first
#' immediate child directory that contains a miniextendr `configure.ac`. Pass
#' `rpkg_subdir` explicitly if auto-detection is ambiguous.
#'
#' @param path Path to the R package root (standalone) or the monorepo workspace
#'   root. Defaults to `"."`. For a monorepo, this is the directory containing
#'   `Cargo.toml` -- the rpkg subdir is resolved automatically.
#' @param rpkg_subdir For monorepo layouts: name of the R package subdirectory
#'   (e.g. `"rpkg"`). If `NULL` (default), auto-detected by scanning immediate
#'   subdirectories for a miniextendr `configure.ac`.
#' @param version Version of miniextendr crates to vendor (default: `"main"`).
#' @param local_path Optional path to local miniextendr repository for vendoring.
#' @param configure_ac Logical. If `TRUE`, overwrites configure.ac with the
#'   current template. Defaults to `FALSE` because users often customise
#'   configure.ac with feature flags. When `FALSE`, a heuristic check warns
#'   if the existing configure.ac appears outdated.
#' @param autoconf Logical. If `TRUE` (default) and `autoconf` is available,
#'   regenerates the configure script after upgrading.
#' @param allow_dirty Logical. If `FALSE` (default), aborts when scaffolding
#'   files have uncommitted changes in git, to prevent accidental data loss.
#'   Set to `TRUE` to force the upgrade even with dirty files.
#' @return Invisibly returns TRUE on success.
#' @export
upgrade_miniextendr_package <- function(path = ".",
                                         rpkg_subdir = NULL,
                                         version = "main",
                                         local_path = NULL,
                                         configure_ac = FALSE,
                                         autoconf = TRUE,
                                         allow_dirty = FALSE) {
  # Resolve path to an absolute path so we can probe it before setting as project.
  resolved_path <- normalizePath(path, mustWork = FALSE)

  # Detect monorepo: if path has a Cargo.toml but no configure.ac (or no
  # DESCRIPTION), it's likely a workspace root rather than an rpkg root.
  project_type <- detect_project_type(resolved_path)
  if (identical(project_type, "monorepo") &&
      !file.exists(file.path(resolved_path, "configure.ac"))) {
    # Path is the workspace root -- need to resolve to the rpkg subdir.
    subdir <- rpkg_subdir %||% find_rpkg_subdir(resolved_path)
    if (is.null(subdir)) {
      cli::cli_abort(c(
        "Could not find an rpkg subdirectory in {.path {resolved_path}}.",
        "i" = "Pass {.code rpkg_subdir = '<name>'} explicitly.",
        "i" = "Expected a subdirectory with a miniextendr {.path configure.ac}."
      ))
    }
    resolved_path <- file.path(resolved_path, subdir)
    cli::cli_alert_info("Monorepo layout detected -- upgrading rpkg subdir {.path {subdir}}")
  }

  with_project(resolved_path)

  if (!is_miniextendr_package()) {
    cli::cli_abort(c(
      "This does not appear to be a miniextendr package.",
      "i" = "Expected configure.ac with CARGO_FEATURES variable and build templates.",
      "i" = "Use {.code create_miniextendr_package()} to scaffold a new package."
    ))
  }

  cli::cli_h1("Upgrading miniextendr package")

  # --- Dirty check ---
  if (!allow_dirty) {
    check_scaffolding_clean(resolved_path)
  }

  # --- Build system templates ---
  cli::cli_h2("Updating build system templates")
  use_miniextendr_stub()
  use_miniextendr_makevars()
  use_miniextendr_mx_abi()
  use_miniextendr_build_rs()
  use_miniextendr_bootstrap()
  use_miniextendr_cleanup()
  use_miniextendr_configure_win()
  use_miniextendr_config_scripts()

  # --- Package metadata ---
  cli::cli_h2("Updating package metadata")
  use_miniextendr_description()
  use_miniextendr_rbuildignore()
  upgrade_gitignore()

  # --- configure.ac ---
  if (configure_ac) {
    cli::cli_h2("Replacing configure.ac")
    use_miniextendr_configure()
  } else {
    check_configure_ac_drift()
  }

  # --- Autoconf ---
  if (autoconf && nzchar(Sys.which("autoconf"))) {
    cli::cli_h2("Regenerating configure script")
    tryCatch(
      miniextendr_autoconf(),
      error = function(e) {
        cli::cli_alert_warning("autoconf failed: {conditionMessage(e)}")
      }
    )
  }

  # --- Summary ---
  cli::cli_h1("Upgrade complete!")
  cli::cli_alert_info("Next steps:")
  cli::cli_bullets(c(
    " " = "Review changes with {.code git diff}",
    " " = "Run {.code minirextendr::miniextendr_build()} to rebuild"
  ))

  invisible(TRUE)
}

#' Find the rpkg subdirectory in a monorepo workspace root
#'
#' Scans immediate subdirectories of `path` for one that contains a
#' `configure.ac` with the `CARGO_FEATURES` marker (the canonical signal that
#' a directory is a miniextendr rpkg). Returns the single match, `NULL` if
#' none is found, or aborts if more than one match is found.
#'
#' @param path Path to the monorepo workspace root.
#' @return Name of the rpkg subdirectory (not a full path), or `NULL`.
#' @noRd
find_rpkg_subdir <- function(path) {
  subdirs <- list.dirs(path, full.names = FALSE, recursive = FALSE)
  matches <- character(0)
  for (d in subdirs) {
    configure_ac <- file.path(path, d, "configure.ac")
    if (file.exists(configure_ac)) {
      content <- readLines(configure_ac, warn = FALSE)
      if (any(grepl("CARGO_FEATURES", content, fixed = TRUE))) {
        matches <- c(matches, d)
      }
    }
  }
  if (length(matches) > 1) {
    cli::cli_abort(c(
      "Multiple rpkg subdirectories detected in {.path {path}}:",
      stats::setNames(matches, rep("*", length(matches))),
      "i" = "Pass {.code rpkg_subdir = '<name>'} explicitly to disambiguate."
    ))
  }
  if (length(matches) == 0L) return(NULL)
  matches
}

#' Check that scaffolding files are clean in git
#'
#' Uses `git status --porcelain` to inspect build system files that will be
#' overwritten during upgrade. Aborts if any have uncommitted changes.
#'
#' Every git invocation runs *in* `proj_dir` (via `run_command(wd = )`), not the
#' caller's working directory. `with_project()` activates the usethis project
#' without changing the working directory (`setwd = FALSE`), so probing git in
#' `getwd()` would inspect an unrelated repo -- or no repo at all, which would
#' silently skip this guard and let the upgrade overwrite the target's
#' uncommitted files.
#'
#' @param proj_dir Package directory to inspect (default `usethis::proj_get()`).
#'   The relative `scaffolding_files` pathspecs are interpreted against it.
#' @noRd
check_scaffolding_clean <- function(proj_dir = usethis::proj_get()) {
  # Bail out if git is not available
  if (!nzchar(Sys.which("git"))) return(invisible())

  # Bail out if not in a git repo
  repo <- tryCatch({
    out <- run_command("git", c("rev-parse", "--show-toplevel"), wd = proj_dir)
    if (!is.null(attr(out, "status"))) NULL else out
  }, error = function(e) NULL)
  if (is.null(repo)) return(invisible())

  # Files that upgrade will overwrite
  scaffolding_files <- c(
    "src/stub.c",
    "src/r_shim.h",
    "src/rust/build.rs",
    "src/Makevars.in",
    "src/Makevars.win",
    "src/win.def.in",
    "inst/include/mx_abi.h",
    "bootstrap.R",
    "cleanup",
    "cleanup.win",
    "cleanup.ucrt",
    "configure.win",
    "configure.ucrt",
    "tools/config.guess",
    "tools/config.sub",
    ".Rbuildignore",
    ".gitignore"
  )

  out <- run_command("git", c("status", "--porcelain", "--", scaffolding_files),
                     wd = proj_dir)
  if (is.null(out) || length(out) == 0) return(invisible())

  cli::cli_abort(c(
    "Scaffolding files have uncommitted changes.",
    "i" = "Commit or stash your changes first, or use {.code allow_dirty = TRUE} to force.",
    paste(" ", out)
  ))
}

#' Upgrade .gitignore patterns
#'
#' Adds current miniextendr patterns (usethis deduplicates) and removes
#' known-obsolete entries that are no longer needed (files now tracked in git).
#'
#' @noRd
upgrade_gitignore <- function() {
  # Add current patterns (usethis handles deduplication)
  use_miniextendr_gitignore()

  # Remove obsolete entries that are now tracked in git
  gitignore_path <- usethis::proj_path(".gitignore")
  if (!fs::file_exists(gitignore_path)) return(invisible())

  lines <- readLines(gitignore_path, warn = FALSE)
  obsolete <- c("src/entrypoint.c", "src/entrypoint.c.in", "src/mx_abi.c",
                "src/mx_abi.c.in", "src/rust/document.rs",
                # Superseded by the correctly-anchored `src/rust/.cargo/config.toml`
                # (#1226); the bare pattern never matched the nested path.
                ".cargo/config.toml")

  # Remove exact matches (trimmed)
  trimmed <- trimws(lines)
  keep <- !(trimmed %in% obsolete)

  if (sum(!keep) > 0) {
    removed <- trimmed[!keep]
    writeLines(lines[keep], gitignore_path)
    for (entry in removed) {
      cli::cli_alert_success("Removed obsolete .gitignore entry: {.val {entry}}")
    }
  }

  invisible()
}

#' Check configure.ac for drift
#'
#' Heuristic check for key structural elements that indicate the configure.ac
#' is up to date. Warns if missing.
#'
#' @noRd
check_configure_ac_drift <- function() {
  configure_ac <- usethis::proj_path("configure.ac")
  if (!fs::file_exists(configure_ac)) return(invisible())

  content <- readLines(configure_ac, warn = FALSE)
  text <- paste(content, collapse = "\n")

  missing <- character()
  if (!grepl("CARGO_STATICLIB_NAME", text, fixed = TRUE)) {
    missing <- c(missing, "CARGO_STATICLIB_NAME substitution")
  }
  if (!grepl("AC_CONFIG_AUX_DIR", text, fixed = TRUE)) {
    missing <- c(missing, "AC_CONFIG_AUX_DIR([tools])")
  }
  if (!grepl("CARGO_TARGET_DIR", text, fixed = TRUE)) {
    missing <- c(missing, "CARGO_TARGET_DIR setup")
  }

  if (length(missing) > 0) {
    cli::cli_warn(c(
      "configure.ac may be outdated (missing: {paste(missing, collapse = ', ')})",
      "i" = "Re-run with {.code configure_ac = TRUE} to replace it with the current template.",
      "!" = "This will overwrite any custom feature flags in configure.ac."
    ))
  }

  invisible()
}
