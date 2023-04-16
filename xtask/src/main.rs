use std::path::PathBuf;

fn main() {
    // println!("Hello, world!");
    // println!("CARGO_MANIFEST_DIR: {}", std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let xtask_directory = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let xtask_directory: PathBuf = xtask_directory.into();
    let workspace_root = xtask_directory.parent().unwrap();
    todo!();
    //TODO:
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
