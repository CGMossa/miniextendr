//! Roxygen tag extraction and processing for R wrapper generation.
//!
//! This module extracts roxygen2-style tags (e.g., `@param`, `@examples`) from Rust
//! doc comments and propagates them to generated R wrapper code.
//!
//! # Usage
//!
//! In Rust doc comments, use roxygen2 tags:
//!
//! ```rust,ignore
//! /// @param x A numeric input.
//! /// @return The squared value.
//! /// @examples
//! /// square(4)
//! #[miniextendr]
//! pub fn square(x: f64) -> f64 { x * x }
//! ```
//!
//! # R Package Configuration
//!
//! For roxygen2 to process multiline tags correctly, add this to your `DESCRIPTION` file:
//!
//! ```text
//! Roxygen: list(markdown = TRUE)
//! ```

use std::collections::HashSet;

/// Tags that allow multi-line content (continuation lines appended).
/// All other tags are treated as single-line.
const MULTILINE_TAGS: &[&str] = &[
    "examples",
    "description",
    "details",
    "return",
    "returns",
    "param",
    "note",
    "seealso",
    "section",
    "format",
    "references",
    "slot",
    "field",
    "value", // synonym for return
    "prop",  // S7 property documentation (roxygen2 8.0.0+)
];

/// Check if a tag name supports multi-line content.
fn is_multiline_tag(tag: &str) -> bool {
    // Extract the tag name from "@tagname ..." or "@tagname"
    let tag_name = tag
        .strip_prefix('@')
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or("");
    MULTILINE_TAGS.contains(&tag_name)
}

/// Extract roxygen tag lines (starting with '@') from Rust doc attributes.
///
/// Most tags capture only a single line. Multi-line tags like `@examples`,
/// `@description`, `@param`, and `@return` append continuation lines.
///
/// For R6 methods, if no explicit tags are found, the first doc comment paragraph
/// is auto-converted to `@description`.
pub(crate) fn roxygen_tags_from_attrs(attrs: &[syn::Attribute]) -> Vec<String> {
    roxygen_tags_from_attrs_impl(attrs)
}

/// Extract roxygen tags for an impl-block method.
///
/// Identical to [`roxygen_tags_from_attrs`] — leading prose is promoted to
/// `@description` in every context. Kept as a named alias so the class-system
/// generators read clearly at the call site.
pub(crate) fn roxygen_tags_from_attrs_for_r6_method(attrs: &[syn::Attribute]) -> Vec<String> {
    roxygen_tags_from_attrs_impl(attrs)
}

/// Core implementation of roxygen tag extraction from `#[doc = "..."]` attributes.
///
/// Walks through doc attributes line by line. Lines starting with `@` begin a new tag.
/// Continuation lines are appended only if the current tag is multiline-capable.
///
/// Before processing, the attribute slice is partitioned into doc and non-doc groups
/// (stable order within each group). All doc attributes are processed first, so that
/// interleaved `#[cfg(...)]` or other non-doc attributes never break multiline-tag
/// continuation. This is a pure parse-side transform — the emitted `TokenStream` is
/// unaffected; only the roxygen text assembly sees the normalised order.
///
/// Leading prose (paragraphs before the first `@tag`) is promoted to a
/// `@description` tag — never `@title`. The `@title` is left to the caller
/// (structural name); see [`leading_prose_from_attrs`] for why.
fn roxygen_tags_from_attrs_impl(attrs: &[syn::Attribute]) -> Vec<String> {
    let mut tags = explicit_roxygen_tags_from_attrs(attrs);

    // Check which tags are present
    let tag_names_set = tag_names(&tags);
    let has_description = tag_names_set.contains("description");

    // Promote leading prose doc comments to `@description` (all paragraphs before
    // the first `@tag`), unless the author already wrote an explicit `@description`.
    //
    // The page `@title` is NOT synthesized from prose. Rustdoc summaries are markdown
    // written for `cargo doc` — intra-doc links (`[`Foo`]`, `[x][crate::y]`) and code
    // spans — which roxygen2's markdown parser tries to resolve as R `\link{}` topics
    // and fails ("could not resolve link to topic" / "refers to un-installed package").
    // Titles come from the structural name instead: the wrapper name for standalone
    // functions (see `lib.rs`) and `@title {Name} Class` for class blocks (see
    // `ClassDocBuilder`). Demoting prose to `@description` keeps it visible while the
    // title stays link-free.
    //
    // `leading_prose_from_attrs` returns `None` for tag-led blocks (no leading prose),
    // so `@inherit`/`@rdname`-only docs never gain a spurious description.
    if !has_description && let Some(desc) = leading_prose_from_attrs(attrs) {
        tags.insert(0, format!("@description {}", desc));
    }

    tags
}

/// Parse only the author-written `@tag` lines from doc attributes — no
/// leading-prose promotion.
///
/// [`roxygen_tags_from_attrs_impl`] layers the leading-prose → `@description`
/// promotion on top of this. [`doc_conflict_warnings`] must use this raw parse
/// instead: comparing the *synthesized* description (all leading prose) against
/// the implicit one (second paragraph) warned on every multi-paragraph doc
/// comment that had no explicit `@description` at all (#1172).
fn explicit_roxygen_tags_from_attrs(attrs: &[syn::Attribute]) -> Vec<String> {
    let mut tags = Vec::new();

    // Partition: doc attrs first (stable), non-doc attrs after.
    // This means interleaved #[cfg(...)] and similar never interrupt doc processing.
    let doc_attrs: Vec<&syn::Attribute> =
        attrs.iter().filter(|a| a.path().is_ident("doc")).collect();

    for attr in doc_attrs {
        let syn::Meta::NameValue(nv) = &attr.meta else {
            continue;
        };
        let syn::Expr::Lit(expr_lit) = &nv.value else {
            continue;
        };
        let syn::Lit::Str(lit) = &expr_lit.lit else {
            continue;
        };
        for line in lit.value().lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('@') {
                tags.push(trimmed.to_string());
            } else if !trimmed.is_empty()
                && let Some(last) = tags.last_mut()
                && is_multiline_tag(last)
            {
                // Continuation line for the current multi-line tag.
                last.push('\n');
                last.push_str(trimmed);
            }
            // Leading prose (before any @tag) is captured separately by
            // `leading_prose_from_attrs` and promoted to @description in
            // `roxygen_tags_from_attrs_impl`.
        }
    }

    tags
}

/// Render roxygen tag lines as "#' ..." comment lines.
///
/// Multiline tags (containing '\n') are split into separate `#'` lines.
pub(crate) fn format_roxygen_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for tag in tags {
        for line in tag.lines() {
            out.push_str("#' ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Push roxygen tag lines into a vector of R wrapper lines.
///
/// Multiline tags (containing '\n') are split into separate `#'` lines.
pub(crate) fn push_roxygen_tags(lines: &mut Vec<String>, tags: &[String]) {
    for tag in tags {
        for line in tag.lines() {
            lines.push(format!("#' {}", line));
        }
    }
}

/// Like [`push_roxygen_tags`] but takes `&[&str]` for filtered tag slices.
pub(crate) fn push_roxygen_tags_str(lines: &mut Vec<String>, tags: &[&str]) {
    for tag in tags {
        for line in tag.lines() {
            lines.push(format!("#' {}", line));
        }
    }
}

/// Return true if the tag list contains a specific roxygen tag.
///
/// Supports both single-word tags (e.g., `"export"`, `"noRd"`) and
/// multi-word tags (e.g., `"keywords internal"`). For single-word tags,
/// matches the first word after `@`. For multi-word tags, matches the
/// full content after `@` (trimmed).
pub(crate) fn has_roxygen_tag(tags: &[String], tag: &str) -> bool {
    if tag.contains(' ') {
        // Multi-word tag: match the full content after @
        tags.iter().any(|t| {
            t.trim_start()
                .strip_prefix('@')
                .is_some_and(|rest| rest.trim() == tag)
        })
    } else {
        tag_names(tags).contains(tag)
    }
}

/// The topic named by the tag list's own `@rdname` tag, if it has one.
///
/// Class generators default every method onto the class page (`@rdname
/// <Class>`); a method-level `/// @rdname other` splits it onto its own page.
/// Every emission path that injects the class default must consult this first
/// so a user-supplied topic is honoured once, not duplicated by the default.
pub(crate) fn rdname_value(tags: &[String]) -> Option<&str> {
    tags.iter().find_map(|t| {
        let rest = t.trim_start().strip_prefix("@rdname")?;
        let value = rest.trim();
        (rest.starts_with(char::is_whitespace) && !value.is_empty()).then_some(value)
    })
}

/// Push `#' @rdname <topic>`: the method's own `@rdname` when present, else
/// `default` (the class page).
pub(crate) fn push_rdname_or_default(lines: &mut Vec<String>, tags: &[String], default: &str) {
    let topic = rdname_value(tags).unwrap_or(default);
    lines.push(format!("#' @rdname {topic}"));
}

/// Build a roxygen `@source` traceability line for a class method.
///
/// Returns `"#' @source Generated by miniextendr from \`Type::method\`"`.
/// Use this wherever a class-generator needs to emit a source-provenance
/// comment linking the generated R wrapper back to the originating Rust
/// `impl` block method.
pub(crate) fn method_source_tag(type_ident: &syn::Ident, method_ident: &syn::Ident) -> String {
    format!(
        "#' @source Generated by miniextendr from `{}::{}`",
        type_ident, method_ident
    )
}

/// Build a roxygen `@source` traceability line for a class definition.
///
/// Returns ``"#' @source Generated by miniextendr from Rust type `Type`"``.
/// Companion to [`method_source_tag`] for class-level documentation blocks.
pub(crate) fn class_source_tag(type_ident: &syn::Ident) -> String {
    format!(
        "#' @source Generated by miniextendr from Rust type `{}`",
        type_ident
    )
}

/// Extract the set of tag names from a list of roxygen tag strings.
///
/// Each tag string is expected to start with `@tagname`. Returns a set of
/// the tag names (without the `@` prefix).
fn tag_names(tags: &[String]) -> HashSet<&str> {
    let mut names = HashSet::new();
    for tag in tags {
        let trimmed = tag.trim_start();
        let name = trimmed
            .strip_prefix('@')
            .and_then(|rest| rest.split_whitespace().next());
        if let Some(name) = name {
            names.insert(name);
        }
    }
    names
}

/// Find the value of a specific roxygen tag (e.g., "title" for `@title ...`).
///
/// Returns `None` if the tag is not present or has no value.
#[cfg_attr(not(feature = "doc-lint"), allow(dead_code))]
pub(crate) fn find_tag_value<'a>(tags: &'a [String], tag_name: &str) -> Option<&'a str> {
    for tag in tags {
        let trimmed = tag.trim_start();
        if let Some(rest) = trimmed.strip_prefix('@') {
            let mut parts = rest.splitn(2, |c: char| c.is_whitespace());
            if let Some(name) = parts.next()
                && name == tag_name
            {
                // Get the value (everything after the tag name)
                return parts.next().map(|s| s.trim());
            }
        }
    }
    None
}

/// Normalize text for comparison: lowercase, collapse whitespace, strip trailing punctuation.
///
/// Used by `doc_conflict_warnings` to compare explicit `@title`/`@description` values
/// with implicit values derived from the doc comment structure. Normalization ensures
/// minor formatting differences (extra spaces, trailing periods) don't trigger false warnings.
#[cfg_attr(not(feature = "doc-lint"), allow(dead_code))]
fn normalize_for_comparison(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut result = String::new();
    for word in lower.split_whitespace() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(word);
    }
    result.truncate(
        result
            .trim_end_matches(|c: char| c.is_ascii_punctuation())
            .len(),
    );
    result
}

/// Extract the implicit title from doc attributes (first sentence, up to first `.` or newline).
///
/// Returns `None` if there are no doc comments or if docs start with a `@tag`.
#[cfg_attr(not(feature = "doc-lint"), allow(dead_code))]
pub(crate) fn implicit_title_from_attrs(attrs: &[syn::Attribute]) -> Option<String> {
    let mut lines = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        let syn::Meta::NameValue(nv) = &attr.meta else {
            continue;
        };
        let syn::Expr::Lit(expr_lit) = &nv.value else {
            continue;
        };
        let syn::Lit::Str(lit) = &expr_lit.lit else {
            continue;
        };

        let content = lit.value();
        let trimmed = content.trim();

        // If we hit a @tag before any content, there's no implicit title
        if trimmed.starts_with('@') {
            if lines.is_empty() {
                return None;
            }
            break;
        }

        // Empty line ends first sentence for title extraction
        if trimmed.is_empty() {
            break;
        }

        // Check if this line contains a sentence-ending period
        if let Some(pos) = trimmed.find(". ") {
            lines.push(trimmed[..pos].to_string());
            break;
        } else if trimmed.ends_with('.') {
            lines.push(trimmed.trim_end_matches('.').to_string());
            break;
        } else {
            lines.push(trimmed.to_string());
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" "))
    }
}

/// Extract the implicit description from doc attributes (second paragraph).
///
/// In roxygen2, the first paragraph is the title and the second paragraph is the
/// description. This function skips the first paragraph (up to the first blank line)
/// and returns the second paragraph.
///
/// Returns `None` if there is no second paragraph, no doc comments, or if docs
/// start with a `@tag`.
#[cfg_attr(not(feature = "doc-lint"), allow(dead_code))]
pub(crate) fn implicit_description_from_attrs(attrs: &[syn::Attribute]) -> Option<String> {
    let mut lines = Vec::new();
    let mut found_first_paragraph = false;
    let mut in_gap = false;

    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        let syn::Meta::NameValue(nv) = &attr.meta else {
            continue;
        };
        let syn::Expr::Lit(expr_lit) = &nv.value else {
            continue;
        };
        let syn::Lit::Str(lit) = &expr_lit.lit else {
            continue;
        };

        let content = lit.value();
        let trimmed = content.trim();

        // If we hit a @tag, stop
        if trimmed.starts_with('@') {
            break;
        }

        if !found_first_paragraph {
            // Still in the first paragraph (title)
            if trimmed.is_empty() {
                // Blank line — first paragraph ended, now in the gap
                found_first_paragraph = true;
                in_gap = true;
            }
            // Non-empty lines before first blank are title — skip
        } else if in_gap {
            // Between paragraphs — skip blank lines
            if !trimmed.is_empty() {
                // Start of second paragraph
                in_gap = false;
                lines.push(trimmed.to_string());
            }
        } else {
            // In second paragraph
            if trimmed.is_empty() {
                // End of second paragraph
                break;
            }
            lines.push(trimmed.to_string());
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" "))
    }
}

/// Collect the leading prose of a doc comment (all paragraphs before the first
/// `@tag`) as roxygen `@description` text, with rustdoc intra-doc links neutralized.
///
/// Each `///` line is one doc attribute; a blank line is an empty attribute and marks
/// a paragraph boundary. Paragraphs are joined with `"\n\n"` so `push_roxygen_tags`
/// renders blank `#'` lines between them — roxygen2 multi-paragraph description text.
///
/// Returns `None` when the block has no leading prose (empty, or starts with a `@tag`),
/// so tag-led blocks never gain a spurious `@description`.
fn leading_prose_from_attrs(attrs: &[syn::Attribute]) -> Option<String> {
    let mut paragraphs: Vec<Vec<String>> = Vec::new();
    let mut current: Vec<String> = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        let syn::Meta::NameValue(nv) = &attr.meta else {
            continue;
        };
        let syn::Expr::Lit(expr_lit) = &nv.value else {
            continue;
        };
        let syn::Lit::Str(lit) = &expr_lit.lit else {
            continue;
        };

        let content = lit.value();
        let trimmed = content.trim();

        if trimmed.starts_with('@') {
            // First tag ends the prose block.
            break;
        }
        if trimmed.is_empty() {
            // Blank line — paragraph boundary.
            if !current.is_empty() {
                paragraphs.push(std::mem::take(&mut current));
            }
        } else {
            current.push(sanitize_roxygen_links(trimmed));
        }
    }
    if !current.is_empty() {
        paragraphs.push(current);
    }

    if paragraphs.is_empty() {
        None
    } else {
        let joined: Vec<String> = paragraphs.into_iter().map(|p| p.join(" ")).collect();
        Some(joined.join("\n\n"))
    }
}

/// Neutralize rustdoc intra-doc link syntax so prose is valid roxygen2 markdown.
///
/// rustdoc `[`Foo`]` / `[Foo]` / `[text][target]` are intra-doc links resolved
/// against *Rust* items by `cargo doc`. roxygen2 (markdown on) reads the same
/// `[...]` as an R `\link{}` to a *help topic*, which can't resolve. We strip the
/// link brackets down to the visible text (keeping any `` `code` `` span), while
/// leaving genuine markdown links `[text](url)` — recognized by the `]( ` that
/// follows — untouched.
fn sanitize_roxygen_links(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    let mut in_code = false;
    while i < s.len() {
        if bytes[i] == b'`' {
            // Inline code span: markdown (and roxygen2) never parse `[...]`
            // inside backticks as a link, so neither do we — stripping there
            // would corrupt code like `x[i]`.
            in_code = !in_code;
            out.push('`');
            i += 1;
            continue;
        }
        if !in_code && bytes[i] == b'[' {
            // `\[` is a backslash-escaped literal bracket (idiomatic rustdoc
            // for suppressing intra-doc links, e.g. `Box<\[u8\]>`). Not a link
            // opener — pass it through; roxygen2's markdown unescapes it.
            if i > 0 && bytes[i - 1] == b'\\' {
                out.push('[');
                i += 1;
                continue;
            }
            // `[` is ASCII, so `i + 1` is a char boundary.
            if let Some(close_rel) = s[i + 1..].find(']') {
                let close = i + 1 + close_rel;
                let inner = &s[i + 1..close];
                match bytes.get(close + 1) {
                    // `[text](url)` — real markdown link. Emit `[` literally and let
                    // the inner text + `](url)` flow through unchanged.
                    Some(b'(') => {
                        out.push('[');
                        i += 1;
                        continue;
                    }
                    // `[text][target]` — reference link. Keep `text`, drop `[target]`.
                    Some(b'[') => {
                        out.push_str(inner);
                        if let Some(t_rel) = s[close + 2..].find(']') {
                            i = close + 2 + t_rel + 1;
                        } else {
                            i = close + 1;
                        }
                        continue;
                    }
                    // `[text]` — shortcut link. Keep `text`, drop the brackets.
                    _ => {
                        out.push_str(inner);
                        i = close + 1;
                        continue;
                    }
                }
            }
        }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Check for conflicts between explicit `@title`/`@description` tags and implicit values.
///
/// When the `doc-lint` feature is enabled, returns tokens that generate compile-time
/// deprecation warnings if explicit roxygen tags differ from the implicit values
/// derived from the doc comment structure.
///
/// The returned tokens should be appended to the macro expansion output.
#[cfg(feature = "doc-lint")]
pub(crate) fn doc_conflict_warnings(
    attrs: &[syn::Attribute],
    _span: proc_macro2::Span,
) -> proc_macro2::TokenStream {
    use quote::quote;

    // Raw parse only: `roxygen_tags_from_attrs` synthesizes a `@description`
    // from leading prose, which this lint must not mistake for an
    // author-written tag (#1172).
    let tags = explicit_roxygen_tags_from_attrs(attrs);
    let mut warnings = proc_macro2::TokenStream::new();

    // Check @title conflict
    if let Some(explicit) = find_tag_value(&tags, "title")
        && let Some(implicit) = implicit_title_from_attrs(attrs)
        && normalize_for_comparison(explicit) != normalize_for_comparison(&implicit)
    {
        let msg = format!(
            "miniextendr doc-lint: explicit @title differs from first doc line. \
             R's roxygen2 uses the first line as the title. \
             implicit: \"{}\", explicit @title: \"{}\"",
            implicit, explicit
        );
        warnings.extend(quote! {
            const _: () = {
                #[deprecated(note = #msg)]
                #[doc(hidden)]
                #[allow(dead_code)]
                const MINIEXTENDR_DOC_LINT_TITLE: () = ();
                let _ = MINIEXTENDR_DOC_LINT_TITLE;
            };
        });
    }

    // Check @description conflict
    if let Some(explicit) = find_tag_value(&tags, "description")
        && let Some(implicit) = implicit_description_from_attrs(attrs)
        && normalize_for_comparison(explicit) != normalize_for_comparison(&implicit)
    {
        let msg = format!(
            "miniextendr doc-lint: explicit @description differs from first paragraph. \
             R's roxygen2 uses the first paragraph as the description. \
             implicit: \"{}\", explicit @description: \"{}\"",
            implicit, explicit
        );
        warnings.extend(quote! {
            const _: () = {
                #[deprecated(note = #msg)]
                #[doc(hidden)]
                #[allow(dead_code)]
                const MINIEXTENDR_DOC_LINT_DESC: () = ();
                let _ = MINIEXTENDR_DOC_LINT_DESC;
            };
        });
    }

    warnings
}

/// No-op when doc-lint feature is disabled.
#[cfg(not(feature = "doc-lint"))]
pub(crate) fn doc_conflict_warnings(
    _attrs: &[syn::Attribute],
    _span: proc_macro2::Span,
) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::new()
}

/// Roxygen tags that only make sense on individual methods, not on impl blocks.
///
/// - `@param` — impl blocks have no parameters (except for R6 class-level param docs,
///   which roxygen2 8.0.0 inherits into all methods; those are exempted by
///   [`strip_method_tags_r6`]).
/// - `@return` / `@returns` — impl blocks have no return value.
/// - `@examples` — examples belong on the method that is being demonstrated.
/// - `@export` — redundant: export for class-level docs is handled by
///   `ClassDocBuilder`, which emits `@export` based on the impl block's
///   `internal` / `noexport` attrs, not on user-supplied roxygen.
const METHOD_ONLY_TAGS: &[&str] = &["param", "return", "returns", "examples", "export"];

/// Tags stripped from impl-block docs for R6 classes — same as `METHOD_ONLY_TAGS`
/// minus `"param"`, since roxygen2 8.0.0 inherits class-level `@param` tags into
/// all R6 methods and strips them from the rendered method entries automatically.
/// This means `/// @param breed …` on an R6 impl block is valid and intentional,
/// not a misplaced method-only tag. Keeping them avoids both the compile warning
/// and the resulting `(no documentation available)` placeholder on subclass ctors.
const METHOD_ONLY_TAGS_R6: &[&str] = &["return", "returns", "examples", "export"];

/// Extract the tag name from a roxygen line (everything between `@` and the
/// first whitespace character). Returns `None` for lines that don't start with
/// a tag.
pub(crate) fn roxygen_tag_name(tag: &str) -> Option<&str> {
    let rest = tag.trim_start().strip_prefix('@')?;
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    Some(&rest[..end])
}

/// Monotonic per-crate counter used to disambiguate the compile-warning const
/// names emitted by [`strip_method_tags`] / [`strip_method_tags_r6`].
///
/// The const name is keyed on the type name, but two `#[miniextendr] impl Foo`
/// blocks on the *same* type (e.g. an inherent impl plus a trait impl, or two
/// inherent impls) each restart their per-block `warning_id` at 0, so without a
/// per-block disambiguator they emit identically-named consts and collide with
/// `error[E0428]: the name ... is defined multiple times` (#1118). Every call
/// site draws a fresh block id from this counter so the names stay unique
/// within a crate compilation — the exact scope const names must be unique in,
/// since the statics reset per rustc process (one process per crate).
pub(crate) fn next_impl_tag_block_id() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static IMPL_TAG_WARN_BLOCK: AtomicUsize = AtomicUsize::new(0);
    IMPL_TAG_WARN_BLOCK.fetch_add(1, Ordering::Relaxed)
}

/// Filter out method-specific roxygen tags from impl-block-level docs and emit
/// compile warnings for each stripped tag.
///
/// Method-specific tags (`@param`, `@return`, `@returns`, `@examples`,
/// `@export`) on an impl block are meaningless — they belong on individual
/// methods, or (for `@export`) are emitted by `ClassDocBuilder`. When users
/// put them on impl blocks, the tags leak into the class-level Rd file where
/// R CMD check warns about "documented arguments not in \\usage" and similar.
///
/// `block_id` is a per-impl-block disambiguator (see [`next_impl_tag_block_id`])
/// that keeps the emitted warning-const names unique across multiple impl
/// blocks on the same type (#1118).
///
/// Returns the filtered tags and a TokenStream of deprecation warnings for
/// each stripped tag. The caller should append the warnings to its output so
/// the user sees them at compile time.
pub(crate) fn strip_method_tags(
    tags: &[String],
    type_name: &str,
    block_id: usize,
    span: proc_macro2::Span,
) -> (Vec<String>, proc_macro2::TokenStream) {
    use quote::quote_spanned;

    let mut filtered = Vec::new();
    let mut warnings = proc_macro2::TokenStream::new();
    let mut warning_id: usize = 0;

    for tag in tags {
        let Some(name) = roxygen_tag_name(tag) else {
            filtered.push(tag.clone());
            continue;
        };
        if !METHOD_ONLY_TAGS.contains(&name) {
            filtered.push(tag.clone());
            continue;
        }
        let msg = format!(
            "miniextendr: @{} on impl block `{}` has no effect — move it to the method. Tag: {}",
            name,
            type_name,
            tag.trim()
        );
        let suffix = format!(
            "{}_{}_{}",
            type_name.replace(|c: char| !c.is_alphanumeric(), "_"),
            block_id,
            warning_id
        );
        let ident = quote::format_ident!("_MINIEXTENDR_IMPL_METHOD_TAG_WARN_{}", suffix);
        let use_ident = quote::format_ident!("_MINIEXTENDR_IMPL_METHOD_TAG_USE_{}", suffix);
        warning_id += 1;
        warnings.extend(quote_spanned! { span =>
            #[deprecated(note = #msg)]
            #[doc(hidden)]
            #[allow(dead_code, non_upper_case_globals)]
            const #ident: () = ();
            #[doc(hidden)]
            #[allow(dead_code, non_upper_case_globals)]
            const #use_ident: () = #ident;
        });
    }

    (filtered, warnings)
}

/// Extract the set of parameter names declared via `@param` in a list of roxygen tags.
///
/// For each `@param <names> <desc>` tag in `tags`, extracts `<names>` and inserts
/// each comma-separated name into the returned `HashSet`. roxygen2 supports
/// documenting several params in one tag (`@param a,b,c desc` documents `a`,
/// `b`, and `c`) — the name token is split on `,` (and each piece trimmed
/// defensively, though roxygen2's own syntax has no spaces around the commas)
/// so multi-name tags don't collapse to a single name. Used by R6 class
/// generators to build the set of class-level params so method param loops
/// can suppress `(no documentation available)` for names already covered at
/// class level (roxygen2 8.0.0 inherits class-level `@param` tags into all
/// methods automatically).
pub(crate) fn extract_param_names(tags: &[String]) -> HashSet<String> {
    let mut names = HashSet::new();
    for tag in tags {
        let Some(names_token) = param_names_token(tag) else {
            continue;
        };
        for name in names_token.split(',') {
            let name = name.trim();
            if !name.is_empty() {
                names.insert(name.to_string());
            }
        }
    }
    names
}

/// Returns `true` if `name` is documented by any `@param` tag in `tags`.
///
/// A tag documents `name` if `name` is exactly one of the comma-separated
/// names in its `@param <names> <desc>` name token (see
/// [`extract_param_names`]). This is an **exact-membership** check, not a
/// prefix match — `starts_with(&format!("@param {name}"))` was the previous
/// (buggy) approach: it misses every name after the first in a comma-list
/// tag (`@param a,b,c desc` "documents" only `a`, so `b`/`c` get a spurious
/// `(no documentation available)` filler that roxygen2 then merges into a
/// duplicate `\item{b}` and `\item{c}` in the rendered Rd — an
/// `R CMD check` `checking Rd \usage sections` WARNING), and it also
/// false-positives on names that are prefixes of other names (`@param x2
/// desc` looks like it documents `x` too).
pub(crate) fn param_documented(tags: &[String], name: &str) -> bool {
    tags.iter().any(|tag| tag_documents_param(tag, name))
}

/// Like [`param_documented`] but returns the matching tag itself rather than
/// a bool, for callers that reuse the tag's rendered text verbatim (S7
/// constructor param doc forwarding pushes the found tag as-is).
pub(crate) fn find_param_tag<'a>(tags: &'a [String], name: &str) -> Option<&'a String> {
    tags.iter().find(|tag| tag_documents_param(tag, name))
}

/// Shared predicate: does this single `@param` tag document `name`?
fn tag_documents_param(tag: &str, name: &str) -> bool {
    let Some(names_token) = param_names_token(tag) else {
        return false;
    };
    names_token.split(',').any(|n| n.trim() == name)
}

/// Returns the `@param` name token (e.g. `a,b,c` in `@param a,b,c desc`) of a
/// single tag, or `None` if `tag` isn't an `@param` tag.
fn param_names_token(tag: &str) -> Option<&str> {
    let trimmed = tag.trim_start();
    let rest = trimmed.strip_prefix("@param ")?;
    rest.split_whitespace().next()
}

/// Split an R formals/argument string on **top-level** commas only.
///
/// Commas nested inside parentheses, brackets, or braces — or inside a single-
/// or double-quoted string literal — are ignored. So `x, mode = c("a", "b"), ...`
/// yields `["x", "mode = c(\"a\", \"b\")", "..."]`, whereas a naive
/// `split(", ")` wrongly breaks the `c("a", "b")` default into two bogus
/// formals (`mode = c("a"` and `"b")`) — the source of spurious `@param "b")`
/// roxygen entries on match_arg'd trait-method shortcuts (ScalerS7 / ScalerR6).
pub(crate) fn split_r_formals(formals: &str) -> Vec<&str> {
    let bytes = formals.as_bytes();
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut quote: Option<u8> = None;
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == b'\\' {
                    i += 1; // skip the escaped char
                } else if b == q {
                    quote = None;
                }
            }
            None => match b {
                b'"' | b'\'' => quote = Some(b),
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => depth -= 1,
                b',' if depth == 0 => {
                    out.push(formals[start..i].trim());
                    start = i + 1;
                }
                _ => {}
            },
        }
        i += 1;
    }
    out.push(formals[start..].trim());
    out.into_iter().filter(|s| !s.is_empty()).collect()
}

/// Extract the R parameter name from a single formal (e.g. `mode = c("a","b")`
/// → `mode`). Pair with [`split_r_formals`], never a raw `split(',')`.
pub(crate) fn formal_name(formal: &str) -> &str {
    formal.split('=').next().unwrap_or(formal).trim()
}

/// Like [`strip_method_tags`] but for R6 impl blocks.
///
/// R6 class-level `@param` tags are **kept** (roxygen2 8.0.0 inherits them into
/// all methods automatically — rd-R6.Rmd §"Class-level docs"). All other
/// method-only tags (`@return`, `@returns`, `@examples`, `@export`) are still
/// stripped with a compile-time warning. No warning is generated for `@param`.
///
/// Returns `(filtered_tags, warnings)` — same shape as [`strip_method_tags`].
/// `block_id` is the per-impl-block disambiguator (see
/// [`next_impl_tag_block_id`]) that keeps warning-const names unique (#1118).
pub(crate) fn strip_method_tags_r6(
    tags: &[String],
    type_name: &str,
    block_id: usize,
    span: proc_macro2::Span,
) -> (Vec<String>, proc_macro2::TokenStream) {
    use quote::quote_spanned;

    let mut filtered = Vec::new();
    let mut warnings = proc_macro2::TokenStream::new();
    let mut warning_id: usize = 0;

    for tag in tags {
        let Some(name) = roxygen_tag_name(tag) else {
            filtered.push(tag.clone());
            continue;
        };
        if !METHOD_ONLY_TAGS_R6.contains(&name) {
            // Keeps @param (and any unrecognised tags) without warning.
            filtered.push(tag.clone());
            continue;
        }
        let msg = format!(
            "miniextendr: @{} on impl block `{}` has no effect — move it to the method. Tag: {}",
            name,
            type_name,
            tag.trim()
        );
        let suffix = format!(
            "{}_{}_{}",
            type_name.replace(|c: char| !c.is_alphanumeric(), "_"),
            block_id,
            warning_id
        );
        let ident = quote::format_ident!("_MINIEXTENDR_IMPL_METHOD_TAG_WARN_{}", suffix);
        let use_ident = quote::format_ident!("_MINIEXTENDR_IMPL_METHOD_TAG_USE_{}", suffix);
        warning_id += 1;
        warnings.extend(quote_spanned! { span =>
            #[deprecated(note = #msg)]
            #[doc(hidden)]
            #[allow(dead_code, non_upper_case_globals)]
            const #ident: () = ();
            #[doc(hidden)]
            #[allow(dead_code, non_upper_case_globals)]
            const #use_ident: () = #ident;
        });
    }

    (filtered, warnings)
}

/// Strip roxygen tag lines from doc attributes, keeping only regular documentation.
///
/// Returns a new vector of attributes with roxygen lines removed from doc comments.
/// Non-doc attributes are passed through unchanged.
///
/// # Algorithm
///
/// Roxygen tags typically appear at the end of documentation blocks. We use a simple
/// but effective approach:
/// 1. Keep all content before the first `@tag` line
/// 2. Strip everything from the first `@tag` to the end of the roxygen region
///
/// A roxygen region ends when we see a non-empty line that doesn't start with `@`
/// and follows an empty line (paragraph break). This handles multi-paragraph tags.
pub(crate) fn strip_roxygen_from_attrs(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    // Collect doc attribute indices and their trimmed content
    let mut doc_info: Vec<(usize, String)> = Vec::new();
    for (i, attr) in attrs.iter().enumerate() {
        if !attr.path().is_ident("doc") {
            continue;
        }
        let syn::Meta::NameValue(nv) = &attr.meta else {
            continue;
        };
        let syn::Expr::Lit(expr_lit) = &nv.value else {
            continue;
        };
        let syn::Lit::Str(lit) = &expr_lit.lit else {
            continue;
        };
        // Trim the leading space that comes from `/// `
        doc_info.push((i, lit.value().trim_start().to_string()));
    }

    // Find roxygen line indices
    let mut roxygen_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut in_roxygen = false;
    let mut prev_was_empty = false;

    for (i, trimmed) in &doc_info {
        if trimmed.starts_with('@') {
            // Start or continue roxygen region
            in_roxygen = true;
            roxygen_indices.insert(*i);
            prev_was_empty = false;
        } else if in_roxygen {
            if trimmed.is_empty() {
                // Empty line in roxygen - might end the block or be part of multi-paragraph tag
                roxygen_indices.insert(*i);
                prev_was_empty = true;
            } else if prev_was_empty {
                // Non-empty line after empty line - end roxygen region
                // This is likely regular documentation
                in_roxygen = false;
                prev_was_empty = false;
            } else {
                // Continuation line (no paragraph break)
                roxygen_indices.insert(*i);
            }
        }
    }

    // Build result excluding roxygen lines
    attrs
        .iter()
        .enumerate()
        .filter(|(i, _)| !roxygen_indices.contains(i))
        .map(|(_, attr)| attr.clone())
        .collect()
}

#[cfg(test)]
mod tests;
