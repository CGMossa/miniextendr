use anyhow::{Context, Result};
use std::path::Path;

pub(crate) fn libgcc_mock() -> Result<()> {
  // TODO: Maybe circumvent the `LIBRARY_PATH` by just dumping it near
  // the exe and hope that it works with the crazy linker.
  //

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
