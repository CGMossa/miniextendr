use std::{
  ffi::{c_int, CStr, CString},
  ptr::slice_from_raw_parts_mut,
};

use anyhow::{anyhow, Context, Result};
use rsys::*;
use rapi_macros::*;

#[embed_r]
fn main() -> Result<()> {
  // unsafe {
  //   Rf_initEmbeddedR(0, [0_i8 as _; 0].as_mut_ptr());
  // }

  unsafe {
    // LGLSXP, INTSXP, REALSXP, CPLXSP, STRSXP and RAWSXP
    let n = 1;
    Rf_allocVector(LGLSXP as _, n);
    Rf_allocVector(INTSXP as _, n);
    Rf_allocVector(REALSXP as _, n);
    Rf_allocVector(CPLXSXP as _, n);
    Rf_allocVector(STRSXP as _, n);
    Rf_allocVector(RAWSXP as _, n);
  }

  // unsafe {
  //   Rf_endEmbeddedR(0);
  // }
  Ok(())
}

#[allow(dead_code, non_snake_case)]
fn Rf_initEmbeddedR_impl<const N: usize>(argc: c_int, argv: [&str; N]) -> i32 {
  /*
  Rf_initialize_R(argc, argv);
  setup_Rmainloop();
  return(1);
  */
  unsafe {
    Rf_initEmbeddedR(
      argc,
      argv.map(|x| CString::new(x).unwrap().into_raw()).as_mut_ptr(),
    );
    setup_Rmainloop();
    return 1;
  }
}

#[allow(dead_code, non_snake_case)]
fn Rf_endEmbeddedR_impl(fatal: c_int) {
  /*
    R_RunExitFinalizers();
    CleanEd();
    R_CleanTempDir();
    if (!fatal)
    {
        Rf_KillAllDevices();
        AllDevicesKilled = TRUE;
    }
    if (!fatal && R_CollectWarnings)
        PrintWarnings(); /* from device close and .Last */
    app_cleanup();
  */
  unsafe {
    R_RunExitFinalizers();
    CleanEd();
    R_CleanTempDir();
    if fatal == 0 {
      Rf_KillAllDevices();
      AllDevicesKilled = Rboolean::TRUE;
    }
    //FIXME: use Rf_GetOption to gather `R_CollectWarnings` which is 00
    GA_appcleanup();
  }
}
