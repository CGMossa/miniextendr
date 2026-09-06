//! Parser for the `list!` macro.
//!
//! A simple macro for constructing R lists from Rust values.
//!
//! ```ignore
//! // Named entries (like R's list())
//! list!(
//!     alpha = 1,
//!     beta = "hello",
//!     gamma = vec![1, 2, 3],
//! )
//!
//! // Unnamed entries
//! list!(1, "hello", vec![1, 2, 3])
//!
//! // Mixed (unnamed entries get empty string names)
//! list!(alpha = 1, 2, beta = "hello")
//! ```

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Ident, LitStr, Token};

/// Parsed `list!` macro input containing zero or more entries.
pub struct ListInput {
    /// The list entries, which may be named, unnamed, or a mix.
    pub entries: Vec<ListEntry>,
}

/// A single entry in the list.
pub struct ListEntry {
    /// The name (None for unnamed entries).
    pub name: Option<ListName>,
    /// The value expression.
    pub value: Expr,
}

/// Name for a list entry -- either a Rust identifier or a string literal.
///
/// Identifiers are used for simple names (e.g., `alpha = 1`), while string
/// literals allow names that are not valid Rust identifiers (e.g., `"my-name" = 1`).
pub enum ListName {
    /// A bare Rust identifier used as the entry name (e.g., `alpha`).
    Ident(Ident),
    /// A string literal used as the entry name (e.g., `"my-name"`).
    Str(LitStr),
}

impl ListName {
    /// Returns the name as a plain `String`, extracting the identifier text
    /// or the string literal value.
    fn to_string_value(&self) -> String {
        match self {
            ListName::Ident(ident) => crate::naming::ident_name(ident),
            ListName::Str(lit) => lit.value(),
        }
    }
}

impl Parse for ListInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(ListInput {
                entries: Vec::new(),
            });
        }

        let entries_punct: Punctuated<ListEntry, Token![,]> = Punctuated::parse_terminated(input)?;
        let entries: Vec<ListEntry> = entries_punct.into_iter().collect();

        Ok(ListInput { entries })
    }
}

impl Parse for ListEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Try to parse as `name = value` or `"name" = value` (like R's list())
        // We need to look ahead to see if there's a `=`

        // Check for identifier followed by =
        if input.peek(Ident) && input.peek2(Token![=]) {
            let name: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: Expr = input.parse()?;
            return Ok(ListEntry {
                name: Some(ListName::Ident(name)),
                value,
            });
        }

        // Check for string literal followed by =
        if input.peek(LitStr) && input.peek2(Token![=]) {
            let name: LitStr = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: Expr = input.parse()?;
            return Ok(ListEntry {
                name: Some(ListName::Str(name)),
                value,
            });
        }

        // Otherwise, parse as unnamed value
        let value: Expr = input.parse()?;
        Ok(ListEntry { name: None, value })
    }
}

/// Expands a parsed `list!` invocation into a `List` constructor token stream.
///
/// For fully unnamed lists, generates `List::from_raw_values(...)`.
/// For named or mixed lists, generates `List::from_raw_pairs(...)` where
/// unnamed entries receive empty-string names.
///
/// Each entry value is converted via `IntoR::into_sexp()`.
pub fn expand_list(input: ListInput) -> TokenStream {
    if input.entries.is_empty() {
        // Empty list
        return quote! {
            ::miniextendr_api::list::List::from_raw_values(::std::vec![])
        };
    }

    // Check if all entries are unnamed
    let all_unnamed = input.entries.iter().all(|e| e.name.is_none());

    // Each entry's `into_sexp()` is wrapped in `__scope.protect_raw(...)` so
    // earlier entries stay rooted across later allocations. Without this, the
    // raw `vec![into_sexp(a), into_sexp(b), ...]` form leaves every prior SEXP
    // unrooted across the next allocation — UAF under gctorture
    // (reviews/2026-05-07-gctorture-audit.md).
    if all_unnamed {
        // Use from_raw_values for unnamed lists
        let values: Vec<TokenStream> = input
            .entries
            .into_iter()
            .map(|entry| {
                let value = entry.value;
                quote! {
                    __scope.protect_raw(::miniextendr_api::into_r::IntoR::into_sexp(#value))
                }
            })
            .collect();

        quote! {
            // SAFETY: list! is invoked from `#[miniextendr]` wrappers, which
            // run on the R main thread. The scope unprotects on drop after
            // the parent list has rooted its children via the write barrier.
            unsafe {
                let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
                ::miniextendr_api::list::List::from_raw_values(::std::vec![#(#values),*])
            }
        }
    } else {
        // Use from_raw_pairs for named (or mixed) lists
        let pairs: Vec<TokenStream> = input
            .entries
            .into_iter()
            .map(|entry| {
                let name = entry.name.map(|n| n.to_string_value()).unwrap_or_default(); // Empty string for unnamed
                let value = entry.value;
                quote! {
                    (#name, __scope.protect_raw(::miniextendr_api::into_r::IntoR::into_sexp(#value)))
                }
            })
            .collect();

        quote! {
            // SAFETY: see above.
            unsafe {
                let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
                ::miniextendr_api::list::List::from_raw_pairs(::std::vec![#(#pairs),*])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let input: ListInput = syn::parse_quote!();
        assert!(input.entries.is_empty());
    }

    #[test]
    fn test_parse_unnamed() {
        let input: ListInput = syn::parse_quote!(1, 2, 3);
        assert_eq!(input.entries.len(), 3);
        assert!(input.entries.iter().all(|e| e.name.is_none()));
    }

    #[test]
    fn test_parse_named_ident() {
        let input: ListInput = syn::parse_quote!(alpha = 1, beta = 2);
        assert_eq!(input.entries.len(), 2);
        match &input.entries[0].name {
            Some(ListName::Ident(i)) => assert_eq!(i.to_string(), "alpha"),
            _ => panic!("expected ident name"),
        }
    }

    #[test]
    fn test_parse_named_string() {
        let input: ListInput = syn::parse_quote!("my-name" = 1);
        assert_eq!(input.entries.len(), 1);
        match &input.entries[0].name {
            Some(ListName::Str(s)) => assert_eq!(s.value(), "my-name"),
            _ => panic!("expected string name"),
        }
    }

    #[test]
    fn test_parse_mixed() {
        let input: ListInput = syn::parse_quote!(alpha = 1, 2, beta = 3);
        assert_eq!(input.entries.len(), 3);
        assert!(input.entries[0].name.is_some());
        assert!(input.entries[1].name.is_none());
        assert!(input.entries[2].name.is_some());
    }
}
