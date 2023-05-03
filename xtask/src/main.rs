use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

mod libgcc_mock;
mod copy_r_headers;
mod allowlist;

//TODO: Propose a `clean` task that would also clean the
// embedded extendrtest package as well as the extendr crate `clean`.

//TODO: generally re-implement the Makefile in Rust and export it?

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
  /// Produce a list of the symbols to be ported through bindgen.
  Allowlist {},
}

fn main() -> Result<()> {
  let xtask_directory: PathBuf = env!("CARGO_MANIFEST_DIR").into();
  let workspace_root: &Path = xtask_directory.parent().unwrap();

  let args = Args::parse();
  match args.command {
    Command::LibraryGccMock {} => libgcc_mock::libgcc_mock()?,
    Command::CopyRHeaders {} => {
      let r_sys_root = workspace_root.join("rsys");
      copy_r_headers::copy_r_headers(r_sys_root.as_path())?
    }
    Command::Allowlist {} => allowlist::allowlist(
      &workspace_root.join("rsys").join("wrapper.h"),
      &workspace_root.join("rsys").join("r").join("include"),
      &workspace_root.join("rsys"),
    )?,
  };
  Ok(())
}
