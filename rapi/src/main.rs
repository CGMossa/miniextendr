use std::{
  ffi::{CStr, CString},
  ptr::slice_from_raw_parts_mut,
};

use anyhow::{anyhow, Context, Result};
use rsys::*;

fn main() -> Result<()> {
  let argv = ["--Rasdoa"];
  let mut argv = argv.map(|x| CString::new(x).unwrap().into_raw());
  let argc = argv.len();

  // let init_status = unsafe { Rf_initEmbeddedR(argc as _, argv.as_mut_ptr())
  // };

  // assert!(init_status >= 0);
  // unsafe {
  //   println!("{}", atan2(50., 2.));
  //   Rprintf(CString::new("arg").unwrap().into_raw());
  // }
  // let d = unsafe { Rf_allocVector(SEXPTYPE::REALSXP as _, 23) };
  // unsafe { Rf_protect(d) };

  // unsafe {
  //   let dx = REAL(d);
  //   let dx_xlength = XLENGTH_EX(d);
  //   let dx = slice_from_raw_parts_mut(dx, dx_xlength as _);

  //   println!("dx: {:?}", dx.as_ref());
  // };

  // unsafe {
  //   setup_term_ui();
  //   readconsolecfg();
  //   // setup_Rmainloop();
  //   // ReadConsole;
  //   // getDLLVersion();
  //   // R_ReplDLLinit();
  //   // while (R_ReplDLLdo1() > 0) { /* add user actions here if desired */ }
  // }

  // unsafe { Rf_unprotect(1) };
  // unsafe {
  //   R_RunExitFinalizers();
  //   R_CleanTempDir();
  //   Rf_KillAllDevices();
  //   // CleanEd(); // doesn't work on host: MSVC
  // }
  // unsafe { Rf_endEmbeddedR(0) };

  Ok(())
}
