//! Miscellaneous test functions.

use miniextendr_api::miniextendr;
use miniextendr_api::prelude::SEXP;

// Test that wildcard `_` parameters work (transformed to synthetic names internally)
#[miniextendr]
/// @title Miscellaneous Tests
/// @name rpkg_misc
/// @description Miscellaneous test helpers
/// @examples
/// underscore_it_all(1L, 2)
/// do_nothing()
pub fn underscore_it_all(_: i32, _: f64) {}

// Simple SEXP return
/// Test returning a scalar integer SEXP directly.
#[miniextendr]
pub fn do_nothing() -> SEXP {
    miniextendr_api::SEXP::scalar_integer(42)
}
