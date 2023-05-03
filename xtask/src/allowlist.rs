use anyhow::Result;
use indexmap::IndexSet;
use std::path::Path;

use itertools::Itertools;

pub fn allowlist(
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
