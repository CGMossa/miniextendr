//! Tests for ExternalPtr identity preservation through function round-trips.
//!
//! Verifies that passing an ExternalPtr through a Rust function and back
//! returns the **same** R object (same SEXP), not a deep copy.

use miniextendr_api::externalptr::ExternalPtr;
use miniextendr_api::miniextendr;

/// A simple struct for identity-preservation tests.
#[derive(miniextendr_api::ExternalPtr)]
pub struct PtrIdentityTest {
    value: i32,
}

#[miniextendr(env)]
impl PtrIdentityTest {
    pub fn new(value: i32) -> Self {
        PtrIdentityTest { value }
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

/// Returns the ExternalPtr unchanged -- should preserve R identity.
/// @param x An ExternalPtr wrapping a PtrIdentityTest.
/// @name rpkg_externalptr_identity
/// @examples
/// a <- PtrIdentityTest$new(10L)
/// b <- ptr_identity(a)
/// identical(a, b) # TRUE
#[miniextendr]
pub fn ptr_identity(x: ExternalPtr<PtrIdentityTest>) -> ExternalPtr<PtrIdentityTest> {
    x
}

/// Takes two ExternalPtrs, returns the one with the larger value.
/// The returned SEXP should be the same as the input's SEXP.
/// @param a An ExternalPtr wrapping a PtrIdentityTest.
/// @param b An ExternalPtr wrapping a PtrIdentityTest.
#[miniextendr]
pub fn ptr_pick_larger(
    a: ExternalPtr<PtrIdentityTest>,
    b: ExternalPtr<PtrIdentityTest>,
) -> ExternalPtr<PtrIdentityTest> {
    if a.value >= b.value { a } else { b }
}
