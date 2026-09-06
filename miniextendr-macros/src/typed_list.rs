//! Parser for the `typed_list!` macro.
//!
//! Without `typed_list!`, an `#[miniextendr]` function that takes `...` reads
//! the dots as raw `&Dots` and validates field names, types, and required-vs-
//! optional shape **at runtime** inside the function body. `typed_list!`
//! lifts that shape into the macro signature: writing
//! `#[miniextendr(dots = typed_list!(alpha => numeric(4), beta? => "list"))]`
//! generates a `dots_typed` binding with strongly-typed accessors, validates
//! field names/types at the call boundary, and surfaces a single batched
//! diagnostic on shape mismatch instead of one-error-per-field.
//!
//! Parses syntax like:
//! ```ignore
//! typed_list!(
//!     alpha => numeric(4),
//!     beta => "list",
//!     gamma => list(),
//!     delta?,
//!     epsilon
//! )
//! ```
//!
//! Expands to a `TypedListSpec` from miniextendr-api.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Ident, LitInt, LitStr, Token};

/// Parsed typed_list! macro input.
pub struct TypedListInput {
    /// Whether @exact mode is enabled (strict, no extra fields).
    pub allow_extra: bool,
    /// The list entries.
    pub entries: Vec<ParsedEntry>,
}

impl Parse for TypedListInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Check for @exact prefix
        let allow_extra = if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let mode: Ident = input.parse()?;
            if mode != "exact" {
                return Err(syn::Error::new_spanned(
                    mode,
                    "expected `exact` after @; use `@exact;` for strict mode",
                ));
            }
            input.parse::<Token![;]>()?;
            false
        } else {
            true
        };

        // Parse entries separated by commas
        let entries_punct: Punctuated<ParsedEntry, Token![,]> =
            Punctuated::parse_terminated(input)?;
        let entries: Vec<ParsedEntry> = entries_punct.into_iter().collect();

        Ok(TypedListInput {
            allow_extra,
            entries,
        })
    }
}

/// A single entry in the typed list spec.
pub struct ParsedEntry {
    /// The field name.
    pub name: Ident,
    /// Whether this field is optional (marked with `?`).
    pub optional: bool,
    /// The type spec (None = Any).
    pub spec: Option<ParsedTypeSpec>,
}

impl Parse for ParsedEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse name (optionally with ? suffix)
        let name: Ident = input.parse()?;

        let optional = if input.peek(Token![?]) {
            input.parse::<Token![?]>()?;
            true
        } else {
            false
        };

        // Check for => type spec
        let spec = if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            Some(input.parse::<ParsedTypeSpec>()?)
        } else {
            None
        };

        Ok(ParsedEntry {
            name,
            optional,
            spec,
        })
    }
}

/// Parsed type specification from a `typed_list!` entry's `=> type` clause.
pub enum ParsedTypeSpec {
    /// A string literal type spec (e.g., `"numeric"`, `"data.frame"`, `"myclass"`).
    /// Known base types are mapped to `TypeSpec` variants; unknown strings become class names.
    StringLit(String),
    /// A call-like type spec (e.g., `numeric()`, `integer(4)`).
    TypeCall {
        /// The type name (must be one of the validated built-in types).
        name: String,
        /// Optional expected vector length (e.g., `4` in `numeric(4)`). `None` means any length.
        len: Option<usize>,
    },
}

impl Parse for ParsedTypeSpec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            // String literal: "numeric", "integer", "data.frame", etc.
            let lit: LitStr = input.parse()?;
            Ok(ParsedTypeSpec::StringLit(lit.value()))
        } else if input.peek(Ident) {
            // Call-like: numeric(), integer(4), list(), etc.
            let name: Ident = input.parse()?;
            let name_str = name.to_string();

            // Validate it's a known type call
            let valid_types = [
                "numeric",
                "integer",
                "logical",
                "character",
                "raw",
                "complex",
                "list",
                "data_frame",
                "factor",
                "matrix",
                "array",
                "function",
                "environment",
                "null",
                "any",
            ];
            if !valid_types.contains(&name_str.as_str()) {
                return Err(syn::Error::new_spanned(
                    &name,
                    format!(
                        "unknown type `{}`; expected one of: {}, or a string literal for class name",
                        name_str,
                        valid_types.join(", ")
                    ),
                ));
            }

            // Check for optional parens with length
            let len = if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                if content.is_empty() {
                    None
                } else {
                    let lit: LitInt = content.parse()?;
                    let val: usize = lit.base10_parse()?;
                    if !content.is_empty() {
                        return Err(syn::Error::new_spanned(
                            &lit,
                            "expected 0 or 1 argument in type spec",
                        ));
                    }
                    Some(val)
                }
            } else {
                None
            };

            Ok(ParsedTypeSpec::TypeCall {
                name: name_str,
                len,
            })
        } else {
            Err(input.error(
                "expected type specification: string literal or type call like `numeric(4)`",
            ))
        }
    }
}

/// Expands a parsed `typed_list!` invocation into a `TypedListSpec` token stream.
///
/// Converts each [`ParsedEntry`] into a `TypedEntry` constructor call and wraps
/// the result in a `TypedListSpec { entries, allow_extra }` literal.
///
/// Returns a `TokenStream` that evaluates to `TypedListSpec` at runtime.
pub fn expand_typed_list(input: TypedListInput) -> TokenStream {
    let allow_extra = input.allow_extra;

    let entries: Vec<TokenStream> = input
        .entries
        .into_iter()
        .map(|entry| {
            let name = crate::naming::ident_name(&entry.name);
            let optional = entry.optional;
            let spec_tokens = match entry.spec {
                None => quote! { ::miniextendr_api::typed_list::TypeSpec::Any },
                Some(ParsedTypeSpec::StringLit(s)) => type_spec_from_string(&s),
                Some(ParsedTypeSpec::TypeCall { name, len }) => type_spec_from_call(&name, len),
            };

            quote! {
                ::miniextendr_api::typed_list::TypedEntry {
                    name: #name,
                    spec: #spec_tokens,
                    optional: #optional,
                }
            }
        })
        .collect();

    quote! {
        ::miniextendr_api::typed_list::TypedListSpec {
            entries: ::std::vec![#(#entries),*],
            allow_extra: #allow_extra,
        }
    }
}

/// Converts a string literal type specification to `TypeSpec` tokens.
///
/// Recognizes known base type names (e.g., `"numeric"`, `"integer"`, `"logical"`)
/// and their aliases (e.g., `"double"` -> Numeric, `"bool"` -> Logical).
/// Unrecognized strings are treated as R class names via `TypeSpec::Class`.
fn type_spec_from_string(s: &str) -> TokenStream {
    // Check for known base types first
    match s.to_lowercase().as_str() {
        "numeric" | "double" => quote! { ::miniextendr_api::typed_list::TypeSpec::Numeric(None) },
        "integer" | "int" => quote! { ::miniextendr_api::typed_list::TypeSpec::Integer(None) },
        "logical" | "bool" => quote! { ::miniextendr_api::typed_list::TypeSpec::Logical(None) },
        "character" | "string" => {
            quote! { ::miniextendr_api::typed_list::TypeSpec::Character(None) }
        }
        "raw" => quote! { ::miniextendr_api::typed_list::TypeSpec::Raw(None) },
        "complex" => quote! { ::miniextendr_api::typed_list::TypeSpec::Complex(None) },
        "list" => quote! { ::miniextendr_api::typed_list::TypeSpec::List(None) },
        "data.frame" | "dataframe" | "data_frame" => {
            quote! { ::miniextendr_api::typed_list::TypeSpec::DataFrame }
        }
        "factor" => quote! { ::miniextendr_api::typed_list::TypeSpec::Factor },
        "matrix" => quote! { ::miniextendr_api::typed_list::TypeSpec::Matrix },
        "array" => quote! { ::miniextendr_api::typed_list::TypeSpec::Array },
        "function" => quote! { ::miniextendr_api::typed_list::TypeSpec::Function },
        "environment" | "env" => {
            quote! { ::miniextendr_api::typed_list::TypeSpec::Environment }
        }
        "null" => quote! { ::miniextendr_api::typed_list::TypeSpec::Null },
        "any" => quote! { ::miniextendr_api::typed_list::TypeSpec::Any },
        // Any other string is treated as a class name
        _ => quote! { ::miniextendr_api::typed_list::TypeSpec::Class(#s) },
    }
}

/// Converts a call-style type specification (e.g., `numeric(4)`) to `TypeSpec` tokens.
///
/// The `name` must be a validated type name from parsing. The optional `len`
/// specifies the expected vector length constraint (e.g., `numeric(4)` means
/// a numeric vector of exactly length 4).
fn type_spec_from_call(name: &str, len: Option<usize>) -> TokenStream {
    let len_tokens = match len {
        None => quote! { None },
        Some(n) => quote! { Some(#n) },
    };

    match name {
        "numeric" => quote! { ::miniextendr_api::typed_list::TypeSpec::Numeric(#len_tokens) },
        "integer" => quote! { ::miniextendr_api::typed_list::TypeSpec::Integer(#len_tokens) },
        "logical" => quote! { ::miniextendr_api::typed_list::TypeSpec::Logical(#len_tokens) },
        "character" => quote! { ::miniextendr_api::typed_list::TypeSpec::Character(#len_tokens) },
        "raw" => quote! { ::miniextendr_api::typed_list::TypeSpec::Raw(#len_tokens) },
        "complex" => quote! { ::miniextendr_api::typed_list::TypeSpec::Complex(#len_tokens) },
        "list" => quote! { ::miniextendr_api::typed_list::TypeSpec::List(#len_tokens) },
        "data_frame" => quote! { ::miniextendr_api::typed_list::TypeSpec::DataFrame },
        "factor" => quote! { ::miniextendr_api::typed_list::TypeSpec::Factor },
        "matrix" => quote! { ::miniextendr_api::typed_list::TypeSpec::Matrix },
        "array" => quote! { ::miniextendr_api::typed_list::TypeSpec::Array },
        "function" => quote! { ::miniextendr_api::typed_list::TypeSpec::Function },
        "environment" => quote! { ::miniextendr_api::typed_list::TypeSpec::Environment },
        "null" => quote! { ::miniextendr_api::typed_list::TypeSpec::Null },
        "any" => quote! { ::miniextendr_api::typed_list::TypeSpec::Any },
        _ => {
            // Should not happen due to validation in parsing
            quote! { ::miniextendr_api::typed_list::TypeSpec::Any }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_entry() {
        let input: TypedListInput = syn::parse_quote!(alpha);
        assert_eq!(input.entries.len(), 1);
        assert_eq!(input.entries[0].name.to_string(), "alpha");
        assert!(!input.entries[0].optional);
        assert!(input.entries[0].spec.is_none());
        assert!(input.allow_extra);
    }

    #[test]
    fn test_parse_optional_entry() {
        let input: TypedListInput = syn::parse_quote!(alpha?);
        assert!(input.entries[0].optional);
    }

    #[test]
    fn test_parse_string_spec() {
        let input: TypedListInput = syn::parse_quote!(alpha => "numeric");
        match &input.entries[0].spec {
            Some(ParsedTypeSpec::StringLit(s)) => assert_eq!(s, "numeric"),
            _ => panic!("expected string literal spec"),
        }
    }

    #[test]
    fn test_parse_call_spec() {
        let input: TypedListInput = syn::parse_quote!(alpha => numeric(4));
        match &input.entries[0].spec {
            Some(ParsedTypeSpec::TypeCall { name, len }) => {
                assert_eq!(name, "numeric");
                assert_eq!(*len, Some(4));
            }
            _ => panic!("expected type call spec"),
        }
    }

    #[test]
    fn test_parse_call_no_len() {
        let input: TypedListInput = syn::parse_quote!(alpha => list());
        match &input.entries[0].spec {
            Some(ParsedTypeSpec::TypeCall { name, len }) => {
                assert_eq!(name, "list");
                assert_eq!(*len, None);
            }
            _ => panic!("expected type call spec"),
        }
    }

    #[test]
    fn test_parse_exact_mode() {
        let input: TypedListInput = syn::parse_quote!(@exact; alpha);
        assert!(!input.allow_extra);
        assert_eq!(input.entries.len(), 1);
    }

    #[test]
    fn test_parse_multiple_entries() {
        let input: TypedListInput = syn::parse_quote!(alpha => numeric(4), beta? => "list", gamma);
        assert_eq!(input.entries.len(), 3);
        assert!(!input.entries[0].optional);
        assert!(input.entries[1].optional);
        assert!(!input.entries[2].optional);
    }
}
