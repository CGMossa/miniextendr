# https://just.systems
#
# Quick reference:
#   Setup (Windows-only):
#     just install-deps-windows - Install Windows system tooling via Scoop (rv, etc.)
#
#   Rust:
#     just check              - Run cargo check
#     just check-features     - Check feature combinations compile
#     just test               - Run cargo tests
#     just clippy             - Run lints
#     just fmt                - Format Rust code
#     just lint               - Run miniextendr-lint on rpkg
#     just cargo-lock-restore - Undo rpkg/src/rust/Cargo.lock drift from local builds
#
#   rpkg (example R package):
#     just configure          - Configure R package build (dev mode, no vendoring)
#     just configure-fast     - Skip configure when Makevars/.cargo/config.toml are up to date
#     just vendor             - Vendor deps for CRAN release prep
#     just devtools-test      - Run R package tests
#     just devtools-document  - Run roxygen2 (NAMESPACE + man pages)
#     just rcmdinstall        - Build and install R package
#
#   minirextendr (helper R package):
#     just minirextendr-document  - Generate documentation
#     just minirextendr-test      - Run tests
#     just minirextendr-check     - Run R CMD check
#     just minirextendr-install   - Install package
#
#   Cross-package trait ABI tests:
#     just cross-document     - Regenerate docs for both packages
#     just cross-install      - Build + install both packages
#     just cross-test         - Run tests for both packages
#     just cross-check        - Run checks for both packages
#     just cross-clean        - Clean both packages
#
#   Templates:
#     just templates-check    - Verify templates haven't drifted
#     just templates-approve  - Accept template changes
#
#   Benchmarks:
#     just bench              - Run all Rust benchmarks
#     just bench-core         - Core benchmarks (high-signal, default features)
#     just bench-features     - Feature-gated benchmarks (connections, rayon, etc.)
#     just bench-full         - Full suite (core + feature matrix)
#     just bench-r            - Run R-side benchmarks (requires rpkg installed)
#     just bench-save         - Save structured baseline (text + CSV + metadata)
#     just bench-compare      - Show top benchmarks from a baseline
#     just bench-drift        - Check for regressions between last 2 baselines
#     just bench-info         - List saved baselines with metadata
#     just bench-compile      - Macro compile-time perf (synthetic crates)
#     just bench-lint         - Lint scan performance
#     just bench-check        - Check benchmark crate compiles
#
#   Documentation site:
#     just site-docs           - Regenerate site/content/manual/ from docs/
#     just site-build          - Build Zola site (run site-docs first for doc changes)
#     just site-serve          - Local preview server (run site-docs first for doc changes)
#     just llm-docs            - Regenerate LLM-ready Rust API docs
#     just llm-docs-check      - Test renderer and verify committed LLM docs are current
#     just bump-version <v>    - Bump version across all Cargo.toml + DESCRIPTION files
#
#   Issue cache:
#     just issues-refresh      - Refresh local ISSUES/ indexes and open-issue bodies
#
#   Vendor sync:
#     just vendor-sync-check  - Verify vendored crates match workspace
#     just vendor-sync-diff   - Show diff between workspace and vendor
#     just lock-shape-check   - Verify Cargo.lock is in tarball-shape (git sources, no checksums)
#     just clean-vendor-leak  - Remove a leaked inst/vendor.tar.xz that would flip configure into tarball mode
#
set shell := ["bash", "-euo", "pipefail", "-c"]
set windows-shell := ["bash", "-euo", "pipefail", "-c"]

# On Windows (Git Bash / MSYS2), cargo and R may not be in /bin/bash's PATH.
# Export CARGO_HOME/bin so recipes can find cargo/rustc.
export PATH := if os() == "windows" {
    env("CARGO_HOME", env("USERPROFILE", "") / ".cargo") / "bin" + ":" + env("PATH", "")
} else {
    env("PATH", "")
}

# Directory for devtools::check output (preserved for investigation)
check_output_dir := justfile_directory() / "rpkg-check-output"

[default]
default:
    @just --list

clean:
    just configure
    just cargo-clean
    cd rpkg && ./cleanup
    cd tests/cross-package && just clean

# Clean build artifacts
#
# NOTE: The `tmp="$(mktemp -d)" && (cd "$tmp" && cargo ...)` pattern is used
# throughout this file to run cargo from a neutral directory, preventing it
# from picking up the wrong .cargo/config.toml. These temp dirs are empty
# (just used as cwd) and cleaned by the OS periodically - not a significant leak.
cargo-clean *cargo_flags:
    cargo clean -p miniextendr-api {{cargo_flags}}
    cargo clean -p miniextendr-macros {{cargo_flags}}
    cargo clean -p miniextendr-bench {{cargo_flags}}
    cargo clean -p miniextendr-lint {{cargo_flags}}
    cargo clean -p miniextendr-engine {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/rust-target" cargo clean --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/rust-target" cargo clean --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo clean --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.crates-io.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.crates-io.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.crates-io.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Check feature combinations compile (F2: feature interaction testing)
# Tests important feature combos that might interact but are only tested independently.
[script("bash")]
check-features:
    set -euo pipefail
    manifest="miniextendr-api/Cargo.toml"
    combos=(
        # serde interacts with many types
        "serde,ndarray"
        "serde,nalgebra"
        "serde,indexmap"
        "serde_json,ndarray"
        # rayon + type features
        "rayon,vctrs"
        "rayon,ndarray"
        "rayon,serde"
        # connections + strict mode
        "connections,strict-default"
        # numeric ecosystem
        "num-bigint,num-complex,num-traits,ordered-float,rust_decimal"
        # string/pattern features together
        "regex,aho-corasick,url,uuid"
        # serialization combos
        "serde,serde_json,borsh,toml"
        # collection combos
        "indexmap,tinyvec,bitvec,bitflags"
        # defaults combos
        "strict-default,coerce-default,r6-default"
        "strict-default,s7-default,worker-default"
        # diagnostic features (macro-coverage does NOT compile alone — it
        # needs worker-thread; this combo is its only compile gate)
        "growth-debug,worker-thread,macro-coverage"
        # newer integrations (no standalone datafusion combo — heavy; it
        # rides in once via full-integrations in the final combo)
        "jiff"
        "blake3,md5,sha2"
        "globset,zstd"
        "arrow"
        # all features together — derived from the Cargo.toml aggregate so
        # the list can't drift again (includes arrow/datafusion/log)
        "full-integrations,connections,nonapi"
    )
    total=${#combos[@]}
    passed=0
    for combo in "${combos[@]}"; do
        echo "--- Checking: $combo ---"
        if cargo check --manifest-path="$manifest" --features "$combo" 2>&1; then
            # NOT ((passed++)): under set -e that aborts the recipe when
            # passed is 0 — post-increment returns the old value's status.
            passed=$((passed + 1))
        else
            echo "FAILED: $combo"
            exit 1
        fi
    done
    echo ""
    echo "=== All $passed/$total feature combinations passed ==="

# Update Cargo.lock files across every tracked manifest.
# rpkg's lock must stay in tarball-shape: framework crates carry
# `source = "git+url#<sha>"`, no `path+...` sources. We KEEP the dev
# [patch."git+url"] override active so cargo resolves against the local
# workspace (a cross-crate feature/dep rename resolves against the working
# tree, not git@main — #883), then `cargo revendor --stamp-lock` rewrites the
# resulting local (no-`source`) framework entries back to the canonical
# git+url#sha shape. No move-aside / bare-git dance.
# Checksums are NOT stripped: cargo-revendor writes valid `.cargo-checksum.json`
# files matching the registry checksums, so the lock retains `checksum = "..."`.
alias cargo-update := update
[script("bash")]
update *cargo_flags:
    set -euo pipefail
    cargo update {{cargo_flags}}
    cargo update --manifest-path=cargo-revendor/Cargo.toml {{cargo_flags}}
    cargo update --manifest-path=tests/cross-package/shared-traits/Cargo.toml {{cargo_flags}}
    cargo update --manifest-path=tests/cross-package/consumer.pkg/src/rust/Cargo.toml {{cargo_flags}}
    cargo update --manifest-path=tests/cross-package/producer.pkg/src/rust/Cargo.toml {{cargo_flags}}
    cargo update --manifest-path=tests/model_project/src/rust/Cargo.toml {{cargo_flags}}
    rust_dir="{{justfile_directory()}}/rpkg/src/rust"
    # Run from rust_dir so cargo's CWD-relative config discovery picks up
    # rust_dir/.cargo/config.toml's [patch] (resolve against the local
    # workspace, not git@main). Then stamp the canonical git+url#sha back.
    ( cd "$rust_dir" && cargo update {{cargo_flags}} )
    cargo revendor --manifest-path "$rust_dir/Cargo.toml" --stamp-lock -v
    just lock-shape-check

# Restore rpkg/src/rust/Cargo.lock from the git index (preserves staged
# changes — matters for the `just vendor` flow which stages a tarball-shape
# lockfile before commit; if nothing is staged the index equals HEAD, so this
# is effectively a HEAD-restore).
#
# `just check / clippy / build / test / doc / doc-check` invoke cargo with
# `--config "patch.'https://github.com/A2-ai/miniextendr'.<crate>.path=…"`,
# which causes cargo to drop the `source = "git+…#<sha>"` line for each
# workspace sibling. The committed lockfile must keep those lines (CRAN/
# tarball builds resolve `vendor/` against them). The drifting recipes
# auto-chain this restore as their last line (`just test` runs it from an EXIT
# trap so a failing leg cannot skip it); run it manually to clean up if a
# recipe aborted mid-way.
cargo-lock-restore:
    git restore --worktree -- rpkg/src/rust/Cargo.lock

# Check all crates
alias cargo-check := check
check *cargo_flags:
    cargo check --benches --tests --examples --workspace {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/rust-target" cargo check --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/rust-target" cargo check --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && (cd "$root/rpkg/src/rust" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo check --benches --tests --examples --workspace --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})
    cargo check --benches --tests --examples --manifest-path cargo-revendor/Cargo.toml {{cargo_flags}}
    @just cargo-lock-restore

# Build all crates
alias cargo-build := build
build *cargo_flags:
    cargo build --benches --tests --examples --workspace {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/rust-target" cargo build --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/rust-target" cargo build --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && (cd "$root/rpkg/src/rust" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo build --benches --tests --examples --workspace --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})
    @just cargo-lock-restore

# Run clippy on all crates
alias cargo-clippy := clippy
clippy *cargo_flags:
    cargo clippy --benches --tests --examples --workspace {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/rust-target" cargo clippy --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/rust-target" cargo clippy --benches --tests --examples --workspace --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    just clippy-rpkg {{cargo_flags}}
    cargo clippy --benches --tests --examples --manifest-path cargo-revendor/Cargo.toml {{cargo_flags}}
    @just cargo-lock-restore

# Run clippy on rpkg's standalone Rust workspace. Explicit patch overrides keep
# this independent of configure-generated .cargo/config.toml, while the trap
# preserves the committed tarball-shaped lockfile even when clippy fails.
[script("bash")]
clippy-rpkg *cargo_flags:
    set -euo pipefail
    root="{{justfile_directory()}}"
    trap 'git -C "$root" restore --worktree -- rpkg/src/rust/Cargo.lock' EXIT
    cd "$root/rpkg/src/rust"
    CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo clippy --all-targets --workspace \
        --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-api.path=\"$root/miniextendr-api\"" \
        --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-macros.path=\"$root/miniextendr-macros\"" \
        --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-lint.path=\"$root/miniextendr-lint\"" \
        {{cargo_flags}}

# Run miniextendr-lint on rpkg (checks #[miniextendr] consistency)
# The lint runs as a build script; this command triggers it via cargo check.
# Lint output appears as cargo warnings. Errors indicate:
# - Multiple unlabeled impl blocks for the same type
# - Class system incompatibilities between inherent and trait impls
[script("bash")]
lint:
    set -euo pipefail
    root="$(pwd)"
    trap 'git -C "$root" restore --worktree -- rpkg/src/rust/Cargo.lock' EXIT
    cd "$root/rpkg/src/rust"
    output=$(cargo check 2>&1) || {
        echo "$output"
        echo ""
        echo "::error::cargo check failed (see output above)"
        exit 1
    }
    lint_issues=$(echo "$output" | grep -E "warning.*miniextendr.*\[MXL|warning.*miniextendr-lint" || true)
    if [[ -n "$lint_issues" ]]; then
        echo "$lint_issues"
        echo ""
        echo "miniextendr-lint found issues (see above)"
    else
        echo "miniextendr-lint: no issues found"
    fi

# Check documentation builds
alias cargo-doc-check := doc-check
doc-check *cargo_flags: configure-all
    cargo doc --no-deps --document-private-items --workspace {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/rust-target" cargo doc --no-deps --document-private-items --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/rust-target" cargo doc --no-deps --document-private-items --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo doc --no-deps --document-private-items --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})
    cargo doc --no-deps --document-private-items --manifest-path cargo-revendor/Cargo.toml {{cargo_flags}}
    @just cargo-lock-restore

# Build and open documentation
alias cargo-doc := doc
doc *cargo_flags: configure-all
    cargo doc --document-private-items --workspace {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/consumer.pkg/rust-target" cargo doc --document-private-items --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/producer.pkg/rust-target" cargo doc --document-private-items --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo doc --document-private-items --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})
    cargo doc --document-private-items --manifest-path cargo-revendor/Cargo.toml {{cargo_flags}}
    @just cargo-lock-restore
    if command -v open >/dev/null 2>&1; then \
      open rpkg/src/rust/target/doc/rpkg/index.html >/dev/null 2>&1 || \
        echo "doc: unable to open generated docs (skipping)"; \
    else \
      echo "doc: open not found; docs at rpkg/src/rust/target/doc/rpkg/index.html"; \
    fi

# Check formatting
alias cargo-fmt-check := fmt-check
fmt-check *cargo_flags:
    cargo fmt --all {{cargo_flags}} -- --check
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo fmt --all --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}} -- --check)
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo fmt --all --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}} -- --check)
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo fmt --all --manifest-path="$root/rpkg/src/rust/Cargo.toml" {{cargo_flags}} -- --check)
    cargo fmt --all --manifest-path cargo-revendor/Cargo.toml {{cargo_flags}} -- --check

# Format all code
alias cargo-fmt := fmt
fmt *cargo_flags:
    cargo fmt --all {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo fmt --all --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo fmt --all --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo fmt --all --manifest-path="$root/rpkg/src/rust/Cargo.toml" {{cargo_flags}})
    cargo fmt --all --manifest-path cargo-revendor/Cargo.toml {{cargo_flags}}

# Run tests: every cargo leg plus the trybuild UI suites; failures are summarized at the end
#
# Legs: root workspace (UI suites skipped there, see `test-ui`), ndarray
# integration tests, consumer.pkg, producer.pkg, rpkg/src/rust (under the
# path overrides), then `just test-ui`. Every leg runs even when an earlier one
# fails (like CI's `--no-fail-fast` across suites), so one red run shows the
# full failure surface instead of the first casualty; the recipe exits non-zero
# if any leg failed. The rpkg leg rewrites rpkg/src/rust/Cargo.lock under the
# path overrides, so the restore runs in an EXIT trap. (Both used to be trailing
# `&&`-chained lines that a failing leg skipped: the lock stayed dirty and the
# UI snapshots went unchecked locally while CI ran them.)
alias cargo-test := test
[script("bash")]
test *args:
    set -uo pipefail
    cargo_flags=""
    test_args=""
    sep=0
    for arg in {{args}}; do
        if [ "$arg" = "--" ]; then sep=1; continue; fi
        if [ "$sep" = "0" ]; then cargo_flags="$cargo_flags $arg"; else test_args="$test_args $arg"; fi
    done
    root="$(pwd)"
    trap 'just cargo-lock-restore' EXIT

    nfail=0
    failed=""
    leg() {
        local name="$1"; shift
        local status=0
        echo "==> just test: $name"
        "$@" || status=$?
        if [ "$status" -eq 0 ]; then
            echo "<== just test: $name ok"
        else
            echo "<== just test: $name FAILED (exit $status)"
            nfail=$((nfail + 1))
            failed="$failed $name"
        fi
    }
    leg_root() {
        MINIEXTENDR_SKIP_UI=1 cargo test --workspace --no-fail-fast $cargo_flags -- --no-capture $test_args
    }
    leg_ndarray() {
        cargo test -p miniextendr-api --features ndarray --no-fail-fast --test ndarray_generic --test ndarray_all_types --test ndarray_string $cargo_flags -- --no-capture $test_args
    }
    leg_cross() {
        local pkg="$1" tmp
        tmp="$(mktemp -d)"
        (cd "$tmp" && CARGO_TARGET_DIR="$root/tests/cross-package/$pkg/rust-target" cargo test --manifest-path="$root/tests/cross-package/$pkg/src/rust/Cargo.toml" --workspace --no-fail-fast $cargo_flags -- --no-capture $test_args)
    }
    leg_rpkg() {
        (cd "$root/rpkg/src/rust" && CARGO_TARGET_DIR="$root/rpkg/src/rust/target" cargo test --workspace --no-fail-fast $cargo_flags \
            --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-api.path=\"$root/miniextendr-api\"" \
            --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-macros.path=\"$root/miniextendr-macros\"" \
            --config "patch.'https://github.com/A2-ai/miniextendr'.miniextendr-lint.path=\"$root/miniextendr-lint\"" \
            -- --no-capture $test_args)
    }
    leg_ui() {
        just test-ui
    }

    leg "root workspace" leg_root
    leg "ndarray" leg_ndarray
    leg "consumer.pkg" leg_cross consumer.pkg
    leg "producer.pkg" leg_cross producer.pkg
    leg "rpkg/src/rust" leg_rpkg
    leg "test-ui" leg_ui

    echo
    if [ "$nfail" -eq 0 ]; then
        echo "just test: all legs passed"
    else
        echo "just test: $nfail leg(s) FAILED:$failed"
        exit 1
    fi

# Run the trybuild UI snapshot tests deterministically across contributor toolchains.
#
# Toolchains with the `rust-src` component installed render stdlib source spans
# (e.g. `$crate::panicking::panic_fmt(...)`) into trybuild `.stderr` output;
# CI's `dtolnay/rust-toolchain@stable` minimal profile has no `rust-src` and
# renders bare `= note:` fallbacks. The committed `.stderr` baselines are the CI
# (no-rust-src) flavor, so a rust-src-equipped local toolchain reports false
# snapshot mismatches (issue #1239). When rust-src is detected this recipe reruns
# the UI suite under a version-named minimal-profile toolchain (a separate rustup
# toolchain from `stable` even at the same version, hence no rust-src) so results
# match CI. Never regenerate the snapshots (`TRYBUILD=overwrite`) under a rust-src
# toolchain — CI is authoritative. See CLAUDE.md "UI test snapshots".
[script("bash")]
test-ui *cargo_flags:
    set -euo pipefail
    run_ui() {
        local tc="$1"
        env ${tc:+RUSTUP_TOOLCHAIN="$tc"} cargo test -p miniextendr-macros --test ui --locked {{cargo_flags}}
        # `ui_vctrs` is behind the non-default `vctrs` feature, so it needs its
        # own invocation (#1336). Kept separate from `--test ui` so the vctrs
        # feature never leaks into the feature-less `ui` snapshot rendering.
        env ${tc:+RUSTUP_TOOLCHAIN="$tc"} cargo test -p miniextendr-macros --test ui_vctrs --features vctrs --locked {{cargo_flags}}
    }
    if rustup component list --installed 2>/dev/null | grep -q '^rust-src'; then
        ver="$(rustc --version | awk '{print $2}')"
        echo "test-ui: rust-src present on the active toolchain — trybuild would render stdlib spans CI's minimal stable does not (#1239)."
        echo "test-ui: isolating the UI suite under version-named minimal-profile toolchain '${ver}'."
        if ! rustup toolchain list 2>/dev/null | grep -q "^${ver}-"; then
            if ! rustup toolchain install "${ver}" --profile minimal --no-self-update; then
                echo "!! test-ui: could not install toolchain '${ver}' (offline?)."
                echo "!! UI snapshots will NOT match CI on a rust-src toolchain; skipping the trybuild suite."
                echo "!! To run it later: rustup toolchain install ${ver} --profile minimal && just test-ui"
                exit 0
            fi
        fi
        if rustup component list --toolchain "${ver}" --installed 2>/dev/null | grep -q '^rust-src'; then
            echo "!! test-ui: toolchain '${ver}' unexpectedly carries rust-src; cannot isolate. Skipping (CI remains authoritative)."
            exit 0
        fi
        run_ui "${ver}"
    else
        echo "test-ui: no rust-src on the active toolchain (matches CI); running the trybuild UI suite in place."
        run_ui ""
    fi

# Run benchmarks (miniextendr-bench)
alias cargo-bench := bench
bench *cargo_flags:
    cargo bench --manifest-path=miniextendr-bench/Cargo.toml {{cargo_flags}}

# Save structured benchmark baseline (raw text + CSV + metadata)
bench-save *cargo_flags:
    bash tests/perf/bench_baseline.sh save -- {{cargo_flags}}

# Show top benchmarks from most recent baseline
bench-compare *csv_file:
    bash tests/perf/bench_baseline.sh compare {{csv_file}}

# Check for performance regressions between last 2 baselines
bench-drift *flags:
    bash tests/perf/bench_baseline.sh drift {{flags}}

# List saved benchmark baselines with metadata
bench-info:
    bash tests/perf/bench_baseline.sh info

# Check benchmark crate
bench-check *cargo_flags:
    cargo check --manifest-path=miniextendr-bench/Cargo.toml --benches --tests --examples {{cargo_flags}}

# Run macro compile-time benchmark (synthetic crates, measures cold/warm/incremental)
bench-compile *flags:
    bash tests/perf/macro_compile_bench.sh {{flags}}

# Run lint scan benchmark
bench-lint *cargo_flags:
    cargo bench --manifest-path=miniextendr-lint/Cargo.toml --bench lint_scan {{cargo_flags}}

# Run core benchmarks (default features, high-signal targets)
bench-core *cargo_flags:
    cargo bench --manifest-path=miniextendr-bench/Cargo.toml --bench ffi_calls --bench into_r --bench from_r --bench translate --bench strings --bench externalptr --bench worker --bench unwind_protect {{cargo_flags}}

# Run feature-gated benchmarks (connections, rayon, serde)
bench-features *cargo_flags:
    cargo bench --manifest-path=miniextendr-bench/Cargo.toml --features connections,rayon,serde {{cargo_flags}}

# Run full benchmark suite (core + feature matrix)
bench-full *cargo_flags:
    cargo bench --manifest-path=miniextendr-bench/Cargo.toml {{cargo_flags}}
    cargo bench --manifest-path=miniextendr-bench/Cargo.toml --features connections,rayon,serde --bench connections --bench rayon --bench refcount_protect {{cargo_flags}}

# Run R-side benchmarks (requires rpkg installed)
[script("bash")]
bench-r:
    set -euo pipefail
    for f in rpkg/tests/testthat/bench-*.R; do
      echo "=== Running $f ==="
      Rscript "$f"
      echo ""
    done

# Show dependency tree
alias cargo-tree := tree
tree *cargo_flags:
    cargo tree --workspace {{cargo_flags}}
    cargo tree --manifest-path=miniextendr-bench/Cargo.toml {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo tree --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo tree --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo tree --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.crates-io.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.crates-io.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.crates-io.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Expand macros for rpkg (requires cargo-expand)
alias cargo-expand := expand
expand *cargo_flags:
    cargo expand --lib -p miniextendr-api {{cargo_flags}}
    cargo expand --lib -p miniextendr-macros {{cargo_flags}}
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo expand --lib --manifest-path="$root/tests/cross-package/consumer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo expand --lib --manifest-path="$root/tests/cross-package/producer.pkg/src/rust/Cargo.toml" {{cargo_flags}})
    root="$(pwd)" && tmp="$(mktemp -d)" && (cd "$tmp" && cargo expand --lib --manifest-path="$root/rpkg/src/rust/Cargo.toml" --config "patch.crates-io.miniextendr-api.path=\"$root/miniextendr-api\"" --config "patch.crates-io.miniextendr-macros.path=\"$root/miniextendr-macros\"" --config "patch.crates-io.miniextendr-lint.path=\"$root/miniextendr-lint\"" {{cargo_flags}})

# Run ./configure
#
# Generates build configuration files (Makevars, cargo config, etc.) using
# the unified install-mode detection in configure.ac:
#   - source mode (default): cargo resolves miniextendr deps from monorepo
#     siblings via [patch] in .cargo/config.toml. No vendor/.
#   - tarball mode: kicks in automatically when inst/vendor.tar.xz exists
#     (i.e. inside R CMD INSTALL <built-tarball>). Configure unpacks the
#     tarball and writes a vendored source-replacement config.
#
# See docs/CRAN_COMPATIBILITY.md for the full table.
configure:
    cd rpkg && \
    if command -v autoconf >/dev/null 2>&1; then autoconf; else echo "autoconf not found; using existing configure"; fi && \
    bash ./configure

# Skip configure when build outputs are already up to date.
#
# Checks that rpkg/src/Makevars and rpkg/src/rust/.cargo/config.toml both
# exist and are newer than their source inputs (configure.ac, Makevars.in,
# cargo-config.toml.in). If so, skips the ~1 s configure round-trip; otherwise
# falls back to the full `just configure` recipe.
#
# Used by high-iteration dev recipes (devtools-document, devtools-test,
# devtools-load) to avoid re-running configure on every invocation.
#
# CAVEATS (mtime-based, not content-based):
#   - Changes to tools/detect-features.R will NOT bust the cache.
#   - Changes to environment variables (CARGO_FEATURES, CARGO_HOME, etc.)
#     will NOT bust the cache.
#   - Changes to monorepo sibling crates will NOT bust the cache.
#   - For a guaranteed fresh configure, run `just configure` directly or
#     touch rpkg/configure.ac to force the stale check to fail.
configure-fast:
    if [ -f rpkg/src/Makevars ] && \
       [ -f rpkg/src/rust/.cargo/config.toml ] && \
       [ rpkg/src/Makevars -nt rpkg/configure.ac ] && \
       [ rpkg/src/Makevars -nt rpkg/src/Makevars.in ] && \
       [ rpkg/src/rust/.cargo/config.toml -nt rpkg/src/rust/cargo-config.toml.in ]; then \
        echo "configure-fast: up to date — skipping (touch rpkg/configure.ac to force)"; \
    else \
        just configure; \
    fi

# Vendor dependencies into inst/vendor.tar.xz for CRAN release preparation.
# Requires cargo-revendor: `just revendor-install`.
#
# Only needed when producing a build-tarball intended for offline install
# (i.e. before `R CMD build .` for a CRAN submission). Day-to-day dev
# (R CMD INSTALL ., devtools::install/test/load) never calls this recipe.
#
# Steps:
#   1. Run cargo-revendor. With [patch."git+url"] active (written by
#      `just configure` in dev-monorepo mode), cargo-revendor resolves the
#      dependency graph against the LOCAL workspace checkout — so a cross-crate
#      feature/dep rename touching both a framework crate and rpkg resolves
#      against the working tree, not git@main (#883). cargo-revendor then stamps
#      the canonical `git+https://...#<commit>` source back into Cargo.lock
#      (the shape cargo's source-replacement mechanism needs at offline install
#      time), and recomputes `.cargo-checksum.json` with real SHA-256s after
#      CRAN-trim, so the committed Cargo.lock keeps its registry checksums.
#   2. Compress vendor/ to inst/vendor.tar.xz.
#
# This recipe assumes `just configure` already ran (the normal `just configure
# && just vendor` flow), so .cargo/config.toml carries the [patch] override.
# There is no longer any bare-git pre-resolution dance: removing the [patch]
# before resolving was exactly the step that made cross-surface rename PRs
# fail to resolve against git@main and forced the #710 admin-merge (#883).
#
# --force re-runs the full vendor even when cargo-revendor's cache thinks the
# output is current. Cheap insurance against a stale committed tarball when
# the workspace crates have edits that don't bump Cargo.lock.
[script("bash")]
vendor:
    set -euo pipefail
    # cargo-revendor auto-reads [patch."git+url"] from .cargo/config.toml
    # (written by `configure` in dev-monorepo mode) to resolve AND copy
    # miniextendr-{api,lint,macros} from this workspace checkout instead of
    # fetching the git URL. PRs that edit a workspace crate alongside rpkg get
    # their edits reflected in vendor/ and inst/vendor.tar.xz, so
    # `vendor-sync-check` passes and the offline tarball ships the PR's code.
    # `--source-root` is no longer needed here (kept as CLI flag for back-compat).
    #
    # `--json` writes a machine-readable summary (incl. `local_crates`) to
    # STDOUT, which we capture; `-v` progress still streams to STDERR (the
    # terminal). They coexist. We grep `local_crates` rather than shell out to
    # `jq` so the assertion has no extra dependency on CI runners.
    revendor_json="$(cargo revendor \
      --manifest-path rpkg/src/rust/Cargo.toml \
      --output rpkg/vendor \
      --compress rpkg/inst/vendor.tar.xz \
      --blank-md \
      --source-marker \
      --force \
      --json \
      -v)"
    # Defense-in-depth (#876): every framework crate MUST have been vendored
    # from the LOCAL workspace, not fetched from git@main. If configure ran in
    # tarball mode (inst/vendor.tar.xz already present) it wrote a [source]
    # replacement but NO [patch."git+url"] override, so cargo-revendor finds
    # "0 local packages" and silently ships git@main — dropping any in-PR edits
    # to a framework crate (the #865 latch-leak failure mode). Catch it here, at
    # the point of damage, instead of hours later in CI.
    leaked=""
    for crate in miniextendr-api miniextendr-lint miniextendr-macros; do
      # `local_crates` is a pretty-printed JSON array, one "name" per line.
      if ! grep -qE "\"$crate\"" <<<"$revendor_json"; then
        leaked="$leaked $crate"
      fi
    done
    if [[ -n "$leaked" ]]; then
      echo "" >&2
      echo "ERROR: framework crate(s)$leaked were vendored from git, not the local workspace." >&2
      echo "" >&2
      echo "cargo-revendor reported these crates as NOT local — they came from the" >&2
      echo "git URL pinned in Cargo.lock (git@main), so any in-PR edits to them were" >&2
      echo "silently dropped from rpkg/inst/vendor.tar.xz (#865 latch leak)." >&2
      echo "" >&2
      echo "Almost always: configure ran in tarball mode (rpkg/inst/vendor.tar.xz" >&2
      echo "was present) so it wrote a [source] replacement but no [patch] override." >&2
      echo "" >&2
      echo "Fix:  just clean-vendor-leak && just configure && just vendor" >&2
      echo "" >&2
      echo "See CLAUDE.md \"The latch leak\" and #876." >&2
      exit 1
    fi
    echo ""
    echo "Vendored framework crates from local workspace: miniextendr-{api,lint,macros}"
    echo "Created rpkg/inst/vendor.tar.xz — DELETE THIS BEFORE RESUMING DEV ITERATION"
    echo "(run 'just clean-vendor-leak' or 'unlink(\"rpkg/inst/vendor.tar.xz\")' in R)"

# Remove a leaked rpkg/inst/vendor.tar.xz.
# inst/vendor.tar.xz is the single signal that flips configure into tarball mode.
# A leftover tarball makes subsequent dev installs silently use stale vendored
# sources instead of workspace edits. Run this if you're seeing
# "tarball install — vendor/ already populated" during dev iteration and you
# didn't intend to stay in tarball mode.
[script("bash")]
clean-vendor-leak:
    set -euo pipefail
    if [ -f rpkg/inst/vendor.tar.xz ]; then
        rm -f rpkg/inst/vendor.tar.xz
        echo "Removed rpkg/inst/vendor.tar.xz (tarball-mode leak)."
    else
        echo "No tarball leak to clean."
    fi

# Internal: abort if rpkg/inst/vendor.tar.xz is present.
# Used as a dep by dev-consume recipes (rcmdinstall, devtools-test,
# devtools-load, devtools-install) that would silently do the wrong thing
# in tarball mode. NOT wired to producer recipes (r-cmd-build, r-cmd-check,
# devtools-build) which intentionally create the tarball and trap-clean it.
# See CLAUDE.md "Vendor tarball is a latch" for context.
[private]
[script("bash")]
_assert-no-vendor-leak:
    set -euo pipefail
    if [ -f rpkg/inst/vendor.tar.xz ]; then
        printf '%s\n' \
          "error: rpkg/inst/vendor.tar.xz is present in the source tree." \
          "" \
          "This usually means a previous build/smoke leaked it. configure" \
          "now flips into tarball mode and may ignore source edits /" \
          "monorepo [patch.\"git+url\"] propagation, making dev iteration unreliable." \
          "" \
          "Fix:  just clean-vendor-leak" \
          "" \
          "Or:   rm rpkg/inst/vendor.tar.xz && just configure" \
          "" \
          "See CLAUDE.md \"Vendor tarball is a latch\" for context." \
          >&2
        exit 1
    fi

# Verify committed Cargo.lock, vendor/, and vendor.tar.xz agree (#157).
# Runs in CI/pre-release to guarantee the offline build artifact is fresh.
vendor-verify:
    cargo run --manifest-path cargo-revendor/Cargo.toml --quiet -- \
      --verify \
      --manifest-path rpkg/src/rust/Cargo.toml \
      --output rpkg/vendor \
      --compress rpkg/inst/vendor.tar.xz \
      -v

# Regression test: bootstrap.R must produce inst/vendor.tar.xz from a clean
# source tree. Catches the #439/#440 regression where a leftover tarball was
# bundled instead of a freshly-bootstrapped one. Requires cargo-revendor on PATH
# and pkgbuild installed in R; skips with a clear message if either is missing.
# See tests/bootstrap-produces-vendor.sh and #441.
#
# Also runs the #876 loud-fail regression: `just vendor` must exit non-zero when
# a framework crate would be vendored from git instead of the local workspace.
# See tests/vendor-loud-fail.sh.
#
# And the #883 cross-surface regression: a PR that adds a framework feature and
# references it from rpkg in the same commit must vendor without admin-merge —
# the framework crate resolves against the local workspace, not git@main.
# See tests/vendor-cross-surface-rename.sh.
test-bootstrap-vendor:
    bash tests/bootstrap-produces-vendor.sh
    bash tests/vendor-loud-fail.sh
    bash tests/vendor-cross-surface-rename.sh

# Full-suite gctorture2 sweep over rpkg's testthat suite (slow — nightly CI).
#
# Catches the long-tail use-after-free class the fast per-function
# gctorture(TRUE) sweep misses: any SEXP reachable through Rust state but not
# rooted in R's protect mechanism. The strict-glibc R-release runner aborts on
# these; other runtimes silently corrupt and "pass". See docs/GCTORTURE_TESTING.md.
#
# Installs rpkg into a throwaway library (the harness does `library(miniextendr)`
# — devtools::load_all is unsafe under gctorture per the doc's pitfall #1), then
# runs scripts/gctorture-full-sweep.R at step=100. The FULL suite at step=100
# needs ~11 hours (measured in CI, 2026-08) — set MINIEXTENDR_GCTORTURE_SHARD=k/n
# or a bigger STEP for local bisects. Wired as a sharded scheduled job, not a
# PR gate (.github/workflows/gctorture-nightly.yml).
#
# Override step with the STEP arg (step=10 for a faster bisect, step=1 = full
# gctorture(TRUE)). Exits non-zero and lists the offending test files on failure.
#
# Full-suite gctorture2 sweep over rpkg tests (slow; nightly CI)
[script("bash")]
gctorture-full STEP="100": _assert-no-vendor-leak configure
    set -euo pipefail
    libdir="$(mktemp -d)"
    # The source-mode `R CMD INSTALL` below drifts rpkg/src/rust/Cargo.lock; restore
    # it (and clean the temp lib) on every exit path, incl. a failed sweep. (#1052)
    trap 'rm -rf "$libdir"; just cargo-lock-restore' EXIT
    R CMD INSTALL --library="$libdir" rpkg
    # Hand the throwaway library to the sweep via env var — the script prepends
    # it to .libPaths() INSIDE the session. An R_LIBS_USER override here never
    # survives startup: locally the repo .Rprofile (rv activation) replaces
    # .libPaths() wholesale, and in CI overriding R_LIBS_USER hides the
    # runner's dependency library (setup-r-dependencies installs there),
    # breaking loadNamespace with "there is no package called 'R6'".
    MINIEXTENDR_GCTORTURE_LIB="$libdir" Rscript scripts/gctorture-full-sweep.R rpkg/tests/testthat {{STEP}}

# Deliberate lock updates go through `just update` / `just vendor` (they re-stamp
# tarball-shape); the R-side dev recipes below only DRIFT rpkg/src/rust/Cargo.lock
# as a source-mode build side effect, so each auto-chains `just cargo-lock-restore`
# to self-clean (matching check/build/clippy). devtools-test restores even on a red
# test via a trap, so a failing suite never leaves the papercut behind. (#1052)
# Load and test rpkg with devtools
[script("bash")]
devtools-test FILTER="": _assert-no-vendor-leak devtools-document
    set -euo pipefail
    trap 'just cargo-lock-restore' EXIT
    if [ -z "{{FILTER}}" ]; then
      Rscript -e 'testthat::set_max_fails(Inf); devtools::test("rpkg")'
    else
      Rscript -e 'testthat::set_max_fails(Inf); devtools::test("rpkg", filter = "{{FILTER}}")'
    fi

# Load rpkg with devtools::load_all
alias devtools-load_all := devtools-load
devtools-load: _assert-no-vendor-leak devtools-document
    Rscript -e 'devtools::load_all("rpkg")'
    @just cargo-lock-restore

# Install rpkg with devtools::install
devtools-install: _assert-no-vendor-leak devtools-document
    Rscript -e 'devtools::install("rpkg")'
    @just cargo-lock-restore

# Install R dependencies used by the repo (devtools, roxygen2, testthat, R6, S7, vctrs, etc.)
install_deps:
    Rscript -e 'install.packages(c("devtools","roxygen2","rcmdcheck","pkgbuild","processx","testthat","R6","S7","vctrs"), repos = "https://cloud.r-project.org")'

# Install Rust dev tools used for day-to-day iteration.
# cargo-limit: provides `cargo lcheck`/`lclippy`/`ltest`/`lbuild` that surface the first
# few errors/warnings without dumping thousands of lines of "noise" into the terminal.
# Prefer these aliases in CLI one-offs; CI still runs full `cargo clippy ... -D warnings`.
# cargo-revendor: required by `just configure` in dev-monorepo mode to sync
# workspace crates into rpkg/vendor/ (otherwise cargo metadata fails on the
# frozen path = "../../vendor/..." deps in rpkg/src/rust/Cargo.toml).
dev-tools-install: revendor-install
    cargo install cargo-limit

# Symlinks rv's cached deps from ~/.cache/rv (warm from main, same rproject.toml)
# into THIS checkout's own rv/library in seconds — no recompile/download — so
# parallel worktrees never race on a shared install and main's library stays
# untouched. Caveat: `rv sync` replaces the library with exactly the lockfile, so
# it prunes miniextendr/minirextendr (dependencies_only) — re-install after any
# resync. Never `ln -s rv/library` to main. See CLAUDE.md → "Agent worktrees".
# Bootstrap a fresh worktree's R library from rv's shared cache (run before configure/install)
worktree-sync:
    RV_LINK_MODE=symlink rv sync

# Install Windows system tooling via Scoop (https://scoop.sh). Currently installs
# `rv` (https://github.com/a2-ai/rv), the R version manager used on this repo.
[windows]
install-deps-windows:
    powershell.exe -NoProfile -Command "scoop install rv"

# Install minirextendr dependencies (for scaffolding helper package)
minirextendr-install-deps:
    Rscript -e 'install.packages(c("cli","curl","desc","fs","gh","glue","rappdirs","rlang","rprojroot","usethis","withr","devtools","roxygen2","testthat"), repos = "https://cloud.r-project.org")'

# Build rpkg with devtools::build
# Depends on `vendor` for the same reason as r-cmd-build — devtools::build
# wraps R CMD build, and the resulting tarball is meaningful only with
# inst/vendor.tar.xz inside.
# Cleanup: inst/vendor.tar.xz is removed on exit so the source tree is never
# left in tarball mode after this recipe finishes.
[script("bash")]
devtools-build: configure vendor
    set -euo pipefail
    trap 'rm -f rpkg/inst/vendor.tar.xz' EXIT
    Rscript -e 'devtools::build("rpkg")'

# No _assert-no-vendor-leak dep — devtools::check internally calls
# pkgbuild::build, which legitimately produces inst/vendor.tar.xz mid-run.
# The guard would fire spuriously. Same for `test-r-build`.
# Check rpkg with devtools::check
# error_on = "error" matches CI behavior (ignore warnings/notes)
# check_dir preserves output for investigation (not auto-cleaned)
devtools-check: devtools-document
    Rscript -e 'devtools::check("rpkg", error_on = "error", check_dir = "{{check_output_dir}}")'

# Document rpkg with devtools::document (roxygen2 → NAMESPACE + man pages).
# R wrappers are generated automatically by Makevars during R CMD INSTALL.
#
# Short-circuits via roxygen2::needs_roxygenize() — skips the full roxygenize
# pass when no R source, NAMESPACE, or man/* mtime changes are detected.
#
# CAVEAT — macro-layer cache misses: needs_roxygenize() tracks R-source mtimes
# only. When the proc-macro changes how it serialises a tag (e.g. a new wrapper
# attribute), those changes flow through rcmdinstall → wrappers.R mtime update,
# which roxygen2 does detect. However, if you change miniextendr-macros/** and
# then run devtools-document WITHOUT first running rcmdinstall, the skip fires
# incorrectly. Rule: after any Rust/macro change, always run:
#   just rcmdinstall && just force-document
devtools-document: configure-fast
    Rscript -e 'if (roxygen2::needs_roxygenize("rpkg")) { devtools::document("rpkg") } else { cat("devtools-document: man/ up to date — skipping (use `just force-document` to override)\n") }'
    @just cargo-lock-restore

# Force-document rpkg unconditionally — bypasses the needs_roxygenize() check.
# Use after Rust/macro changes where the mtime cache may not have caught up:
#   just rcmdinstall && just force-document
force-document: configure-fast
    Rscript -e 'devtools::document("rpkg")'
    @just cargo-lock-restore

# Document ALL R packages in the workspace
# This includes: rpkg, minirextendr, and cross-package test packages
document-all: devtools-document minirextendr-document
    cd tests/cross-package && just document-all

# Configure ALL R packages that need vendoring/configure
# This includes: rpkg and cross-package test packages (minirextendr has no configure step)
configure-all: configure cross-configure

alias rcmdinstall := r-cmd-install
r-cmd-install *args: _assert-no-vendor-leak configure
    R CMD INSTALL {{args}} rpkg
    @just cargo-lock-restore

# Build R package tarball
# Depends on `r-cmd-install` so the host wrapper-gen pass regenerates the UNTRACKED
# generated files (R/miniextendr-wrappers.R + src/rust/wasm_registry.rs) on disk
# before `R CMD build` packs them into the tarball. These are gitignored, so the
# only way they reach the tarball is from disk — and a stale/absent wasm_registry.rs
# silently breaks wasm-from-tarball installs (the wasm cdylib is a SIDE_MODULE host
# R cannot dyn.load, so there is NO wasm-side regeneration fallback). r-cmd-install
# runs in source mode (its _assert-no-vendor-leak dep refuses if a tarball exists),
# so it must complete before `vendor` seals inst/vendor.tar.xz.
#
# Also depends (transitively, via r-cmd-install) on `vendor` so the tarball ships
# inst/vendor.tar.xz, which is what triggers tarball-mode install (offline, vendored
# sources). Without this a maintainer can silently produce a tarball that
# source-mode-installs over the network — defeating the point for CRAN submission.
#
# Cleanup: rpkg/inst/vendor.tar.xz must be removed after R CMD build copies
# it into the built tarball. Otherwise the leftover in the source tree makes
# the next `just rcmdinstall` / `devtools::install("rpkg")` silently switch
# to tarball mode (configure's only signal is `[ -f inst/vendor.tar.xz ]`),
# which freezes out monorepo workspace-crate edits via `[patch."git+url"]`.
alias rcmdbuild := r-cmd-build
[script("bash")]
r-cmd-build *args: r-cmd-install vendor
    set -euo pipefail
    trap 'rm -f rpkg/inst/vendor.tar.xz' EXIT
    R CMD build {{args}} --no-manual --log --debug rpkg

# Run R CMD check on rpkg
# Depends on `r-cmd-install` (regenerates the untracked generated files into the
# source tree before the tarball is built — see r-cmd-build for the full rationale)
# and `vendor` (ensures inst/vendor.tar.xz ships so the tarball installs offline).
# R CMD check copies the tarball to a temp dir where monorepo [patch] paths
# are unavailable — configure detects this and uses vendored sources instead.
#
# Cleanup: same reason as r-cmd-build — see comment there.
alias rcmdcheck := r-cmd-check
[script("bash")]
r-cmd-check *args: r-cmd-install vendor
    set -euo pipefail
    trap 'rm -f rpkg/inst/vendor.tar.xz' EXIT
    ERROR_ON="warning"
    CHECK_DIR=""
    for arg in {{args}}; do
      case "$arg" in
        ERROR_ON=*) ERROR_ON="${arg#ERROR_ON=}" ;;
        CHECK_DIR=*) CHECK_DIR="${arg#CHECK_DIR=}" ;;
        *) echo "Ignoring unknown arg '$arg'" ;;
      esac
    done
    CHECK_DIR_ARG="NULL"
    if [ -n "$CHECK_DIR" ]; then
      case "$CHECK_DIR" in
        /*) CHECK_DIR_ARG="'$CHECK_DIR'" ;;
        *)  CHECK_DIR_ARG="'$(pwd)/$CHECK_DIR'" ;;
      esac
    fi
    Rscript -e "rcmdcheck::rcmdcheck('rpkg', args = c('--as-cran','--no-manual'), error_on = '${ERROR_ON}', check_dir = ${CHECK_DIR_ARG})"

# Extract and inspect R package tarball contents (for debugging build artifacts)
#
# Builds tarball with --compression=none and extracts to rpkg_build/ for inspection.
# Useful for verifying what gets included in CRAN submissions.
[script("bash")]
test-r-build: configure
    set -euo pipefail
    # MSYS2 tar interprets D: as remote host; --force-local treats all as local
    TAR_FORCE_LOCAL=""
    case "$(uname -s)" in MSYS*|MINGW*|CYGWIN*) TAR_FORCE_LOCAL="--force-local";; esac
    # Extract package info from DESCRIPTION
    pkg=$(Rscript -e 'd <- read.dcf("rpkg/DESCRIPTION")[1,]; cat(d[["Package"]])')
    ver=$(Rscript -e 'd <- read.dcf("rpkg/DESCRIPTION")[1,]; cat(d[["Version"]])')
    # Build tarball
    R CMD build --compression=none rpkg
    # Determine tarball name (.tar or .tar.gz)
    tarball="${pkg}_${ver}.tar"
    [[ -f "$tarball" ]] || tarball="${tarball}.gz"
    # Extract for inspection
    out_dir="rpkg_build/${pkg}_${ver}"
    mkdir -p "$out_dir"
    (cd "$out_dir" && tar $TAR_FORCE_LOCAL -xf "$tarball" --strip-components=1)
    echo "Extracted to: $out_dir"

# ============================================================================
# minirextendr R package development
# ============================================================================

# Generate documentation for minirextendr R package
minirextendr-document:
    Rscript -e 'devtools::document("minirextendr")'

# Run tests for minirextendr R package
[script("bash")]
minirextendr-test FILTER="":
    export MINIEXTENDR_LOCAL_PATH="$(pwd)"
    if [ -z "{{FILTER}}" ]; then
      Rscript -e 'testthat::set_max_fails(Inf); devtools::test("minirextendr")'
    else
      Rscript -e 'testthat::set_max_fails(Inf); devtools::test("minirextendr", filter = "{{FILTER}}")'
    fi

# Check minirextendr R package with devtools::check
minirextendr-check:
    MINIEXTENDR_LOCAL_PATH="$(pwd)" Rscript -e 'devtools::check("minirextendr", error_on = "error")'

# Install minirextendr R package with devtools::install
minirextendr-install:
    Rscript -e 'devtools::install("minirextendr")'

# Load minirextendr with devtools::load_all (for interactive development)
minirextendr-load:
    Rscript -e 'devtools::load_all("minirextendr")'

# Build minirextendr R package tarball
minirextendr-build:
    R CMD build --no-manual minirextendr

# Run R CMD check on minirextendr package
[script("bash")]
minirextendr-rcmdcheck:
    export MINIEXTENDR_LOCAL_PATH="$(pwd)"
    Rscript -e "rcmdcheck::rcmdcheck('minirextendr', args = c('--no-manual'), error_on = 'warning')"

# Standalone round-trip regression (heavy; maintainer/nightly only).
#
# Drives the gated testthat e2e tests in minirextendr/tests/testthat/test-templates.R
# — including the #757/#775 standalone build → wrapper-gen → library() round-trip
# at :521 — by flipping the MINIEXTENDR_RUN_E2E bypass so skip_e2e() does NOT
# skip_on_ci(). MINIEXTENDR_LOCAL_PATH points the scaffolder at this checkout.
#
# Cold-compiles a Rust crate, runs autoconf, and (for the standalone test)
# network-fetches + vendors miniextendr `main` via cargo-revendor — minutes per
# run, network-dependent. NOT part of the per-PR suite; runs in the nightly /
# label-gated CI job `r-roundtrip-e2e` (see .github/workflows/ci.yml). Requires
# cargo, autoconf, R, and cargo-revendor on PATH; individual tests self-skip with
# a clear reason if a tool is missing. See #775 / #805.
[script("bash")]
minirextendr-roundtrip:
    export MINIEXTENDR_LOCAL_PATH="$(pwd)"
    export MINIEXTENDR_RUN_E2E=1
    Rscript -e 'testthat::set_max_fails(Inf); devtools::test("minirextendr", filter = "templates")'

# Full development cycle for minirextendr: document, test, check
minirextendr-dev: minirextendr-document minirextendr-test minirextendr-check

# ============================================================================
# Cross-package trait dispatch testing (tests/cross-package)
# ============================================================================

cross-document:
    cd tests/cross-package && just document-all

cross-configure:
    cd tests/cross-package && just configure-all

alias cross-build := cross-install
cross-install:
    cd tests/cross-package && just install-all

cross-test:
    cd tests/cross-package && just test-all

cross-check:
    cd tests/cross-package && just check-all

cross-clean:
    cd tests/cross-package && just clean

cross-dev:
    cd tests/cross-package && just dev

# ============================================================================
# Templates / drift check
# ============================================================================
#
# Pattern:
# - upstream snapshot   : built from sources within this repo (see templates-sources)
# - inst/templates/**   : your edited copies
# - patches/templates.patch : the *approved* delta
#
# Workflow:
#   just templates-check         # fails if inst/templates drift beyond approved patch
#   just templates-approve       # accept current delta as approved (regen patch)

local_root  := "minirextendr/inst/templates"
patch_file  := "patches/templates.patch"

# Configure your upstream locations here.
#
# Use TAB-separated pairs: <relative/path/in/templates>\t<source/path>
# - For a directory source, end BOTH sides with a trailing slash.
# - Paths with spaces are OK (TAB is the separator).
#
# The templates are scaffolding for new packages. The rpkg files are the
# "upstream" source, and templates may have intentional differences like
# {{package_rs}} placeholders. The patch file captures approved differences.

[script("bash")]
templates-sources:
    set -euo pipefail

    # Two template types exist:
    #   - rpkg/          : Standalone R package template
    #   - monorepo/      : Rust workspace with embedded R package
    #
    # Only include files where rpkg is the source of truth.
    cat <<'EOF'
    # rel	src
    # === R Package Template (rpkg/) ===
    rpkg/bootstrap.R	rpkg/bootstrap.R
    rpkg/build.rs	rpkg/src/rust/build.rs
    rpkg/cleanup	rpkg/cleanup
    rpkg/cleanup.ucrt	rpkg/cleanup.ucrt
    rpkg/cleanup.win	rpkg/cleanup.win
    rpkg/configure.ac	rpkg/configure.ac
    rpkg/configure.ucrt	rpkg/configure.ucrt
    rpkg/configure.win	rpkg/configure.win
    rpkg/gitignore	rpkg/.gitignore
    rpkg/Makevars.in	rpkg/src/Makevars.in
    rpkg/Makevars.win	rpkg/src/Makevars.win
    rpkg/stub.c	rpkg/src/stub.c
    rpkg/tools/detect-features.R	rpkg/tools/detect-features.R
    rpkg/tools/lock-shape-check.R	rpkg/tools/lock-shape-check.R
    rpkg/win.def.in	rpkg/src/win.def.in
    # === Monorepo Template (monorepo/) ===
    # The embedded R package uses same sources as rpkg/ template
    monorepo/rpkg/bootstrap.R	rpkg/bootstrap.R
    monorepo/rpkg/build.rs	rpkg/src/rust/build.rs
    monorepo/rpkg/cleanup	rpkg/cleanup
    monorepo/rpkg/cleanup.ucrt	rpkg/cleanup.ucrt
    monorepo/rpkg/cleanup.win	rpkg/cleanup.win
    monorepo/rpkg/configure.ac	rpkg/configure.ac
    monorepo/rpkg/configure.ucrt	rpkg/configure.ucrt
    monorepo/rpkg/configure.win	rpkg/configure.win
    monorepo/rpkg/gitignore	rpkg/.gitignore
    monorepo/rpkg/Makevars.in	rpkg/src/Makevars.in
    monorepo/rpkg/Makevars.win	rpkg/src/Makevars.win
    monorepo/rpkg/stub.c	rpkg/src/stub.c
    monorepo/rpkg/tools/detect-features.R	rpkg/tools/detect-features.R
    monorepo/rpkg/tools/lock-shape-check.R	rpkg/tools/lock-shape-check.R
    monorepo/rpkg/win.def.in	rpkg/src/win.def.in
    EOF

# Internal helper: populate an upstream snapshot into DEST.
# The snapshot is a tree laid out to match inst/templates.
[script("bash")]
_templates-upstream-populate dest:
    set -euo pipefail

    dest="{{dest}}"
    mkdir -p "$dest"

    manifest="$(just --quiet templates-sources)"

    add() {
      local rel="$1" src="$2" dst="$dest/$rel"
      if [[ "$rel" == */ ]]; then
        mkdir -p "$dst"
        rsync -a "$src" "$dst"
      else
        mkdir -p "$(dirname "$dst")"
        cp -a "$src" "$dst"
      fi
    }

    while IFS=$'\t' read -r rel src; do
      [[ -z "${rel:-}" ]] && continue

      rel="$(printf '%s' "$rel" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
      src="$(printf '%s' "$src" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"

      [[ -z "$rel" ]] && continue
      [[ "$rel" == \#* ]] && continue

      if [[ -z "$src" ]]; then
        echo "_templates-upstream-populate: missing source path for rel='$rel'" >&2
        exit 2
      fi
      if [[ ! -e "$src" ]]; then
        echo "_templates-upstream-populate: source not found: $src (for rel='$rel')" >&2
        exit 2
      fi

      # Disallow absolute paths to keep this repo-portable
      if [[ "$src" = /* ]]; then
        echo "_templates-upstream-populate: absolute paths are not allowed (got: $src)" >&2
        exit 2
      fi

      add "$rel" "$src"
    done <<<"$manifest"

# Audit the end-user Claude skill set (minirextendr/inst/claude/skills/)
# against a freshly scaffolded package: cited paths must exist in the scaffold
# layout; cited symbols must exist in this repo's sources. Installs
# minirextendr into a temp library (pure R, no compile), scaffolds a throwaway
# package, then runs scripts/skill-freshness-audit.sh --user-layout on it.
# Requires minirextendr's Imports (cli, fs, usethis) plus cargo on PATH.
[script("bash")]
user-skills-check:
    set -euo pipefail
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp/lib"
    R CMD INSTALL --no-multiarch --library="$tmp/lib" minirextendr >/dev/null 2>&1
    # Prepend the temp lib INSIDE the R session, not via R_LIBS: rv's
    # .Rprofile activation replaces .libPaths() wholesale (activate.R uses
    # `.libPaths(rv_lib, include.site = FALSE)`), which silently dropped an
    # R_LIBS entry in any rv-activated checkout — the recipe then failed with
    # "there is no package called 'minirextendr'". The prepend runs after
    # .Rprofile, so it works both locally (rv lib supplies cli/fs/usethis)
    # and in CI (no rv activation; default libs supply them).
    Rscript -e ".libPaths(c('$tmp/lib', .libPaths())); minirextendr::create_miniextendr_package(file.path('$tmp', 'skillcheck.pkg'), open = FALSE)"
    bash scripts/skill-freshness-audit.sh --user-layout "$tmp/skillcheck.pkg"

# Accept the current delta as approved by regenerating patches/templates.patch
# (Builds an upstream snapshot from templates-sources before diffing.)
[script("bash")]
templates-approve:
    set -euo pipefail

    mkdir -p "$(dirname "{{patch_file}}")"

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp/a" "$tmp/b"

    just _templates-upstream-populate "$tmp/a"

    # Populate b/ with template versions of same files (not entire template dir)
    manifest="$(just --quiet templates-sources)"
    while IFS=$'\t' read -r rel src; do
      [[ -z "${rel:-}" ]] && continue
      rel="$(printf '%s' "$rel" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
      [[ -z "$rel" ]] && continue
      [[ "$rel" == \#* ]] && continue

      template_file="{{local_root}}/$rel"
      if [[ -e "$template_file" ]]; then
        mkdir -p "$(dirname "$tmp/b/$rel")"
        cp -a "$template_file" "$tmp/b/$rel"
      fi
    done <<<"$manifest"

    # diff exits 1 when differences exist; that's expected here.
    # -U2 = 2 context lines (default is 3)
    (cd "$tmp" && diff -ruN -U2 a b) > "{{patch_file}}" || true
    echo "Wrote {{patch_file}}"

# Verify: upstream snapshot + approved patch == inst/templates
# - exits nonzero on drift
# - exits nonzero if the patch no longer applies cleanly
[script("bash")]
templates-check:
    set -euo pipefail

    test -f "{{patch_file}}"

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT

    just _templates-upstream-populate "$tmp"

    # Apply approved delta (no-op if patch is empty)
    if [[ -s "{{patch_file}}" ]]; then
      patch -d "$tmp" -p1 --forward --batch < "{{patch_file}}" >/dev/null
    fi

    # Compare only files defined in templates-sources (not entire templates dir)
    tmp2="$(mktemp -d)"
    trap 'rm -rf "$tmp" "$tmp2"' EXIT

    manifest="$(just --quiet templates-sources)"
    while IFS=$'\t' read -r rel src; do
      [[ -z "${rel:-}" ]] && continue
      rel="$(printf '%s' "$rel" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
      [[ -z "$rel" ]] && continue
      [[ "$rel" == \#* ]] && continue

      template_file="{{local_root}}/$rel"
      if [[ -e "$template_file" ]]; then
        mkdir -p "$(dirname "$tmp2/$rel")"
        cp -a "$template_file" "$tmp2/$rel"
      fi
    done <<<"$manifest"

    diff -ruN "$tmp" "$tmp2"

# Check the CLAUDE.md <-> AGENTS.md sibling invariant (exits nonzero on violation)
agents-md-check:
    bash scripts/agents-md-check.sh

# Refresh the local, untracked GitHub issue cache. Pass an alternate output
# directory when refreshing the main checkout from an agent worktree.
issues-refresh output_dir="ISSUES":
    bash scripts/refresh-issues-cache.sh "{{output_dir}}"

# ==============================================================================
# Vendor sync check (ensure vendored crates match workspace)
# ==============================================================================
# After `just vendor`, rpkg/vendor/ should contain synced copies of workspace
# crates (miniextendr-api, miniextendr-macros, etc.) alongside external deps.
#
# This check verifies the vendored copies haven't drifted from the workspace.
# Run `just vendor` to refresh if this check fails.

# Check that vendored miniextendr crates match workspace sources
[script("bash")]
vendor-sync-check:
    set -euo pipefail

    vendor_dir="rpkg/vendor"
    drift_found=0

    if [[ ! -d "$vendor_dir" ]]; then
      echo "WARNING: $vendor_dir not present (run 'just configure' first) — nothing to check."
      exit 0
    fi

    for crate in miniextendr-api miniextendr-macros miniextendr-lint miniextendr-engine; do
      # Accept both flat (vendor/<name>/) and versioned (vendor/<name>-<version>/) layouts
      crate_dir=""
      if [[ -d "$vendor_dir/$crate" ]]; then
        crate_dir="$vendor_dir/$crate"
      else
        versioned=$(find "$vendor_dir" -maxdepth 1 -type d -name "${crate}-[0-9]*" | head -1)
        if [[ -n "$versioned" ]]; then
          crate_dir="$versioned"
        fi
      fi

      if [[ -z "$crate_dir" ]]; then
        echo "WARNING: $vendor_dir/$crate (or versioned) not found (run 'just configure' first)"
        continue
      fi

      # Compare src directories (the actual code)
      if ! diff -rq "$crate/src" "$crate_dir/src" >/dev/null 2>&1; then
        echo "DRIFT: $crate/src differs from $crate_dir/src"
        drift_found=1
      fi
    done

    if [[ $drift_found -eq 1 ]]; then
      echo ""
      echo "Vendored crates have drifted from workspace sources."
      echo "Run 'just vendor' to refresh vendored copies."
      exit 1
    else
      echo "Vendor sync check passed: all miniextendr crates match."
    fi

# Verify lint crate compiles against the current miniextendr-macros parser
lint-sync-check:
    cargo check --manifest-path=miniextendr-lint/Cargo.toml

# Verify the committed NAMESPACE / man are in sync with #[miniextendr] Rust source,
# and that the host wrapper-gen pass produces non-stub generated files.
#
# `R/miniextendr-wrappers.R` and `src/rust/wasm_registry.rs` are NOT tracked
# (gitignored, regenerated every install) so there is no committed copy to drift
# — we instead assert the install produced them non-empty / non-stub. NAMESPACE
# and man/ are still committed (and derive from the freshly-regenerated wrappers.R),
# so those are git-diffed. Catches the failure mode where a Rust-side edit ships
# without the corresponding regenerated NAMESPACE / man/ (see #602).
[script("bash")]
wrappers-sync-check: _assert-no-vendor-leak configure
    set -euo pipefail
    # The source-mode `R CMD INSTALL` below drifts rpkg/src/rust/Cargo.lock; restore
    # it on every exit path, incl. the exit-1 diff-failure branches. (#1052)
    install_log="$(mktemp)"
    trap 'just cargo-lock-restore; rm -f "$install_log"' EXIT

    # Install regenerates R/miniextendr-wrappers.R + src/rust/wasm_registry.rs on disk.
    # Capture output (instead of >/dev/null) so the audit grep-gate below can scan
    # it; print the log only if the install itself fails, to preserve diagnostics.
    if ! R CMD INSTALL rpkg >"$install_log" 2>&1; then
      cat "$install_log" >&2
      exit 1
    fi

    # Regression guard (2026-07-12 audit): the impl-method roxygen-tag deprecation
    # nudge (`miniextendr: @<tag> on impl block ...`, emitted by
    # miniextendr-macros/src/roxygen.rs) must not reappear from an rpkg fixture.
    # Scoped to our macro's unique prefix so upstream cargo/rustc future-incompat
    # noise (e.g. proc-macro-error2) can't flake this gate.
    # Cold-build gate: the nudge is a rustc deprecation warning emitted only when
    # the rpkg crate recompiles — a warm cargo cache skips rustc, but introducing a
    # tag IS a source change forcing recompile, and CI Sync Checks always starts cold.
    if grep -n 'miniextendr: @' "$install_log" >&2; then
      echo "" >&2
      echo "ERROR: impl-method roxygen-tag deprecation nudge fired during R CMD INSTALL (above)." >&2
      echo "Move each flagged @tag off the impl block onto its method (see rpkg/src/rust/impl_dots_tests.rs)." >&2
      exit 1
    fi

    # The generated (untracked) files must exist and be real, not stubs.
    if [ ! -s rpkg/R/miniextendr-wrappers.R ]; then
      echo "ERROR: R CMD INSTALL did not produce rpkg/R/miniextendr-wrappers.R." >&2
      exit 1
    fi
    wasm_hash=$(grep 'content-hash:' rpkg/src/rust/wasm_registry.rs | awk '{print $NF}')
    if [ ! -s rpkg/src/rust/wasm_registry.rs ] || [ "$wasm_hash" = "0000000000000000" ]; then
      echo "ERROR: rpkg/src/rust/wasm_registry.rs is missing or a stub (content-hash=$wasm_hash)." >&2
      echo "The wrapper-gen pass must produce a real wasm32 snapshot." >&2
      exit 1
    fi

    # NAMESPACE / man derive from the freshly-regenerated wrappers.R — regenerate
    # and verify the committed copies match.
    Rscript -e 'devtools::document("rpkg")' >/dev/null

    if ! git diff --exit-code -- rpkg/NAMESPACE rpkg/man; then
      echo ""
      echo "Committed NAMESPACE / man drift from #[miniextendr] Rust source."
      echo "Run 'just rcmdinstall && just force-document' and commit the regenerated NAMESPACE + man/."
      exit 1
    fi

    echo "Wrappers sync check passed."

# Show diff between workspace and vendored crates
[script("bash")]
vendor-sync-diff:
    set -euo pipefail

    vendor_dir="rpkg/vendor"

    for crate in miniextendr-api miniextendr-macros miniextendr-lint miniextendr-engine; do
      # Accept both flat (vendor/<name>/) and versioned (vendor/<name>-<version>/) layouts
      crate_dir=""
      if [[ -d "$vendor_dir/$crate" ]]; then
        crate_dir="$vendor_dir/$crate"
      else
        versioned=$(find "$vendor_dir" -maxdepth 1 -type d -name "${crate}-[0-9]*" | head -1)
        if [[ -n "$versioned" ]]; then
          crate_dir="$versioned"
        fi
      fi

      if [[ -n "$crate_dir" ]]; then
        echo "=== $crate ==="
        diff -ruN "$crate/src" "$crate_dir/src" || true
        echo ""
      fi
    done

# Check that rpkg/src/rust/Cargo.lock is in tarball-shape.
#
# Tarball-shape (post-#408):
#   - miniextendr-{api,lint,macros} have source = "git+https://github.com/A2-ai/miniextendr#<sha>"
#     (not path+, not missing entirely).
#   - no [[patch.unused]] blocks (signals a wider [patch.crates-io] than
#     the manifest needs; produces spurious commit-time diff).
#
# checksum = "..." lines are ALLOWED (cargo-revendor writes valid
# .cargo-checksum.json files whose `package` field matches them).
[script("bash")]
lock-shape-check:
    set -euo pipefail
    lock=rpkg/src/rust/Cargo.lock
    if [ ! -f "$lock" ]; then
        echo "lock-shape-check: $lock not found — skipping"
        exit 0
    fi
    bad=0
    for crate in miniextendr-api miniextendr-lint miniextendr-macros; do
        if ! grep -A 3 "^name = \"$crate\"$" "$lock" \
                | grep -q '^source = "git+https://github\.com/A2-ai/miniextendr'; then
            echo "lock-shape-check: $lock — $crate missing canonical git+url source" >&2
            bad=1
        fi
    done
    if grep -qE '^\[\[patch\.unused\]\]' "$lock"; then
        echo "lock-shape-check: $lock has [[patch.unused]] blocks (narrow [patch.crates-io])" >&2
        bad=1
    fi
    if [ $bad -eq 1 ]; then
        echo "  Run: just cargo-lock-restore    # restore from HEAD" >&2
        echo "    or just vendor                # regenerate canonical shape" >&2
        exit 1
    fi
    echo "lock-shape-check: OK"

# ============================================================================
# mx CLI (miniextendr-cli)
# ============================================================================

# Build the miniextendr CLI binary (with dev commands)
cli-build *cargo_flags:
    cargo build -p miniextendr-cli --features dev {{cargo_flags}}

# Install the miniextendr CLI binary (with dev commands)
cli-install *cargo_flags:
    cargo install --path miniextendr-cli --features dev {{cargo_flags}}

# ============================================================================
# cargo-revendor (standalone workspace, not part of miniextendr workspace)
# ============================================================================

# Install cargo-revendor
revendor-install:
    cargo install --path cargo-revendor

# Build cargo-revendor (without installing)
revendor-build *cargo_flags:
    cargo build --manifest-path cargo-revendor/Cargo.toml {{cargo_flags}}

# Clean build cargo-revendor with timing analysis
revendor-timings:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Cleaning..."
    cd cargo-revendor && cargo clean 2>/dev/null || true
    echo "Building with --timings..."
    cd cargo-revendor && cargo build --timings 2>&1 | tail -3
    echo ""
    python3 -c "
    import re, json
    html = open('cargo-revendor/target/cargo-timings/cargo-timing.html').read()
    m = re.search(r'UNIT_DATA\s*=\s*(\[.+?\]);\s', html, re.DOTALL)
    if not m:
        print('Could not parse timing data')
        exit()
    data = json.loads(m.group(1))
    data.sort(key=lambda x: x.get('duration', 0), reverse=True)
    total = sum(d.get('duration', 0) for d in data)
    print(f'Compile units: {len(data)}')
    print(f'Total CPU time: {total:.1f}s')
    print()
    print(f'{\"TIME\":>6}  CRATE')
    print('-' * 50)
    for d in data[:15]:
        print(f'  {d.get(\"duration\", 0):5.1f}s  {d.get(\"name\", \"?\")} {d.get(\"version\", \"\")}')
    "

# Run cargo-revendor tests (offline only)
revendor-test:
    cargo test --manifest-path cargo-revendor/Cargo.toml

# Run cargo-revendor tests (including network tests)
revendor-test-all:
    cargo test --manifest-path cargo-revendor/Cargo.toml -- --include-ignored --test-threads=1

# ── Native R package bindgen corpus ─────────────────────────────────────────

# Run bindgen corpus test (C-only mode, ~69 packages)
bindgen-corpus:
    bash dev/run_bindgen_corpus.sh

# Run bindgen corpus test (C + C++ mode, ~308 packages)
bindgen-corpus-v3:
    bash dev/run_bindgen_corpus_v3.sh

# Add all CRAN corpus packages (with inst/include) to rv, excluding Rcpp/cpp11 ecosystem
bindgen-corpus-add-packages:
    #!/usr/bin/env bash
    set -euo pipefail
    PKGS=$(awk -F',' 'NR==1{for(i=1;i<=NF;i++){gsub(/"/,"",$i);if($i=="excluded_rcpp_ecosystem")col=i}} NR>1{gsub(/"/,"",$col);gsub(/"/,"",$1);if($col=="FALSE")print $1}' dev/cran_native_packages_full.csv | tr '\n' ' ')
    echo "Adding $(echo $PKGS | wc -w | tr -d ' ') packages to rv..."
    rv add --no-sync $PKGS
    echo "Run 'rv sync' to install them."

# Remove all CRAN corpus packages from rv (keeps core dev deps)
bindgen-corpus-remove-packages:
    #!/usr/bin/env bash
    set -euo pipefail
    PKGS=$(awk -F',' 'NR==1{for(i=1;i<=NF;i++){gsub(/"/,"",$i);if($i=="excluded_rcpp_ecosystem")col=i}} NR>1{gsub(/"/,"",$col);gsub(/"/,"",$1);if($col=="FALSE")print $1}' dev/cran_native_packages_full.csv | tr '\n' ' ')
    echo "Removing $(echo $PKGS | wc -w | tr -d ' ') corpus packages from rv..."
    rv remove --no-sync $PKGS
    echo "Run 'rv sync' to clean the library."

# ── Documentation site ──────────────────────────────────────────────────────

# Regenerate the committed, LLM-ready Rust API corpus from rustdoc JSON.
llm-docs:
    bash rust-llm-docs/generate-miniextendr-docs.sh

# Test the renderer, regenerate the corpus, and fail if committed output drifted.
[script("bash")]
llm-docs-check:
    set -euo pipefail
    python3 -m unittest discover -s rust-llm-docs -p 'test_*.py'
    just llm-docs
    git diff --exit-code -- rust-llm-docs/generated

# Regenerate site/content/manual/ from docs/.
# Run before site-build or site-serve when previewing doc changes locally.
# CI runs docs-to-site.sh automatically before each deploy.
site-docs:
    bash scripts/docs-to-site.sh

# Build Zola site (output in site/public/). Run `just site-docs` first if docs or plans changed.
site-build:
    cd site && zola build

# Local preview server (http://127.0.0.1:1111). Run `just site-docs` first if docs or plans changed.
site-serve:
    cd site && zola serve

# Regenerate the manual and verify Zola's link checker is clean. Gates CI
# (the Docs Link Check job) — run this after any docs/*.md heading or link
# edit. `site/config.toml` sets `internal_level = "error"` (a broken internal
# anchor fails the build) and `external_level = "warn"` (external URLs are
# warn-only — CI runners get bot-blocked by some hosts, so external liveness
# is a quarterly-audit concern, not enforced here).
site-check: site-docs
    cd site && zola check

# Bump version across all Cargo.toml and DESCRIPTION files.
bump-version version:
    bash scripts/bump-version.sh {{version}}

# ── Local webR/wasm dev container ────────────────────────────────────────────
#
# `Dockerfile.webr` inherits the webR base via ghcr.io/a2-ai/webr-mirror
# (digest-pinned, identical bytes to upstream — see .github/workflows/mirror-webr.yml)
# and layers `just`, `autoconf`, dev tools. See the Dockerfile header for
# what's already in the base. Use this when reproducing webR/wasm32 build
# issues locally.

docker_webr_image := "miniextendr-webr-dev:latest"

# Build the local webR dev image. Re-run when Dockerfile.webr changes.
docker-webr-build:
    docker build -f Dockerfile.webr -t {{docker_webr_image}} .

# Drop into an interactive shell in the webR dev container with this repo
# bind-mounted at /work. From inside, run `just configure / rcmdinstall /
# devtools-test` etc. as if on Linux. The container is amd64-only (webR
# upstream); on Apple Silicon Docker emulates via Rosetta — slower but works.
docker-webr-shell:
    docker run --rm -it \
        -v "{{justfile_directory()}}:/work" \
        -w /work \
        {{docker_webr_image}} bash

# Non-interactive: run an arbitrary command inside the container with the
# repo mounted. e.g. `just docker-webr-run "cargo check --target wasm32-unknown-emscripten -p miniextendr-api"`.
docker-webr-run *cmd:
    docker run --rm \
        -v "{{justfile_directory()}}:/work" \
        -w /work \
        {{docker_webr_image}} bash -c "{{cmd}}"

# Smoke-test the wasm32 build path inside the container. Checks
# `miniextendr-api` (no git deps, so works directly). For an rpkg-side
# wasm32 check, drop into a shell with `just docker-webr-shell` and run
# `bash ./rpkg/configure && cd rpkg/src/rust && cargo check --target
# wasm32-unknown-emscripten` — `configure` writes the right `[patch."git+url"]`
# overrides into `.cargo/config.toml` for the workspace siblings.
docker-webr-test: docker-webr-build
    just docker-webr-run "cargo check --target wasm32-unknown-emscripten -p miniextendr-api"

# Run the webR/wasm32 smoke test for rpkg.
# Builds rpkg as a wasm32 side-module inside the dev container and loads it
# in a webR Node session via the canonical runner
# tests/webr-node-smoke/smoke.mjs, plus an informational testthat pass
# (#1255; counts reported, test failures never gate — disable with
# SMOKE_TESTTHAT=0).
docker-webr-smoke *args: docker-webr-build
    bash tests/webr-smoke.sh {{args}}

# ── arm64-native webR dev container (#788) ───────────────────────────────────
#
# ⚠️ DRAFT — composed but NOT YET VALIDATED on arm64 hardware (no Docker /
# arm64 build in the dev sandbox). See Dockerfile.webr-arm64's header + the
# validation checklist in docs/WEBR.md.
#
# `Dockerfile.webr-arm64` builds natively on an arm64 host (Apple Silicon) from
# prebuilt parts — emscripten/emsdk:4.0.8-arm64 (matches the wasm R's emcc ABI)
# + native arm64 Rust/R, with the portable wasm sysroot COPY'd out of the amd64
# mirror. No qemu, no source emcc/flang/R→wasm build. Contrast docker-webr-*
# above, which runs the amd64 image under Rosetta.

docker_webr_arm64_image := "miniextendr-webr-dev-arm64:latest"

# Build the arm64-native webR dev image. Re-run when Dockerfile.webr-arm64
# changes. Must run on an arm64 host (the emsdk base + native toolchain are
# arm64); the donor stage is pinned `--platform=linux/amd64` for the FS copy.
docker-webr-arm64-build:
    docker build -f Dockerfile.webr-arm64 -t {{docker_webr_arm64_image}} .

# Interactive shell in the arm64 dev container, repo bind-mounted at /work.
docker-webr-arm64-shell: docker-webr-arm64-build
    docker run --rm -it \
        -v "{{justfile_directory()}}:/work" \
        -w /work \
        {{docker_webr_arm64_image}} bash

# Non-interactive: run an arbitrary command inside the arm64 container with the
# repo mounted.
docker-webr-arm64-run *cmd: docker-webr-arm64-build
    docker run --rm \
        -v "{{justfile_directory()}}:/work" \
        -w /work \
        {{docker_webr_arm64_image}} bash -c "{{cmd}}"

# arm64-native end-to-end smoke. Same three-phase flow as docker-webr-smoke but
# against the arm64 image (WEBR_ARM64=1 selects the native-R orchestration +
# arm64 image inside tests/webr-smoke.sh).
docker-webr-arm64-smoke *args: docker-webr-arm64-build
    WEBR_ARM64=1 bash tests/webr-smoke.sh {{args}}
