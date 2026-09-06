//! Test: `Missing<T>` cannot be a scalar match_arg/choices parameter. The choice
//! list lives in the R formal default, which `Missing<T>` forbids; `Option<T>`
//! is the optional form (#1473).

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_missing_scalar(#[miniextendr(choices("a", "b"))] x: Missing<String>) {}

fn main() {}
