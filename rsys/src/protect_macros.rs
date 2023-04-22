use super::*;
use std::ffi::c_int;

#[inline]
pub unsafe fn PROTECT(s: SEXP) -> SEXP {
  Rf_protect(s)
}

#[inline]
pub unsafe fn UNPROTECT(n: c_int) {
  Rf_unprotect(n)
}

#[inline]
pub unsafe fn UNPROTECT_PTR(s: SEXP) {
  Rf_unprotect_ptr(s)
}

#[inline]
pub unsafe fn REPROTECT(x: SEXP, i: PROTECT_INDEX) {
  R_Reprotect(x, i)
}
