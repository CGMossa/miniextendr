# rpkg/ — example R package (`miniextendr`)

The exemplar consumer of the framework. Builds via `configure.ac` + `tools/*.R` (no `just` dependency for end users). See root `CLAUDE.md` for shared rules; this file covers rpkg-specific quirks.

## Loaded name
Package loads as `library(miniextendr)`, not `library(rpkg)` (DESCRIPTION's `Package:` field). Don't reference `rpkg::` from R code.

## Dev loop
```bash
just configure          # REQUIRED before any R CMD op in dev mode
just rcmdinstall        # build + install; compiles Rust, generates R wrappers
just devtools-document  # roxygen2 → NAMESPACE + man/
just devtools-test      # testthat
just r-cmd-build / r-cmd-check  # tarball + check
```
After **anything** that affects R wrapper output (proc-macro roxygen, `r_wrappers.rs`, adding `#[miniextendr]` fns) run `just rcmdinstall && just force-document`, then commit `NAMESPACE` + `man/*.Rd` in the same PR. `R/miniextendr-wrappers.R` and `src/rust/wasm_registry.rs` are **gitignored** (regenerated on every install, like `inst/vendor.tar.xz`) — nothing to commit there. CI's `just wrappers-sync-check` regenerates wrappers.R and git-diffs NAMESPACE + man to catch drift.

## Where installs land (main vs worktree)
`just rcmdinstall` / `R CMD INSTALL` deposit `miniextendr` into `.libPaths()[1]`,
which rv's `activate.R` sets to this checkout's own `rv/library/<ver>/<arch>`.
- **Main checkout**: installs land in `<repo>/rv/library/…`, alongside all deps.
- **Linked worktree**: run `just worktree-sync` (= `RV_LINK_MODE=symlink rv sync`) **first** — it symlinks the cached deps from `~/.cache/rv` (shared, warm from main) into the worktree's own `rv/library`, then `rcmdinstall` installs `miniextendr` there as a real dir. The worktree is fully isolated; main is untouched; parallel worktrees don't race. `rv sync` prunes non-lockfile packages, so sync **before** installing dev packages (and re-install if you sync again). Full flow + rationale in root `CLAUDE.md` → *Agent worktrees*. **Never `ln -s` `rv/library`** to main — reintroduces the parallel-install race.

## File-edit rules (templates → generated)
- `src/Makevars` ← `src/Makevars.in`
- `src/rust/.cargo/config.toml` ← the `cargo-config` block in `configure.ac`
- `src/miniextendr-win.def` ← `src/win.def.in`
- `configure` ← `configure.ac` (run `autoconf` after editing)
- `src/stub.c` — static, no substitution. Minimal empty C so R produces a `.so`.

`configure.ac` rules: don't mutate sources at configure time (dirties VCS). Don't call `minirextendr::*` from configure — put helpers in `tools/*.R`, invoke via `Rscript tools/foo.R`.

## Install-mode latch
`inst/vendor.tar.xz` is the **single signal** flipping configure into tarball mode:
- Absent → source mode (workspace `[patch."git+url"]`).
- Present → tarball mode (offline build from `vendored-sources`).

The tarball is **gitignored** (since 2026-04-18) — CI regenerates per-build. Locally, recipes that produce the tarball (`r-cmd-build`, `r-cmd-check`, `devtools-build`) trap-clean on exit; recipes that consume configure state (`rcmdinstall`, `devtools-test/-load/-install`) refuse to run if it's present. If you ever see "monorepo edits silently ignored" → `just clean-vendor-leak`.

## Sandbox compilation
Any compiling command (`devtools-document`, `rcmdinstall`, `cargo build`, `R CMD INSTALL/check`) needs `dangerouslyDisableSandbox: true` when invoked via the Bash tool.

## Gctorture fixtures
Any new path storing SEXPs across allocations (typical: `Vec<SEXP>` / sidecar fields / generic-list buffers / `from_raw_pairs`/`from_raw_values` inputs) needs a no-arg `gc_stress_<feature>()` exported wrapper in `src/rust/gc_stress_fixtures.rs`. The fast gctorture sweep only exercises no-arg exports. See `docs/GCTORTURE_TESTING.md` and #430.

## S3 generics from impl blocks need a hand-written Rd alias
A method in a `#[miniextendr(s3)]` impl block makes the generated wrappers `export()` a bare S3 generic (`parse_value`) next to the `S3method()` registration. The class Rd page aliases only the method (`parse_value.SerdeChecker`), so `R CMD check` reports the generic under "Undocumented code objects". Convention: add a roxygen block with `@name <generic>`, `@param x`, `@param ...` and a one-line title to `R/generics.R` (see the `check_value` / `parse_value` blocks there). Neither `rcmdinstall` nor `force-document` warns about the omission; verify with `Rscript -e 'tools::undoc(dir = "rpkg")'` from the repo root (empty output = clean) or the built-tarball check.

## Capturing R output
```bash
just <recipe> 2>&1 > /tmp/<name>.log
```
Then Read the log — never `tail`/`head`. Check directory: `rpkg-check-output/miniextendr.Rcheck/` (`00check.log`, `00install.out`, `tests/`).

## Built tarball vs source dir
Always `R CMD check` the **built tarball**, not the source dir — source-dir check skips `Authors@R` → `Author/Maintainer` conversion.

## Subdirs
- `R/miniextendr-wrappers.R` — **generated + gitignored**, do not hand-edit (regenerated every install).
- `R/` (other) — hand-written R API.
- `man/` — generated by roxygen2 (tracked).
- `src/rust/` — Rust crate.
- `src/rust/wasm_registry.rs` — **generated + gitignored** wasm32 snapshot (regenerated every install).
- `tools/` — `Rscript`-invoked helpers for `configure`.
- `tests/testthat/` — R-side tests.
- `inst/vendor.tar.xz` — gitignored install-mode latch.
