fn main() {
  println!("{}", unsafe { rsys::tan(0.4) });
  println!("{}", unsafe { rsys::tanh(0.4) });
  println!("{}", unsafe { rsys::tanh(0.324) });
  println!("Hello world!");

  unsafe { rsys::Rprintf("Hello world!".as_ptr() as _) };
}
