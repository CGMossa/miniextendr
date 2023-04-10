fn main() {
    println!("{}", unsafe { r_sys::tan(0.4) });
    println!("{}", unsafe { r_sys::tanh(0.4) });
    println!("Hello world!");
}
