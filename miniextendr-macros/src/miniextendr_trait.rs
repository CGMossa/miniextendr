//! # Trait Support for `#[miniextendr]`
//!
//! This module handles `#[miniextendr]` applied to trait definitions,
//! generating the ABI infrastructure for cross-package trait dispatch.
//!
//! ## Overview
//!
//! When `#[miniextendr]` is applied to a trait, it generates:
//!
//! 1. **Type tag constant** (`TAG_<TraitName>`) - 128-bit identifier for runtime type checking
//! 2. **Vtable struct** (`<TraitName>VTable`) - Function pointer table for method dispatch
//! 3. **View struct** (`<TraitName>View`) - Runtime wrapper combining data pointer and vtable
//! 4. **Method shims** - `extern "C"` functions that convert SEXP arguments and call methods
//! 5. **Vtable builder** - `__<trait>_build_vtable::<T>()` for impl blocks
//!
//! ## Usage
//!
//! ```ignore
//! #[miniextendr]
//! pub trait Counter {
//!     fn value(&self) -> i32;
//!     fn increment(&mut self);
//!     fn add(&mut self, n: i32);
//! }
//! ```
//!
//! Generates (conceptually):
//!
//! ```text
//! // Original trait (passed through)
//! pub trait Counter {
//!     fn value(&self) -> i32;
//!     fn increment(&mut self);
//!     fn add(&mut self, n: i32);
//! }
//!
//! // Type tag for runtime identification
//! pub const TAG_COUNTER: mx_tag = mx_tag::new(0x..., 0x...);
//!
//! // Vtable with one entry per method
//! #[repr(C)]
//! pub struct CounterVTable {
//!     pub value: mx_meth,
//!     pub increment: mx_meth,
//!     pub add: mx_meth,
//! }
//!
//! // View combining data pointer and vtable
//! #[repr(C)]
//! pub struct CounterView {
//!     pub data: *mut std::ffi::c_void,
//!     pub vtable: *const CounterVTable,
//! }
//!
//! // Shim for each method
//! unsafe extern "C" fn __counter_value_shim<T: Counter>(
//!     data: *mut c_void, argc: i32, argv: *const SEXP
//! ) -> SEXP {
//!     // 1. Check arity
//!     // 2. Cast data to &T
//!     // 3. Call method
//!     // 4. Convert result to SEXP
//!     // 5. Catch panics
//! }
//!
//! // Builder to create vtable for a concrete type
//! pub const fn __counter_build_vtable<T: Counter>() -> CounterVTable {
//!     CounterVTable {
//!         value: __counter_value_shim::<T>,
//!         increment: __counter_increment_shim::<T>,
//!         add: __counter_add_shim::<T>,
//!     }
//! }
//! ```
//!
//! ## Supported Method Signatures
//!
//! Methods must follow these constraints:
//!
//! - **Receiver**: `&self` or `&mut self` for instance methods, or none for static methods
//! - **Arguments**: Types that implement `TryFromSexp`
//! - **Return**: Types that implement `IntoR`, or `()`
//! - **No generics**: Methods cannot have generic type parameters
//! - **No async**: Async methods are not supported
//! - **Static methods**: Methods without a receiver are allowed and resolved at compile time
//!   (they don't go through the vtable)
//!
//! ## Default Methods
//!
//! Default method implementations are supported. The vtable builder will
//! use the default implementation if the concrete type doesn't override it.
//!
//! ```ignore
//! #[miniextendr]
//! pub trait Counter {
//!     fn value(&self) -> i32;
//!
//!     // Default implementation - included in vtable
//!     fn is_zero(&self) -> bool {
//!         self.value() == 0
//!     }
//! }
//! ```
//!
//! ## Error Handling
//!
//! Method shims handle errors as follows:
//!
//! - **Arity mismatch**: Raises R error ("expected N arguments, got M")
//! - **Type conversion failure**: Raises R error with the error message
//! - **Panic**: Caught via `with_r_unwind_protect`, converted to R error
//!
//! ## Thread Safety
//!
//! All generated shims are **main-thread only**. They do not route through
//! `with_r_thread` because R invokes `.Call` on the main thread.

use proc_macro2::TokenStream;
use syn::ItemTrait;

/// Expand `#[miniextendr]` applied to a trait definition.
///
/// # Arguments
///
/// * `attr` - Attribute arguments (currently unused, reserved for future options)
/// * `item` - The trait definition token stream
///
/// # Returns
///
/// Expanded token stream containing:
/// - Original trait definition
/// - Type tag constant
/// - Vtable struct
/// - View struct
/// - Method shims
/// - Vtable builder function
///
/// # Errors
///
/// Returns a compile error if:
/// - Methods have unsupported signatures
/// - Methods are async
pub fn expand_trait(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let trait_item = syn::parse_macro_input!(item as ItemTrait);

    // Validate trait constraints
    if let Err(e) = validate_trait(&trait_item) {
        return e.into_compile_error().into();
    }

    // Generate the expanded code
    let expanded = generate_trait_abi(&trait_item);

    expanded.into()
}

/// Validate that the trait meets requirements for ABI generation.
///
/// # Constraints
///
/// - All methods must have `&self` or `&mut self` receiver
/// - Methods cannot be async
/// - Methods cannot have generic parameters
/// - Generic type parameters on the trait itself are allowed
fn validate_trait(trait_item: &ItemTrait) -> syn::Result<()> {
    let trait_name = &trait_item.ident;

    // Validate each method
    for item in &trait_item.items {
        if let syn::TraitItem::Fn(method) = item {
            validate_method(method, trait_name)?;
        }
    }

    Ok(())
}

/// Validate a single trait method for ABI compatibility.
///
/// Rejects async methods, methods with generic type parameters, and methods
/// that take `self` by value (only `&self` and `&mut self` are allowed).
/// Static methods (no receiver) are permitted.
fn validate_method(method: &syn::TraitItemFn, trait_name: &syn::Ident) -> syn::Result<()> {
    let method_name = &method.sig.ident;

    // Check for async
    if method.sig.asyncness.is_some() {
        return Err(syn::Error::new_spanned(
            method.sig.asyncness,
            format!(
                "#[miniextendr] trait `{}::{}` cannot be async",
                trait_name, method_name
            ),
        ));
    }

    // Check for type/const generics on method.
    // Lifetime params are allowed (erased at codegen); type/const params would require
    // monomorphization and are incompatible with the vtable-shim codegen path.
    {
        let has_type_or_const = method
            .sig
            .generics
            .params
            .iter()
            .any(|p| matches!(p, syn::GenericParam::Type(_) | syn::GenericParam::Const(_)));
        if has_type_or_const {
            return Err(syn::Error::new_spanned(
                &method.sig.generics,
                format!(
                    "#[miniextendr] trait method `{}::{}` cannot have generic type or const parameters",
                    trait_name, method_name
                ),
            ));
        }
    }

    // Check receiver - must be &self, &mut self, self: &Self, self: &mut Self, or no receiver
    // Static methods are allowed but won't be included in the vtable
    // (they're resolved at compile time via <Type as Trait>::method())
    let receiver = method.sig.inputs.first();
    if let Some(syn::FnArg::Receiver(r)) = receiver {
        // Accept either:
        // - `&self` / `&mut self` (r.reference is Some)
        // - `self: &Self` / `self: &mut Self` (r.colon_token is Some with reference type)
        let is_ref = if r.reference.is_some() {
            true
        } else if r.colon_token.is_some() {
            // Check if the type is a reference type (&Self or &mut Self)
            matches!(r.ty.as_ref(), syn::Type::Reference(_))
        } else {
            false
        };

        if !is_ref {
            return Err(syn::Error::new_spanned(
                r,
                format!(
                    "#[miniextendr] trait method `{}::{}` receiver must be `&self` or `&mut self`, not `self` by value",
                    trait_name, method_name
                ),
            ));
        }
    }
    // If receiver is None or FnArg::Typed (no self), it's a static method - allowed

    Ok(())
}

/// Generate the ABI infrastructure for a trait.
///
/// This is the main code generation function that produces:
/// - Type tag constant
/// - Vtable struct
/// - View struct (skipped for generic traits)
/// - Method shims (with trait type params threaded through)
/// - Vtable builder (with trait type params threaded through)
fn generate_trait_abi(trait_item: &ItemTrait) -> TokenStream {
    let trait_name = &trait_item.ident;
    let vis = &trait_item.vis;

    // Generate names for generated items
    let tag_name = quote::format_ident!("TAG_{}", trait_name.to_string().to_uppercase());
    let vtable_name = quote::format_ident!("{}VTable", trait_name);
    let view_name = quote::format_ident!("{}View", trait_name);
    let build_vtable_fn =
        quote::format_ident!("__{}_build_vtable", trait_name.to_string().to_lowercase());

    // Collect trait-level generic type parameters
    let trait_type_params: Vec<&syn::GenericParam> = trait_item.generics.params.iter().collect();
    let trait_param_idents: Vec<&syn::Ident> = trait_type_params
        .iter()
        .filter_map(|p| {
            if let syn::GenericParam::Type(tp) = p {
                Some(&tp.ident)
            } else {
                None
            }
        })
        .collect();
    let has_generics = !trait_param_idents.is_empty();

    // Collect associated types
    let assoc_types: Vec<&syn::Ident> = trait_item
        .items
        .iter()
        .filter_map(|item| {
            if let syn::TraitItem::Type(t) = item {
                Some(&t.ident)
            } else {
                None
            }
        })
        .collect();

    // Collect trait where clause
    let trait_where_clause = &trait_item.generics.where_clause;

    // Collect method information
    // Filter to only include instance methods (with &self or &mut self) that aren't skipped
    let methods: Vec<_> = {
        let mut collected = Vec::new();
        for item in &trait_item.items {
            if let syn::TraitItem::Fn(method) = item {
                let info = match extract_method_info(method) {
                    Ok(info) => info,
                    Err(e) => return e.into_compile_error(),
                };
                if info.has_self && !info.skip {
                    collected.push(info);
                }
            }
        }
        collected
    }
    .into_iter()
    .collect();

    // Generate tag path string for hashing
    // IMPORTANT: For cross-package trait dispatch, the tag must NOT include module_path!()
    // Different packages defining the same trait signature should get the same tag.
    // We use just the trait name - in practice, trait names + methods should be unique enough.
    let tag_path = trait_name.to_string();

    // Generate vtable fields
    let vtable_fields: Vec<_> = methods
        .iter()
        .map(|m| {
            let name = &m.name;
            quote::quote! {
                pub #name: ::miniextendr_api::abi::mx_meth
            }
        })
        .collect();

    // Compute extra bounds needed for shims
    let extra_bounds =
        compute_extra_bounds(&methods, trait_name, &assoc_types, &trait_param_idents);

    // Build trait bound for __ImplT
    let impl_t = quote::format_ident!("__ImplT");
    let trait_bound = if trait_param_idents.is_empty() {
        quote::quote! { #trait_name }
    } else {
        quote::quote! { #trait_name<#(#trait_param_idents),*> }
    };

    // Build combined where clause
    let all_where_predicates = build_where_predicates(trait_where_clause, &extra_bounds);
    let where_clause = if all_where_predicates.is_empty() {
        quote::quote! {}
    } else {
        quote::quote! { where #(#all_where_predicates),* }
    };

    // Generate shim functions
    let shim_fns: Vec<_> = methods
        .iter()
        .map(|m| {
            generate_method_shim(
                trait_name,
                m,
                &extra_bounds,
                &trait_param_idents,
                &trait_type_params,
                trait_where_clause,
            )
        })
        .collect();

    // Generate vtable field initializers (turbofish includes trait type params)
    let vtable_inits: Vec<_> = methods
        .iter()
        .map(|m| {
            let name = &m.name;
            let shim_name = quote::format_ident!(
                "__{}_{}_shim",
                trait_name.to_string().to_lowercase(),
                crate::naming::unraw(name)
            );
            quote::quote! {
                #name: #shim_name::<#(#trait_param_idents,)* #impl_t>
            }
        })
        .collect();

    // Generate method wrappers for the View struct
    // Skip entirely for generic traits (type-erased view can't know type params)
    let view_methods: Vec<_> = if has_generics {
        vec![]
    } else {
        methods.iter().filter_map(generate_view_method).collect()
    };

    // Strip #[miniextendr(...)] attrs from trait items before emitting
    let mut clean_trait = trait_item.clone();
    for item in &mut clean_trait.items {
        if let syn::TraitItem::Fn(method) = item {
            method
                .attrs
                .retain(|attr| !attr.path().is_ident("miniextendr"));
        }
    }

    let trait_name_str = trait_name.to_string();
    let source_loc_doc = crate::source_location_doc(trait_name.span());

    let impl_bounds = &extra_bounds.impl_bounds;

    // View struct and its impls (skipped for generic traits)
    let view_tokens = if has_generics {
        quote::quote! {}
    } else {
        quote::quote! {
            #[doc = concat!(
                "Runtime view for objects implementing `",
                stringify!(#trait_name),
                "`."
            )]
            #[doc = #source_loc_doc]
            #[doc = concat!("Generated from source file `", file!(), "`.")]
            ///
            /// Combines a data pointer with a vtable pointer for method dispatch.
            /// Use `try_from_sexp` to create a view from an R external pointer.
            #[repr(C)]
            #vis struct #view_name {
                /// Pointer to the concrete object data.
                pub data: *mut ::std::os::raw::c_void,
                /// Pointer to the vtable for this trait.
                pub vtable: *const #vtable_name,
            }

            // TraitView implementation
            impl ::miniextendr_api::TraitView for #view_name {
                const TAG: ::miniextendr_api::abi::mx_tag = #tag_name;

                #[inline]
                unsafe fn from_raw_parts(
                    data: *mut ::std::os::raw::c_void,
                    vtable: *const ::std::os::raw::c_void,
                ) -> Self {
                    Self {
                        data,
                        vtable: vtable.cast::<#vtable_name>(),
                    }
                }
            }

            // Method wrappers on View
            impl #view_name {
                /// Try to create a view from an R SEXP.
                ///
                /// Returns `Some(Self)` if the object implements this trait,
                /// `None` otherwise.
                ///
                /// # Safety
                ///
                /// - `sexp` must be a valid R external pointer (EXTPTRSXP)
                /// - Must be called on R's main thread
                #[inline]
                pub unsafe fn try_from_sexp(sexp: ::miniextendr_api::SEXP) -> Option<Self> {
                    <Self as ::miniextendr_api::TraitView>::try_from_sexp(sexp)
                }

                /// Try to create a view, panicking with error message on failure.
                ///
                /// # Safety
                ///
                /// Same as `try_from_sexp`.
                #[inline]
                pub unsafe fn from_sexp(sexp: ::miniextendr_api::SEXP) -> Self {
                    Self::try_from_sexp(sexp)
                        .expect(concat!("Object does not implement ", #trait_name_str, " trait"))
                }

                #(#view_methods)*
            }
        }
    };

    // For generic traits (with type params like <T>), skip shim and builder generation.
    // These are generated at the impl site with concrete types to avoid recursive trait
    // resolution overflow (e.g., `Vec<T>: TryFromSexp` triggers infinite recursion through
    // `impl<T> TryFromSexp for Vec<Vec<T>>`).
    let shim_and_builder = if has_generics {
        quote::quote! {}
    } else {
        quote::quote! {
            // Method shims
            #(#shim_fns)*

            #[doc = concat!(
                "Build a vtable for a concrete type implementing `",
                stringify!(#trait_name),
                "`."
            )]
            #[doc = #source_loc_doc]
            #[doc = concat!("Generated from source file `", file!(), "`.")]
            #vis const fn #build_vtable_fn<#(#trait_type_params,)* #impl_t: #trait_bound #(+ #impl_bounds)*>() -> #vtable_name
            #where_clause
            {
                #vtable_name {
                    #(#vtable_inits),*
                }
            }
        }
    };

    // TPIE: Generate macro_rules! for non-generic traits without associated types.
    // This enables `#[miniextendr] impl Trait for Type {}` (empty body) to auto-expand wrappers.
    let tpie_macro = if !has_generics && assoc_types.is_empty() {
        // Collect ALL non-skipped methods (including static) for TPIE metadata
        let tpie_method_metadata: Vec<TokenStream> = {
            let mut collected = Vec::new();
            for item in &trait_item.items {
                if let syn::TraitItem::Fn(method) = item {
                    let info = match extract_method_info(method) {
                        Ok(info) => info,
                        Err(e) => return e.into_compile_error(),
                    };
                    if !info.skip {
                        let r_name_ident = if let Some(ref rn) = info.r_name {
                            quote::format_ident!("{}", rn)
                        } else {
                            method.sig.ident.clone()
                        };
                        let sig = &method.sig;
                        collected.push(quote::quote! {
                            method { r_name = #r_name_ident; #sig; }
                        });
                    }
                }
            }
            collected
        };

        let tpie_macro_name = quote::format_ident!("__mx_impl_{}", trait_name);
        quote::quote! {
            #[macro_export]
            #[doc(hidden)]
            macro_rules! #tpie_macro_name {
                ($concrete_type:ty, $trait_path:path, $class_system:ident, $no_rd:tt, $internal:tt, $noexport:tt) => {
                    $crate::__mx_trait_impl_expand! {
                        concrete_type = $concrete_type;
                        trait_path = $trait_path;
                        class_system = $class_system;
                        no_rd = $no_rd;
                        internal = $internal;
                        noexport = $noexport;
                        #(#tpie_method_metadata)*
                    }
                };
            }
        }
    } else {
        quote::quote! {}
    };

    quote::quote! {
        // Pass through the original trait (with #[miniextendr] attrs stripped from items)
        #clean_trait

        #[doc = concat!(
            "Type tag for runtime identification of the `",
            stringify!(#trait_name),
            "` trait."
        )]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #vis const #tag_name: ::miniextendr_api::abi::mx_tag =
            ::miniextendr_api::abi::mx_tag_from_path(#tag_path);

        #[doc = concat!("Vtable for the `", stringify!(#trait_name), "` trait.")]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        ///
        /// Contains one `mx_meth` function pointer per trait method.
        #[repr(C)]
        #[doc(hidden)]
        #vis struct #vtable_name {
            #(#vtable_fields),*
        }

        #view_tokens

        #shim_and_builder

        #tpie_macro
    }
}

/// Generate a method wrapper for the View struct.
///
/// This creates a method on the View that calls through the vtable.
/// Returns None for methods with `Self` in return types or `&Self` in parameters,
/// since these can't be meaningfully expressed on the type-erased View.
fn generate_view_method(method: &MethodInfo) -> Option<TokenStream> {
    // Skip methods where Self appears in return type or parameters.
    // In the View context, Self refers to the View struct, not the concrete type,
    // so these methods can't work through the type-erased vtable dispatch.
    if method.return_type.as_ref().is_some_and(type_contains_self) {
        return None;
    }
    if method.param_types.iter().any(type_contains_self) {
        return None;
    }

    let method_name = &method.name;
    let param_names = &method.param_names;
    let param_types = &method.param_types;

    // Generate function parameters
    let params: Vec<_> = param_names
        .iter()
        .zip(param_types.iter())
        .map(|(name, ty)| {
            quote::quote! { #name: #ty }
        })
        .collect();

    // Generate self receiver
    let self_param = if method.is_mut {
        quote::quote! { &mut self }
    } else {
        quote::quote! { &self }
    };

    // Generate argument array for vtable call
    let argc = param_types.len() as i32;
    let arg_conversions: Vec<_> = param_names
        .iter()
        .map(|name| {
            quote::quote! {
                ::miniextendr_api::trait_abi::to_sexp(#name)
            }
        })
        .collect();

    // Generate vtable call
    let vtable_call = if argc > 0 {
        quote::quote! {
            let args: [::miniextendr_api::SEXP; #argc as usize] = [#(#arg_conversions),*];
            ((*self.vtable).#method_name)(self.data, #argc, args.as_ptr())
        }
    } else {
        quote::quote! {
            ((*self.vtable).#method_name)(self.data, 0, ::std::ptr::null())
        }
    };

    // Generate return type handling
    let return_type = &method.return_type;
    let (return_sig, result_conversion) = if let Some(ret_ty) = return_type {
        (
            quote::quote! { -> #ret_ty },
            quote::quote! {
                ::miniextendr_api::trait_abi::from_sexp::<#ret_ty>(result)
            },
        )
    } else {
        (
            quote::quote! {},
            quote::quote! {
                let _ = result;
            },
        )
    };

    Some(quote::quote! {
        #[doc = concat!("Call `", stringify!(#method_name), "` through the vtable.")]
        #[inline]
        pub fn #method_name(#self_param #(, #params)*) #return_sig {
            unsafe {
                let result = { #vtable_call };
                // Approach 1 (issue #345): if the shim returned a tagged error SEXP,
                // re-panic with the reconstructed RCondition so the consumer's outer
                // `with_r_unwind_protect` guard can apply rust_* class layering.
                ::miniextendr_api::trait_abi::repanic_if_rust_error(result);
                #result_conversion
            }
        }
    })
}

/// Generate a method shim function for a trait method.
///
/// The shim is an `extern "C"` function that:
/// 1. Checks argument arity
/// 2. Wraps everything in `with_r_unwind_protect` to prevent unwinding across FFI
/// 3. Converts SEXP arguments to Rust types
/// 4. Calls the actual method on the concrete type
/// 5. Converts the result back to SEXP
/// 6. On panic, converts to R error via `with_r_unwind_protect`
///
/// For generic traits, the shim carries the trait's type parameters plus `__ImplT`.
fn generate_method_shim(
    trait_name: &syn::Ident,
    method: &MethodInfo,
    extra_bounds: &ExtraBounds,
    trait_param_idents: &[&syn::Ident],
    trait_type_params: &[&syn::GenericParam],
    trait_where_clause: &Option<syn::WhereClause>,
) -> TokenStream {
    let method_name = &method.name;
    let shim_name = quote::format_ident!(
        "__{}_{}_shim",
        trait_name.to_string().to_lowercase(),
        method_name
    );
    let impl_t = quote::format_ident!("__ImplT");

    let param_count = method.param_types.len();
    let expected_argc = param_count as i32;

    // Generate argument extraction
    // For &Self params, extract ExternalPtr<__ImplT> and borrow from it
    let arg_extractions: Vec<_> = method
        .param_names
        .iter()
        .zip(method.param_types.iter())
        .enumerate()
        .map(|(i, (name, ty))| {
            let name_str = crate::naming::ident_name(name);
            let (is_self_ref, is_mut) = param_is_self_ref(ty);
            if is_self_ref {
                let extptr_name = quote::format_ident!("__extptr_{}", crate::naming::unraw(name));
                if is_mut {
                    quote::quote! {
                        let mut #extptr_name: ::miniextendr_api::ExternalPtr<#impl_t> = unsafe {
                            ::miniextendr_api::trait_abi::extract_arg(argc, argv, #i, #name_str)
                        };
                        let #name: &mut #impl_t = &mut *#extptr_name;
                    }
                } else {
                    quote::quote! {
                        let #extptr_name: ::miniextendr_api::ExternalPtr<#impl_t> = unsafe {
                            ::miniextendr_api::trait_abi::extract_arg(argc, argv, #i, #name_str)
                        };
                        let #name: &#impl_t = &*#extptr_name;
                    }
                }
            } else {
                quote::quote! {
                    let #name: #ty = unsafe {
                        ::miniextendr_api::trait_abi::extract_arg(argc, argv, #i, #name_str)
                    };
                }
            }
        })
        .collect();

    // Generate method call (uses __ImplT)
    let param_names = &method.param_names;
    let method_call = if method.is_mut {
        quote::quote! {
            let self_ref = unsafe { &mut *data.cast::<#impl_t>() };
            self_ref.#method_name(#(#param_names),*)
        }
    } else {
        quote::quote! {
            let self_ref = unsafe { &*data.cast::<#impl_t>().cast_const() };
            self_ref.#method_name(#(#param_names),*)
        }
    };

    // Generate result conversion
    let result_conversion = if method.return_type.is_some() {
        quote::quote! {
            unsafe { ::miniextendr_api::trait_abi::to_sexp(result) }
        }
    } else {
        quote::quote! {
            let _ = result;
            unsafe { ::miniextendr_api::trait_abi::nil() }
        }
    };

    // Build trait bound for __ImplT
    let trait_bound = if trait_param_idents.is_empty() {
        quote::quote! { #trait_name }
    } else {
        quote::quote! { #trait_name<#(#trait_param_idents),*> }
    };

    let impl_bounds = &extra_bounds.impl_bounds;
    let all_where_predicates = build_where_predicates(trait_where_clause, extra_bounds);
    let where_clause = if all_where_predicates.is_empty() {
        quote::quote! {}
    } else {
        quote::quote! { where #(#all_where_predicates),* }
    };

    let method_name_str = format!("{}::{}", trait_name, method_name);

    quote::quote! {
        #[doc = concat!(
            "Method shim for `",
            stringify!(#trait_name),
            "::",
            stringify!(#method_name),
            "`."
        )]
        ///
        /// Converts SEXP arguments, calls the method, and returns SEXP result.
        /// Both Rust panics and R longjmps are caught via `with_r_unwind_protect`.
        #[doc(hidden)]
        unsafe extern "C" fn #shim_name<#(#trait_type_params,)* #impl_t: #trait_bound #(+ #impl_bounds)*>(
            data: *mut ::std::os::raw::c_void,
            argc: i32,
            argv: *const ::miniextendr_api::SEXP,
        ) -> ::miniextendr_api::SEXP
        #where_clause
        {
            // Check arity (before unwind protect - uses r_stop which doesn't return)
            unsafe {
                ::miniextendr_api::trait_abi::check_arity(argc, #expected_argc, #method_name_str);
            }

            // Wrap in with_r_unwind_protect_shim: catches Rust panics and returns
            // a tagged error SEXP instead of calling Rf_errorcall directly. The
            // tagged SEXP is returned to the View method wrapper which re-panics
            // via repanic_if_rust_error, allowing the consumer's outer
            // `with_r_unwind_protect` guard to produce rust_* class layering
            // (issue #345). R-origin longjmps still propagate via R_ContinueUnwind.
            ::miniextendr_api::unwind_protect::with_r_unwind_protect_shim(|| {
                // Extract arguments
                #(#arg_extractions)*

                // Call method
                let result = { #method_call };

                // Convert result
                #result_conversion
            })
        }
    }
}

/// Information extracted from a trait method for code generation.
///
/// Collects everything needed to generate vtable shims, view methods,
/// and extra trait bounds for a single method in a `#[miniextendr]` trait.
#[derive(Debug)]
struct MethodInfo {
    /// Method name (Rust identifier).
    name: syn::Ident,
    /// Whether the method has a self receiver (instance method).
    /// False for static/associated methods.
    has_self: bool,
    /// Whether receiver is `&mut self` (vs `&self`). Only meaningful if `has_self` is true.
    is_mut: bool,
    /// Parameter types (excluding the self receiver).
    param_types: Vec<syn::Type>,
    /// Parameter names (excluding the self receiver). Uses `arg{i}` for unnamed patterns.
    param_names: Vec<syn::Ident>,
    /// Return type. `None` when the method returns `()` (unit type or no return annotation).
    return_type: Option<syn::Type>,
    /// Whether method is marked `#[miniextendr(skip)]`, excluding it from codegen.
    skip: bool,
    /// Override the R-facing method name (from `#[miniextendr(r_name = "...")]`).
    /// When set, R wrappers and TPIE metadata use this name instead of the Rust ident.
    r_name: Option<String>,
}

// region: Self-type detection helpers

/// Check if a type syntactically contains `Self`.
///
/// Used to detect when a method returns `Self` (or `Option<Self>`, `Vec<Self>`, etc.)
/// so the generated shim can add `IntoR` bounds.
fn type_contains_self(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(tp) => {
            for seg in &tp.path.segments {
                if seg.ident == "Self" {
                    return true;
                }
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner) = arg
                            && type_contains_self(inner)
                        {
                            return true;
                        }
                    }
                }
            }
            false
        }
        syn::Type::Reference(r) => type_contains_self(&r.elem),
        syn::Type::Tuple(t) => t.elems.iter().any(type_contains_self),
        syn::Type::Slice(s) => type_contains_self(&s.elem),
        syn::Type::Array(a) => type_contains_self(&a.elem),
        syn::Type::Paren(p) => type_contains_self(&p.elem),
        _ => false,
    }
}

/// Check if a parameter type is `&Self` or `&mut Self`.
///
/// Returns `(is_self_ref, is_mut)`. When true, the generated shim extracts
/// an `ExternalPtr<T>` from the SEXP and borrows from it instead of trying
/// to extract `&T` directly (which doesn't implement `TryFromSexp`).
fn param_is_self_ref(ty: &syn::Type) -> (bool, bool) {
    if let syn::Type::Reference(r) = ty
        && let syn::Type::Path(tp) = r.elem.as_ref()
        && tp.path.is_ident("Self")
    {
        return (true, r.mutability.is_some());
    }
    (false, false)
}

/// Check if a type syntactically contains `Self::AssocType` for a given associated type name.
///
/// Recursively walks the type tree looking for a 2-segment path where the first
/// segment is `Self` and the second matches `assoc_name` (e.g., `Self::Item`).
/// Used to determine whether extra `where` bounds are needed for associated types.
fn type_contains_self_assoc(ty: &syn::Type, assoc_name: &syn::Ident) -> bool {
    match ty {
        syn::Type::Path(tp) => {
            if tp.path.segments.len() == 2
                && tp.path.segments[0].ident == "Self"
                && tp.path.segments[1].ident == *assoc_name
            {
                return true;
            }
            for seg in &tp.path.segments {
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner) = arg
                            && type_contains_self_assoc(inner, assoc_name)
                        {
                            return true;
                        }
                    }
                }
            }
            false
        }
        syn::Type::Reference(r) => type_contains_self_assoc(&r.elem, assoc_name),
        syn::Type::Tuple(t) => t
            .elems
            .iter()
            .any(|e| type_contains_self_assoc(e, assoc_name)),
        syn::Type::Slice(s) => type_contains_self_assoc(&s.elem, assoc_name),
        syn::Type::Array(a) => type_contains_self_assoc(&a.elem, assoc_name),
        syn::Type::Paren(p) => type_contains_self_assoc(&p.elem, assoc_name),
        _ => false,
    }
}

/// Rewrite `Self` and `Self::AssocType` in a type tree to use `__ImplT`.
///
/// Transforms:
/// - `Self` → `__ImplT`
/// - `Self::Item` → `<__ImplT as TraitName>::Item`
/// - Recursively processes generic arguments (e.g., `Option<Self::Item>` →
///   `Option<<__ImplT as TraitName>::Item>`)
fn rewrite_self_in_type(
    ty: &syn::Type,
    trait_name: &syn::Ident,
    assoc_types: &[&syn::Ident],
) -> syn::Type {
    match ty {
        syn::Type::Path(tp) => {
            // Check for Self::AssocType (2-segment path: Self::Item)
            if tp.path.segments.len() == 2
                && tp.path.segments[0].ident == "Self"
                && assoc_types.iter().any(|a| *a == &tp.path.segments[1].ident)
            {
                let assoc = &tp.path.segments[1].ident;
                let impl_t = quote::format_ident!("__ImplT");
                return syn::parse_quote!(<#impl_t as #trait_name>::#assoc);
            }
            // Check for bare Self
            if tp.path.is_ident("Self") {
                let impl_t = quote::format_ident!("__ImplT");
                return syn::parse_quote!(#impl_t);
            }
            // Recursively process generic args
            let mut new_tp = tp.clone();
            for seg in &mut new_tp.path.segments {
                if let syn::PathArguments::AngleBracketed(args) = &mut seg.arguments {
                    for arg in &mut args.args {
                        if let syn::GenericArgument::Type(inner) = arg {
                            *inner = rewrite_self_in_type(inner, trait_name, assoc_types);
                        }
                    }
                }
            }
            syn::Type::Path(new_tp)
        }
        syn::Type::Reference(r) => {
            let mut new_r = r.clone();
            new_r.elem = Box::new(rewrite_self_in_type(&r.elem, trait_name, assoc_types));
            syn::Type::Reference(new_r)
        }
        syn::Type::Tuple(t) => {
            let mut new_t = t.clone();
            for elem in &mut new_t.elems {
                *elem = rewrite_self_in_type(elem, trait_name, assoc_types);
            }
            syn::Type::Tuple(new_t)
        }
        syn::Type::Slice(s) => {
            let mut new_s = s.clone();
            new_s.elem = Box::new(rewrite_self_in_type(&s.elem, trait_name, assoc_types));
            syn::Type::Slice(new_s)
        }
        syn::Type::Array(a) => {
            let mut new_a = a.clone();
            new_a.elem = Box::new(rewrite_self_in_type(&a.elem, trait_name, assoc_types));
            syn::Type::Array(new_a)
        }
        syn::Type::Paren(p) => {
            let mut new_p = p.clone();
            new_p.elem = Box::new(rewrite_self_in_type(&p.elem, trait_name, assoc_types));
            syn::Type::Paren(new_p)
        }
        _ => ty.clone(),
    }
}

/// Check if a type syntactically contains a specific identifier.
///
/// Used to detect trait type parameters (like `T`) in method signatures so that
/// appropriate `TryFromSexp` or `IntoR` bounds can be added. Recursively walks
/// through path segments, generic arguments, references, tuples, slices, and arrays.
fn type_contains_ident(ty: &syn::Type, ident: &syn::Ident) -> bool {
    match ty {
        syn::Type::Path(tp) => {
            if tp.path.segments.len() == 1 && tp.path.segments[0].ident == *ident {
                return true;
            }
            for seg in &tp.path.segments {
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner) = arg
                            && type_contains_ident(inner, ident)
                        {
                            return true;
                        }
                    }
                }
            }
            false
        }
        syn::Type::Reference(r) => type_contains_ident(&r.elem, ident),
        syn::Type::Tuple(t) => t.elems.iter().any(|e| type_contains_ident(e, ident)),
        syn::Type::Slice(s) => type_contains_ident(&s.elem, ident),
        syn::Type::Array(a) => type_contains_ident(&a.elem, ident),
        syn::Type::Paren(p) => type_contains_ident(&p.elem, ident),
        _ => false,
    }
}

/// Extra trait bounds inferred from method signatures.
///
/// For generic traits and methods that reference `Self` or associated types,
/// the generated shim and vtable builder functions need additional bounds
/// beyond `__ImplT: TraitName`. This struct collects those bounds.
struct ExtraBounds {
    /// Bounds added directly to `__ImplT` (e.g., `IntoR` when methods return `Self`,
    /// or `TypedExternal + Send + 'static` when methods take `&Self` parameters).
    impl_bounds: Vec<TokenStream>,
    /// Where clause predicates for complex types (e.g.,
    /// `<__ImplT as Trait>::Item: IntoR` or `Vec<T>: TryFromSexp`).
    where_predicates: Vec<TokenStream>,
}

/// Compute extra bounds needed for the shim and build_vtable functions.
///
/// - Methods returning `Self` → `__ImplT: IntoR`
/// - Methods with `&Self` params → `__ImplT: TypedExternal + Send + 'static`
/// - Methods returning types with `Self::AssocType` or trait type params →
///   full rewritten return type `: IntoR` (e.g., `Option<<__ImplT as RIterator>::Item>: IntoR`)
/// - Methods with trait type params in params →
///   full param type `: TryFromSexp` (e.g., `Vec<T>: TryFromSexp`)
fn compute_extra_bounds(
    methods: &[MethodInfo],
    trait_name: &syn::Ident,
    assoc_types: &[&syn::Ident],
    trait_param_idents: &[&syn::Ident],
) -> ExtraBounds {
    let mut impl_bounds = Vec::new();
    let mut where_predicates = Vec::new();

    let mut needs_into_r = false;
    let mut needs_typed_external = false;

    // Track full types needing bounds (deduplicated by token string)
    let mut return_type_bound_keys: std::collections::BTreeMap<String, syn::Type> =
        Default::default();
    let mut param_type_bound_keys: std::collections::BTreeMap<String, syn::Type> =
        Default::default();

    for method in methods {
        // Bare Self in returns → __ImplT: IntoR (as impl bound)
        if method.return_type.as_ref().is_some_and(type_contains_self) {
            needs_into_r = true;
        }
        // &Self in params → __ImplT: TypedExternal + 'static
        if method.param_types.iter().any(|ty| param_is_self_ref(ty).0) {
            needs_typed_external = true;
        }

        // Full return type bounds for Self::AssocType and/or trait type params.
        // Instead of bare `<__ImplT as Trait>::Item: IntoR`, we add the FULL rewritten
        // return type: `Option<<__ImplT as Trait>::Item>: IntoR`, `Vec<T>: IntoR`, etc.
        // This is required because IntoR impls are concrete (no blanket `Option<T: IntoR>: IntoR`).
        if let Some(ref ret_ty) = method.return_type {
            let has_assoc = assoc_types
                .iter()
                .any(|a| type_contains_self_assoc(ret_ty, a));
            let has_param = trait_param_idents
                .iter()
                .any(|p| type_contains_ident(ret_ty, p));
            if has_assoc || has_param {
                let rewritten = rewrite_self_in_type(ret_ty, trait_name, assoc_types);
                let key = quote::quote!(#rewritten).to_string();
                return_type_bound_keys.entry(key).or_insert(rewritten);
            }
        }

        // Full param type bounds for trait type params.
        // Instead of bare `T: TryFromSexp`, we add `Vec<T>: TryFromSexp` etc.
        for param_ty in &method.param_types {
            if !param_is_self_ref(param_ty).0 {
                let has_param = trait_param_idents
                    .iter()
                    .any(|p| type_contains_ident(param_ty, p));
                if has_param {
                    let key = quote::quote!(#param_ty).to_string();
                    param_type_bound_keys.entry(key).or_insert(param_ty.clone());
                }
            }
        }
    }

    if needs_into_r {
        impl_bounds.push(quote::quote! { ::miniextendr_api::IntoR });
    }
    if needs_typed_external {
        impl_bounds.push(quote::quote! { ::miniextendr_api::TypedExternal + Send + 'static });
    }

    // Add full return type bounds: RewrittenType: IntoR
    for ty in return_type_bound_keys.values() {
        where_predicates.push(quote::quote! {
            #ty: ::miniextendr_api::IntoR
        });
    }

    // Add full param type bounds: ParamType: TryFromSexp, Error: Display
    for ty in param_type_bound_keys.values() {
        where_predicates.push(quote::quote! {
            #ty: ::miniextendr_api::TryFromSexp
        });
        where_predicates.push(quote::quote! {
            <#ty as ::miniextendr_api::TryFromSexp>::Error: ::std::fmt::Display
        });
    }

    ExtraBounds {
        impl_bounds,
        where_predicates,
    }
}

/// Build combined where predicates from the trait's own where clause and computed extra bounds.
///
/// Merges the original trait-level where clause predicates with the extra
/// bounds computed from method signatures (e.g., `IntoR` for return types,
/// `TryFromSexp` for parameters containing trait type params).
///
/// Returns a flat list of predicates suitable for use in a `where` clause.
fn build_where_predicates(
    trait_where_clause: &Option<syn::WhereClause>,
    extra_bounds: &ExtraBounds,
) -> Vec<TokenStream> {
    let mut all = Vec::new();
    if let Some(wc) = trait_where_clause {
        for pred in &wc.predicates {
            all.push(quote::quote! { #pred });
        }
    }
    all.extend(extra_bounds.where_predicates.iter().cloned());
    all
}

/// Extract method information from a trait method definition.
///
/// Parses the method signature to determine receiver type, parameter names/types,
/// return type, and any `#[miniextendr(...)]` attributes like `skip` and `r_name`.
/// Parameters with non-ident patterns are assigned synthetic names (`arg0`, `arg1`, etc.).
fn extract_method_info(method: &syn::TraitItemFn) -> syn::Result<MethodInfo> {
    let name = method.sig.ident.clone();

    // Check for #[miniextendr(skip)] and #[miniextendr(r_name = "...")]
    let mut skip = false;
    let mut r_name: Option<String> = None;
    for attr in &method.attrs {
        if !attr.path().is_ident("miniextendr") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                skip = true;
            } else if meta.path.is_ident("r_name") {
                let value: syn::LitStr = meta.value()?.parse()?;
                r_name = Some(value.value());
            } else {
                return Err(meta.error(
                    "unknown #[miniextendr] option on trait method; expected `skip` or `r_name`",
                ));
            }
            Ok(())
        })?;
    }

    // Check for receiver
    let (has_self, is_mut) = method.sig.inputs.first().map_or((false, false), |arg| {
        if let syn::FnArg::Receiver(r) = arg {
            (true, r.mutability.is_some())
        } else {
            (false, false)
        }
    });

    // Extract parameters (skip self if present)
    let skip_count = if has_self { 1 } else { 0 };
    let mut param_types = Vec::new();
    let mut param_names = Vec::new();
    for (i, arg) in method.sig.inputs.iter().skip(skip_count).enumerate() {
        if let syn::FnArg::Typed(pat_type) = arg {
            param_types.push((*pat_type.ty).clone());
            if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                param_names.push(pat_ident.ident.clone());
            } else {
                param_names.push(quote::format_ident!("arg{}", i));
            }
        }
    }

    // Extract return type
    let return_type = match &method.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => {
            // Check if it's unit type ()
            if matches!(ty.as_ref(), syn::Type::Tuple(t) if t.elems.is_empty()) {
                None
            } else {
                Some((**ty).clone())
            }
        }
    };

    Ok(MethodInfo {
        name,
        has_self,
        is_mut,
        param_types,
        param_names,
        return_type,
        skip,
        r_name,
    })
}

#[cfg(test)]
mod tests;
// endregion
