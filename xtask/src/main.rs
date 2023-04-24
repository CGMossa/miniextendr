use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

/// Tasks to aid in the development of R FFI wrappers.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[command(subcommand)]
  command: Command,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
  /// Create a fake library to satisfy the error `-lgcc_eh`
  /// that occurs when using the Rtools linker.
  /// This needs to be done system-wide, thus it needs an
  /// environment variable to point to where the fake library
  /// should reside.
  LibraryGccMock {},
  /// Copy R headers and binaries to the workspace directory.
  CopyRHeaders {
    //TODO: optional R_HOME
    //TODO: optional target-directory ~> default to r-sys-root
  },
}

fn main() -> Result<()> {
  let xtask_directory: PathBuf = env!("CARGO_MANIFEST_DIR").into();
  let workspace_root: &Path = xtask_directory.parent().unwrap();

  let args = Args::parse();
  match args.command {
    Command::LibraryGccMock {} => libgcc_mock()?,
    Command::CopyRHeaders {} => {
      let r_sys_root = workspace_root.join("rsys");
      copy_r_headers(r_sys_root.as_path())?
    }
    #[allow(unreachable_patterns)]
    _ => {
      todo!()
    }
  };
  Ok(())
}

fn libgcc_mock() -> Result<()> {
  // let out_dir = env::var("OUT_DIR")?;
  let libgcc_var = std::env::var("LIBRARY_PATH")
    .context("Set `LIBRARY_PATH` on your System / User")?;
  if libgcc_var.is_empty() {
    anyhow::bail!("Environment variable `LIBRARY_PATH` cannot be empty.")
  }
  // create a directory in an arbitrary location (e.g. libgcc_mock)
  let libgcc_mock_path = Path::new(&libgcc_var);
  if !libgcc_mock_path.exists() {
    std::fs::create_dir(libgcc_mock_path)?
  }
  if !libgcc_mock_path.join("libgcc_eh.a").exists() {
    std::fs::File::create(libgcc_mock_path.join("libgcc_eh.a"))?;
    std::fs::File::create(libgcc_mock_path.join("libgcc_s.a"))?;
  }
  Ok(())
}

/// Copy R headers and binaries over to the repository
fn copy_r_headers(r_sys_root: &Path) -> Result<()> {
  let r_copied_headers_path = r_sys_root.join("r");
  let r_home: PathBuf = env!("R_HOME").into();

  fs_extra::dir::remove(&r_copied_headers_path)?;
  std::fs::create_dir_all(&r_copied_headers_path)?;
  fs_extra::dir::copy(
    r_home.join("include"),
    &r_copied_headers_path,
    &fs_extra::dir::CopyOptions::new(),
  )?;

  // //TODO: Only copy over the used DLLs.
  let r_copied_binaries = r_copied_headers_path.join("bin");
  std::fs::create_dir_all(&r_copied_binaries)?;
  fs_extra::dir::copy(
    &r_home.join("bin").join("x64"),
    &r_copied_binaries,
    &fs_extra::dir::CopyOptions::new(),
  )?;
  Ok(())
}
