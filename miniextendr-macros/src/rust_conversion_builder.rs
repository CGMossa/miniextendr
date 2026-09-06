//! Shared utilities for converting R SEXP parameters to Rust types.
//!
//! This module provides a builder for generating Rust conversion code from R SEXP arguments,
//! ensuring consistent behavior across standalone functions and impl methods.

use crate::miniextendr_fn::CoercionMapping;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

/// Builder for generating Rust conversion statements from R SEXP parameters.
///
/// Handles:
/// - Unit types `()` → identity binding
/// - `&Dots` → special wrapper with storage
/// - Slices `&[T]` → TryFromSexp
/// - `&str` → String + Borrow (for worker thread compatibility)
/// - Scalar references → DATAPTR_RO_unchecked
/// - Coercion → extract R native type + TryCoerce
/// - Default → TryFromSexp
pub struct RustConversionBuilder {
    /// Enable coercion for all parameters
    coerce_all: bool,
    /// Parameter names that should use coercion
    coerce_params: Vec<String>,
    /// Enable strict input conversion for lossy types
    strict: bool,
    /// Parameter names with `match_arg + several_ok` — use `match_arg_vec_from_sexp` instead of `TryFromSexp`.
    match_arg_several_ok_params: Vec<String>,
}

impl RustConversionBuilder {
    /// Create a new conversion builder.
    pub fn new() -> Self {
        Self {
            coerce_all: false,
            coerce_params: Vec::new(),
            strict: false,
            match_arg_several_ok_params: Vec::new(),
        }
    }

    /// Enable coercion for all parameters.
    pub fn with_coerce_all(mut self) -> Self {
        self.coerce_all = true;
        self
    }

    /// Add a single parameter name that should use coercion.
    ///
    /// `param_name` is matched against the identifier in the function signature.
    /// Can be called multiple times to add several parameters.
    pub fn with_coerce_param(mut self, param_name: String) -> Self {
        self.coerce_params.push(param_name);
        self
    }

    /// Enable strict input conversion for lossy types (i64/u64/isize/usize + Vec variants).
    pub fn with_strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Mark a parameter as `match_arg + several_ok` — uses `match_arg_vec_from_sexp`
    /// instead of `TryFromSexp` for converting STRSXP → `Vec<EnumType>`.
    pub fn with_match_arg_several_ok(mut self, param_name: String) -> Self {
        self.match_arg_several_ok_params.push(param_name);
        self
    }

    /// Check if a parameter should use coercion.
    ///
    /// Returns `true` if `coerce_all` is set or `param_name` appears in the per-parameter list.
    fn should_coerce(&self, param_name: &str) -> bool {
        self.coerce_all || self.coerce_params.contains(&param_name.to_string())
    }

    /// Generate a conversion expression that returns a tagged condition SEXP on failure.
    ///
    /// The R wrapper inspects `.val` and raises a structured `rust_*` condition; the
    /// `return` happens from inside the C wrapper body before any further conversion.
    ///
    /// - `try_expr`: The `Result<T, E>`-producing expression
    /// - `error_msg`: Human-readable error message for the failure
    /// - `ident`: The binding name for the converted value
    /// - `ty`: The target Rust type (for the `let` binding)
    /// - `span`: Source span for error reporting
    fn conversion_stmt(
        &self,
        try_expr: TokenStream,
        error_msg: &str,
        ident: &syn::Ident,
        ty: &syn::Type,
        span: proc_macro2::Span,
    ) -> TokenStream {
        quote_spanned! {span=>
            let #ident: #ty = match #try_expr {
                Ok(v) => v,
                // SAFETY: emitted into the wrapper's with_r_unwind_protect closure (R main thread).
                Err(e) => return unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                    &format!("{}: {e}", #error_msg),
                    ::miniextendr_api::error_value::kind::CONVERSION,
                    ::core::option::Option::None,
                    Some(__miniextendr_call),
                ) },
            };
        }
    }

    /// Like `conversion_stmt` but without a type annotation on the binding.
    fn conversion_stmt_untyped(
        &self,
        try_expr: TokenStream,
        error_msg: &str,
        ident: &syn::Ident,
        span: proc_macro2::Span,
    ) -> TokenStream {
        quote_spanned! {span=>
            let #ident = match #try_expr {
                Ok(v) => v,
                // SAFETY: emitted into the wrapper's with_r_unwind_protect closure (R main thread).
                Err(e) => return unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                    &format!("{}: {e}", #error_msg),
                    ::miniextendr_api::error_value::kind::CONVERSION,
                    ::core::option::Option::None,
                    Some(__miniextendr_call),
                ) },
            };
        }
    }

    /// Generate conversion statement for a single parameter.
    ///
    /// This is the non-split variant: owned conversions and borrow statements are
    /// concatenated into a single list, suitable for main-thread execution where
    /// everything runs in the same scope.
    ///
    /// - `pat_type`: the typed pattern from the function signature (e.g., `x: i32`).
    /// - `sexp_ident`: the identifier of the raw SEXP variable holding the R argument.
    ///
    /// Returns a flat list of `let` binding statements that convert `sexp_ident` into
    /// the Rust type declared in `pat_type`.
    pub fn build_conversion(
        &self,
        pat_type: &syn::PatType,
        sexp_ident: &syn::Ident,
    ) -> Vec<TokenStream> {
        // `build_conversion` is the single-scope (main-thread) path: every statement
        // runs inside the same `with_r_unwind_protect` closure where the argument
        // SEXPs are live for the whole call. So `&str` can borrow R's CHARSXP pool
        // directly — zero-copy — exactly like `&[T]` already does. The owning-`String`
        // detour only exists to satisfy `Send` when the value must cross the worker
        // boundary (`build_conversion_split`), which never applies here.
        let (owned, borrowed) = self.build_conversion_split_inner(pat_type, sexp_ident, true);
        owned.into_iter().chain(borrowed).collect()
    }

    /// Generate conversion statements split into two phases for worker thread execution.
    ///
    /// For reference types like `&str`, we need to:
    /// 1. Convert SEXP to owned type (String) -- runs on the main thread before the
    ///    worker closure, so the owned value can be moved into the closure.
    /// 2. Borrow from the owned type (`&str`) -- runs inside the worker closure.
    ///
    /// For non-reference types (scalars, `Vec`, etc.) everything goes into the first
    /// phase and the second vec is empty.
    ///
    /// - `pat_type`: the typed pattern from the function signature (e.g., `s: &str`).
    /// - `sexp_ident`: the identifier of the raw SEXP variable holding the R argument.
    ///
    /// Returns `(owned_conversions, borrow_statements)` where each element is a list
    /// of `let` binding token streams.
    pub fn build_conversion_split(
        &self,
        pat_type: &syn::PatType,
        sexp_ident: &syn::Ident,
    ) -> (Vec<TokenStream>, Vec<TokenStream>) {
        // Worker path: `&str` MUST be owned-then-borrowed because a borrowed view
        // over R's CHARSXP pool is `!Send` and cannot move into the worker closure.
        self.build_conversion_split_inner(pat_type, sexp_ident, false)
    }

    /// Inner implementation of [`Self::build_conversion_split`].
    ///
    /// `zero_copy_str` controls how a `&str` argument is lowered:
    /// - `true` (main-thread, single-scope): emit a direct zero-copy `&str` borrow
    ///   over R's CHARSXP pool via `TryFromSexp` — no `String` allocation. Sound
    ///   because the SEXP outlives the borrow inside the `with_r_unwind_protect`
    ///   closure, and the `&str`'s lifetime is tied to that scope (storing it beyond
    ///   the call is a borrow-checker error).
    /// - `false` (worker): convert to an owned `String` on the main thread, then
    ///   borrow `&str` inside the worker closure — the `String` is `Send`, the
    ///   borrowed view is not.
    fn build_conversion_split_inner(
        &self,
        pat_type: &syn::PatType,
        sexp_ident: &syn::Ident,
        zero_copy_str: bool,
    ) -> (Vec<TokenStream>, Vec<TokenStream>) {
        let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
            return (vec![], vec![]);
        };
        let ident = &pat_ident.ident;
        // `r#`-free spelling for synthesized bindings (`__storage_where`).
        let plain = crate::naming::unraw(ident);
        let ty = pat_type.ty.as_ref();

        match ty {
            // Unit type: ()
            // Note: We never generate `mut` on conversion bindings - the user's function
            // has its own parameter binding that will be `mut` if they specified it.
            syn::Type::Tuple(t) if t.elems.is_empty() => {
                let stmt = quote! { let #ident = (); };
                (vec![stmt], vec![])
            }

            // Reference types: &T, &mut T
            syn::Type::Reference(r) => {
                let param_name = crate::naming::ident_name(ident);
                let is_dots = matches!(
                    r.elem.as_ref(),
                    syn::Type::Path(tp)
                        if tp.path.segments.last()
                            .map(|s| s.ident == "Dots")
                            .unwrap_or(false)
                );
                let is_slice = matches!(r.elem.as_ref(), syn::Type::Slice(_));
                let is_str = matches!(
                    r.elem.as_ref(),
                    syn::Type::Path(tp) if tp.path.is_ident("str")
                );

                // &[T] / &mut [T] with match_arg + several_ok:
                // two-phase: pre-call Vec<T>, in-call borrow
                if is_slice
                    && self
                        .match_arg_several_ok_params
                        .contains(&param_name.to_string())
                    && let Some((crate::SeveralOkContainer::BorrowedSlice, inner_ty)) =
                        crate::classify_several_ok_container(ty)
                {
                    let is_mut = r.mutability.is_some();
                    let storage_ident = quote::format_ident!("__storage_{}", plain);
                    let error_msg = format!(
                        "failed to convert parameter '{}' to &{}[{}]: invalid choice",
                        param_name,
                        if is_mut { "mut " } else { "" },
                        quote::quote!(#inner_ty)
                    );
                    let vec_ty: syn::Type = syn::parse_quote!(::std::vec::Vec<#inner_ty>);
                    let span = ty.span();
                    let try_expr = quote_spanned! {span=>
                        ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                    };
                    // Emit owned Vec<T> binding.
                    // For &mut [T] the storage binding needs `mut`.
                    let owned_stmt = if is_mut {
                        // Need `let mut storage_ident: vec_ty = ...`; inline the mut variant.
                        let em = &error_msg;
                        quote_spanned! {span=>
                            let mut #storage_ident: #vec_ty = match #try_expr {
                                Ok(v) => v,
                                // SAFETY: emitted into the wrapper's with_r_unwind_protect closure (R main thread).
                                Err(e) => return unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                                    &format!("{}: {e}", #em),
                                    ::miniextendr_api::error_value::kind::CONVERSION,
                                    ::core::option::Option::None,
                                    Some(__miniextendr_call),
                                ) },
                            };
                        }
                    } else {
                        self.conversion_stmt(try_expr, &error_msg, &storage_ident, &vec_ty, span)
                    };
                    let borrow_stmt = if is_mut {
                        quote_spanned! {span=>
                            let #ident: #ty = &mut #storage_ident;
                        }
                    } else {
                        quote_spanned! {span=>
                            let #ident: #ty = &#storage_ident;
                        }
                    };
                    return (vec![owned_stmt], vec![borrow_stmt]);
                }

                if is_dots {
                    // &Dots: create wrapper with storage (main thread only - requires SEXP)
                    let storage_ident = quote::format_ident!("{}_storage", plain);
                    let stmt = quote! {
                        let #storage_ident = ::miniextendr_api::dots::Dots { inner: #sexp_ident };
                        let #ident = &#storage_ident;
                    };
                    (vec![stmt], vec![])
                } else if is_slice {
                    // &[T]: use TryFromSexp (backed by DATAPTR_RO)
                    let error_msg = format!(
                        "failed to convert parameter '{}' to slice: wrong type or length",
                        ident
                    );
                    let span = ty.span();
                    let try_expr = quote_spanned! {span=>
                        ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                    };
                    let stmt = self.conversion_stmt_untyped(try_expr, &error_msg, ident, span);
                    (vec![stmt], vec![])
                } else if is_str {
                    let span = ty.span();
                    let error_msg = format!(
                        "failed to convert parameter '{}' to string: expected character vector",
                        ident
                    );
                    if zero_copy_str {
                        // Main-thread path: borrow R's CHARSXP pool directly via the
                        // `&'static str` TryFromSexp impl — zero allocation. The SEXP
                        // is a live wrapper argument for the whole call, so the borrow
                        // is sound; its lifetime is tied to this scope, so storing it
                        // beyond the call is a borrow-checker error (no-store guarantee).
                        let try_expr = quote_spanned! {span=>
                            ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                        };
                        let stmt = self.conversion_stmt_untyped(try_expr, &error_msg, ident, span);
                        (vec![stmt], vec![])
                    } else {
                        // Worker path: convert to owned String, then borrow using the
                        // Borrow trait. The String moves into the worker closure (it is
                        // Send); a borrowed view over R's CHARSXP pool is not.
                        let owned_ident = quote::format_ident!("__owned_{}", plain);
                        // Owned conversion: SEXP -> String
                        let string_ty: syn::Type = syn::parse_quote!(String);
                        let try_expr = quote_spanned! {span=>
                            ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                        };
                        let owned_stmt = self.conversion_stmt(
                            try_expr,
                            &error_msg,
                            &owned_ident,
                            &string_ty,
                            span,
                        );
                        // Borrow: String -> &str (using Borrow trait)
                        let borrow_stmt = quote_spanned! {span=>
                            let #ident: &str = ::std::borrow::Borrow::borrow(&#owned_ident);
                        };
                        (vec![owned_stmt], vec![borrow_stmt])
                    }
                } else {
                    // &T for other types: use TryFromSexp for the reference type.
                    let error_msg = format!(
                        "failed to convert parameter '{}' to {}: wrong type",
                        ident,
                        quote!(#ty)
                    );
                    let span = ty.span();
                    let try_expr = quote_spanned! {span=>
                        ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                    };
                    let stmt = self.conversion_stmt(try_expr, &error_msg, ident, ty, span);
                    (vec![stmt], vec![])
                }
            }

            // All other types
            _ => {
                let param_name = crate::naming::ident_name(ident);

                // Strict mode: use checked input helpers for lossy types
                if self.strict
                    && let Some(strict_expr) =
                        crate::return_type_analysis::strict_input_conversion_for_type(
                            ty,
                            sexp_ident,
                            &param_name,
                        )
                {
                    let span = ty.span();
                    let stmt = quote_spanned! {span=>
                        let #ident: #ty = #strict_expr;
                    };
                    return (vec![stmt], vec![]);
                }

                // match_arg + several_ok: use match_arg_vec_from_sexp for container types
                if self
                    .match_arg_several_ok_params
                    .contains(&param_name.to_string())
                    && let Some((container, inner_ty)) = crate::classify_several_ok_container(ty)
                {
                    let span = ty.span();
                    match container {
                        crate::SeveralOkContainer::Vec => {
                            let error_msg = format!(
                                "failed to convert parameter '{}' to Vec<{}>: invalid choice",
                                param_name,
                                quote!(#inner_ty)
                            );
                            let try_expr = quote_spanned! {span=>
                                ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                            };
                            let stmt = self.conversion_stmt(try_expr, &error_msg, ident, ty, span);
                            return (vec![stmt], vec![]);
                        }
                        crate::SeveralOkContainer::BoxedSlice => {
                            let error_msg = format!(
                                "failed to convert parameter '{}' to Box<[{}]>: invalid choice",
                                param_name,
                                quote!(#inner_ty)
                            );
                            let try_expr = quote_spanned! {span=>
                                ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                                    .map(|v| v.into_boxed_slice())
                            };
                            let stmt = self.conversion_stmt(try_expr, &error_msg, ident, ty, span);
                            return (vec![stmt], vec![]);
                        }
                        crate::SeveralOkContainer::Array(n) => {
                            let error_msg = format!(
                                "failed to convert parameter '{}': invalid choice",
                                param_name,
                            );
                            let param_name_lit = &param_name;
                            let span = ty.span();
                            // First extract the Vec via match_arg_vec_from_sexp (handles
                            // match_arg validation + error reporting), then convert length-check
                            // separately via a direct panic (caught by the framework).
                            let vec_ty: syn::Type = syn::parse_quote!(::std::vec::Vec<#inner_ty>);
                            let vec_ident = quote::format_ident!("__vec_{}", plain);
                            let try_expr = quote_spanned! {span=>
                                ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                            };
                            let vec_stmt = self
                                .conversion_stmt(try_expr, &error_msg, &vec_ident, &vec_ty, span);
                            // Length check + array conversion via panic (framework catches panics)
                            let arr_stmt = quote_spanned! {span=>
                                let #ident: #ty = {
                                    if #vec_ident.len() != #n {
                                        panic!(
                                            "parameter `{}`: expected {} values for [_; {}], got {}",
                                            #param_name_lit, #n, #n, #vec_ident.len()
                                        );
                                    }
                                    <[#inner_ty; #n]>::try_from(#vec_ident)
                                        .unwrap_or_else(|_| unreachable!())
                                };
                            };
                            return (vec![vec_stmt, arr_stmt], vec![]);
                        }
                        crate::SeveralOkContainer::BorrowedSlice => {
                            let storage_ident = quote::format_ident!("__storage_{}", plain);
                            let error_msg = format!(
                                "failed to convert parameter '{}' to &[{}]: invalid choice",
                                param_name,
                                quote!(#inner_ty)
                            );
                            let vec_ty: syn::Type = syn::parse_quote!(::std::vec::Vec<#inner_ty>);
                            let try_expr = quote_spanned! {span=>
                                ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                            };
                            let owned_stmt = self.conversion_stmt(
                                try_expr,
                                &error_msg,
                                &storage_ident,
                                &vec_ty,
                                span,
                            );
                            let borrow_stmt = quote_spanned! {span=>
                                let #ident: #ty = &#storage_ident;
                            };
                            return (vec![owned_stmt], vec![borrow_stmt]);
                        }
                    }
                }

                let should_coerce = self.should_coerce(&param_name);
                let coercion_mapping = if should_coerce {
                    CoercionMapping::from_type(ty)
                } else {
                    None
                };

                let span = ty.span();
                let stmt = match coercion_mapping {
                    Some(CoercionMapping::Scalar { r_native, target }) => {
                        let error_msg_convert = format!(
                            "failed to convert parameter '{}' from R: wrong type",
                            param_name
                        );
                        let error_msg_coerce = format!(
                            "failed to coerce parameter '{}' to {}: overflow, NaN, or precision loss",
                            param_name,
                            quote!(#target)
                        );
                        quote_spanned! {span=>
                            let #ident: #target = {
                                let __r_val: #r_native = match ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident) {
                                    Ok(v) => v,
                                    // SAFETY: emitted into the wrapper's with_r_unwind_protect closure (R main thread).
                                    Err(e) => return unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                                        &format!("{}: {e}", #error_msg_convert),
                                        ::miniextendr_api::error_value::kind::CONVERSION,
                                        ::core::option::Option::None,
                                        Some(__miniextendr_call),
                                    ) },
                                };
                                match ::miniextendr_api::TryCoerce::<#target>::try_coerce(__r_val) {
                                    Ok(v) => v,
                                    // SAFETY: emitted into the wrapper's with_r_unwind_protect closure (R main thread).
                                    Err(e) => return unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                                        &format!("{}: {e}", #error_msg_coerce),
                                        ::miniextendr_api::error_value::kind::CONVERSION,
                                        ::core::option::Option::None,
                                        Some(__miniextendr_call),
                                    ) },
                                }
                            };
                        }
                    }
                    Some(CoercionMapping::Vec {
                        r_native_elem,
                        target_elem,
                    }) => {
                        let error_msg_convert = format!(
                            "failed to convert parameter '{}' to vector: wrong type",
                            param_name
                        );
                        // Project principle "collect all errors in vectorized ops":
                        // walk the whole slice and batch every failing element
                        // (indexed) into one diagnostic via the #1192 accumulator
                        // (`BatchedErrors`), rather than short-circuiting at the first.
                        // Baked #1217-PR-A decision: the outer prefix carries only the
                        // parameter name; the container label (`Vec<u32>`) is supplied to
                        // `into_error`, and the trailing "overflow, NaN, or precision
                        // loss" hint is dropped (each per-index `{e}` already says why).
                        let error_msg_coerce =
                            format!("failed to coerce parameter '{}'", param_name);
                        let container_label = format!("Vec<{}>", quote!(#target_elem));
                        quote_spanned! {span=>
                            let #ident: Vec<#target_elem> = {
                                let __r_slice: &[#r_native_elem] = match ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident) {
                                    Ok(v) => v,
                                    // SAFETY: emitted into the wrapper's with_r_unwind_protect closure (R main thread).
                                    Err(e) => return unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                                        &format!("{}: {e}", #error_msg_convert),
                                        ::miniextendr_api::error_value::kind::CONVERSION,
                                        ::core::option::Option::None,
                                        Some(__miniextendr_call),
                                    ) },
                                };
                                let mut __coerced: Vec<#target_elem> = Vec::with_capacity(__r_slice.len());
                                let mut __errors = ::miniextendr_api::from_r::BatchedErrors::default();
                                for (__i, __elem) in __r_slice.iter().copied().enumerate() {
                                    match ::miniextendr_api::TryCoerce::<#target_elem>::try_coerce(__elem) {
                                        Ok(__v) => __coerced.push(__v),
                                        Err(__e) => __errors.push(|| format!("invalid value at index {__i}: {__e}")),
                                    }
                                }
                                if __errors.is_empty() {
                                    __coerced
                                } else {
                                    // `into_error` always yields `SexpError::InvalidValue`; take its
                                    // inner message directly so the outer format doesn't double the
                                    // "invalid value: " prefix that `SexpError`'s Display would add
                                    // (mirrors `collect_coerced`'s per-element unwrap in from_r.rs).
                                    let __batched = match __errors.into_error(#container_label) {
                                        ::miniextendr_api::from_r::SexpError::InvalidValue(__m) => __m,
                                        __other => ::std::string::ToString::to_string(&__other),
                                    };
                                    // SAFETY: emitted into the wrapper's with_r_unwind_protect closure (R main thread).
                                    return unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                                        &format!("{}: {__batched}", #error_msg_coerce),
                                        ::miniextendr_api::error_value::kind::CONVERSION,
                                        ::core::option::Option::None,
                                        Some(__miniextendr_call),
                                    ) };
                                }
                            };
                        }
                    }
                    None => {
                        let error_msg = format!(
                            "failed to convert parameter '{}' to {}: wrong type, length, or contains NA",
                            param_name,
                            quote!(#ty)
                        );
                        let try_expr = quote_spanned! {span=>
                            ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                        };
                        self.conversion_stmt(try_expr, &error_msg, ident, ty, span)
                    }
                };
                (vec![stmt], vec![])
            }
        }
    }

    /// Generate conversion statements for all parameters in a function signature.
    ///
    /// Iterates over `inputs` (the function's parameter list) paired with `sexp_idents`
    /// (the corresponding SEXP variable names), calling [`build_conversion`](Self::build_conversion)
    /// for each typed parameter. Receiver parameters (`self`) are silently skipped.
    ///
    /// Returns a flat list of all conversion statements, in parameter order.
    pub fn build_conversions(
        &self,
        inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
        sexp_idents: &[syn::Ident],
    ) -> Vec<TokenStream> {
        let mut all_statements = Vec::new();

        for (arg, sexp_ident) in inputs.iter().zip(sexp_idents.iter()) {
            if let syn::FnArg::Typed(pat_type) = arg {
                let statements = self.build_conversion(pat_type, sexp_ident);
                all_statements.extend(statements);
            }
        }

        all_statements
    }
}

impl Default for RustConversionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
