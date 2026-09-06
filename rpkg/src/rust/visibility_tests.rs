//! Tests for R return value visibility (visible/invisible returns).

use miniextendr_api::miniextendr;

#[miniextendr]
/// @title Visibility and Interrupt Tests
/// @name rpkg_visibility_interrupts
/// @description Visibility and interrupt checks
/// @examples
/// invisibly_return_no_arrow()
/// force_invisible_i32()
/// with_interrupt_check(2L)
/// try(invisibly_option_return_none())
/// \dontrun{
/// unsafe_C_check_interupt_after()
/// }
pub fn invisibly_return_no_arrow() {}

/// Test that a function with explicit -> () return is invisible in R.
#[miniextendr]
#[allow(clippy::unused_unit)]
pub fn invisibly_return_arrow() -> () {}

/// Test that returning None from Option<()> produces an R error.
#[miniextendr]
pub fn invisibly_option_return_none() -> Option<()> {
    None // expectation: error!
}

/// Test that returning Some(()) from Option<()> is invisible in R.
#[miniextendr]
pub fn invisibly_option_return_some() -> Option<()> {
    Some(())
}

/// Test that returning Ok(()) from Result<(), ()> is invisible in R.
#[miniextendr]
#[allow(clippy::result_unit_err)]
pub fn invisibly_result_return_ok() -> Result<(), ()> {
    Ok(())
}

/// Test that Result Err(()) returns NULL in R, Ok returns doubled value.
/// @param x Integer input (negative triggers Err).
#[miniextendr]
#[allow(clippy::result_unit_err)]
pub fn result_null_on_err(x: i32) -> Result<i32, ()> {
    if x >= 0 {
        Ok(x * 2)
    } else {
        Err(()) // Should return NULL
    }
}

// Test explicit invisible attribute (force i32 return to be invisible)
/// Test that #[miniextendr(invisible)] forces an i32 return to be invisible in R.
#[miniextendr(invisible)]
pub fn force_invisible_i32() -> i32 {
    42
}

// Test explicit visible attribute (force () return to be visible)
/// Test that #[miniextendr(visible)] forces a unit return to be visible in R.
#[miniextendr(visible)]
pub fn force_visible_unit() {}

// Test check_interrupt attribute - checks for Ctrl+C before executing
/// Test that `#[miniextendr(check_interrupt)]` checks for user interrupts before execution.
/// @param x Integer scalar input.
#[miniextendr(check_interrupt)]
pub fn with_interrupt_check(x: i32) -> i32 {
    x * 2
}

/// Test `unwrap_in_r`: the `Result` is converted to `list(error = ...)` on the
/// R side instead of raising a condition.
/// @param x Integer input (negative triggers Err with message).
#[miniextendr(unwrap_in_r)]
pub fn result_unwrap_in_r(x: i32) -> Result<i32, String> {
    if x >= 0 {
        Ok(x * 2)
    } else {
        Err(format!("negative input: {}", x))
    }
}
