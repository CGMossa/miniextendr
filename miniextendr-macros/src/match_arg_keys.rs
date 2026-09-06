//! Write-time placeholder & symbol-name formatting for the match_arg pipeline.
//!
//! Tradeoff signpost: this module is the **wiring** between the
//! `#[derive(MatchArg)]` proc macro (see `match_arg_derive`) and the
//! cdylib's write-time substitution pass. The `match.arg` codegen path
//! deliberately splits responsibilities — the proc macro emits placeholders
//! into the R wrapper, and the cdylib resolves them to a concrete `c("a",
//! "b", ...)` literal at write time using `MX_MATCH_ARG_CHOICES`. That
//! split lets the variant list change without re-running `cargo expand` on
//! every consumer. Compared to hand-rolling `match.arg` in a wrapper body,
//! the user gets partial matching, R-visible defaults, and `@param` doc
//! lines for free, without ever stringifying the variant list in source.
//!
//! All four shapes (`choices_placeholder`, `param_doc_placeholder`,
//! `choices_helper_c_name`, `choices_helper_def_ident`) share the same
//! `{c_ident_without_prefix}_{r_param}` stem so the cdylib's write-time pass
//! can correlate them. Keep them together so the shape can't drift.
//!
//! The `c_ident_without_prefix` input has `C_` already stripped (or is a stem
//! already, e.g. `MyType__method`). Callers that have a full `c_ident` should
//! pass `c_ident.trim_start_matches("C_")`.
//!
//! All four helpers call `c_stem(...)` internally so callers may also pass a
//! full `c_ident` (e.g. `"C_my_fn"`) — the `C_` prefix is normalized away.

fn c_stem(c_ident: &str) -> &str {
    c_ident.trim_start_matches("C_")
}

/// R-side placeholder that the cdylib resolves to a `c("a", "b", ...)` literal
/// at write time. Substituted by `MX_MATCH_ARG_CHOICES` entries.
pub(crate) fn choices_placeholder(c_ident: &str, r_param: &str) -> String {
    format!(".__MX_MATCH_ARG_CHOICES_{}_{}__", c_stem(c_ident), r_param)
}

/// R-side placeholder for the `@param` doc line, substituted by
/// `MX_MATCH_ARG_PARAM_DOCS` entries at write time. See #210.
pub(crate) fn param_doc_placeholder(c_ident: &str, r_param: &str) -> String {
    format!(
        ".__MX_MATCH_ARG_PARAM_DOC_{}_{}__",
        c_stem(c_ident),
        r_param
    )
}

/// C symbol name for the helper fn that returns the enum's choices SEXP,
/// called from the R wrapper's match.arg prelude.
pub(crate) fn choices_helper_c_name(c_ident: &str, r_param: &str) -> String {
    format!("C_{}__match_arg_choices__{}", c_stem(c_ident), r_param)
}

/// Rust ident holding the `R_CallMethodDef` for the match_arg choices helper.
pub(crate) fn choices_helper_def_ident(c_ident: &str, r_param: &str) -> syn::Ident {
    quote::format_ident!(
        "call_method_def_{}",
        choices_helper_c_name(c_ident, r_param)
    )
}

/// Extract the unquoted form of a user-supplied `default = "..."` literal
/// for a `match_arg` parameter.
///
/// The user writes the value as it appears in R source, so a string default
/// is `default = "\"zstd\""` — the `String` we receive is `"zstd"` (with the
/// quote chars). Strip the outer quotes if present; otherwise pass the raw
/// value through unchanged. The write-time pass validates the result against
/// the enum's `CHOICES` and panics on miss, so a malformed literal (e.g.
/// `default = "1L"`) still surfaces as a clear runtime error at cdylib load.
pub(crate) fn extract_match_arg_default(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(raw)
        .to_string()
}

/// Derive a safe Rust ident from a write-time placeholder string.
///
/// Strips surrounding underscores and turns every `.` into `_`, so a placeholder
/// like `.__MX_MATCH_ARG_CHOICES_foo_bar__` becomes an ident suffix that quotes
/// cleanly in emitted code.
pub(crate) fn placeholder_ident_suffix(placeholder: &str) -> String {
    placeholder.trim_matches('_').replace('.', "_")
}

/// Emit the `MX_MATCH_ARG_CHOICES` static + its linkme registration.
///
/// Factored so lib.rs (standalone fns) and miniextendr_impl.rs (impl methods)
/// can't drift apart — both previously open-coded the same quote! block.
///
/// `preferred_default` is the unquoted form of the user's `default = "..."`
/// (e.g. `"zstd"`). Pass `""` when the user supplied no default — the write
/// pass then keeps the natural enum order.
pub(crate) fn choices_entry_tokens(
    cfg_attrs: &[syn::Attribute],
    entry_ident: &syn::Ident,
    placeholder: &str,
    choices_ty: &syn::Type,
    preferred_default: &str,
) -> proc_macro2::TokenStream {
    quote::quote! {
        #(#cfg_attrs)*
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_MATCH_ARG_CHOICES), linkme(crate = ::miniextendr_api::linkme))]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        static #entry_ident: ::miniextendr_api::registry::MatchArgChoicesEntry =
            ::miniextendr_api::registry::MatchArgChoicesEntry {
                placeholder: #placeholder,
                choices_str: || {
                    <#choices_ty as ::miniextendr_api::match_arg::MatchArg>::CHOICES
                        .iter()
                        .map(|c| format!(
                            "\"{}\"",
                            ::miniextendr_api::match_arg::escape_r_string(c)
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                },
                preferred_default: #preferred_default,
            };
    }
}

/// Emit the `MX_MATCH_ARG_PARAM_DOCS` static + its linkme registration.
///
/// `optional` marks an `Option<T>`-typed scalar param (#1473); the write pass
/// appends ", or NULL for no choice" to the rendered `@param` line.
pub(crate) fn param_doc_entry_tokens(
    cfg_attrs: &[syn::Attribute],
    entry_ident: &syn::Ident,
    placeholder: &str,
    several_ok: bool,
    optional: bool,
    choices_ty: &syn::Type,
) -> proc_macro2::TokenStream {
    quote::quote! {
        #(#cfg_attrs)*
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_MATCH_ARG_PARAM_DOCS), linkme(crate = ::miniextendr_api::linkme))]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        static #entry_ident: ::miniextendr_api::registry::MatchArgParamDocEntry =
            ::miniextendr_api::registry::MatchArgParamDocEntry {
                placeholder: #placeholder,
                several_ok: #several_ok,
                optional: #optional,
                choices_str: || {
                    <#choices_ty as ::miniextendr_api::match_arg::MatchArg>::CHOICES
                        .iter()
                        .map(|c| format!("\"{}\"", c))
                        .collect::<Vec<_>>()
                        .join(", ")
                },
            };
    }
}
