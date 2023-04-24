use std::ffi::{CStr, CString};

use anyhow::{anyhow, Context, Result};
use rsys::*;

fn main() -> Result<()> {
  // Rf_initEmbeddedR
  // Rf_endEmbeddedR
  let mut argv = [b"R"];
  let mut argv = argv.map(|x| CStr::from_bytes_with_nul(x)?);
  let argc = argv.len();
  unsafe {
    Rf_initEmbeddedR(argc as _, argv.as_mut_ptr());
  }
  unsafe {
    println!("{}", atan2(50., 2.));
  }
  unsafe { Rf_allocVector(SEXPTYPE::INTSXP as _, 23) };

  unsafe {
    Rf_endEmbeddedR(0);
  }
  Ok(())
}
