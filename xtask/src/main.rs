use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

mod allowlist;
mod copy_r_headers;
mod find_macros;
mod libgcc_mock;

//TODO: Propose a `clean` task that would also clean the
// embedded extendrtest package as well as the extendr crate `clean`.

//TODO: generally re-implement the Makefile in Rust and export it?

/// Tasks to aid in the development of R FFI wrappers.
#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Args {
  #[command(subcommand)]
  command: Option<Command>,
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
  FindMacros {},
}

fn main() -> Result<()> {
  let xtask_directory: PathBuf = env!("CARGO_MANIFEST_DIR").into();
  let workspace_root: &Path = xtask_directory.parent().unwrap();
  let rsys_root = workspace_root.join("rsys");

  let args = Args::parse();
  if let Some(command) = args.command {
    match command {
      Command::LibraryGccMock {} => libgcc_mock::libgcc_mock()?,
      Command::CopyRHeaders {} => {
        copy_r_headers::copy_r_headers(rsys_root.as_path())?
      }
      Command::Allowlist {} => allowlist::allowlist(
        &workspace_root.join("rsys").join("wrapper.h"),
        &workspace_root.join("rsys").join("r").join("include"),
        &workspace_root.join("rsys"),
      )?,
      Command::FindMacros {} => find_macros::find_macros(
        rsys_root.as_path(),
        &workspace_root.join("rsys").join("wrapper.h"),
        &workspace_root.join("rsys").join("r").join("include"),
        &workspace_root.join("rsys").join("r"),
      )?,
      // _ => todo!("DEBUGGING"),
    };
  }
  Ok(())
}
