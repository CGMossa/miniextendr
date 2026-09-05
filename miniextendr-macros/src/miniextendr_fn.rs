//! Function signature parsing for `#[miniextendr]`.
//!
//! This module handles parsing and normalizing Rust function signatures for the
//! `#[miniextendr]` attribute macro. It provides:
//!
//! - [`MiniextendrFunctionParsed`]: Parsed function with normalization and codegen helpers
//! - [`MiniextendrFnAttrs`]: Parsed `#[miniextendr(...)]` attribute options
//! - [`CoercionMapping`]: Type coercion analysis for automatic R→Rust conversion

use crate::r_wrapper_const_ident_for;

// region: Coercion analysis

/// Result of coercion analysis for a type.
/// Contains the R native type to extract from SEXP and the target type to coerce to.
pub(crate) enum CoercionMapping {
    /// Scalar coercion: extract R native type, coerce to target.
    Scalar {
        /// The R-native scalar type to extract from the SEXP (e.g., `i32` for R integers,
        /// `f64` for R reals). This is the type that R stores internally.
        r_native: proc_macro2::TokenStream,
        /// The Rust target type to coerce into (e.g., `u16`, `bool`, `f32`).
        target: proc_macro2::TokenStream,
    },
    /// Vec coercion: extract R native slice, coerce element-wise to `Vec<target>`.
    Vec {
        /// The R-native element type of the source slice (e.g., `i32` for integer vectors,
        /// `f64` for real vectors).
        r_native_elem: proc_macro2::TokenStream,
        /// The Rust target element type for the resulting `Vec` (e.g., `u16`, `bool`, `f32`).
        target_elem: proc_macro2::TokenStream,
    },
}

impl CoercionMapping {
    /// Determines the coercion mapping for a Rust type, if it needs coercion from
    /// an R-native type.
    ///
    /// Returns `None` if the type is already R-native (`i32`, `f64`, `String`, etc.)
    /// or is not a recognized coercible type.
    ///
    /// # Recognized coercions
    ///
    /// - **Scalar integer-like** (`u16`, `i16`, `i8`, `u32`, `u64`, `i64`, `isize`, `usize`):
    ///   coerced from `i32` (R's native integer type).
    /// - **Scalar `bool`**: coerced from `i32` (R's logical vectors use `i32` internally).
    /// - **Scalar `f32`**: coerced from `f64` (R's native real type).
    /// - **`Vec<T>`** variants: element-wise coercion from the corresponding R-native slice type.
    pub(crate) fn from_type(ty: &syn::Type) -> Option<Self> {
        match ty {
            syn::Type::Path(type_path) => {
                let seg = type_path.path.segments.last()?;
                let type_name = seg.ident.to_string();

                // Check for Vec<T> types
                if type_name == "Vec" {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments
                        && let Some(syn::GenericArgument::Type(syn::Type::Path(inner_path))) =
                            args.args.first()
                    {
                        let inner_name = inner_path.path.segments.last()?.ident.to_string();
                        return match inner_name.as_str() {
                            // Vec<integer-like> from &[i32]
                            "u16" | "i16" | "i8" | "u32" | "u64" | "i64" | "isize" | "usize" => {
                                let target_elem: proc_macro2::TokenStream =
                                    inner_name.parse().ok()?;
                                Some(Self::Vec {
                                    r_native_elem: quote::quote!(i32),
                                    target_elem,
                                })
                            }
                            // Vec<bool> from &[i32] (R logical vectors use i32)
                            "bool" => Some(Self::Vec {
                                r_native_elem: quote::quote!(i32),
                                target_elem: quote::quote!(bool),
                            }),
                            // Vec<f32> from &[f64]
                            "f32" => Some(Self::Vec {
                                r_native_elem: quote::quote!(f64),
                                target_elem: quote::quote!(f32),
                            }),
                            _ => None,
                        };
                    }
                    return None;
                }

                // Check for scalar types
                match type_name.as_str() {
                    // Integer-like types from i32
                    "u16" | "i16" | "i8" | "u32" | "u64" | "i64" | "isize" | "usize" => {
                        let target: proc_macro2::TokenStream = type_name.parse().ok()?;
                        Some(Self::Scalar {
                            r_native: quote::quote!(i32),
                            target,
                        })
                    }
                    // bool from i32 (R logical vectors use i32 internally)
                    "bool" => Some(Self::Scalar {
                        r_native: quote::quote!(i32),
                        target: quote::quote!(bool),
                    }),
                    // f32 from f64
                    "f32" => Some(Self::Scalar {
                        r_native: quote::quote!(f64),
                        target: quote::quote!(f32),
                    }),
                    // R-native types or unknown - no coercion
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

// endregion

// region: Type inspection helpers

/// Check if a type path ends with the given identifier (e.g., "Dots", "Missing").
///
/// Handles fully-qualified paths like `miniextendr_api::dots::Dots` as well as
/// bare `Dots`.
fn type_ends_with(ty: &syn::Type, name: &str) -> bool {
    match ty {
        syn::Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|s| s.ident == name)
            .unwrap_or(false),
        syn::Type::Reference(r) => type_ends_with(&r.elem, name),
        _ => false,
    }
}

/// Check if a type is `Dots` or `&Dots` (the variadic `...` parameter type).
pub(crate) fn is_dots_type(ty: &syn::Type) -> bool {
    type_ends_with(ty, "Dots")
}

/// Result of normalizing Rust variadic syntax (`...`) into an explicit `&Dots`
/// parameter.
#[derive(Debug, Clone)]
pub(crate) struct VariadicDots {
    /// Whether the original signature used Rust variadic syntax.
    pub has_dots: bool,
    /// User-provided variadic identifier, e.g. `dots` in `dots: ...`.
    pub named_dots: Option<syn::Ident>,
}

/// Replace Rust variadic syntax with a trailing `&miniextendr_api::dots::Dots`
/// parameter so downstream codegen never emits a non-extern variadic Rust fn.
pub(crate) fn rewrite_variadic_dots(sig: &mut syn::Signature) -> syn::Result<VariadicDots> {
    use syn::spanned::Spanned;

    let has_dots = sig.variadic.is_some();
    let named_dots = if has_dots {
        let dots = sig.variadic.as_ref().unwrap();
        if let Some(named_dots) = dots.pat.as_ref() {
            if let syn::Pat::Ident(named_dots_ident) = named_dots.0.as_ref() {
                Some(named_dots_ident.ident.clone())
            } else {
                return Err(syn::Error::new(
                    named_dots.0.span(),
                    "variadic pattern must be a simple identifier (e.g. `dots: ...`) or unnamed `...`",
                ));
            }
        } else {
            None
        }
    } else {
        None
    };

    if has_dots {
        sig.variadic = None;
        sig.inputs
            .push(if let Some(named_dots) = named_dots.as_ref() {
                syn::parse_quote!(#named_dots: &::miniextendr_api::dots::Dots)
            } else {
                // Cannot use `_` as a variable name, so unnamed `...` needs a
                // stable synthetic binding that does not collide with user args.
                for arg in &sig.inputs {
                    let syn::FnArg::Typed(pat_type) = arg else {
                        continue;
                    };
                    if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref()
                        && pat_ident.ident == "__miniextendr_dots"
                    {
                        return Err(syn::Error::new(
                            pat_ident.ident.span(),
                            "parameter named `__miniextendr_dots` conflicts with implicit dots parameter; use named dots like `my_dots: ...` instead",
                        ));
                    }
                }
                syn::parse_quote!(__miniextendr_dots: &::miniextendr_api::dots::Dots)
            });
    }

    Ok(VariadicDots {
        has_dots,
        named_dots,
    })
}

/// Return the identifier for a trailing `Dots` / `&Dots` parameter, if present.
pub(crate) fn trailing_dots_ident(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
) -> Option<syn::Ident> {
    let syn::FnArg::Typed(pat_type) = inputs.last()? else {
        return None;
    };
    if !is_dots_type(pat_type.ty.as_ref()) {
        return None;
    }
    let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
        return None;
    };
    Some(pat_ident.ident.clone())
}

/// Check if a type is `Missing<T>`.
pub(crate) fn is_missing_type(ty: &syn::Type) -> bool {
    type_ends_with(ty, "Missing")
}

/// Check if a type is a vector-like type that `several_ok` can populate.
///
/// Accepts `Vec<T>`, `Box<[T]>`, `&[T]` / `&mut [T]`, and `[T; N]`. Rejects
/// scalar types (like `Mode`, `String`, `&str`) so `several_ok` — which
/// produces a multi-element R character vector via
/// `match.arg(..., several.ok = TRUE)` — fails at compile time instead of
/// deserialization time.
pub(crate) fn is_vector_like_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(tp) => {
            let Some(seg) = tp.path.segments.last() else {
                return false;
            };
            if seg.ident == "Vec" {
                return true;
            }
            if seg.ident == "Box" {
                let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
                    return false;
                };
                return matches!(
                    args.args.first(),
                    Some(syn::GenericArgument::Type(syn::Type::Slice(_)))
                );
            }
            false
        }
        syn::Type::Reference(r) => matches!(&*r.elem, syn::Type::Slice(_)),
        syn::Type::Slice(_) => true,
        syn::Type::Array(_) => true,
        _ => false,
    }
}

/// Extract the inner type `T` from `Missing<T>`, if the type is `Missing<T>`.
///
/// Returns `None` if the type is not `Missing<T>` or has no generic argument.
pub(crate) fn get_missing_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(tp) = ty else {
        return None;
    };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Missing" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
        Some(inner)
    } else {
        None
    }
}

/// Validate a parameter's type for `Missing` and `Dots` conflicts.
///
/// Returns `Err` if:
/// - `Missing<Missing<T>>` (nested Missing)
/// - `Missing<Dots>` or `Missing<&Dots>`
pub(crate) fn validate_param_type(ty: &syn::Type, span: proc_macro2::Span) -> syn::Result<()> {
    if let Some(inner) = get_missing_inner_type(ty) {
        if is_missing_type(inner) {
            return Err(syn::Error::new(
                span,
                "Missing<T> cannot be nested; use Missing<T> with the inner type directly",
            ));
        }
        if is_dots_type(inner) {
            return Err(syn::Error::new(
                span,
                "Missing<T> cannot wrap Dots; variadic parameters (...) are always present when called",
            ));
        }
    }
    Ok(())
}

/// Validate per-parameter attribute conflicts.
///
/// Returns `Err` if:
/// - `coerce` + `match_arg` on the same parameter
/// - `coerce` + `choices(...)` on the same parameter
/// - `choices(...)` + explicit `default` on the same parameter
/// - `default` on a `&Dots` parameter
pub(crate) fn validate_per_param_attr_conflicts(
    attr: &PerParamMiniextendrAttr,
    param_name: &str,
    is_dots: bool,
    ty: Option<&syn::Type>,
    span: proc_macro2::Span,
) -> syn::Result<()> {
    if attr.has_coerce && attr.has_match_arg {
        return Err(syn::Error::new(
            span,
            format!(
                "cannot combine coerce and match_arg on parameter `{}`; \
                 coerce converts the R type while match_arg validates string values",
                param_name
            ),
        ));
    }
    if attr.has_coerce && attr.choices.is_some() {
        return Err(syn::Error::new(
            span,
            format!(
                "cannot combine coerce and choices on parameter `{}`; \
                 coerce converts the R type while choices validates string values",
                param_name
            ),
        ));
    }
    if attr.choices.is_some() && attr.default_value.is_some() {
        return Err(syn::Error::new(
            span,
            format!(
                "cannot combine choices() and default on parameter `{}`; \
                 choices auto-generates its default from the first choice value",
                param_name
            ),
        ));
    }
    if attr.has_several_ok && attr.choices.is_none() && !attr.has_match_arg {
        return Err(syn::Error::new(
            span,
            format!(
                "several_ok requires choices() or match_arg on parameter `{}`; \
                 several_ok enables multi-value match.arg which needs a choice list",
                param_name
            ),
        ));
    }
    if attr.has_several_ok
        && let Some(ty) = ty
    {
        // Unwrap Missing<T> so several_ok is allowed on optional vector params.
        let check_ty = get_missing_inner_type(ty).unwrap_or(ty);
        if !is_vector_like_type(check_ty) {
            return Err(syn::Error::new(
                span,
                format!(
                    "several_ok requires a vector type on parameter `{}`; \
                     several_ok enables multi-value match.arg which returns a character vector. \
                     Use `Vec<T>`, `Box<[T]>`, `&[T]`, or `[T; N]` instead of a scalar type",
                    param_name
                ),
            ));
        }
    }
    if is_dots && attr.default_value.is_some() {
        return Err(syn::Error::new(
            span,
            format!(
                "variadic (...) parameter `{}` cannot have a default value",
                param_name
            ),
        ));
    }
    if let Some(ty) = ty
        && is_missing_type(ty)
        && attr.default_value.is_some()
    {
        return Err(syn::Error::new(
            span,
            format!(
                "`Missing<T>` parameter `{}` cannot have a default value. \
                 `Missing<T>` detects omitted arguments via `missing()` in R, \
                 which is incompatible with default values in the R function signature. \
                 Use `Option<T>` with `#[miniextendr(default = \"...\")]` instead.",
                param_name
            ),
        ));
    }
    Ok(())
}

// endregion

// region: Per-parameter attribute parsing

/// Parsed per-parameter `#[miniextendr(...)]` attribute content.
///
/// A single attribute can contain multiple items, e.g.
/// `#[miniextendr(match_arg, default = "Safe")]`.
#[derive(Default)]
pub(crate) struct PerParamMiniextendrAttr {
    /// Whether `coerce` was present, enabling automatic type coercion for this parameter
    /// (e.g., `i32` to `u16`, `f64` to `f32`).
    pub has_coerce: bool,
    /// Whether `match_arg` was present, generating R `match.arg()` validation for
    /// string parameters against a set of allowed values.
    pub has_match_arg: bool,
    /// Default value from `default = "..."`, if present. The tuple contains the default
    /// value string and the attribute span (for error reporting).
    pub default_value: Option<(String, proc_macro2::Span)>,
    /// Choices for string parameters: `#[miniextendr(choices("a", "b", "c"))]`.
    pub choices: Option<Vec<String>>,
    /// Whether `several_ok` was present, enabling multi-value `match.arg(several.ok = TRUE)`.
    /// Only valid with `choices(...)` or `match_arg`.
    pub has_several_ok: bool,
}

/// Parse all per-parameter options from a `#[miniextendr(...)]` attribute.
///
/// Handles mixed content like `#[miniextendr(match_arg, default = "\"Safe\"")]`
/// and `#[miniextendr(choices("a", "b", "c"))]`.
///
/// Returns `None` if `attr` is not a `#[miniextendr(...)]` attribute, if it cannot
/// be parsed, or if it contains only function-level options (like `strict`) with
/// no per-parameter options.
///
/// # Arguments
///
/// * `attr` - A `syn::Attribute` to inspect. Only attributes with path `miniextendr`
///   are considered.
pub(crate) fn parse_per_param_attr(attr: &syn::Attribute) -> Option<PerParamMiniextendrAttr> {
    use syn::spanned::Spanned;
    if !attr.path().is_ident("miniextendr") {
        return None;
    }

    let syn::Meta::List(meta_list) = &attr.meta else {
        return None;
    };

    let mut result = PerParamMiniextendrAttr::default();
    let mut is_per_param = false;

    let metas = match meta_list
        .parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
    {
        Ok(m) => m,
        Err(_) => return None,
    };

    for meta in &metas {
        match meta {
            syn::Meta::Path(path) => {
                if path.is_ident("coerce") {
                    result.has_coerce = true;
                    is_per_param = true;
                } else if path.is_ident("match_arg") {
                    result.has_match_arg = true;
                    is_per_param = true;
                } else if path.is_ident("several_ok") {
                    result.has_several_ok = true;
                    is_per_param = true;
                }
                // Other paths (like `strict`) are function-level, ignore here
            }
            syn::Meta::NameValue(nv) => {
                if nv.path.is_ident("default")
                    && let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = &nv.value
                {
                    result.default_value = Some((lit_str.value(), attr.span()));
                    is_per_param = true;
                }
                // Other name-value pairs are function-level, ignore here
            }
            syn::Meta::List(list) => {
                if list.path.is_ident("choices") {
                    // Parse choices("a", "b", "c") — a comma-separated list of string literals
                    let choice_lits = match list.parse_args_with(
                        syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated,
                    ) {
                        Ok(lits) => lits,
                        Err(_) => continue,
                    };
                    let choices: Vec<String> = choice_lits.iter().map(|l| l.value()).collect();
                    result.choices = Some(choices);
                    is_per_param = true;
                }
                // Other list forms are function-level, ignore here
            }
        }
    }

    if !is_per_param {
        return None;
    }
    Some(result)
}

/// Returns `true` if `attr` is a `#[miniextendr(...)]` attribute containing `coerce`.
///
/// The `coerce` flag may be combined with other per-parameter options (e.g.,
/// `#[miniextendr(coerce, default = "0")]`).
pub(crate) fn is_miniextendr_coerce_attr(attr: &syn::Attribute) -> bool {
    parse_per_param_attr(attr).is_some_and(|a| a.has_coerce)
}

/// Returns `true` if `attr` is a `#[miniextendr(...)]` attribute containing `match_arg`.
///
/// The `match_arg` flag may be combined with other per-parameter options (e.g.,
/// `#[miniextendr(match_arg, choices("a", "b"))]`).
pub(crate) fn is_miniextendr_match_arg_attr(attr: &syn::Attribute) -> bool {
    parse_per_param_attr(attr).is_some_and(|a| a.has_match_arg)
}

/// Returns `true` if `attr` is a `#[miniextendr(...)]` attribute containing `choices(...)`.
///
/// The `choices(...)` option may be combined with other per-parameter options (e.g.,
/// `#[miniextendr(match_arg, choices("a", "b"))]`).
pub(crate) fn is_miniextendr_choices_attr(attr: &syn::Attribute) -> bool {
    parse_per_param_attr(attr).is_some_and(|a| a.choices.is_some())
}

/// Returns `true` if `attr` is a `#[miniextendr(...)]` attribute containing `several_ok`.
pub(crate) fn is_miniextendr_several_ok_attr(attr: &syn::Attribute) -> bool {
    parse_per_param_attr(attr).is_some_and(|a| a.has_several_ok)
}

/// Extracts the list of choice strings from a `#[miniextendr(choices("a", "b", "c"))]` attribute.
///
/// Returns `None` if the attribute does not contain `choices(...)` or is not a
/// `#[miniextendr(...)]` attribute.
pub(crate) fn parse_choices_attr(attr: &syn::Attribute) -> Option<Vec<String>> {
    parse_per_param_attr(attr).and_then(|a| a.choices)
}

/// Extracts the default value from a `#[miniextendr(default = "...")]` attribute.
///
/// Returns `Some((default_value, attr_span))` if the attribute contains a `default` option.
/// The span is used for error reporting when the default references a non-existent parameter.
pub(crate) fn parse_default_attr(attr: &syn::Attribute) -> Option<(String, proc_macro2::Span)> {
    parse_per_param_attr(attr).and_then(|a| a.default_value)
}
// endregion

// region: Function parsing

/// Parsed + normalized Rust function item for `#[miniextendr]`.
///
/// This performs signature normalization that the wrapper generator depends on:
/// - `...` → a final `&miniextendr_api::dots::Dots` argument
/// - `_` wildcard patterns → synthetic identifiers (`__unused0`, `__unused1`, ...)
/// - Destructuring patterns (tuple, struct) → synthetic identifiers with let-binding in body
/// - consumes `#[miniextendr(coerce)]` parameter attributes and records which params had it
pub(crate) struct MiniextendrFunctionParsed {
    /// The normalized function item (with dots transformed, wildcards renamed).
    item: syn::ItemFn,
    /// Whether the original function had `...` (variadic).
    has_dots: bool,
    /// If dots were named (e.g., `my_dots: ...`), the identifier.
    named_dots: Option<syn::Ident>,
    /// All per-parameter `#[miniextendr(...)]` options (coerce, match_arg,
    /// default, choices, several_ok), keyed by the (possibly synthesized) Rust
    /// parameter name. Replaces five parallel `HashSet` / `HashMap` fields.
    per_param: std::collections::HashMap<String, ParamAttrs>,
}

/// Collapsed per-parameter attribute state for a single function parameter.
///
/// Built during parsing from `#[miniextendr(coerce | match_arg | several_ok |
/// default = "…" | choices("…"))]` on the argument. Accessors on
/// [`MiniextendrFunctionParsed`] query this struct rather than looking
/// through multiple side-tables.
#[derive(Default, Debug, Clone)]
pub(crate) struct ParamAttrs {
    pub coerce: bool,
    pub match_arg: bool,
    pub several_ok: bool,
    pub choices: Option<Vec<String>>,
    pub default: Option<String>,
}

/// Parses a Rust `fn` item from a token stream, performing all normalizations
/// required by the `#[miniextendr]` codegen pipeline.
///
/// # Normalizations performed
///
/// 1. **Variadic (`...`) rewriting**: Replaces Rust variadic syntax with a typed
///    `&miniextendr_api::dots::Dots` parameter. Named dots (`my_dots: ...`) preserve
///    the user's identifier; unnamed `...` becomes `__miniextendr_dots`.
/// 2. **Wildcard pattern renaming**: `_` parameter patterns become `__unused0`,
///    `__unused1`, etc., so they can be passed by name to the C wrapper.
/// 3. **Destructuring expansion**: Tuple/struct destructuring patterns are replaced
///    with synthetic identifiers (`__param_0`, ...) and a `let` binding is prepended
///    to the function body.
/// 4. **Per-parameter attribute consumption**: `#[miniextendr(coerce)]`,
///    `#[miniextendr(match_arg)]`, `#[miniextendr(default = "...")]`, and
///    `#[miniextendr(choices(...))]` are consumed from parameters and recorded in
///    the corresponding `per_param_*` fields.
/// 5. **Validation**: Rejects `#[export_name]` on non-extern functions, rejects
///    unsupported parameter patterns, and validates that defaults reference existing
///    parameter names.
impl syn::parse::Parse for MiniextendrFunctionParsed {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::spanned::Spanned;

        let mut item: syn::ItemFn = input.parse()?;

        // dots support: parse variadic name (if any) and replace `...` with `&Dots`.
        let dots_info = rewrite_variadic_dots(&mut item.sig)?;
        let has_dots = dots_info.has_dots;
        let named_dots = dots_info.named_dots;

        // Reject #[export_name] for regular functions (not extern "C-unwind").
        // For extern functions, #[export_name] can be used as an alternative to #[no_mangle].
        let is_extern = item.sig.abi.is_some();
        if !is_extern {
            for attr in &item.attrs {
                if attr.path().is_ident("export_name") {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "#[export_name] is not supported with #[miniextendr] on regular functions; \
                         use `#[miniextendr(c_symbol = \"...\")]` to customize the C symbol name. \
                         For extern \"C-unwind\" functions, #[export_name] is allowed.",
                    ));
                }
            }
        }

        // Transform `_` wildcard patterns to synthetic identifiers, and consume
        // per-parameter `#[miniextendr(coerce)]`, `#[miniextendr(default = "...")]`,
        // and `#[miniextendr(choices(...))]` attributes.
        let mut per_param: std::collections::HashMap<String, ParamAttrs> =
            std::collections::HashMap::new();
        let mut per_param_default_spans: std::collections::HashMap<String, proc_macro2::Span> =
            std::collections::HashMap::new();
        let mut unused_counter = 0usize;
        let mut pattern_destructures: Vec<(Box<syn::Pat>, syn::Ident)> = Vec::new();
        for arg in &mut item.sig.inputs {
            let syn::FnArg::Typed(pat_type) = arg else {
                // Self parameters are not allowed in standalone functions.
                // Users should use #[miniextendr(env|r6|s3|s4|s7)] on impl blocks instead.
                // The error is raised in lib.rs c_wrapper_inputs generation.
                continue;
            };

            let had_coerce_attr = pat_type.attrs.iter().any(is_miniextendr_coerce_attr);
            let had_match_arg_attr = pat_type.attrs.iter().any(is_miniextendr_match_arg_attr);
            let had_several_ok = pat_type.attrs.iter().any(is_miniextendr_several_ok_attr);
            let default_with_span = pat_type.attrs.iter().find_map(parse_default_attr);
            let had_choices = pat_type.attrs.iter().find_map(parse_choices_attr);

            // Remove miniextendr attributes from parameters (coerce, match_arg, choices, several_ok, default)
            pat_type.attrs.retain(|attr| {
                !is_miniextendr_coerce_attr(attr)
                    && !is_miniextendr_match_arg_attr(attr)
                    && !is_miniextendr_choices_attr(attr)
                    && !is_miniextendr_several_ok_attr(attr)
                    && parse_default_attr(attr).is_none()
            });

            // Validate type-based constraints (Missing nesting, Missing<Dots>)
            validate_param_type(pat_type.ty.as_ref(), pat_type.ty.span())?;

            // Resolve the Rust parameter name — either the user's identifier,
            // or a synthesized one for wildcard / destructuring patterns.
            let param_name: String = match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                syn::Pat::Wild(_) => {
                    let synthetic_name = format!("__unused{}", unused_counter);
                    unused_counter += 1;
                    let synthetic_ident = syn::Ident::new(&synthetic_name, pat_type.pat.span());
                    *pat_type.pat = syn::Pat::Ident(syn::PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: synthetic_ident,
                        subpat: None,
                    });
                    synthetic_name
                }
                syn::Pat::Tuple(_) | syn::Pat::TupleStruct(_) | syn::Pat::Struct(_) => {
                    let synthetic_name = format!("__param_{}", unused_counter);
                    unused_counter += 1;
                    let synthetic_ident = syn::Ident::new(&synthetic_name, pat_type.pat.span());
                    let original_pat = pat_type.pat.clone();
                    *pat_type.pat = syn::Pat::Ident(syn::PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: synthetic_ident.clone(),
                        subpat: None,
                    });
                    pattern_destructures.push((original_pat, synthetic_ident));
                    synthetic_name
                }
                _ => {
                    return Err(syn::Error::new(
                        pat_type.pat.span(),
                        "miniextendr parameters must be identifiers or destructuring patterns (tuple, struct)",
                    ));
                }
            };
            let param_name_for_validation = param_name.clone();

            // Record per-parameter attrs in one entry instead of five side-tables.
            if had_coerce_attr
                || had_match_arg_attr
                || had_several_ok
                || had_choices.is_some()
                || default_with_span.is_some()
            {
                let entry = per_param.entry(param_name.clone()).or_default();
                if had_coerce_attr {
                    entry.coerce = true;
                }
                if had_match_arg_attr {
                    entry.match_arg = true;
                }
                if had_several_ok {
                    entry.several_ok = true;
                }
                if let Some(choices) = had_choices.clone() {
                    entry.choices = Some(choices);
                }
                if let Some((default, span)) = default_with_span.clone() {
                    entry.default = Some(default);
                    per_param_default_spans.insert(param_name, span);
                }
            }

            // Validate per-parameter attribute conflicts (coerce+match_arg, coerce+choices, etc.)
            let per_param_combined = PerParamMiniextendrAttr {
                has_coerce: had_coerce_attr,
                has_match_arg: had_match_arg_attr,
                default_value: default_with_span,
                choices: had_choices,
                has_several_ok: had_several_ok,
            };
            validate_per_param_attr_conflicts(
                &per_param_combined,
                &param_name_for_validation,
                is_dots_type(pat_type.ty.as_ref()),
                Some(pat_type.ty.as_ref()),
                pat_type.ty.span(),
            )?;
        }

        // Insert destructuring let-bindings for pattern parameters at the start of the function body
        for (pat, ident) in pattern_destructures.iter().rev() {
            item.block.stmts.insert(
                0,
                syn::parse_quote! {
                    let #pat = #ident;
                },
            );
        }

        // Validate: all defaults reference existing parameters
        let param_names: std::collections::HashSet<String> = item
            .sig
            .inputs
            .iter()
            .filter_map(|input| {
                if let syn::FnArg::Typed(pat_type) = input
                    && let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref()
                {
                    Some(pat_ident.ident.to_string())
                } else {
                    None
                }
            })
            .collect();

        let mut invalid_params: Vec<String> = per_param
            .iter()
            .filter_map(|(name, attrs)| {
                if attrs.default.is_some() && !param_names.contains(name) {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
        invalid_params.sort();

        if !invalid_params.is_empty() {
            // Use the span of the first invalid param's attribute for the error
            let error_span = invalid_params
                .first()
                .and_then(|p| per_param_default_spans.get(p).copied())
                .unwrap_or_else(|| item.sig.ident.span());
            return Err(syn::Error::new(
                error_span,
                format!(
                    "default attribute(s) reference non-existent parameter(s): {}",
                    invalid_params.join(", ")
                ),
            ));
        }

        Ok(Self {
            item,
            has_dots,
            named_dots,
            per_param,
        })
    }
}

/// Accessors and codegen helpers for [`MiniextendrFunctionParsed`].
///
/// Accessors are split into two groups:
/// - **Parsed metadata**: dots, coerce, match_arg, choices, and defaults from
///   per-parameter `#[miniextendr(...)]` attributes.
/// - **Signature components**: attrs, vis, abi, ident, generics, inputs, output
///   from the normalized `syn::ItemFn`.
///
/// Codegen helpers produce identifiers and perform mutations needed by the
/// `#[miniextendr]` expansion pipeline.
impl MiniextendrFunctionParsed {
    // region: Accessors for parsed metadata

    /// Whether the original function had `...` (variadic).
    pub(crate) fn has_dots(&self) -> bool {
        self.has_dots
    }

    /// If dots were named (e.g., `my_dots: ...`), returns the identifier.
    pub(crate) fn named_dots(&self) -> Option<&syn::Ident> {
        self.named_dots.as_ref()
    }

    /// Check if a parameter is the dots (`...`) param.
    /// After parsing, dots are rewritten to `&Dots` — this checks the original name.
    pub(crate) fn is_dots_param(&self, ident: &syn::Ident) -> bool {
        if !self.has_dots {
            return false;
        }
        // Named dots: check if ident matches the original name (e.g., `dots`, `my_dots`)
        if let Some(ref named) = self.named_dots {
            return ident == named;
        }
        // Unnamed dots: the variadic was replaced with `_dots` as the param name
        ident == "_dots"
    }

    /// Check if a parameter name had `#[miniextendr(coerce)]` attribute.
    pub(crate) fn has_coerce_attr(&self, param_name: &str) -> bool {
        self.per_param.get(param_name).is_some_and(|a| a.coerce)
    }

    /// Check if a parameter name had `#[miniextendr(match_arg)]` attribute.
    pub(crate) fn has_match_arg_attr(&self, param_name: &str) -> bool {
        self.per_param.get(param_name).is_some_and(|a| a.match_arg)
    }

    /// Iterator over parameter names annotated with `#[miniextendr(match_arg)]`.
    pub(crate) fn match_arg_params(&self) -> impl Iterator<Item = &String> {
        self.per_param
            .iter()
            .filter_map(|(name, a)| if a.match_arg { Some(name) } else { None })
    }

    /// Get the choices for a parameter, if any.
    pub(crate) fn choices_for_param(&self, param_name: &str) -> Option<&[String]> {
        self.per_param
            .get(param_name)
            .and_then(|a| a.choices.as_deref())
    }

    /// Iterator over parameter names annotated with `#[miniextendr(choices(…))]`,
    /// together with their choice lists.
    pub(crate) fn choices_params(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        self.per_param
            .iter()
            .filter_map(|(name, a)| a.choices.as_ref().map(|c| (name, c)))
    }

    /// Check if a parameter has `several_ok` (multi-value match.arg).
    pub(crate) fn has_several_ok(&self, param_name: &str) -> bool {
        self.per_param.get(param_name).is_some_and(|a| a.several_ok)
    }

    /// Returns all parameter defaults as an owned map from parameter name to
    /// default value string (the raw R expression used in the wrapper formals,
    /// e.g. `"NULL"`, `"TRUE"`, `"\"Safe\""`).
    pub(crate) fn param_defaults(&self) -> std::collections::HashMap<String, String> {
        self.per_param
            .iter()
            .filter_map(|(name, a)| a.default.as_ref().map(|d| (name.clone(), d.clone())))
            .collect()
    }
    // endregion

    // region: Accessors for signature components

    /// Original attributes on the function item (doc comments, cfgs, etc.).
    pub(crate) fn attrs(&self) -> &[syn::Attribute] {
        &self.item.attrs
    }

    /// Visibility of the function (`pub`, `pub(crate)`, or private).
    pub(crate) fn vis(&self) -> &syn::Visibility {
        &self.item.vis
    }

    /// Explicit ABI, if the function was declared `extern "C-unwind"`.
    pub(crate) fn abi(&self) -> Option<&syn::Abi> {
        self.item.sig.abi.as_ref()
    }

    /// Function identifier after normalization.
    pub(crate) fn ident(&self) -> &syn::Ident {
        &self.item.sig.ident
    }

    /// Generic parameters on the function signature.
    pub(crate) fn generics(&self) -> &syn::Generics {
        &self.item.sig.generics
    }

    /// Function inputs after normalization (dots rewritten, wildcards renamed).
    pub(crate) fn inputs(&self) -> &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]> {
        &self.item.sig.inputs
    }

    /// Function return type.
    pub(crate) fn output(&self) -> &syn::ReturnType {
        &self.item.sig.output
    }

    /// The normalized function item (with original doc comments).
    pub(crate) fn item(&self) -> &syn::ItemFn {
        &self.item
    }

    /// The normalized function item with roxygen tags stripped from doc comments.
    ///
    /// This is used for emitting the Rust function without R-specific documentation
    /// tags (e.g., `@param`, `@examples`) that don't belong in rustdoc.
    pub(crate) fn item_without_roxygen(&self) -> syn::ItemFn {
        let mut item = self.item.clone();
        item.attrs = crate::roxygen::strip_roxygen_from_attrs(&item.attrs);
        item
    }
    // endregion

    // region: Codegen helpers

    /// Returns `true` if this function needs an internal C wrapper (`C_<crate>_<name>` function).
    ///
    /// Rust-ABI functions (no explicit `extern`) need a generated `extern "C-unwind"` wrapper
    /// that handles SEXP conversion and error propagation. Functions already declared as
    /// `extern "C-unwind"` are passed through directly without wrapping.
    pub(crate) fn uses_internal_c_wrapper(&self) -> bool {
        self.abi().is_none()
    }

    /// Returns the identifier for the generated `const &str` holding the R wrapper code.
    ///
    /// The R wrapper is a string constant containing the R function definition that
    /// calls `.Call(C_<crate>_<name>, ...)`. It is collected via linkme distributed slices to
    /// produce the `R/miniextendr-wrappers.R` file.
    pub(crate) fn r_wrapper_const_ident(&self) -> syn::Ident {
        r_wrapper_const_ident_for(self.ident())
    }

    /// Returns the identifier for the C-callable entry point.
    ///
    /// - **Rust ABI functions**: Returns `C_<crate>_<name>` (the generated wrapper
    ///   function, crate-prefixed for webR cross-package symbol uniqueness — #1273).
    /// - **`extern "C-unwind"` functions**: Returns the function's own name, or the
    ///   value from `#[export_name = "..."]` if present. The user owns these symbols,
    ///   including their cross-package uniqueness under webR.
    pub(crate) fn c_wrapper_ident(&self) -> syn::Ident {
        if self.uses_internal_c_wrapper() {
            crate::naming::bare_fn_c_wrapper_ident(self.ident())
        } else {
            // For extern functions, check for #[export_name = "..."]
            self.export_name_ident()
                .unwrap_or_else(|| self.ident().clone())
        }
    }

    /// Extracts the custom symbol name from `#[export_name = "..."]`, if present.
    ///
    /// Only meaningful for `extern "C-unwind"` functions, where `#[export_name]` is
    /// allowed as an alternative to `#[no_mangle]`. Returns `None` if no such attribute exists.
    pub(crate) fn export_name_ident(&self) -> Option<syn::Ident> {
        for attr in &self.item.attrs {
            if attr.path().is_ident("export_name")
                && let syn::Meta::NameValue(meta) = &attr.meta
                && let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &meta.value
            {
                return Some(syn::Ident::new(&lit_str.value(), lit_str.span()));
            }
        }
        None
    }

    /// Add `#[inline(never)]` if no `#[inline(...)]` attribute is present.
    /// Only for Rust ABI functions - extern "C-unwind" functions are passed through as-is.
    ///
    /// Preventing inlining ensures:
    /// - Worker-dispatched functions retain a distinct call frame
    /// - Panic handling and unwinding retain the intended boundary
    /// - Stack traces show the actual function name
    pub(crate) fn add_inline_never_if_needed(&mut self) {
        let has_explicit_abi = self.item.sig.abi.is_some();
        let has_inline = self
            .item
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("inline"));
        if !has_inline && !has_explicit_abi {
            self.item.attrs.push(syn::parse_quote!(#[inline(never)]));
        }
    }
    // endregion
}
// endregion

// region: Attribute parsing

/// Parse the value of a `name = "..."` meta item as a string literal.
///
/// Returns a compile error spanning the offending token when the RHS is not a
/// `&str` literal. `field` is used in the diagnostic (e.g. `"c_symbol"`).
/// `postfix = "..."` must be a non-empty R identifier fragment: it is appended
/// verbatim to the Rust name, so anything outside letters, digits, `_` and `.`
/// would produce an R name that needs backticks.
pub(crate) fn validate_postfix(val: &str, span: &dyn quote::ToTokens) -> syn::Result<()> {
    if val.is_empty() {
        return Err(syn::Error::new_spanned(span, "postfix must not be empty"));
    }
    if !val
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        return Err(syn::Error::new_spanned(
            span,
            "postfix must be a valid R identifier fragment (letters, digits, `_`, `.`)",
        ));
    }
    Ok(())
}

fn parse_lit_str(nv: &syn::MetaNameValue, field: &str) -> syn::Result<String> {
    match &nv.value {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) => Ok(lit.value()),
        syn::Expr::Lit(expr_lit) => Err(syn::Error::new_spanned(
            &expr_lit.lit,
            format!("{field} expects a string literal"),
        )),
        other => Err(syn::Error::new_spanned(
            other,
            format!("{field} expects a string literal"),
        )),
    }
}

/// Comma-separated list of all fn-level boolean flags, for error messages.
///
/// Kept as a single constant so the three "unknown option" error paths (Path,
/// NameValue bool, parenthesized bool) all read from the same list and can't
/// drift.
const FN_BOOL_FLAGS_HELP: &str = "invisible, visible, check_interrupt, worker, no_worker, coerce, no_coerce, \
     rng, unwrap_in_r, strict, no_strict, \
     no_preconditions, no_call_attribution, fast, no_fast, \
     internal, noexport, export";

/// Comma-separated list of fn-level nested options, for error messages.
const FN_NESTED_OPTIONS_HELP: &str = "`s3(...)`, `lifecycle(...)`, `defaults(...)`";

/// Parsed arguments for the `#[miniextendr(...)]` attribute on functions.
///
/// This is intentionally a small, "data-only" struct that:
/// - Owns the parsing rules for the attribute
/// - Produces a normalized, easy-to-consume representation for codegen
///
/// # Accepted flags
///
/// - `invisible` / `visible`: control whether the generated R wrapper returns invisibly
/// - `check_interrupt`: insert `R_CheckUserInterrupt()` before calling Rust
/// - `worker`: opt into worker-thread execution (default is main thread)
/// - `coerce`: enable automatic coercion for supported parameter types
/// - `rng`: enable RNG state management (GetRNGstate/PutRNGstate)
/// - `unwrap_in_r`: return `Result<T, E>` to R without unwrapping
/// - `prefer = "auto" | "list" | "externalptr" | "vector"`: prefer a specific `IntoR` path
/// - `no_preconditions`: drop the R-side `stopifnot(...)` block. `TryFromSexp`
///   still raises on bad input; the message comes from Rust rather than R.
///   Saves ~300 ns per assertion (~600 ns per scalar arg, ~1230 ns for a 1-arg
///   numeric scalar fn). Hot-path opt-in. Opt out with `no_fast` when `fast-default`
///   is enabled.
/// - `no_call_attribution`: emit `.call = NULL` instead of `.call = match.call()`
///   in the generated R wrapper. The error fallback `sys.call()` preserves
///   wrapper-invocation attribution (positional args instead of named). Saves
///   ~1200 ns per call regardless of arg count.
/// - `fast`: shorthand for `no_preconditions + no_call_attribution`. The
///   biggest single-knob wrapper speedup.
/// - `no_fast`: explicit opt-out of both knobs (useful when `fast-default`
///   feature is enabled crate-wide to restore full error UX for a specific fn).
///
/// See `analysis/scaffolding-deep-findings-2026-05-20.md` for the measurement
/// underlying these options (~13× speedup possible on the wrapper layer).
///
/// # Note
///
/// Unknown flags are rejected with a compile error to avoid silently ignoring typos.
#[derive(Default)]
pub(crate) struct MiniextendrFnAttrs {
    /// Force execution on worker thread (set by `worker`).
    pub(crate) force_worker: bool,
    /// Override visibility; `Some(true)` makes the wrapper return invisibly, `Some(false)` forces visibility.
    pub(crate) force_invisible: Option<bool>,
    /// Insert `R_CheckUserInterrupt()` before calling the Rust function.
    pub(crate) check_interrupt: bool,
    /// Enable automatic coercion for all parameters that support it.
    pub(crate) coerce_all: bool,
    /// Enable RNG state management (GetRNGstate/PutRNGstate).
    pub(crate) rng: bool,
    /// Return `Result<T, E>` to R without unwrapping.
    pub(crate) unwrap_in_r: bool,
    /// Skip emission of the R-side `stopifnot(...)` precondition block.
    ///
    /// `TryFromSexp` already raises a typed Rust error on mismatched input,
    /// so the information isn't lost — just routed through the Rust error
    /// message rather than R's stopifnot text. Useful for hot paths where the
    /// per-call precondition cost (~300 ns per assertion, ~600 ns per arg for
    /// numeric scalars) dominates over actual work.
    ///
    /// Set by `#[miniextendr(no_preconditions)]` or implied by `fast`.
    /// Use `no_fast` to opt out when `fast-default` is enabled.
    pub(crate) no_preconditions: bool,
    /// Emit `.call = NULL` instead of `.call = match.call()` in the generated
    /// R wrapper.
    ///
    /// `match.call()` costs ~1200 ns per call (fixed, independent of arg
    /// count) but is only consulted on the error path by
    /// `.miniextendr_raise_condition`. When `.call = NULL`, the helper falls
    /// back to `sys.call()` which surfaces the same wrapper invocation
    /// (positional args instead of named).
    ///
    /// Set by `#[miniextendr(no_call_attribution)]` or implied by `fast`.
    /// Use `no_fast` to opt out when `fast-default` is enabled.
    pub(crate) no_call_attribution: bool,
    /// Preferred return conversion: forces `AsList`/`AsExternalPtr`/`AsRNative` wrapping
    /// of the return value before `IntoR::into_sexp` is called.
    pub(crate) return_pref: ReturnPref,
    /// Span of the `prefer = ...` attribute, for error reporting when the return type
    /// falls into a codegen category (`Option<T>`, `Result<T, E>`, `()`, `Self`, raw
    /// `SEXP`, ...) that can't honor it.
    pub(crate) return_pref_span: Option<proc_macro2::Span>,
    /// S3 generic name (if this function is an S3 method).
    ///
    /// Use `#[miniextendr(s3(generic = "vec_proxy", class = "my_vctr"))]` to mark a function
    /// as an S3 method for an existing generic.
    pub(crate) s3_generic: Option<String>,
    /// S3 class suffix for the method (e.g., "my_vctr" or "my_vctr.my_vctr" for double-dispatch).
    pub(crate) s3_class: Option<String>,
    /// Typed list validation spec for dots parameter.
    ///
    /// Use `#[miniextendr(dots = typed_list!(...))]` to automatically validate dots
    /// at the start of the function and bind the result to `dots_typed`.
    pub(crate) dots_spec: Option<proc_macro2::TokenStream>,
    /// Span of the `dots = ...` attribute for error reporting.
    pub(crate) dots_span: Option<proc_macro2::Span>,
    /// Lifecycle specification for deprecation/experimental status.
    pub(crate) lifecycle: Option<crate::lifecycle::LifecycleSpec>,
    /// Strict output conversion: panic instead of lossy widening for i64/u64/isize/usize.
    pub(crate) strict: bool,
    /// Mark as internal: adds `@keywords internal`, suppresses `@export`.
    pub(crate) internal: bool,
    /// Suppress `@export` without adding `@keywords internal`.
    pub(crate) noexport: bool,
    /// Force `@export` even on non-pub functions. Antidote to `noexport`.
    pub(crate) export: bool,
    /// Custom roxygen documentation override.
    ///
    /// When set, replaces auto-extracted roxygen from Rust doc comments.
    /// Each `\n` in the string becomes a separate `#'` line.
    pub(crate) doc: Option<String>,
    /// Custom C symbol name for the generated wrapper.
    ///
    /// Overrides the default `C_<crate>_<fn_name>` naming convention. The value is used
    /// verbatim (no crate prefix) — the author owns cross-package uniqueness on webR (#1273).
    /// Must be a valid C identifier (alphanumeric + underscore, starting with letter or underscore).
    pub(crate) c_symbol: Option<String>,
    /// Override R wrapper function name.
    ///
    /// Use `#[miniextendr(r_name = "is.my_type")]` to give the R wrapper a different name
    /// than the Rust function. The C symbol is still derived from the Rust name.
    /// Cannot be combined with `s3(generic/class)` — use `generic`/`class` for S3 naming.
    pub(crate) r_name: Option<String>,
    /// Append a fixed suffix to the Rust name for the R wrapper
    /// (`#[miniextendr(noexport, postfix = "_impl")]` on `fn f` yields `f_impl`).
    /// States the "hand-written `f()` delegates to generated `f_impl()`"
    /// convention without repeating the name in `r_name`. Exclusive with
    /// `r_name` and `s3(...)`; the C symbol is unchanged.
    pub(crate) postfix: Option<String>,
    /// `call = caller`: attribute conditions to the wrapper's caller instead of
    /// the wrapper's own call (the body binds `.mx_call` from the parent frame
    /// and passes `.call = .mx_call`; see
    /// `crate::r_wrapper_builder::CallAttribution::Caller`). For `noexport` /
    /// `internal` entry points behind a hand-written R function, so errors name
    /// the public function, not the bridge.
    pub(crate) call_caller: bool,
    /// R code to inject at the very top of the wrapper body (before all built-in checks).
    ///
    /// Use `#[miniextendr(r_entry = "x <- as.integer(x)")]` to run R code before
    /// missing-default handling, lifecycle checks, stopifnot, and match.arg.
    /// Multi-line via `\n`. No validation of R syntax.
    pub(crate) r_entry: Option<String>,
    /// R code to inject after all built-in checks, immediately before `.Call()`.
    ///
    /// Use `#[miniextendr(r_post_checks = "message('calling rust')")]` to run R code
    /// after all precondition checks but before the Rust function is invoked.
    /// Multi-line via `\n`. No validation of R syntax.
    pub(crate) r_post_checks: Option<String>,
    /// Register `on.exit()` cleanup code in the R wrapper.
    ///
    /// Short form: `#[miniextendr(r_on_exit = "close(con)")]` → `on.exit(close(con), add = TRUE)`
    ///
    /// Long form: `#[miniextendr(r_on_exit(expr = "close(con)", add = false))]`
    ///
    /// Defaults: `add = TRUE`, `after = TRUE`. Injected after `r_entry`, before other checks.
    pub(crate) r_on_exit: Option<ROnExit>,
}

/// Parsed `r_on_exit` attribute for `on.exit()` cleanup code in R wrappers.
///
/// Two forms:
/// - Short: `r_on_exit = "expr"` → `ROnExit { expr, add: true, after: true }`
/// - Long: `r_on_exit(expr = "...", add = false, after = false)`
///
/// Defaults match R conventions for composable code: `add = TRUE`, `after = TRUE`.
#[derive(Debug, Clone)]
pub(crate) struct ROnExit {
    pub expr: String,
    pub add: bool,
    pub after: bool,
}

impl ROnExit {
    /// Generate the R `on.exit(...)` call string.
    ///
    /// - `add = FALSE` (R default): `on.exit(expr)`
    /// - `add = TRUE, after = TRUE`: `on.exit(expr, add = TRUE)`
    /// - `add = TRUE, after = FALSE`: `on.exit(expr, add = TRUE, after = FALSE)`
    pub fn to_r_code(&self) -> String {
        if !self.add {
            format!("on.exit({})", self.expr)
        } else if !self.after {
            format!("on.exit({}, add = TRUE, after = FALSE)", self.expr)
        } else {
            format!("on.exit({}, add = TRUE)", self.expr)
        }
    }
}

#[derive(Clone, Copy, Default)]
/// Preferred return-conversion path for `IntoR`.
pub(crate) enum ReturnPref {
    /// Use the default `IntoR` implementation for the type.
    #[default]
    Auto,
    /// Force list conversion via the `AsList` wrapper.
    List,
    /// Force external pointer conversion via the `AsExternalPtr` wrapper.
    ExternalPtr,
    /// Force native vector/scalar conversion via the `AsRNative` wrapper.
    Native,
}

/// Parses the comma-separated option list inside `#[miniextendr(...)]`.
///
/// Supports three syntactic forms for each option:
/// - **Bare identifier**: `#[miniextendr(invisible)]`
/// - **Name-value**: `#[miniextendr(prefer = "list")]` or `#[miniextendr(invisible = true)]`
/// - **Nested list**: `#[miniextendr(s3(generic = "...", class = "..."))]`
///
/// Options with negated forms (`no_worker`, `no_coerce`, `no_strict`) explicitly
/// disable the corresponding flag, which is useful for overriding feature-based
/// defaults.
///
/// An empty input (plain `#[miniextendr]`) resolves all options to their feature-based
/// defaults (e.g., `worker-default`, `coerce-default`, `strict-default`).
///
/// # Errors
///
/// Returns a compile error for:
/// - Unknown option names (prevents silent typos)
/// - Mutually exclusive options (`internal` + `noexport`)
/// - Invalid values for key-value options (e.g., bad `prefer` or `c_symbol`)
/// - Missing required sub-options (e.g., `s3(...)` without `class`)
impl syn::parse::Parse for MiniextendrFnAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::spanned::Spanned;
        // Use Option<bool> for fields that support feature defaults.
        // None = not explicitly set → resolve from cfg!(feature = "...") at end.
        let mut force_worker: Option<bool> = None;
        let mut force_invisible: Option<bool> = None;
        let mut check_interrupt = false;
        let mut coerce_all: Option<bool> = None;
        let mut rng = false;
        let mut unwrap_in_r = false;
        let mut no_preconditions: Option<bool> = None;
        let mut no_call_attribution: Option<bool> = None;
        let mut return_pref = ReturnPref::Auto;
        let mut return_pref_span: Option<proc_macro2::Span> = None;
        let mut s3_generic = None;
        let mut s3_class = None;
        let mut dots_spec = None;
        let mut dots_span = None;
        let mut lifecycle = None;
        let mut strict: Option<bool> = None;
        let mut internal = false;
        let mut noexport = false;
        let mut export = false;
        let mut doc = None;
        let mut c_symbol = None;
        let mut r_name = None;
        let mut postfix = None;
        let mut call_caller = false;
        let mut r_entry = None;
        let mut r_post_checks = None;
        let mut r_on_exit = None;

        // Empty input (`#[miniextendr]`) → skip the parse loop and fall through
        // to the single Ok(Self {...}) at the bottom; every local is already
        // seeded with its default value above.
        let metas = if input.is_empty() {
            syn::punctuated::Punctuated::new()
        } else {
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated(input)?
        };

        for meta in metas {
            match meta {
                // Simple identifiers: invisible, visible, check_interrupt, coerce, worker, rng
                syn::Meta::Path(path) => {
                    if let Some(ident) = path.get_ident() {
                        if ident == "invisible" {
                            force_invisible = Some(true);
                        } else if ident == "visible" {
                            force_invisible = Some(false);
                        } else if ident == "check_interrupt" {
                            check_interrupt = true;
                        } else if ident == "coerce" {
                            coerce_all = Some(true);
                        } else if ident == "no_coerce" {
                            coerce_all = Some(false);
                        } else if ident == "rng" {
                            rng = true;
                        } else if ident == "unwrap_in_r" {
                            unwrap_in_r = true;
                        } else if ident == "worker" {
                            force_worker = Some(true);
                        } else if ident == "no_worker" {
                            force_worker = Some(false);
                        } else if ident == "strict" {
                            strict = Some(true);
                        } else if ident == "no_strict" {
                            strict = Some(false);
                        } else if ident == "no_preconditions" {
                            no_preconditions = Some(true);
                        } else if ident == "no_call_attribution" {
                            no_call_attribution = Some(true);
                        } else if ident == "fast" {
                            // Bundle alias: drop the two biggest R-side
                            // overheads in the generated wrapper.
                            no_preconditions = Some(true);
                            no_call_attribution = Some(true);
                        } else if ident == "no_fast" {
                            // Explicit opt-out: restore full error UX even when
                            // `fast-default` feature is enabled crate-wide.
                            no_preconditions = Some(false);
                            no_call_attribution = Some(false);
                        } else if ident == "internal" {
                            internal = true;
                        } else if ident == "noexport" {
                            noexport = true;
                        } else if ident == "export" {
                            export = true;
                        } else {
                            return Err(syn::Error::new_spanned(
                                ident,
                                format!(
                                    "unknown `#[miniextendr]` option; expected one of: {FN_BOOL_FLAGS_HELP}"
                                ),
                            ));
                        }
                    }
                }
                syn::Meta::NameValue(nv) => {
                    // Check for boolean flag options: option = true / option = false
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Bool(lit_bool),
                        ..
                    }) = &nv.value
                    {
                        let val = lit_bool.value;
                        if let Some(ident) = nv.path.get_ident() {
                            if ident == "invisible" {
                                force_invisible = Some(val);
                            } else if ident == "visible" {
                                force_invisible = Some(!val);
                            } else if ident == "check_interrupt" {
                                check_interrupt = val;
                            } else if ident == "worker" {
                                force_worker = Some(val);
                            } else if ident == "no_worker" {
                                force_worker = Some(!val);
                            } else if ident == "coerce" {
                                coerce_all = Some(val);
                            } else if ident == "no_coerce" {
                                coerce_all = Some(!val);
                            } else if ident == "rng" {
                                rng = val;
                            } else if ident == "unwrap_in_r" {
                                unwrap_in_r = val;
                            } else if ident == "strict" {
                                strict = Some(val);
                            } else if ident == "no_strict" {
                                strict = Some(!val);
                            } else if ident == "no_preconditions" {
                                no_preconditions = Some(val);
                            } else if ident == "no_call_attribution" {
                                no_call_attribution = Some(val);
                            } else if ident == "fast" {
                                no_preconditions = Some(val);
                                no_call_attribution = Some(val);
                            } else if ident == "no_fast" {
                                no_preconditions = Some(!val);
                                no_call_attribution = Some(!val);
                            } else if ident == "internal" {
                                internal = val;
                            } else if ident == "noexport" {
                                noexport = val;
                            } else if ident == "export" {
                                export = val;
                            } else {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    format!(
                                        "unknown `#[miniextendr]` option `{ident}`; expected one of: \
                                         {FN_BOOL_FLAGS_HELP}"
                                    ),
                                ));
                            }
                            continue;
                        }
                    }

                    if nv.path.is_ident("prefer") {
                        let v = parse_lit_str(&nv, "prefer")?;
                        return_pref_span = Some(nv.span());
                        return_pref = match v.as_str() {
                            "list" => ReturnPref::List,
                            "externalptr" => ReturnPref::ExternalPtr,
                            "vector" | "native" => ReturnPref::Native,
                            "auto" => ReturnPref::Auto,
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    &nv.value,
                                    "prefer must be one of: auto, list, externalptr, vector/native",
                                ));
                            }
                        };
                    } else if nv.path.is_ident("dots") {
                        // dots = typed_list!(...) - capture the macro invocation
                        // Store span for error reporting
                        dots_span = Some(nv.path.span());
                        if let syn::Expr::Macro(expr_macro) = &nv.value {
                            if expr_macro.mac.path.is_ident("typed_list") {
                                // Capture the entire macro invocation as TokenStream
                                dots_spec = Some(quote::quote!(#expr_macro));
                            } else {
                                return Err(syn::Error::new_spanned(
                                    &expr_macro.mac.path,
                                    "dots expects `typed_list!(...)` macro",
                                ));
                            }
                        } else {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "dots expects `typed_list!(...)` macro",
                            ));
                        }
                    } else if nv.path.is_ident("lifecycle") {
                        // lifecycle = "stage"
                        if let Some(spec) = crate::lifecycle::parse_lifecycle_attr(
                            &syn::Meta::NameValue(nv.clone()),
                        )? {
                            lifecycle = Some(spec);
                        }
                    } else if nv.path.is_ident("doc") {
                        doc = Some(parse_lit_str(&nv, "doc")?);
                    } else if nv.path.is_ident("c_symbol") {
                        let val = parse_lit_str(&nv, "c_symbol")?;
                        if val.is_empty()
                            || (!val.starts_with(|c: char| c.is_ascii_alphabetic())
                                && !val.starts_with('_'))
                        {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "c_symbol must be a valid C identifier",
                            ));
                        }
                        if !val.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "c_symbol must be a valid C identifier (alphanumeric and underscore only)",
                            ));
                        }
                        c_symbol = Some(val);
                    } else if nv.path.is_ident("r_name") {
                        let val = parse_lit_str(&nv, "r_name")?;
                        if val.is_empty() {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "r_name must not be empty",
                            ));
                        }
                        r_name = Some(val);
                    } else if nv.path.is_ident("postfix") {
                        let val = parse_lit_str(&nv, "postfix")?;
                        validate_postfix(&val, &nv.value)?;
                        postfix = Some(val);
                    } else if nv.path.is_ident("call") {
                        let is_caller = match &nv.value {
                            syn::Expr::Path(p) => p.path.is_ident("caller"),
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(s),
                                ..
                            }) => s.value() == "caller",
                            _ => false,
                        };
                        if !is_caller {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "`call = ...` accepts only `caller` (attribute conditions to the \
                                 wrapper's caller); the default already attributes them to the \
                                 wrapper's own call",
                            ));
                        }
                        call_caller = true;
                    } else if nv.path.is_ident("r_entry") {
                        r_entry = Some(parse_lit_str(&nv, "r_entry")?);
                    } else if nv.path.is_ident("r_post_checks") {
                        r_post_checks = Some(parse_lit_str(&nv, "r_post_checks")?);
                    } else if nv.path.is_ident("r_on_exit") {
                        // Short form: r_on_exit = "expr" → on.exit(expr, add = TRUE)
                        r_on_exit = Some(ROnExit {
                            expr: parse_lit_str(&nv, "r_on_exit")?,
                            add: true,
                            after: true,
                        });
                    } else {
                        let key_name = nv
                            .path
                            .get_ident()
                            .map(|i| i.to_string())
                            .unwrap_or_default();
                        return Err(syn::Error::new_spanned(
                            nv,
                            format!(
                                "unknown `#[miniextendr]` key-value option `{}`. \
                                 Key-value options are: `prefer = \"...\"`, `dots = typed_list!(...)`, \
                                 `lifecycle = \"...\"`, `doc = \"...\"`, `c_symbol = \"...\"`, \
                                 `r_name = \"...\"`, `postfix = \"...\"`, `call = caller`, `r_entry = \"...\"`, \
                                 `r_post_checks = \"...\"`, \
                                 `r_on_exit = \"...\"`",
                                key_name,
                            ),
                        ));
                    }
                }
                syn::Meta::List(list) => {
                    if list.path.is_ident("defaults") {
                        // Ignore defaults(...) - it's handled by impl method parsing
                        // This allows #[miniextendr(defaults(...))] on impl methods
                    } else if list.path.is_ident("lifecycle") {
                        // lifecycle(stage = "deprecated", when = "0.4.0", ...)
                        if let Some(spec) =
                            crate::lifecycle::parse_lifecycle_attr(&syn::Meta::List(list.clone()))?
                        {
                            lifecycle = Some(spec);
                        }
                    } else if list.path.is_ident("s3") {
                        // Parse s3(generic = "...", class = "...")
                        list.parse_nested_meta(|meta| {
                            if meta.path.is_ident("generic") {
                                let _: syn::Token![=] = meta.input.parse()?;
                                let value: syn::LitStr = meta.input.parse()?;
                                s3_generic = Some(value.value());
                            } else if meta.path.is_ident("class") {
                                let _: syn::Token![=] = meta.input.parse()?;
                                let value: syn::LitStr = meta.input.parse()?;
                                s3_class = Some(value.value());
                            } else {
                                return Err(
                                    meta.error("unknown s3 option; expected `generic` or `class`")
                                );
                            }
                            Ok(())
                        })?;
                        // Validate: s3 requires class (generic can default to function name)
                        if s3_class.is_none() {
                            return Err(syn::Error::new_spanned(
                                &list,
                                "s3(...) requires `class = \"...\"` to specify the S3 class suffix; \
                                 `generic` is optional and defaults to the function name",
                            ));
                        }
                    } else if list.path.is_ident("r_on_exit") {
                        // Long form: r_on_exit(expr = "...", add = false, after = false)
                        let mut expr = None;
                        let mut add = true;
                        let mut after = true;
                        list.parse_nested_meta(|meta| {
                            if meta.path.is_ident("expr") {
                                let _: syn::Token![=] = meta.input.parse()?;
                                let value: syn::LitStr = meta.input.parse()?;
                                expr = Some(value.value());
                            } else if meta.path.is_ident("add") {
                                let _: syn::Token![=] = meta.input.parse()?;
                                let value: syn::LitBool = meta.input.parse()?;
                                add = value.value;
                            } else if meta.path.is_ident("after") {
                                let _: syn::Token![=] = meta.input.parse()?;
                                let value: syn::LitBool = meta.input.parse()?;
                                after = value.value;
                            } else {
                                return Err(meta.error(
                                    "unknown r_on_exit option; expected `expr`, `add`, or `after`",
                                ));
                            }
                            Ok(())
                        })?;
                        let expr = expr.ok_or_else(|| {
                            syn::Error::new_spanned(
                                &list,
                                "r_on_exit(...) requires `expr = \"...\"` specifying the R expression",
                            )
                        })?;
                        r_on_exit = Some(ROnExit { expr, add, after });
                    } else if let Some(ident) = list.path.get_ident() {
                        // Bool-flag parenthesized form (e.g. `strict(true)`) is not
                        // supported — write `strict` alone or `strict = true` instead.
                        let opt_name = ident.to_string();
                        return Err(syn::Error::new_spanned(
                            &list,
                            format!(
                                "`{opt_name}` does not accept parenthesized arguments. \
                                 Use `{opt_name}` alone or `{opt_name} = true/false`.",
                            ),
                        ));
                    } else {
                        // path(something) where path is not a single ident
                        return Err(syn::Error::new_spanned(
                            list,
                            format!(
                                "unrecognized nested option. Nested options are: {FN_NESTED_OPTIONS_HELP}"
                            ),
                        ));
                    }
                }
            }
        }

        // Validate: `internal` and `noexport` are redundant together
        if internal && noexport {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`internal` and `noexport` cannot be used together. \
                 `internal` already suppresses @export and also adds @keywords internal. \
                 Use `internal` alone to mark as internal, or `noexport` alone to only suppress export.",
            ));
        }

        // Validate: `export` conflicts with `noexport` and `internal`
        if export && noexport {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`export` and `noexport` are contradictory.",
            ));
        }
        if export && internal {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`export` and `internal` are contradictory.",
            ));
        }

        // Validate: `r_name` is incompatible with S3 naming (`s3(generic/class)`)
        // Validate: `postfix` derives the wrapper name from the Rust name; it
        // cannot combine with another naming source.
        if postfix.is_some() && r_name.is_some() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`postfix` and `r_name` both set the R wrapper name; use one of them.",
            ));
        }
        if postfix.is_some() && (s3_generic.is_some() || s3_class.is_some()) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`postfix` cannot be used with `s3(generic = ..., class = ...)`. \
                 S3 method names are always `generic.class`.",
            ));
        }

        // Validate: `call = caller` is for internal entry points only, and
        // needs a call slot to point somewhere.
        if call_caller && !(noexport || internal) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`call = caller` attributes conditions to the wrapper's caller, which is only \
                 meaningful for a package-internal entry point; add `noexport` or `internal`.",
            ));
        }
        if call_caller && no_call_attribution == Some(true) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`call = caller` cannot be combined with `no_call_attribution` / `fast`: those \
                 emit `.call = NULL`, so there is no call slot to point at the caller.",
            ));
        }

        if r_name.is_some() && (s3_generic.is_some() || s3_class.is_some()) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`r_name` cannot be used with `s3(generic = ..., class = ...)`. \
                 S3 method names are always `generic.class`. Use `generic` and `class` instead.",
            ));
        }

        Ok(Self {
            force_worker: force_worker.unwrap_or(cfg!(feature = "worker-default")),
            force_invisible,
            check_interrupt,
            coerce_all: coerce_all.unwrap_or(cfg!(feature = "coerce-default")),
            rng,
            unwrap_in_r,
            no_preconditions: no_preconditions.unwrap_or(cfg!(feature = "fast-default")),
            // An explicit `call = caller` overrides the `fast-default` feature's
            // `.call = NULL`: the user asked for attribution.
            no_call_attribution: if call_caller {
                false
            } else {
                no_call_attribution.unwrap_or(cfg!(feature = "fast-default"))
            },
            return_pref,
            return_pref_span,
            s3_generic,
            s3_class,
            dots_spec,
            dots_span,
            lifecycle,
            strict: strict.unwrap_or(cfg!(feature = "strict-default")),
            internal,
            noexport,
            export,
            doc,
            c_symbol,
            r_name,
            postfix,
            call_caller,
            r_entry,
            r_post_checks,
            r_on_exit,
        })
    }
}
// endregion
