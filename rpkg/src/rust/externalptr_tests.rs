//! Tests for ExternalPtr functionality.

use miniextendr_api::externalptr::ErasedExternalPtr;
use miniextendr_api::externalptr::ExternalPtr;
use miniextendr_api::miniextendr;
use miniextendr_api::prelude::SEXP;

/// A simple test struct for ExternalPtr
#[derive(miniextendr_api::ExternalPtr, Debug)]
pub struct Counter {
    pub value: i32,
}

/// Another test struct to verify type safety
#[derive(miniextendr_api::ExternalPtr, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Create a new Counter ExternalPtr with the given initial value.
/// @name rpkg_externalptr
/// @examples
/// ptr <- extptr_counter_new(1L)
/// p <- extptr_point_new(0.1, 0.2)
/// @aliases extptr_counter_new extptr_point_new
/// @param initial Initial value for the counter.
#[miniextendr]
pub fn extptr_counter_new(initial: i32) -> miniextendr_api::externalptr::ExternalPtr<Counter> {
    miniextendr_api::externalptr::ExternalPtr::new(Counter { value: initial })
}

/// Get the current value from a Counter ExternalPtr.
/// @param ptr ExternalPtr wrapping a Counter.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_counter_get(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::prelude::SEXP;
    unsafe {
        match ExternalPtr::<Counter>::wrap_sexp(ptr) {
            Some(ext) => SEXP::scalar_integer(ext.value),
            None => SEXP::scalar_integer(i32::MIN), // NA_INTEGER equivalent
        }
    }
}

/// Increment the Counter value in-place via ErasedExternalPtr::downcast_mut.
/// @param ptr ExternalPtr wrapping a Counter.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_counter_increment(ptr: SEXP) -> SEXP {
    use miniextendr_api::prelude::SEXP;
    unsafe {
        // Get mutable access via downcast_mut on erased pointer
        let mut erased = ErasedExternalPtr::from_sexp(ptr);
        if let Some(counter) = erased.downcast_mut::<Counter>() {
            counter.value += 1;
            return SEXP::scalar_integer(counter.value);
        }
        SEXP::scalar_integer(i32::MIN) // NA_INTEGER equivalent
    }
}

/// Create a new Point ExternalPtr with the given coordinates.
/// @param x Numeric x-coordinate.
/// @param y Numeric y-coordinate.
#[miniextendr]
pub fn extptr_point_new(x: f64, y: f64) -> miniextendr_api::externalptr::ExternalPtr<Point> {
    miniextendr_api::externalptr::ExternalPtr::new(Point { x, y })
}

/// Get the x-coordinate from a Point ExternalPtr.
/// @param ptr ExternalPtr wrapping a Point.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_point_get_x(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::prelude::SEXP;
    unsafe {
        match ExternalPtr::<Point>::wrap_sexp(ptr) {
            Some(ext) => SEXP::scalar_real(ext.x),
            None => SEXP::scalar_real(f64::NAN),
        }
    }
}

/// Get the y-coordinate from a Point ExternalPtr.
/// @param ptr ExternalPtr wrapping a Point.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_point_get_y(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::prelude::SEXP;
    unsafe {
        match ExternalPtr::<Point>::wrap_sexp(ptr) {
            Some(ext) => SEXP::scalar_real(ext.y),
            None => SEXP::scalar_real(f64::NAN),
        }
    }
}

/// Test that interpreting a Point ExternalPtr as a Counter returns None (type mismatch).
/// @param ptr ExternalPtr wrapping a Point.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_type_mismatch_test(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::prelude::SEXP;
    unsafe {
        // Try to interpret a Point as a Counter - should return None
        match ExternalPtr::<Counter>::wrap_sexp(ptr) {
            Some(_) => SEXP::scalar_integer(1), // Unexpected success
            None => SEXP::scalar_integer(0),    // Expected failure - type mismatch
        }
    }
}

/// Test that wrap_sexp returns None for a null external pointer (R's new("externalptr")).
/// @param ptr A null ExternalPtr from R.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_null_test(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::prelude::SEXP;
    unsafe {
        // R's new("externalptr") creates a null external pointer
        // Our wrap_sexp should return None for it
        match ExternalPtr::<Counter>::wrap_sexp(ptr) {
            Some(_) => SEXP::scalar_integer(1), // Unexpected - null pointer should fail
            None => SEXP::scalar_integer(0),    // Expected - null pointer detected
        }
    }
}

/// Test `ErasedExternalPtr::is::<Counter>`() type check.
/// @param ptr ExternalPtr to test.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_is_counter(ptr: SEXP) -> SEXP {
    use miniextendr_api::prelude::SEXP;
    unsafe {
        let erased = ErasedExternalPtr::from_sexp(ptr);
        if erased.is::<Counter>() {
            SEXP::scalar_integer(1)
        } else {
            SEXP::scalar_integer(0)
        }
    }
}

/// Test `ErasedExternalPtr::is::<Point>`() type check.
/// @param ptr ExternalPtr to test.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_is_point(ptr: SEXP) -> SEXP {
    use miniextendr_api::prelude::SEXP;
    unsafe {
        let erased = ErasedExternalPtr::from_sexp(ptr);
        if erased.is::<Point>() {
            SEXP::scalar_integer(1)
        } else {
            SEXP::scalar_integer(0)
        }
    }
}

/// Test creating and reading an ExternalPtr entirely on the main thread.
#[miniextendr(noexport)]
pub fn test_extptr_on_main_thread() -> i32 {
    let ptr = ExternalPtr::new(Counter { value: 99 });
    ptr.value
}

// Exercise the documented `Send` contract with an R-owned pointer: argument
// conversion borrows the live EXTPTRSXP on the main thread, then the generated
// wrapper moves the handle to the worker before this function reads it.
#[cfg(feature = "worker-thread")]
#[miniextendr(worker, noexport)]
pub fn extptr_worker_value(ptr: ExternalPtr<Counter>) -> i32 {
    ptr.value
}

// Round-trip a borrowed handle through the worker. Returning it must preserve
// the original R identity; no fresh EXTPTRSXP or deep-cloned Counter is wanted.
#[cfg(feature = "worker-thread")]
#[miniextendr(worker, noexport)]
pub fn extptr_worker_identity(ptr: ExternalPtr<Counter>) -> ExternalPtr<Counter> {
    ptr
}

// `into_inner` performs checked R external-pointer operations. Calling it on
// the worker must route those operations to the main thread, move out Counter,
// and leave the original R external pointer cleared rather than double-freeing.
#[cfg(feature = "worker-thread")]
#[miniextendr(worker, noexport)]
pub fn extptr_worker_into_inner(ptr: ExternalPtr<Counter>) -> i32 {
    ExternalPtr::into_inner(ptr).value
}
