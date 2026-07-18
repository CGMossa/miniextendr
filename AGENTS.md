@/Users/elea/.codex/RTK.md

# miniextendr

Rust-R interoperability framework for building R packages with Rust backends.

## Principles

- **No backwards compatibility**: unreleased project — remove deprecated code, don't shim around it.
- **Simple over complex**: only make changes directly requested or clearly necessary. Trust the framework — no defensive error handling for internal invariants.
- **Fix warnings you see**: no "known issues". Every warning, lint, or test failure gets fixed, even if pre-existing and unrelated. Leave the code cleaner than you found it.
- **Deferred items = GitHub issues**: any scope cut, known limitation, or partial fix needs `gh issue create` referenced in the PR. No "out of scope" without a tracked issue. Before creating one, run `just issues-refresh`, read `ISSUES/_open-index.json` end-to-end, then search/read the relevant `ISSUES/issue-*.md` bodies and `ISSUES/_closed-index.json` for overlap.
- **`just` is maintainer-only**: end-user packages must build via `configure.ac` / `tools/*.R` / standard R mechanisms. Never require `just` in scaffolded packages.
- **`configure.ac` never mutates sources**: don't rewrite Cargo.toml / Cargo.lock / .rs during `./configure` — dirties VCS. Use `cargo revendor --freeze` at vendor time.
- **`configure.ac` must not call `minirextendr::*`**: put helpers in `tools/` and invoke via `Rscript tools/foo.R`.
- **Edit `.in` templates, not generated files**:
  - `rpkg/src/rust/.cargo/config.toml` ← the `cargo-config` block in `rpkg/configure.ac`
  - `rpkg/src/Makevars` ← `rpkg/src/Makevars.in`
  - `rpkg/src/miniextendr-win.def` ← `rpkg/src/win.def.in`
  - `rpkg/configure` ← `rpkg/configure.ac` (then `autoconf`)
  - `rpkg/src/stub.c` — static, no substitution.
- **Collect all errors in vectorized ops**: don't bail on first failure — give users one batched diagnostic.
- **Flat plans, no phases**: list work in flat priority order, not "Phase 1/2/3".
- **Prefer `From`/`TryFrom` over `as` casts**: propagate the error rather than silently truncating.

### Rust/FFI gotchas

- **`DataFrame` (view) vs `BuiltDataFrame` (owned) split (#1128)**: `DataFrame` is a cheap `Copy` **view** over a bare SEXP with no GC root (sound only while an R `.Call` frame or a `ProtectScope` keeps it reachable). Every **Rust-side constructor** returns `BuiltDataFrame`, an owned RAII handle that roots the frame (`R_PreserveObject`/`R_ReleaseObject`, `!Send`): `IntoDataFrame::into_dataframe`, `SerdeRowBuilder::finish`, `DataFrame::builder().build()`, serde `*_to_dataframe`, `NamedList::as_data_frame`. It `Deref`s to `DataFrame` and returns to R via `IntoR`. `!Send` means it can't cross back out of `with_r_thread`/`run_on_worker` — build+read+drop it on the R thread. The **editing** methods also return rooted handles (#1247): the new-frame producers (`drop`/`select`/`select_rows`/`prepend_column`/`with_column`) return `BuiltDataFrame`, with inherent forwards on `BuiltDataFrame` for the whole editing set (the `drop` forward shadows `Drop::drop` — plain deref would hit E0040), so constructor→edit chains are rooted at every link. `rename`/`strip_prefix` stay in-place view methods. Remaining footgun: hand-smuggling the view out of a handle (`*built`) then letting the handle drop. `column: SEXP` args to `with_column`/`prepend_column` must be caller-rooted across the call.
- **Pointer provenance**: cache `*mut T` via a mutable path (`&mut T`, `Box::into_raw`, `downcast_mut`, `ptr::from_mut`). Never write through a `cached_ptr` derived from `&T` / `downcast_ref` — UB under Stacked Borrows.
- **`cargo package` for workspace resolution**: when vendoring, let `cargo package` expand workspace inheritance — never hard-code workspace dependency replacements.
- **m4 in `AC_CONFIG_COMMANDS`**: `$1` is empty (use `$0` or avoid `sh -c`). Escape `[` / `]` in sed/grep as `@<:@` / `@:>@`.
- **Windows paths in TOML**: forward slashes only. Strip `\\?\` prefix from `canonicalize()` output before writing.
- **macOS tar xattrs**: set `COPYFILE_DISABLE=1` when creating tarballs to avoid Apple metadata warnings on Linux/Windows GNU tar.
- **`cargo-revendor`**: standalone workspace (excluded from miniextendr workspace). Build/test via `just revendor-build`/`just revendor-test`. `--freeze` resolves `Cargo.toml` against `vendor/` only.

### `just` vs raw `cargo`

Always use `just check/clippy/test/fmt/vendor/lint` — the recipes iterate every manifest in the multi-crate workspace (root, `rpkg/src/rust`, `tests/cross-package/*`) with the correct `[patch.crates-io]` overrides. Raw `cargo --workspace` only sees the top-level manifest. If a recipe doesn't exist, say so and fall back — don't invent shortcuts.

**Interactive iteration**: prefer `cargo-limit` aliases (`cargo lcheck/lclippy/ltest/lbuild`) — they truncate output to the first few errors. Install with `just dev-tools-install`. CI and `just` recipes keep plain `cargo` (CI needs full output + `-D warnings`).

## Project Structure

```
miniextendr-api/      # Runtime (FFI, ExternalPtr, ALTREP, worker thread)
miniextendr-macros/   # Proc macros (#[miniextendr], derives; naming in src/naming.rs)
miniextendr-bench/    # Benchmarks (separate workspace member)
miniextendr-lint/     # Static analysis
miniextendr-engine/   # Standalone R embedding (REngine) for Rust binaries/tests
cargo-revendor/       # Standalone cargo subcommand (not in workspace)
rpkg/                 # Example R package (installed as `miniextendr`)
minirextendr/         # Pure R scaffolding helper
tests/cross-package/  # producer.pkg / consumer.pkg — trait ABI tests
site/                 # Zola docs → GitHub Pages
background/           # Reference docs (gitignored)
```

Every directory that has a `CLAUDE.md` has a sibling `AGENTS.md`: each
**subdirectory** one is a one-line `@CLAUDE.md` import (codex's include
mechanism), so it inherits that `CLAUDE.md` verbatim with no drift — when a new
`CLAUDE.md` is added, drop a matching `@CLAUDE.md` `AGENTS.md` beside it. This
**root** `AGENTS.md` is a hand-kept mirror of the root `CLAUDE.md`, not an
import: a project-wide rule changed there must be mirrored here by hand. CI
enforces the structural invariant (sibling presence, no orphans, the
`@CLAUDE.md` import line) via `just agents-md-check` (Sync Checks job); prose
parity of this root mirror stays hand-maintained.

## Build Commands

```bash
# Rust
just check / test / clippy / fmt / lint

# rpkg (R package)
just configure           # REQUIRED before any R CMD operation (dev mode)
just rcmdinstall         # Build + install (compiles Rust, auto-generates R wrappers)
just devtools-document   # roxygen2 → NAMESPACE + man/ (short-circuits if nothing changed)
just force-document      # like devtools-document but always runs — use after macro changes
just devtools-test       # R tests
just r-cmd-build         # Build tarball
just r-cmd-check         # Check built tarball (save to log — see Capturing Output)
just devtools-check      # Check, preserving output in rpkg-check-output/

# CRAN release prep (only step needed; configure auto-detects tarball mode)
just vendor              # regen Cargo.lock in tarball-shape, vendor deps, compress to inst/vendor.tar.xz

# Cross-package
just cross-install / cross-test / cross-check

# minirextendr
just minirextendr-install / minirextendr-test / minirextendr-check

# Site
just site-build / site-serve   # http://127.0.0.1:1111

# LLM-ready Rust API corpus
just llm-docs / llm-docs-check
```

### Configure is mandatory

Always `bash ./configure` (not bare `./configure` — `#!/bin/sh` causes spurious errors in `AC_CONFIG_COMMANDS` passthrough). Configure:
1. Generates `Makevars` from `.in` templates
2. Auto-detects install mode (source vs tarball) from `[ -f inst/vendor.tar.xz ]`
3. Writes `.cargo/config.toml` per mode (source: `[patch."git+url"]` for monorepo siblings or empty; tarball: `[source]` replacement to `vendored-sources`)
4. Does **not** create `inst/vendor.tar.xz` — that's `just vendor`

Package loads as `library(miniextendr)`, not `library(rpkg)`. Always check the **built tarball**, not the source dir (R CMD check on a source dir skips `Authors@R` → `Author/Maintainer` conversion).

### Install modes (source vs tarball)

| Mode | Triggered when | What configure does |
|---|---|---|
| **Source** | `inst/vendor.tar.xz` absent | No vendoring. In monorepo: writes `[patch."git+url"]` → workspace siblings. Otherwise: minimal config, cargo follows git URL. |
| **Tarball** | `inst/vendor.tar.xz` present | Unpacks tarball, writes `[source]` replacement to `vendored-sources`, build is offline. |

That's the entire decision. No `NOT_CRAN`, no `FORCE_VENDOR`, no `PREPARE_CRAN`, no `BUILD_CONTEXT`. See `docs/CRAN_COMPATIBILITY.md`.

## Development Workflow

### After Rust changes (especially macro changes)

```bash
just configure && just rcmdinstall && just force-document
```

Use `just force-document` (not `just devtools-document`) after **anything** that affects R wrapper output: proc-macro roxygen changes, `r_wrappers.rs` / class-system codegen changes, adding/removing `#[miniextendr]` functions. `force-document` bypasses `roxygen2::needs_roxygenize()` — required when macro output has changed but the mtime cache has not caught up.

`just devtools-document` (short-circuiting variant) is safe for **pure R/roxygen changes** — skips roxygenize when `needs_roxygenize()` returns `FALSE`.

**A new export needs a *second* install to be runtime-callable.** `just rcmdinstall` regenerates `wrappers.R` but installs against the *existing* `NAMESPACE`; `just force-document` then writes the new export into `NAMESPACE` on disk but does **not** reinstall — so a freshly added `#[miniextendr]` fn is absent from the *installed* package until you install again. To gctorture / testthat a new export, run the loop twice: `rcmdinstall && force-document && rcmdinstall`. (Committing the regenerated `NAMESPACE` / `man` only needs the single pass.)

Generated documentation (`NAMESPACE`, `man/*.Rd`) must be committed in sync
with the Rust changes that produced it. `rpkg/R/miniextendr-wrappers.R` and
`rpkg/src/rust/wasm_registry.rs` are gitignored and regenerated during install;
they must be present on disk when building the tarball, not committed.

The pre-commit hook (`.githooks/pre-commit`) guards the tarball-shape Cargo.lock.
Enable once per clone: `git config core.hooksPath .githooks`.

### Adding a `#[miniextendr]` function

1. `pub fn` with `#[miniextendr]` — registration is automatic via linkme `#[distributed_slice]`, no module macro needed
2. Module must be reachable via `mod` from `lib.rs` (`#[cfg(feature = "foo")]` on `mod` declaration is sufficient for feature-gated modules)
3. `just configure && just rcmdinstall && just force-document`

Build sequence: `Makevars` → `cargo build --lib` (staticlib, the only cargo invocation) → C-link `$(OBJECTS)` + staticlib → final `.so` (`$(SHLIB)`, via `stub.c`'s force-link) → `dyn.load($(SHLIB))` + registered `miniextendr_write_wrappers`/`miniextendr_write_wasm_registry` routines → `R/miniextendr-wrappers.R` + `src/rust/wasm_registry.rs`. No separate cdylib build — wrapper-gen loads the installed `.so` itself; tarball/wasm32 installs skip it and use the pre-shipped files.

### Reproducing CI clippy before PR

`just clippy` ≠ CI. Three CI clippy steps must all pass `-D warnings`:

- `clippy_default`: `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `clippy_all`: same + `full-codegen` and a curated integration feature list — read the list from `.github/workflows/ci.yml` (`clippy_all` step); hard-coding it here drifts (it already has)
- `clippy_all_s7`: same list with `full-codegen-s7` (the `s7-default` mutex side) instead of `full-codegen`

`--all-features` fails (`r6-default` and `s7-default` are mutually exclusive). CI runs a newer toolchain, so lints like `collapsible_match`, `manual_checked_ops` can fire on CI with green local. Reproduce all three before pushing.

### sccache + `[profile.dev] incremental`

CI uses sccache with `CARGO_INCREMENTAL=0`. Setting `[profile.dev] incremental = false` in a workspace's `Cargo.toml` jumps sccache hit rate to ~100% (incremental builds poison cache keys with per-invocation hashes). Trade-off: loses local incremental compilation — usually worth it for CI and agent worktrees (`cargo clean` often). Not set project-wide currently — flag in PR description if changing.

### `inst/vendor.tar.xz` is not tracked

Gitignored — generated in CI (every R CMD check runs `just vendor` first) and at release time. Tracked tarballs caused binary merge conflicts, 22 MB/commit bloat, and stale-after-rebase drift. Regenerate locally with `just vendor` — cheap and deterministic from `Cargo.lock` + workspace sources.

## Generated artifacts (do not hand-edit, do not count toward LoC)

- `rust-llm-docs/generated/*.md` — tracked, LLM-ready Rust API digests and
  trait-impl inventories. Regenerate with `just llm-docs`; verify drift and
  renderer coverage with `just llm-docs-check`.
- `rpkg/src/rust/wasm_registry.rs` (~13K lines) — wasm32-target substitute
  for linkme `#[distributed_slice]` tables. Auto-generated by
  `miniextendr_write_wasm_registry` (registered as a C-callable, see
  `miniextendr-api/src/registry.rs`). **Gitignored** (regenerated on every
  host `R CMD INSTALL`) — like `inst/vendor.tar.xz`, it caused constant merge
  conflicts. It ships in the tarball **from disk** (`.Rbuildignore` does not
  exclude it); `just r-cmd-build` regenerates it first so the built tarball is
  complete. wasm-from-tarball has NO regeneration fallback (the wasm cdylib is a
  SIDE_MODULE host R cannot dyn.load), so a missing/stub copy silently breaks it.
- `rpkg/R/miniextendr-wrappers.R` (~32K lines) — the R-callable layer, generated
  by the same host cdylib pass. **Gitignored** for the same reason. Native
  tarball installs can regenerate it via the Makevars `#1022` fallback, but
  `just r-cmd-build` ships it anyway. Regenerate locally with `just rcmdinstall`.
- `NAMESPACE` + `man/*.Rd` — derived (by roxygen2) from the regenerated
  wrappers.R + hand-written R. **Still tracked** (small, low-conflict). Run
  `just configure && just rcmdinstall && just force-document` after macro changes
  and commit them. CI's `just wrappers-sync-check` regenerates wrappers.R then
  git-diffs NAMESPACE + man to catch drift.

## Key Concepts

- **Execution model**: generated Rust runs on R's main thread inside
  `R_UnwindProtect` by default. `#[miniextendr(worker)]` and `worker-default`
  opt into the dedicated worker; R API work routes back through `with_r_thread`.
- **ExternalPtr**: Box-like owned pointer over `EXTPTRSXP`. Stores `Box<Box<dyn Any>>` — thin ptr in `R_ExternalPtrAddr` → fat ptr on heap (carries `Any` vtable). Type safety via `Any::downcast`, not R symbols. Non-generic `release_any` finalizer. `cached_ptr` must have mutable provenance.
- **TypedExternal**: R-visible type name (`TYPE_NAME_CSTR` display tag, `TYPE_ID_CSTR` errors). Not authoritative for type safety — `Any::downcast` is.
- **ALTREP**: Single-struct pattern, no wrapper. Two paths:
  - *Field-based derive* — `#[derive(AltrepInteger)]` with `#[altrep(len = "field", elt = "field", class = "Name")]` generates everything
  - *Manual* — `#[altrep(manual)]` generates lowlevel traits + registration; user writes `AltrepLen` and `Alt*Data`
  - `AltrepExtract` trait abstracts data extraction (blanket impl for `TypedExternal`; override for custom storage)
  - `#[miniextendr]` on 1-field structs is **removed** — use derives
- **R_UnwindProtect**: runs Rust destructors on R errors
- **GC**: `OwnedProtect` / `ProtectScope` for RAII protect/unprotect
- **Dots (`...`)**: `_dots: &Dots`, or `name: ...` for a custom name. See `docs/DOTS_TYPED_LIST.md`.
- **typed_list!**: `#[miniextendr(dots = typed_list!(...))]` validates and creates `dots_typed`
- **`impl Trait`**: return position only (`-> impl IntoR`). Argument position fails type inference (E0283 across `let` bindings for `TryFromSexp + Trait`).
- **S4 helpers**: `slot()`/`slot<-()` live in `methods` — resolve via `getNamespace("methods")`, not `R_BaseEnv`.

### FFI thread checking (`#[r_ffi_checked]`)

On `unsafe extern` blocks, generates both checked and `*_unchecked` variants. Checked routes through `with_r_thread()` (debug assert); unchecked bypasses. Use `*_unchecked` inside ALTREP callbacks, `with_r_unwind_protect`, or `with_r_thread` blocks to skip the debug assertion. Statics pass through unchanged. `^nonapi^` variants require `#[cfg(feature = "nonapi")]`.

### Adding a new conversion type (e.g., `Box<[T]>`)

Modify in order:
1. `miniextendr-api/src/from_r.rs` — `TryFromSexp` impls (native, NA-aware, bool, String variants)
2. `miniextendr-api/src/into_r.rs` — `IntoR` impls (`Box<[T: RNativeType]>` blanket + explicit `Box<[bool]>` / `Box<[String]>`)
3. `miniextendr-api/src/coerce.rs` — `Coerce<Box<[R]>> for Box<[T]>`
4. Serde docs — `src/serde.rs`, `src/serde/de.rs`, `src/serde/traits.rs`
5. rpkg fixtures — `rpkg/src/rust/<type>_tests.rs` + `rpkg/tests/testthat/test-<type>.R`
6. `vendor_miniextendr(path = "rpkg", local_path = ".")` to sync vendor/

`bool` ≠ `RNativeType` (R uses `i32` for logicals) → separate impls. Proc-macro handles `Box<[T]>` generically via `TryFromSexp`/`IntoR` — no macro changes.

## Code Style

- **Never `mod.rs`** — use `foo.rs` + `foo/` directory. Migrate if touched.
- **Section comments**: `// region:` / `// endregion` (IDE-foldable). Migrate `// =====`, `// ──`, `// ---` when touched.

## miniextendr-lint

Build-time static analysis (runs via `build.rs` during `cargo build`/`check`). Disable with `MINIEXTENDR_LINT=0`.

- **MXL008**: trait-impl class-system compat with inherent impl
- **MXL009**: multiple impl blocks missing distinct labels → add `#[miniextendr(label = "...")]`
- **MXL010**: duplicate labels
- **MXL106**: non-`pub` function that would get `@export` → make `pub` or add `#[miniextendr(noexport)]`
- **MXL110**: parameter name is an R reserved word → codegen will break
- **MXL111**: `s4_*` method name on `#[miniextendr(s4)]` impl — codegen auto-prepends, you'll get `s4_s4_*`
- **MXL203**: redundant `internal` + `noexport`
- **MXL300**: direct `Rf_error`/`Rf_errorcall` → replace with `panic!()` (framework converts to R error)
- **MXL301**: `_unchecked` FFI outside known-safe contexts

## Common Issues

- **"could not find function"**: check `#[miniextendr]` + `pub`, module reachable from `lib.rs`, then `just configure && just rcmdinstall && just force-document`.
- **"configure: command not found"**: `cd rpkg && autoconf && bash ./configure`.
- **Permission errors installing**: `R_LIBS=/tmp/codex/R_lib R CMD INSTALL rpkg` or `just devtools-install`. `/tmp/codex/` is writable in Codex sandboxes.
- **Stale `.snap.new`**: diff vs `.snap`; if expected, `mv` over the old snapshot. Re-run `just test`.
- **Segfaults**: `R -d lldb -e '…'`; at `(lldb)` type `run`, then `bt` / `frame select` / `p`.

## Capturing Command Output

Redirect long-running R/cargo commands to a log, then **Read tool** the log — never `tail` / `head`:

```bash
just <recipe> 2>&1 > /tmp/<name>.log
```

Common: `devtools-doc.log`, `rcmdinstall.log`, `rcmdcheck.log`, `devtools-test.log`, `vendor.log`, `devtools-check.log`, `minirextendr-test.log`. `rpkg-check-output/miniextendr.Rcheck/` has `00check.log`, `00install.out`, `tests/`.

## Sandbox Restrictions

Some agent sandboxes block compilation. For any compiling command (`just force-document`, `rcmdinstall`, `cargo build`, `R CMD INSTALL/check`), run with the agent's full-access or unsandboxed mode.

## File Deletion Safety

- **Never `rm`** or permanent-delete in automated workflows.
- Use trash (`trash`, `gio trash`, platform equivalent).
- No trash utility available? Stop and ask.

## Agent Worktrees

- Run agents in worktrees (`isolation: "worktree"`) to avoid collisions.
- **Worktree R library: `rv sync` per worktree, then install** (never symlink to main). `rv/library/` is gitignored, so a fresh worktree's library is empty. Populate it from rv's shared global cache (`~/.cache/rv`, warm from main — same `rproject.toml`), then install the dev packages:
  ```bash
  just worktree-sync   # = RV_LINK_MODE=symlink rv sync — symlinks the ~110 cached deps into this worktree's OWN rv/library (~8s)
  just configure && just rcmdinstall && just force-document
  ```
  `RV_LINK_MODE=symlink` (in `just worktree-sync`) links deps as symlinks into `~/.cache/rv` (zero-copy, no recompile/download) instead of the macOS copy-on-write default. Each worktree gets its own real `rv/library` (deps = symlinks to the shared read-only cache; dev packages = real installed dirs), so parallel worktrees never race and main is untouched. rv's native cross-project cache model: https://a2-ai.github.io/rv-docs/concepts/cache/ . **Order matters**: `rv sync` prunes anything not in the lockfile — including `miniextendr`/`minirextendr` (`dependencies_only`) — so sync first, install second; a later `rv sync` wipes the dev packages (re-install). **Never `ln -s rv/library`** to main (the old fix; reintroduces the shared-install race).
- **Merge**: rebase worktree onto current main, *then* merge. Rebase must happen immediately before the merge, not when the agent finishes.
- **Sequential merging** of multiple worktrees: rebase → merge → rebase next → merge. Each rebase must see prior merge commits on main, otherwise changes get silently overwritten.
- **Never copy whole files** worktree → main — rebase/merge is the only correct flow.
- If agent didn't commit, commit in the worktree first.
- Don't delete a worktree until its branch is pushed or merged.
- **Clean up after push**: `git worktree remove -f -f <agent-worktree-path>` + `git worktree prune`. Each worktree holds a full `target/` (2–3 GB/agent).
- **Rebase conflicts**: plain `git rebase origin/main`. NEVER `-X theirs` blanket — drops main's changes to shared files (justfile, lockfiles, etc.). Resolve everything by hand **except regenerated artifacts** — never hand-merge their hunks; take either side (`git checkout --theirs` / `git add`) then regenerate:
  - `rpkg/inst/vendor.tar.xz` (binary tarball) → `just vendor`, amended into the vendor-refresh commit.
  - `patches/templates.patch` (rpkg→templates delta, a generated diff) → `just templates-approve`, then verify with `just templates-check`.

## Sync Checks

### Vendor sync

`just vendor-sync-check` verifies vendored copies match workspace sources; `just vendor` refreshes.

**Stale vendor freeze recovery**: `just vendor --freeze` writes `path = "../../vendor/..."` into `rpkg/src/rust/Cargo.toml` `[dependencies]` and `[patch.crates-io]`. After merging main, the frozen vendor/ can go stale and `cargo metadata` fails. Fix: reset frozen path deps back to `"*"`, delete `rpkg/vendor/` + `rpkg/src/rust/Cargo.lock`, run `just configure`.

### Template sync (rpkg → templates)

`minirextendr/inst/templates/` are **derived from rpkg** (master source).

1. Apply changes to `rpkg/` first
2. Port to `minirextendr/inst/templates/`
3. `just templates-approve` locks the delta

`just templates-check` verifies no unexpected drift. Approved delta is recorded in `patches/templates.patch` — templates may have extra standalone-project logic (checking for miniextendr-api before using path overrides, running `cargo vendor` for transitive deps).

## Documentation Site

Zola static site in `site/` → GitHub Pages at `https://a2-ai.github.io/miniextendr/`. Content pages are TOML-frontmatter markdown (`+++`). `weight` controls sort order.

GitHub Actions auto-deploys on push to `main` when `site/**`, `docs/**`, or `*/src/**` changes: runs `scripts/docs-to-site.sh` → builds nightly rustdoc (`--document-private-items --document-hidden-items --show-type-layout --enable-index-page --generate-link-to-definition -Z rustdoc-map`) → builds Zola → copies rustdoc to `site/public/rustdoc/` → deploys. Rustdoc index at `.../rustdoc/`; individual crates at `.../rustdoc/miniextendr_api/` etc.

`site/content/manual/` is **auto-generated from `docs/`** by `scripts/docs-to-site.sh` (1:1 conversion, not curated summaries). Edit `docs/*.md` only — never edit `site/content/manual/*.md` directly. The generator derives frontmatter (title + description) from each doc's `# Heading` and first paragraph. `site/content/_index.md` and anything outside `manual/` are hand-authored and must be edited directly.

**The generated manual (`site/content/manual/*.md`, except the hand-authored `_index.md`) and the Zola build output (`site/public/`) are gitignored** since #593 — so edit `docs/*.md` and commit only that; there is **nothing to commit on the site side**. CI runs `docs-to-site.sh` and rebuilds the site from `docs/` before each deploy, so the live site is always correct. To preview locally, run `just site-docs` (+ `just site-build`); don't `git add` the output.

### Site scripts

| Script | What it does |
|--------|-------------|
| `scripts/docs-to-site.sh` | Converts `docs/*.md` → `site/content/manual/` (frontmatter derived from heading + first paragraph) |
| `scripts/bump-version.sh <version>` | Bumps `version =` / `Version:` across all Cargo.toml and DESCRIPTION files (workspace + rpkg + minirextendr + cross-package) |

### Just recipes

| Recipe | What it runs |
|--------|-------------|
| `just site-docs` | `docs-to-site.sh` — run before `site-build`/`site-serve` when previewing doc changes locally |
| `just site-build` | `zola build` only (output in `site/public/`) |
| `just site-serve` | `zola serve` only (http://127.0.0.1:1111) |
| `just bump-version <v>` | `bump-version.sh <v>` — use before a release commit |

## Reference Docs (`background/`, gitignored)

- **Official R**: `R Internals.html`, `Writing R Extensions.html`, `ALTREP_...html`, `Autoconf.html`, `GNU make.html`
- **R source**: `r-source-tags-R-4-5-2/` — key paths `src/include/Rinternals.h`, `src/include/R_ext/Altrep.h`, `src/main/altclasses.c`, `src/main/memory.c`
- **Reference R packages**: `S7-main/`, `R6-main/`, `vctrs-main/`, `roxygen2-main/`, `mirai-main/`
- **ALTREP examples**: `Rpkg-mutable-master/`, `Rpkg-simplemmap-master/`, `vectorwindow-main/`

**Always check `background/` for R API details before guessing.**

## Skill freshness audit (quarterly)

The Claude Code skills under `.claude/skills/<slug>/SKILL.md` cite file paths,
symbols, and line numbers that drift as the code evolves. Run
`bash scripts/skill-freshness-audit.sh` **once a quarter** (and repair drift in
the same pass — source wins, fix the SKILL.md). It flags, per skill: missing
cited paths (BLOCKING, exits non-zero so it can gate CI), symbols that grep
finds nowhere in the repo (WARN), and out-of-range `file.rs:NNN` line cites
(WARN). The script's header documents its known false-positive modes.

The same quarterly pass also rebases two upstream release pins (folded from
#596): the `r-universe-org/macos-libs` `cranlibs-everything.tar.xz` date
pinned in `minirextendr/inst/templates/r-release.yml` (Gotcha 6) and
`docs/RELEASE_WORKFLOW.md`, and the r-windows rtools release number in
`docs/CRAN_COMPATIBILITY.md`'s Layer 3 table. Also force-rebase both before
any release tag, and smoke-test a bumped tarball URL via a real CI run
(download + extract paths drift upstream).

## Reviews

- **Reviews** (`reviews/*.md`): when things go wrong (test/CI failure, runtime error, unexpected behavior), write a short file: *what was attempted*, *what went wrong*, *root cause*, *fix*. Accumulates institutional knowledge on non-obvious failure modes.
- **Forward-looking work lives in GitHub issues**, not plan files. Open a `gh issue create` (with the full design context in the body) for anything that isn't shippable in the current PR. Flat priority order — no phases.
- **Vendor audit**: when deps change, audit `vendor/` for crates worth integrating — open an issue per candidate (e.g., R-relevant error types, serialization, data structures) with the integration sketch in the body.

## Using Codex for Reviews

Use `codex exec` for non-interactive (bare `codex` needs a TTY and fails under Bash tool):

```bash
codex exec -m gpt-5.5 --full-auto "prompt"
codex exec -m gpt-5.5 review "review these changes"
```

## Working a plan file end-to-end (worktree → PR)

A plan file (`plans/*.md`) fully specifies one fix: the bug, the exact change,
verification, and deliverable — it is the execution spec for work being done
*now* (deferred or forward-looking work still goes to a GitHub issue, never a new
plan file — see the Reviews rule above). To execute one in isolation, the rule is
**one plan = one worktree = one PR**:

1. **R version** — `rig default 4.6 && R --version` (must read `4.6.x`, else rv
   enters safe mode and installs break). Re-check at the start of every fresh
   shell.
2. **Read the plan first, from the main checkout.** Plan files are usually
   *untracked*, so they will NOT appear in a fresh worktree. Read
   `plans/<plan>.md` (and any journal it cites, e.g.
   `git show <branch>:journal/<file>.md`) before you branch.
3. **Worktree** (never work on `main`; never collide with a sibling agent):
   ```bash
   git worktree add -b <branch-named-in-plan> ../mx-<slug> origin/main
   cd ../mx-<slug>
   just worktree-sync      # populate this worktree's rv/library from ~/.cache/rv (~8s)
   just configure
   ```
4. **Implement exactly what the plan specifies** — it already chose the
   approach; don't redesign. Fix any warning/lint in code you touch.
5. **Verify — `fmt` and `clippy` are SEPARATE gates; run BOTH:**
   ```bash
   just fmt                      # rustfmt; clippy does NOT cover it, CI fails on it separately
   just check && just test 2>&1 > /tmp/plan-check.log   # then Read the log
   # clippy, all -D warnings: clippy_default + clippy_all + clippy_all_s7
   #   read the feature lists from .github/workflows/ci.yml — never guess them
   ```
   A **new `#[miniextendr]` export** is not runtime-callable until a *double*
   install: `just rcmdinstall && just force-document && just rcmdinstall`, then a
   **targeted** `testthat::test_file(...)` (full `just devtools-test` can OOM via
   callr fan-out). Commit regenerated `NAMESPACE`/`man` in sync
   (`git config core.hooksPath .githooks` first). Known non-issue: the
   `derive_dataframe_*` UI trybuild `.stderr` snapshots skew when the active
   toolchain carries the `rust-src` component (it renders stdlib spans CI's
   minimal stable does not — issue #1239, **not** a rustc-version difference).
   `just test`/`just test-ui` now isolate the UI suite under a version-named
   minimal-profile toolchain automatically (the workspace leg sets
   `MINIEXTENDR_SKIP_UI=1`). Never `TRYBUILD=overwrite` under a rust-src
   toolchain; CI stable is authoritative. See CLAUDE.md "UI test snapshots".
6. **Commit early** — right after the code change compiles — so a mid-run
   disconnect loses nothing; amend/extend as verification completes.
7. **PR** — `git push -u origin <branch>` then `gh pr create --base main` with
   the title from the plan's Deliverable section and a *factual* body (reference
   the hiccup/journal; NEVER leave a literal `TODO: replace this line`). **Do NOT
   merge.** Any scope cut → `gh issue create`, referenced in the PR body.
8. **Cleanup after merge** — `git worktree remove -f -f ../mx-<slug> && git
   worktree prune` (each worktree holds a 2–3 GB `target/`).

## Compaction

Between **200k–400k tokens**, run `/compact` before auto-compaction forces it. Compact sooner during exploration; later during mid-refactor when recent turns are load-bearing.

**Preserve**: current goal, active PRs/branches, file:line for uncommitted in-progress edits, unresolved review comments / test failures, session feedback not yet in `AGENTS.md` or memory.

**Discard**: tool-output dumps, file contents already at their final edited form, search results, anything already in a commit message or memory file.
