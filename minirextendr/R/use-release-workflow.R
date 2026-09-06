# Release workflow scaffolding

#' Add an r-release.yml workflow with miniextendr platform fixes
#'
#' Copies the bundled `r-release.yml` template into a target package's
#' `.github/workflows/` directory. The template encodes the four platform
#' gotchas surfaced when running release workflows against AlmaLinux 8
#' container builds and macOS arm64 runners (issue #448):
#'
#' 1. `LANG=C.UTF-8` scoped to container jobs -- AlmaLinux 8 minimal defaults
#'    to the `C` locale, which fails miniextendr's UTF-8 assertion.
#' 2. Not set workflow-wide -- macOS rejects `C.UTF-8` (glibc-only identifier).
#' 3. `dnf install git gh` + `gh auth setup-git` for AlmaLinux 8 minimal.
#' 4. `CARGO_NET_GIT_FETCH_WITH_CLI=true` workflow-wide so cargo uses the
#'    system git binary and can authenticate private git dependencies.
#'
#' See the [release workflow guide](https://a2-ai.github.io/miniextendr/manual/release-workflow/)
#' for the full platform rationale and the explanation of the UTF-8 check.
#'
#' For **monorepo layouts** (where the R package lives in a subdirectory such as
#' `rpkg/`), the `bash ./configure` and `R CMD build` steps must run from inside
#' that subdirectory. Pass `rpkg_subdir` to activate this; the generated workflow
#' will add `working-directory: <rpkg_subdir>` to those steps. When `rpkg_subdir`
#' is `NULL` (default) and `auto_detect_subdir` is `TRUE` (default),
#' auto-detection via `detect_project_type()` is attempted.
#' For a confirmed standalone package, set `auto_detect_subdir = FALSE` to skip
#' detection and write the plain template unchanged.
#'
#' @param path Path to the package root (standalone) or monorepo workspace root.
#'   Defaults to `"."`.
#' @param rpkg_subdir For monorepo layouts: name of the R package subdirectory
#'   (e.g. `"rpkg"`). If `NULL` (default) and `auto_detect_subdir` is `TRUE`,
#'   the subdirectory is auto-detected. If a string, it is used directly
#'   regardless of `auto_detect_subdir`.
#' @param auto_detect_subdir If `TRUE` (default) and `rpkg_subdir` is `NULL`,
#'   attempts to auto-detect the monorepo layout via `detect_project_type()`.
#'   Set to `FALSE` to force standalone mode and suppress auto-detection.
#' @param overwrite If `TRUE`, replace an existing
#'   `.github/workflows/r-release.yml`. Default `FALSE`.
#' @return The path to the written file, invisibly.
#' @export
use_release_workflow <- function(path = ".", rpkg_subdir = NULL,
                                  auto_detect_subdir = TRUE,
                                  overwrite = FALSE) {
  dest_dir <- file.path(path, ".github", "workflows")
  dest_file <- file.path(dest_dir, "r-release.yml")

  if (fs::file_exists(dest_file) && !overwrite) {
    cli::cli_abort(c(
      "{.path {dest_file}} already exists.",
      "i" = "Use {.code overwrite = TRUE} to replace it."
    ))
  }

  # Resolve rpkg_subdir:
  #   - explicit string -> use it directly.
  #   - NULL + auto_detect_subdir = TRUE -> attempt auto-detection.
  #   - NULL + auto_detect_subdir = FALSE -> standalone (no detection).
  resolved_subdir <- NULL
  if (!is.null(rpkg_subdir)) {
    resolved_subdir <- as.character(rpkg_subdir)
  } else if (auto_detect_subdir) {
    resolved_path <- normalizePath(path, mustWork = FALSE)
    project_type <- detect_project_type(resolved_path)
    if (identical(project_type, "monorepo") &&
        !file.exists(file.path(resolved_path, "configure.ac"))) {
      resolved_subdir <- find_rpkg_subdir(resolved_path)
      if (is.null(resolved_subdir)) {
        cli::cli_warn(c(
          "Monorepo layout detected but could not find an rpkg subdirectory.",
          "i" = "Pass {.code rpkg_subdir = '<name>'} explicitly for a monorepo-aware workflow."
        ))
      }
    }
  }

  src <- system.file("templates", "r-release.yml", package = "minirextendr",
                     mustWork = TRUE)

  fs::dir_create(dest_dir, recurse = TRUE)

  if (is.null(resolved_subdir)) {
    # Standalone: copy template unchanged.
    fs::file_copy(src, dest_file, overwrite = TRUE)
  } else {
    # Monorepo: insert `working-directory: <subdir>` after configure and build steps.
    cli::cli_alert_info("Adding {.code working-directory: {resolved_subdir}} to configure/build steps")
    lines <- readLines(src, warn = FALSE)
    lines <- release_workflow_insert_workdir(lines, resolved_subdir)
    writeLines(lines, dest_file)
  }

  cli::cli_alert_success("Written {.path {dest_file}}")
  cli::cli_alert_info(c(
    "Review the template and extend it with your package's setup steps.",
    "i" = "See {.url https://a2-ai.github.io/miniextendr/manual/release-workflow/} for the rationale."
  ))

  invisible(dest_file)
}

#' Insert working-directory lines into r-release.yml lines
#'
#' Post-processes the r-release.yml template lines to add
#' `working-directory: <subdir>` after each `run: bash ./configure` and
#' `run: |` line that begins a `R CMD build` block.
#'
#' The template has two instances of each step (one per job). Both are patched.
#'
#' @param lines Character vector of template lines.
#' @param subdir Subdirectory name (e.g. `"rpkg"`).
#' @return Modified character vector.
#' @noRd
release_workflow_insert_workdir <- function(lines, subdir) {
  # Insert `working-directory:` *before* the matched `run:` line. Inserting
  # after breaks the build step: `run: |` starts a block scalar that would
  # absorb a following same-indent line as text, leaving the step with no
  # actual `run` value. Same-level sibling keys must precede the block scalar.
  wd_line <- paste0("        working-directory: ", subdir)
  result <- character(0)
  i <- 1L
  while (i <= length(lines)) {
    line <- lines[[i]]
    # Match the two configure/build `run:` patterns used in the template.
    # Pattern 1: `        run: bash ./configure`  (single-line run)
    # Pattern 2: `        run: |`                 (multi-line run starting R CMD build)
    is_configure_step <- grepl("^        run: bash \\./configure\\s*$", line)
    is_build_step <- grepl("^        run: \\|\\s*$", line) &&
      i < length(lines) && grepl("R CMD build", lines[[i + 1L]])
    if (is_configure_step || is_build_step) {
      result <- c(result, wd_line)
    }
    result <- c(result, line)
    i <- i + 1L
  }
  result
}
