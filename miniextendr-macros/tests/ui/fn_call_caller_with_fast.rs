//! Test: `#[miniextendr(noexport, call = caller, fast)]`.
//!
//! `fast` (and `no_call_attribution`) emit `.call = NULL`, so there is no call
//! slot for `call = caller` to redirect; the combination is rejected.

use miniextendr_macros::miniextendr;

#[miniextendr(noexport, call = caller, fast)]
pub fn bad_call_caller_fast(x: i32) -> Result<i32, String> {
    Ok(x)
}

fn main() {}
