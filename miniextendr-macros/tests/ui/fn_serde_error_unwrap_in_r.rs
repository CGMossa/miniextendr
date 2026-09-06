//! Test: `#[miniextendr(serde_error(..))]` combined with `unwrap_in_r`.
//!
//! `serde_error(..)` classes the condition raised from the `Err` arm; `unwrap_in_r`
//! returns the whole `Result` to R as a value and never raises, so there is no
//! condition to class. The combination is rejected instead of silently ignored.

use miniextendr_macros::miniextendr;

#[miniextendr(serde_error(prefix = "p"), unwrap_in_r)]
fn bad_serde_error_unwrap_in_r(x: i32) -> Result<i32, String> {
    Ok(x)
}

fn main() {}
