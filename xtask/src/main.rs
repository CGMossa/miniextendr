use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    LibraryGccMock {},
    CopyRHeaders {
        //TODO: optional R_HOME
        //TODO: optional target-directory ~> default to r-sys-root
    },
}

fn main() -> Result<()> {
    // println!("Hello, world!");
    // println!("CARGO_MANIFEST_DIR: {}", std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let xtask_directory = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let xtask_directory: PathBuf = xtask_directory.into();
    let workspace_root: &Path = xtask_directory.parent().unwrap();

    let args = Args::parse();
    match args.command {
        Command::LibraryGccMock {} => libgcc_mock()?,
        Command::CopyRHeaders {} => {
            let r_sys_root = workspace_root.join("r-sys");
            copy_r_headers(r_sys_root.as_path())
        }
        _ => {
            todo!()
        }
    };
    Ok(())
}

fn libgcc_mock() -> Result<()> {
    use std::env;
    use std::fs;

    // let out_dir = env::var("OUT_DIR")?;
    let libgcc_var =
        env::var("LIBRARY_PATH").context("Set `LIBRARY_PATH` on your System / User")?;
    if libgcc_var.is_empty() {
        anyhow::bail!("Environment variable `LIBRARY_PATH` cannot be empty.")
    }
    // create a directory in an arbitrary location (e.g. libgcc_mock)
    let libgcc_mock_path = Path::new(&libgcc_var);
    if !libgcc_mock_path.exists() {
        fs::create_dir(libgcc_mock_path)?
    }
    if !libgcc_mock_path.join("libgcc_eh.a").exists() {
        fs::File::create(libgcc_mock_path.join("libgcc_eh.a"))?;
        fs::File::create(libgcc_mock_path.join("libgcc_s.a"))?;
    }
    Ok(())
}

fn copy_r_headers(r_sys_root: &Path) {
    /*

    // copy R headers and binaries over to the repository
    fs_extra::dir::remove("r")?;
    std::fs::create_dir_all("r")?;
    fs_extra::dir::copy(&r_paths.include, "r", &fs_extra::dir::CopyOptions::new())?;

    //TODO: Only copy over the used DLLs.
    fs_extra::dir::copy(
        &r_paths.library.parent().unwrap(),
        "r",
        &fs_extra::dir::CopyOptions::new(),
    )?;
    */
}
