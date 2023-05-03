#[path = "src/libgcc_mock.rs"]
mod libgcc_mock;

use anyhow::Result;

fn main() -> Result<()> {
  libgcc_mock::libgcc_mock()?;
  Ok(())
}
