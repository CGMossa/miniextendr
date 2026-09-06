use super::*;

// region: Doc-lint feature tests (implicit title/description extraction)

#[cfg(feature = "doc-lint")]
mod doc_lint_tests {
    use super::*;

    /// Helper to create doc attributes from lines (simulates `/// line1`, `/// line2`, etc.)
    fn make_doc_attrs(lines: &[&str]) -> Vec<syn::Attribute> {
        lines
            .iter()
            .map(|line| syn::parse_quote!(#[doc = #line]))
            .collect()
    }

    #[test]
    fn test_implicit_title_simple() {
        let attrs = make_doc_attrs(&["Simple title"]);
        assert_eq!(
            implicit_title_from_attrs(&attrs),
            Some("Simple title".to_string())
        );
    }

    #[test]
    fn test_implicit_title_with_period() {
        // Title ends at first period
        let attrs = make_doc_attrs(&["This is the title. This is description."]);
        assert_eq!(
            implicit_title_from_attrs(&attrs),
            Some("This is the title".to_string())
        );
    }

    #[test]
    fn test_implicit_title_trailing_period_stripped() {
        let attrs = make_doc_attrs(&["Title with trailing period."]);
        assert_eq!(
            implicit_title_from_attrs(&attrs),
            Some("Title with trailing period".to_string())
        );
    }

    #[test]
    fn test_implicit_title_multiline_before_blank() {
        // Title spans multiple lines until blank line
        let attrs = make_doc_attrs(&[
            "First part of title",
            "second part of title",
            "",
            "Description",
        ]);
        assert_eq!(
            implicit_title_from_attrs(&attrs),
            Some("First part of title second part of title".to_string())
        );
    }

    #[test]
    fn test_implicit_title_none_when_starts_with_tag() {
        let attrs = make_doc_attrs(&["@param x A parameter"]);
        assert_eq!(implicit_title_from_attrs(&attrs), None);
    }

    #[test]
    fn test_implicit_title_empty_docs() {
        let attrs: Vec<syn::Attribute> = vec![];
        assert_eq!(implicit_title_from_attrs(&attrs), None);
    }

    #[test]
    fn test_implicit_description_is_second_paragraph() {
        // First paragraph = title, second paragraph = description
        let attrs = make_doc_attrs(&["This is the title.", "", "This is the description."]);
        assert_eq!(
            implicit_description_from_attrs(&attrs),
            Some("This is the description.".to_string())
        );
    }

    #[test]
    fn test_implicit_description_multiline_second_paragraph() {
        let attrs = make_doc_attrs(&[
            "Title line.",
            "",
            "First line of description.",
            "Second line of description.",
        ]);
        assert_eq!(
            implicit_description_from_attrs(&attrs),
            Some("First line of description. Second line of description.".to_string())
        );
    }

    #[test]
    fn test_implicit_description_stops_at_third_paragraph() {
        let attrs = make_doc_attrs(&[
            "Title.",
            "",
            "This is description.",
            "",
            "This is details (not description).",
        ]);
        assert_eq!(
            implicit_description_from_attrs(&attrs),
            Some("This is description.".to_string())
        );
    }

    #[test]
    fn test_implicit_description_none_when_only_one_paragraph() {
        // Only a title, no second paragraph
        let attrs = make_doc_attrs(&["Just a title."]);
        assert_eq!(implicit_description_from_attrs(&attrs), None);
    }

    #[test]
    fn test_implicit_description_none_when_starts_with_tag() {
        let attrs = make_doc_attrs(&["@title Explicit title", "@description Explicit desc"]);
        assert_eq!(implicit_description_from_attrs(&attrs), None);
    }

    #[test]
    fn test_implicit_description_skips_multiple_blank_lines() {
        let attrs = make_doc_attrs(&["Title.", "", "", "Description after multiple blanks."]);
        assert_eq!(
            implicit_description_from_attrs(&attrs),
            Some("Description after multiple blanks.".to_string())
        );
    }

    // region: #1172 — lint must only fire on author-written @description

    /// Multi-paragraph prose with no explicit tags must not warn. Before the
    /// fix, `doc_conflict_warnings` saw the `@description` synthesized from
    /// leading prose by `roxygen_tags_from_attrs` and compared it against the
    /// second paragraph — a guaranteed mismatch for every multi-paragraph doc.
    #[test]
    fn test_no_desc_warning_for_prose_only_docs() {
        let attrs = make_doc_attrs(&[
            "Create a lazy integer sequence ALTREP.",
            "",
            "Elements are computed on demand.",
            "@param n Length.",
        ]);
        let warnings = doc_conflict_warnings(&attrs, proc_macro2::Span::call_site());
        assert!(
            warnings.is_empty(),
            "prose-only docs must not trigger the @description lint: {warnings}"
        );
    }

    #[test]
    fn test_desc_warning_fires_on_drifted_explicit_description() {
        let attrs = make_doc_attrs(&[
            "Title.",
            "",
            "Second paragraph.",
            "@description Something entirely different.",
        ]);
        let warnings = doc_conflict_warnings(&attrs, proc_macro2::Span::call_site());
        assert!(
            warnings.to_string().contains("MINIEXTENDR_DOC_LINT_DESC"),
            "drifted explicit @description must still warn: {warnings}"
        );
    }

    #[test]
    fn test_no_desc_warning_when_explicit_matches_second_paragraph() {
        let attrs = make_doc_attrs(&[
            "Title.",
            "",
            "Second paragraph.",
            "@description Second paragraph.",
        ]);
        let warnings = doc_conflict_warnings(&attrs, proc_macro2::Span::call_site());
        assert!(
            warnings.is_empty(),
            "matching explicit @description must not warn: {warnings}"
        );
    }

    // endregion
}
// endregion

// region: #613 — doc attrs partitioned before non-doc attrs (Option B from #586)
//
// With partition-based normalization, all doc attrs are collected and processed
// contiguously first (stable order within the doc group). Non-doc attrs like
// #[cfg(...)] are skipped entirely for roxygen processing.  This means doc
// content that was separated by a #[cfg(...)] in source order now correctly
// continues multiline tags.

#[test]
fn attr_interrupt_cfg_between_examples_lines_doc_continues() {
    // Simulates: /// @examples\n/// ex()\n  #[cfg(feature="x")]\n/// more_ex()
    // With partition, all doc attrs are processed contiguously, so more_ex()
    // DOES continue the @examples block (correct behaviour).
    let attrs: Vec<syn::Attribute> = vec![
        syn::parse_quote!(#[doc = "@examples"]),
        syn::parse_quote!(#[doc = "ex()"]),
        syn::parse_quote!(#[cfg(feature = "x")]),
        syn::parse_quote!(#[doc = "more_ex()"]),
    ];
    let tags = roxygen_tags_from_attrs(&attrs);
    let examples_tag = tags.iter().find(|t| t.starts_with("@examples")).unwrap();
    assert!(
        examples_tag.contains("more_ex()"),
        "doc after cfg must continue @examples with partition: {:?}",
        tags
    );
}

#[test]
fn attr_interrupt_before_any_doc_does_not_affect_result() {
    // Non-doc attr BEFORE any doc content is harmless
    let attrs: Vec<syn::Attribute> = vec![
        syn::parse_quote!(#[cfg(feature = "x")]),
        syn::parse_quote!(#[doc = "@param x A value"]),
    ];
    let tags = roxygen_tags_from_attrs(&attrs);
    assert!(tags.iter().any(|t| t.starts_with("@param")));
}

#[test]
fn attr_interrupt_cfg_between_return_lines_doc_continues() {
    // Split: @return block, then cfg, then bare prose.
    // With partition, prose DOES continue the @return tag (correct behaviour).
    let attrs: Vec<syn::Attribute> = vec![
        syn::parse_quote!(#[doc = "@return The output value."]),
        syn::parse_quote!(#[cfg(feature = "x")]),
        syn::parse_quote!(#[doc = "Extra continuation line."]),
    ];
    let tags = roxygen_tags_from_attrs(&attrs);
    let return_tag = tags.iter().find(|t| t.starts_with("@return")).unwrap();
    assert!(
        return_tag.contains("Extra continuation line."),
        "@return must include post-cfg doc continuation with partition: {:?}",
        tags
    );
}

#[test]
fn attr_interrupt_multiple_cfg_between_doc_all_continue() {
    // Multiple non-doc attrs between doc lines — all doc still processes contiguously
    let attrs: Vec<syn::Attribute> = vec![
        syn::parse_quote!(#[doc = "@examples"]),
        syn::parse_quote!(#[cfg(feature = "a")]),
        syn::parse_quote!(#[doc = "line_a()"]),
        syn::parse_quote!(#[cfg(feature = "b")]),
        syn::parse_quote!(#[doc = "line_b()"]),
    ];
    let tags = roxygen_tags_from_attrs(&attrs);
    let examples_tag = tags.iter().find(|t| t.starts_with("@examples")).unwrap();
    assert!(
        examples_tag.contains("line_a()") && examples_tag.contains("line_b()"),
        "all doc lines must continue @examples across multiple cfg: {:?}",
        tags
    );
}

// endregion

// region: method doc prose is promoted to @description, never @title
//
// The page @title is the structural class/method name (emitted by ClassDocBuilder /
// lib.rs), so the core never derives a @title from prose. Leading prose folds into a
// single multi-paragraph @description.

fn make_r6_method_doc_attrs(lines: &[&str]) -> Vec<syn::Attribute> {
    lines
        .iter()
        .map(|line| syn::parse_quote!(#[doc = #line]))
        .collect()
}

#[test]
fn method_prose_emits_description_not_title() {
    let attrs = make_r6_method_doc_attrs(&["Compute the sum of all elements."]);
    let tags = roxygen_tags_from_attrs_for_r6_method(&attrs);
    assert!(
        !tags.iter().any(|t| t.starts_with("@title")),
        "prose must not become @title: {:?}",
        tags
    );
    let desc = tags.iter().find(|t| t.starts_with("@description"));
    assert!(desc.is_some(), "expected @description: {:?}", tags);
    assert!(
        desc.unwrap().contains("Compute the sum of all elements."),
        "wrong @description content: {:?}",
        tags
    );
}

#[test]
fn method_prose_paragraphs_fold_into_one_description() {
    let attrs = make_r6_method_doc_attrs(&[
        "Compute the sum of all elements.",
        "",
        "This is the second paragraph.",
    ]);
    let tags = roxygen_tags_from_attrs_for_r6_method(&attrs);
    assert!(
        !tags
            .iter()
            .any(|t| t.starts_with("@title") || t.starts_with("@details")),
        "no @title or @details from prose: {:?}",
        tags
    );
    let desc = tags.iter().find(|t| t.starts_with("@description")).unwrap();
    // Both paragraphs land in the description, separated by a blank line.
    assert!(
        desc.contains("Compute the sum of all elements.")
            && desc.contains("This is the second paragraph."),
        "both paragraphs must be in @description: {:?}",
        tags
    );
}

#[test]
fn method_no_doc_emits_nothing() {
    let attrs: Vec<syn::Attribute> = vec![];
    let tags = roxygen_tags_from_attrs_for_r6_method(&attrs);
    assert!(
        tags.is_empty(),
        "expected empty tags for no-doc: {:?}",
        tags
    );
}

// endregion

// region: prose → @description promotion (via roxygen_tags_from_attrs)

fn make_doc_attrs_plain(lines: &[&str]) -> Vec<syn::Attribute> {
    lines
        .iter()
        .map(|line| syn::parse_quote!(#[doc = #line]))
        .collect()
}

#[test]
fn prose_promoted_to_description_never_title_or_details() {
    // The core never derives @title (structural name, set by the caller) nor @details
    // (folded into @description) from prose.
    let attrs = make_doc_attrs_plain(&[
        "Title.",
        "",
        "Description.",
        "",
        "What used to be details.",
        "@param x A value",
    ]);
    let tags = roxygen_tags_from_attrs(&attrs);
    assert!(
        !tags.iter().any(|t| t.starts_with("@title")),
        "no @title from prose: {:?}",
        tags
    );
    assert!(
        !tags.iter().any(|t| t.starts_with("@details")),
        "no @details from prose: {:?}",
        tags
    );
    let desc = tags.iter().find(|t| t.starts_with("@description")).unwrap();
    // All three leading paragraphs fold into the single @description.
    assert!(
        desc.contains("Title.")
            && desc.contains("Description.")
            && desc.contains("What used to be details."),
        "all leading paragraphs must fold into @description: {:?}",
        tags
    );
    // @description precedes @param.
    assert!(
        tags.iter().position(|t| t.starts_with("@description"))
            < tags.iter().position(|t| t.starts_with("@param")),
        "@description must precede @param: {:?}",
        tags
    );
}

#[test]
fn explicit_description_not_clobbered_by_prose() {
    // An author-written @description suppresses prose promotion entirely.
    let attrs = make_doc_attrs_plain(&[
        "Leading prose that would otherwise become the description.",
        "@description Explicit description.",
        "@param x A value",
    ]);
    let tags = roxygen_tags_from_attrs(&attrs);
    let count = tags
        .iter()
        .filter(|t| t.starts_with("@description"))
        .count();
    assert_eq!(count, 1, "expected exactly one @description: {:?}", tags);
    let desc = tags.iter().find(|t| t.starts_with("@description")).unwrap();
    assert!(
        desc.contains("Explicit description.") && !desc.contains("Leading prose"),
        "explicit @description must win: {:?}",
        tags
    );
}

#[test]
fn tag_led_block_gets_no_description_or_title() {
    // An @inherit-led block (no leading prose) must NOT gain a spurious description
    // or title; the tag is preserved.
    let attrs = make_doc_attrs_plain(&["@inherit foo"]);
    let tags = roxygen_tags_from_attrs(&attrs);
    assert!(
        !tags
            .iter()
            .any(|t| t.starts_with("@title") || t.starts_with("@description")),
        "tag-led block must not gain @title/@description: {:?}",
        tags
    );
    assert!(
        tags.iter().any(|t| t.starts_with("@inherit")),
        "the @inherit tag must be preserved: {:?}",
        tags
    );
}

// endregion

// region: rustdoc intra-doc links neutralized for roxygen2 (#1054 follow-up)
//
// rpkg's DESCRIPTION enables `Config/roxygen2/markdown: TRUE`, so roxygen2 reads
// `[Foo]` as a `\link{}` to an R topic. rustdoc summaries use the same syntax for
// *Rust* items, which roxygen2 can't resolve. `sanitize_roxygen_links` strips the
// link brackets (keeping `` `code` `` spans) before prose becomes `@description`.

#[test]
fn sanitize_strips_shortcut_and_reference_links() {
    // Shortcut `[`Foo`]` → keep the code span; reference `[x][crate::y]` → keep `x`.
    assert_eq!(
        sanitize_roxygen_links("same column shape as [`REMapB`]"),
        "same column shape as `REMapB`"
    );
    assert_eq!(
        sanitize_roxygen_links("see [`AsSerialize`][serde::AsSerialize] wrapper"),
        "see `AsSerialize` wrapper"
    );
    assert_eq!(
        sanitize_roxygen_links("a plain [Topic] link"),
        "a plain Topic link"
    );
}

#[test]
fn sanitize_keeps_real_markdown_links() {
    // `[text](url)` is valid roxygen2 markdown — leave it alone.
    let s = "see [the docs](https://example.com) for more";
    assert_eq!(sanitize_roxygen_links(s), s);
}

#[test]
fn sanitize_leaves_brackets_inside_code_spans() {
    // Markdown (and roxygen2) never parse `[...]` inside backticks as a
    // link — stripping there would corrupt code like `x[i]`.
    let s = "index with `x[i]` or `m[, 1]` as usual";
    assert_eq!(sanitize_roxygen_links(s), s);
    // A rustdoc link *around* a code span is still stripped.
    assert_eq!(
        sanitize_roxygen_links("see [`Vec<T>`] and `v[0]`"),
        "see `Vec<T>` and `v[0]`"
    );
}

#[test]
fn sanitize_is_utf8_safe() {
    // Multi-byte chars around a link must not panic or corrupt.
    assert_eq!(
        sanitize_roxygen_links("café [`Foo`] — déjà"),
        "café `Foo` — déjà"
    );
}

#[test]
fn sanitize_preserves_escaped_brackets() {
    // `\[u8\]` is idiomatic rustdoc for a literal bracket (suppresses
    // intra-doc link resolution). Eating the brackets but not the
    // backslashes produced invalid Rd macros like `Box<\u8\>` in
    // altrep_vec.Rd. Pass the escape through; roxygen2 markdown unescapes.
    let s = r"Create a Box<\[u8\]> ALTREP raw vector.";
    assert_eq!(sanitize_roxygen_links(s), s);
    // An unescaped shortcut link in the same string is still stripped.
    assert_eq!(
        sanitize_roxygen_links(r"see [Foo] and Box<\[u8\]>"),
        r"see Foo and Box<\[u8\]>"
    );
}

#[test]
fn leading_prose_promotes_and_sanitizes() {
    let attrs = make_doc_attrs_plain(&[
        "`HashMap` field — same column shape as [`REMapB`].",
        "",
        "Second paragraph references [`S7PropOuter`].",
    ]);
    let desc = leading_prose_from_attrs(&attrs).expect("expected leading prose");
    assert!(
        desc.contains("`REMapB`") && !desc.contains("[`REMapB`]"),
        "links must be neutralized: {desc:?}"
    );
    assert!(
        desc.contains("`S7PropOuter`") && !desc.contains("[`S7PropOuter`]"),
        "second-paragraph links must be neutralized: {desc:?}"
    );
    // Paragraph boundary preserved as a blank-line separator.
    assert!(
        desc.contains("\n\n"),
        "paragraph break must be preserved: {desc:?}"
    );
}

#[test]
fn leading_prose_none_for_tag_led_block() {
    let attrs = make_doc_attrs_plain(&["@param x A value"]);
    assert_eq!(leading_prose_from_attrs(&attrs), None);
}

// endregion

// region: Tag extraction tests

#[test]
fn test_format_single_line_tags() {
    let tags = vec![
        "@param x An input".to_string(),
        "@return Output".to_string(),
    ];
    let formatted = format_roxygen_tags(&tags);
    assert_eq!(formatted, "#' @param x An input\n#' @return Output\n");
}

#[test]
fn test_format_multiline_tag() {
    // Simulates: @description First line\nSecond line
    let tags = vec!["@description First line\nSecond line".to_string()];
    let formatted = format_roxygen_tags(&tags);
    assert_eq!(formatted, "#' @description First line\n#' Second line\n");
}

#[test]
fn test_push_multiline_tag() {
    let tags = vec!["@description Line one\nLine two\nLine three".to_string()];
    let mut lines = Vec::new();
    push_roxygen_tags(&mut lines, &tags);
    assert_eq!(
        lines,
        vec!["#' @description Line one", "#' Line two", "#' Line three"]
    );
}

#[test]
fn test_has_roxygen_tag_multiline() {
    // Tag name detection should work even with multiline content
    let tags = vec!["@description First\nSecond".to_string()];
    assert!(has_roxygen_tag(&tags, "description"));
    assert!(!has_roxygen_tag(&tags, "param"));
}
// endregion

// region: has_roxygen_tag: single-word and multi-word matching

#[test]
fn test_has_roxygen_tag_single_word() {
    let tags = vec!["@export".to_string(), "@noRd".to_string()];
    assert!(has_roxygen_tag(&tags, "export"));
    assert!(has_roxygen_tag(&tags, "noRd"));
    assert!(!has_roxygen_tag(&tags, "param"));
}

#[test]
fn test_has_roxygen_tag_keywords_internal() {
    let tags = vec!["@keywords internal".to_string()];
    assert!(has_roxygen_tag(&tags, "keywords internal"));
    // Single-word "keywords" should also match
    assert!(has_roxygen_tag(&tags, "keywords"));
    assert!(!has_roxygen_tag(&tags, "internal"));
}

#[test]
fn test_has_roxygen_tag_keywords_internal_with_whitespace() {
    // Extra whitespace around the tag content
    let tags = vec!["  @keywords internal  ".to_string()];
    assert!(has_roxygen_tag(&tags, "keywords internal"));
}

#[test]
fn test_has_roxygen_tag_keywords_other() {
    // @keywords with a different value should not match "keywords internal"
    let tags = vec!["@keywords datasets".to_string()];
    assert!(has_roxygen_tag(&tags, "keywords"));
    assert!(!has_roxygen_tag(&tags, "keywords internal"));
}

#[test]
fn test_has_roxygen_tag_param_with_name() {
    // "param x" is a multi-word search that should match "@param x"
    let tags = vec!["@param x An input".to_string()];
    assert!(has_roxygen_tag(&tags, "param"));
    // Multi-word match: "param x An input" won't match "param x" because
    // the full content after @ is "param x An input", not "param x"
    assert!(!has_roxygen_tag(&tags, "param x"));
}
// endregion

// region: tag_names: extraction tests

#[test]
fn test_tag_names_extracts_first_word() {
    let tags = vec![
        "@param x Input".to_string(),
        "@return Output".to_string(),
        "@export".to_string(),
    ];
    let names = tag_names(&tags);
    assert!(names.contains("param"));
    assert!(names.contains("return"));
    assert!(names.contains("export"));
    assert!(!names.contains("x"));
}

#[test]
fn test_tag_names_ignores_non_tag_lines() {
    let tags = vec!["Just a comment".to_string(), "@title Real tag".to_string()];
    let names = tag_names(&tags);
    assert!(names.contains("title"));
    assert_eq!(names.len(), 1);
}

#[test]
fn test_tag_names_handles_leading_whitespace() {
    let tags = vec!["  @export".to_string()];
    let names = tag_names(&tags);
    assert!(names.contains("export"));
}

#[test]
fn test_find_tag_value() {
    let tags = vec![
        "@title My Title".to_string(),
        "@description A longer description".to_string(),
        "@param x An input".to_string(),
    ];
    assert_eq!(find_tag_value(&tags, "title"), Some("My Title"));
    assert_eq!(
        find_tag_value(&tags, "description"),
        Some("A longer description")
    );
    assert_eq!(find_tag_value(&tags, "param"), Some("x An input"));
    assert_eq!(find_tag_value(&tags, "return"), None);
}

// region: strip_method_tags — impl-block roxygen tag filtering

#[test]
fn strip_method_tags_drops_param_return_examples_export() {
    let tags: Vec<String> = [
        "@param x an input",
        "@return something",
        "@returns another thing",
        "@examples f()",
        "@export",
        "@title Keep me",
        "@name keep_me",
        "@description Keep this too",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect();

    let span = proc_macro2::Span::call_site();
    let (kept, _warnings) = strip_method_tags(&tags, "MyType", span);
    assert_eq!(
        kept,
        vec![
            "@title Keep me".to_string(),
            "@name keep_me".to_string(),
            "@description Keep this too".to_string(),
        ]
    );
}

#[test]
fn strip_method_tags_preserves_export_variants() {
    // @exportClass / @exportMethod / @exportPattern are valid on class-level
    // docs (S4). Only the bare @export tag should be stripped.
    let tags: Vec<String> = [
        "@export",
        "@exportClass MyS4Class",
        "@exportMethod show",
        "@exportPattern ^foo",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect();

    let (kept, _warnings) = strip_method_tags(&tags, "MyType", proc_macro2::Span::call_site());
    assert_eq!(
        kept,
        vec![
            "@exportClass MyS4Class".to_string(),
            "@exportMethod show".to_string(),
            "@exportPattern ^foo".to_string(),
        ]
    );
}

#[test]
fn strip_method_tags_leaves_prose_untouched() {
    let tags: Vec<String> = ["This is prose.", "@title A title", "More prose"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    let (kept, _warnings) = strip_method_tags(&tags, "MyType", proc_macro2::Span::call_site());
    assert_eq!(kept, tags);
}

#[test]
fn strip_method_tags_wraps_each_warning_in_its_own_anonymous_const_scope() {
    // #1118: two impl blocks on the same type each strip a method-only tag.
    // Previously this needed a per-block `block_id` disambiguator baked into
    // the const names, or the two emissions collided with E0428 once spliced
    // side by side. Wrapping each warning in its own `const _: () = { .. };`
    // block gives every emission a fresh, anonymous item scope instead — so
    // two calls produce textually *identical* tokens and still never collide.
    let tags: Vec<String> = vec!["@param x an input".to_string()];
    let span = proc_macro2::Span::call_site();

    let (_, warnings_a) = strip_method_tags(&tags, "MyType", span);
    let (_, warnings_b) = strip_method_tags(&tags, "MyType", span);

    let sa = warnings_a.to_string();
    let sb = warnings_b.to_string();

    assert_eq!(
        sa, sb,
        "no per-block disambiguator needed — anonymous scoping does the work"
    );
    assert!(
        sa.contains("const _"),
        "warning must be wrapped in an anonymous const scope: {sa}"
    );
    assert!(
        sa.contains("_MINIEXTENDR_IMPL_METHOD_TAG_WARN"),
        "WARN const missing: {sa}"
    );
}

#[test]
fn strip_method_tags_activates_use_const_referencing_warn_ident() {
    // #1206: an unused `#[deprecated]` const warns nowhere — it's dead code
    // implying a feature. A sibling USE const whose initializer reads the
    // WARN const's value makes rustc's `deprecated` lint actually fire at the
    // impl-block span, turning the silent no-op into a real compile warning
    // that points the user at the misplaced tag.
    let tags: Vec<String> = vec!["@param x an input".to_string()];
    let span = proc_macro2::Span::call_site();

    let (_, warnings) = strip_method_tags(&tags, "MyType", span);
    let s = warnings.to_string();

    assert!(
        s.contains("_MINIEXTENDR_IMPL_METHOD_TAG_WARN"),
        "WARN const missing: {s}"
    );
    assert!(
        s.contains("_MINIEXTENDR_IMPL_METHOD_TAG_USE"),
        "USE const missing: {s}"
    );
    // The USE const's initializer must read the WARN const by name, so
    // referencing it trips rustc's `deprecated` lint on the WARN const.
    assert!(
        s.contains(
            "const _MINIEXTENDR_IMPL_METHOD_TAG_USE : () = _MINIEXTENDR_IMPL_METHOD_TAG_WARN"
        ),
        "USE const must initialize from the WARN const's value: {s}"
    );
}

#[test]
fn strip_method_tags_r6_activates_use_const_for_stripped_tags() {
    // Same activation for the R6 variant, exercised on a tag R6 still strips
    // (`@return` — unlike `@param`, which R6 intentionally keeps).
    let tags: Vec<String> = vec!["@return something".to_string()];
    let span = proc_macro2::Span::call_site();

    let (kept, warnings) = strip_method_tags_r6(&tags, "MyR6Type", span);
    assert!(kept.is_empty(), "@return must still be stripped for R6");

    let s = warnings.to_string();
    assert!(
        s.contains("_MINIEXTENDR_IMPL_METHOD_TAG_WARN"),
        "WARN const missing: {s}"
    );
    assert!(
        s.contains("_MINIEXTENDR_IMPL_METHOD_TAG_USE"),
        "USE const missing: {s}"
    );
    assert!(
        s.contains(
            "const _MINIEXTENDR_IMPL_METHOD_TAG_USE : () = _MINIEXTENDR_IMPL_METHOD_TAG_WARN"
        ),
        "USE const must initialize from the WARN const's value: {s}"
    );
}

// endregion

#[test]
fn split_r_formals_ignores_nested_commas() {
    // Plain formals split as before.
    assert_eq!(split_r_formals("x, y, z"), vec!["x", "y", "z"]);
    // A match_arg default `c("fast", "slow")` must stay one formal, not be
    // shredded into `mode = c("fast"` + `"slow")` (the ScalerS7/ScalerR6 bug).
    assert_eq!(
        split_r_formals(r#"x, mode = c("fast", "slow"), ..."#),
        vec!["x", r#"mode = c("fast", "slow")"#, "..."]
    );
    // Nested calls / brackets are respected too.
    assert_eq!(
        split_r_formals("self, opts = list(a = 1, b = 2), ..."),
        vec!["self", "opts = list(a = 1, b = 2)", "..."]
    );
    assert!(split_r_formals("").is_empty());
}

#[test]
fn formal_name_strips_default() {
    assert_eq!(formal_name("x"), "x");
    assert_eq!(formal_name(r#"mode = c("fast", "slow")"#), "mode");
    assert_eq!(formal_name("..."), "...");
}

#[test]
fn test_normalize_for_comparison() {
    // Basic normalization
    assert_eq!(normalize_for_comparison("Hello World"), "hello world");
    // Collapse whitespace
    assert_eq!(normalize_for_comparison("Hello    World"), "hello world");
    // Strip trailing punctuation
    assert_eq!(normalize_for_comparison("Hello World."), "hello world");
    assert_eq!(normalize_for_comparison("Hello World!"), "hello world");
    // Combined
    assert_eq!(
        normalize_for_comparison("  Hello    World.  "),
        "hello world"
    );
}
// endregion

// region: comma-list @param dedup (duplicated \argument Rd entries, #1261 item 2)

#[test]
fn extract_param_names_splits_comma_list() {
    // A single `@param a,b,c desc` tag documents three names, not one.
    let tags = vec!["@param a,b,c Numeric scalars.".to_string()];
    let names = extract_param_names(&tags);
    assert_eq!(names.len(), 3);
    assert!(names.contains("a"));
    assert!(names.contains("b"));
    assert!(names.contains("c"));
}

#[test]
fn extract_param_names_single_name_unaffected() {
    let tags = vec!["@param x Input value.".to_string()];
    let names = extract_param_names(&tags);
    assert_eq!(names.len(), 1);
    assert!(names.contains("x"));
}

#[test]
fn param_documented_true_for_every_name_in_comma_list() {
    let tags = vec!["@param a,b,c Numeric scalars.".to_string()];
    assert!(param_documented(&tags, "a"));
    assert!(param_documented(&tags, "b"));
    assert!(param_documented(&tags, "c"));
}

#[test]
fn param_documented_false_for_undocumented_name() {
    let tags = vec!["@param a,b,c Numeric scalars.".to_string()];
    assert!(!param_documented(&tags, "d"));
}

#[test]
fn param_documented_no_prefix_false_positive() {
    // The old `starts_with(&format!("@param {name}"))` check would wrongly
    // treat "@param x2 desc" as documenting "x" too, since "@param x2 desc"
    // starts with "@param x". Exact comma-split membership must not repeat
    // that false positive.
    let tags = vec!["@param x2 Second input.".to_string()];
    assert!(!param_documented(&tags, "x"));
    assert!(param_documented(&tags, "x2"));
}

#[test]
fn find_param_tag_returns_the_comma_list_tag_for_any_covered_name() {
    let tags = vec!["@param a,b,c Numeric scalars.".to_string()];
    assert_eq!(
        find_param_tag(&tags, "b"),
        Some(&"@param a,b,c Numeric scalars.".to_string())
    );
    assert_eq!(find_param_tag(&tags, "d"), None);
}
// endregion
