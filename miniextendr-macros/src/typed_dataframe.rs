//! Parser and expansion for the `typed_dataframe!` macro.
//!
//! Generates a struct that wraps an R `data.frame`, validates declared
//! columns via `TryFromSexp`, and exposes borrowed per-column accessors.
//!
//! # Syntax
//!
//! ```ignore
//! typed_dataframe! {
//!     /// The shape we accept for the Theoph PK dataset.
//!     pub TheophDf {
//!         subject: i32,
//!         weight: f64,
//!         dose: f64,
//!         flag: Option<i32>,   // optional column
//!     }
//! }
//! ```
//!
//! Expands to a struct `TheophDf` with `TryFromSexp` and per-column
//! accessors (`subject() -> &[i32]`, `flag() -> Option<&[i32]>`).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Ident, Token, Type, TypePath, Visibility};

/// Parsed input for `typed_dataframe!`.
pub struct TypedDataframeInput {
    /// Outer attributes (doc comments etc.) on the struct.
    pub attrs: Vec<Attribute>,
    /// Visibility (e.g. `pub`).
    pub vis: Visibility,
    /// Struct name.
    pub name: Ident,
    /// Declared columns.
    pub fields: Vec<TypedDataframeField>,
    /// If `false`, the input data.frame may not have extra (un-declared) columns.
    /// Default is `true`. Toggle with leading `@exact;` prefix.
    pub allow_extra: bool,
}

/// A single column declaration.
pub struct TypedDataframeField {
    /// Field attributes (doc comments etc.).
    pub attrs: Vec<Attribute>,
    /// Column name (as used in R `names(df)`).
    pub name: Ident,
    /// Element type (e.g. `i32`, `f64`).
    pub elem_ty: Type,
    /// True if the field was declared `Option<T>` (column may be absent).
    pub optional: bool,
}

impl Parse for TypedDataframeInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Optional @exact; prefix for strict mode.
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

        // Parse outer attributes (doc comments etc.).
        let attrs = input.call(Attribute::parse_outer)?;

        // Visibility (pub, pub(crate), or nothing).
        let vis: Visibility = input.parse()?;

        // Struct name.
        let name: Ident = input.parse()?;

        // Brace-delimited field list.
        let content;
        syn::braced!(content in input);

        let fields_punct: Punctuated<TypedDataframeField, Token![,]> =
            Punctuated::parse_terminated(&content)?;
        let fields: Vec<TypedDataframeField> = fields_punct.into_iter().collect();

        if fields.is_empty() {
            return Err(syn::Error::new(
                name.span(),
                "typed_dataframe! requires at least one column declaration",
            ));
        }

        Ok(TypedDataframeInput {
            attrs,
            vis,
            name,
            fields,
            allow_extra,
        })
    }
}

impl Parse for TypedDataframeField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;

        // Detect `Option<T>` and unwrap to the inner type.
        let (elem_ty, optional) = match unwrap_option(&ty) {
            Some(inner) => (inner, true),
            None => (ty, false),
        };

        Ok(TypedDataframeField {
            attrs,
            name,
            elem_ty,
            optional,
        })
    }
}

/// If `ty` is `Option<T>`, return `T`; otherwise `None`.
fn unwrap_option(ty: &Type) -> Option<Type> {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return None;
    };
    // Must be exactly `Option<...>` (last segment) — we don't try to verify
    // the full path because users may write `std::option::Option<T>`.
    let segment = path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    if args.args.len() != 1 {
        return None;
    }
    let syn::GenericArgument::Type(inner) = args.args.first()? else {
        return None;
    };
    Some(inner.clone())
}

/// Expand a parsed `typed_dataframe!` into the generated struct, impls, and
/// per-column accessors.
pub fn expand_typed_dataframe(input: TypedDataframeInput) -> TokenStream {
    let TypedDataframeInput {
        attrs,
        vis,
        name,
        fields,
        allow_extra,
    } = input;

    let struct_name_str = name.to_string();

    // Per-field generated bits.
    let col_fields = fields.iter().map(|f| {
        let col_ident = format_ident!("{}_col", crate::naming::unraw(&f.name));
        if f.optional {
            quote! { #col_ident: ::std::option::Option<::miniextendr_api::SEXP> }
        } else {
            quote! { #col_ident: ::miniextendr_api::SEXP }
        }
    });

    let accessors = fields.iter().map(|f| {
        let method_ident = &f.name;
        let col_ident = format_ident!("{}_col", crate::naming::unraw(&f.name));
        let elem_ty = &f.elem_ty;
        let method_attrs = &f.attrs;
        if f.optional {
            quote! {
                #(#method_attrs)*
                #[inline]
                pub fn #method_ident(&self) -> ::std::option::Option<&[#elem_ty]> {
                    self.#col_ident.map(|col| unsafe {
                        let len = self.nrow;
                        let ptr = <#elem_ty as ::miniextendr_api::RNativeType>::dataptr_mut(col)
                            as *const #elem_ty;
                        if len == 0 { &[] } else { ::std::slice::from_raw_parts(ptr, len) }
                    })
                }
            }
        } else {
            quote! {
                #(#method_attrs)*
                #[inline]
                pub fn #method_ident(&self) -> &[#elem_ty] {
                    unsafe {
                        let len = self.nrow;
                        let ptr = <#elem_ty as ::miniextendr_api::RNativeType>::dataptr_mut(self.#col_ident)
                            as *const #elem_ty;
                        if len == 0 { &[] } else { ::std::slice::from_raw_parts(ptr, len) }
                    }
                }
            }
        }
    });

    // Per-field validation: walk each declared column, batch errors.
    let validation_locals = fields.iter().map(|f| {
        let col_ident = format_ident!("{}_col", crate::naming::unraw(&f.name));
        let name_str = crate::naming::ident_name(&f.name);
        let elem_ty = &f.elem_ty;
        if f.optional {
            quote! {
                let #col_ident: ::std::option::Option<::miniextendr_api::SEXP> = {
                    match view.column_raw(#name_str) {
                        ::std::option::Option::None => ::std::option::Option::None,
                        ::std::option::Option::Some(col) => {
                            let actual = ::miniextendr_api::SexpExt::type_of(&col);
                            let expected = <#elem_ty as ::miniextendr_api::RNativeType>::SEXP_TYPE;
                            if actual != expected {
                                __errs.push(::std::format!(
                                    "column `{}`: expected {} ({:?}), got {:?}",
                                    #name_str,
                                    ::std::stringify!(#elem_ty),
                                    expected,
                                    actual,
                                ));
                                ::std::option::Option::None
                            } else {
                                ::std::option::Option::Some(col)
                            }
                        }
                    }
                };
            }
        } else {
            quote! {
                let #col_ident: ::miniextendr_api::SEXP = {
                    match view.column_raw(#name_str) {
                        ::std::option::Option::None => {
                            __errs.push(::std::format!(
                                "missing required column `{}` (expected {} / {:?})",
                                #name_str,
                                ::std::stringify!(#elem_ty),
                                <#elem_ty as ::miniextendr_api::RNativeType>::SEXP_TYPE,
                            ));
                            ::miniextendr_api::SEXP::nil()
                        }
                        ::std::option::Option::Some(col) => {
                            let actual = ::miniextendr_api::SexpExt::type_of(&col);
                            let expected = <#elem_ty as ::miniextendr_api::RNativeType>::SEXP_TYPE;
                            if actual != expected {
                                __errs.push(::std::format!(
                                    "column `{}`: expected {} ({:?}), got {:?}",
                                    #name_str,
                                    ::std::stringify!(#elem_ty),
                                    expected,
                                    actual,
                                ));
                            }
                            col
                        }
                    }
                };
            }
        }
    });

    let col_idents = fields
        .iter()
        .map(|f| format_ident!("{}_col", crate::naming::unraw(&f.name)));
    let col_idents_for_struct = col_idents.clone();

    let declared_names: Vec<String> = fields
        .iter()
        .map(|f| crate::naming::ident_name(&f.name))
        .collect();
    let declared_count = fields.len();
    let allow_extra_check = if allow_extra {
        quote! {}
    } else {
        quote! {
            // Strict mode: reject any column not declared.
            const __DECLARED: &[&str] = &[#( #declared_names ),*];
            let mut __extras: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
            for col_name in view.names() {
                if !__DECLARED.contains(&col_name.as_str()) {
                    __extras.push(col_name.to_string());
                }
            }
            if !__extras.is_empty() {
                __errs.push(::std::format!(
                    "unexpected extra columns: {:?}",
                    __extras
                ));
            }
        }
    };

    quote! {
        #(#attrs)*
        #[allow(non_snake_case)]
        #vis struct #name {
            sexp: ::miniextendr_api::SEXP,
            nrow: usize,
            #( #col_fields, )*
        }

        impl #name {
            /// Number of rows in the underlying data.frame.
            #[inline]
            pub fn nrow(&self) -> usize { self.nrow }

            /// Number of declared columns.
            #[inline]
            pub fn ncol(&self) -> usize { #declared_count }

            /// The backing SEXP (preserves the data.frame class attribute).
            #[inline]
            pub fn as_sexp(&self) -> ::miniextendr_api::SEXP { self.sexp }

            #( #accessors )*
        }

        impl ::miniextendr_api::from_r::TryFromSexp for #name {
            type Error = ::miniextendr_api::from_r::SexpError;

            fn try_from_sexp(
                sexp: ::miniextendr_api::SEXP,
            ) -> ::std::result::Result<Self, Self::Error> {
                use ::miniextendr_api::SexpExt as _;

                // 1. Must be a data.frame.
                if !sexp.is_data_frame() {
                    return ::std::result::Result::Err(
                        ::miniextendr_api::from_r::SexpError::InvalidValue(
                            ::std::format!(
                                "{}: expected a data.frame, got {:?}",
                                #struct_name_str,
                                sexp.type_of(),
                            )
                        )
                    );
                }

                // 2. Build the view for O(1) name lookup + nrow extraction.
                let view = ::miniextendr_api::dataframe::DataFrame::from_sexp(sexp)
                    .map_err(|e| ::miniextendr_api::from_r::SexpError::InvalidValue(
                        ::std::format!("{}: {}", #struct_name_str, e)
                    ))?;
                let nrow = view.nrow();

                // 3. Walk each declared column, batching all errors.
                let mut __errs: ::std::vec::Vec<::std::string::String> =
                    ::std::vec::Vec::new();

                #( #validation_locals )*

                #allow_extra_check

                if !__errs.is_empty() {
                    return ::std::result::Result::Err(
                        ::miniextendr_api::from_r::SexpError::InvalidValue(
                            ::std::format!(
                                "{}: {}",
                                #struct_name_str,
                                __errs.join("; ")
                            )
                        )
                    );
                }

                ::std::result::Result::Ok(#name {
                    sexp,
                    nrow,
                    #( #col_idents_for_struct, )*
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let input: TypedDataframeInput = syn::parse_quote! {
            pub TheophDf {
                subject: i32,
                weight: f64,
            }
        };
        assert_eq!(input.name.to_string(), "TheophDf");
        assert_eq!(input.fields.len(), 2);
        assert_eq!(input.fields[0].name.to_string(), "subject");
        assert!(!input.fields[0].optional);
        assert!(input.allow_extra);
    }

    #[test]
    fn test_parse_optional() {
        let input: TypedDataframeInput = syn::parse_quote! {
            pub Df { x: i32, y: Option<f64> }
        };
        assert!(!input.fields[0].optional);
        assert!(input.fields[1].optional);
    }

    #[test]
    fn test_parse_exact_mode() {
        let input: TypedDataframeInput = syn::parse_quote! {
            @exact;
            pub Df { x: i32 }
        };
        assert!(!input.allow_extra);
    }

    #[test]
    fn test_parse_with_doc_attrs() {
        let input: TypedDataframeInput = syn::parse_quote! {
            /// Theoph PK shape.
            pub TheophDf {
                /// Subject id.
                subject: i32,
            }
        };
        assert!(!input.attrs.is_empty());
        assert!(!input.fields[0].attrs.is_empty());
    }

    #[test]
    fn test_parse_empty_fails() {
        let result: syn::Result<TypedDataframeInput> = syn::parse2(quote! {
            pub Df {}
        });
        assert!(result.is_err());
    }
}
