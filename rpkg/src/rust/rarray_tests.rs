//! Test fixtures for RArray/RMatrix/RVector.

use miniextendr_api::prelude::SEXP;
use miniextendr_api::prelude::*;
use miniextendr_api::rarray::{RMatrix, RVector};
use miniextendr_api::{OwnedProtect, SEXPTYPE, sys};

/// Get dimensions of a matrix as integer vector.
#[miniextendr]
pub fn rarray_matrix_dims(x: SEXP) -> Vec<i32> {
    let mat = unsafe { RMatrix::<f64>::from_sexp(x).expect("expected numeric matrix") };
    let dims = unsafe { mat.dims() };
    vec![dims[0] as i32, dims[1] as i32]
}

/// Get total length of a matrix.
#[miniextendr]
pub fn rarray_matrix_len(x: SEXP) -> i32 {
    let mat = unsafe { RMatrix::<f64>::from_sexp(x).expect("expected numeric matrix") };
    mat.len() as i32
}

/// Sum all elements of a numeric vector via RVector.
#[miniextendr]
pub fn rarray_vector_sum(x: SEXP) -> f64 {
    let vec = unsafe { RVector::<f64>::from_sexp(x).expect("expected numeric vector") };
    let slice = unsafe { vec.as_slice() };
    slice.iter().sum()
}

/// Get a specific column from a numeric matrix as a Vec.
/// @param x A numeric matrix.
/// @param col 1-based column index. Errors if `col` is not in `1..=ncol`.
#[miniextendr]
pub fn rarray_matrix_column(x: SEXP, col: i32) -> Vec<f64> {
    let mat = unsafe { RMatrix::<f64>::from_sexp(x).expect("expected numeric matrix") };
    let ncol = unsafe { mat.dims()[1] };
    if col < 1 {
        panic!(
            "column {col} is out of bounds (must be a positive 1-based index, matrix has {ncol} columns)"
        );
    }
    let column = unsafe { mat.column(col as usize - 1) };
    column.to_vec()
}

/// Construct a numeric matrix while allocating inside the initializer.
/// @param nrow Non-negative row count.
/// @param ncol Non-negative column count.
#[miniextendr(noexport)]
pub fn rarray_construct_matrix(nrow: i32, ncol: i32) -> RMatrix<f64> {
    let nrow = usize::try_from(nrow).expect("nrow must be non-negative");
    let ncol = usize::try_from(ncol).expect("ncol must be non-negative");

    unsafe {
        RMatrix::new([nrow, ncol], |slice| {
            // Force a GC-capable allocation while the constructor's data SEXP
            // and the mutable slice derived from it must remain valid.
            let _scratch = OwnedProtect::new(sys::Rf_allocVector(SEXPTYPE::REALSXP, 1));
            slice.fill(42.0);
        })
    }
}
