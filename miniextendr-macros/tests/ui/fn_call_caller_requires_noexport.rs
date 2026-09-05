//! Test: `#[miniextendr(call = caller)]` on an exported function.
//!
//! Caller attribution exists for package-internal entry points wrapped by a
//! hand-written R function. On an exported function the "caller" is arbitrary
//! user code, so the option is rejected unless `noexport` or `internal` is set.

use miniextendr_macros::miniextendr;

#[miniextendr(call = caller)]
pub fn bad_call_caller_exported(x: i32) -> Result<i32, String> {
    Ok(x)
}

fn main() {}
