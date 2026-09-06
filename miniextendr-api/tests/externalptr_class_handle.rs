//! Integration tests for `ExternalPtr<T>` argument conversion accepting
//! class-wrapped handles (audit A9 —
//! `audit/2026-07-03-api-sense-conversions-dataframe-errors.md` #5).
//!
//! Covers the shapes reachable without R-level class machinery: an
//! environment binding `.ptr` directly, an object carrying `.ptr` as an
//! attribute (the S7 property-storage shape), and a list with a `.ptr`
//! element (a hand-built S3 wrapper with R-side state, #1469). R6 (`.__enclos_env__` ->
//! `private` -> `.ptr`) and S4 (`ptr` slot via `methods::slot()`) need
//! `R6::R6Class`/`setClass` machinery to construct a real instance, so those
//! are covered in rpkg testthat instead
//! (`test-conditions-comprehensive.R`).

mod r_test_utils;

use miniextendr_api::externalptr::ExternalPtr;
use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::prelude::{SEXP, SexpExt};
use miniextendr_api::sys::{Rf_defineVar, Rf_install};
use miniextendr_api::{OwnedProtect, r_str};

#[test]
fn externalptr_class_handle_suite() {
    r_test_utils::with_r_thread(|| {
        test_env_ptr_binding_unwraps();
        test_attribute_ptr_unwraps();
        test_list_ptr_element_unwraps();
        test_list_without_ptr_element_falls_through_to_attribute();
        test_plain_env_still_errors_with_hint();
        test_wrong_type_handle_still_reports_mismatch();
    });
}

/// A plain environment binding `.ptr` directly (the shape a user-authored
/// `#[miniextendr(env)]`-adjacent object, or any hand-rolled env wrapper,
/// would use) unwraps to the underlying `ExternalPtr`.
fn test_env_ptr_binding_unwraps() {
    let ext = ExternalPtr::new(41i32);
    let ptr_sexp = ext.as_sexp();

    let env = r_str!("new.env(parent = emptyenv())").expect("new.env() should evaluate");
    let _env_guard = unsafe { OwnedProtect::new(env) };

    unsafe {
        let sym = Rf_install(c".ptr".as_ptr());
        Rf_defineVar(sym, ptr_sexp, env);
    }

    let unwrapped = ExternalPtr::<i32>::try_from_sexp(env)
        .expect("env with a `.ptr` binding should unwrap to the ExternalPtr");
    assert_eq!(unwrapped.as_ref().copied(), Some(41));
}

/// An object carrying `.ptr` as an attribute (S7 stores properties as
/// attributes on the base object) unwraps the same way.
fn test_attribute_ptr_unwraps() {
    let ext = ExternalPtr::new(7i32);
    let ptr_sexp = ext.as_sexp();

    let carrier = SEXP::scalar_integer(0);
    let _carrier_guard = unsafe { OwnedProtect::new(carrier) };
    unsafe {
        let sym = Rf_install(c".ptr".as_ptr());
        carrier.set_attr(sym, ptr_sexp);
    }

    let unwrapped = ExternalPtr::<i32>::try_from_sexp(carrier)
        .expect("object with a `.ptr` attribute should unwrap to the ExternalPtr");
    assert_eq!(unwrapped.as_ref().copied(), Some(7));
}

/// A list with a `.ptr` element (the hand-built S3 wrapper shape,
/// `structure(list(.ptr = ptr, log = ...), class = "Foo")`) unwraps by name,
/// wherever the element sits.
fn test_list_ptr_element_unwraps() {
    let ext = ExternalPtr::new(11i32);
    let ptr_sexp = ext.as_sexp();

    let wrapper = r_str!("list(log = character(), .ptr = NULL)").expect("list() should evaluate");
    let _guard = unsafe { OwnedProtect::new(wrapper) };
    wrapper.set_vector_elt(1, ptr_sexp);

    let unwrapped = ExternalPtr::<i32>::try_from_sexp(wrapper)
        .expect("list with a `.ptr` element should unwrap to the ExternalPtr");
    assert_eq!(unwrapped.as_ref().copied(), Some(11));
}

/// A list whose names do not include `.ptr` is not a handle by element, but
/// the attribute check still runs on it (an S7 class over `class_list` is a
/// `VECSXP` with its properties stored as attributes).
fn test_list_without_ptr_element_falls_through_to_attribute() {
    let ext = ExternalPtr::new(13i32);
    let ptr_sexp = ext.as_sexp();

    let plain = r_str!("list(a = 1, b = 2)").expect("list() should evaluate");
    let _plain_guard = unsafe { OwnedProtect::new(plain) };
    ExternalPtr::<i32>::try_from_sexp(plain)
        .expect_err("named list without `.ptr` is not a handle");

    let carrier = r_str!("list(a = 1)").expect("list() should evaluate");
    let _carrier_guard = unsafe { OwnedProtect::new(carrier) };
    unsafe {
        let sym = Rf_install(c".ptr".as_ptr());
        carrier.set_attr(sym, ptr_sexp);
    }
    let unwrapped = ExternalPtr::<i32>::try_from_sexp(carrier)
        .expect("list with a `.ptr` attribute should unwrap to the ExternalPtr");
    assert_eq!(unwrapped.as_ref().copied(), Some(13));
}

/// An environment with no `.ptr` binding at all (and no R6 `.__enclos_env__`
/// chain) is not a handle — conversion fails with the new hint message
/// rather than the old bare `SexpTypeError`.
fn test_plain_env_still_errors_with_hint() {
    let env = r_str!("new.env(parent = emptyenv())").expect("new.env() should evaluate");
    let _guard = unsafe { OwnedProtect::new(env) };

    let err = ExternalPtr::<i32>::try_from_sexp(env).expect_err("plain env is not a handle");
    let msg = err.to_string();
    assert!(
        msg.contains("miniextendr class object"),
        "expected the class-handle hint in the error, got: {msg}"
    );
}

/// A `.ptr` attribute pointing at the *wrong* Rust type still fails — with
/// the existing downcast type-mismatch error, not the new hint. Unwrapping
/// loosens the accepted R-side shape, not `Any::downcast` type safety.
fn test_wrong_type_handle_still_reports_mismatch() {
    let ext = ExternalPtr::new(3.5f64);
    let ptr_sexp = ext.as_sexp();

    let carrier = SEXP::scalar_integer(0);
    let _carrier_guard = unsafe { OwnedProtect::new(carrier) };
    unsafe {
        let sym = Rf_install(c".ptr".as_ptr());
        carrier.set_attr(sym, ptr_sexp);
    }

    let err = ExternalPtr::<i32>::try_from_sexp(carrier)
        .expect_err("f64 handle should not satisfy ExternalPtr<i32>");
    let msg = err.to_string();
    assert!(
        msg.contains("i32") && msg.contains("f64"),
        "expected both type names in the downcast mismatch, got: {msg}"
    );
}
