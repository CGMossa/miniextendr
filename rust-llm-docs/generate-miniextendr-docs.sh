#!/usr/bin/env bash
# Regenerate the LLM-parseable doc corpus for every root-workspace crate plus
# the standalone cargo-revendor utility.
#
#   rust-llm-docs/generate-miniextendr-docs.sh
#
# Outputs to rust-llm-docs/generated/:
#   <crate>.md                     — single-file API digest (rustdoc_megadoc.py)
#   <crate>-impl-inventory.md      — every trait impl grouped by trait + span
#   conversion-impl-inventory.md   — conversion traits only, the dedup-audit view
#
# Requires a nightly-capable rustc (we use RUSTC_BOOTSTRAP=1 on stable) and
# python3 (no third-party deps for the local-crate path). R-package and
# cross-package crates are fixture surfaces rather than framework API crates.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
GEN="$HERE/generated"
mkdir -p "$GEN"

export RUSTC_BOOTSTRAP=1
export RUSTDOCFLAGS="-D warnings -Z unstable-options --output-format json"
DOC_FLAGS=(--no-deps --document-private-items)

cd "$ROOT"

# Cargo target directories are configurable globally, per workspace, and via
# `CARGO_TARGET_DIR`. Resolve them from the same working directory as each
# `cargo doc` invocation so the renderer reads the JSON Cargo actually wrote.
cargo_target_dir() {
  cargo metadata --format-version 1 --no-deps \
    | python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])'
}

ROOT_TARGET="$(cargo_target_dir)"
REVENDOR_TARGET="$(cd "$ROOT/cargo-revendor" && cargo_target_dir)"

echo ">> cargo doc (api, full features)"
cargo doc "${DOC_FLAGS[@]}" -p miniextendr-api --features full

echo ">> cargo doc (macros, engine, lint, cli)"
cargo doc "${DOC_FLAGS[@]}" \
  -p miniextendr-macros -p miniextendr-engine -p miniextendr-lint -p miniextendr-cli

echo ">> cargo doc (bench, all features)"
cargo doc "${DOC_FLAGS[@]}" -p miniextendr-bench --all-features

echo ">> cargo doc (cargo-revendor — standalone workspace)"
(cd "$ROOT/cargo-revendor" && cargo doc "${DOC_FLAGS[@]}")

# json basename -> output basename  (cli's lib crate is named `miniextendr`)
gen() {
  local json="$1" out="$2"
  python3 "$HERE/rustdoc_megadoc.py"        "$ROOT_TARGET/doc/${json}.json" "$GEN/${out}.md" >/dev/null
  python3 "$HERE/rustdoc_impl_inventory.py" "$ROOT_TARGET/doc/${json}.json" \
    --source-label "target/doc/${json}.json" \
    --out "$GEN/${out}-impl-inventory.md" >/dev/null
  echo "   ${out}.md + ${out}-impl-inventory.md"
}

# like gen() but for the cargo-revendor standalone workspace (separate target/)
gen_revendor() {
  local json="$1" out="$2"
  python3 "$HERE/rustdoc_megadoc.py"        "$REVENDOR_TARGET/doc/${json}.json" "$GEN/${out}.md" >/dev/null
  python3 "$HERE/rustdoc_impl_inventory.py" "$REVENDOR_TARGET/doc/${json}.json" \
    --source-label "cargo-revendor/target/doc/${json}.json" \
    --out "$GEN/${out}-impl-inventory.md" >/dev/null
  echo "   ${out}.md + ${out}-impl-inventory.md"
}

echo ">> rendering markdown"
gen miniextendr_api    miniextendr-api
gen miniextendr_macros miniextendr-macros
gen miniextendr_engine miniextendr-engine
gen miniextendr_bench  miniextendr-bench
gen miniextendr_lint   miniextendr-lint
gen miniextendr        miniextendr-cli
gen_revendor cargo_revendor cargo-revendor

# Conversion-only inventory — the dedup-audit lens.
python3 "$HERE/rustdoc_impl_inventory.py" "$ROOT_TARGET/doc/miniextendr_api.json" \
  --traits TryFromSexp,IntoR,Coerce,TryCoerce,IntoRAs,RSerializeNative,RDeserializeNative,IntoRAltrep,AltrepSerialize \
  --source-label "target/doc/miniextendr_api.json" \
  --out "$GEN/conversion-impl-inventory.md" >/dev/null
echo "   conversion-impl-inventory.md"

# Manual-vs-macro lens — hand-rolled impls a macro could absorb.
python3 "$HERE/rustdoc_manual_vs_macro.py" "$ROOT_TARGET/doc/miniextendr_api.json" \
  --source-label "target/doc/miniextendr_api.json" \
  --out "$GEN/conversion-manual-vs-macro.md" >/dev/null
echo "   conversion-manual-vs-macro.md"
echo ">> done -> $GEN"
