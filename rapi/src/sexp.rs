use std::fmt::{Debug, Pointer};

use rsys::SEXP;

struct Sexp(SEXP);

impl Debug for Sexp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("Sexp").field(&self.0).finish()
  }
}

impl Pointer for Sexp {
  fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    todo!()
  }
}
