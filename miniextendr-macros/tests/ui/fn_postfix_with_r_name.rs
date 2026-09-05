//! Test: `#[miniextendr(postfix = "...")]` combined with `r_name = "..."`.
//!
//! Both options set the R wrapper's name (`postfix` derives it from the Rust
//! identifier, `r_name` replaces it), so giving both is contradictory and is
//! rejected rather than letting one silently win.

use miniextendr_macros::miniextendr;

#[miniextendr(noexport, postfix = "_impl", r_name = "widget_internal")]
fn bad_postfix_with_r_name(x: i32) -> i32 {
    x
}

fn main() {}
