//! Side-by-side fixture for `docs/CALL_ATTRIBUTION.md`.
//!
//! Two functions raise the same error message. One goes through the standard
//! `#[miniextendr]` wrapper (which emits `.call = match.call()`); the other is
//! `extern "C-unwind"`, which has no generated R wrapper and so no call slot.
//! The R-side error rendering is dramatically different.

use miniextendr_api::miniextendr;
use miniextendr_api::prelude::SEXP;

/// Wrapped path. The generated R wrapper passes `.call = match.call()` into the
/// C entry; on panic, `Rf_errorcall(call, msg)` shows the user's call frame.
///
/// @param left Ignored.
/// @param right Ignored.
/// @noRd
#[miniextendr(noexport)]
pub fn call_attr_with(_left: i32, _right: i32) -> i32 {
    panic!("left + right is too risky")
}

/// Unwrapped path. `extern "C-unwind"` bypasses the wrapper entirely — there is
/// no call slot and no `with_r_unwind_protect`. We raise an R error directly
/// with `Rf_error`, which carries no call attribution.
///
/// @param left Ignored.
/// @param right Ignored.
/// @noRd
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_call_attr_without(_left: SEXP, _right: SEXP) -> SEXP {
    unsafe {
        ::miniextendr_api::sys::Rf_error(c"%s".as_ptr(), c"left + right is too risky".as_ptr()) // mxl::allow(MXL300)
    }
}

// region: call = caller — internal entry points behind a hand-written R function (#1450)

/// Internal entry point that attributes conditions to its caller. The
/// hand-written `call_attr_caller()` in `R/call_attribution.R` delegates here,
/// so `conditionCall(e)` names that public function with its formals matched.
///
/// @param x Must be positive.
/// @noRd
#[miniextendr(noexport, call = caller)]
pub fn call_attr_caller_impl(x: i32) -> Result<i32, String> {
    if x <= 0 {
        return Err(format!("x must be positive, got {x}"));
    }
    Ok(x)
}

/// Default attribution for comparison: the same shape without `call = caller`
/// reports its own wrapper call (`call_attr_self_impl(x = value)`).
///
/// @param x Must be positive.
/// @noRd
#[miniextendr(noexport)]
pub fn call_attr_self_impl(x: i32) -> Result<i32, String> {
    if x <= 0 {
        return Err(format!("x must be positive, got {x}"));
    }
    Ok(x)
}

// endregion
