#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

mod bindings {
  include!("../r-bindings.rs");
}
pub use bindings::*;

#[repr(u32)]
pub enum SEXPTYPE {
  /// nil = NULL
  NILSXP = 0,
  /// symbols
  SYMSXP = 1,
  /// lists of dotted pairs
  LISTSXP = 2,
  /// closures
  CLOSXP = 3,
  /// environments
  ENVSXP = 4,
  /// promises: [un]evaluated closure arguments
  PROMSXP = 5,
  /// language constructs (special lists)
  LANGSXP = 6,
  /// special forms
  SPECIALSXP = 7,
  /// builtin non-special forms
  BUILTINSXP = 8,
  /// "scalar" string type (internal only
  CHARSXP = 9,
  /// logical vectors
  LGLSXP = 10,
  /// integer vectors
  INTSXP = 13,
  /// real variables
  REALSXP = 14,
  /// complex variables
  CPLXSXP = 15,
  /// string vectors
  STRSXP = 16,
  /// dot-dot-dot object
  DOTSXP = 17,
  /// make "any" args work
  ANYSXP = 18,
  /// generic vectors
  VECSXP = 19,
  /// expressions vectors
  EXPRSXP = 20,
  /// byte code
  BCODESXP = 21,
  /// external pointer
  EXTPTRSXP = 22,
  /// weak reference
  WEAKREFSXP = 23,
  /// raw bytes
  RAWSXP = 24,
  /// S4 non-vector
  S4SXP = 25,
  /// fresh node creaed in new page
  NEWSXP = 30,
  /// node released by GC
  FREESXP = 31,
  /// Closure or Builtin
  FUNSXP = 99,
}

// pub mod protect_macros;

//TODO: add support for ref-counts and other function-like macros
// mod r_function_like_macros;

/* From Rinternals.h
 *
enum {SORTED_DECR_NA_1ST = -2,
      SORTED_DECR = -1,
      UNKNOWN_SORTEDNESS = INT_MIN, /*INT_MIN is NA_INTEGER! */
      SORTED_INCR = 1,
      SORTED_INCR_NA_1ST = 2,
      KNOWN_UNSORTED = 0};
*/
