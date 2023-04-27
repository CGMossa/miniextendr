#[allow(unused_imports)]
use anyhow::{anyhow, Context, Result};
use rapi_macros::*;
use rsys::*;

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
