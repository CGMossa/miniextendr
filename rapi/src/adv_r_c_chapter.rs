// use rsys::{protect_macros::*, *};

/*

SEXP add(SEXP a, SEXP b) {
  SEXP result = PROTECT(allocVector(REALSXP, 1));
  REAL(result)[0] = asReal(a) + asReal(b);
  UNPROTECT(1);

  return result;
}
*/

// unsafe fn add(a: SEXP, b: SEXP) -> SEXP {
//   let mut result: SEXP = PROTECT(Rf_allocVector(REALSXP, 1));
//   *REAL(result) = Rf_asReal(a) + Rf_asReal(b);
//   UNPROTECT(1);
//   result
// }

// #[cfg(test)]
// mod tests {
//   use super::*;

//   #[test]
//   fn test_add() {
//     let a = 46;
//     let b = -4;

//     // println!("{}", *add(a, b));
//   }
// }
