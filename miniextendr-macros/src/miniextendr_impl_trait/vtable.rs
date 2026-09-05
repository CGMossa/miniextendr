//! Vtable static generation, C wrapper generation, and method attribute parsing for trait impls.
//!
//! This module contains the core codegen for `#[miniextendr]` on `impl Trait for Type` blocks.
//! It produces the vtable static constant, C-callable wrapper functions for each method and
//! associated constant, and delegates to [`super::r_wrappers`] for R wrapper code generation.

use proc_macro2::TokenStream;
use quote::format_ident;
use syn::ItemImpl;

use super::r_wrappers::{TraitWrapperOpts, generate_trait_r_wrapper};
use super::{TraitConst, TraitMethod, type_to_uppercase_name};
use crate::miniextendr_impl::ClassSystem;

/// Generate the vtable static, C wrappers, R wrappers, and call defs for a trait implementation.
///
/// This is the main entry point for trait impl codegen. For a given
/// `impl Trait for Type { ... }` block, it produces:
///
/// - The cleaned original impl block (with `#[miniextendr]` attrs stripped from methods)
/// - A `static __VTABLE_{CRATE}_{TRAIT}_FOR_{TYPE}: {Trait}VTable` constant
/// - C wrapper functions and `R_CallMethodDef` entries for each method and const
/// - R wrapper code string (class-system specific)
/// - Two `const` items: `{TYPE}_{TRAIT}_CALL_DEFS` and `R_WRAPPERS_{TYPE}_{TRAIT}_IMPL`
///
/// # Arguments
///
/// - `impl_item`: The parsed `impl Trait for Type` block
/// - `trait_path`: Full path to the trait (e.g., `crate::Counter`)
/// - `concrete_type`: The implementing type (e.g., `MyCounter`)
/// - `class_system`: Which R class system (env, r6, s3, s4, s7) to generate wrappers for
/// - `blanket`: If true, skip emitting the impl block (a blanket impl already provides it)
/// - `internal`: If true, add `@keywords internal` to R documentation
/// - `noexport`: If true, suppress `@export` in R documentation
pub(super) fn generate_vtable_static(
    impl_item: &ItemImpl,
    trait_path: &syn::Path,
    concrete_type: &syn::Type,
    class_system: ClassSystem,
    blanket: bool,
    internal: bool,
    noexport: bool,
) -> TokenStream {
    // Extract trait name for naming
    let Some(trait_name) = trait_path.segments.last().map(|s| &s.ident) else {
        return syn::Error::new_spanned(trait_path, "trait path must have at least one segment")
            .into_compile_error();
    };

    // Extract type args from trait path's last segment
    // e.g., for `RExtend<i32>`, extract `[i32]`; for `RMakeIter<i32, IterableVecIter>`, extract `[i32, IterableVecIter]`
    let trait_type_args: Vec<syn::Type> = trait_path
        .segments
        .last()
        .and_then(|seg| {
            if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                Some(
                    args.args
                        .iter()
                        .filter_map(|arg| {
                            if let syn::GenericArgument::Type(ty) = arg {
                                Some(ty.clone())
                            } else {
                                None
                            }
                        })
                        .collect(),
                )
            } else {
                None
            }
        })
        .unwrap_or_default();

    // Extract type identifier (for simple types)
    let type_ident = match concrete_type {
        syn::Type::Path(type_path) => {
            let Some(last_seg) = type_path.path.segments.last() else {
                return syn::Error::new_spanned(
                    concrete_type,
                    "type path must have at least one segment",
                )
                .into_compile_error();
            };
            last_seg.ident.clone()
        }
        _ => format_ident!("Unknown"),
    };

    // Extract type name for naming (simplified - handles Path types)
    let type_name_str = type_to_uppercase_name(concrete_type);
    let trait_name_upper = trait_name.to_string().to_uppercase();
    let trait_name_lower = trait_name.to_string().to_lowercase();

    // Generate names (crate-prefixed vtable static — #1273 — routed through the
    // shared naming.rs helper so this can't drift from the other two
    // reconstruction sites in `miniextendr_impl_trait.rs`)
    let vtable_static_name = crate::naming::vtable_static_ident(&trait_name_upper, &type_name_str);
    let vtable_type_name = format_ident!("{}VTable", trait_name);

    // Build path to vtable builder function
    // If trait is `foo::Counter`, builder is `foo::__counter_build_vtable`
    // Strip type args from the path (builder uses turbofish instead)
    let mut builder_path = trait_path.clone();
    if let Some(last) = builder_path.segments.last_mut() {
        last.ident = format_ident!("__{}_build_vtable", trait_name_lower);
        last.arguments = syn::PathArguments::None;
    }

    // Build the vtable type path (same module as trait)
    // Strip type args (vtable type is not generic)
    let mut vtable_type_path = trait_path.clone();
    if let Some(last) = vtable_type_path.segments.last_mut() {
        last.ident = vtable_type_name.clone();
        last.arguments = syn::PathArguments::None;
    }

    // Parse methods and consts from the impl block
    let all_methods = match extract_methods(impl_item) {
        Ok(m) => m,
        Err(e) => return e.into_compile_error(),
    };
    let consts = extract_consts(impl_item);

    // Separate skipped methods: skipped methods are kept in the emitted impl block
    // but excluded from C wrappers, R wrappers, call defs, and vtable shims.
    let methods: Vec<&TraitMethod> = all_methods.iter().filter(|m| !m.skip).collect();

    // Generate C wrappers and call defs for each non-skipped method
    let method_c_wrappers: Vec<TokenStream> = methods
        .iter()
        .map(|m| generate_trait_method_c_wrapper(m, &type_ident, trait_name, trait_path))
        .collect();

    // Generate C wrappers for consts
    let const_c_wrappers: Vec<TokenStream> = consts
        .iter()
        .map(|c| generate_trait_const_c_wrapper(c, &type_ident, trait_name, trait_path))
        .collect();

    // Combine C wrappers
    let c_wrappers: Vec<TokenStream> = method_c_wrappers
        .into_iter()
        .chain(const_c_wrappers)
        .collect();

    // Check if impl block has @noRd doc comment (strip @param from class-level tags)
    let raw_impl_tags = crate::roxygen::roxygen_tags_from_attrs(&impl_item.attrs);
    let (impl_doc_tags, param_warnings) = crate::roxygen::strip_method_tags(
        &raw_impl_tags,
        &type_ident.to_string(),
        crate::roxygen::next_impl_tag_block_id(),
        impl_item.impl_token.span,
    );
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(&impl_doc_tags, "noRd");

    // Generate R wrapper code string based on class system (only non-skipped methods)
    let methods_owned: Vec<TraitMethod> = methods.iter().map(|m| (*m).clone()).collect();
    let r_wrapper_string = match generate_trait_r_wrapper(
        &type_ident,
        trait_name,
        &methods_owned,
        &consts,
        TraitWrapperOpts {
            class_system,
            class_has_no_rd,
            internal,
            noexport,
        },
    ) {
        Ok(s) => s,
        Err(e) => return e.into_compile_error(),
    };

    // Generate constant name for R wrapper registration
    let r_wrappers_const = format_ident!(
        "R_WRAPPERS_{}_{}_IMPL",
        type_ident.to_string().to_uppercase(),
        trait_name_upper
    );

    // Generate trait dispatch entry name
    let dispatch_entry_name = format_ident!(
        "__MX_DISPATCH_{}_{}_FOR_{}",
        trait_name_upper,
        type_ident.to_string().to_uppercase(),
        type_name_str
    );

    // Build TAG path for the trait (same module as trait, e.g. counter::TAG_COUNTER)
    let mut trait_tag_path = trait_path.clone();
    if let Some(last) = trait_tag_path.segments.last_mut() {
        last.ident = format_ident!("TAG_{}", trait_name_upper);
        last.arguments = syn::PathArguments::None;
    }

    // Format R wrapper as raw string literal
    let r_wrapper_str = crate::r_wrapper_raw_literal(&r_wrapper_string);
    let source_loc_doc = crate::source_location_doc(type_ident.span());
    let source_start = type_ident.span().start();
    let source_line_lit = syn::LitInt::new(&source_start.line.to_string(), type_ident.span());
    let source_col_lit =
        syn::LitInt::new(&(source_start.column + 1).to_string(), type_ident.span());

    // Strip #[miniextendr(...)] attrs from methods before emitting,
    // so they don't trigger another macro expansion.
    //
    // Skip emitting the impl block when:
    // - Body is empty (no methods, no consts) — blanket impl provides it
    // - `blanket` flag is set — a blanket impl exists, methods are only for
    //   C wrapper signature extraction, not for actual trait implementation
    let has_items = !all_methods.is_empty() || !consts.is_empty();
    let clean_impl_tokens = if has_items && !blanket {
        let mut clean_impl = impl_item.clone();
        for item in &mut clean_impl.items {
            if let syn::ImplItem::Fn(method) = item {
                method
                    .attrs
                    .retain(|attr| !attr.path().is_ident("miniextendr"));
            }
        }
        quote::quote! { #clean_impl }
    } else {
        quote::quote! {}
    };

    // For generic traits (with type args like <i32>), generate concrete vtable shims
    // and inline vtable construction. For non-generic traits, use the builder function.
    //
    // Both paths emit the static as `pub` with `#[unsafe(no_mangle)]` so that a separate
    // compilation unit (e.g. a WASM codegen path) can reference it by name.
    let vtable_static_tokens = if trait_type_args.is_empty() {
        // Non-generic: use the builder function generated at the trait definition site
        quote::quote! {
            #[unsafe(no_mangle)]
            pub static #vtable_static_name: #vtable_type_path =
                #builder_path::<#concrete_type>();
        }
    } else {
        // Generic: generate concrete vtable shims and inline construction
        // Only non-skipped methods go into vtable shims
        let methods_for_vtable: Vec<TraitMethod> = methods.iter().map(|m| (*m).clone()).collect();
        let concrete_shims = generate_concrete_vtable_shims(
            &methods_for_vtable,
            &type_ident,
            trait_name,
            trait_path,
            concrete_type,
        );
        let vtable_inits: Vec<TokenStream> = methods
            .iter()
            .filter(|m| m.has_self)
            .map(|m| {
                let name = &m.ident;
                let shim_name = crate::naming::vtshim_ident(&type_ident, trait_name, &m.ident);
                quote::quote! { #name: #shim_name }
            })
            .collect();
        quote::quote! {
            #concrete_shims
            #[unsafe(no_mangle)]
            pub static #vtable_static_name: #vtable_type_path = #vtable_type_path {
                #(#vtable_inits),*
            };
        }
    };

    quote::quote! {
        // Pass through the original impl block (with method attrs stripped)
        // — omitted when the body is empty (blanket impl covers it)
        #clean_impl_tokens

        // Warnings for @param tags on trait impl blocks
        #param_warnings

        #[doc = concat!(
            "Vtable for `",
            stringify!(#concrete_type),
            "` implementing `",
            stringify!(#trait_path),
            "`."
        )]
        #[doc = "Generated by `#[miniextendr]` on the trait impl block."]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[doc(hidden)]
        #vtable_static_tokens

        // C wrappers and call method defs for trait methods
        #(#c_wrappers)*

        // R wrapper registration via distributed slice
        #[doc = concat!(
            "R wrapper code for `",
            stringify!(#type_ident),
            "` implementing `",
            stringify!(#trait_name),
            "`."
        )]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[doc(hidden)]
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_R_WRAPPERS), linkme(crate = ::miniextendr_api::linkme))]
        static #r_wrappers_const: ::miniextendr_api::registry::RWrapperEntry =
            ::miniextendr_api::registry::RWrapperEntry {
                priority: ::miniextendr_api::registry::RWrapperPriority::TraitImpl,
                source_file: file!(),
                content: concat!(
                    "# Generated from Rust impl `",
                    stringify!(#trait_name),
                    "` for `",
                    stringify!(#type_ident),
                    "` (",
                    file!(),
                    ":",
                    #source_line_lit,
                    ":",
                    #source_col_lit,
                    ")",
                    #r_wrapper_str
                ),
            };

        // Trait dispatch entry for universal_query
        #[doc = concat!(
            "Trait dispatch entry: `",
            stringify!(#trait_name),
            "` for `",
            stringify!(#type_ident),
            "`."
        )]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[doc(hidden)]
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_TRAIT_DISPATCH), linkme(crate = ::miniextendr_api::linkme))]
        static #dispatch_entry_name: ::miniextendr_api::registry::TraitDispatchEntry =
            ::miniextendr_api::registry::TraitDispatchEntry {
                concrete_tag: ::miniextendr_api::abi::mx_tag_from_path(
                    concat!(module_path!(), "::", stringify!(#type_ident))
                ),
                trait_tag: #trait_tag_path,
                vtable: unsafe {
                    // SAFETY: vtable is a static reference valid for program lifetime
                    ::std::ptr::from_ref(&#vtable_static_name).cast::<::std::os::raw::c_void>()
                },
                vtable_symbol: stringify!(#vtable_static_name),
            };
    }
}

/// Generate concrete vtable shims for a generic trait impl.
///
/// For generic traits (e.g., `RExtend<T>`), shims and the vtable builder function
/// cannot be generated at the trait definition site because where clauses like
/// `Vec<T>: TryFromSexp` cause recursive trait resolution overflow. Instead, we
/// generate fully monomorphized shims at the impl site where `T` is known (e.g., `T = i32`).
///
/// Each instance method gets a concrete `unsafe extern "C"` shim named
/// `__vtshim_{crate}_{Type}__{Trait}__{method}` (crate-prefixed for webR
/// cross-package symbol uniqueness — #1273) that:
/// 1. Checks argument arity
/// 2. Wraps everything in `with_r_unwind_protect`
/// 3. Extracts SEXP arguments to concrete Rust types
/// 4. Calls the method via fully-qualified syntax `<Type as Trait>::method()`
/// 5. Converts the result back to SEXP
///
/// Static methods are skipped (not part of the vtable).
fn generate_concrete_vtable_shims(
    methods: &[TraitMethod],
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    trait_path: &syn::Path,
    concrete_type: &syn::Type,
) -> TokenStream {
    let mut shims = Vec::new();

    for method in methods {
        if !method.has_self {
            continue; // Static methods not in vtable
        }

        let method_ident = &method.ident;
        let shim_name = crate::naming::vtshim_ident(type_ident, trait_name, method_ident);

        // Count non-self parameters
        let param_count = method
            .sig
            .inputs
            .iter()
            .filter(|a| !matches!(a, syn::FnArg::Receiver(_)))
            .count();
        let expected_argc = param_count as i32;

        // Generate argument extraction (concrete types, no generics)
        let arg_extractions: Vec<TokenStream> = method
            .sig
            .inputs
            .iter()
            .filter(|a| !matches!(a, syn::FnArg::Receiver(_)))
            .enumerate()
            .map(|(i, arg)| {
                if let syn::FnArg::Typed(pt) = arg {
                    let name = if let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() {
                        pat_ident.ident.clone()
                    } else {
                        format_ident!("arg{}", i)
                    };
                    let name_str = name.to_string();

                    // Handle &Self params: extract ExternalPtr<ConcreteType>
                    if is_self_ref_type(&pt.ty) {
                        let extptr_name = format_ident!("__extptr_{}", name);
                        quote::quote! {
                            let #extptr_name: ::miniextendr_api::ExternalPtr<#concrete_type> = unsafe {
                                ::miniextendr_api::trait_abi::extract_arg(argc, argv, #i, #name_str)
                            };
                            let #name = &*#extptr_name;
                        }
                    } else {
                        let ty = &pt.ty;
                        quote::quote! {
                            let #name: #ty = unsafe {
                                ::miniextendr_api::trait_abi::extract_arg(argc, argv, #i, #name_str)
                            };
                        }
                    }
                } else {
                    quote::quote! {}
                }
            })
            .collect();

        // Collect param names for the method call
        let param_names: Vec<syn::Ident> = method
            .sig
            .inputs
            .iter()
            .filter(|a| !matches!(a, syn::FnArg::Receiver(_)))
            .enumerate()
            .map(|(i, arg)| {
                if let syn::FnArg::Typed(pt) = arg
                    && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
                {
                    return pat_ident.ident.clone();
                }
                format_ident!("arg{}", i)
            })
            .collect();

        // Generate method call using fully-qualified syntax to avoid ambiguity
        // with generic trait paths like `RExtend<i32>::method()` where `<` would
        // be parsed as a comparison operator in expression position.
        let method_call = if method.is_mut {
            quote::quote! {
                let self_ref = unsafe { &mut *data.cast::<#concrete_type>() };
                <#concrete_type as #trait_path>::#method_ident(self_ref, #(#param_names),*)
            }
        } else {
            quote::quote! {
                let self_ref = unsafe { &*data.cast::<#concrete_type>().cast_const() };
                <#concrete_type as #trait_path>::#method_ident(self_ref, #(#param_names),*)
            }
        };

        // Generate result conversion
        let has_return = match &method.sig.output {
            syn::ReturnType::Default => false,
            syn::ReturnType::Type(_, ty) => {
                !matches!(ty.as_ref(), syn::Type::Tuple(t) if t.elems.is_empty())
            }
        };
        let result_conversion = if has_return {
            quote::quote! {
                unsafe { ::miniextendr_api::trait_abi::to_sexp(result) }
            }
        } else {
            quote::quote! {
                let _ = result;
                unsafe { ::miniextendr_api::trait_abi::nil() }
            }
        };

        let method_name_str = format!("{}::{}", trait_name, method_ident);

        shims.push(quote::quote! {
            #[doc(hidden)]
            #[allow(non_snake_case)]
            unsafe extern "C" fn #shim_name(
                data: *mut ::std::os::raw::c_void,
                argc: i32,
                argv: *const ::miniextendr_api::SEXP,
            ) -> ::miniextendr_api::SEXP {
                unsafe {
                    ::miniextendr_api::trait_abi::check_arity(argc, #expected_argc, #method_name_str);
                }
                // with_r_unwind_protect_shim: returns tagged error SEXP on panic
                // so the View method wrapper can re-panic via repanic_if_rust_error,
                // allowing rust_* class layering (issue #345).
                ::miniextendr_api::unwind_protect::with_r_unwind_protect_shim(|| {
                    #(#arg_extractions)*
                    let result = { #method_call };
                    #result_conversion
                })
            }
        });
    }

    quote::quote! { #(#shims)* }
}

/// Extract all methods from a trait impl block as [`TraitMethod`] structs.
///
/// Parses each `ImplItem::Fn` to determine receiver type, mutability,
/// `#[miniextendr(...)]` attributes (coerce, skip, r_name, defaults, etc.),
/// and roxygen `@param` tags from doc comments.
fn extract_methods(impl_item: &ItemImpl) -> syn::Result<Vec<TraitMethod>> {
    let mut methods = Vec::new();
    for item in &impl_item.items {
        if let syn::ImplItem::Fn(method) = item {
            // Check receiver type
            let (has_self, is_mut) = method.sig.inputs.first().map_or((false, false), |arg| {
                if let syn::FnArg::Receiver(r) = arg {
                    (true, r.mutability.is_some())
                } else {
                    (false, false)
                }
            });
            let attrs = parse_trait_method_attrs(&method.attrs)?;

            // Extract @param tags (and a per-method @rdname override) from
            // method doc comments
            let all_tags = crate::roxygen::roxygen_tags_from_attrs(&method.attrs);
            let rdname = crate::roxygen::rdname_value(&all_tags).map(str::to_owned);
            let param_tags: Vec<String> = all_tags
                .into_iter()
                .filter(|tag| tag.starts_with("@param"))
                .collect();

            methods.push(TraitMethod {
                ident: method.sig.ident.clone(),
                sig: method.sig.clone(),
                has_self,
                is_mut,
                worker: attrs.worker,
                unsafe_main_thread: attrs.unsafe_main_thread,
                coerce: attrs.coerce,
                check_interrupt: attrs.check_interrupt,
                rng: attrs.rng,
                unwrap_in_r: attrs.unwrap_in_r,
                param_defaults: attrs.defaults,
                param_tags,
                rdname,
                skip: attrs.skip,
                r_name: attrs.r_name,
                strict: attrs.strict,
                lifecycle: attrs.lifecycle,
                r_entry: attrs.r_entry,
                r_post_checks: attrs.r_post_checks,
                r_on_exit: attrs.r_on_exit,
                no_shortcut: attrs.no_shortcut,
                per_param: attrs.per_param,
            });
        }
    }
    Ok(methods)
}

/// Parsed `#[miniextendr(...)]` attributes for a single trait method.
///
/// Extracted from method-level attributes to control C wrapper behavior,
/// threading, and R wrapper generation.
struct TraitMethodAttrs {
    /// Dispatch to worker thread. Set by explicit `#[miniextendr(worker)]` or `worker-default` feature.
    worker: bool,
    /// Force execution on R's main thread (overrides explicit or feature-selected worker dispatch).
    unsafe_main_thread: bool,
    /// Enable `Rf_coerceVector` for all parameters.
    coerce: bool,
    /// Call `R_CheckUserInterrupt` before the method body.
    check_interrupt: bool,
    /// Wrap the call in `GetRNGstate`/`PutRNGstate` for reproducible random number generation.
    rng: bool,
    /// Return `Result<T, E>` to R without unwrapping (R wrapper receives the result variant).
    unwrap_in_r: bool,
    /// Exclude this method from all generated wrappers (C, R, vtable shims).
    skip: bool,
    /// Parameter default values: keys are parameter names, values are R expressions.
    defaults: std::collections::HashMap<String, String>,
    /// Override the R-facing method name.
    r_name: Option<String>,
    /// Strict output conversion: panic instead of lossy widening for i64/u64/isize/usize.
    strict: bool,
    /// Lifecycle specification for deprecation/experimental status.
    lifecycle: Option<crate::lifecycle::LifecycleSpec>,
    /// R code to inject at the very top of the wrapper body.
    r_entry: Option<String>,
    /// R code to inject after all checks, immediately before `.Call()`.
    r_post_checks: Option<String>,
    /// Register `on.exit()` cleanup code in the R wrapper.
    r_on_exit: Option<crate::miniextendr_fn::ROnExit>,
    /// Opt out of the S7 fast-dispatch shortcut (`#[miniextendr(s7(no_shortcut))]`).
    no_shortcut: bool,
    /// Per-parameter `match_arg`/`choices`/`several_ok` attributes, keyed by
    /// Rust parameter name. See `TraitMethod::per_param`.
    per_param: std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
}

/// Parse `#[miniextendr(...)]` attributes from a trait method.
///
/// Supports two syntax styles:
/// - **Flat**: `#[miniextendr(worker, coerce, rng)]`
/// - **Nested class-system**: `#[miniextendr(env(worker, coerce))]`
///
/// Both styles can coexist. The `worker` flag controls whether static methods
/// dispatch to the worker thread (defaults to `cfg!(feature = "worker-default")`).
fn parse_trait_method_attrs(attrs: &[syn::Attribute]) -> syn::Result<TraitMethodAttrs> {
    let mut worker = false;
    let mut unsafe_main_thread = false;
    let mut coerce = false;
    let mut check_interrupt = false;
    let mut rng = false;
    let mut unwrap_in_r = false;
    let mut skip = false;
    let mut strict = false;
    let mut defaults = std::collections::HashMap::new();
    let mut r_name: Option<String> = None;
    let mut lifecycle: Option<crate::lifecycle::LifecycleSpec> = None;
    let mut r_entry: Option<String> = None;
    let mut r_post_checks: Option<String> = None;
    let mut r_on_exit: Option<crate::miniextendr_fn::ROnExit> = None;
    let mut no_shortcut = false;
    let mut per_param: std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs> =
        std::collections::HashMap::new();

    for attr in attrs {
        if !attr.path().is_ident("miniextendr") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            let is_class_meta = meta.path.is_ident("env")
                || meta.path.is_ident("r6")
                || meta.path.is_ident("s7")
                || meta.path.is_ident("s3")
                || meta.path.is_ident("s4");

            if is_class_meta {
                meta.parse_nested_meta(|inner| {
                    if inner.path.is_ident("worker") {
                        worker = true;
                    } else if inner.path.is_ident("main_thread") {
                        unsafe_main_thread = true;
                    } else if inner.path.is_ident("coerce") {
                        coerce = true;
                    } else if inner.path.is_ident("check_interrupt") {
                        check_interrupt = true;
                    } else if inner.path.is_ident("unwrap_in_r") {
                        unwrap_in_r = true;
                    } else if inner.path.is_ident("no_shortcut") {
                        no_shortcut = true;
                    } else {
                        return Err(inner.error(
                            "unknown nested option; expected `worker`, `main_thread`, `coerce`, \
                             `check_interrupt`, `unwrap_in_r`, or `no_shortcut`",
                        ));
                    }
                    Ok(())
                })?;
            } else if meta.path.is_ident("worker") {
                worker = true;
            } else if meta.path.is_ident("main_thread") {
                unsafe_main_thread = true;
            } else if meta.path.is_ident("coerce") {
                coerce = true;
            } else if meta.path.is_ident("check_interrupt") {
                check_interrupt = true;
            } else if meta.path.is_ident("rng") {
                rng = true;
            } else if meta.path.is_ident("unwrap_in_r") {
                unwrap_in_r = true;
            } else if meta.path.is_ident("skip") {
                skip = true;
            } else if meta.path.is_ident("no_shortcut") {
                no_shortcut = true;
            } else if meta.path.is_ident("r_name") {
                let value: syn::LitStr = meta.value()?.parse()?;
                r_name = Some(value.value());
            } else if meta.path.is_ident("strict") {
                strict = true;
            } else if meta.path.is_ident("defaults") {
                // Parse defaults(param = "value", param2 = "value2", ...)
                meta.parse_nested_meta(|inner| {
                    let param_name = inner
                        .path
                        .get_ident()
                        .map(|i| i.to_string())
                        .unwrap_or_default();
                    let value: syn::LitStr = inner.value()?.parse()?;
                    defaults.insert(param_name, value.value());
                    Ok(())
                })?;
            } else if meta.path.is_ident("lifecycle") {
                if meta.input.peek(syn::Token![=]) {
                    // lifecycle = "stage"
                    let _: syn::Token![=] = meta.input.parse()?;
                    let value: syn::LitStr = meta.input.parse()?;
                    let stage = crate::lifecycle::LifecycleStage::from_str(&value.value())
                        .ok_or_else(|| {
                            syn::Error::new(
                                value.span(),
                                "invalid lifecycle stage; expected one of: experimental, stable, superseded, soft-deprecated, deprecated, defunct",
                            )
                        })?;
                    lifecycle = Some(crate::lifecycle::LifecycleSpec::new(stage));
                } else {
                    // lifecycle(stage = "deprecated", when = "0.4.0", ...)
                    let mut spec = crate::lifecycle::LifecycleSpec::default();
                    meta.parse_nested_meta(|inner| {
                        let key = inner.path.get_ident()
                            .ok_or_else(|| inner.error("expected identifier"))?
                            .to_string();
                        let _: syn::Token![=] = inner.input.parse()?;
                        let value: syn::LitStr = inner.input.parse()?;
                        match key.as_str() {
                            "stage" => {
                                spec.stage = crate::lifecycle::LifecycleStage::from_str(&value.value())
                                    .ok_or_else(|| syn::Error::new(value.span(), "invalid lifecycle stage"))?;
                            }
                            "when" => spec.when = Some(value.value()),
                            "what" => spec.what = Some(value.value()),
                            "with" => spec.with = Some(value.value()),
                            "details" => spec.details = Some(value.value()),
                            "id" => spec.id = Some(value.value()),
                            _ => return Err(inner.error(
                                "unknown lifecycle option; expected: stage, when, what, with, details, id"
                            )),
                        }
                        Ok(())
                    })?;
                    lifecycle = Some(spec);
                }
            } else if meta.path.is_ident("r_entry") {
                let _: syn::Token![=] = meta.input.parse()?;
                let value: syn::LitStr = meta.input.parse()?;
                r_entry = Some(value.value());
            } else if meta.path.is_ident("r_post_checks") {
                let _: syn::Token![=] = meta.input.parse()?;
                let value: syn::LitStr = meta.input.parse()?;
                r_post_checks = Some(value.value());
            } else if meta.path.is_ident("r_on_exit") {
                if meta.input.peek(syn::Token![=]) {
                    // Short form: r_on_exit = "expr"
                    let _: syn::Token![=] = meta.input.parse()?;
                    let value: syn::LitStr = meta.input.parse()?;
                    r_on_exit = Some(crate::miniextendr_fn::ROnExit {
                        expr: value.value(),
                        add: true,
                        after: true,
                    });
                } else {
                    // Long form: r_on_exit(expr = "...", add = false, after = false)
                    let mut expr = None;
                    let mut add = true;
                    let mut after = true;
                    meta.parse_nested_meta(|inner| {
                        if inner.path.is_ident("expr") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            expr = Some(value.value());
                        } else if inner.path.is_ident("add") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitBool = inner.input.parse()?;
                            add = value.value;
                        } else if inner.path.is_ident("after") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitBool = inner.input.parse()?;
                            after = value.value;
                        } else {
                            return Err(inner.error(
                                "unknown r_on_exit option; expected `expr`, `add`, or `after`",
                            ));
                        }
                        Ok(())
                    })?;
                    let expr = expr.ok_or_else(|| {
                        meta.error("r_on_exit(...) requires `expr = \"...\"` specifying the R expression")
                    })?;
                    r_on_exit = Some(crate::miniextendr_fn::ROnExit { expr, add, after });
                }
            } else if meta.path.is_ident("choices") {
                // `choices(param = "a, b, c", param2 = "x, y")` — explicit string choice lists.
                //
                // NOTE: `match_arg`/`match_arg_several_ok` (choices derived from a
                // `#[derive(MatchArg)]` enum's `CHOICES` const) are intentionally NOT
                // supported here yet. The inherent-impl path additionally emits a
                // `__match_arg_choices__<param>` C helper + `MX_MATCH_ARG_CHOICES`
                // registration (`generate_method_match_arg_helpers` in
                // `miniextendr_impl.rs`) that resolves the write-time placeholder
                // formal default into the enum's literal choice list — trait methods
                // have no equivalent C-wrapper generation yet, so accepting `match_arg`
                // here would silently ship a formal default that never resolves
                // (`object '.__MX_MATCH_ARG_CHOICES_...__' not found` at call time).
                // `choices(...)` needs no such resolution (the literal list is baked in
                // at macro-expansion time), so it's safe to support today. Tracked in
                // the trait-method-emitter follow-up issue for full match_arg parity.
                meta.parse_nested_meta(|inner| {
                    let name = inner
                        .path
                        .get_ident()
                        .ok_or_else(|| inner.error("expected parameter name"))?
                        .to_string();
                    let _: syn::Token![=] = inner.input.parse()?;
                    let value: syn::LitStr = inner.input.parse()?;
                    let choices = crate::r_wrapper_builder::split_choice_list(&value.value());
                    per_param.entry(name).or_default().choices = Some(choices);
                    Ok(())
                })?;
            } else if meta.path.is_ident("choices_several_ok") {
                // `choices_several_ok(param = "a, b, c")` — choices + several_ok.
                meta.parse_nested_meta(|inner| {
                    let name = inner
                        .path
                        .get_ident()
                        .ok_or_else(|| inner.error("expected parameter name"))?
                        .to_string();
                    let _: syn::Token![=] = inner.input.parse()?;
                    let value: syn::LitStr = inner.input.parse()?;
                    let choices = crate::r_wrapper_builder::split_choice_list(&value.value());
                    let entry = per_param.entry(name).or_default();
                    entry.choices = Some(choices);
                    entry.several_ok = true;
                    Ok(())
                })?;
            } else {
                return Err(meta.error(
                    "unknown #[miniextendr] option on trait impl method; expected one of: \
                     `env`, `r6`, `s7`, `s3`, `s4`, `worker`, `main_thread`, `coerce`, \
                     `check_interrupt`, `rng`, `unwrap_in_r`, `skip`, `no_shortcut`, `r_name`, \
                     `defaults`, `strict`, `lifecycle`, `r_entry`, `r_post_checks`, `r_on_exit`, \
                     `choices`, `choices_several_ok`",
                ));
            }
            Ok(())
        })?;
    }

    Ok(TraitMethodAttrs {
        worker: worker || cfg!(feature = "worker-default"),
        unsafe_main_thread,
        coerce,
        check_interrupt,
        rng,
        unwrap_in_r,
        skip,
        strict,
        defaults,
        r_name,
        lifecycle,
        r_entry,
        r_post_checks,
        r_on_exit,
        no_shortcut,
        per_param,
    })
}

/// Extract associated constant items from a trait impl block.
///
/// Each `const NAME: Type = value;` in the impl block becomes a [`TraitConst`]
/// that will get its own zero-argument C wrapper for R access.
fn extract_consts(impl_item: &ItemImpl) -> Vec<TraitConst> {
    impl_item
        .items
        .iter()
        .filter_map(|item| {
            if let syn::ImplItem::Const(const_item) = item {
                Some(TraitConst {
                    ident: const_item.ident.clone(),
                    ty: const_item.ty.clone(),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Check if a type is `&Self` or `&mut Self`.
///
/// Used to detect trait method parameters that take another instance of the same type
/// (e.g., `ROrd::cmp(&self, other: &Self)`) so we can generate `ExternalPtr<T>` extraction
/// and dereference in the C wrapper.
pub(super) fn is_self_ref_type(ty: &syn::Type) -> bool {
    if let syn::Type::Reference(r) = ty
        && let syn::Type::Path(tp) = r.elem.as_ref()
        && tp.path.is_ident("Self")
    {
        return true;
    }
    false
}

/// Generate a C wrapper function and `R_CallMethodDef` for a single trait method.
///
/// Uses `CWrapperContext` builder to produce:
/// - An `extern "C"` function callable from R via `.Call()`
/// - A `R_CallMethodDef` constant for symbol registration
///
/// Instance methods (`has_self`) extract the object via `ErasedExternalPtr::from_sexp`
/// and call with fully-qualified trait syntax `<Type as Trait>::method(self_ref, ...)`.
/// Static methods run on the worker thread when the `worker` flag is set.
///
/// `&Self` parameters are rewritten to `ExternalPtr<Type>` for the C wrapper,
/// then dereferenced to `&Type` when calling the actual trait method.
pub(super) fn generate_trait_method_c_wrapper(
    method: &TraitMethod,
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    trait_path: &syn::Path,
) -> TokenStream {
    use crate::c_wrapper_builder::{CWrapperContext, ReturnHandling, ThreadStrategy};

    let method_ident = &method.ident;
    let c_ident = method.c_wrapper_ident(type_ident, trait_name);
    let call_method_def_ident = method.call_method_def_ident(type_ident, trait_name);

    // Thread strategy: instance methods stay on main thread (self_ref can't cross threads);
    // static methods use worker thread only when worker=true (explicit or worker-default feature)
    let thread_strategy = if method.has_self || method.unsafe_main_thread {
        ThreadStrategy::MainThread
    } else if method.worker {
        ThreadStrategy::WorkerThread
    } else {
        ThreadStrategy::MainThread
    };

    // Build rust argument names from the signature (excluding self receiver)
    let rust_args: Vec<syn::Ident> = method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg {
                if let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() {
                    Some(pat_ident.ident.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Filter inputs to exclude the receiver (builder handles self separately with has_self())
    // Also handle &Self params: replace with ExternalPtr<ConcreteType> so the builder
    // can auto-extract them, and track which params need dereferencing in the call.
    let mut self_ref_params = std::collections::HashSet::new();
    let filtered_inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]> = method
        .sig
        .inputs
        .iter()
        .filter(|arg| !matches!(arg, syn::FnArg::Receiver(_)))
        .map(|arg| {
            if let syn::FnArg::Typed(pt) = arg
                && is_self_ref_type(&pt.ty)
            {
                // Track this param for dereferencing in the call expression
                if let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() {
                    self_ref_params.insert(pat_ident.ident.to_string());
                }
                // Replace &Self with ExternalPtr<ConcreteType>
                let pat = &pt.pat;
                return syn::parse_quote!(#pat: ::miniextendr_api::ExternalPtr<#type_ident>);
            }
            arg.clone()
        })
        .collect();

    // Build call args: &Self params use `&*param` (deref ExternalPtr), others use `param`
    let call_args: Vec<proc_macro2::TokenStream> = rust_args
        .iter()
        .map(|arg| {
            if self_ref_params.contains(&arg.to_string()) {
                quote::quote! { &*#arg }
            } else {
                quote::quote! { #arg }
            }
        })
        .collect();

    // Determine return handling
    let return_handling = if method.unwrap_in_r && output_is_result(&method.sig.output) {
        ReturnHandling::IntoR
    } else {
        crate::c_wrapper_builder::detect_return_handling(&method.sig.output)
    };

    // Generate R wrapper const name (not actually used but needed by builder)
    let r_wrappers_const = format_ident!(
        "R_WRAPPERS_{}_{}_IMPL",
        type_ident.to_string().to_uppercase(),
        trait_name.to_string().to_uppercase()
    );

    // Build the wrapper using the builder infrastructure
    // Use custom call_method_def_ident to avoid collisions with inherent impl methods
    let mut builder = CWrapperContext::builder(method_ident.clone(), c_ident)
        .r_wrapper_const(r_wrappers_const)
        .inputs(filtered_inputs)
        .output(method.sig.output.clone())
        .thread_strategy(thread_strategy)
        .return_handling(return_handling)
        .type_context(type_ident.clone())
        .call_method_def_ident(call_method_def_ident);

    if method.has_self {
        // Instance method: generate self extraction and call with self_ref
        let trait_method_name = format!("{}::{}()", trait_name, method_ident);
        let self_extraction = if method.is_mut {
            quote::quote! {
                let mut self_ptr = unsafe {
                    ::miniextendr_api::externalptr::ErasedExternalPtr::from_sexp(self_sexp)
                };
                let self_ref = self_ptr.downcast_mut::<#type_ident>()
                    .unwrap_or_else(|| panic!(
                        "type mismatch in {}: expected ExternalPtr<{}>, got different type. \
                         This can happen if you pass an object of a different type to a trait method.",
                        #trait_method_name,
                        stringify!(#type_ident)
                    ));
            }
        } else {
            quote::quote! {
                let self_ptr = unsafe {
                    ::miniextendr_api::externalptr::ErasedExternalPtr::from_sexp(self_sexp)
                };
                let self_ref = self_ptr.downcast_ref::<#type_ident>()
                    .unwrap_or_else(|| panic!(
                        "type mismatch in {}: expected ExternalPtr<{}>, got different type. \
                         This can happen if you pass an object of a different type to a trait method.",
                        #trait_method_name,
                        stringify!(#type_ident)
                    ));
            }
        };

        // Call expression with self_ref and dereferenced &Self params
        // Use fully-qualified syntax to avoid ambiguity with generic traits
        // (e.g., `RExtend<i32>::method()` is ambiguous — `<` parsed as comparison)
        let call_expr = quote::quote! {
            <#type_ident as #trait_path>::#method_ident(self_ref, #(#call_args),*)
        };

        builder = builder
            .pre_call(vec![self_extraction])
            .call_expr(call_expr)
            .has_self();
    } else {
        // Static method: call directly without self
        let call_expr = quote::quote! {
            <#type_ident as #trait_path>::#method_ident(#(#call_args),*)
        };

        builder = builder.call_expr(call_expr);
    }

    // Apply coerce_all if the method has #[miniextendr(coerce)]
    if method.coerce {
        builder = builder.coerce_all();
    }

    // Apply check_interrupt if the method has #[miniextendr(check_interrupt)]
    if method.check_interrupt {
        builder = builder.check_interrupt();
    }

    // Apply rng if the method has #[miniextendr(rng)]
    if method.rng {
        builder = builder.rng();
    }

    // Apply strict mode for lossy type conversions
    if method.strict {
        builder = builder.strict();
    }

    // The builder generates both the C wrapper and the R_CallMethodDef
    builder.build().generate()
}

/// Returns true when the return type is syntactically `Result<_, _>`.
///
/// Used to determine whether `unwrap_in_r` mode should use `ReturnHandling::IntoR`
/// (passing the Result through to R) instead of the default return handling.
fn output_is_result(output: &syn::ReturnType) -> bool {
    match output {
        syn::ReturnType::Type(_, ty) => matches!(
            ty.as_ref(),
            syn::Type::Path(p)
                if p.path
                    .segments
                    .last()
                    .map(|s| s.ident == "Result")
                    .unwrap_or(false)
        ),
        syn::ReturnType::Default => false,
    }
}

/// Generate a C wrapper function and `R_CallMethodDef` for a trait associated constant.
///
/// The generated wrapper takes no arguments and returns the constant value
/// converted to SEXP. Uses fully-qualified syntax `<Type as Trait>::CONST`
/// to access the value. Always runs on the main thread.
pub(super) fn generate_trait_const_c_wrapper(
    trait_const: &TraitConst,
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    trait_path: &syn::Path,
) -> TokenStream {
    use crate::c_wrapper_builder::{CWrapperContext, ThreadStrategy};

    let const_ident = &trait_const.ident;
    let c_ident = trait_const.c_wrapper_ident(type_ident, trait_name);
    let call_method_def_ident = trait_const.call_method_def_ident(type_ident, trait_name);
    let const_ty = &trait_const.ty;

    // Generate R wrapper const name
    let r_wrappers_const = format_ident!(
        "R_WRAPPERS_{}_{}_IMPL",
        type_ident.to_string().to_uppercase(),
        trait_name.to_string().to_uppercase()
    );

    // Build the call expression to access the const
    let call_expr = quote::quote! {
        <#type_ident as #trait_path>::#const_ident
    };

    // Determine return type handling - we need to convert the const to SEXP
    // The return type is `-> Type` not just `Type`
    let return_type: syn::ReturnType = syn::parse_quote!(-> #const_ty);
    let return_handling = crate::c_wrapper_builder::detect_return_handling(&return_type);

    // Build wrapper - no inputs, just returns the const value
    let builder = CWrapperContext::builder(const_ident.clone(), c_ident)
        .r_wrapper_const(r_wrappers_const)
        .inputs(Default::default()) // no inputs
        .output(return_type)
        .call_expr(call_expr)
        .thread_strategy(ThreadStrategy::MainThread)
        .return_handling(return_handling)
        .type_context(type_ident.clone())
        .call_method_def_ident(call_method_def_ident);

    builder.build().generate()
}
