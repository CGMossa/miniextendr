//! # `#[derive(RFactor)]` - Enum ↔ R Factor Support
//!
//! This module implements the `#[derive(RFactor)]` macro which generates
//! the `RFactor` trait implementation for C-style enums, enabling automatic
//! conversion between Rust enums and R factors.
//!
//! ## Usage
//!
//! ```ignore
//! #[derive(Copy, Clone, RFactor)]
//! enum Color {
//!     Red,
//!     Green,
//!     Blue,
//! }
//!
//! // Generates impl RFactor for Color, IntoR for Color, TryFromSexp for Color,
//! // and Vec/Option variants.
//! ```
//!
//! ## Attributes
//!
//! - `#[r_factor(rename = "name")]` - Rename a variant's level string
//! - `#[r_factor(rename_all = "snake_case")]` - Rename all variants (snake_case, kebab-case, lower, upper)
//!
//! ```ignore
//! #[derive(Copy, Clone, RFactor)]
//! #[r_factor(rename_all = "snake_case")]
//! enum Status {
//!     InProgress,  // level: "in_progress"
//!     #[r_factor(rename = "done")]
//!     Completed,   // level: "done"
//! }
//! ```
//!
//! ## Interaction Factors
//!
//! For enums wrapping another RFactor type (like R's `interaction()`):
//!
//! ```ignore
//! #[derive(Copy, Clone, RFactor)]
//! enum Supplement { OJ, VC }
//!
//! #[derive(Copy, Clone, RFactor)]
//! #[r_factor(interaction = ["OJ", "VC"])]  // inner type's levels
//! enum SpeciesSupplement {
//!     Setosa(Supplement),
//!     Versicolor(Supplement),
//!     Virginica(Supplement),
//! }
//! // Levels: ["Setosa.OJ", "Setosa.VC", "Versicolor.OJ", ...]
//! ```

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type};

use crate::naming::apply_rename_all;

/// Parsed `#[r_factor(...)]` attributes from an enum or variant.
#[derive(Default)]
struct RFactorAttrs {
    /// Per-variant rename: `#[r_factor(rename = "custom_name")]`.
    rename: Option<String>,
    /// Enum-level rename-all: `#[r_factor(rename_all = "snake_case")]`.
    /// Applied to all variants that don't have an explicit `rename`.
    rename_all: Option<String>,
    /// Inner type's level names for interaction factors:
    /// `#[r_factor(interaction = ["A", "B"])]`.
    /// When present, triggers interaction factor codegen instead of simple factor.
    interaction: Option<Vec<String>>,
    /// Separator between outer and inner level names in interaction factors.
    /// Defaults to `"."`. Specified via `#[r_factor(sep = "_")]`.
    sep: Option<String>,
}

/// Parse `#[r_factor(...)]` attributes from a list of `syn::Attribute`.
///
/// Extracts `rename`, `rename_all`, `interaction`, and `sep` keys.
/// Returns `Err` for unknown attribute keys.
fn parse_r_factor_attrs(attrs: &[syn::Attribute]) -> syn::Result<RFactorAttrs> {
    let mut result = RFactorAttrs::default();

    for attr in attrs {
        if attr.path().is_ident("r_factor") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.rename = Some(value.value());
                } else if meta.path.is_ident("rename_all") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.rename_all = Some(value.value());
                } else if meta.path.is_ident("interaction") {
                    // Parse as array: interaction = ["A", "B", "C"]
                    let _eq: syn::Token![=] = meta.input.parse()?;
                    let content;
                    syn::bracketed!(content in meta.input);
                    let levels: syn::punctuated::Punctuated<syn::LitStr, syn::Token![,]> =
                        content.parse_terminated(|input| input.parse(), syn::Token![,])?;
                    result.interaction = Some(levels.iter().map(|s| s.value()).collect());
                } else if meta.path.is_ident("sep") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.sep = Some(value.value());
                } else {
                    return Err(meta.error("unknown r_factor attribute"));
                }
                Ok(())
            })?;
        }
    }

    Ok(result)
}

/// Main entry point for `#[derive(RFactor)]`.
///
/// Dispatches to either [`derive_simple_factor`] (C-style unit variants) or
/// [`derive_interaction_factor`] (tuple variants wrapping an inner RFactor type),
/// based on whether `#[r_factor(interaction = [...])]` is present.
///
/// Generates:
/// - `impl MatchArg` (string choices for `match.arg`)
/// - `impl RFactor` (1-based level index conversion)
/// - `impl IntoR` (Rust enum -> R factor SEXP)
/// - `impl TryFromSexp` (R factor SEXP -> Rust enum)
///
/// Returns `Err` for structs, unions, or invalid attribute combinations.
pub fn derive_r_factor(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse enum-level attributes
    let attrs = parse_r_factor_attrs(&input.attrs)?;

    // Get enum variants
    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        Data::Struct(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "#[derive(RFactor)] can only be applied to enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "#[derive(RFactor)] can only be applied to enums",
            ));
        }
    };

    // Branch based on whether this is an interaction factor
    if let Some(inner_levels) = &attrs.interaction {
        derive_interaction_factor(
            name,
            &impl_generics,
            &ty_generics,
            where_clause,
            variants,
            inner_levels,
            attrs.sep.as_deref().unwrap_or("."),
            attrs.rename_all.as_deref(),
        )
    } else {
        derive_simple_factor(
            name,
            &impl_generics,
            &ty_generics,
            where_clause,
            variants,
            attrs.rename_all.as_deref(),
        )
    }
}

/// Generate `RFactor`, `MatchArg`, `IntoR`, and `TryFromSexp` impls for simple
/// (unit variant) enums.
///
/// Each variant maps to a 1-based level index and a level name string.
/// Level names are determined by variant ident, optionally transformed by
/// `rename_all` or overridden by per-variant `#[r_factor(rename = "...")]`.
///
/// Uses a `OnceLock`-cached levels SEXP for efficient repeated conversion.
///
/// Returns `Err` if any variant has fields (only C-style enums are supported
/// in the simple path).
fn derive_simple_factor(
    name: &syn::Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
    rename_all: Option<&str>,
) -> syn::Result<TokenStream> {
    let mut level_names = Vec::new();
    let mut variant_idents = Vec::new();

    for variant in variants {
        // Check for fields (only allow unit variants)
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                variant,
                "#[derive(RFactor)] only supports fieldless (C-style) enum variants \
                 (use #[r_factor(interaction = [...])] for tuple variants)",
            ));
        }

        // Parse variant-level attributes
        let var_attrs = parse_r_factor_attrs(&variant.attrs)?;

        // Determine level name
        let level_name = if let Some(r) = var_attrs.rename {
            r
        } else {
            apply_rename_all(&crate::naming::ident_name(&variant.ident), rename_all)
        };

        level_names.push(level_name);
        variant_idents.push(&variant.ident);
    }

    // Generate indices (1-based for R)
    let indices: Vec<i32> = (1..=variant_idents.len() as i32).collect();
    let level_name_strs: Vec<&str> = level_names.iter().map(|s| s.as_str()).collect();

    Ok(quote! {
        impl #impl_generics ::miniextendr_api::match_arg::MatchArg for #name #ty_generics #where_clause {
            const CHOICES: &'static [&'static str] = &[#(#level_name_strs),*];

            fn from_choice(choice: &str) -> Option<Self> {
                match choice {
                    #(#level_name_strs => Some(Self::#variant_idents),)*
                    _ => None,
                }
            }

            fn to_choice(self) -> &'static str {
                match self {
                    #(Self::#variant_idents => #level_name_strs,)*
                }
            }
        }

        impl #impl_generics ::miniextendr_api::RFactor for #name #ty_generics #where_clause {
            fn to_level_index(self) -> i32 {
                match self {
                    #(Self::#variant_idents => #indices,)*
                }
            }

            fn from_level_index(idx: i32) -> Option<Self> {
                match idx {
                    #(#indices => Some(Self::#variant_idents),)*
                    _ => None,
                }
            }
        }

        impl #impl_generics ::miniextendr_api::IntoR for #name #ty_generics #where_clause {
            type Error = std::convert::Infallible;

            fn try_into_sexp(self) -> Result<::miniextendr_api::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }

            unsafe fn try_into_sexp_unchecked(self) -> Result<::miniextendr_api::SEXP, Self::Error> {
                self.try_into_sexp()
            }

            fn into_sexp(self) -> ::miniextendr_api::SEXP {
                static LEVELS_CACHE: ::std::sync::OnceLock<::miniextendr_api::SEXP> =
                    ::std::sync::OnceLock::new();
                let levels = *LEVELS_CACHE.get_or_init(|| {
                    ::miniextendr_api::build_levels_sexp_cached(
                        <Self as ::miniextendr_api::match_arg::MatchArg>::CHOICES
                    )
                });
                ::miniextendr_api::build_factor(&[<Self as ::miniextendr_api::RFactor>::to_level_index(self)], levels)
            }
        }

        impl #impl_generics ::miniextendr_api::TryFromSexp for #name #ty_generics #where_clause {
            type Error = ::miniextendr_api::SexpError;

            fn try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Result<Self, Self::Error> {
                ::miniextendr_api::factor_from_sexp(sexp)
            }
        }
    })
}

/// Generate `RFactor`, `MatchArg`, `IntoR`, and `TryFromSexp` impls for interaction
/// (tuple variant) enums.
///
/// Interaction factors combine an outer enum (the variant) with an inner `RFactor`
/// type, producing combined level names like `"Outer.Inner"`. The level order is
/// outer-varies-slowest (matches R's `interaction(..., lex.order = TRUE)`).
///
/// Generates a compile-time assertion that the specified `inner_levels` match the
/// inner type's `MatchArg::CHOICES`, catching mismatches early.
///
/// All variants must be single-field tuples wrapping the same inner type.
///
/// # Arguments
///
/// * `inner_levels` - The expected level strings of the inner type
///   (from `#[r_factor(interaction = [...])]`)
/// * `sep` - Separator between outer and inner level names (default `"."`)
/// * `rename_all` - Optional rename transformation for outer variant names
#[allow(clippy::too_many_arguments)] // generics plumbing, single call site
fn derive_interaction_factor(
    name: &syn::Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
    inner_levels: &[String],
    sep: &str,
    rename_all: Option<&str>,
) -> syn::Result<TokenStream> {
    let mut outer_names = Vec::new();
    let mut variant_idents = Vec::new();
    let mut inner_type: Option<Type> = None;

    for variant in variants {
        // Require single-field tuple variants
        let field_ty = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                fields.unnamed.first().unwrap().ty.clone()
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    variant,
                    "interaction factors require single-field tuple variants: Variant(InnerType)",
                ));
            }
        };

        // All variants must have the same inner type
        if let Some(ref existing) = inner_type {
            if field_ty != *existing {
                return Err(syn::Error::new_spanned(
                    &variant.fields,
                    "all variants must have the same inner type",
                ));
            }
        } else {
            inner_type = Some(field_ty);
        }

        // Parse variant-level attributes for rename
        let var_attrs = parse_r_factor_attrs(&variant.attrs)?;
        let outer_name = if let Some(r) = var_attrs.rename {
            r
        } else {
            apply_rename_all(&crate::naming::ident_name(&variant.ident), rename_all)
        };

        outer_names.push(outer_name);
        variant_idents.push(&variant.ident);
    }

    let inner_type = inner_type.ok_or_else(|| {
        syn::Error::new_spanned(name, "interaction factor must have at least one variant")
    })?;

    let n_outer = outer_names.len();
    let n_inner = inner_levels.len();

    // Generate combined levels at compile time using concat!
    // Order: outer varies slowest (lex.order style)
    // [Outer1.Inner1, Outer1.Inner2, ..., Outer2.Inner1, ...]
    let mut combined_levels = Vec::new();
    for outer_name in &outer_names {
        for inner_name in inner_levels {
            // Use concat! for compile-time string concatenation
            let combined = format!("{}{}{}", outer_name, sep, inner_name);
            combined_levels.push(combined);
        }
    }
    let combined_level_strs: Vec<&str> = combined_levels.iter().map(|s| s.as_str()).collect();

    // Generate to_level_index match arms
    // Index = outer_idx_0 * n_inner + inner_idx_0 + 1
    let n_inner_lit = n_inner as i32;
    let to_index_arms: Vec<_> = variant_idents
        .iter()
        .enumerate()
        .map(|(outer_idx, var_ident)| {
            let outer_idx_lit = outer_idx as i32;
            quote! {
                Self::#var_ident(inner) => {
                    let inner_idx_0 = <#inner_type as ::miniextendr_api::RFactor>::to_level_index(inner) - 1;
                    #outer_idx_lit * #n_inner_lit + inner_idx_0 + 1
                }
            }
        })
        .collect();

    // Generate from_level_index match arms
    // outer_idx_0 = (idx - 1) / n_inner
    // inner_idx_1 = (idx - 1) % n_inner + 1
    let from_index_arms: Vec<_> = (0..n_outer)
        .map(|outer_idx| {
            let var_ident = &variant_idents[outer_idx];
            let start_idx = (outer_idx * n_inner + 1) as i32;
            let end_idx = ((outer_idx + 1) * n_inner) as i32;
            quote! {
                #start_idx..=#end_idx => {
                    let inner_idx_1 = (idx - 1) % #n_inner_lit + 1;
                    <#inner_type as ::miniextendr_api::RFactor>::from_level_index(inner_idx_1)
                        .map(Self::#var_ident)
                }
            }
        })
        .collect();

    // Generate inner level strings for the const assertion
    let inner_level_strs: Vec<&str> = inner_levels.iter().map(|s| s.as_str()).collect();

    Ok(quote! {
        // Compile-time assertion: verify specified inner levels match the actual inner type's CHOICES.
        // This catches mismatches between the `interaction = [...]` attribute and the inner type.
        const _: () = {
            const ACTUAL: &[&str] = <#inner_type as ::miniextendr_api::match_arg::MatchArg>::CHOICES;
            const EXPECTED: &[&str] = &[#(#inner_level_strs),*];

            // Check level count
            assert!(
                ACTUAL.len() == EXPECTED.len(),
                "interaction factor: inner type level count mismatch"
            );

            // Check each level matches (const string comparison)
            let mut i = 0;
            while i < ACTUAL.len() {
                let actual_bytes = ACTUAL[i].as_bytes();
                let expected_bytes = EXPECTED[i].as_bytes();
                assert!(
                    actual_bytes.len() == expected_bytes.len(),
                    "interaction factor: inner type level string length mismatch"
                );
                let mut j = 0;
                while j < actual_bytes.len() {
                    assert!(
                        actual_bytes[j] == expected_bytes[j],
                        "interaction factor: inner type level string content mismatch"
                    );
                    j += 1;
                }
                i += 1;
            }
        };

        impl #impl_generics ::miniextendr_api::match_arg::MatchArg for #name #ty_generics #where_clause {
            const CHOICES: &'static [&'static str] = &[#(#combined_level_strs),*];

            fn from_choice(choice: &str) -> Option<Self> {
                let idx_1 = Self::CHOICES.iter().position(|&l| l == choice).map(|i| i as i32 + 1)?;
                <Self as ::miniextendr_api::RFactor>::from_level_index(idx_1)
            }

            fn to_choice(self) -> &'static str {
                Self::CHOICES[(<Self as ::miniextendr_api::RFactor>::to_level_index(self) - 1) as usize]
            }
        }

        impl #impl_generics ::miniextendr_api::RFactor for #name #ty_generics #where_clause {
            fn to_level_index(self) -> i32 {
                match self {
                    #(#to_index_arms)*
                }
            }

            fn from_level_index(idx: i32) -> Option<Self> {
                match idx {
                    #(#from_index_arms)*
                    _ => None,
                }
            }
        }

        impl #impl_generics ::miniextendr_api::IntoR for #name #ty_generics #where_clause {
            type Error = std::convert::Infallible;

            fn try_into_sexp(self) -> Result<::miniextendr_api::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }

            unsafe fn try_into_sexp_unchecked(self) -> Result<::miniextendr_api::SEXP, Self::Error> {
                self.try_into_sexp()
            }

            fn into_sexp(self) -> ::miniextendr_api::SEXP {
                static LEVELS_CACHE: ::std::sync::OnceLock<::miniextendr_api::SEXP> =
                    ::std::sync::OnceLock::new();
                let levels = *LEVELS_CACHE.get_or_init(|| {
                    ::miniextendr_api::build_levels_sexp_cached(
                        <Self as ::miniextendr_api::match_arg::MatchArg>::CHOICES
                    )
                });
                ::miniextendr_api::build_factor(&[<Self as ::miniextendr_api::RFactor>::to_level_index(self)], levels)
            }
        }

        impl #impl_generics ::miniextendr_api::TryFromSexp for #name #ty_generics #where_clause {
            type Error = ::miniextendr_api::SexpError;

            fn try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Result<Self, Self::Error> {
                ::miniextendr_api::factor_from_sexp(sexp)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_levels_generation() {
        let input: DeriveInput = syn::parse_quote! {
            #[r_factor(interaction = ["Small", "Large"])]
            enum ColorSize {
                Red(Size),
                Green(Size),
                Blue(Size),
            }
        };

        let result = derive_r_factor(input).unwrap();
        let code = result.to_string();

        // Check that combined levels are generated
        assert!(code.contains("Red.Small"));
        assert!(code.contains("Red.Large"));
        assert!(code.contains("Green.Small"));
        assert!(code.contains("Green.Large"));
        assert!(code.contains("Blue.Small"));
        assert!(code.contains("Blue.Large"));

        // Check that const assertion is generated for level validation
        assert!(code.contains("const _ : () ="));
        assert!(code.contains("ACTUAL"));
        assert!(code.contains("EXPECTED"));
        assert!(code.contains("inner type level count mismatch"));
    }

    #[test]
    fn test_interaction_custom_separator() {
        let input: DeriveInput = syn::parse_quote! {
            #[r_factor(interaction = ["X", "Y"], sep = "_")]
            enum AB {
                A(Inner),
                B(Inner),
            }
        };

        let result = derive_r_factor(input).unwrap();
        let code = result.to_string();

        // Check custom separator
        assert!(code.contains("A_X"));
        assert!(code.contains("A_Y"));
        assert!(code.contains("B_X"));
        assert!(code.contains("B_Y"));
    }

    #[test]
    fn test_interaction_with_rename() {
        let input: DeriveInput = syn::parse_quote! {
            #[r_factor(interaction = ["S", "L"], rename_all = "lower")]
            enum ColorSize {
                Red(Size),
                Green(Size),
            }
        };

        let result = derive_r_factor(input).unwrap();
        let code = result.to_string();

        // Check renamed outer levels
        assert!(code.contains("red.S"));
        assert!(code.contains("red.L"));
        assert!(code.contains("green.S"));
        assert!(code.contains("green.L"));
    }
}
