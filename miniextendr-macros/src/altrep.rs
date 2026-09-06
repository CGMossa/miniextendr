//! ALTREP registration code generation.
//!
//! This module generates the full ALTREP registration stack for data structs:
//! `TypedExternal`, `AltrepClass`, `RegisterAltrep`, `IntoR`, linkme entry,
//! and `Ref`/`Mut` accessor types.
//!
//! # Usage
//!
//! For types with field-based derives (auto-generates trait impls):
//! ```ignore
//! #[derive(AltrepInteger)]
//! #[altrep(len = "len", elt = "value", class = "MyConstInt")]
//! struct MyConstInt { value: i32, len: usize }
//! ```
//!
//! For types with manual trait impls (lowlevel + registration, user writes data traits):
//! ```ignore
//! #[derive(AltrepInteger)]
//! #[altrep(manual, class = "MyCustom", serialize)]
//! struct MyCustomData { ... }
//!
//! impl AltrepLen for MyCustomData { ... }
//! impl AltIntegerData for MyCustomData { ... }
//! // Family derived from AltrepInteger — generates Altrep, AltVec, AltInteger, InferBase.
//! ```

/// Generates full ALTREP registration for a data struct.
///
/// Generates TypedExternal, AltrepClass, RegisterAltrep, IntoR, linkme entry, and Ref/Mut.
/// The struct must already implement the low-level ALTREP traits (via `impl_alt*_from_data!`
/// or `#[derive(AltrepInteger)]`) and `InferBase`.
pub(crate) fn generate_direct_altrep_registration(
    ident: &syn::Ident,
    generics: &syn::Generics,
    class_name: &str,
) -> syn::Result<proc_macro2::TokenStream> {
    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Class name as CStr literal
    let class_cstr = syn::LitCStr::new(&std::ffi::CString::new(class_name).unwrap(), ident.span());

    // TypedExternal constants — needed for ExternalPtr<T> to work
    let type_name_str = class_name;
    let type_name_bytes = format!("{}\0", type_name_str);
    let type_name_byte_lit = syn::LitByteStr::new(type_name_bytes.as_bytes(), ident.span());

    let ref_ident = quote::format_ident!("{}Ref", ident);
    let mut_ident = quote::format_ident!("{}Mut", ident);

    let into_r_doc = format!(
        "Convert [`{}`] to an R ALTREP SEXP.\n\nIn debug builds, asserts that we're on R's main thread.",
        ident
    );
    let ref_doc = format!(
        "Immutable reference wrapper for [`{}`] ALTREP data. Implements `TryFromSexp` and `Deref<Target = {}>`.",
        ident, ident
    );
    let mut_doc = format!(
        "Mutable reference wrapper for [`{}`] ALTREP data. Implements `TryFromSexp`, `Deref`, and `DerefMut`.",
        ident
    );

    // For non-generic types, emit a registration fn + a distributed_slice
    // entry pairing the fn pointer with its `#[no_mangle]` symbol name.
    //
    // The function is `pub extern "C"` with `#[unsafe(no_mangle)]` so that a separate
    // compilation unit (the WASM snapshot codegen path) can reference it by name via
    // an `extern { fn __mx_altrep_reg_<crate>_<Ident>(); }` declaration (crate-prefixed
    // for webR cross-package symbol uniqueness — #1273). The entry static
    // carries the symbol string for the host-time snapshot writer (so it doesn't have
    // to recover the name from a fn pointer).
    //
    // The ident is used verbatim (no `.to_lowercase()`) to avoid case-collision footguns:
    // `MyType` vs `MYType` would both produce the same symbol name if lowercased.
    let altrep_reg_entry = if generics.params.is_empty() {
        let reg_fn_name = crate::naming::altrep_reg_fn_ident(ident);
        let entry_ident = quote::format_ident!("__MX_ALTREP_REG_ENTRY_{}", ident);
        quote::quote! {
            #[doc(hidden)]
            #[unsafe(no_mangle)]
            pub extern "C" fn #reg_fn_name() {
                <#ident as ::miniextendr_api::altrep_registration::RegisterAltrep>::get_or_init_class();
            }

            #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_ALTREP_REGISTRATIONS), linkme(crate = ::miniextendr_api::linkme))]
            #[doc(hidden)]
            #[allow(non_upper_case_globals)]
            static #entry_ident: ::miniextendr_api::registry::AltrepRegistration =
                ::miniextendr_api::registry::AltrepRegistration {
                    register: #reg_fn_name,
                    symbol: stringify!(#reg_fn_name),
                };
        }
    } else {
        quote::quote! {}
    };

    let source_loc_doc = crate::source_location_doc(ident.span());

    Ok(quote::quote! {
        // TypedExternal — enables ExternalPtr<T> storage.
        // NOTE: We intentionally do NOT implement IntoExternalPtr, because ALTREP types
        // have their own IntoR impl that creates an ALTREP SEXP (not a plain ExternalPtr).
        impl ::miniextendr_api::externalptr::TypedExternal for #ident #ty_generics #where_clause {
            const TYPE_NAME: &'static str = #type_name_str;
            const TYPE_NAME_CSTR: &'static [u8] = #type_name_byte_lit;
            const TYPE_ID_CSTR: &'static [u8] =
                concat!(module_path!(), "::", stringify!(#ident), "\0").as_bytes();
        }

        // AltrepClass — class name and base type
        #[doc = concat!("ALTREP class descriptor for [`", stringify!(#ident), "`].")]
        #[doc = #source_loc_doc]
        impl ::miniextendr_api::altrep::AltrepClass for #ident #ty_generics #where_clause {
            const CLASS_NAME: &'static ::core::ffi::CStr = #class_cstr;
            const BASE: ::miniextendr_api::altrep::RBase =
                <#ident #ty_generics as ::miniextendr_api::altrep_data::InferBase>::BASE;
        }

        // RegisterAltrep — OnceLock class registration via InferBase
        #[doc = concat!("Registration entry point for [`", stringify!(#ident), "`] ALTREP class.")]
        #[doc = #source_loc_doc]
        impl ::miniextendr_api::altrep_registration::RegisterAltrep for #ident #ty_generics #where_clause {
            fn get_or_init_class() -> ::miniextendr_api::sys::altrep::R_altrep_class_t {
                use ::std::sync::OnceLock;
                static CLASS: OnceLock<::miniextendr_api::sys::altrep::R_altrep_class_t> = OnceLock::new();
                *CLASS.get_or_init(move || {
                    let cls = unsafe {
                        <#ident as ::miniextendr_api::altrep_data::InferBase>::make_class(
                            <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                            ::miniextendr_api::AltrepPkgName::as_ptr(),
                        )
                    };
                    unsafe {
                        <#ident as ::miniextendr_api::altrep_data::InferBase>::install_methods(cls);
                    }
                    cls
                })
            }
        }

        // IntoR — convert to R ALTREP SEXP (wraps self in ExternalPtr)
        #[doc = #into_r_doc]
        impl ::miniextendr_api::IntoR for #ident #ty_generics #where_clause {
            type Error = ::core::convert::Infallible;

            fn try_into_sexp(self) -> ::core::result::Result<::miniextendr_api::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }

            unsafe fn try_into_sexp_unchecked(self) -> ::core::result::Result<::miniextendr_api::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }

            fn into_sexp(self) -> ::miniextendr_api::SEXP {
                use ::miniextendr_api::altrep_registration::RegisterAltrep;
                use ::miniextendr_api::externalptr::ExternalPtr;
                use ::miniextendr_api::SEXP;
                use ::miniextendr_api::sys::{Rf_protect, Rf_unprotect};

                let ext_ptr = ExternalPtr::new(self);
                let cls = Self::get_or_init_class();
                let data1 = ext_ptr.as_sexp();
                unsafe {
                    Rf_protect(data1);
                    let altrep = cls.new_altrep(data1, SEXP::nil());
                    Rf_unprotect(1);
                    altrep
                }
            }

            unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::SEXP {
                use ::miniextendr_api::altrep_registration::RegisterAltrep;
                use ::miniextendr_api::externalptr::ExternalPtr;
                use ::miniextendr_api::sys::{Rf_protect_unchecked, Rf_unprotect_unchecked};

                let ext_ptr = ExternalPtr::new_unchecked(self);
                let cls = Self::get_or_init_class();
                let data1 = ext_ptr.as_sexp();
                unsafe {
                    Rf_protect_unchecked(data1);
                    let altrep = cls.new_altrep_unchecked(
                        data1,
                        ::miniextendr_api::SEXP::nil(),
                    );
                    Rf_unprotect_unchecked(1);
                    altrep
                }
            }
        }

        // Ref/Mut accessor types for receiving ALTREP back from R
        #[doc = #ref_doc]
        pub struct #ref_ident(::miniextendr_api::externalptr::ExternalPtr<#ident #ty_generics>);

        impl ::miniextendr_api::TryFromSexp for #ref_ident {
            type Error = ::miniextendr_api::SexpTypeError;

            fn try_from_sexp(sexp: ::miniextendr_api::SEXP) -> ::core::result::Result<Self, Self::Error> {
                use ::miniextendr_api::SEXPTYPE;

                if !::miniextendr_api::SexpExt::is_altrep(&sexp) {
                    return Err(::miniextendr_api::SexpTypeError {
                        expected: <#ident #ty_generics as ::miniextendr_api::altrep::AltrepClass>::BASE.sexptype(),
                        actual: ::miniextendr_api::SexpExt::type_of(&sexp),
                    });
                }

                match unsafe { ::miniextendr_api::altrep_data1_as::<#ident #ty_generics>(sexp) } {
                    Some(ptr) => Ok(#ref_ident(ptr)),
                    None => Err(::miniextendr_api::SexpTypeError {
                        expected: SEXPTYPE::EXTPTRSXP,
                        actual: ::miniextendr_api::SexpExt::type_of(&sexp),
                    }),
                }
            }
        }

        impl ::core::ops::Deref for #ref_ident {
            type Target = #ident #ty_generics;

            fn deref(&self) -> &Self::Target {
                &*self.0
            }
        }

        #[doc = #mut_doc]
        pub struct #mut_ident(::miniextendr_api::externalptr::ExternalPtr<#ident #ty_generics>);

        impl ::miniextendr_api::TryFromSexp for #mut_ident {
            type Error = ::miniextendr_api::SexpTypeError;

            fn try_from_sexp(sexp: ::miniextendr_api::SEXP) -> ::core::result::Result<Self, Self::Error> {
                use ::miniextendr_api::SEXPTYPE;

                if !::miniextendr_api::SexpExt::is_altrep(&sexp) {
                    return Err(::miniextendr_api::SexpTypeError {
                        expected: <#ident #ty_generics as ::miniextendr_api::altrep::AltrepClass>::BASE.sexptype(),
                        actual: ::miniextendr_api::SexpExt::type_of(&sexp),
                    });
                }

                match unsafe { ::miniextendr_api::altrep_data1_as::<#ident #ty_generics>(sexp) } {
                    Some(ptr) => Ok(#mut_ident(ptr)),
                    None => Err(::miniextendr_api::SexpTypeError {
                        expected: SEXPTYPE::EXTPTRSXP,
                        actual: ::miniextendr_api::SexpExt::type_of(&sexp),
                    }),
                }
            }
        }

        impl ::core::ops::Deref for #mut_ident {
            type Target = #ident #ty_generics;

            fn deref(&self) -> &Self::Target {
                &*self.0
            }
        }

        impl ::core::ops::DerefMut for #mut_ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut *self.0
            }
        }

        #altrep_reg_entry
    })
}

/// Entry point for `#[derive(Altrep)]`.
///
/// Generates ALTREP registration only (TypedExternal, AltrepClass,
/// RegisterAltrep, IntoR, linkme entry, Ref/Mut accessor types).
///
/// The struct must already have low-level ALTREP traits implemented.
/// For most use cases, prefer a family-specific derive instead:
/// `#[derive(AltrepInteger)]`, `#[derive(AltrepReal)]`, etc.
/// Those generate both the low-level traits AND registration.
/// Use `#[altrep(manual)]` on a family derive to skip data trait generation
/// when you provide your own `AltrepLen` + `Alt*Data` impls.
///
/// # Helper attributes
///
/// ```ignore
/// #[altrep(class = "CustomName")]  // override ALTREP class name (default: struct name)
/// ```
pub fn derive_altrep(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    use syn::spanned::Spanned;

    let ident = &input.ident;

    if !matches!(input.data, syn::Data::Struct(_)) {
        return Err(syn::Error::new(
            input.span(),
            "#[derive(Altrep)] can only be applied to structs",
        ));
    }

    // Parse class name from #[altrep(class = "...")]
    let mut class_name = None::<String>;

    for attr in &input.attrs {
        if !attr.path().is_ident("altrep") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("class") {
                let value: syn::LitStr = meta.value()?.parse()?;
                class_name = Some(value.value());
            } else {
                return Err(meta.error("unknown #[altrep(...)] attribute; expected `class`"));
            }
            Ok(())
        })?;
    }

    let class_name = class_name.unwrap_or_else(|| crate::naming::ident_name(ident));

    generate_direct_altrep_registration(ident, &input.generics, &class_name)
}
