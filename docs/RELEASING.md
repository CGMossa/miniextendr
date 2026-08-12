# Releasing miniextendr

How to cut a tagged release of the two shippable artifacts — the **`miniextendr`
CLI** (a Rust binary) and **`rpkg`** (the R package, which installs as
`library(miniextendr)`). There is no release-automation workflow today; releases
are manual tag-and-push. This document is the recipe.

> Not to be confused with [`RELEASE_WORKFLOW.md`](./RELEASE_WORKFLOW.md), which
> is about the CI workflow *downstream consumers* scaffold
> (`minirextendr::use_release_workflow()`) — not about releasing miniextendr
> itself.

## Release gates

Complete the [Maintainer Guide release checklist](./MAINTAINER.md#release-checklist)
before creating the tag. The repository-wide definition of blocking,
conditional, informational, and currently unavailable signals lives in the
[README CI and release gate table](../README.md#ci-and-release-gates).

Run evidence must point at the exact release commit. A green run for an older
`main` commit does not qualify, and GitHub currently does not enforce the
aggregate check in repository settings (tracked in
[#1391](https://github.com/A2-ai/miniextendr/issues/1391)).

## Version bumping

One script bumps every version-bearing file in lockstep:

```bash
just bump-version 0.2.0        # or: bash scripts/bump-version.sh 0.2.0
```

It updates `[workspace.package].version` in the root `Cargo.toml` (which the CLI
inherits via `version.workspace = true`) and the `Version:` field of every
DESCRIPTION (`rpkg`, `minirextendr`, the cross-package test fixtures). Commit the
result, then tag:

```bash
git commit -am "release: v0.2.0"
git tag -a v0.2.0 -m "v0.2.0"
git push && git push --tags
```

## Releasing the CLI

The CLI is a plain binary crate (`[[bin]] name = "miniextendr"`). Its only
dependencies are crates.io packages — it does **not** depend on
`miniextendr-api`/`-macros`, so it has no vendoring or git-pin entanglement.

Once the tag is pushed, users install straight from it — no crates.io publish, no
CI infrastructure required:

```bash
cargo install --git https://github.com/A2-ai/miniextendr --tag v0.2.0 miniextendr-cli
# add --features dev for the contributor-only `dev` subcommands
```

`cargo install --path miniextendr-cli` (or `just cli-install`) remains the
from-checkout path for local development.

> crates.io publishing is not set up. The crate carries no `publish = false`, so
> it *could* be published, but the git-tag install above covers distribution
> without it.

## Releasing rpkg

`rpkg` ships in **two modes**, and they behave differently. Both start from the
pushed tag above.

### Mode A — GitHub source install (`remotes`/`pak`/`devtools`)

```r
remotes::install_github("A2-ai/miniextendr", subdir = "rpkg", ref = "v0.2.0")
# pak::pak("A2-ai/miniextendr/rpkg@v0.2.0")
```

`remotes` downloads the **whole-repo** zipball (no `.git`) — so the framework
crates (`miniextendr-api`, `-lint`, `-macros`) travel along as sibling
directories — and builds via `pkgbuild`. Because DESCRIPTION sets
`Config/build/bootstrap: TRUE`, `pkgbuild` runs `rpkg/bootstrap.R` first.

On the default `build = TRUE` path, `pkgbuild` runs `bootstrap.R`. It vendors
via `cargo-revendor` **only when the manifest declares a path-dependency
sibling** (`path = "../…"`, which a staged/git install would strand). A
git-only package — the exemplar and the typical scaffold — falls through to a
plain source build (configure's `[patch]` for in-tree siblings, or cargo
fetching the git URL), so `cargo-revendor` is **not** required. (Before the
`declares_path_dep()` gate, bootstrap hard-aborted without cargo-revendor even
for git-only packages — `install_github` failed out of the box.)

How the framework crates resolve (read from `configure.ac`):

- configure detects the monorepo by a **filesystem probe** — it walks up ≤5
  levels looking for `miniextendr-api/Cargo.toml` (configure.ac:463–473),
  *not* by a `.git` ancestor. The full-repo zipball satisfies this, so the
  siblings are found and (in pure source mode) a
  `[patch."https://github.com/A2-ai/miniextendr"]` block redirects the three
  crates to those local, tag-state sources.
- The framework **version is pinned by the committed `src/rust/Cargo.lock`**
  (shipped via `Config/build/extra-sources`); its framework entries carry
  `source = "git+…#<sha>"`, and the vendor/offline branch reproduces that sha.
  The unpinned `git = …` in `Cargo.toml` only decides resolution when the lock
  is regenerated — see "Known gap".

**Prerequisites:** Rust toolchain (`SystemRequirements: Cargo, rustc >= 1.85`),
`cargo-revendor` on PATH (undeclared — sharp edge), and network access.
`autoconf` is **not** needed — `configure` is committed.

### Mode B — released vendored tarball (reproducible, offline, CRAN-shaped)

This is the recommended release artifact. **The build produces the vendored
tarball for you** — you do not run `just vendor` by hand:

```r
devtools::build("rpkg")        # → miniextendr_0.2.0.tar.gz, vendored
```

Any **pkgbuild-backed** build honors `Config/build/bootstrap: TRUE` and runs
`bootstrap.R`, which produces `inst/vendor.tar.xz` and seals it into the tarball:
`devtools::build()`, `rcmdcheck`, and `r-lib/actions/check-r-package` all qualify.
(Bare `R CMD build` from a stock R does **not** — `Config/build/bootstrap` is a
pkgbuild extension.) The `just r-cmd-build` / `just r-cmd-check` recipes call
`just vendor` explicitly as well, which adds a defense-in-depth assertion that the
framework crates were vendored from the local workspace rather than git@main
(#876) — but the tarball would be vendored either way.

Attach the resulting `miniextendr_0.2.0.tar.gz` to a GitHub Release. Users install
it offline and reproducibly:

```r
install.packages("miniextendr_0.2.0.tar.gz", repos = NULL, type = "source")
# remotes::install_url(".../releases/download/v0.2.0/miniextendr_0.2.0.tar.gz")
```

At install, `configure` detects `inst/vendor.tar.xz` — the **single latch** — and
switches to tarball mode: offline build from `vendored-sources`, no network, no
`cargo-revendor`.

**Why prefer this over Mode A:** the tarball is **self-contained and offline** —
vendored sources are sealed inside, so the user needs no network and no
`cargo-revendor`, and the framework code is frozen at build time (built inside
the monorepo, `configure` writes a `[patch."git+url"]` override and
`cargo-revendor` vendors the framework crates from the local checkout). Mode A
is a from-source build that still depends, at the user's machine, on a Rust
toolchain, `cargo-revendor`, network access, and the committed lock sha.

> **Building release artifacts on CI** (esp. macOS): the Rust toolchain ABI must
> match CRAN's R, or the `.so` fails to load / trips `--as-cran`. configure bakes
> a per-install `MACOSX_DEPLOYMENT_TARGET` floor into `.cargo/config.toml`
> (configure.ac:520–577), but a release runner should also pin the SDK. See
> [RELEASE_WORKFLOW.md](./RELEASE_WORKFLOW.md) Gotchas 5–6 — and note
> `minirextendr::use_release_workflow(rpkg_subdir = "rpkg")` scaffolds a
> known-good workflow for this exact subdir layout.

## Known gap

The framework git dependencies in `rpkg/src/rust/Cargo.toml` (and the
`tests/model_project` / `minirextendr` fixtures) are declared **unpinned**
(`git = "…"`, no `tag`/`rev`). Reproducibility today rests entirely on the
committed `Cargo.lock` sha — not the manifest. If anything regenerates the lock
during a release build (a `cargo update`, or a cargo old enough to rewrite a v4
lock — see configure.ac:920), the framework jumps to whatever `main` HEAD is at
that moment, silently. Pinning `tag =`/`rev =` in the manifest would make the
release self-describing instead of lock-dependent. Track as an issue rather than
leave implicit.
