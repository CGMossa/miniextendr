use anyhow::{Context, Result};
use std::{
  env, fs,
  path::{Path, PathBuf},
};

// TODO
/* rust

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("dist") => dist()?,
        _ => print_help(),
    }
    Ok(())
}

*/

fn main() -> Result<()> {
  let meta =
    rustc_version::version_meta().context("Failed to get Rust version info")?;
  match meta.channel {
    rustc_version::Channel::Dev => println!("cargo:rustc-cfg=dev"),
    rustc_version::Channel::Nightly => println!("cargo:rustc-cfg=nightly"),
    rustc_version::Channel::Beta => println!("cargo:rustc-cfg=beta"),
    rustc_version::Channel::Stable => println!("cargo:rustc-cfg=stable"),
  }
  let rsys_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
  // let out_dir = env::var("OUT_DIR")?;
  let libgcc_var = env::var("LIBRARY_PATH")
    .context("Set `LIBRARY_PATH` on your System / User")?;
  if libgcc_var.is_empty() {
    anyhow::bail!("Environment variable `LIBRARY_PATH` cannot be empty.")
  }
  // create a directory in an arbitrary location (e.g. libgcc_mock)
  let libgcc_mock_path = Path::new(&libgcc_var);
  if !libgcc_mock_path.exists() {
    fs::create_dir(libgcc_mock_path)?
  }
  if !libgcc_mock_path.join("libgcc_eh.a").exists() {
    fs::File::create(libgcc_mock_path.join("libgcc_eh.a"))?;
    fs::File::create(libgcc_mock_path.join("libgcc_s.a"))?;
  }

  let target = env::var("TARGET").context("Could not get the target triple")?;
  
  let allowlist_path = rsys_dir.join("allowlist.txt");
  println!("cargo:rerun-if-changed={}", allowlist_path.display());
  let allowlist_pattern = std::fs::read_to_string(allowlist_path)?;
  let allowlist_pattern: String = allowlist_pattern.lines().collect();
  let bindings_builder = bindgen::builder()
        // .layout_tests(true)
        .clang_arg(format!(
            "-I{}",
            "C:\\rtools42\\x86_64-w64-mingw32.static.posix\\include" // r#"C:\Users\minin\scoop\apps\llvm\current\include"#
        ))
        // Blocklist some types on i686
        // https://github.com/rust-lang/rust-bindgen/issues/1823
        // https://github.com/rust-lang/rust/issues/54341
        // https://github.com/extendr/libR-sys/issues/39
        // .blocklist_item("max_align_t")
        .blocklist_item("__mingw_ldbl_type_t")
        // .clang_arg(format!(
        //     "-I{}",
        //     r#"C:\Users\minin\scoop\apps\llvm\current\include"#
        // ))
        .allowlist_function(&allowlist_pattern)
        .allowlist_type(&allowlist_pattern)
        .allowlist_var(&allowlist_pattern)
        // Remove constants
        .blocklist_item("^M_.*")
        .clang_arg(format!("--target={target}"))
        .clang_arg(format!("-I{}", "r/include/"))
        .clang_arg("-std=c11")
        //https://cran.r-project.org/doc/manuals/R-exts.html#Portable-C-and-C_002b_002b-code
        .blocklist_function("finite")
        // .fit_macro_constants(true)
        .default_enum_style(bindgen::EnumVariation::NewType {
            is_bitfield: false,
            is_global: false,
        })
        // .enable_cxx_namespaces() // yields only a `root` module.
        // .enable_function_attribute_detection()
        // .constified_enum_module(".*")
        // .conservative_inline_namespaces()
        .rustified_enum(".*")
        .generate_block(true)
        // .generate_inline_functions(true)   // a lot of stuff generated
        // .array_pointers_in_arguments(true) // does nothing
        // .bindgen_wrapper_union(".*")       // does nothing
        .translate_enum_integer_types(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate_comments(true)
        .clang_arg("-fparse-all-comments")
        .parse_callbacks(Box::new(TrimCommentsCallback))
        .formatter(bindgen::Formatter::Rustfmt)
        .header("wrapper.h");

  //TODO: add nonAPI.txt
  // Rscript -e 'cat(tools:::nonAPI, "\n")' | uniq | sort

  let bindings = bindings_builder
        // .rust_target(bindgen::RustTarget::Stable_1_64)
        .generate()?;
  // let path_to_r_bindings = format!("{}/r-bindings.rs", out_dir);
  let path_to_r_bindings = "r-bindings.rs";
  bindings.write_to_file(path_to_r_bindings)?;
  println!("cargo:rerun-if-changed={path_to_r_bindings}");

  // make sure cargo links properly against library
  let rlib_path = rsys_dir.join("r").join("bin").join("x64").canonicalize()?;
  let rlib_path = rlib_path.display();
  println!("cargo:rustc-link-search={rlib_path}");
  println!("cargo:rustc-link-lib=dylib=R");

  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=wrapper.h");

  Ok(())
}

#[derive(Debug)]
struct TrimCommentsCallback;

impl bindgen::callbacks::ParseCallbacks for TrimCommentsCallback {
  /// Trims the comments found by clang.
  fn process_comment(&self, comment: &str) -> Option<String> {
    let mut comment = comment.trim().to_string();

    let finder = linkify::LinkFinder::new();
    let comment_links = comment.clone();
    let links = finder.links(comment_links.as_str()).collect::<Vec<_>>();
    for link in &links {
      comment.replace_range(
        link.start()..link.end(),
        &format!("<{}>", link.as_str()),
      );
    }
    // let comment = comment.replace("\\n", "\n");
    // let comment = comment.replace("[", r"`[");
    // let comment = comment.replace("]", r"]`");
    Some(comment)
  }
  //TODO: this needs more information before it can be useful
  // fn item_name(&self, original_item_name: &str) -> Option<String> {
  //     // if all uppercase, assume constant
  //     let all_is_uppercase = original_item_name.chars().all(|x| match x {
  //         '_' => true,
  //         a if a.is_alphabetic() => a.is_uppercase(),
  //         _ => true,
  //     });
  //     if all_is_uppercase {
  //         return None;
  //     }
  //     // assume camel case
  //     let new_item_name = original_item_name
  //         .char_indices()
  //         .flat_map(|(x, a)| {
  //             [
  //                 (x != 0 && a.is_uppercase()).then_some('_'),
  //                 Some(a.to_ascii_lowercase()),
  //             ]
  //         })
  //         .flatten()
  //         .collect();
  //     Some(new_item_name)
  // }

  // fn generated_name_override(
  //     &self,
  //     item_info: bindgen::callbacks::ItemInfo<'_>,
  // ) -> Option<String> {
  //     match item_info.kind {
  //         bindgen::callbacks::ItemKind::Function => None,
  //         bindgen::callbacks::ItemKind::Var => {
  //             // assume camel case
  //             let original_item_name = item_info.name;
  //             let new_item_name = original_item_name
  //                 .char_indices()
  //                 .flat_map(|(x, a)| {
  //                     [
  //                         (x != 0 && a.is_uppercase()).then_some('_'),
  //                         Some(a.to_ascii_lowercase()),
  //                     ]
  //                 })
  //                 .flatten()
  //                 .collect();
  //             Some(new_item_name)
  //         }
  //         _ => todo!(),
  //     }
  // }
}
