fn main() {
    println!("{}", unsafe { r_sys::tan(0.4) });
    println!("{}", unsafe { r_sys::tanh(0.4) });
    println!("{}", unsafe { r_sys::tanh(0.324) });
    println!("Hello world!");

    unsafe { r_sys::rprintf("Hello world!".as_ptr() as _) };
}
