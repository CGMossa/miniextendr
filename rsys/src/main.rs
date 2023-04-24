use rsys::{Rf_endEmbeddedR, Rf_initEmbeddedR};

fn main() {
  println!("[Rust]: Hello world!");

  unsafe {
    let mut arg = [0_i8 as _; 0];
    Rf_initEmbeddedR(0, arg.as_mut_ptr());
    rsys::Rprintf("[R]: Hello world!\n".as_ptr() as _);

    Rf_endEmbeddedR(0);
  };
}
