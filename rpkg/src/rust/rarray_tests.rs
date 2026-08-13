//! Test fixtures for RArray/RMatrix/RVector.

use miniextendr_api::list::List;
use miniextendr_api::prelude::SEXP;
use miniextendr_api::prelude::*;
use miniextendr_api::rarray::{RMatrix, RVector};

fn strings_from_sexp(sexp: SEXP) -> Vec<String> {
    Vec::<String>::try_from_sexp(sexp).expect("expected character names")
}

/// Get dimensions of a matrix as integer vector.
/// @param x An R vector or matrix accepted by the selected helper.
#[miniextendr]
pub fn rarray_matrix_dims(x: SEXP) -> Vec<i32> {
    let mat = unsafe { RMatrix::<f64>::from_sexp(x).expect("expected numeric matrix") };
    let dims = unsafe { mat.dims() };
    vec![dims[0] as i32, dims[1] as i32]
}

/// Get total length of a matrix.
/// @param x An R vector or matrix accepted by the selected helper.
#[miniextendr]
pub fn rarray_matrix_len(x: SEXP) -> i32 {
    let mat = unsafe { RMatrix::<f64>::from_sexp(x).expect("expected numeric matrix") };
    mat.len() as i32
}

/// Sum all elements of a numeric vector via RVector.
/// @param x An R vector or matrix accepted by the selected helper.
#[miniextendr]
pub fn rarray_vector_sum(x: SEXP) -> f64 {
    let vec = unsafe { RVector::<f64>::from_sexp(x).expect("expected numeric vector") };
    let slice = unsafe { vec.as_slice() };
    slice.iter().sum()
}

/// Get a specific column from a numeric matrix as a Vec.
/// @param x An R vector or matrix accepted by the selected helper.
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

/// Return row names through `RMatrix::get_rownames`.
/// @param x An R vector or matrix accepted by the selected helper.
#[miniextendr]
pub fn rarray_matrix_rownames(x: SEXP) -> Vec<String> {
    let mat = unsafe { RMatrix::<f64>::from_sexp(x).expect("expected numeric matrix") };
    let names = unsafe { mat.get_rownames().expect("expected row names") };
    strings_from_sexp(names)
}

/// Return column names through `RMatrix::get_colnames`.
/// @param x An R vector or matrix accepted by the selected helper.
#[miniextendr]
pub fn rarray_matrix_colnames(x: SEXP) -> Vec<String> {
    let mat = unsafe { RMatrix::<f64>::from_sexp(x).expect("expected numeric matrix") };
    let names = unsafe { mat.get_colnames().expect("expected column names") };
    strings_from_sexp(names)
}

/// Return row names through `List::get_rownames` for a VECSXP matrix.
/// @param x An R vector or matrix accepted by the selected helper.
#[miniextendr]
pub fn list_matrix_rownames(x: SEXP) -> Vec<String> {
    let list = List::try_from_sexp(x).expect("expected list matrix");
    strings_from_sexp(list.get_rownames().expect("expected row names"))
}

/// Return column names through `List::get_colnames` for a VECSXP matrix.
/// @param x An R vector or matrix accepted by the selected helper.
#[miniextendr]
pub fn list_matrix_colnames(x: SEXP) -> Vec<String> {
    let list = List::try_from_sexp(x).expect("expected list matrix");
    strings_from_sexp(list.get_colnames().expect("expected column names"))
}
