# cargo-revendor

Standalone `cargo` subcommand for R/CRAN-friendly vendoring. **Excluded from the miniextendr workspace** — has its own `Cargo.toml`/`Cargo.lock`/`target/`. See root `CLAUDE.md` for shared rules.

## Why standalone
End users install via `cargo install cargo-revendor`; it must build without dragging in the miniextendr workspace `[patch."git+url"]` table. Inclusion in the parent workspace would break that.

## Dev loop
- `just revendor-build` — builds against this crate's own manifest.
- `just revendor-test` — runs `cargo test` here.
- Never `cargo --workspace`-it from the root; the root manifest doesn't include it.

## Key features
- **`--freeze`** — resolves `Cargo.toml` against the local `vendor/` only (writes `path = "../../vendor/..."` into `[dependencies]` and `[patch.crates-io]`). Not invoked by `just vendor` (removed; see `docs/CRAN_COMPATIBILITY.md`).
- **`--sync`** — refreshes vendor/ from a Cargo.lock without re-resolving versions.
- **`--versioned-dirs`** — opt-in for now; #239 tracks making it default.
- **`cargo package` for workspace resolution** — let cargo expand workspace inheritance; never hard-code workspace dependency replacements.

## When a package tarball lacks vendored sources
`configure` never creates `inst/vendor.tar.xz`. A maintainer must run an explicit
tarball-producing vendor workflow before release; otherwise the artifact stays
in source mode and fails on CRAN's offline farm. Intended canary.
