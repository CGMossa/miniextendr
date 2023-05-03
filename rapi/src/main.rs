#[allow(unused_imports)]
use anyhow::{anyhow, Context, Result};
use rapi_macros::*;
use rsys::*;

/*
#[derive(Debug)]
struct PlzBreak(i32);

let n = n as usize;

(0..n)
    .into_iter()
    .map(|xi| ExternalPtr::new(PlzBreak(xi as i32)))
    .collect::<List>()
*/

#[embed_r]
fn main() -> Result<()> {
  // let total_protected = 10_000;
  // let total_protected = 10_000 + 5;
  let total_protected = 10_000 * 6;
  let dummy_string = "Jamie Foxx";
  for _p in 0..total_protected {
    unsafe {
      Rf_protect(Rf_mkCharLenCE(
        dummy_string.as_ptr() as _,
        dummy_string.len() as _,
        cetype_t::CE_UTF8,
      ));

      // LGLSXP, INTSXP, REALSXP, CPLXSP, STRSXP and RAWSXP
      // let n = 10;
      // Rf_protect(Rf_allocVector(STRSXP as _, n));
      // Rf_protect(Rf_allocVector(STRSXP as _, n));
      // Rf_allocVector(INTSXP as _, n);
      // Rf_allocVector(REALSXP as _, n);
      // Rf_allocVector(CPLXSXP as _, n);
      // Rf_allocVector(STRSXP as _, n);
      // Rf_allocVector(RAWSXP as _, n);
    }
  }

  Ok(())
}
