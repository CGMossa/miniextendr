use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use indexmap::IndexSet;
use std::path::{Path, PathBuf};

use itertools::Itertools;

/// Tasks to aid in the development of R FFI wrappers.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[command(subcommand)]
  command: Command,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
  /// Create a fake library to satisfy the error `-lgcc_eh`
  /// that occurs when using the Rtools linker.
  /// This needs to be done system-wide, thus it needs an
  /// environment variable to point to where the fake library
  /// should reside.
  LibraryGccMock {},
  /// Copy R headers and binaries to the workspace directory.
  CopyRHeaders {
    //TODO: optional R_HOME
    //TODO: optional target-directory ~> default to r-sys-root
  },
  /// Produce a list of the symbols to be ported through bindgen.
  Allowlist {},
}

fn main() -> Result<()> {
  let xtask_directory: PathBuf = env!("CARGO_MANIFEST_DIR").into();
  let workspace_root: &Path = xtask_directory.parent().unwrap();

  let args = Args::parse();
  match args.command {
    Command::LibraryGccMock {} => libgcc_mock()?,
    Command::CopyRHeaders {} => {
      let r_sys_root = workspace_root.join("rsys");
      copy_r_headers(r_sys_root.as_path())?
    }
    Command::Allowlist {} => allowlist(
      &workspace_root.join("rsys").join("wrapper.h"),
      &workspace_root.join("rsys").join("r").join("include"),
      &workspace_root.join("rsys"),
    )?,
  };
  Ok(())
}

fn libgcc_mock() -> Result<()> {
  // let out_dir = env::var("OUT_DIR")?;
  let libgcc_var = std::env::var("LIBRARY_PATH")
    .context("Set `LIBRARY_PATH` on your System / User")?;
  if libgcc_var.is_empty() {
    anyhow::bail!("Environment variable `LIBRARY_PATH` cannot be empty.")
  }
  // create a directory in an arbitrary location (e.g. libgcc_mock)
  let libgcc_mock_path = Path::new(&libgcc_var);
  if !libgcc_mock_path.exists() {
    std::fs::create_dir(libgcc_mock_path)?
  }
  if !libgcc_mock_path.join("libgcc_eh.a").exists() {
    std::fs::File::create(libgcc_mock_path.join("libgcc_eh.a"))?;
    std::fs::File::create(libgcc_mock_path.join("libgcc_s.a"))?;
  }
  Ok(())
}

/// Copy R headers and binaries over to the repository
fn copy_r_headers(r_sys_root: &Path) -> Result<()> {
  let r_copied_headers_path = r_sys_root.join("r");
  let r_home: PathBuf = env!("R_HOME").into();

  fs_extra::dir::remove(&r_copied_headers_path)?;
  std::fs::create_dir_all(&r_copied_headers_path)?;
  fs_extra::dir::copy(
    r_home.join("include"),
    &r_copied_headers_path,
    &fs_extra::dir::CopyOptions::new(),
  )?;

  // //TODO: Only copy over the used DLLs.
  let r_copied_binaries = r_copied_headers_path.join("bin");
  std::fs::create_dir_all(&r_copied_binaries)?;
  fs_extra::dir::copy(
    r_home.join("bin").join("x64"),
    &r_copied_binaries,
    &fs_extra::dir::CopyOptions::new(),
  )?;
  Ok(())
}

fn allowlist(
  main_header: &Path, include_path: &Path, rsys_path: &Path,
) -> Result<()> {
  use clang::{Clang, Index};

  let clang = Clang::new().unwrap();
  let index = Index::new(&clang, false, false);

  // Parse wrapper.h
  let tu = index
    .parser(main_header)
    // .parser("wrapper.h")
    .arguments(&[format!("-I{}", include_path.display()),
    // format!("--target={target}")
    "-std=c11".into()])
    .skip_function_bodies(true)
    .detailed_preprocessing_record(true)
    .parse()
    .unwrap();

  for ele in tu.get_diagnostics() {
    if let clang::diagnostic::Severity::Error = ele.get_severity() {
      eprintln!("{}", ele.get_text());
    }
  }

  // Extract all the AST entities into `e`, as well as listing up all the
  // include files in a chain into `include_files`.
  let r_ast_entities: indexmap::IndexSet<_> = tu
    .get_entity()
    .get_children()
    .into_iter()
    .filter(|x| !x.is_in_system_header())
    .collect();

  // Put all the symbols into allowlist
  let r_symbols = r_ast_entities.into_iter().filter(|x| {
    use clang::EntityKind::*;
    matches!(
      x.get_kind(),
      EnumDecl
        | FunctionDecl
        | TypedefDecl
        | StructDecl
        | VarDecl
        | UnionDecl
        | EnumConstantDecl
        | GenericSelectionExpr
        | MacroDefinition
        | MacroExpansion
    )
  });
  let (anonymous_items, named_items): (Vec<_>, Vec<_>) =
    r_symbols.partition(|x| x.is_anonymous());
  // print annonymous items as well to figure out what to do about them
  std::fs::remove_file(rsys_path.join("anonymous_items.txt")).unwrap_or(());
  std::fs::write(
    rsys_path.join("anonymous_items.txt"),
    anonymous_items
      .into_iter()
      .map(|x| {
        let range = x.get_range().unwrap();

        let start_line: usize = range.get_start().get_file_location().line as _;
        let end_line: usize = range.get_end().get_file_location().line as _;
        // dbg!(range, start_line, end_line);
        let file = range.get_start().get_file_location().file.unwrap();
        let contents: String = file
          .get_contents()
          .unwrap()
          .lines()
          .skip(start_line - 1)
          .take(end_line - start_line + 1)
          .join("\n");

        Ok(format!(
          "// {}\n{}",
          file.get_path().strip_prefix(rsys_path)?.display(),
          contents
        ))
      })
      .collect::<Result<Vec<_>>>()?
      .join("\n"),
  )?;

  let mut allowlist: IndexSet<_> = named_items
    .into_iter()
    .filter(|e| {
      // skip unnamed items
      // this occurs on llvm 16.0.0, see
      // https://github.com/rust-lang/rust-bindgen/issues/2488
      // and this is how it is decided to check for this
      !e.is_anonymous()
    })
    .map(|x| x.get_name().unwrap())
    .collect();

  // This cannot be detected because the #define-ed constants are aliased in
  // another #define c.f. https://github.com/wch/r-source/blob/9f284035b7e503aebe4a804579e9e80a541311bb/src/include/R_ext/GraphicsEngine.h#L93
  allowlist.insert("R_GE_version".to_string());

  //TODO:
  // // Join into a regex pattern to supply into bindgen::Builder.
  // let allowlist_pattern = allowlist
  //     // Exclude non-API calls
  //     .difference(&get_non_api())
  //     .cloned()
  //     .collect::<Vec<_>>();
  let allowlist: Vec<_> = allowlist.into_iter().collect();
  std::fs::remove_file(rsys_path.join("allowlist.txt")).unwrap_or(());
  std::fs::write(rsys_path.join("allowlist.txt"), allowlist.join("|\n"))?;
  Ok(())
}
