fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let out_dir = std::env::var("OUT_DIR")?;
    let bindings = bindgen::builder()
        .layout_tests(true)
        .clang_arg(format!("-I{}", "r/include/"))
        .clang_arg(format!(
            "-I{}",
            "C:\\rtools42\\x86_64-w64-mingw32.static.posix\\include"
        ))
        // .enable_cxx_namespaces() // yields only a `root` module.
        .enable_function_attribute_detection()
        .generate_block(true)
        .generate_comments(true)
        // .generate_inline_functions(true)   // a lot of stuff generated
        // .array_pointers_in_arguments(true) // does nothing
        // .bindgen_wrapper_union(".*")       // does nothing
        // enum?
        .rustified_enum(".*")
        // bitfield?
        // .bitfield_enum(".*")
        // comments
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .parse_callbacks(Box::new(TrimCommentsCallback))
        .clang_arg("-fparse-all-comments")
        .rustfmt_bindings(true)
        .header("wrapper.h")
        .generate()?;
    // bindings.emit_warnings();
    // let path_to_r_bindings = format!("{}/r-bindings.rs", out_dir);
    let path_to_r_bindings = format!("r-bindings.rs");
    bindings.write_to_file(&path_to_r_bindings)?;

    // make sure cargo links properly against library
    println!("cargo:rustc-link-search={}", "r/bin/x64/");
    println!("cargo:rustc-link-lib=dylib=R");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    Ok(()).into()
}

#[derive(Debug)]
struct TrimCommentsCallback;

impl bindgen::callbacks::ParseCallbacks for TrimCommentsCallback {
    /// Trims the comments found by clang.
    fn process_comment(&self, comment: &str) -> Option<String> {
        comment.trim().to_string().into()
    }
}
