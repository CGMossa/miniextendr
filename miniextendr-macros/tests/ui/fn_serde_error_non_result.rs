//! Test: `#[miniextendr(serde_error(..))]` on a function that does not return `Result`.
//!
//! The attribute only affects the generated `Err` arm, so a non-`Result` return
//! type has nothing for it to act on. Rejected at compile time.

use miniextendr_macros::miniextendr;

#[miniextendr(serde_error(prefix = "p"))]
fn bad_serde_error_non_result(x: i32) -> i32 {
    x
}

fn main() {}
