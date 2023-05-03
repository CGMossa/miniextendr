use anyhow::Result;
use std::path::{Path, PathBuf};

/// Copy R headers and binaries over to the repository
pub(crate) fn copy_r_headers(r_sys_root: &Path) -> Result<()> {
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
    r_home.join("bin").join("x64"),
    &r_copied_binaries,
    &fs_extra::dir::CopyOptions::new(),
  )?;
  Ok(())
}
