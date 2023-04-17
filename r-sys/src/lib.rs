#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

use std::{
    ffi::c_int,
    os::raw::{c_char, c_void},
};

include!("../r-bindings.rs");

pub const M_E: f64 = std::f64::consts::E;
pub const M_LOG2E: f64 = std::f64::consts::LOG2_E;
pub const M_LOG10E: f64 = std::f64::consts::LOG10_E;
pub const M_LN2: f64 = std::f64::consts::LN_2;
pub const M_LN10: f64 = std::f64::consts::LN_10;
pub const M_PI: f64 = std::f64::consts::PI;
pub const M_PI_2: f64 = std::f64::consts::FRAC_PI_2;
pub const M_PI_4: f64 = std::f64::consts::FRAC_PI_4;
pub const M_1_PI: f64 = std::f64::consts::FRAC_1_PI;
pub const M_2_PI: f64 = std::f64::consts::FRAC_2_PI;
pub const M_2_SQRTPI: f64 = std::f64::consts::FRAC_2_SQRT_PI;
pub const M_SQRT2: f64 = std::f64::consts::SQRT_2;
pub const M_SQRT1_2: f64 = std::f64::consts::FRAC_1_SQRT_2;

#[allow(dead_code)]
#[inline]
unsafe fn ISNA(x: f64) -> i32 {
    R_IsNA(x)
}

trait RExt<Arg = Self> {
    unsafe fn is_nan(x: Arg) -> bool;
}
impl RExt<Self> for f64 {
    #[inline]
    unsafe fn is_nan(x: Self) -> bool {
        // _isnan(x) != 0
        __isnan(x) != 0
    }
}
impl RExt<Self> for f32 {
    #[inline]
    unsafe fn is_nan(x: Self) -> bool {
        // _isnanf(x) != 0
        __isnanf(x) != 0
    }
}
impl RExt<Self> for u128 {
    #[inline]
    unsafe fn is_nan(x: Self) -> bool {
        __isnanl(x) != 0
    }
}

#[allow(dead_code)]
#[inline]
pub unsafe fn R_FINITE(x: f64) -> i32 {
    R_finite(x)
}

#[inline]
pub unsafe fn R_Calloc<T>(n: usize) -> *mut T {
    let size = std::mem::size_of::<T>();
    let ptr = unsafe { R_chk_calloc(n, size) } as *mut T;
    #[allow(clippy::let_and_return)]
    ptr
}

#[inline]
pub unsafe fn R_Realloc<T>(ptr: *mut T, n: usize) -> *mut T {
    let size = std::mem::size_of::<T>() * n;
    let new_ptr = unsafe { R_chk_realloc(ptr as *mut c_void, size) } as *mut T;
    #[allow(clippy::let_and_return)]
    new_ptr
}

#[inline]
pub unsafe fn R_Free<T>(ptr: *mut T) {
    R_chk_free(ptr as *mut c_void);
}

#[inline]
pub unsafe fn Memcpy<T>(dst: *mut T, src: *const T, n: usize) {
    std::ptr::copy_nonoverlapping(src, dst, n);
}

#[inline]
pub unsafe fn Memzero<T>(dst: *mut T, n: usize) {
    std::ptr::write_bytes(dst, 0, n);
}

#[inline]
pub unsafe fn CallocCharBuf(n: usize) -> *mut char {
    R_Calloc::<char>(n + 1)
}

// omitting the fortran macros

#[inline]
pub unsafe fn CHAR(x: SEXP) -> *const c_char {
    unsafe { R_CHAR(x) }
}

#[inline]
pub unsafe fn IS_SIMPLE_SCALAR(x: SEXP, type_: c_int) -> bool {
    (IS_SCALAR(x, type_) != 0) && (ATTRIB(x) == R_NilValue)
}

// region: SWITCH_TO_REFCNT

// #[cfg(feature = "switch_to_refcnt")]
// pub mod switch_to_refcnt {
// use super::*;
// use super::NAMEDMAX as NAMEDMAX_;
// pub const NAMEDMAX: c_int = NAMEDMAX_ as _;

//TODO: ensure that INCREMENT_NAMED and DECREMENT_NAMED doesn't work

#[inline]
pub unsafe fn INCREMENT_NAMED(x: SEXP) {
    if NAMED(x) != NAMEDMAX as c_int {
        SET_NAMED(x, NAMED(x) + 1);
    }
}

#[inline]
pub unsafe fn DECREMENT_NAMED(x: SEXP) {
    let n = NAMED(x);
    if n > 0 && n <= NAMEDMAX as c_int {
        SET_NAMED(x, n - 1);
    }
}

// /* Macros for some common idioms. */
#[inline]
pub unsafe fn MAYBE_SHARED(x: SEXP) -> bool {
    // # define MAYBE_SHARED(x) (NAMED(x) > 1)
    REFCNT(x) > 1
}

#[inline]
pub unsafe fn NO_REFERENCES(x: SEXP) -> bool {
    // # define NO_REFERENCES(x) (NAMED((x) ==) 0)
    REFCNT(x) == 0
}

#[inline]
pub unsafe fn MAYBE_REFERENCED(x: SEXP) -> bool {
    !NO_REFERENCES(x)
}

#[inline]
pub unsafe fn NOT_SHARED(x: SEXP) -> bool {
    !MAYBE_SHARED(x)
}

// endregion

#[inline]
pub unsafe fn cons(a: SEXP, b: SEXP) -> SEXP {
    Rf_cons(a, b)
}

#[inline]
pub unsafe fn lcons(a: SEXP, b: SEXP) -> SEXP {
    Rf_lcons(a, b)
}

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

#[inline]
pub unsafe fn KNOWN_SORTED(sorted: c_int) -> bool {
    use _bindgen_ty_1::*;
    (sorted == SORTED_DECR)
        || (sorted == SORTED_INCR)
        || (sorted == SORTED_DECR_NA_1ST)
        || (sorted == SORTED_INCR_NA_1ST)
}

#[inline]
pub unsafe fn KNOWN_NA_1ST(sorted: c_int) -> bool {
    use _bindgen_ty_1::*;
    (sorted == SORTED_INCR_NA_1ST) || (sorted == SORTED_DECR_NA_1ST)
}
#[inline]
pub unsafe fn KNOWN_INCR(sorted: c_int) -> bool {
    use _bindgen_ty_1::*;
    (sorted == SORTED_INCR) || (sorted == SORTED_INCR_NA_1ST)
}
#[inline]
pub unsafe fn KNOWN_DECR(sorted: c_int) -> bool {
    use _bindgen_ty_1::*;

    (sorted == SORTED_DECR) || (sorted == SORTED_DECR_NA_1ST)
}

// include\Rinternals.h
#[inline]
pub unsafe fn error_return(msg: *const c_char) -> SEXP {
    Rf_error(msg);
    return R_NilValue;
}

// include\Rinternals.h
#[inline]
pub unsafe fn errorcall_return(cl: SEXP, msg: *const c_char) -> SEXP {
    Rf_errorcall(cl, msg);
    return R_NilValue;
}

// include\Rinternals.h
#[inline]
pub unsafe fn BCODE_CONSTS(x: SEXP) -> SEXP {
    // re-enable in Defn.h after removing here
    CDR(x)
}
// include\Rinternals.h
#[inline]
pub unsafe fn PREXPR(e: SEXP) -> SEXP {
    R_PromiseExpr(e)
}

// include\Rinternals.h
#[inline]
pub unsafe fn BODY_EXPR(e: SEXP) -> SEXP {
    R_ClosureExpr(e)
}
