# minirextendr/

Pure R scaffolding helper — generates new R packages with a `miniextendr` Rust backend. End users install this from CRAN/GitHub and run `minirextendr::use_miniextendr_package()`. See root `CLAUDE.md` for shared rules.

## Loaded name
Package loads as `library(minirextendr)`. The exemplar consumer (`rpkg/`) is its own package (`library(miniextendr)`).

## Dev loop
```bash
just minirextendr-install
just minirextendr-test
just minirextendr-check
```

## Templates pipeline
`inst/templates/` is **derived from `rpkg/`** (master source). Workflow:
1. Edit `rpkg/` first.
2. Port changes into `inst/templates/`.
3. `just templates-approve` locks the delta into `patches/templates.patch`.
4. `just templates-check` verifies no unexpected drift in CI.

Templates may have extra standalone-project logic (e.g., checking for miniextendr-api before applying path overrides, running `cargo vendor` for transitive deps). That delta is what `patches/templates.patch` records.

## Key R modules
- `R/vendor.R` — `vendor_miniextendr()`, `strip_toml_sections()` (must strip `[[bench]]`, `[[test]]`, `[dev-dependencies]` when those dirs are absent in scaffolded packages).
- `R/use_*.R` — scaffolding (`use_miniextendr_package`, `use_release_workflow`, `use_template`).
- `R/upgrade.R` — `upgrade_miniextendr_package()`.
- `R/doctor.R` — `miniextendr_doctor()`: detects stale `inst/vendor.tar.xz` leak + missing `.cargo/config.toml` + git-tracked generated files (`git rm --cached` advice, #1250); `webr = TRUE` adds the webR import lint.
- `R/webr-lint.R` — `miniextendr_webr_import_lint()`: flags namespace-level imports of compiled packages (static probe + curated fallback, #925).
- `R/check_static.R` — `miniextendr_check_static()` (revamp shipped in PR #296).

## Gotchas
- `usethis::write_over()` skips silently in non-interactive mode. `use_template()` deletes the target first so `upgrade_miniextendr_package()` actually overwrites.
- Cargo directory source can't find manually-extracted crates from `.crate` files — use `[patch.crates-io]` + path deps for workspace crates.
- Regression tests in `tests/testthat/` grep function source for literal strings (`deparse(body)` style). Don't inline a helper just to satisfy them — fix the test or accept the indirection.
- `configure` never vendors. `bootstrap.R` may create `inst/vendor.tar.xz` only while a build frontend is producing a package tarball.

## End-user contract
**`just` is maintainer-only.** Scaffolded packages must build via `configure.ac` / `tools/*.R` / standard R mechanisms. If a template requires `just`, fix the template.
