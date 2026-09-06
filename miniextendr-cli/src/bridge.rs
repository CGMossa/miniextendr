use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Context, Result, bail};

/// Locate the `Rscript` binary.
///
/// Search order:
/// 1. `$R_HOME/bin/Rscript`
/// 2. `Rscript` on `$PATH`
pub fn find_rscript() -> Result<PathBuf> {
    if let Ok(r_home) = std::env::var("R_HOME") {
        let candidate = PathBuf::from(&r_home).join("bin").join("Rscript");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    // Fall back to PATH lookup
    which("Rscript").context(
        "Rscript not found. Install R or set R_HOME.\n\
         This command requires R — native commands (cargo, lint, config) do not.",
    )
}

/// Simple `which` implementation.
fn which(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(name);
            if candidate.is_file() {
                Some(candidate)
            } else {
                None
            }
        })
    })
}

/// Run `Rscript -e '<expr>'` in the given directory.
///
/// Forwards stdout/stderr directly for interactive feel.
/// Returns an error if the process exits non-zero.
pub fn rscript_eval(expr: &str, cwd: &std::path::Path, quiet: bool) -> Result<ExitStatus> {
    rscript_eval_args(expr, &[], cwd, quiet)
}

/// Evaluate fixed R code with data passed through commandArgs(trailingOnly=TRUE).
/// Rscript's -e handling unescapes backslashes, so user strings belong in argv.
pub fn rscript_eval_args(
    expr: &str,
    args: &[&str],
    cwd: &std::path::Path,
    quiet: bool,
) -> Result<ExitStatus> {
    let rscript = find_rscript()?;
    let mut cmd = Command::new(&rscript);
    cmd.arg("-e").arg(expr).args(args).current_dir(cwd);

    if quiet {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    } else {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    }

    let status = cmd
        .status()
        .with_context(|| format!("failed to run {}", rscript.display()))?;

    if !status.success() {
        bail!("Rscript exited with status {}", status.code().unwrap_or(-1));
    }
    Ok(status)
}

/// Run an arbitrary command, forwarding stdio.
pub fn run_command(
    program: &str,
    args: &[impl AsRef<OsStr>],
    cwd: &std::path::Path,
    quiet: bool,
) -> Result<ExitStatus> {
    let mut cmd = Command::new(program);
    cmd.args(args).current_dir(cwd);

    if quiet {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    } else {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    }

    let status = cmd
        .status()
        .with_context(|| format!("failed to run {program}"))?;

    if !status.success() {
        bail!(
            "{program} exited with status {}",
            status.code().unwrap_or(-1)
        );
    }
    Ok(status)
}

/// Run an arbitrary command and capture stdout.
pub fn run_command_capture(
    program: &str,
    args: &[impl AsRef<OsStr>],
    cwd: &std::path::Path,
) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| format!("failed to run {program}"))?;

    if !output.status.success() {
        bail!(
            "{program} exited with status {}",
            output.status.code().unwrap_or(-1)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Run a shell command via `bash -c`.
pub fn bash(script: &str, cwd: &std::path::Path, quiet: bool) -> Result<ExitStatus> {
    run_command("bash", &["-c", script], cwd, quiet)
}

/// Check if a program is available on PATH.
pub fn has_program(name: &str) -> bool {
    which(name).is_some()
}

/// Get version output from a program.
pub fn program_version(name: &str) -> Option<String> {
    Command::new(name).arg("--version").output().ok().map(|o| {
        let s = String::from_utf8_lossy(&o.stdout).to_string();
        let line = s.lines().next().unwrap_or("").trim().to_string();
        if line.is_empty() {
            // Some tools print version to stderr
            let s = String::from_utf8_lossy(&o.stderr).to_string();
            s.lines().next().unwrap_or("").trim().to_string()
        } else {
            line
        }
    })
}
