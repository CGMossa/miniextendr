//! # List and Preference Derive Macros
//!
//! This module implements derive macros for bidirectional Rust struct <-> R list
//! conversion, plus "preference" derives that control how a type is converted to R
//! when returned from `#[miniextendr]` functions.
//!
//! ## List Derives
//!
//! - `#[derive(IntoList)]` -- Rust struct -> R named/unnamed list
//! - `#[derive(TryFromList)]` -- R list -> Rust struct
//!
//! ## Preference Derives
//!
//! These marker derives select the `IntoR` strategy for a type. Only one
//! preference derive should be applied to a given type:
//!
//! - `#[derive(PreferList)]` -- convert via `IntoList::into_list`
//! - `#[derive(PreferExternalPtr)]` -- wrap in `ExternalPtr::new`
//! - `#[derive(PreferDataFrame)]` -- convert via `ColumnSource::into_column_list`
//! - `#[derive(PreferRNativeType)]` -- convert via `AsRNative` wrapper
//!
//! Stacking two of them is a conflict: each derive emits a fixed-name marker const
//! (`prefer_conflict_marker`), so a second `Prefer*` produces a guided
//! "duplicate definitions" error pointing at the call-site `As*` wrappers as the
//! way to choose a representation per return value (see #870).
//!
//! ## Field Attributes
//!
//! - `#[into_list(ignore)]` -- skip this field during IntoList/TryFromList conversion.
//!   For `TryFromList`, ignored fields are filled with `Default::default()`.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, parse_quote, spanned::Spanned};

/// Check whether a struct field has the `#[into_list(ignore)]` attribute.
///
/// Returns `Ok(true)` if the field should be excluded from list conversion,
/// or `Err` if an unknown option is found inside `#[into_list(...)]`.
fn field_is_ignored(field: &syn::Field) -> syn::Result<bool> {
    let mut ignored = false;

    for attr in &field.attrs {
        if !attr.path().is_ident("into_list") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("ignore") {
                ignored = true;
                return Ok(());
            }

            Err(meta.error("unknown #[into_list(...)] option; supported: ignore"))
        })?;
    }

    Ok(ignored)
}

/// Derive `IntoList` for structs (Rust -> R).
///
/// Generates an `impl IntoList for T` that converts the struct into an R list:
/// - Named structs (`struct Foo { x: i32 }`) produce a named R list: `list(x = 1L)`
/// - Tuple structs (`struct Foo(i32, i32)`) produce an unnamed R list: `list(1L, 2L)`
/// - Unit structs (`struct Foo`) produce an empty R list: `list()`
///
/// Fields marked with `#[into_list(ignore)]` are excluded from the list.
/// Each non-ignored field's type must implement `IntoR` (enforced via where-clause bounds).
///
/// Returns `Err` if applied to a non-struct type or if an unknown field attribute is found.
pub fn derive_into_list(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_data = match input.data {
        syn::Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "IntoList can only be derived for structs",
            ));
        }
    };

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut bounds: Vec<syn::WherePredicate> = Vec::new();

    let (destructure_pat, list_construction) = match &struct_data.fields {
        // Named struct: create named R list
        Fields::Named(fields) => {
            let mut names: Vec<String> = Vec::new();
            let mut idents: Vec<syn::Ident> = Vec::new();

            for f in fields.named.iter() {
                let ident = f.ident.as_ref().unwrap().clone();
                if field_is_ignored(f)? {
                    continue;
                }
                let ty = &f.ty;
                bounds.push(parse_quote!(#ty: ::miniextendr_api::into_r::IntoR));
                names.push(ident.to_string());
                idents.push(ident);
            }

            let pat = if idents.is_empty() {
                quote! { { .. } }
            } else {
                quote! { { #(#idents),*, .. } }
            };
            // Use from_raw_pairs to allow heterogeneous field types.
            // Each `into_sexp()` is wrapped in `__scope.protect_raw` so prior
            // field SEXPs survive subsequent allocations — UAF otherwise
            // (reviews/2026-05-07-gctorture-audit.md).
            let construction = quote! {
                // SAFETY: IntoList runs on the R main thread.
                unsafe {
                    let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
                    ::miniextendr_api::list::List::from_raw_pairs(vec![ #( (#names, __scope.protect_raw(#idents.into_sexp())) ),* ])
                }
            };
            (pat, construction)
        }

        // Tuple struct: create unnamed R list (positional access)
        Fields::Unnamed(fields) => {
            let mut pat_elems: Vec<proc_macro2::TokenStream> = Vec::new();
            let mut value_idents: Vec<syn::Ident> = Vec::new();

            for (idx, f) in fields.unnamed.iter().enumerate() {
                if field_is_ignored(f)? {
                    pat_elems.push(quote! { _ });
                    continue;
                }
                let ident = syn::Ident::new(&format!("_field{idx}"), f.span());
                let ty = &f.ty;
                bounds.push(parse_quote!(#ty: ::miniextendr_api::into_r::IntoR));
                pat_elems.push(quote! { #ident });
                value_idents.push(ident);
            }

            let pat = quote! { ( #(#pat_elems),* ) };
            let construction = quote! {
                // SAFETY: see above.
                unsafe {
                    let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
                    ::miniextendr_api::list::List::from_raw_values(vec![ #( __scope.protect_raw(#value_idents.into_sexp()) ),* ])
                }
            };
            (pat, construction)
        }

        // Unit struct: empty list
        Fields::Unit => {
            let pat = quote! {};
            let construction = quote! {
                ::miniextendr_api::list::List::from_raw_values(vec![])
            };
            (pat, construction)
        }
    };

    // Extend where-clause with bounds
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: <syn::Token![where]>::default(),
        predicates: syn::punctuated::Punctuated::new(),
    });
    for b in bounds {
        where_clause.predicates.push(b);
    }

    let expand = quote! {
        impl #impl_generics ::miniextendr_api::list::IntoList for #name #ty_generics #where_clause {
            fn into_list(self) -> ::miniextendr_api::list::List {
                use ::miniextendr_api::into_r::IntoR;
                let Self #destructure_pat = self;
                #list_construction
            }
        }
    };

    Ok(expand)
}

/// Derive `TryFromList` for structs (R -> Rust).
///
/// Generates an `impl TryFromList for T` that extracts struct fields from an R list:
/// - Named structs: extract by field name from a named R list
/// - Tuple structs: extract by position (index 0, 1, 2, ...)
/// - Unit structs: accept any list (no extraction needed)
///
/// Fields marked with `#[into_list(ignore)]` are filled with `Default::default()`.
/// Each non-ignored field's type must implement `TryFromSexp` (enforced via where-clause bounds).
///
/// Returns `Err` if applied to a non-struct type or if an unknown field attribute is found.
pub fn derive_try_from_list(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_data = match input.data {
        syn::Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "TryFromList can only be derived for structs",
            ));
        }
    };

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut bounds: Vec<syn::WherePredicate> = Vec::new();

    let from_list_body = match &struct_data.fields {
        // Named struct: extract by field name
        Fields::Named(fields) => {
            let mut field_extractions: Vec<proc_macro2::TokenStream> = Vec::new();
            let mut field_inits: Vec<proc_macro2::TokenStream> = Vec::new();

            for f in fields.named.iter() {
                let ident = f.ident.as_ref().unwrap().clone();
                let ty = &f.ty;

                if field_is_ignored(f)? {
                    bounds.push(parse_quote!(#ty: ::core::default::Default));
                    field_inits.push(quote! { #ident: ::core::default::Default::default() });
                    continue;
                }

                // Bound on `TryFromSexp` only — not the associated `Error`
                // type — and require the field's error convert into `SexpError`
                // (the trait's error). Pinning `Error = SexpError` rejected any
                // field whose error is `SexpTypeError` etc. — e.g. `Vec<f64>`
                // (#861). The `?`/`map_err` below performs the conversion.
                bounds.push(parse_quote!(#ty: ::miniextendr_api::from_r::TryFromSexp));
                bounds.push(parse_quote!(::miniextendr_api::from_r::SexpError: ::core::convert::From<<#ty as ::miniextendr_api::from_r::TryFromSexp>::Error>));

                let name_str = ident.to_string();
                // Fetch the raw element, then convert — so a present-but-wrong-type
                // field reports the real conversion error instead of being
                // misreported as a missing field.
                field_extractions.push(quote! {
                    let #ident: #ty = {
                        let __elem = list.get_named_sexp(#name_str)
                            .ok_or_else(|| ::miniextendr_api::from_r::SexpError::MissingField(#name_str.into()))?;
                        <#ty as ::miniextendr_api::from_r::TryFromSexp>::try_from_sexp(__elem)
                            .map_err(::miniextendr_api::from_r::SexpError::from)?
                    };
                });
                field_inits.push(quote! { #ident });
            }

            quote! {
                #(#field_extractions)*
                Ok(Self { #(#field_inits),* })
            }
        }

        // Tuple struct: extract by position
        Fields::Unnamed(fields) => {
            let mut field_extractions: Vec<proc_macro2::TokenStream> = Vec::new();
            let mut ctor_args: Vec<proc_macro2::TokenStream> = Vec::new();
            let mut ignored_fields: Vec<bool> = Vec::with_capacity(fields.unnamed.len());
            for f in fields.unnamed.iter() {
                ignored_fields.push(field_is_ignored(f)?);
            }
            let input_fields: usize = ignored_fields.iter().filter(|&&b| !b).count();
            let mut input_idx: usize = 0;

            for (idx, f) in fields.unnamed.iter().enumerate() {
                let ty = &f.ty;

                if ignored_fields[idx] {
                    bounds.push(parse_quote!(#ty: ::core::default::Default));
                    ctor_args.push(quote! { ::core::default::Default::default() });
                    continue;
                }

                let ident = syn::Ident::new(&format!("_field{idx}"), f.span());
                // See the named-struct branch: bound `TryFromSexp` plus a
                // convertibility bound into `SexpError`, not `Error = SexpError`
                // (#861).
                bounds.push(parse_quote!(#ty: ::miniextendr_api::from_r::TryFromSexp));
                bounds.push(parse_quote!(::miniextendr_api::from_r::SexpError: ::core::convert::From<<#ty as ::miniextendr_api::from_r::TryFromSexp>::Error>));

                let idx_isize = input_idx as isize;
                field_extractions.push(quote! {
                    let #ident: #ty = {
                        let __elem = list.get(#idx_isize)
                            .ok_or_else(|| ::miniextendr_api::from_r::SexpError::Length(
                                ::miniextendr_api::from_r::SexpLengthError {
                                    expected: #input_fields,
                                    actual: list.len() as usize,
                                }
                            ))?;
                        <#ty as ::miniextendr_api::from_r::TryFromSexp>::try_from_sexp(__elem)
                            .map_err(::miniextendr_api::from_r::SexpError::from)?
                    };
                });
                ctor_args.push(quote! { #ident });
                input_idx += 1;
            }

            quote! {
                #(#field_extractions)*
                Ok(Self( #(#ctor_args),* ))
            }
        }

        // Unit struct: just return Self
        Fields::Unit => {
            quote! { Ok(Self) }
        }
    };

    // Extend where-clause with bounds
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: <syn::Token![where]>::default(),
        predicates: syn::punctuated::Punctuated::new(),
    });
    for b in bounds {
        where_clause.predicates.push(b);
    }

    let expand = quote! {
        impl #impl_generics ::miniextendr_api::list::TryFromList for #name #ty_generics #where_clause {
            type Error = ::miniextendr_api::from_r::SexpError;

            fn try_from_list(list: ::miniextendr_api::list::List) -> Result<Self, Self::Error> {
                #from_list_body
            }
        }
    };

    Ok(expand)
}

/// Emit a fixed-name conflict marker so that stacking two `Prefer*` derives on a
/// single type produces a *guided* compile error instead of a cryptic `E0119`
/// conflicting-`IntoR`-implementation error.
///
/// Each `Prefer*` derive declares an inherent associated const with the **same**
/// self-describing name in an `impl #name` block. A type carries exactly one
/// representation default, so a second `Prefer*` derive makes rustc report a
/// `duplicate definitions with name ...` error (E0592) — and the duplicated
/// identifier itself spells out the fix: pick one type-level default, or choose a
/// representation per return value at the call site via the `As*` wrappers
/// (`AsList`, `AsExternalPtr`, `AsDataFrame`, ...).
///
/// This fires regardless of derive order and without any cross-derive attribute
/// inspection — each derive only needs to know its own fixed marker name. (The raw
/// E0119 on `IntoR` may still co-fire; the duplicate-marker error is the actionable
/// one because its identifier names both the conflict and the remedy.)
fn prefer_conflict_marker(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Deliberately a NAMED const, not `const _: () = ...`: two stacked `Prefer*`
    // derives must emit the identical name so rustc's duplicate-definition
    // check (E0592) fires. An anonymous `const _` block never collides with
    // itself, which would silently remove this diagnostic — do not "clean up"
    // this into the anonymous form.
    quote! {
        #[allow(non_upper_case_globals, dead_code)]
        impl #impl_generics #name #ty_generics #where_clause {
            /// Stacking two `Prefer*` derives on one type is a conflict: a type has
            /// exactly one `IntoR` default. Keep a single `Prefer*`, or drop them all
            /// and choose a representation per return value with a call-site `As*`
            /// wrapper (`AsList`, `AsExternalPtr`, `AsDataFrame`, ...).
            const __miniextendr_conflicting_Prefer_derives__keep_ONE_or_use_call_site_As_wrappers: () = ();
        }
    }
}

/// Derive `PreferList`: emits an `IntoR` impl that converts to R by first calling
/// `IntoList::into_list`, then `into_sexp`.
///
/// The type must also derive `IntoList` for this to compile. The generated
/// `IntoR::Error` is `Infallible` (list conversion is infallible for valid structs).
pub fn derive_prefer_list(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let conflict_marker = prefer_conflict_marker(&input);

    let expand = quote! {
        impl #impl_generics ::miniextendr_api::into_r::IntoR for #name #ty_generics #where_clause {
            type Error = std::convert::Infallible;

            #[inline]
            fn try_into_sexp(self) -> Result<::miniextendr_api::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }

            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<::miniextendr_api::SEXP, Self::Error> {
                self.try_into_sexp()
            }

            #[inline]
            fn into_sexp(self) -> ::miniextendr_api::SEXP {
                ::miniextendr_api::list::IntoList::into_list(self).into_sexp()
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::SEXP {
                ::miniextendr_api::list::IntoList::into_list(self).into_sexp()
            }
        }

        #conflict_marker
    };

    Ok(expand)
}

/// Derive `PreferExternalPtr`: emits an `IntoR` impl that wraps the value in
/// `ExternalPtr::new` before converting to SEXP.
///
/// The type must implement `TypedExternal` (typically via `#[derive(ExternalPtr)]`).
/// The generated `IntoR::Error` is `Infallible`.
pub fn derive_prefer_externalptr(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let conflict_marker = prefer_conflict_marker(&input);

    let expand = quote! {
        impl #impl_generics ::miniextendr_api::into_r::IntoR for #name #ty_generics #where_clause {
            type Error = std::convert::Infallible;

            #[inline]
            fn try_into_sexp(self) -> Result<::miniextendr_api::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }

            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<::miniextendr_api::SEXP, Self::Error> {
                self.try_into_sexp()
            }

            #[inline]
            fn into_sexp(self) -> ::miniextendr_api::SEXP {
                ::miniextendr_api::externalptr::ExternalPtr::new(self).into_sexp()
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::SEXP {
                ::miniextendr_api::externalptr::ExternalPtr::new(self).into_sexp()
            }
        }

        #conflict_marker
    };

    Ok(expand)
}

/// Derive `PreferDataFrame`: emits an `IntoR` impl that converts to R via
/// `ColumnSource::into_column_list`, then `into_sexp`.
///
/// The type must implement `ColumnSource` (typically the companion struct generated
/// by `#[derive(DataFrameRow)]`). The generated `IntoR::Error` is `Infallible`.
pub fn derive_prefer_data_frame(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let conflict_marker = prefer_conflict_marker(&input);

    let expand = quote! {
        impl #impl_generics ::miniextendr_api::into_r::IntoR for #name #ty_generics #where_clause {
            type Error = std::convert::Infallible;

            #[inline]
            fn try_into_sexp(self) -> Result<::miniextendr_api::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }

            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<::miniextendr_api::SEXP, Self::Error> {
                self.try_into_sexp()
            }

            #[inline]
            fn into_sexp(self) -> ::miniextendr_api::SEXP {
                ::miniextendr_api::convert::ColumnSource::into_column_list(self).into_sexp()
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::SEXP {
                ::miniextendr_api::convert::ColumnSource::into_column_list(self).into_sexp()
            }
        }

        #conflict_marker
    };

    Ok(expand)
}

/// Derive `PreferRNativeType`: emits an `IntoR` impl that wraps the value in
/// `AsRNative(self)` before calling `IntoR::into_sexp`.
///
/// This routes conversion through native R vector allocation, bypassing list/ExternalPtr
/// paths. The type must also implement `RNativeType` for the `AsRNative` wrapper to compile.
/// The generated `IntoR::Error` is `Infallible`.
pub fn derive_prefer_rnative(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let conflict_marker = prefer_conflict_marker(&input);

    let expand = quote! {
        impl #impl_generics ::miniextendr_api::into_r::IntoR for #name #ty_generics #where_clause {
            type Error = std::convert::Infallible;

            #[inline]
            fn try_into_sexp(self) -> Result<::miniextendr_api::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }

            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<::miniextendr_api::SEXP, Self::Error> {
                self.try_into_sexp()
            }

            #[inline]
            fn into_sexp(self) -> ::miniextendr_api::SEXP {
                ::miniextendr_api::into_r::IntoR::into_sexp(
                    ::miniextendr_api::convert::AsRNative(self)
                )
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::SEXP {
                ::miniextendr_api::into_r::IntoR::into_sexp_unchecked(
                    ::miniextendr_api::convert::AsRNative(self)
                )
            }
        }

        #conflict_marker
    };

    Ok(expand)
}

/// Derive `PreferVctrs`: emits an `IntoR` impl that converts the type to its R vctrs object
/// via `IntoVctrs::into_vctrs`.
///
/// Used alongside `#[derive(Vctrs)]` (which supplies the `IntoVctrs` impl) so the type can be
/// returned directly from `#[miniextendr]` functions instead of writing
/// `value.into_vctrs().map_err(...)` by hand. The generated `IntoR::Error` is
/// `VctrsBuildError`; a build failure surfaces in R as an error condition.
#[cfg(feature = "vctrs")]
pub fn derive_prefer_vctrs(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let conflict_marker = prefer_conflict_marker(&input);

    let expand = quote! {
        impl #impl_generics ::miniextendr_api::into_r::IntoR for #name #ty_generics #where_clause {
            type Error = ::miniextendr_api::vctrs::VctrsBuildError;

            #[inline]
            fn try_into_sexp(self) -> Result<::miniextendr_api::SEXP, Self::Error> {
                ::miniextendr_api::vctrs::IntoVctrs::into_vctrs(self)
            }

            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<::miniextendr_api::SEXP, Self::Error> {
                self.try_into_sexp()
            }
        }

        #conflict_marker
    };

    Ok(expand)
}
