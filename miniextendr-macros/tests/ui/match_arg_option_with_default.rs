//! Test: an `Option<T>` match_arg parameter is the optional form (its R formal
//! defaults to NULL, meaning no choice), so a `default` contradicts it and must
//! error at compile time (#1473).

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_optional_default(#[miniextendr(match_arg, default = "\"a\"")] x: Option<String>) {}

fn main() {}
