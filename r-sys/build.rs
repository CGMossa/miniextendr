fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;
    // println!("cargo:rustc-env=R_HOME={}", r_paths.r_home.display());
    // println!("cargo:r_home={}", r_paths.r_home.display()); // Becomes DEP_R_R_HOME for clients
    let bindings = bindgen::builder()
        .clang_arg(format!("-I{}", "r/include/"))
        // .enable_cxx_namespaces()
        // .enable_function_attribute_detection()
        .header("wrapper.h")
        .generate()?;
    // bindings.emit_warnings();

    bindings.write_to_file(format!("{}/r-bindings.rs", out_dir))?;

    // make sure cargo links properly against library
    // println!("cargo:rustc-link-search={}", "r/bin/x64/");
    // println!("cargo:rustc-link-lib=dylib=R");

    // println!("cargo:rerun-if-changed=build.rs");
    // println!("cargo:rerun-if-changed=wrapper.h");
    Ok(()).into()
}
