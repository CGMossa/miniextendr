//! Smoke floor for the `miniextendr` binary: every subcommand's `--help`
//! exits 0, error paths produce a diagnostic (not a panic), and `init
//! package` scaffolds the canonical build system end to end.
//!
//! Runs the default-features binary via `CARGO_BIN_EXE_miniextendr`, so the
//! assertions here match what `cargo test --workspace --locked` builds in CI
//! (the `dev` feature and its subcommands are intentionally not exercised).

use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_miniextendr"))
}

fn run(args: &[&str]) -> Output {
    bin().args(args).output().expect("binary runs")
}

/// Default-features subcommands, mirroring `Command` in `src/cli.rs`.
const SUBCOMMANDS: &[&str] = &[
    "init",
    "workflow",
    "status",
    "cargo",
    "vendor",
    "feature",
    "render",
    "rust",
    "config",
    "lint",
    "clean",
    "completions",
];

#[test]
fn help_exits_zero_and_lists_subcommands() {
    let out = run(&["--help"]);
    assert!(out.status.success(), "--help failed: {out:?}");
    let text = String::from_utf8_lossy(&out.stdout);
    for sub in SUBCOMMANDS {
        assert!(text.contains(sub), "--help does not list `{sub}`");
    }
}

#[test]
fn every_subcommand_help_exits_zero() {
    for sub in SUBCOMMANDS {
        let out = run(&[sub, "--help"]);
        assert!(out.status.success(), "`{sub} --help` failed: {out:?}");
    }
}

#[test]
fn missing_path_is_a_clean_diagnostic() {
    let out = run(&["--path", "/nonexistent-miniextendr-test", "status", "has"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Error") && !stderr.contains("panicked"),
        "expected clean diagnostic, got: {stderr}"
    );
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let dir =
            std::env::temp_dir().join(format!("miniextendr-cli-smoke-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        Scratch(dir)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// End-to-end (minus configure/compile): `init package` through the real
/// binary produces the canonical two-mode build system.
#[test]
fn init_package_scaffolds_canonical_build_system() {
    let scratch = Scratch::new();
    let pkg_dir = scratch.0.join("smoke.pkg");
    let out = run(&["init", "package", &pkg_dir.to_string_lossy()]);
    assert!(out.status.success(), "init package failed: {out:?}");

    let read = |rel: &str| -> String {
        std::fs::read_to_string(pkg_dir.join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
    };

    // The canonical install-mode latch, not the retired four-mode model.
    let configure_ac = read("configure.ac");
    assert!(configure_ac.contains("inst/vendor.tar.xz"));
    for retired in ["PREPARE_CRAN", "NOT_CRAN", "BUILD_CONTEXT"] {
        assert!(
            !configure_ac.contains(retired),
            "configure.ac contains {retired}"
        );
    }
    assert!(
        !pkg_dir.join("src/rust/cargo-config.toml.in").exists(),
        "retired cargo-config.toml.in scaffolded"
    );

    // The generated DESCRIPTION declares the miniextendr R version floor
    // (#1366). Literal on purpose — this is a binary-only crate, so the
    // integration test cannot import scaffold::R_VERSION_FLOOR; the
    // r_floor_matches_minirextendr_and_rpkg unit test pins the value.
    let desc = read("DESCRIPTION");
    assert!(
        desc.contains("Depends: R (>= 4.5)\n"),
        "scaffolded DESCRIPTION missing the R version floor: {desc}"
    );

    // Build-system surface present.
    for rel in [
        "DESCRIPTION",
        "NAMESPACE",
        "LICENSE",
        "bootstrap.R",
        "cleanup",
        "configure.win",
        "src/Makevars.in",
        "src/Makevars.win",
        "src/win.def.in",
        "src/stub.c",
        "src/r_shim.h",
        "src/rust/Cargo.toml",
        "src/rust/lib.rs",
        "src/rust/build.rs",
        "inst/include/mx_abi.h",
        "tools/config.guess",
        "tools/lock-shape-check.R",
        ".Rbuildignore",
        ".gitignore",
        "miniextendr.yml",
        "R/smoke.pkg-package.R",
    ] {
        assert!(pkg_dir.join(rel).is_file(), "missing {rel}");
    }

    // Re-running against the same directory refuses politely.
    let again = run(&["init", "package", &pkg_dir.to_string_lossy()]);
    assert!(!again.status.success());
}
