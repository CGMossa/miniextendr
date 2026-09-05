# CRAN compatibility and vendoring

How miniextendr keeps the CRAN install path working without polluting day-to-day
development.

## TL;DR

There are exactly two install modes. Configure auto-detects based on a single
signal and configures cargo accordingly:

| Mode | Triggered when | Cargo behavior |
|---|---|---|
| **Source install** | `inst/vendor.tar.xz` is **absent** in the package being installed | Cargo resolves dependencies normally. In monorepo dev, configure writes a `[patch."git+url"]` block in `.cargo/config.toml` that points the three workspace crates at sibling paths. Otherwise cargo fetches the git URL declared in `Cargo.toml`. |
| **Tarball install** | `inst/vendor.tar.xz` is **present** | Configure unpacks the tarball into `vendor/`, writes a `.cargo/config.toml` with `[source.crates-io]` and `[source."git+..."]` redirected to `vendored-sources`, and cargo builds offline. |

That's the entire decision tree. There is no `NOT_CRAN` env var, no
`PREPARE_CRAN`, no `FORCE_VENDOR`, no auto-detected "build context"; just the
file-existence test. `configure` is a consumer of that signal: it never runs
`cargo-revendor` and never creates `inst/vendor.tar.xz`.

## Vendoring belongs to tarball production

Only workflows that are deliberately producing a package tarball may create
`inst/vendor.tar.xz`:

- `just vendor` and `minirextendr_vendor()` create it explicitly before a
  maintainer builds a release artifact.
- `bootstrap.R` creates it before `R CMD build` when a build frontend such as
  `pkgbuild` honors `Config/build/bootstrap: TRUE`.

Everything else stays in source mode when the file is absent. This includes
`bash ./configure`, `R CMD INSTALL .`, and a fresh scaffold outside a Git
checkout. Git ancestry is not an install-mode signal.

A package tarball built without `inst/vendor.tar.xz` is not repaired during
installation. It remains in source mode and cargo may need network access; that
artifact is therefore not CRAN-ready. The failure on CRAN's offline farm is the
intended canary for a maintainer who shipped an incomplete release artifact.

## Where each install path lands

| You ran | Mode | Vendor used? | How vendor was produced |
|---|---|---|---|
| `R CMD INSTALL .` (source dir) | Source | No | n/a; configure does not vendor |
| `devtools::install(build = FALSE)` / `load_all` | Source | No | n/a; no package tarball is produced |
| `R CMD build rpkg` directly | Source unless pre-vendored | Only if already present | run `just vendor` / `miniextendr_vendor()` first for a release artifact |
| `devtools::build("rpkg")` / `pkgbuild::build()` | Tarball | Yes when `cargo-revendor` is available | `bootstrap.R` vendors before `R CMD build` |
| `just r-cmd-build` / `just r-cmd-check` | Tarball | Yes | explicit `just vendor` (recipe dependency) before R CMD build |
| CRAN's autobuilder on a submitted tarball | Tarball | Yes | maintainer's `just vendor` baked it into the tarball |

The mode always maps directly to the file-existence test. Production and
consumption are deliberately separate: tarball workflows produce the signal;
configure only consumes it.

## Why

The previous design fired vendoring on every `just configure` so that the
`path = "../../vendor/..."` deps frozen into `Cargo.toml` would resolve. That
meant:

- 8–12 minutes of `cargo revendor` on every dev iteration.
- sccache hit rates collapsed because per-invocation Cargo.lock churn poisoned
  the cache keys.
- `remotes::install_github("A2-ai/miniextendr", subdir = "rpkg")` couldn't run
  without `cargo-revendor` installed and network access to clone the monorepo
  itself for path-dep bootstrap.
- Four overlapping flags (`NOT_CRAN`, `FORCE_VENDOR`, `PREPARE_CRAN`, the
  Rbuild-tempdir heuristic) disagreed about what mode any given invocation was
  in.

Lifting vendoring to a CRAN-prep-only step deletes all of that. Day-to-day
development uses cargo's normal resolution and the `[patch.crates-io]`-style
override that `just check`/`build`/`test` recipes have always done. CRAN
release prep stays self-contained: maintainer runs `just vendor`, ships the
resulting tarball.

## Maintainer release workflow

```bash
just vendor             # 1) Regenerate Cargo.lock in tarball-shape, vendor
                        #    deps to rpkg/vendor/, compress to inst/vendor.tar.xz.
                        #    Dirties Cargo.lock + writes inst/vendor.tar.xz.
just r-cmd-build        # 2) R CMD build rpkg → miniextendr_X.Y.Z.tar.gz.
                        #    Depends on `just vendor` so the tarball ships
                        #    inst/vendor.tar.xz.
just r-cmd-check        # 3) R CMD check the built tarball (--as-cran).
```

Day-to-day commands (`just rcmdinstall`, `just devtools-install`,
`just devtools-test`, `just devtools-document`, `just devtools-load`) do **not**
depend on `just vendor`. They install via source mode, which doesn't need a
vendor tarball at all. Run `just vendor` only when you're producing a build
artifact for CRAN.

## What `just vendor` actually does

```text
1. Run `cargo revendor` (the [patch."git+url"] override written by
   `just configure` stays ACTIVE). cargo-revendor pins cargo's working dir to
   the manifest dir, so it resolves miniextendr-{api,lint,macros} against the
   LOCAL workspace checkout — a cross-crate feature/dep rename resolves against
   the working tree, not git@main (#883). That leaves them as local (no-source)
   lock entries, so cargo-revendor then STAMPS
   `source = "git+https://github.com/A2-ai/miniextendr#<commit>"` back on.
   It also recomputes `.cargo-checksum.json` after CRAN-trim: the original
   `package` hash (matching the lockfile's `checksum = ...` line) is preserved
   and the `files` map is refreshed to reflect the trimmed files, so the
   committed Cargo.lock can retain its `checksum = ...` lines.
2. Produce rpkg/vendor/ and rpkg/inst/vendor.tar.xz.
```

The stamped git source is what cargo's source replacement needs to redirect to
vendor at install time. Without it, source replacement reports "the source
git+... requires a lock file to be present first before it can be used against
vendored source code". The stamp is reconstructed *after* a local resolve
rather than produced by resolving against bare git, which is what used to break
coordinated cross-crate renames — see
[Cargo.lock shape](./CARGO_LOCK_SHAPE.md) for the full story.

## Cargo.lock shape, drift, and why dev iteration may dirty it

> **See [Cargo.lock shape](./CARGO_LOCK_SHAPE.md)** for a dedicated walkthrough
> of the invariants, the failure modes when they're violated, and the manual
> steps `just vendor` / `miniextendr_vendor()` automate. Summary below.

The committed `rpkg/src/rust/Cargo.lock` is in tarball-shape: workspace crates
have `source = "git+https://github.com/A2-ai/miniextendr#<hash>"`. Registry
`checksum = ...` lines are now **retained** — cargo-revendor writes valid
`.cargo-checksum.json` files that match them.

When you run `cargo build` / `cargo check` in source mode, cargo silently
rewrites the lockfile in place: it re-resolves the workspace crates through
the `[patch."git+url"]` override (so they become `path` sources). **This drift
is expected and harmless for local iteration.** Don't commit it; run
`just vendor` to restore the canonical shape.

If you ever see CI complain that the committed lockfile is in source-shape
instead of tarball-shape, run `just vendor` and commit the regenerated
artifact.

The pre-commit hook (`.githooks/pre-commit`) blocks commits that would
introduce `path+` sources into `rpkg/src/rust/Cargo.lock`.
Run `just lock-shape-check` to verify the committed lockfile is in the correct
shape at any time.

## CI strategy

- **`r-tests`** (Linux): runs `R CMD INSTALL .` on the source dir. Tests source
  mode end-to-end. Does **not** install `cargo-revendor`. This job is the
  implicit smoke test for the source-only install path.
- **`r-check-linux` / `cran-check`**: runs `R CMD check`, which internally
  builds a tarball and tests offline install. Runs `just vendor` first.
  `inst/vendor.tar.xz` is cached across runs keyed on `Cargo.lock` and the
  workspace `Cargo.toml`s, so a no-op re-run skips the vendor step.
- **`sync-checks`**: runs `just vendor` *without* the cache, plus
  `just vendor-sync-check`, to guarantee the tarball is reproducible from
  workspace sources before merge.

## inst/vendor.tar.xz is gitignored

It used to be committed. That caused 22 MB/commit bloat, binary merge
conflicts on every PR that touched a workspace crate, and stale-after-rebase
drift. CI regenerates the tarball before every R CMD check; release tooling
regenerates it at version bump time. Don't try to commit it.

## Stale tarball warning

`inst/vendor.tar.xz` must not linger in the source tree after `just r-cmd-build`
or `just r-cmd-check` finish. Both recipes set `trap 'rm -f rpkg/inst/vendor.tar.xz' EXIT`,
but the trap does not fire on `SIGKILL`. If the file is left behind:

1. `just configure` sees it and sets `IS_TARBALL_INSTALL=true`.
2. The next `just rcmdinstall` (or `R CMD INSTALL rpkg`) runs `make` with
   `IS_TARBALL_INSTALL=true` and `ABS_RPKG_SRCDIR` pointing to the **source**
   `rpkg/src/`. The tarball-mode cleanup in `Makevars.in` then deletes
   `src/rust/.cargo/` from the source tree.
3. The monorepo `[patch."git+url"]` override is gone; cargo silently resolves
   the three workspace crates from `git+https://...#<sha>` instead of local siblings.

Recovery: use `just clean-vendor-leak` (monorepo) or
`miniextendr_clean_vendor_leak()` (scaffolded packages) to remove the stale
tarball, then `just configure` to regenerate `.cargo/config.toml`.
`miniextendr_doctor()` detects both the stale tarball and a missing
`config.toml` and prints the fix.

Dev-consume recipes (`just rcmdinstall`, `just devtools-test`,
`just devtools-load`, `just devtools-install`) will abort with an error if the
tarball is present in the source tree, preventing silent tarball-mode iteration.
See CLAUDE.md "Vendor tarball is a latch" for the full context and the
`just test-bootstrap-vendor` regression test (#441).

## Constraints, in case you're tempted

- `Cargo.toml` must keep miniextendr-{api,lint,macros} declared as `git = "..."`.
  Path deps to `../../vendor/...` would require `vendor/` to exist in source
  mode, which is exactly what we removed. Path deps to monorepo siblings
  (`../../../miniextendr-api`) would break tarball install (the tarball doesn't
  carry siblings).
- Configure must not mutate `Cargo.toml` or `*.rs` (CLAUDE.md project rule).
  Mutating `Cargo.lock` in tarball mode is acceptable — it's an artifact, not
  a source — but `just vendor` does that pre-build, not configure at install
  time.
- `[ -f inst/vendor.tar.xz ]` is the only source-vs-tarball signal. Don't add
  a second one. Maintenance load lives in the number of switches.

## Using a local miniextendr checkout (dev-only)

When you are developing both miniextendr and a consumer package simultaneously,
use `minirextendr::use_local_miniextendr(path)` to wire the consumer's
`configure.ac` to resolve framework crates from your local checkout instead of
the published git URL:

```r
minirextendr::use_local_miniextendr("/path/to/miniextendr")
# Then re-run configure to pick up the override:
minirextendr::miniextendr_configure()
```

This writes a one-line `.miniextendr-local` marker at the package root.
`bash ./configure` reads it with plain shell (`head -n 1`) and sets
`MONOREPO_ROOT` to the recorded path so the existing
`[patch."https://github.com/A2-ai/miniextendr"]` block in
`.cargo/config.toml` resolves the three workspace crates from the local tree.

The marker file is added to `.gitignore` and `.Rbuildignore` automatically
so it can never appear in a distributed tarball. Tarball mode
(`inst/vendor.tar.xz` present) always takes precedence — the local override
becomes inert. `miniextendr_doctor()` warns if the marker is present before
distribution. Call `minirextendr::unuse_local_miniextendr()` to remove the
marker when you no longer need the override.

## Toolchain ABI matching

The vendoring story above keeps **Rust dependency sources** in sync between
maintainer and CRAN's builder. A second, orthogonal problem is keeping the
**Rust toolchain's compile-time targets** (SDK, deployment floor, system
library prefix) in sync with the C toolchain CRAN's R was built with. A
mismatch here produces `.so`s that link locally and segfault under CRAN's R,
or trip `--as-cran` notes about deployment-target drift.

miniextendr defends against this with three layered checks. The first two are
load-bearing, the third documents the values.

### Layer 1 — Per-install floor (`./configure` emits `[env]`)

`./configure` derives `MACOSX_DEPLOYMENT_TARGET` (and equivalents) from the
host R's `R CMD config CC` flags at install time, then writes them into
`.cargo/config.toml`'s `[env]` table. Every `R CMD INSTALL` of the
miniextendr-based package — including end-user installs that never see a
GitHub Actions runner — picks up the same values R is configured against.

This is the load-bearing layer for end users. They don't run the release
workflow; they `install.packages()` (binary) or `R CMD INSTALL` (source) and
expect the result to work with the R they have.

### Layer 2 — CI overlay (`r-release.yml` workflow `env:` pins)

The CI workflow that produces release binaries pins the **CRAN-canonical**
values directly via `MACOSX_DEPLOYMENT_TARGET` in `$GITHUB_ENV` plus
`xcode-select -s` to select the matching SDK. These override whatever Layer 1
derived: the release artifact targets CRAN's exact ABI floor, not the
runner's host R's floor.

GitHub Actions shell `env:` wins over cargo `[env]` by default (cargo's
`[env]` table is "set if not already set" semantics), so the two layers
cooperate without conflict: CI uses the pinned values, end-user installs use
whatever the host R reports.

The same workflow also prefetches CRAN's curated system libraries
(`r-universe-org/macos-libs`) into `/opt/R/<arch>/lib` and points
`PKG_CONFIG_PATH` at them, so any Rust `-sys` crate's `pkg-config` lookup
resolves against the same C libraries CRAN's R was linked with.

See [RELEASE_WORKFLOW.md Gotchas 5 and 6](./RELEASE_WORKFLOW.md#gotcha-5-macos-sdk-and-deployment-target-must-match-cran)
for the exact YAML snippets and citations.

### Layer 3 — Documented canonical values

The current CRAN-canonical pins are:

| Platform | Arch | Pin | Source |
|---|---|---|---|
| macOS | arm64 | `MACOSX_DEPLOYMENT_TARGET=11.0` for the **toolchain**; `14.0` for the **binary** | [R-admin §"Building binary packages"](https://github.com/r-devel/r-svn/blob/d7049b9c6335346e4448f5e3718c979a1c3ad2d2/doc/manual/R-admin.texi) — `Xcode_26.0` |
| macOS | x86_64 | `MACOSX_DEPLOYMENT_TARGET=11.0` | R-admin (same) — `Xcode_16.2` |
| Windows | x86_64 | rtools45 / GCC 14, mingw runtime release `6768` | [`r-windows/rtools-base`](https://github.com/r-windows/rtools-base) |
| Linux | x86_64 | distro-supplied glibc; no per-platform pin | n/a |

The Windows rtools version is **not currently consumed** by miniextendr's
template — there's no `build-windows` job in `r-release.yml` yet. The value
is documented here for completeness, and configure handles the rtools linker
pin for end-user Windows installs already once the per-install `[env]` floor
lands. See [issue tracker](https://github.com/A2-ai/miniextendr/issues?q=is%3Aissue+rtools+windows) for the rtools rebase cadence.

### How to update the pins when CRAN moves

CRAN's macOS binary build moves SDKs roughly once per major macOS release.
When it does, three places must be updated in lockstep:

1. `minirextendr/inst/templates/r-release.yml` — the `xcode-select -s` path
   and `MACOSX_DEPLOYMENT_TARGET` value in the "Pin macOS SDK and deployment
   target" step.
2. `docs/RELEASE_WORKFLOW.md` Gotcha 5 — the version table and prose.
3. This file — the table in "Layer 3" above.

Cross-reference `r-devel/actions/setup-macos-tools` to confirm the upstream
values; the `r-devel/actions` repo tracks CRAN's actual build matrix.

## Symbols cleanup, for grep-bait

Removed entirely from this codebase:

- `NOT_CRAN`
- `FORCE_VENDOR`
- `PREPARE_CRAN`
- `BUILD_CONTEXT` (the dev-monorepo / dev-detached / vendored-install /
  prepare-cran enum)
- `cargo revendor --freeze` invocations from `just vendor`
- The "auto-vendor on first install" + git-clone-bootstrap fallback in
  `configure.ac`
- The unpack-vendor-from-Makevars step in `Makevars.in`

If you find a stray reference, it's vestigial — delete it.
