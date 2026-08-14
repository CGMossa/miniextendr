# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE).
# Invoked by pkgbuild (devtools::build, r-lib/actions/check-r-package) in
# the source directory before R CMD build seals the tarball. Two jobs:
#   1. Run ./configure so Makevars and .cargo/config.toml exist before
#      R CMD build collects them.
#   2. Produce inst/vendor.tar.xz via cargo-revendor, so the sealed
#      tarball ships with vendored sources for offline install.
#
# Order matters: configure runs FIRST. In a dev/monorepo checkout (the
# source dir still has a workspace .git ancestor — pkgbuild's
# copy-method defaults to "none"), configure detects the monorepo and
# writes a [patch."<git-url>"] block into src/rust/.cargo/config.toml
# redirecting miniextendr-{api,lint,macros} to the local workspace.
# cargo-revendor then resolves the dependency graph against THOSE local
# sources, so a cross-crate feature/dep rename touching both a framework
# crate and rpkg resolves against the PR's checkout instead of git@main
# (#883). cargo-revendor stamps the canonical `source = "git+<url>#<sha>"`
# attribution back into Cargo.lock itself (the offline source-replacement
# shape), so there is no longer any bare-git pre-resolution dance here.
# src/rust/.cargo is .Rbuildignore'd, so the dev [patch] config is never
# sealed into the tarball — install-time configure rewrites it to the
# [source] replacement form.
#
# Vendoring lives here because bootstrap.R is part of tarball production.
# configure.ac never creates inst/vendor.tar.xz; it only consumes an existing
# tarball to select offline mode. A package artifact built without the vendor
# tarball therefore remains in source mode and is not CRAN-ready.
#
# At install time bootstrap.R does NOT run (Config/build/bootstrap is
# pkgbuild-only). The bundled inst/vendor.tar.xz from step 2 is what
# configure detects to build offline.

# MINIEXTENDR_BOOTSTRAP=1 tells configure's leaked-tarball guard (#1029) that
# this ./configure was invoked by bootstrap, not directly. The vendor step that
# precedes devtools::build()/check() leaves inst/vendor.tar.xz in the
# (git-tracked) source dir on purpose; without this signal configure would
# (correctly, for a *direct* invocation) treat that tarball-in-a-git-tree as a
# leak and abort. We pass it inline to the configure call only, so it never
# leaks into the cargo-revendor step or the surrounding R session.
if (.Platform$OS.type == "windows") {
  if (file.exists("configure.ucrt")) {
    system2("sh", "configure.ucrt", env = "MINIEXTENDR_BOOTSTRAP=1")
  } else if (file.exists("configure.win")) {
    system2("sh", "configure.win", env = "MINIEXTENDR_BOOTSTRAP=1")
  }
} else {
  system2("bash", "./configure", env = "MINIEXTENDR_BOOTSTRAP=1")
}

# A manifest-declared path-dependency sibling (e.g. a core crate at
# `path = "../../../my-core"`) is NOT source-replaceable: a git/staged install
# (remotes, pak) copies the package out of its workspace and strands it, so it
# MUST be vendored here while still reachable. A git-only package has no such
# sibling and builds straight from source — configure's [patch] override for
# in-tree siblings, or cargo fetching the git URL — so vendoring, and thus
# cargo-revendor, is optional. Heuristic: a `path =` entry in any dependency
# table — incl. [workspace.dependencies] and subtables; [patch.*]/[lib] excluded.
declares_path_dep <- function(manifest = "src/rust/Cargo.toml") {
  if (!file.exists(manifest)) return(FALSE)
  in_deps <- FALSE
  for (ln in readLines(manifest, warn = FALSE)) {
    s <- trimws(ln)
    if (startsWith(s, "[")) {
      in_deps <- grepl("dependencies(\\.[^]]+)?\\]$", s) &&
        !startsWith(s, "[patch")
      next
    }
    if (in_deps && grepl("(^|[][{, \t])path[ \t]*=", s)) return(TRUE)
  }
  FALSE
}

if (!file.exists("inst/vendor.tar.xz")) {
  cargo_revendor <- Sys.which("cargo-revendor")
  if (!nzchar(cargo_revendor)) {
    if (declares_path_dep()) {
      stop(
        "bootstrap.R: cargo-revendor not on PATH, but this package declares a\n",
        "path-dependency sibling that must be vendored before the package is\n",
        "copied out of its workspace. Install it with:\n",
        "  cargo install --git https://github.com/A2-ai/miniextendr ",
        "cargo-revendor --locked",
        call. = FALSE
      )
    }
    message(
      "bootstrap.R: cargo-revendor not on PATH and no path-dependency sibling ",
      "to vendor; building from source (cargo fetches dependencies over the network)."
    )
  } else {
    message("bootstrap.R: generating inst/vendor.tar.xz via cargo-revendor")
    dir.create("inst", showWarnings = FALSE)
    # --freeze rewrites local path-dependency siblings (e.g. a core crate at
    # `path = "../../../my-core"`) to point at vendor/, so the sealed tarball is
    # self-contained: a path dep is NOT source-replaceable, so without --freeze the
    # shipped Cargo.toml would still reference a sibling that does not travel inside
    # the tarball and the offline install would fail to resolve it. Inert for the
    # common git-only package (no local path deps to rewrite, no committed patches),
    # where it only normalises Cargo.lock. cargo-revendor auto-detects the source
    # root from `cargo metadata`, so no --source-root is needed here.
    status <- system2("cargo", c(
      "revendor",
      "--manifest-path", "src/rust/Cargo.toml",
      "--output", "vendor",
      "--freeze",
      "--compress", "inst/vendor.tar.xz",
      "--blank-md",
      "--source-marker",
      "--force",
      "-v"
    ))
    if (status != 0) {
      stop("bootstrap.R: cargo revendor failed (exit ", status, ")", call. = FALSE)
    }
  }
}
