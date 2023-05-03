use anyhow::{anyhow, Result};
use clang::EntityKind::*;
use indexmap::IndexSet;
use itertools::Itertools;
use std::path::Path;

pub fn find_macros(
  rsys_root: &Path, wrapper_path: &Path, include_path: &Path,
  r_headers_path: &Path,
) -> Result<()> {
  if !include_path.exists() {
    return Err(anyhow!(
      "R-headers are not present, use `cargo xtask copy-r-headers` first."
    ));
  }

  let clang = clang::Clang::new().unwrap();
  let index = clang::Index::new(&clang, true, true);

  let mut parser = index.parser(wrapper_path);
  let parser = parser.arguments(&[format!("-I{}", include_path.display(),)]);
  let tu = parser
        // .briefs_in_completion_results(true)
        .detailed_preprocessing_record(true)
        // .skip_function_bodies(true)
        .parse()?;
  let entities = tu
    .get_entity()
    .get_children()
    .into_iter()
    .filter(|x| !x.is_in_system_header());

  let skipped_ranges = tu
    .get_skipped_ranges()
    .into_iter()
    .filter(|x| !x.is_in_system_header())
    .collect_vec();
  dbg!(&skipped_ranges.len());
  // - [ ] print skipped ranges
  // - [ ] then use it to check if the found macros are within them, if so,
  //   don't print them

  let ranges: IndexSet<_> = skipped_ranges
        .into_iter()
        .chain(
            entities
                .filter(|e| !matches!(e.get_kind(), MacroDefinition | MacroExpansion))
                .flat_map(|x| x.get_range()),
        )
        .map(|ele| {
            assert_eq!(
                ele.get_start().get_file_location(),
                ele.get_start().get_expansion_location(),
            );
            assert_eq!(
                ele.get_end().get_file_location(),
                ele.get_end().get_expansion_location()
            );
            let line_start: usize = ele.get_start().get_file_location().line as _;
            let line_end: usize = ele.get_end().get_file_location().line as _;
            let source = ele
                .get_start()
                .get_file_location()
                .file
                .unwrap()
                .get_contents()
                .unwrap()
                .lines()
                .skip(line_start - 1)
                .take(line_end - line_start + 1)
                .join("\n");
            (
                ele.get_start().get_file_location().file.unwrap().get_path(),
                // ele.get_start().get_file_location().line..=ele.get_end().get_file_location().line,
                line_start..=line_end + 1,
                source,
            )
        })
        // does nothing
        // .unique_by(|x| (x.0.clone(), x.1.clone()))
        .collect();
  dbg!(ranges.len());

  // this doesn't include skipped ranges
  let all_entities = tu.get_entity().get_children();
  let r_entities =
    all_entities.into_iter().filter(|x| !x.is_in_system_header()).collect_vec();

  dbg!(r_entities.len());
  // extract all macros
  // dbg!(&r_entities.iter().filter(|x| !x.is_builtin_macro()).count()); // does
  // nothing
  let r_macro_entities: Vec<_> = r_entities
    .iter()
    .filter(|x| {
      matches!(
        x.get_kind(),
        clang::EntityKind::MacroDefinition | MacroExpansion
      )
    })
    .collect();
  dbg!(r_macro_entities.len());

  //TODO: add something here so that macros appear that are outside of
  // skipped ranges and then macros that don't appear in skipped ranges.
  let mut macros_and_ranges = IndexSet::new();
  for rmacro_entity in r_macro_entities {
    // skipping non-function-like-macros
    if !rmacro_entity.is_function_like_macro() {
      continue;
    }

    // check if the macro is in a skipped range
    let source_location = rmacro_entity.get_location().unwrap();
    let macro_matching_skipped_ranges = ranges
      .iter()
      .filter(|(file, range, _source)| {
        (*file == source_location.get_file_location().file.unwrap().get_path())
          && range.contains(&(source_location.get_file_location().line as _))
      })
      .collect_vec();
    assert!(macro_matching_skipped_ranges.len() <= 1);
    if let Some(skipped_range_containing_macro) =
      macro_matching_skipped_ranges.get(0).cloned()
    {
      macros_and_ranges.insert(skipped_range_containing_macro.clone());
      continue;
    }
    // alright this macro is not in a skipped range, but needs to be printed
    // anyways
    let source_range = rmacro_entity.get_range().unwrap();
    let line_start = source_range.get_start().get_file_location().line;
    let line_end = source_range.get_end().get_file_location().line;
    let line_start: usize = line_start as _;
    let line_end: usize = line_end as _;

    let source_file = source_location.get_file_location().file.unwrap();
    let source = source_file.get_contents().unwrap();
    let source = source
      .lines()
      .skip(line_start - 1)
      .take(line_end - line_start + 1)
      .join("\n");
    let source_location_path = source_file.get_path();
    macros_and_ranges.insert((
      source_location_path,
      line_start..=line_end,
      source,
    ));
  }
  let macros_and_ranges = macros_and_ranges;

  // this prints all ranges
  std::fs::write(
    rsys_root.join("macros_and_skipped_ranges.txt"),
    macros_and_ranges
      .iter()
      .map(|(file, _range, source)| {
        let source_location_path = file
          .strip_prefix(r_headers_path)
          .map(|x| x.display().to_string())
          .unwrap_or(format!("unable to strip r-include: {:}", file.display()));
        //FIXME

        format!("// {}\n{}\n", source_location_path, source)
      })
      .join("\n"),
  )?;

  let preprocessing_entities = r_entities
    .iter()
    .filter(|x| matches!(x.get_kind(), PreprocessingDirective))
    .collect_vec();
  dbg!(preprocessing_entities.len());
  // if this changes, the stuff in here has to be processed as well.
  assert_eq!(preprocessing_entities.len(), 0);

  Ok(())
}
