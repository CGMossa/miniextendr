# Diagnostic functions for miniextendr project health

#' Run comprehensive miniextendr project diagnostics
#'
#' The primary diagnostic function for miniextendr projects. Checks the
#' health of a miniextendr project, including toolchain availability,
#' vendored crate status, generated file freshness, and common
#' configuration mistakes.
#'
#' When the project is inside a git work tree, the doctor also flags
#' configure-/install-generated files (`src/rust/.cargo/config.toml`,
#' `src/Makevars`, `*-wrappers.R`, `src/rust/wasm_registry.rs`) that are
#' tracked in git. The scaffold's `.gitignore` keeps these out of version
#' control, but a `.gitignore` only affects *untracked* files, so packages
#' scaffolded before the pattern fix in #1226 may have committed them --
#' producing VCS churn on every `bash ./configure` and potentially leaking
#' one machine's install-mode configuration into the repository. The remedy
#' is `git rm --cached <path>` (removes the file from the index, keeps it on
#' disk); the doctor only advises and never modifies the git index. This
#' check is skipped silently when git is unavailable or there is no
#' enclosing git repository (e.g. an extracted source tarball or CRAN's
#' build farm).
#'
#' The NAMESPACE section also flags stale-export drift: explicit `export()`
#' directives with no backing definition in the union of top-level objects
#' defined in `R/` sources (including the generated `R/<pkg>-wrappers.R`)
#' and `importFrom()` names (re-exports). After a `#[miniextendr]` function
#' is removed or renamed, `pkgload::load_all()` and `testthat` merely warn
#' on the superset NAMESPACE and proceed, so the drift stays invisible in
#' the dev loop until `R CMD check` or `library()` fails far from the cause;
#' [miniextendr_build()] reconciles it. The comparison is fully static (no
#' package code is loaded or executed). `exportPattern()` name-sets are not
#' expanded, `S3method()` / `exportMethods()` / `exportClasses()` directives
#' are out of scope, and the check skips when the wrappers file has not been
#' generated yet or when a whole-package `import()` directive makes exports
#' statically unattributable.
#'
#' For more targeted checks, see [miniextendr_status()] (file presence)
#' and [miniextendr_validate()] (configuration correctness).
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param webr Also lint namespace-level imports for webR compatibility (see
#'   [miniextendr_webr_import_lint()]). Off by default so packages that never
#'   target webR see no noise.
#' @return Invisibly returns a list with `pass`, `warn`, and `fail` entries.
#' @export
miniextendr_doctor <- function(path = ".", webr = FALSE) {
  with_project(path)
  cli::cli_h1("miniextendr doctor")

  results <- list(pass = character(), warn = character(), fail = character())

  # -- Toolchain checks --
  cli::cli_h2("Toolchain")

  # Rust
  rustc <- Sys.which("rustc")
  if (nzchar(rustc)) {
    version <- tryCatch(
      run_command("rustc", c("--version"))[1],
      error = function(e) "unknown"
    )
    cli::cli_alert_success("Rust: {version}")
    results$pass <- c(results$pass, "Rust installed")
  } else {
    cli::cli_alert_danger("Rust not found")
    results$fail <- c(results$fail, "Rust not found - install from https://rustup.rs")
  }

  # Cargo
  cargo <- Sys.which("cargo")
  if (nzchar(cargo)) {
    cli::cli_alert_success("cargo available")
    results$pass <- c(results$pass, "cargo available")
  } else {
    cli::cli_alert_danger("cargo not found")
    results$fail <- c(results$fail, "cargo not found")
  }

  # autoconf
  autoconf <- Sys.which("autoconf")
  if (nzchar(autoconf)) {
    cli::cli_alert_success("autoconf available")
    results$pass <- c(results$pass, "autoconf available")
  } else {
    cli::cli_alert_warning("autoconf not found (needed for configure.ac changes)")
    results$warn <- c(results$warn, "autoconf not found")
  }

  # R development headers
  r_home <- R.home()
  r_include <- file.path(r_home, "include", "R.h")
  if (file.exists(r_include)) {
    cli::cli_alert_success("R development headers present")
    results$pass <- c(results$pass, "R headers present")
  } else {
    cli::cli_alert_danger("R development headers not found at {.path {r_include}}")
    results$fail <- c(results$fail, "R development headers missing")
  }

  # -- Cargo.toml check --
  cli::cli_h2("Cargo.toml")
  cargo_toml_path <- tryCatch(usethis::proj_path("src", "rust", "Cargo.toml"), error = function(e) NULL)
  if (is.null(cargo_toml_path) || !file.exists(cargo_toml_path)) {
    cli::cli_alert_danger("{.path src/rust/Cargo.toml} not found")
    results$fail <- c(results$fail, "src/rust/Cargo.toml not found")
  } else {
    cargo_contents <- readLines(cargo_toml_path, warn = FALSE)
    if (any(grepl("miniextendr-api", cargo_contents, fixed = TRUE))) {
      cli::cli_alert_success("{.path src/rust/Cargo.toml} declares {.code miniextendr-api}")
      results$pass <- c(results$pass, "Cargo.toml declares miniextendr-api")
    } else {
      cli::cli_alert_danger("{.path src/rust/Cargo.toml} is missing {.code miniextendr-api} dependency")
      results$fail <- c(results$fail, "Cargo.toml missing miniextendr-api")
    }

    # Check for relative path = ... entries in [dependencies] only.
    # [patch.crates-io] relative paths are written by use_vendor_lib() and are
    # intentional -- do NOT flag them.
    rel_path_deps <- parse_relative_path_deps(cargo_contents)
    if (length(rel_path_deps) > 0L) {
      for (dep_info in rel_path_deps) {
        cli::cli_alert_warning(
          paste0(
            "{.path src/rust/Cargo.toml} {.code [dependencies]} entry ",
            "{.val {dep_info$crate}} uses a relative {.code path = {dep_info$path}}. ",
            "This will break under {.code R CMD INSTALL} (path resolves against the ",
            "temp build dir, not your package root). Use an absolute path or manage ",
            "it via {.code use_vendor_lib()}."
          )
        )
        results$warn <- c(
          results$warn,
          paste0("relative path dep in [dependencies]: ", dep_info$crate)
        )
      }
    } else {
      results$pass <- c(results$pass, "no relative path deps in [dependencies]")
    }
  }

  # inst/vendor.tar.xz is gitignored and only belongs in the source tree
  # transiently (during `miniextendr_vendor()` + `R CMD build`). Its presence
  # is resolved once here (the #908 marker check below needs it); the actual
  # pass/warn/fail report happens in the single "Vendor tarball" check below
  # (#BUG6 -- this used to be reported twice, with contradictory severity).
  vendor_tarball_path <- tryCatch(
    usethis::proj_path("inst", "vendor.tar.xz"),
    error = function(e) NULL
  )

  # -- Local miniextendr override marker (#908) --
  cli::cli_h2("Local override marker")
  # .miniextendr-local is a dev-only marker written by use_local_miniextendr().
  # It is gitignored and Rbuildignored; warn loudly if still present so the
  # developer doesn't accidentally ship a package that requires a local path.
  local_marker_path <- tryCatch(
    usethis::proj_path(".miniextendr-local"),
    error = function(e) NULL
  )
  if (!is.null(local_marker_path) && file.exists(local_marker_path)) {
    local_mx_path <- tryCatch(
      trimws(readLines(local_marker_path, n = 1L, warn = FALSE)),
      error = function(e) ""
    )
    tarball_present <- !is.null(vendor_tarball_path) && file.exists(vendor_tarball_path)
    if (tarball_present) {
      # Marker is inert (tarball mode wins), but still stale -- inform rather than warn.
      cli::cli_alert_info(
        "{.path .miniextendr-local} is present but inert: tarball mode wins over the \\
local override. Remove the marker before distributing: \\
{.code unuse_local_miniextendr()}."
      )
      results$warn <- c(results$warn, ".miniextendr-local present (inert: tarball mode wins)")
    } else {
      cli::cli_alert_warning(
        "Local miniextendr override is active ({.path .miniextendr-local} = \\
{.path {local_mx_path}}). Run {.code unuse_local_miniextendr()} before \\
vendoring or distributing this package."
      )
      results$warn <- c(results$warn, ".miniextendr-local override active \u2014 run unuse_local_miniextendr() before distributing")
    }
  } else if (!is.null(local_marker_path)) {
    cli::cli_alert_success("No {.path .miniextendr-local} override marker")
    results$pass <- c(results$pass, "no .miniextendr-local override")
  }

  # -- Hand-rolled [patch."https://github.com/A2-ai/miniextendr"] in Cargo.toml (#823 workaround) --
  # Warn if the user manually added this block to src/rust/Cargo.toml; the
  # supported path is use_local_miniextendr() which writes .miniextendr-local
  # and lets configure.ac manage the patch block in .cargo/config.toml.
  if (!is.null(cargo_toml_path) && file.exists(cargo_toml_path)) {
    cargo_lines_for_patch <- readLines(cargo_toml_path, warn = FALSE)
    if (any(grepl('patch.*github.com.*A2-ai.*miniextendr', cargo_lines_for_patch))) {
      cli::cli_alert_warning(
        "{.path src/rust/Cargo.toml} contains a hand-rolled \\
{.code [patch.\"https://github.com/A2-ai/miniextendr\"]} block. \\
This is the manual workaround from #823; use \\
{.code use_local_miniextendr()} instead so configure.ac manages the \\
patch block in {.path src/rust/.cargo/config.toml}."
      )
      results$warn <- c(results$warn, "hand-rolled [patch] block in src/rust/Cargo.toml \u2014 use use_local_miniextendr()")
    }
  }

  cargo_config_path <- tryCatch(
    usethis::proj_path("src", "rust", ".cargo", "config.toml"),
    error = function(e) NULL
  )
  if (!is.null(cargo_config_path) && !file.exists(cargo_config_path)) {
    cli::cli_alert_danger(
      "{.path src/rust/.cargo/config.toml} is missing"
    )
    cli::cli_alert_info(
      "This is generated by {.code bash ./configure}. If {.path inst/vendor.tar.xz} \\
was present, a previous tarball-mode build may have deleted it via the \\
tarball-mode cleanup in Makevars. Fix: \\
{.code rm -f inst/vendor.tar.xz && bash ./configure}"
    )
    results$fail <- c(results$fail, "src/rust/.cargo/config.toml missing")
  } else if (!is.null(cargo_config_path)) {
    cli::cli_alert_success("{.path src/rust/.cargo/config.toml} present")
    results$pass <- c(results$pass, "cargo config.toml present")
  }

  # -- Generated file freshness --
  cli::cli_h2("Generated files")

  template_pairs <- list(
    list(template = "src/Makevars.in", generated = "src/Makevars")
  )

  for (pair in template_pairs) {
    template_path <- tryCatch(usethis::proj_path(pair$template), error = function(e) NULL)
    generated_path <- tryCatch(usethis::proj_path(pair$generated), error = function(e) NULL)

    if (is.null(template_path)) next

    if (!file.exists(template_path)) next

    if (!file.exists(generated_path)) {
      cli::cli_alert_warning("{.path {pair$generated}} missing (run ./configure)")
      results$warn <- c(results$warn, paste(pair$generated, "missing"))
    } else {
      tmpl_mtime <- file.mtime(template_path)
      gen_mtime <- file.mtime(generated_path)
      if (tmpl_mtime > gen_mtime) {
        cli::cli_alert_warning("{.path {pair$generated}} is stale (template is newer)")
        results$warn <- c(results$warn, paste(pair$generated, "stale"))
      } else {
        cli::cli_alert_success("{.path {pair$generated}} up to date")
        results$pass <- c(results$pass, paste(pair$generated, "fresh"))
      }
    }
  }

  # -- Tracked generated files (#1250) --
  # The scaffold's .gitignore keeps configure-/install-generated files out of
  # version control, but a .gitignore only affects UNTRACKED files. Packages
  # scaffolded before the #1226 gitignore-pattern fix (the old
  # `.cargo/config.toml` pattern was mis-anchored and never matched the
  # nested path) may have already committed src/rust/.cargo/config.toml --
  # install-mode-specific and rewritten by every `bash ./configure`, so a
  # tracked copy produces confusing VCS churn and can leak one machine's
  # local override into the repo. upgrade_miniextendr_package() cannot
  # un-track a file; only `git rm --cached` can, so the doctor advises and
  # never mutates the index. Skipped silently (NULL, no section printed)
  # when git is unavailable or there is no enclosing git work tree (CRAN's
  # offline farm, extracted tarballs) -- same spirit as the vendor-tarball
  # check's no-.git-ancestor gate below.
  tracked_generated <- tracked_generated_files(usethis::proj_get())
  if (!is.null(tracked_generated)) {
    cli::cli_h2("Tracked generated files")
    if (length(tracked_generated) == 0L) {
      cli::cli_alert_success("No generated files tracked in git")
      results$pass <- c(results$pass, "no generated files tracked in git")
    } else {
      cli::cli_alert_warning(
        "{length(tracked_generated)} generated file{?s} {?is/are} tracked in git:"
      )
      for (tracked_file in tracked_generated) {
        cli::cli_bullets(c(
          "x" = "{.path {tracked_file}} \u2014 fix: {.code git rm --cached {tracked_file}}"
        ))
      }
      cli::cli_bullets(c(
        "i" = paste0(
          "These files are regenerated by {.code bash ./configure} / ",
          "{.code R CMD INSTALL} and are install-mode- or machine-specific; ",
          "tracking them causes VCS churn and can leak one machine's local ",
          "configuration into the repository."
        ),
        "i" = paste0(
          "{.code git rm --cached} removes a file from the index but keeps ",
          "it on disk; the scaffold {.path .gitignore} (refresh with ",
          "{.code upgrade_miniextendr_package()}) then keeps it untracked."
        )
      ))
      results$warn <- c(
        results$warn,
        paste0("generated file tracked in git: ", tracked_generated)
      )
    }
  }

  # -- NAMESPACE check --
  cli::cli_h2("NAMESPACE")

  namespace_path <- tryCatch(usethis::proj_path("NAMESPACE"), error = function(e) NULL)
  if (!is.null(namespace_path) && file.exists(namespace_path)) {
    ns_content <- readLines(namespace_path, warn = FALSE)
    if (any(grepl("useDynLib", ns_content))) {
      cli::cli_alert_success("NAMESPACE contains useDynLib")
      results$pass <- c(results$pass, "useDynLib present")
    } else {
      cli::cli_alert_danger("NAMESPACE missing useDynLib directive")
      results$fail <- c(results$fail, "NAMESPACE missing useDynLib")
    }

    # -- Stale-export drift (#1304/#1305) --
    # miniextendr_build() self-heals a NAMESPACE that still lists an export
    # whose backing #[miniextendr] function was removed or renamed (#1288),
    # but pkgload::load_all() / testthat only *warn* on a superset NAMESPACE
    # ("Objects listed as exports, but not present in namespace") and
    # proceed -- a dev loop that never runs a full build carries the drift
    # green all the way to R CMD check / library(), which hard-fail far from
    # the cause. Compare explicit export() directives against the union of
    # top-level definitions in R/ (which includes the generated
    # R/<pkg>-wrappers.R) and importFrom() names (re-exports). Fully static:
    # doctor never loads or executes package code. Warn, not fail: dynamic
    # definitions (computed assign() names, makeActiveBinding()) are
    # invisible to a static scan.
    drift <- stale_namespace_exports(usethis::proj_get())
    if (identical(drift$status, "skip")) {
      if (identical(drift$reason, "no-wrappers-file")) {
        cli::cli_alert_info(
          "Stale-export check skipped: no generated {.path R/<pkg>-wrappers.R} \\
on disk yet (fresh clone or scaffold). Run {.code miniextendr_build()} to \\
generate the wrappers."
        )
      } else if (identical(drift$reason, "whole-package-import")) {
        cli::cli_alert_info(
          "Stale-export check skipped: {.code import({drift$detail})} imports a \\
whole namespace, so exports cannot be attributed statically."
        )
      } else if (identical(drift$reason, "parse-error")) {
        cli::cli_alert_info(
          "Stale-export check skipped: {.path {drift$detail}} could not be parsed."
        )
      }
    } else {
      if (isTRUE(drift$has_export_patterns)) {
        cli::cli_alert_info(
          "{.code exportPattern()} directives present: pattern-derived exports \\
are not validated (only explicit {.code export()} names are checked)."
        )
      }
      if (length(drift$stale) == 0L) {
        cli::cli_alert_success(
          "All {.code export()} directives have a backing definition"
        )
        results$pass <- c(results$pass, "no stale NAMESPACE exports")
      } else {
        cli::cli_alert_warning(
          "{length(drift$stale)} NAMESPACE export{?s} {?has/have} no backing \\
definition in {.path R/} or the generated wrappers (stale-export drift):"
        )
        for (stale_export in drift$stale) {
          cli::cli_bullets(c("x" = "{.code export({stale_export})}"))
        }
        cli::cli_bullets(c(
          "i" = paste0(
            "This typically means a {.code #[miniextendr]} function was ",
            "removed or renamed after the last {.code devtools::document()}: ",
            "{.code pkgload::load_all()} / {.code testthat} only warn on the ",
            "superset NAMESPACE and proceed, but {.code R CMD check} and ",
            "{.code library()} fail."
          ),
          "i" = paste0(
            "Fix: run {.code miniextendr_build()} (reconciles NAMESPACE and ",
            "reinstalls), or {.code devtools::document()} followed by a ",
            "reinstall."
          )
        ))
        results$warn <- c(
          results$warn,
          paste0("stale NAMESPACE export: ", drift$stale)
        )
      }
    }
  }

  # -- Vendor tarball leak --
  # inst/vendor.tar.xz is gitignored and only belongs in the source tree
  # transiently (during `miniextendr_vendor()` + `R CMD build`). This is the
  # single check for it (#BUG6 -- previously duplicated as both a hard fail
  # and a separate warning, with contradictory guidance). Severity is gated
  # on whether we are in a developer source tree (a `.git` ancestor present):
  #   - dev source tree: this is the latch leak (CLAUDE.md "The latch leak
  #     (#441)") -- a previous build's trap-clean was bypassed, and if left
  #     behind it makes the post-build Makevars cleanup delete
  #     src/rust/.cargo/ from the source tree on the next
  #     `miniextendr_build()`. Fail loudly.
  #   - no `.git` ancestor: may be intentional CRAN-prep staging or a
  #     legitimate offline install. Configure never creates the tarball; warn
  #     because the helper cannot distinguish those from a leaked build artifact.
  cli::cli_h2("Vendor tarball")

  is_dev_source_tree <- !is.null(find_root_with_file(".git", usethis::proj_get()))

  if (!is.null(vendor_tarball_path) && fs::file_exists(vendor_tarball_path)) {
    if (is_dev_source_tree) {
      cli::cli_alert_danger("{.path inst/vendor.tar.xz} is present in the source tree.")
      cli::cli_bullets(c(
        "i" = paste0(
          "This flips {.code ./configure} into offline tarball mode and hides ",
          "workspace edits. If left behind after an interrupted build, the ",
          "post-build Makevars cleanup also deletes {.path src/rust/.cargo/}, ",
          "breaking the monorepo [patch] override on the next ",
          "{.code miniextendr_build()}."
        ),
        "i" = "Run {.code miniextendr_clean_vendor_leak()} to remove it."
      ))
      results$fail <- c(results$fail, "stale inst/vendor.tar.xz in source tree")
    } else {
      cli::cli_alert_warning("{.path inst/vendor.tar.xz} is present.")
      cli::cli_bullets(c(
        "i" = paste0(
          "No {.code .git} ancestor found. If this is a dev tree, the tarball ",
          "may have leaked from an interrupted package-build workflow."
        ),
        "i" = paste0(
          "Tarball mode skips wrapper regeneration, so new ",
          "{.code #[miniextendr]} functions never become callable."
        ),
        "i" = paste0(
          "If you are staging for CRAN, this is expected. Otherwise run ",
          "{.code miniextendr_clean_vendor_leak()}."
        )
      ))
      results$warn <- c(results$warn, "inst/vendor.tar.xz present (may flip tarball mode)")
    }
  } else {
    cli::cli_alert_success("No {.path inst/vendor.tar.xz} leak detected")
    results$pass <- c(results$pass, "No vendor tarball leak")
  }

  # -- Cargo.lock shape --
  cli::cli_h2("Cargo.lock shape")

  lock_path <- tryCatch(usethis::proj_path("src", "rust", "Cargo.lock"), error = function(e) NULL)
  if (!is.null(lock_path) && fs::file_exists(lock_path)) {
    lock_lines <- readLines(lock_path, warn = FALSE)
    # Only `path+` source entries count as drift. `checksum = "..."` lines are
    # canonical post-#408 (cargo-revendor writes valid .cargo-checksum.json), so
    # they are allowed in tarball-shape and must not be flagged here.
    n_path <- sum(grepl('^source = "path\\+', lock_lines))

    if (n_path > 0L) {
      cli::cli_alert_warning(
        "{.path src/rust/Cargo.lock} has drifted into source-shape ({n_path} {.code path+} source entr{?y/ies})."
      )
      cli::cli_alert_info(
        "Run {.code miniextendr_repair_lock()} to restore tarball-shape."
      )
      results$warn <- c(results$warn, "Cargo.lock drifted into source-shape")
    } else {
      cli::cli_alert_success("{.path src/rust/Cargo.lock} is in tarball-shape")
      results$pass <- c(results$pass, "Cargo.lock in tarball-shape")
    }
  }

  # -- Git Hooks --
  cli::cli_h2("Git Hooks")

  hook_status <- has_miniextendr_git_hooks(usethis::proj_get())
  if (all(hook_status)) {
    cli::cli_alert_success("miniextendr git hooks installed (pre-commit, post-merge)")
    results$pass <- c(results$pass, "git hooks installed")
  } else {
    missing <- names(hook_status)[!hook_status]
    cli::cli_alert_warning("Missing miniextendr git hooks: {paste(missing, collapse = ', ')}")
    cli::cli_alert_info("Run {.code minirextendr::use_miniextendr_git_hooks()} to install")
    results$warn <- c(results$warn, "git hooks missing")
  }

  # -- webR namespace imports (opt-in) --
  if (isTRUE(webr)) {
    cli::cli_h2("webR namespace imports")
    webr_results <- webr_report_findings(
      webr_import_findings(usethis::proj_get())
    )
    results$pass <- c(results$pass, webr_results$pass)
    results$warn <- c(results$warn, webr_results$warn)
    results$fail <- c(results$fail, webr_results$fail)
  }

  # -- Summary --
  cli::cli_h2("Summary")
  cli::cli_alert_success("{length(results$pass)} passed")
  if (length(results$warn) > 0) {
    cli::cli_alert_warning("{length(results$warn)} warning(s)")
  }
  if (length(results$fail) > 0) {
    cli::cli_alert_danger("{length(results$fail)} failure(s)")
    for (f in results$fail) {
      cli::cli_bullets(c("x" = f))
    }
  }

  invisible(results)
}

# Parse src/rust/Cargo.toml lines and return a list of list(crate, path) for
# each [dependencies] entry that has a relative path = "..." value.
#
# Rules:
#   - Only entries inside a [dependencies] section are checked.
#   - [patch.crates-io] and all other sections are ignored.
#   - A path is relative when it does NOT start with "/" (or on Windows "X:/").
#   - Both bare `crate = { path = "..." }` inline tables and multi-line
#     dependency blocks preceded by `[dependencies.crate]` are handled.
#
# @param lines Character vector: the raw lines of Cargo.toml.
# @return A list of named lists with elements `crate` and `path`.
parse_relative_path_deps <- function(lines) {
  results <- list()

  # Track which TOML section we are in.
  # section: "deps" for [dependencies] / [dependencies.*], anything else ignored.
  section <- "other"
  current_crate <- NULL  # set when we enter [dependencies.crate_name]

  for (line in lines) {
    stripped <- trimws(line)

    # Skip blank lines and comments.
    if (nchar(stripped) == 0L || startsWith(stripped, "#")) {
      # A blank line between a multi-line dep block and the next entry doesn't
      # reset current_crate; only a new section header does.
      next
    }

    # Section header?
    if (startsWith(stripped, "[")) {
      # Match e.g. [dependencies], [dependencies.foo], [dev-dependencies], etc.
      if (grepl("^\\[dependencies\\]", stripped)) {
        section <- "deps"
        current_crate <- NULL
      } else if (grepl("^\\[dependencies\\.", stripped)) {
        section <- "deps"
        # Extract crate name from [dependencies.crate_name]
        current_crate <- sub("^\\[dependencies\\.([^]]+)\\].*", "\\1", stripped)
      } else {
        # Any other section (including [patch.crates-io]) -- stop watching.
        section <- "other"
        current_crate <- NULL
      }
      next
    }

    if (section != "deps") next

    # Inline table: `crate_name = { ..., path = "...", ... }`
    # e.g. my-crate = { path = "../my-crate" }
    #      my-crate = { version = "1.0", path = "../my-crate", features = [] }
    inline_match <- regmatches(
      stripped,
      regexpr(
        '^([A-Za-z0-9_-]+)\\s*=\\s*\\{[^}]*path\\s*=\\s*"([^"]*)"',
        stripped,
        perl = TRUE
      )
    )
    if (length(inline_match) > 0L && nchar(inline_match) > 0L) {
      m <- regmatches(
        stripped,
        regexec(
          '^([A-Za-z0-9_-]+)\\s*=\\s*\\{[^}]*path\\s*=\\s*"([^"]*)"',
          stripped,
          perl = TRUE
        )
      )[[1L]]
      crate <- m[[2L]]
      path  <- m[[3L]]
      if (!is_absolute_path(path)) {
        results <- c(results, list(list(crate = crate, path = path)))
      }
      next
    }

    # Multi-line block: we entered via [dependencies.crate_name] and now see
    # a line like `path = "..."`.
    if (!is.null(current_crate)) {
      path_match <- regmatches(
        stripped,
        regexec('^path\\s*=\\s*"([^"]*)"', stripped, perl = TRUE)
      )[[1L]]
      if (length(path_match) >= 2L) {
        path <- path_match[[2L]]
        if (!is_absolute_path(path)) {
          results <- c(results, list(list(crate = current_crate, path = path)))
        }
      }
    }
  }

  results
}

# Returns TRUE when path is absolute (starts with "/" or a Windows drive letter).
is_absolute_path <- function(path) {
  grepl("^(/|[A-Za-z]:[/\\\\])", path)
}

#' Git pathspecs for the configure-/install-generated files that the scaffold
#' `.gitignore` keeps out of version control (see
#' `inst/templates/rpkg/gitignore`). Only the mode-/machine-specific files
#' belong here, not build artifacts (`*.o`, `*.so`, target dirs) or
#' `inst/vendor.tar.xz`, which has its own dedicated doctor check. Keep the
#' primary real-world offender (`src/rust/.cargo/config.toml`, #1226/#1250)
#' first so it leads the report.
#' @noRd
MX_GENERATED_GITIGNORED_PATHSPECS <- c(
  "src/rust/.cargo/config.toml",
  "src/Makevars",
  "src/*-wrappers.R",
  "R/*-wrappers.R",
  "src/rust/wasm_registry.rs"
)

#' Generated files that are tracked in git
#'
#' A `.gitignore` only affects untracked files: packages scaffolded before the
#' #1226 pattern fix (the old `.cargo/config.toml` pattern was mis-anchored
#' and never matched the nested path) may have `git add`ed generated files
#' before the corrected pattern arrived, and `upgrade_gitignore()` cannot
#' un-track them. Runs a single batched `git ls-files` over every pathspec in
#' `MX_GENERATED_GITIGNORED_PATHSPECS` so all offenders are collected in one
#' pass rather than bailing at the first.
#'
#' `git ls-files` reads the index, so a file counts as tracked from the moment
#' it is `git add`ed, committed or not -- exactly the set that
#' `git rm --cached` acts on. Read-only: never mutates the index.
#'
#' @param proj_dir Package directory to inspect. Git runs *in* this directory
#'   (via `run_command(wd = )`, not the caller's working directory -- same
#'   rationale as `check_scaffolding_clean()`), so the relative pathspecs
#'   resolve against the package root and the reported paths come back
#'   relative to it, ready to paste into `git rm --cached`.
#' @return Character vector of tracked generated paths relative to `proj_dir`;
#'   `character(0)` when the check ran and found none. `NULL` when the check
#'   cannot run at all: git missing from PATH, or `proj_dir` not inside a git
#'   work tree (CRAN's offline farm, an extracted source tarball, or a test
#'   fixture's bare `.git` stub directory).
#' @noRd
tracked_generated_files <- function(proj_dir = usethis::proj_get()) {
  if (!nzchar(Sys.which("git"))) {
    return(NULL)
  }

  # Not in a git work tree: skip silently. `git rev-parse` exits non-zero
  # both outside any repo and for a fake/corrupt `.git` directory.
  probe <- run_command("git", c("rev-parse", "--show-toplevel"), wd = proj_dir)
  if (!is.null(attr(probe, "status"))) {
    return(NULL)
  }

  # `git ls-files -- <pathspecs>` lists each TRACKED file matching the
  # pathspecs (glob patterns included) and exits 0 whether or not anything
  # matched, so one invocation batches the whole sweep.
  tracked <- run_command(
    "git",
    c("ls-files", "--", MX_GENERATED_GITIGNORED_PATHSPECS),
    wd = proj_dir
  )
  if (!is.null(attr(tracked, "status"))) {
    return(NULL)
  }
  tracked[nzchar(tracked)]
}

#' Stale NAMESPACE exports (#1304/#1305)
#'
#' Compares the package's explicit `export()` directives against the union of
#' (a) top-level objects defined in `R/` sources -- which includes the
#' generated `R/<pkg>-wrappers.R`, since it lives in `R/` -- and (b) names
#' brought in by `importFrom()` directives (roxygen re-exports are
#' `importFrom()` + `export()` pairs, so imported names are legitimate
#' backing). Everything is derived statically (`parse()` +
#' `parseNamespaceFile()`); the package is never loaded or executed, so the
#' check is side-effect-free and safe to run on a broken tree.
#'
#' Directive semantics (deliberate):
#' - Only plain `export()` names are checked. Each must resolve to an object
#'   in the namespace at `loadNamespace()` time, so a name with no backing
#'   definition is a guaranteed `library()` / `R CMD check` failure.
#' - `exportPattern()` patterns are not expanded: they only *add* exports
#'   (whatever happens to match at load time), so they cannot rescue a
#'   missing explicit export, and expanding them statically would guess.
#'   Their presence is surfaced via `has_export_patterns` so the doctor can
#'   note that pattern-derived exports go unvalidated.
#' - `S3method()` registers methods rather than exporting objects, and
#'   `exportMethods()` / `exportClasses()` (S4) are backed by `setMethod()` /
#'   `setClass()` side effects, not top-level assignments -- a static
#'   assignment scan would false-positive them, so they are out of scope.
#' - A whole-package `import(pkg)` makes every export statically
#'   unattributable (any name might come from `pkg`), so the check skips
#'   rather than enumerate a dependency's exports (which would require
#'   loading its namespace).
#'
#' The generated wrappers file is gitignored in the scaffold: a fresh clone
#' legitimately has NAMESPACE exports for every Rust wrapper but no
#' `R/<pkg>-wrappers.R` on disk until the first build regenerates it. Running
#' the comparison there would flag every Rust-backed export, so the check
#' skips until the wrappers file exists.
#'
#' @param pkg_dir Package directory to inspect.
#' @return `list(status = "ok", stale = <chr>, has_export_patterns = <flag>)`
#'   when the check ran (`stale` empty = clean). `list(status = "skip",
#'   reason = <chr>, detail = <chr or NULL>)` when it cannot run without
#'   guessing; `reason` is one of `"no-namespace"`, `"no-wrappers-file"`,
#'   `"whole-package-import"` (`detail` = the imported package), or
#'   `"parse-error"` (`detail` = the unparseable file, relative to
#'   `pkg_dir`).
#' @noRd
stale_namespace_exports <- function(pkg_dir = usethis::proj_get()) {
  skip <- function(reason, detail = NULL) {
    list(status = "skip", reason = reason, detail = detail)
  }

  if (!file.exists(file.path(pkg_dir, "NAMESPACE"))) {
    return(skip("no-namespace"))
  }
  if (!wrappers_file_exists(pkg_dir)) {
    return(skip("no-wrappers-file"))
  }

  ns <- tryCatch(
    parseNamespaceFile(basename(pkg_dir), dirname(pkg_dir)),
    error = function(e) NULL
  )
  if (is.null(ns)) {
    return(skip("parse-error", detail = "NAMESPACE"))
  }

  has_export_patterns <- length(ns$exportPatterns) > 0L

  imported <- character()
  for (entry in ns$imports) {
    # parseNamespaceFile() shapes (see also webr_import_findings()):
    # bare character = import(pkg); list with an `except` element =
    # import(pkg, except = ...); otherwise list(pkg, names) = importFrom().
    if (is.character(entry) || "except" %in% names(entry)) {
      return(skip("whole-package-import", detail = entry[[1L]]))
    }
    imported <- c(imported, as.character(entry[[2L]]))
  }

  r_dir <- file.path(pkg_dir, "R")
  r_files <- if (dir.exists(r_dir)) {
    # .R/.r/.S/.s/.q are the code extensions R CMD INSTALL collates (WRE 1.1.5).
    list.files(r_dir, pattern = "\\.[RrSsq]$", full.names = TRUE)
  } else {
    character()
  }
  scan <- r_top_level_definitions(r_files)
  if (length(scan$failed) > 0L) {
    return(skip(
      "parse-error",
      detail = file.path("R", basename(scan$failed[[1L]]))
    ))
  }

  list(
    status = "ok",
    stale = setdiff(unique(as.character(ns$exports)), c(scan$names, imported)),
    has_export_patterns = has_export_patterns
  )
}

#' Top-level object definitions in R source files
#'
#' Statically collects the names a set of R source files define at top level,
#' without evaluating any code: `<-` / `=` / `<<-` assignments to a symbol or
#' string literal (functions, operators, replacement functions, plain
#' objects), literal-name `assign()` / `delayedAssign()` calls, and
#' `setGeneric("name", ...)` (which creates an object of that name).
#' Top-level `if` branches and `{` blocks are descended into --
#' `loadNamespace()` evaluates whichever branch applies at load time, and
#' taking the optimistic union of all branches avoids false positives.
#' Dynamic definitions (computed names, `makeActiveBinding()`) are invisible
#' to this scan -- one reason the doctor reports stale exports as warnings,
#' not failures.
#'
#' @param r_files Character vector of file paths to parse.
#' @return `list(names = <chr>, failed = <chr>)`: the unique defined names,
#'   and any files that could not be parsed (syntax errors).
#' @noRd
r_top_level_definitions <- function(r_files) {
  defined <- character()
  failed <- character()

  add_name <- function(x) {
    if (is.name(x)) {
      defined[[length(defined) + 1L]] <<- as.character(x)
    } else if (is.character(x) && length(x) == 1L && !is.na(x)) {
      defined[[length(defined) + 1L]] <<- x
    }
  }

  walk <- function(e) {
    if (!is.call(e) || !is.name(e[[1L]])) {
      return(invisible(NULL))
    }
    op <- as.character(e[[1L]])
    if (op %in% c("<-", "=", "<<-") && length(e) == 3L) {
      # `x -> y` parses as `y <- x`, so right-assign is covered too.
      add_name(e[[2L]])
    } else if (op %in% c("assign", "delayedAssign", "setGeneric") &&
      length(e) >= 2L) {
      # Positional literal first argument only; a computed name is a dynamic
      # definition this scan deliberately cannot see.
      add_name(e[[2L]])
    } else if (op == "if") {
      if (length(e) >= 3L) walk(e[[3L]])
      if (length(e) >= 4L) walk(e[[4L]])
    } else if (op == "{") {
      for (i in seq_len(length(e) - 1L)) walk(e[[i + 1L]])
    }
    invisible(NULL)
  }

  for (f in r_files) {
    exprs <- tryCatch(
      parse(f, keep.source = FALSE, encoding = "UTF-8"),
      error = function(e) NULL
    )
    if (is.null(exprs)) {
      failed <- c(failed, f)
      next
    }
    for (e in exprs) walk(e)
  }

  list(names = unique(defined), failed = failed)
}
