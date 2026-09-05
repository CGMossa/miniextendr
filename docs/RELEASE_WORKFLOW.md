# Release Workflow: Platform Gotchas

Every miniextendr consumer eventually writes an `r-release.yml` GitHub Actions
workflow that builds and checks their package on multiple platforms. AlmaLinux 8
containers and macOS runners surface six reproducible gotchas that downstream
maintainers hit independently. The first four are locale / auth issues uncovered
in issue #448; the last two align the macOS Rust toolchain ABI with CRAN's
binary distribution. This document explains each one, gives the canonical fix,
and explains why miniextendr's own locale check exists — so you know not to file
a bug upstream.

Use `minirextendr::use_release_workflow()` to scaffold a template that has all
six fixes already baked in.

## The six gotchas

### Gotcha 1: AlmaLinux 8 minimal defaults to the `C` locale

AlmaLinux 8's container image ships with only the `C`/`POSIX` locale installed.
When R starts, `l10n_info()[["UTF-8"]]` returns `FALSE`, which causes
`miniextendr_assert_utf8_locale()` to fire:

```
Error: miniextendr requires a UTF-8 locale (R >= 4.2.0 uses UTF-8 by default)
```

**Fix**: set `LANG=C.UTF-8` and `LC_ALL=C.UTF-8` at the **job** level (not the
workflow level — see Gotcha 2) for container jobs, or install a full locale pack
and use `en_US.UTF-8`:

```yaml
jobs:
  build-linux:
    runs-on: ubuntu-latest
    container: almalinux:8
    env:
      LANG: C.UTF-8
      LC_ALL: C.UTF-8
    steps:
      - name: Install build tools
        run: |
          dnf install -y --setopt=install_weak_deps=False \
            git gh glibc-langpack-en
```

`glibc-langpack-en` is lightweight and also makes `en_US.UTF-8` available as an
alternative.

### Gotcha 2: macOS rejects `C.UTF-8`

`C.UTF-8` is a glibc extension. macOS's `setlocale(3)` does not recognise it;
the call silently falls back to `C`, which is non-UTF-8, and the same assertion
fires.

**Fix**: do **not** set `LANG` or `LC_ALL` at the workflow-level `env:` block.
macOS runners default to UTF-8 (`en_US.UTF-8` / `UTF-8`) and work correctly
without any locale override. Scope the locale env vars to the container jobs
only (as shown in Gotcha 1).

```yaml
# WRONG — breaks macOS:
env:
  LANG: C.UTF-8

# CORRECT — scoped to the AlmaLinux container job only:
jobs:
  build-linux:
    container: almalinux:8
    env:
      LANG: C.UTF-8
```

### Gotcha 3: AlmaLinux minimal lacks `git`; cargo libgit2 cannot auth private deps

The AlmaLinux 8 minimal image ships without `git`. Cargo's built-in libgit2
path cannot read git's credential helper, so private git dependencies fail to
fetch even if a token is available.

**Fix**: install `git` and `gh` before any cargo operation, then call
`gh auth setup-git` to configure git's credential helper:

```yaml
    steps:
      - name: Install build tools (AlmaLinux 8 minimal)
        run: |
          dnf install -y --setopt=install_weak_deps=False \
            git gh dnf-plugins-core which procps-ng glibc-langpack-en
      - uses: actions/checkout@v4
      - name: Configure git auth for cargo private deps
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: gh auth setup-git
```

`gh auth setup-git` requires `GH_TOKEN` (or `GITHUB_TOKEN`) in the step's
environment; without it `gh` exits with "no token". Also set
`CARGO_NET_GIT_FETCH_WITH_CLI=true` (see Gotcha 4) so cargo uses the CLI git
instead of libgit2.

> **PAT required for cross-org private deps.** `GITHUB_TOKEN` only grants
> access to the workflow's own repository. If your package depends on private
> git repos in *another* GitHub organization or user account, `GITHUB_TOKEN`
> will fail with a 404. Use a fine-grained PAT stored as a repository secret
> (e.g. `secrets.GH_PAT`) and substitute it in the `GH_TOKEN:` env line.

### Gotcha 4: macOS needs auth + `CARGO_NET_GIT_FETCH_WITH_CLI`

macOS runners ship with `git` preinstalled, but cargo still uses its built-in
libgit2 by default. libgit2 does not read the macOS Keychain credential store
or the git credential helper configured by `gh auth setup-git`, so private git
deps fail.

**Fix**: call `gh auth setup-git` (same as Linux), and set
`CARGO_NET_GIT_FETCH_WITH_CLI=true` workflow-wide so cargo uses the system
`git` binary and inherits its credential config:

```yaml
env:
  CARGO_NET_GIT_FETCH_WITH_CLI: "true"

jobs:
  build-macos:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v4
      - name: Configure git auth for cargo private deps
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: gh auth setup-git
```

`CARGO_NET_GIT_FETCH_WITH_CLI=true` is safe to set at the workflow level —
it has no negative effect on AlmaLinux or any other platform.

### Gotcha 5: macOS SDK and deployment target must match CRAN

CRAN's macOS binary R is built against a specific Xcode SDK with a specific
`MACOSX_DEPLOYMENT_TARGET`. The current binary targets (per [R Installation
and Administration, "Building binary
packages"](https://github.com/r-devel/r-svn/blob/d7049b9c6335346e4448f5e3718c979a1c3ad2d2/doc/manual/R-admin.texi))
are:

| Arch | macOS target | Deployment floor |
|---|---|---|
| arm64 | Sonoma 14 | `MACOSX_DEPLOYMENT_TARGET=14.0` |
| x86_64 | Big Sur 11 | `MACOSX_DEPLOYMENT_TARGET=11.0` |

If a GitHub Actions runner builds Rust artefacts against a different SDK or
without a deployment-target pin, two things break under CRAN's R:

- The Rust `cdylib` / `staticlib` emits load commands referencing newer SDK
  symbols. R's package linker (built against CRAN's SDK) can't resolve them,
  so `R CMD INSTALL` fails or the resulting `.so` segfaults on load.
- `dyld` mismatch warnings ("was built for newer macOS version (X) than being
  linked (Y)") appear in `R CMD check` and trip `--as-cran` notes.

**Fix**: select the matching Xcode and export the deployment target in
`$GITHUB_ENV` before any Rust toolchain install or cargo build. The pin
**must run before** `dtolnay/rust-toolchain` and `r-lib/actions/setup-r` so
both pick up the value. Branch on `uname -m` so the same step works on both
runners:

```yaml
- name: Pin macOS SDK and deployment target
  run: |
    if [ "$(uname -m)" = "x86_64" ]; then
      sudo xcode-select -s /Applications/Xcode_16.2.app || true
      echo "MACOSX_DEPLOYMENT_TARGET=11.0" >> $GITHUB_ENV
    else
      sudo xcode-select -s /Applications/Xcode_26.0.app || true
      echo "MACOSX_DEPLOYMENT_TARGET=14.0" >> $GITHUB_ENV
    fi
    echo "xcode is set: $(xcode-select --print-path)"
    xcrun --show-sdk-version || true
```

The values track the "Set SDK to match CRAN" step of
[`r-devel/actions/setup-macos-tools@ec72e88`](https://github.com/r-devel/actions/blob/ec72e88/setup-macos-tools/action.yml).
The scaffolded template inlines (rather than `uses:`-references) those lines
so the workflow stays hermetic if the upstream repo is ever archived or
restructured.

**Why inlined, not `uses:`-referenced?** `r-devel/actions` is a small,
maintainer-driven repository; a deletion or refactor would silently break
every downstream workflow that referenced it. Inlining trades a one-line
`uses:` for ~10 lines of explicit shell that the maintainer can audit and the
runner can execute without an external repo lookup.

**Per-install floor.** configure also emits these values in
`.cargo/config.toml` `[env]` so end-user `R CMD INSTALL` (no GitHub Actions
involvement) gets the same pins derived from the host's R. The workflow pin
is the CI overlay that locks the values for release builds independent of
whatever R is installed on the runner. See
[CRAN_COMPATIBILITY.md](./CRAN_COMPATIBILITY.md#toolchain-abi-matching) for
the layered defense.

### Gotcha 6: CRAN system libraries on macOS

CRAN's macOS binary R is linked against a curated set of system libraries
(libcurl, openssl, libtiff, libwebp, …) staged under `/opt/R/<arch>/lib`,
built and packaged by
[`r-universe-org/macos-libs`](https://github.com/r-universe-org/macos-libs).
A Rust crate with a `-sys` C dependency (e.g. `openssl-sys`, `curl-sys`,
`libtiff-sys`) that uses `pkg-config` to discover its system library will, on
a stock GitHub Actions macOS runner, resolve against **Homebrew's** version
rather than CRAN's. The two ABIs are not always compatible — same SONAME,
different symbol exports — and packages that pass locally segfault under
CRAN's R.

**Fix**: prefetch the curated tarball into `/opt/`, then point
`PKG_CONFIG_PATH` at `/opt/R/<arch>/lib/pkgconfig`. Any subsequent
`cargo build` of a `-sys` crate resolves against the same library set CRAN
built against:

```yaml
- name: Download CRAN system libraries
  run: |
    sudo mkdir -p /opt
    sudo chown $USER /opt
    curl --retry 3 --fail-with-body -sSL \
      https://github.com/r-universe-org/macos-libs/releases/download/2025-12-13/cranlibs-everything.tar.xz \
      -o libs.tar.xz
    sudo tar -xf libs.tar.xz -C / opt
    rm -f libs.tar.xz
    echo "PKG_CONFIG_PATH=/opt/R/$(uname -m)/lib/pkgconfig:/opt/R/$(uname -m)/share/pkgconfig" >> $GITHUB_ENV
```

The tarball URL pins a specific release date so the workflow is reproducible
across re-runs. Bump the URL when the upstream repo cuts a new release; the
underlying library versions only change when CRAN itself updates them.
Inlined from the "Download CRAN system libraries" step of
[`r-devel/actions/setup-macos-tools@ec72e88`](https://github.com/r-devel/actions/blob/ec72e88/setup-macos-tools/action.yml)
for the same hermetic reason as Gotcha 5.

**What was skipped from upstream** (and why):

- `brew unlink $(brew list --formula)` — destructive on shared runners and
  unnecessary once `PKG_CONFIG_PATH` is set (cargo's `pkg-config` lookup
  prefers the listed paths over the system default).
- gfortran install — no Fortran-linked Rust deps in miniextendr today.
  Downstream packages that depend on a Fortran library (e.g. via a `-sys`
  crate wrapping LAPACK) can add it back.
- TinyTeX — the default scaffold builds with `--no-manual`, so no PDF
  toolchain is required.
- xQuartz — no X11 dependencies in the default scaffold.
- Adding `/opt/R/<arch>/bin` to `$GITHUB_PATH` — `r-lib/actions/setup-r`
  installs R independently and puts its `bin/` on PATH; the upstream's path
  addition is for runs that don't use `setup-r`.

## Why `package_init` checks for UTF-8

`miniextendr_assert_utf8_locale()` is called during R package initialization
(from `R_init_<package>`) via `miniextendr-api/src/encoding.rs`. It calls
`l10n_info()[["UTF-8"]]` (public R API) and aborts if the result is `FALSE`.

This check makes native/unmarked CHARSXPs safe to interpret as UTF-8. Explicit
per-string tags remain possible in a UTF-8 session: the conversion layer uses a
zero-copy path for UTF-8/ASCII, translates Latin-1 text, and rejects `bytes`
strings. In a non-UTF-8 locale, native strings have no compatible zero-copy
interpretation for Rust's `str`, so the framework asserts up-front.

The check is not a bug in miniextendr — it is a guard against using the
framework in an environment where string correctness cannot be guaranteed. The
platform is wrong (or differently configured), not the framework. The fixes in
Gotchas 1–2 bring the platform into compliance; the assertion then passes and
package load succeeds normally.

R >= 4.2.0 itself defaulted to UTF-8 on all mainstream platforms (Windows via
UTF-8 code page, macOS and modern Linux always UTF-8), so the assertion fires
only on minimal container images (like AlmaLinux 8) that ship without any
UTF-8 locale installed.

## Reference template

`minirextendr/inst/templates/r-release.yml` is a drop-in baseline workflow
with all four fixes applied. Scaffold it into your package with:

```r
minirextendr::use_release_workflow()
```

The template targets AlmaLinux 8 (RHEL 8 family — common in enterprise release
pipelines) and macOS arm64 (`macos-14`). Extend it with additional steps for
your package's specific setup (R package installation, cargo caching, upload
steps, etc.).

## Monorepo layouts (`rpkg_subdir`)

If the R package lives in a subdirectory of a Cargo workspace (the layout this
repo uses — `rpkg/` under the workspace root), `bash ./configure` and `R CMD
build` must run from inside that subdirectory. Pass `rpkg_subdir` to inject
`working-directory:` on every relevant step:

```r
minirextendr::use_release_workflow(rpkg_subdir = "rpkg")
```

When `rpkg_subdir = NULL` (default) the call attempts auto-detection via
`detect_project_type()`; pass `auto_detect_subdir = FALSE` to force standalone
mode on a confirmed single-package layout.

**Why the `working-directory:` key precedes `run:`** — the build step uses a
`run: |` block scalar (`R CMD build .` + `R CMD check ...`). YAML resolves the
scalar's content by the indent of its first line, so any same-indent line that
follows `run: |` is absorbed into the scalar text instead of becoming a sibling
mapping key. `release_workflow_insert_workdir()` inserts `working-directory:`
*before* the matched `run:` line so it stays a peer of `run:`. The structural
test in `minirextendr/tests/testthat/test-monorepo-gaps.R` parses the patched
workflow with `yaml::read_yaml()` and asserts every configure and build step
exposes `working-directory: <subdir>` as a real key.
