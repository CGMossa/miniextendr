# Workflow helper functions

#' Run autoconf to generate configure script
#'
#' Runs `autoconf -vif` in the package root to regenerate the configure
#' script from configure.ac. Requires autoconf to be installed.
#'
#' @param path Path to the R package root, or `NULL` to use the active project.
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_autoconf <- function(path = ".") {
  with_project(path)
  check_autoconf()

  cli::cli_alert("Running autoconf...")

  result <- run_with_logging(
    "autoconf",
    args = c("-v", "-i", "-f"),
    log_prefix = "autoconf",
    wd = usethis::proj_get()
  )

  check_result(result, "autoconf")

  # Make configure executable
  configure_path <- usethis::proj_path("configure")
  if (fs::file_exists(configure_path)) {
    fs::file_chmod(configure_path, "755")
    cli::cli_alert_success("Generated {.path configure}")
  }

  invisible(TRUE)
}

#' Run configure script
#'
#' Runs `./configure` in the package root to generate Makevars,
#' Cargo.toml, and other build files from templates.
#'
#' @param path Path to the R package root, or `NULL` to use the active project.
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_configure <- function(path = ".") {
  with_project(path)
  configure_path <- usethis::proj_path("configure")

  if (!fs::file_exists(configure_path)) {
    cli::cli_abort(c(
      "configure script not found",
      "i" = "Run {.code minirextendr::miniextendr_autoconf()} first"
    ))
  }

  # Ensure configure is executable
  perms <- fs::file_info(configure_path)$permissions
  if (!grepl("x", as.character(perms))) {
    cli::cli_alert_info("Making {.path configure} executable")
    fs::file_chmod(configure_path, "755")
  }

  cli::cli_alert("Running ./configure...")

  result <- run_with_logging(
    "bash",
    args = c("./configure"),
    log_prefix = "configure",
    wd = usethis::proj_get(),
    env = if (requireNamespace("devtools", quietly = TRUE)) devtools::r_env_vars() else character()
  )

  check_result(result, "./configure")

  # Also mention config.log if it exists
  config_log <- usethis::proj_path("config.log")
  if (fs::file_exists(config_log)) {
    cli::cli_alert_info("Configure log also saved to: {.path config.log}")
  }

  cli::cli_alert_success("Generated build files")
  invisible(TRUE)
}

#' Full R package build workflow
#'
#' Runs the complete R package build pipeline:
#' autoconf -> configure -> `R CMD INSTALL` (compiles Rust + generates the
#' `R/<pkg>-wrappers.R` file via the wrapper-gen pass) -> roxygen2 -> conditional
#' reinstall. This is the high-level workflow for building the entire package;
#' for compiling just the Rust crate, use [cargo_build()] instead.
#'
#' @section Why a conditional reinstall:
#' The `R/<pkg>-wrappers.R` file is generated *during* install, and its roxygen
#' `@export` tags are what [devtools::document()] reads to write `NAMESPACE`.
#' That creates a chicken-and-egg ordering: `document()` can only see the
#' wrappers after install, but install collates `NAMESPACE` (the export set the
#' installed image actually exposes) *before* `document()` rewrites it. On a
#' first build -- or any build that adds or renames an exported function -- the
#' freshly-installed image therefore lags the on-disk `NAMESPACE` by one build,
#' and `library(pkg)` exposes nothing new until the package is built a second
#' time.
#'
#' To collapse that into a single pass, this workflow snapshots `NAMESPACE`
#' before and after `document()`. If `document()` changed it (new or renamed
#' exports), the package is reinstalled once so the installed image matches the
#' freshly-written `NAMESPACE`. The reinstall happens at most once: after it,
#' the wrappers and `NAMESPACE` are already in their final form, so a repeat
#' `document()` would be a fixpoint and no further install is needed.
#'
#' @section Removal/rename self-heal:
#' The addition case above assumes the install step succeeds. It doesn't when
#' an exported `#[miniextendr]` function is *removed or renamed*: the on-disk
#' `NAMESPACE` is then a superset of the freshly regenerated wrappers, and
#' `R CMD INSTALL`'s test-load aborts with `undefined exports: <old_name>`
#' before `document()` gets a chance to drop the stale entry (#1288).
#'
#' To self-heal this, the install step's failure is caught and deferred (with
#' a loud warning) rather than propagated immediately. `document()` still
#' runs and reconciles `NAMESPACE` down to the new export set --
#' `pkgload`'s `setup_ns_exports()` only warns on a `NAMESPACE` superset and
#' proceeds, so `document()` survives it even though the installed image
#' can't. The reinstall is then made **mandatory** whenever the install step
#' was deferred, even if `document()` left `NAMESPACE` textually unchanged --
#' a digest-only check would otherwise skip the reinstall and leave the
#' broken image in place. No error classification is performed: any Step 3
#' failure is deferred and retried once, because a genuinely broken build
#' re-fails identically at the mandatory retry (or earlier, since
#' `document()`'s `pkgload::load_all(compile = NA)` recompiles the crate
#' too) and errors loudly there. The invariant holds regardless:
#' `miniextendr_build(install = TRUE)` returns `TRUE` only if the last
#' install attempt succeeded with test-load.
#'
#' @section Mid-build source-tree restore:
#' The install step's `R CMD build` runs the scaffolded `bootstrap.R` in the
#' source tree, sealing `inst/vendor.tar.xz` there by design (the built
#' tarball must carry it). Left in place, that latch would flip the rest of
#' the build into tarball mode -- where wrapper regeneration is skipped -- so
#' `document()` would reconcile `NAMESPACE` against stale wrappers and both
#' self-heal paths above would break for any post-first-build export change
#' (#1294). `miniextendr_build()` therefore restores the dev source tree
#' (manifest/lockfile snapshot written back; a newly-sealed
#' `inst/vendor.tar.xz` deleted) immediately after the install step, not
#' only on exit. A latch that existed *before* the build is never deleted --
#' instead a warning is emitted up front, because a latched tree builds in
#' tarball mode throughout and the dev-loop self-heal is structurally
#' disabled; delete `inst/vendor.tar.xz` (or run `minirextendr_doctor()`) to
#' resume source-mode development.
#'
#' @section Fresh-package bootstrap:
#' A brand-new package has no generated `R/<pkg>-wrappers.R` yet. The wrappers
#' are produced by the wrapper-gen pass during install, but a plain
#' `devtools::install(build = TRUE)` cannot bootstrap them: its `R CMD build`
#' step runs `bootstrap.R`, which vendors into `inst/vendor.tar.xz` as part of
#' producing the package tarball, and
#' that latch flips `./configure` into offline *tarball* mode -- which ships
#' pre-generated wrappers and skips wrapper generation. With no wrappers to
#' ship, the install either fails loudly ("tarball is missing pre-generated
#' wrappers") or, worse, leaves the namespace empty.
#'
#' When `miniextendr_build()` detects that the wrappers file is absent, it
#' first runs a bootstrap (clear any stale latch -> configure ->
#' `devtools::install(build = FALSE, MINIEXTENDR_FORCE_WRAPPER_GEN=1)` ->
#' `devtools::document()`) so the wrappers exist before the normal build path
#' runs. The `MINIEXTENDR_FORCE_WRAPPER_GEN` override also protects this helper
#' from a pre-existing tarball-mode configuration while it establishes the
#' source-mode wrapper set. Once wrappers are present, the normal build path
#' runs unchanged.
#'
#' @param path Path to the R package root, or `NULL` to use the active project.
#' @param install Whether to run the `R CMD INSTALL` steps. If `FALSE`, only
#'   runs autoconf + configure + roxygen2 (no compile, no reinstall).
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_build <- function(path = ".", install = TRUE) {
  with_project(path)
  cli::cli_h1("miniextendr build workflow")

  pkg_path <- usethis::proj_get()
  has_devtools <- requireNamespace("devtools", quietly = TRUE)

  # A build = TRUE install (Step 3/5 below) runs the package's bootstrap.R in the
  # source tree, which vendors when no inst/vendor.tar.xz is present: that
  # FREEZES src/rust/Cargo.toml (rewriting a `path = "../../../my-core"` sibling
  # to `vendor/my-core`) and leaves inst/vendor.tar.xz behind -- flipping the tree
  # into tarball mode and stranding the next source-mode build. The dev loop must
  # leave a clean source tree, so snapshot the manifest/lock + tarball presence
  # now and restore (mirrors the `just cran-prep` trap, git-independent).
  #
  # #1294: the restore must ALSO run mid-build, right after Step 3 -- not only
  # at function exit. bootstrap.R seals the latch into the SOURCE tree by
  # design (the built tarball must carry inst/vendor.tar.xz), so after a
  # build = TRUE install Steps 4/5 would otherwise run latched in tarball
  # mode: Step 4's compile_dll re-runs ./configure, which resolves tarball
  # mode and SKIPS wrapper regeneration (the Makevars #1022 guard), leaving
  # wrappers.R stale and NAMESPACE unreconciled -- breaking both the additive
  # (#860) and the removal/rename (#1288) self-heal for any post-first-build
  # export change. Hence the restore body lives in a local closure, called
  # both inline after the install block and from on.exit (idempotent
  # backstop). It never touches vendor/ or .cargo/: configure owns
  # .cargo/config.toml, and vendor/ may be user-provisioned (offline
  # crates.io deps).
  rust_manifest <- fs::path(pkg_path, "src", "rust", "Cargo.toml")
  rust_lock <- fs::path(pkg_path, "src", "rust", "Cargo.lock")
  vendor_tarball <- fs::path(pkg_path, "inst", "vendor.tar.xz")
  snap_manifest <- if (fs::file_exists(rust_manifest)) readLines(rust_manifest, warn = FALSE) else NULL
  snap_lock <- if (fs::file_exists(rust_lock)) readLines(rust_lock, warn = FALSE) else NULL
  tarball_preexisting <- fs::file_exists(vendor_tarball)
  if (tarball_preexisting) {
    # A pre-existing latch is never deleted mid-build (it may be a deliberate
    # release-prep artifact) -- but with it in place every step runs in
    # tarball mode and the #1022 guard skips wrapper regeneration, so export
    # changes never reach NAMESPACE: dev-loop self-heal is structurally
    # disabled in a latched tree. Warn loudly up front.
    cli::cli_warn(c(
      "Pre-existing {.path inst/vendor.tar.xz} latch: this tree builds in tarball mode.",
      "i" = "Tarball mode skips wrapper regeneration, so export changes will not \\
             reach {.path NAMESPACE} (dev-loop self-heal is disabled).",
      "i" = "Delete {.path inst/vendor.tar.xz} (or run {.code minirextendr_doctor()}, \\
             which detects the stale latch) to resume source-mode development."
    ))
  }
  restore_dev_tree <- function() {
    if (!is.null(snap_manifest)) writeLines(snap_manifest, rust_manifest)
    if (!is.null(snap_lock)) writeLines(snap_lock, rust_lock)
    if (!tarball_preexisting && fs::file_exists(vendor_tarball)) fs::file_delete(vendor_tarball)
    invisible(TRUE)
  }
  on.exit(restore_dev_tree(), add = TRUE)

  cli::cli_h2("Step 1: autoconf")
  miniextendr_autoconf()

  cli::cli_h2("Step 2: configure")
  miniextendr_configure()

  # Deferred Step-3 install failure (removal/rename case, #1288). Initialised
  # before the `if (install)` block so the Step-5 condition below can read it
  # even when install = FALSE or devtools is unavailable (both leave it NULL,
  # so Step 5's retry never fires in those paths).
  step3_error <- NULL

  if (install) {
    if (!has_devtools) {
      cli::cli_h2("Step 3: install (compile Rust + generate R wrappers)")
      cli::cli_warn("devtools not installed, skipping install step")
    } else {
      # Fresh-package bootstrap (#822). On a brand-new package the generated
      # R/<pkg>-wrappers.R does not exist yet. The normal devtools::install()
      # below uses build = TRUE, whose R CMD build step runs bootstrap.R, which
      # vendors inst/vendor.tar.xz while producing the package tarball and flips
      # ./configure into tarball mode
      # -- a mode that SKIPS wrapper generation. So the very build that should
      # have created the wrappers can't, and library() exposes nothing. Detect
      # this and generate the wrappers first via an in-place source-mode install
      # (build = FALSE), which never touches inst/vendor.tar.xz.
      if (!wrappers_file_exists(pkg_path)) {
        cli::cli_h2("Step 3a: bootstrap wrappers (fresh package, source mode)")
        bootstrap_fresh_wrappers(pkg_path)
      }

      cli::cli_h2("Step 3: install (compile Rust + generate R wrappers)")
      # Removal/rename case (#1288): if an exported #[miniextendr] function was
      # removed or renamed since the last build, the on-disk NAMESPACE is a
      # superset of the freshly regenerated wrappers, and R CMD INSTALL's
      # test-load aborts with "undefined exports: <old_name>" -- before
      # document() gets a chance to drop the stale entry. Defer any Step-3
      # failure (no error classification: the message shape is version-fragile
      # and unconditional deferral is safe -- see roxygen section below) and
      # let Step 4's document() reconcile NAMESPACE, then force a Step-5 retry.
      tryCatch(
        {
          install_pkg(pkg_path)
          cli::cli_alert_success("Installed package")
        },
        error = function(e) step3_error <<- e
      )
      if (!is.null(step3_error)) {
        cli::cli_warn(c(
          "Install failed before NAMESPACE reconciliation; deferring.",
          "i" = "Expected when an exported #[miniextendr] function was removed or \\
                 renamed: the stale NAMESPACE still lists the old export and \\
                 R CMD INSTALL's load test aborts with 'undefined exports' (#1288).",
          "i" = "Continuing to the document step, then retrying the install once.",
          "x" = conditionMessage(step3_error)
        ))
      }
    }
  }

  # #1294: restore the dev source tree NOW, before Step 4 -- Step 3's
  # build = TRUE install ran bootstrap.R in the source tree, sealing
  # inst/vendor.tar.xz there. With the latch still present, Step 4's
  # compile_dll configure would resolve tarball mode and skip wrapper
  # regeneration (the #1022 guard), so document() would reconcile NAMESPACE
  # against STALE wrappers and both self-heal paths (#860 additive, #1288
  # removal/rename) would break for post-first-build export changes. With
  # the latch gone, configure re-resolves source mode, rewrites
  # .cargo/config.toml itself, and wrapper regen runs against the un-frozen
  # manifest. Step 5's install re-runs bootstrap.R (one extra cargo-revendor
  # per export-changing build); the on.exit backstop cleans up at exit.
  # No-op when nothing was sealed (install = FALSE, no devtools,
  # bootstrap-only) and never deletes a pre-existing latch.
  restore_dev_tree()

  cli::cli_h2("Step 4: roxygen2 (update NAMESPACE + man pages)")
  if (!has_devtools) {
    cli::cli_warn("devtools not installed, skipping roxygen2 step")
  } else {
    namespace_path <- fs::path(pkg_path, "NAMESPACE")
    namespace_before <- namespace_digest(namespace_path)

    devtools::document(pkg_path)
    cli::cli_alert_success("Updated NAMESPACE and documentation")

    namespace_after <- namespace_digest(namespace_path)

    # Chicken-and-egg fix: the wrappers file is generated during install and
    # document() reads its @export tags to write NAMESPACE. So the install in
    # Step 3 collated the *previous* NAMESPACE -- if document() just added or
    # renamed exports, the installed image is one build behind. Reinstall once
    # so library(pkg) exposes the new exports in a single miniextendr_build()
    # pass. The reinstall is bounded to one pass: after it the wrappers and
    # NAMESPACE are already final, so re-running document() would be a fixpoint.
    #
    # #1288: the same reinstall is also forced whenever Step 3 was deferred
    # (step3_error set), even if document() left NAMESPACE textually unchanged.
    # That covers the removal/rename case: Step 3 failed because the on-disk
    # NAMESPACE was a superset of the regenerated wrappers (stale export from a
    # removed/renamed #[miniextendr] fn); document() reconciles NAMESPACE down
    # to the new export set (pkgload's setup_ns_exports only warns on the
    # superset, so document() itself survives it), but the *installed* image is
    # still the broken one from before Step 3's failure -- so a digest-only
    # check would wrongly skip Step 5 here. Making the retry mandatory on any
    # deferral keeps the invariant: miniextendr_build(install = TRUE) returns
    # TRUE only if the last install attempt succeeded with test-load.
    if (install && has_devtools && (!identical(namespace_before, namespace_after) || !is.null(step3_error))) {
      if (!is.null(step3_error)) {
        cli::cli_h2("Step 5: reinstall (retrying install after NAMESPACE reconciliation)")
        cli::cli_alert_info(
          "Step 3's install was deferred; retrying now that {.code document()} has \\
           reconciled {.path NAMESPACE} against the regenerated wrappers."
        )
      } else {
        cli::cli_h2("Step 5: reinstall (NAMESPACE exports changed)")
        cli::cli_alert_info(
          "{.code document()} changed {.path NAMESPACE}; reinstalling so the \\
           installed image exports the new wrappers."
        )
      }
      install_pkg(pkg_path)
      cli::cli_alert_success("Reinstalled against updated NAMESPACE")
    }
  }

  cli::cli_alert_success("Build complete!")
  invisible(TRUE)
}

# Install the package via devtools, forcing the wrapper-gen pass.
#
# MINIEXTENDR_FORCE_WRAPPER_GEN forces regeneration of R/<pkg>-wrappers.R +
# wasm_registry.rs even if an inst/vendor.tar.xz latch has flipped configure
# into tarball mode (which otherwise skips it). Without this, a build run
# against a leaked tarball installs stale wrappers and library() exposes no
# functions. The prior value is restored on exit so the override doesn't leak
# into the rest of the R session.
#
# reload = FALSE: do NOT reload the freshly-installed package into the building
# session. The default (reload = TRUE) re-registers the package's namespace from
# its just-written installed image; the subsequent devtools::document() then runs
# pkgload::load_all(), whose unregister() step reads that installed namespace's
# lazy-load DB (R/<pkg>.rdb). On R >= 4.6 (libdeflate-compressed .rdb) that read
# can fail with "internal error 1 in R_decompress1 with libdeflate" / "lazy-load
# database is corrupt", aborting the build. The build session never needs the
# package loaded -- document() works from source -- so reloading is both pointless
# and the trigger. Skipping it makes the install -> document hand-off robust.
install_pkg <- function(pkg_path) {
  old_force <- Sys.getenv("MINIEXTENDR_FORCE_WRAPPER_GEN", unset = NA)
  Sys.setenv(MINIEXTENDR_FORCE_WRAPPER_GEN = "1")
  on.exit(
    if (is.na(old_force)) Sys.unsetenv("MINIEXTENDR_FORCE_WRAPPER_GEN")
    else Sys.setenv(MINIEXTENDR_FORCE_WRAPPER_GEN = old_force),
    add = TRUE
  )
  tryCatch(
    # reload = FALSE: never reload the .rdb-backed installed namespace into this
    # session. miniextendr_build() reinstalls over the same library path; a loaded
    # namespace holding lazy promises into a rewritten .rdb fails with "internal
    # error 1 in R_decompress1" / "lazy-load database is corrupt" when pkgload
    # later unregisters it (R 4.6.0's libdeflate backend surfaced this, #1000).
    devtools::install(pkg_path, upgrade = FALSE, quiet = FALSE, reload = FALSE),
    error = function(e) {
      cli::cli_abort(c(
        "Package installation failed",
        "i" = conditionMessage(e)
      ))
    }
  )
}

# Stable fingerprint of a NAMESPACE file for diffing before/after document().
# Returns NA_character_ when the file is absent (e.g. a brand-new package),
# which compares unequal to any real content via identical() and so triggers
# the reinstall on first build.
namespace_digest <- function(namespace_path) {
  if (!fs::file_exists(namespace_path)) {
    return(NA_character_)
  }
  paste(readLines(namespace_path, warn = FALSE), collapse = "\n")
}

#' Does the package's generated R wrapper file exist yet?
#'
#' The wrapper-gen pass writes `R/<pkg>-wrappers.R`; its presence is the signal
#' that the package has been bootstrapped at least once. A fresh scaffold
#' has the Rust sources but no wrappers file.
#'
#' @param pkg_path Absolute path to the package root.
#' @return `TRUE` if any `R/*-wrappers.R` file exists.
#' @noRd
wrappers_file_exists <- function(pkg_path) {
  r_dir <- fs::path(pkg_path, "R")
  if (!fs::dir_exists(r_dir)) {
    return(FALSE)
  }
  length(fs::dir_ls(r_dir, glob = "*-wrappers.R", fail = FALSE)) > 0
}

#' Bootstrap a fresh package's R wrappers via a forced wrapper-gen install
#'
#' On a brand-new package there is no generated `R/<pkg>-wrappers.R`. The
#' wrappers are emitted by the wrapper-gen pass during install. A plain
#' `devtools::install(build = TRUE)` can't bootstrap them because its
#' `R CMD build` step runs `bootstrap.R`, which vendors the tarball and
#' flips `./configure` into wrapper-skipping tarball mode.
#'
#' This helper clears any stale latch, re-runs `./configure`, then does an
#' in-place `devtools::install(build = FALSE)` with
#' `MINIEXTENDR_FORCE_WRAPPER_GEN=1` to force the wrapper-gen pass even if stale
#' generated configuration still describes tarball mode. Configure itself never
#' creates `inst/vendor.tar.xz`. Once wrappers exist, `devtools::document()` is
#' run and the package is installed once more so the namespace-aware install
#' lands.
#'
#' @param pkg_path Absolute path to the package root.
#' @return Invisibly `TRUE`.
#' @noRd
bootstrap_fresh_wrappers <- function(pkg_path) {
  cli::cli_alert_info(c(
    "No generated {.path R/*-wrappers.R} found \u2014 bootstrapping wrappers ",
    "before the full build."
  ))

  # Clear any stale tarball-mode latch so configure gets a clean source-mode
  # start. Configure never creates a replacement vendor tarball.
  clear_install_mode_latch(pkg_path)

  # Re-configure now that the latch is gone, so the tarball-mode
  # .cargo/config.toml (if any) is replaced with the correct variant.
  miniextendr_configure(pkg_path)

  # Generate wrappers via an in-place install. build = FALSE skips R CMD build
  # (and therefore bootstrap.R's tarball-producing vendor step).
  # MINIEXTENDR_FORCE_WRAPPER_GEN=1 also protects against stale generated
  # tarball-mode configuration.
  old_force <- Sys.getenv("MINIEXTENDR_FORCE_WRAPPER_GEN", unset = NA)
  Sys.setenv(MINIEXTENDR_FORCE_WRAPPER_GEN = "1")
  on.exit(
    if (is.na(old_force)) Sys.unsetenv("MINIEXTENDR_FORCE_WRAPPER_GEN")
    else Sys.setenv(MINIEXTENDR_FORCE_WRAPPER_GEN = old_force),
    add = TRUE
  )

  # Removal/rename case (#1288), mature-package flavour: reachable when a
  # package's wrappers.R was deleted (e.g. `just clean`) while NAMESPACE still
  # retains exports of a since-removed/renamed fn. Defer this first install's
  # failure the same way Step 3 above does -- but the discriminator here is
  # wrappers_file_exists(), not a bounded retry count: the Makevars `** libs`
  # target writes wrappers in-place *before* R CMD INSTALL's test-load runs,
  # so a test-load-only failure still leaves wrappers on disk (defer + warn;
  # document() + the second install below heal it). If wrappers are still
  # absent, the failure was real (e.g. a genuine compile error) -- re-raise
  # with the original wording and the captured message.
  bootstrap_error <- NULL
  tryCatch(
    # reload = FALSE: see install_pkg() -- avoid loading the .rdb-backed namespace
    # that the document()+reinstall below would then corrupt for pkgload (#1000).
    devtools::install(pkg_path, build = FALSE, upgrade = FALSE, quiet = FALSE,
                      reload = FALSE),
    error = function(e) bootstrap_error <<- e
  )

  if (!is.null(bootstrap_error)) {
    if (wrappers_file_exists(pkg_path)) {
      cli::cli_warn(c(
        "Bootstrap install failed before NAMESPACE reconciliation; deferring.",
        "i" = "Expected when NAMESPACE still lists an export removed or renamed \\
               since the wrappers file was last generated (#1288).",
        "i" = "Continuing to the document step, then retrying the install once.",
        "x" = conditionMessage(bootstrap_error)
      ))
    } else {
      cli::cli_abort(c(
        "Bootstrap install failed",
        "i" = conditionMessage(bootstrap_error)
      ))
    }
  }

  if (!wrappers_file_exists(pkg_path)) {
    cli::cli_abort(c(
      "Bootstrap install completed but no {.path R/*-wrappers.R} was generated.",
      "i" = paste(
        "Expected the wrapper-gen pass to write it. Check that",
        "{.code #[miniextendr]} functions are reachable from {.file src/rust/lib.rs}."
      )
    ))
  }

  # document() so NAMESPACE exports the freshly-generated wrappers, then
  # install once more so the installed package's namespace matches.
  # MINIEXTENDR_FORCE_WRAPPER_GEN is still set via on.exit above.
  devtools::document(pkg_path)
  tryCatch(
    # reload = FALSE: this reinstall rewrites the lazy-load .rdb; reloading it over
    # the document()-loaded namespace is exactly what triggers the R_decompress1
    # "lazy-load database is corrupt" failure pkgload reports (#1000).
    devtools::install(pkg_path, build = FALSE, upgrade = FALSE, quiet = FALSE,
                      reload = FALSE),
    error = function(e) {
      cli::cli_abort(c(
        "Bootstrap re-install (after document) failed",
        "i" = conditionMessage(e)
      ))
    }
  )

  cli::cli_alert_success("Bootstrapped R wrappers")
  invisible(TRUE)
}

#' Remove the install-mode latch and its source-mode-incompatible siblings
#'
#' `inst/vendor.tar.xz` is the single signal that flips `./configure` into
#' offline tarball mode. The unpacked `vendor/` directory and the
#' tarball-mode `src/rust/.cargo/config.toml` are downstream artifacts of the
#' same mode. Clearing all three guarantees the next `./configure` resolves in
#' source mode. Safe and idempotent -- no-op if nothing is present.
#'
#' @param pkg_path Absolute path to the package root.
#' @return Invisibly `TRUE`.
#' @noRd
clear_install_mode_latch <- function(pkg_path) {
  latch <- fs::path(pkg_path, "inst", "vendor.tar.xz")
  if (fs::file_exists(latch)) {
    fs::file_delete(latch)
    cli::cli_alert_info("Cleared stale {.path inst/vendor.tar.xz} latch.")
  }
  vendor_dir <- fs::path(pkg_path, "vendor")
  if (fs::dir_exists(vendor_dir)) {
    fs::dir_delete(vendor_dir)
  }
  cargo_dir <- fs::path(pkg_path, "src", "rust", ".cargo")
  if (fs::dir_exists(cargo_dir)) {
    fs::dir_delete(cargo_dir)
  }
  invisible(TRUE)
}

#' Prepare vendor tarball for CRAN submission
#'
#' High-level workflow that vendors all crate dependencies and compresses
#' them into `inst/vendor.tar.xz` for offline CRAN install. Wraps
#' [vendor_crates_io()] (which delegates to `cargo-revendor`) plus tarball
#' compression. `cargo-revendor` resolves against the local workspace when a
#' dev `[patch."<git-url>"]` override is present (so a cross-crate rename
#' resolves against the working tree, not git@main) and stamps the canonical
#' `git+url#<sha>` source into `Cargo.lock`; checksum lines are retained.
#'
#' Run this before `R CMD build` when preparing a CRAN submission.
#' Day-to-day development (`R CMD INSTALL .`, `devtools::install/test/load`)
#' does not need it: install mode is auto-detected from
#' `inst/vendor.tar.xz` presence, and without the file cargo resolves deps
#' over the network.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns the path to the created tarball.
#' @export
miniextendr_vendor <- function(path = ".") {
  with_project(path)
  cli::cli_h1("miniextendr vendor workflow")

  cargo_toml <- usethis::proj_path("src", "rust", "Cargo.toml")
  if (!fs::file_exists(cargo_toml)) {
    cli::cli_abort(c(
      "{.path src/rust/Cargo.toml} not found",
      "i" = "Run {.code miniextendr_configure()} first"
    ))
  }

  # Step 1: cargo revendor + CRAN-trim (delegates to vendor_crates_io).
  #
  # cargo-revendor resolves the dependency graph with the dev
  # [patch."git+url"] override active (it pins cargo's CWD to the manifest
  # dir, so a monorepo .cargo/config.toml is honoured), then stamps the
  # framework crates' `source = "git+url#<sha>"` attribution back into
  # Cargo.lock -- the shape offline source-replacement needs. So a cross-crate
  # feature/dep rename resolves against the local workspace, not git@main
  # (#883), and there is no bare-git "regenerate the lock first" dance: the
  # step that disabled the patch was exactly what broke cross-surface renames.
  # In a standalone package (no [patch] override) cargo resolves the framework
  # crates from their git URL directly and the natural git source is kept.
  cli::cli_h2("Step 1: vendor all dependencies")
  vendor_crates_io()

  vendor_dir <- usethis::proj_path("vendor")
  lockfile <- usethis::proj_path("src", "rust", "Cargo.lock")
  inst_dir <- usethis::proj_path("inst")
  tarball <- fs::path(inst_dir, "vendor.tar.xz")

  # Step 3: compress into inst/vendor.tar.xz
  # Note: Cargo.lock checksum lines are intentionally retained. cargo-revendor
  # (post PR #408) writes valid .cargo-checksum.json entries with real SHA-256s,
  # so stripping `checksum = "..."` from Cargo.lock is no longer needed and
  # would diverge from the `just vendor` reference output.
  cli::cli_h2("Step 2: compress vendor tarball")
  fs::dir_create(inst_dir)

  # Create staging directory for clean compression
  staging <- fs::path_temp("vendor-compress")
  on.exit(unlink(staging, recursive = TRUE), add = TRUE)
  if (fs::dir_exists(staging)) fs::dir_delete(staging)
  fs::dir_create(staging)
  fs::dir_copy(vendor_dir, fs::path(staging, "vendor"))

  # Truncate .md files (avoids CRAN notes about non-portable content)
  md_files <- fs::dir_ls(fs::path(staging, "vendor"), recurse = TRUE, glob = "*.md")
  for (f in md_files) {
    writeLines(character(), f)
  }

  # Create xz-compressed tarball.
  # Suppress macOS xattr metadata (AppleDouble `._*` files + LIBARCHIVE.xattr.*
  # PAX headers) that trigger GNU tar warnings on CRAN Linux machines.
  # COPYFILE_DISABLE=1 stops `._*` files; --no-xattrs stops PAX headers.
  old_copyfile <- Sys.getenv("COPYFILE_DISABLE", unset = NA)
  Sys.setenv(COPYFILE_DISABLE = "1")
  on.exit(
    if (is.na(old_copyfile)) Sys.unsetenv("COPYFILE_DISABLE")
    else Sys.setenv(COPYFILE_DISABLE = old_copyfile),
    add = TRUE
  )
  tar_args <- c("-cJf", tarball, "-C", staging, "vendor")
  has_no_xattrs <- identical(
    suppressWarnings(tryCatch(
      system2(
        "tar",
        c("--no-xattrs", "-cf", "/dev/null", "--files-from", "/dev/null"),
        stdout = FALSE, stderr = FALSE
      ),
      error = function(e) 127L
    )),
    0L
  )
  if (has_no_xattrs) {
    tar_args <- c("--no-xattrs", tar_args)
  }
  tar_output <- system2("tar", tar_args, stdout = TRUE, stderr = TRUE)
  if (!is.null(attr(tar_output, "status"))) {
    cli::cli_abort(c(
      "Failed to create vendor tarball",
      "i" = paste(tar_output, collapse = "\n")
    ))
  }

  size_mb <- round(as.numeric(fs::file_size(tarball)) / 1024 / 1024, 1)
  cli::cli_alert_success("Created {.path inst/vendor.tar.xz} ({size_mb} MB)")
  cli::cli_alert_info("Include this in your CRAN submission (R CMD build will bundle it)")
  cli::cli_alert_warning(c(
    "{.path inst/vendor.tar.xz} flips {.code ./configure} into offline tarball mode."
  ))
  cli::cli_bullets(c(
    "i" = "Run {.code R CMD build .} to produce the release tarball, then delete {.path inst/vendor.tar.xz} to resume source-mode dev:",
    " " = "{.code unlink(\"inst/vendor.tar.xz\")}",
    "i" = "If your package has a local path-dependency sibling, vendoring also froze {.path src/rust/Cargo.toml} (and {.path Cargo.lock}) to resolve against {.path vendor/}. After the build, restore source shape:",
    " " = "{.code git checkout src/rust/Cargo.toml src/rust/Cargo.lock}"
  ))

  invisible(tarball)
}

#' Run R CMD check on a miniextendr package
#'
#' Builds the package tarball and runs R CMD check. Ensures dependencies
#' are vendored so the check works in the isolated temp directory where
#' R CMD check unpacks the tarball.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param args Character vector of extra arguments passed to `R CMD check`.
#'   Defaults to `c("--as-cran", "--no-manual")`.
#' @param error_on Severity level to error on. One of `"error"`, `"warning"`,
#'   or `"note"`. Passed to [rcmdcheck::rcmdcheck()].
#' @param build_args Character vector of extra arguments passed to `R CMD build`.
#' @return The [rcmdcheck::rcmdcheck()] result object, invisibly.
#' @seealso [miniextendr_check_static()] for a fast no-compile variant suitable
#'   for un-vendored packages.
#' @export
miniextendr_check <- function(path = ".",
                               args = c("--as-cran", "--no-manual"),
                               error_on = "warning",
                               build_args = character()) {
  with_project(path)
  if (!requireNamespace("rcmdcheck", quietly = TRUE)) {
    cli::cli_abort(c(
      "rcmdcheck is required for miniextendr_check()",
      "i" = 'Install it with: install.packages("rcmdcheck")'
    ))
  }

  cli::cli_h1("miniextendr check workflow")
  pkg_path <- usethis::proj_get()

  cli::cli_h2("Step 1: build (autoconf + configure + install + roxygen2)")
  miniextendr_build(install = TRUE)

  cli::cli_h2("Step 2: R CMD check")
  cli::cli_alert("Running rcmdcheck with args: {.val {args}}")

  result <- rcmdcheck::rcmdcheck(
    pkg_path,
    args = args,
    build_args = build_args,
    error_on = error_on
  )

  invisible(result)
}
