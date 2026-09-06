//! # `#[derive(ExternalPtr)]` - ExternalPtr Support
//!
//! This module implements the `#[derive(ExternalPtr)]` macro which generates
//! a `TypedExternal` impl for use with `ExternalPtr<T>`.
//!
//! Trait ABI wrapper infrastructure is automatically generated when you use
//! `#[miniextendr]` on `impl Trait for Type` blocks.
//!
//! ## Sidecar codegen vs `#[miniextendr]` fns
//!
//! `#[derive(ExternalPtr)]` with `#[r_data]` fields emits a pair of C/R
//! wrappers per sidecar field (`<Type>_get_<field>` /
//! `<Type>_set_<field>`). **These wrappers do NOT go through
//! `c_wrapper_builder::CWrapperContext`** — they are hand-rolled in this
//! module so they can present the simplest signature R expects from a slot
//! accessor:
//!
//! | Aspect | `#[miniextendr]` fn / method (`c_wrapper_builder`) | Sidecar accessor (here) |
//! |---|---|---|
//! | First C param | `__miniextendr_call: SEXP` | none |
//! | C `numArgs` | 1 + user args | `1` (getter) / `2` (setter) |
//! | R `.Call` | `.Call(C_…, .call = match.call(), …)` | `.Call(C_…, x)` / `.Call(C_…, x, value)` |
//! | Error transport | tagged-condition SEXP via `with_r_unwind_protect(…, Some(call))` | tagged-condition SEXP via `with_r_unwind_protect(…, None)`; the R guard's `sys.call()` supplies attribution |
//!
//! **Do not add `.call = match.call()` to a sidecar R wrapper.** The C
//! function doesn't have a slot for it — R will throw "Incorrect number of
//! arguments" at runtime. The two paths drifting apart is tracked in #348.
//!
//! ## Usage
//!
//! ### Basic (no traits)
//!
//! ```ignore
//! #[derive(ExternalPtr)]
//! struct MyData {
//!     value: i32,
//! }
//! // Generates: impl TypedExternal for MyData { ... }
//! ```
//!
//! ### With R Sidecar Slots and Class System
//!
//! The `#[r_data]` attribute marks fields for R-side storage. Use `#[externalptr(...)]`
//! to specify a class system for appropriate R wrapper generation:
//!
//! | Class System | Attribute | R Accessors |
//! |--------------|-----------|-------------|
//! | Environment | `#[externalptr(env)]` (default) | `Type_get_field()`, `Type_set_field()` |
//! | R6 | `#[externalptr(r6)]` | Active bindings in R6Class |
//! | S3 | `#[externalptr(s3)]` | `$.class`, `$<-.class` methods |
//! | S4 | `#[externalptr(s4)]` | Slot accessors |
//! | S7 | `#[externalptr(s7)]` | Properties via `new_property()` |
//!
//! Three field tiers are supported:
//!
//! 1. **Raw SEXP** (`SEXP`) - Direct SEXP access, no conversion
//! 2. **Zero-overhead scalars** (`i32`, `f64`, `bool`, `u8`) - Direct R memory access
//! 3. **Conversion types** (anything else) - Uses `IntoR`/`TryFromSexp` traits
//!
//! ```ignore
//! #[derive(ExternalPtr)]
//! #[externalptr(r6)]  // R6 class - generates active bindings
//! pub struct MyType {
//!     pub x: i32,
//!
//!     #[r_data]
//!     r: RSidecar,  // Selector - enables R accessors for this type
//!
//!     #[r_data]
//!     pub raw_slot: SEXP,  // Raw SEXP, no conversion
//!
//!     #[r_data]
//!     pub count: i32,  // Zero-overhead: stored as R INTEGER(1)
//!
//!     #[r_data]
//!     pub score: f64,  // Zero-overhead: stored as R REAL(1)
//!
//!     #[r_data]
//!     pub name: String,  // Conversion: uses IntoR/TryFromSexp
//! }
//! // Generates: active bindings `count`, `score`, `name` in R6Class
//! ```
//!
//! ### Trait ABI wiring
//!
//! ```ignore
//! #[derive(ExternalPtr)]
//! struct MyCounter {
//!     value: i32,
//! }
//!
//! #[miniextendr]
//! impl Counter for MyCounter { /* ... */ }
//! ```
//!
//! ## Generated Types (trait impls)
//!
//! ### Wrapper Struct
//!
//! ```ignore
//! #[repr(C)]
//! struct __MxWrapperMyCounter {
//!     erased: mx_erased,  // Must be first field
//!     data: MyCounter,
//! }
//! ```
//!
//! ### Base Vtable
//!
//! ```ignore
//! static __MX_BASE_VTABLE_MYCOUNTER: mx_base_vtable = mx_base_vtable {
//!     drop: __mx_drop_mycounter,
//!     concrete_tag: TAG_MYCOUNTER,
//!     query: __mx_query_mycounter,
//! };
//! ```
//!
//! ### Query Function
//!
//! The query function maps trait tags to vtable pointers:
//!
//! ```ignore
//! unsafe extern "C" fn __mx_query_mycounter(
//!     ptr: *mut mx_erased,
//!     trait_tag: mx_tag,
//! ) -> *const c_void {
//!     if trait_tag == TAG_COUNTER {
//!         return std::ptr::from_ref(&__VTABLE_MYPKG_COUNTER_FOR_MYCOUNTER).cast::<c_void>();
//!     }
//!     std::ptr::null()
//! }
//! ```

use proc_macro2::{Span, TokenStream};
use syn::{DeriveInput, Field, Ident, Visibility};

use crate::miniextendr_impl::ClassSystem;

/// Parse `#[externalptr(...)]` attributes to extract class system.
///
/// Supported forms:
/// - `#[externalptr(env)]` - Environment style (default)
/// - `#[externalptr(r6)]` - R6 class
/// - `#[externalptr(s3)]` - S3 class
/// - `#[externalptr(s4)]` - S4 class
/// - `#[externalptr(s7)]` - S7 class
fn parse_externalptr_attrs(input: &DeriveInput) -> syn::Result<ClassSystem> {
    let mut class_system = ClassSystem::Env; // Default

    for attr in &input.attrs {
        if attr.path().is_ident("externalptr") {
            attr.parse_nested_meta(|meta| {
                let ident_str = meta
                    .path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_default();

                match ident_str.as_str() {
                    "env" => class_system = ClassSystem::Env,
                    "r6" => class_system = ClassSystem::R6,
                    "s3" => class_system = ClassSystem::S3,
                    "s4" => class_system = ClassSystem::S4,
                    "s7" => class_system = ClassSystem::S7,
                    "vctrs" => class_system = ClassSystem::Vctrs,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &meta.path,
                            format!(
                                "unknown class system '{}'; expected one of: env, r6, s3, s4, s7, vctrs",
                                ident_str
                            ),
                        ));
                    }
                }
                Ok(())
            })?;
        }
    }

    Ok(class_system)
}

/// Check if a field has the `#[r_data]` attribute.
fn has_r_data_attr(field: &Field) -> bool {
    field.attrs.iter().any(|a| a.path().is_ident("r_data"))
}

/// Parse `prop_doc = "..."` from an `#[r_data(prop_doc = "...")]` attribute.
///
/// Returns `None` if the field has no `#[r_data]` attribute, no `prop_doc` key,
/// or if the attribute has no parenthesized arguments (bare `#[r_data]`).
fn parse_r_data_prop_doc(field: &Field) -> syn::Result<Option<String>> {
    for attr in &field.attrs {
        if !attr.path().is_ident("r_data") {
            continue;
        }
        // `#[r_data]` with no arguments — no prop_doc
        if matches!(attr.meta, syn::Meta::Path(_)) {
            return Ok(None);
        }
        let mut prop_doc = None;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("prop_doc") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                prop_doc = Some(lit.value());
                Ok(())
            } else {
                Err(meta.error(format!(
                    "unknown key `{}`; supported: `prop_doc`",
                    meta.path
                        .get_ident()
                        .map(|i| i.to_string())
                        .unwrap_or_default()
                )))
            }
        })?;
        return Ok(prop_doc);
    }
    Ok(None)
}

/// Check if a field type is `RSidecar`.
///
/// Returns `true` if the last path segment of the field's type is `RSidecar`,
/// which acts as the selector marker enabling R sidecar accessor generation.
fn is_rsidecar_type(field: &Field) -> bool {
    if let syn::Type::Path(type_path) = &field.ty {
        type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident == "RSidecar")
            .unwrap_or(false)
    } else {
        false
    }
}

/// Check if a field is public.
fn is_pub(field: &Field) -> bool {
    matches!(field.vis, Visibility::Public(_))
}

/// The kind of sidecar slot, determining how getter/setter FFI functions are generated.
///
/// Each kind maps to a different codegen strategy for reading from and writing to
/// the Rust struct through R's `.Call` interface:
/// - Raw SEXP: no conversion, direct pass-through.
/// - Zero-overhead scalars: use R's `Rf_Scalar*`/`Rf_as*` for single-element coercion.
/// - Conversion: use the `IntoR`/`TryFromSexp` traits for arbitrary types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlotKind {
    /// SEXP type -- raw SEXP access (getter returns SEXP, setter takes SEXP).
    RawSexp,
    /// Zero-overhead scalar: `i32` (or `i16`/`i8`) stored as `INTEGER(1)`.
    ScalarInt,
    /// Zero-overhead scalar: `f64` (or `f32`) stored as `REAL(1)`.
    ScalarReal,
    /// Zero-overhead scalar: `bool` (or `Rbool`) stored as `LOGICAL(1)`.
    ScalarLogical,
    /// Zero-overhead scalar: `u8` stored as `RAW(1)`.
    ScalarRaw,
    /// Conversion type -- uses `IntoR`/`TryFromSexp` traits for arbitrary Rust types.
    Conversion,
}

/// Information about a single `#[r_data]`-annotated sidecar slot field.
///
/// Collected during struct field parsing and used to generate FFI getter/setter
/// functions and R wrapper code for each public slot.
struct SidecarSlot {
    /// Rust identifier of the field (e.g., `count`, `name`).
    name: Ident,
    /// Rust type of the field, used in conversion-based getter/setter codegen.
    ty: syn::Type,
    /// Zero-based index of this slot in the protection VECSXP
    /// (offset from `PROT_BASE_LEN`, which reserves slots for type ID and user data).
    index: usize,
    /// Whether the field is `pub`. Only public fields get R accessor functions.
    is_public: bool,
    /// Determines the codegen strategy for reading/writing this slot.
    kind: SlotKind,
    /// Optional documentation string for the S7 `@prop` tag.
    /// Sourced from `#[r_data(prop_doc = "...")]`. `None` means no doc was supplied;
    /// a default fallback string is used at emit time.
    prop_doc: Option<String>,
}

/// Aggregated sidecar information extracted from struct field analysis.
///
/// Contains everything needed to generate sidecar accessor code: the
/// selector presence, the list of typed slots, and the target class system.
struct SidecarInfo {
    /// Whether the struct contains an `RSidecar`-typed field marked with `#[r_data]`.
    /// At most one selector is allowed per struct.
    has_selector: bool,
    /// The `#[r_data]` slot fields (excluding the RSidecar selector itself),
    /// each carrying its index, kind, and visibility.
    slots: Vec<SidecarSlot>,
    /// The R class system chosen via `#[externalptr(...)]`, controlling the
    /// style of generated R wrapper code.
    class_system: ClassSystem,
}

/// Determine the [`SlotKind`] for a field type by inspecting its last path segment.
///
/// Recognizes `SEXP`, scalar numerics (`i32`, `i16`, `i8`, `f64`, `f32`),
/// booleans (`bool`, `Rbool`), and raw bytes (`u8`). Everything else falls
/// through to [`SlotKind::Conversion`].
fn slot_kind_for_type(ty: &syn::Type) -> SlotKind {
    if let syn::Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
    {
        let ident = &seg.ident;
        // Check for raw SEXP access
        if ident == "SEXP" {
            return SlotKind::RawSexp;
        }
        // Check for zero-overhead scalar types
        if ident == "i32" || ident == "i16" || ident == "i8" {
            return SlotKind::ScalarInt;
        }
        if ident == "f64" || ident == "f32" {
            return SlotKind::ScalarReal;
        }
        if ident == "bool" || ident == "Rbool" {
            return SlotKind::ScalarLogical;
        }
        if ident == "u8" {
            return SlotKind::ScalarRaw;
        }
    }
    // Everything else uses conversion
    SlotKind::Conversion
}

/// Parse struct fields for sidecar information.
///
/// Iterates over all fields, identifying `#[r_data]` markers. Fields with
/// `RSidecar` type are tracked as selector markers (at most one allowed);
/// all other `#[r_data]` fields become [`SidecarSlot`] entries with their
/// slot kind inferred from the field type.
///
/// Returns `Err` if more than one `RSidecar` field is found.
fn parse_sidecar_info(input: &DeriveInput, class_system: ClassSystem) -> syn::Result<SidecarInfo> {
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return Ok(SidecarInfo {
                has_selector: false,
                slots: vec![],
                class_system,
            });
        }
    };

    let mut selector_fields: Vec<&Field> = vec![];
    let mut slots = vec![];
    let mut slot_index = 0usize;

    for field in fields.iter() {
        if !has_r_data_attr(field) {
            continue;
        }

        if is_rsidecar_type(field) {
            // RSidecar is the selector marker, not a slot
            selector_fields.push(field);
        } else if let Some(ref ident) = field.ident {
            // Any other type with #[r_data] becomes a slot
            let kind = slot_kind_for_type(&field.ty);
            let prop_doc = parse_r_data_prop_doc(field)?;
            slots.push(SidecarSlot {
                name: ident.clone(),
                ty: field.ty.clone(),
                index: slot_index,
                is_public: is_pub(field),
                kind,
                prop_doc,
            });
            slot_index += 1;
        }
    }

    // Check for multiple selectors
    if selector_fields.len() > 1 {
        return Err(syn::Error::new_spanned(
            selector_fields[1],
            "only one RSidecar field is allowed per struct",
        ));
    }

    Ok(SidecarInfo {
        has_selector: !selector_fields.is_empty(),
        slots,
        class_system,
    })
}

/// Generate the token stream for a sidecar getter function body.
///
/// Reads the field value from the Rust struct (accessed via the external
/// pointer address) and converts it to an R SEXP. The conversion strategy
/// depends on the slot kind:
/// - `RawSexp`: returns the SEXP field directly.
/// - Scalar kinds: wraps in `Rf_Scalar*` for zero-overhead conversion.
/// - `Conversion`: clones the value and calls `IntoR::into_sexp`.
///
/// The body runs inside `with_r_unwind_protect` (see the emission site in
/// [`generate_sidecar_accessors`]); a non-external-pointer argument, a null
/// pointer address, or a wrong stored type panics with the same
/// `expected ExternalPtr<T>` message the main class-method path uses. The
/// panic is transported as a tagged condition and re-raised by the R wrapper.
fn generate_getter_body(
    struct_name: &syn::Ident,
    slot: &SidecarSlot,
    _prot_index_lit: &syn::LitInt,
) -> TokenStream {
    let field_name = &slot.name;

    // Helper: generate the pointer extraction code for Box<dyn Any> storage.
    // R_ExternalPtrAddr returns *mut Box<dyn Any>; we downcast to &T.
    // Failure modes panic (caught by the surrounding with_r_unwind_protect
    // and raised as an R error) instead of silently returning R_NilValue.
    let extract_ref = quote::quote! {
        use ::miniextendr_api::{SEXP, SexpExt};
        use ::miniextendr_api::sys::R_ExternalPtrAddr;
        if x.type_of() != ::miniextendr_api::SEXPTYPE::EXTPTRSXP {
            ::std::panic!(concat!(
                "expected ExternalPtr<", stringify!(#struct_name),
                ">, got a non-external-pointer object"
            ));
        }
        let any_raw = R_ExternalPtrAddr(x) as *mut Box<dyn ::std::any::Any>;
        if any_raw.is_null() {
            ::std::panic!(concat!(
                "expected ExternalPtr<", stringify!(#struct_name),
                ">, got a null external pointer"
            ));
        }
        let any_box: &Box<dyn ::std::any::Any> = &*any_raw;
        let data: &#struct_name = any_box
            .downcast_ref::<#struct_name>()
            .expect(concat!("expected ExternalPtr<", stringify!(#struct_name), ">"));
    };

    match slot.kind {
        SlotKind::RawSexp => {
            // Raw SEXP field - return directly (already an R value)
            quote::quote! {
                unsafe {
                    #extract_ref
                    data.#field_name
                }
            }
        }
        SlotKind::ScalarInt => {
            quote::quote! {
                use ::miniextendr_api::SEXP;
                unsafe {
                    #extract_ref
                    SEXP::scalar_integer(data.#field_name)
                }
            }
        }
        SlotKind::ScalarReal => {
            quote::quote! {
                use ::miniextendr_api::SEXP;
                unsafe {
                    #extract_ref
                    SEXP::scalar_real(data.#field_name)
                }
            }
        }
        SlotKind::ScalarLogical => {
            quote::quote! {
                use ::miniextendr_api::SEXP;
                unsafe {
                    #extract_ref
                    SEXP::scalar_logical(data.#field_name)
                }
            }
        }
        SlotKind::ScalarRaw => {
            quote::quote! {
                use ::miniextendr_api::SEXP;
                unsafe {
                    #extract_ref
                    SEXP::scalar_raw(data.#field_name)
                }
            }
        }
        SlotKind::Conversion => {
            // Use IntoR trait for conversion (e.g., String -> character)
            let ty = &slot.ty;
            quote::quote! {
                use ::miniextendr_api::into_r::IntoR;
                unsafe {
                    #extract_ref
                    let val: #ty = data.#field_name.clone();
                    <#ty as IntoR>::into_sexp(val)
                }
            }
        }
    }
}

/// Generate the token stream for a sidecar setter function body.
///
/// Converts the incoming R SEXP `value` and writes it to the corresponding
/// Rust struct field. The conversion strategy depends on the slot kind:
/// - `RawSexp`: stores the SEXP directly.
/// - Scalar kinds: uses `Rf_as*` or coercion for single-element extraction;
///   an input that doesn't reduce to a single non-NA scalar raises a
///   conversion error instead of silently storing an NA/`false` sentinel.
/// - `Conversion`: uses `TryFromSexp::try_from_sexp`; a failed conversion
///   raises a conversion error instead of silently dropping the write.
///
/// Returns the external pointer `x` (for R's invisible return convention).
/// The body runs inside `with_r_unwind_protect` (see the emission site in
/// [`generate_sidecar_accessors`]); a bad receiver panics with the main-path
/// `expected ExternalPtr<T>` message and conversion failures return a tagged
/// condition SEXP with kind `conversion` — both are re-raised by the R
/// wrapper. Sidecar accessors have no `__miniextendr_call` slot (#344/#348),
/// so the tagged conditions carry null call attribution.
fn generate_setter_body(
    struct_name: &syn::Ident,
    slot: &SidecarSlot,
    _prot_index_lit: &syn::LitInt,
) -> TokenStream {
    let field_name = &slot.name;

    // Helper: extract mutable reference via Box<dyn Any> downcast.
    // Failure modes panic (caught by the surrounding with_r_unwind_protect
    // and raised as an R error) instead of silently no-op'ing.
    let extract_mut = quote::quote! {
        use ::miniextendr_api::SexpExt;
        use ::miniextendr_api::sys::R_ExternalPtrAddr;
        if x.type_of() != ::miniextendr_api::SEXPTYPE::EXTPTRSXP {
            ::std::panic!(concat!(
                "expected ExternalPtr<", stringify!(#struct_name),
                ">, got a non-external-pointer object"
            ));
        }
        let any_raw = R_ExternalPtrAddr(x) as *mut Box<dyn ::std::any::Any>;
        if any_raw.is_null() {
            ::std::panic!(concat!(
                "expected ExternalPtr<", stringify!(#struct_name),
                ">, got a null external pointer"
            ));
        }
        let any_box: &mut Box<dyn ::std::any::Any> = &mut *any_raw;
        let data = any_box
            .downcast_mut::<#struct_name>()
            .expect(concat!("expected ExternalPtr<", stringify!(#struct_name), ">"));
    };

    // Conversion-failure error message prefix, mirroring the main path's
    // "failed to convert parameter '<name>' to <ty>: ..." wording.
    let err_prefix = format!(
        "failed to convert value for sidecar field '{}' on `{}`",
        field_name, struct_name
    );
    let scalar_err = |expected: &str| -> syn::LitStr {
        syn::LitStr::new(
            &format!("{err_prefix}: expected {expected}"),
            field_name.span(),
        )
    };

    match slot.kind {
        SlotKind::RawSexp => {
            quote::quote! {
                unsafe {
                    #extract_mut
                    data.#field_name = value;
                    x
                }
            }
        }
        SlotKind::ScalarInt => {
            let err_msg = scalar_err("a single non-NA integer-compatible value");
            quote::quote! {
                use ::miniextendr_api::SexpExt;
                unsafe {
                    #extract_mut
                    data.#field_name = match value.as_integer() {
                        Some(v) => v,
                        None => return ::miniextendr_api::error_value::make_rust_condition_value(
                            #err_msg,
                            ::miniextendr_api::error_value::kind::CONVERSION,
                            ::core::option::Option::None,
                            ::core::option::Option::None,
                        ),
                    };
                    x
                }
            }
        }
        SlotKind::ScalarReal => {
            let err_msg = scalar_err("a single non-NA numeric-compatible value");
            quote::quote! {
                use ::miniextendr_api::SexpExt;
                unsafe {
                    #extract_mut
                    data.#field_name = match value.as_real() {
                        Some(v) => v,
                        None => return ::miniextendr_api::error_value::make_rust_condition_value(
                            #err_msg,
                            ::miniextendr_api::error_value::kind::CONVERSION,
                            ::core::option::Option::None,
                            ::core::option::Option::None,
                        ),
                    };
                    x
                }
            }
        }
        SlotKind::ScalarLogical => {
            let err_msg = scalar_err("a single non-NA logical-compatible value");
            quote::quote! {
                use ::miniextendr_api::SexpExt;
                unsafe {
                    #extract_mut
                    data.#field_name = match value.as_logical() {
                        Some(v) => v,
                        None => return ::miniextendr_api::error_value::make_rust_condition_value(
                            #err_msg,
                            ::miniextendr_api::error_value::kind::CONVERSION,
                            ::core::option::Option::None,
                            ::core::option::Option::None,
                        ),
                    };
                    x
                }
            }
        }
        SlotKind::ScalarRaw => {
            let err_msg = scalar_err("at least one raw-compatible value");
            quote::quote! {
                use ::miniextendr_api::{SexpExt, SEXPTYPE};
                unsafe {
                    #extract_mut
                    let raw_vec = value.coerce(SEXPTYPE::RAWSXP);
                    if raw_vec.len() == 0 {
                        return ::miniextendr_api::error_value::make_rust_condition_value(
                            #err_msg,
                            ::miniextendr_api::error_value::kind::CONVERSION,
                            ::core::option::Option::None,
                            ::core::option::Option::None,
                        );
                    }
                    data.#field_name = raw_vec.raw_elt(0);
                    x
                }
            }
        }
        SlotKind::Conversion => {
            let ty = &slot.ty;
            let err_msg_lit = syn::LitStr::new(&err_prefix, field_name.span());
            quote::quote! {
                use ::miniextendr_api::TryFromSexp;
                unsafe {
                    #extract_mut
                    data.#field_name = match <#ty as TryFromSexp>::try_from_sexp(value) {
                        Ok(val) => val,
                        Err(e) => return ::miniextendr_api::error_value::make_rust_condition_value(
                            &format!("{}: {e}", #err_msg_lit),
                            ::miniextendr_api::error_value::kind::CONVERSION,
                            ::core::option::Option::None,
                            ::core::option::Option::None,
                        ),
                    };
                    x
                }
            }
        }
    }
}

/// Generate R wrapper code (roxygen-annotated R functions) for a single sidecar slot.
///
/// Produces getter and setter R functions that call the corresponding C entry
/// points via `.Call`. The generated R code includes roxygen tags (`@rdname`,
/// `@param`, `@return`, `@export`) for documentation.
///
/// All class systems generate the same standalone function pattern
/// (`Type_get_field` / `Type_set_field`); only the roxygen title suffix
/// differs. Class-specific integration (e.g., R6 active bindings, S7
/// properties) is handled separately by [`generate_class_integration_r_code`].
///
/// Each wrapper body carries the shared tagged-condition guard
/// ([`crate::method_return_builder::standalone_body`]) so Rust-origin
/// panics/conversion failures are re-raised as structured R conditions.
/// Note the sidecar C functions have **no** `.call` slot — the `.Call()` must
/// not pass `.call = match.call()` (#344/#348); `sys.call()` inside the guard
/// supplies fallback attribution instead.
fn generate_r_wrapper_for_slot(
    class_system: ClassSystem,
    type_name: &str,
    field_name: &str,
    getter_c_name: &str,
    setter_c_name: &str,
) -> String {
    // Only the roxygen title suffix differs between class systems.
    let suffix = match class_system {
        ClassSystem::Env => "",
        ClassSystem::R6 => " (for R6)",
        ClassSystem::S3 => " (for S3)",
        ClassSystem::S4 => " (for S4)",
        ClassSystem::S7 => " (for S7)",
        ClassSystem::Vctrs => " (for vctrs)",
    };
    let r_getter_name = format!("{}_get_{}", type_name, field_name);
    let r_setter_name = format!("{}_set_{}", type_name, field_name);
    let getter_body = crate::method_return_builder::standalone_body(
        &format!(".Call({getter_c_name}, x)"),
        ".val",
        "  ",
    );
    let setter_body = crate::method_return_builder::standalone_body(
        &format!(".Call({setter_c_name}, x, value)"),
        "invisible(x)",
        "  ",
    );
    format!(
        r#"
#' Get `{field}` field from {type}{suffix}
#' @rdname {type}
#' @param x The {type} external pointer
#' @return The value of the `{field}` field
#' @export
{r_getter} <- function(x) {{
  {getter_body}
}}

#' Set `{field}` field on {type}{suffix}
#' @rdname {type}
#' @param x The {type} external pointer
#' @param value The new value to set
#' @return The {type} pointer (invisibly)
#' @export
{r_setter} <- function(x, value) {{
  {setter_body}
}}
"#,
        type = type_name,
        field = field_name,
        suffix = suffix,
        r_getter = r_getter_name,
        r_setter = r_setter_name,
        getter_body = getter_body,
        setter_body = setter_body,
    )
}

/// Generate class-integrated R code for sidecar fields.
///
/// For R6: generates `Type$set("active", "field", ...)` calls that add active bindings
/// referencing the sidecar getter/setter .Call entrypoints.
///
/// For S7: generates `.rdata_properties_Type <- list(...)` with S7::new_property()
/// definitions that can be spliced into the S7 class's properties list.
///
/// Other class systems return empty strings (their standalone accessors suffice).
fn generate_class_integration_r_code(
    class_system: ClassSystem,
    type_name: &str,
    pub_slots: &[&SidecarSlot],
) -> String {
    if pub_slots.is_empty() {
        return String::new();
    }

    // Shared tagged-condition guard (one line); `sys.call()` supplies fallback
    // attribution because the sidecar C functions carry no `.call` slot
    // (#344/#348 — never add `.call = match.call()` to a sidecar `.Call()`).
    let guard =
        |indent: &str| crate::method_return_builder::condition_check_lines(indent).join("\n");

    match class_system {
        ClassSystem::R6 => {
            // Generate $set("active", ...) calls for each sidecar field.
            // These are appended after the R6Class definition and add active bindings
            // that delegate to the sidecar .Call accessors.
            let mut code = String::new();
            code.push_str(&format!(
                "\n# Auto-generated active bindings for {type} sidecar fields.\n",
                type = type_name,
            ));
            code.push_str(
                "# These are applied when `r_data_accessors` is set on the impl block.\n",
            );
            code.push_str(&format!(
                ".rdata_active_bindings_{type} <- function(cls) {{\n\
                 \x20 # R CMD check: self/private are R6 runtime bindings (set by cls$set)\n\
                 \x20 self <- private <- NULL\n",
                type = type_name,
            ));
            for slot in pub_slots {
                let field = crate::naming::ident_name(&slot.name);
                // Must match the sidecar accessor's actual C symbol exactly (#1273
                // crate-prefixing) — routed through the shared naming.rs helpers.
                let getter_c = crate::naming::sidecar_getter_c_name(type_name, &field);
                let setter_c = crate::naming::sidecar_setter_c_name(type_name, &field);
                code.push_str(&format!(
                    "  cls$set(\"active\", \"{field}\", function(value) {{\n\
                     \x20   if (missing(value)) {{\n\
                     \x20     .val <- .Call({getter_c}, private$.ptr)\n\
                     {getter_guard}\n\
                     \x20     .val\n\
                     \x20   }} else {{\n\
                     \x20     .val <- .Call({setter_c}, private$.ptr, value)\n\
                     {setter_guard}\n\
                     \x20     invisible(self)\n\
                     \x20   }}\n\
                     \x20 }}, overwrite = TRUE)\n",
                    field = field,
                    getter_c = getter_c,
                    setter_c = setter_c,
                    getter_guard = guard("      "),
                    setter_guard = guard("      "),
                ));
            }
            code.push_str("}\n");
            code
        }
        ClassSystem::S7 => {
            // Generate a helper list of S7::new_property() definitions.
            // The S7 wrapper generator references this when `r_data_accessors` is set.
            let mut code = String::new();
            code.push_str(&format!(
                "\n# Auto-generated S7 property definitions for {type} sidecar fields.\n",
                type = type_name,
            ));
            code.push_str(&format!(
                ".rdata_properties_{type} <- list(\n",
                type = type_name,
            ));
            for (i, slot) in pub_slots.iter().enumerate() {
                let field = crate::naming::ident_name(&slot.name);
                // Must match the sidecar accessor's actual C symbol exactly (#1273
                // crate-prefixing) — routed through the shared naming.rs helpers.
                let getter_c = crate::naming::sidecar_getter_c_name(type_name, &field);
                let setter_c = crate::naming::sidecar_setter_c_name(type_name, &field);
                let comma = if i < pub_slots.len() - 1 { "," } else { "" };
                code.push_str(&format!(
                    "    {field} = S7::new_property(\n\
                     \x20       getter = function(self) {{\n\
                     \x20         .val <- .Call({getter_c}, self@.ptr)\n\
                     {getter_guard}\n\
                     \x20         .val\n\
                     \x20       }},\n\
                     \x20       setter = function(self, value) {{\n\
                     \x20         .val <- .Call({setter_c}, self@.ptr, value)\n\
                     {setter_guard}\n\
                     \x20         self\n\
                     \x20       }}\n\
                     \x20   ){comma}\n",
                    field = field,
                    getter_c = getter_c,
                    setter_c = setter_c,
                    getter_guard = guard("          "),
                    setter_guard = guard("          "),
                    comma = comma,
                ));
            }
            code.push_str(")\n");
            code
        }
        // Other class systems don't need class-integrated code
        _ => String::new(),
    }
}

/// Generate sidecar accessor constants and `extern "C-unwind"` functions.
///
/// For each public `#[r_data]` field, generates:
/// - A getter FFI function (`C_<crate>__mx_rdata_get_Type_field`)
/// - A setter FFI function (`C_<crate>__mx_rdata_set_Type_field`)
/// - `R_CallMethodDef` entries for routine registration
/// - R wrapper function code (roxygen-documented)
/// - Class-integration code for R6 / S7 (active bindings / properties)
///
/// The generated constants are:
/// - `RDATA_CALL_DEFS_{TYPE}`: slice of `R_CallMethodDef` for registration
/// - `R_WRAPPERS_RDATA_{TYPE}`: string literal of R wrapper code
///
/// Returns `Err` if the struct has generic type parameters (`.Call` entrypoints
/// cannot be generic).
fn generate_sidecar_accessors(input: &DeriveInput, info: &SidecarInfo) -> syn::Result<TokenStream> {
    // Reject generic structs — .Call entrypoints cannot be generic
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "ExternalPtr does not support generic structs; \
             .Call entrypoints cannot be generic",
        ));
    }

    // If no selector or no public slots, nothing to register
    let pub_slots: Vec<_> = info.slots.iter().filter(|s| s.is_public).collect();
    if !info.has_selector || pub_slots.is_empty() {
        return Ok(quote::quote! {});
    }

    let name = &input.ident;
    let name_str = name.to_string();
    let name_upper = name_str.to_uppercase();

    // Base prot slot indices (type ID at 0, user at 1)
    const PROT_BASE_LEN: usize = 2;

    // Generate getter/setter functions and R wrappers for each pub slot
    let mut c_functions = vec![];
    let mut r_wrappers = String::new();

    // Add documentation header that establishes the @name topic for sidecar accessors.
    // This ensures @rdname references have a valid target even if the main class
    // definition uses @noRd.
    if !pub_slots.is_empty() {
        r_wrappers.push_str(&format!(
            r#"
#' @title {type} Sidecar Accessors
#' @name {type}
#' @description Getter and setter functions for `#[r_data]` fields on `{type}`.
#' @source Generated by miniextendr from `#[derive(ExternalPtr)]` on `{type}`
NULL

"#,
            type = name_str,
        ));
    }

    for slot in &pub_slots {
        let field_name = &slot.name;
        let field_name_str = crate::naming::ident_name(field_name);
        let prot_index = PROT_BASE_LEN + slot.index;

        // C function names (crate-prefixed for webR cross-package symbol
        // uniqueness — #1273, routed through the shared naming.rs helpers)
        let getter_c_name = crate::naming::sidecar_getter_c_name(&name_str, &field_name_str);
        let setter_c_name = crate::naming::sidecar_setter_c_name(&name_str, &field_name_str);
        let getter_fn_name = Ident::new(&getter_c_name, Span::call_site());
        let setter_fn_name = Ident::new(&setter_c_name, Span::call_site());
        let source_location_doc = crate::source_location_doc(field_name.span());
        let getter_doc = format!(
            "Generated sidecar getter for `{}` field on Rust type `{}`.",
            field_name_str, name_str
        );
        let setter_doc = format!(
            "Generated sidecar setter for `{}` field on Rust type `{}`.",
            field_name_str, name_str
        );
        let getter_doc_lit = syn::LitStr::new(&getter_doc, field_name.span());
        let setter_doc_lit = syn::LitStr::new(&setter_doc, field_name.span());

        let prot_index_lit = syn::LitInt::new(&prot_index.to_string(), Span::call_site());

        // Generate getter/setter bodies based on slot kind
        let getter_body = generate_getter_body(name, slot, &prot_index_lit);
        let setter_body = generate_setter_body(name, slot, &prot_index_lit);

        // Generate C getter function.
        //
        // The body runs under `with_r_unwind_protect` so Rust panics (and R
        // longjmps during allocation) become a tagged condition SEXP that the
        // R wrapper re-raises, matching the main call-slot path. Sidecar
        // accessors have no `__miniextendr_call` slot (#344/#348), so the
        // transport uses null call attribution; the R wrapper's guard passes
        // `sys.call()` to `.miniextendr_raise_condition` as the fallback.
        c_functions.push(quote::quote! {
            #[doc = #getter_doc_lit]
            #[doc = #source_location_doc]
            #[doc = concat!("Generated from source file `", file!(), "`.")]
            #[doc(hidden)]
            #[unsafe(no_mangle)]
            pub unsafe extern "C-unwind" fn #getter_fn_name(
                x: ::miniextendr_api::SEXP
            ) -> ::miniextendr_api::SEXP {
                ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                    || { #getter_body },
                    ::core::option::Option::None,
                )
            }
        });

        // Generate C setter function (same unwind/condition transport as the
        // getter above).
        c_functions.push(quote::quote! {
            #[doc = #setter_doc_lit]
            #[doc = #source_location_doc]
            #[doc = concat!("Generated from source file `", file!(), "`.")]
            #[doc(hidden)]
            #[unsafe(no_mangle)]
            pub unsafe extern "C-unwind" fn #setter_fn_name(
                x: ::miniextendr_api::SEXP,
                value: ::miniextendr_api::SEXP,
            ) -> ::miniextendr_api::SEXP {
                ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                    || { #setter_body },
                    ::core::option::Option::None,
                )
            }
        });

        // Generate R_CallMethodDef entries via distributed slice
        let getter_c_name_cstr = format!("{}\0", getter_c_name);
        let setter_c_name_cstr = format!("{}\0", setter_c_name);
        let getter_cstr_lit =
            syn::LitByteStr::new(getter_c_name_cstr.as_bytes(), Span::call_site());
        let setter_cstr_lit =
            syn::LitByteStr::new(setter_c_name_cstr.as_bytes(), Span::call_site());
        let getter_def_ident = Ident::new(
            &format!(
                "__MX_CALL_DEF_RDATA_GET_{}_{}",
                name_upper,
                field_name_str.to_uppercase()
            ),
            Span::call_site(),
        );
        let setter_def_ident = Ident::new(
            &format!(
                "__MX_CALL_DEF_RDATA_SET_{}_{}",
                name_upper,
                field_name_str.to_uppercase()
            ),
            Span::call_site(),
        );

        c_functions.push(quote::quote! {
            #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS), linkme(crate = ::miniextendr_api::linkme))]
            #[doc(hidden)]
            static #getter_def_ident: ::miniextendr_api::sys::R_CallMethodDef =
                ::miniextendr_api::sys::R_CallMethodDef {
                    name: #getter_cstr_lit.as_ptr().cast(),
                    fun: Some(unsafe { ::std::mem::transmute(#getter_fn_name as unsafe extern "C-unwind" fn(_) -> _) }),
                    numArgs: 1,
                };
        });
        c_functions.push(quote::quote! {
            #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS), linkme(crate = ::miniextendr_api::linkme))]
            #[doc(hidden)]
            static #setter_def_ident: ::miniextendr_api::sys::R_CallMethodDef =
                ::miniextendr_api::sys::R_CallMethodDef {
                    name: #setter_cstr_lit.as_ptr().cast(),
                    fun: Some(unsafe { ::std::mem::transmute(#setter_fn_name as unsafe extern "C-unwind" fn(_, _) -> _) }),
                    numArgs: 2,
                };
        });

        // Generate R wrapper code based on class system
        let field_start = field_name.span().start();
        r_wrappers.push_str(&format!(
            "# Generated from Rust source line {}:{}\n# Wraps sidecar field `{}` on Rust type `{}` via `{}` and `{}`.\n",
            field_start.line,
            field_start.column + 1,
            field_name_str,
            name_str,
            getter_c_name,
            setter_c_name,
        ));
        r_wrappers.push_str(&generate_r_wrapper_for_slot(
            info.class_system,
            &name_str,
            &field_name_str,
            &getter_c_name,
            &setter_c_name,
        ));
    }

    // Generate class-integrated R code for R6 and S7.
    // This code is appended after the standalone accessors so that
    // `r_data_accessors` in the impl block can auto-integrate sidecar fields.
    r_wrappers.push_str(&generate_class_integration_r_code(
        info.class_system,
        &name_str,
        &pub_slots,
    ));

    let const_name_wrappers = Ident::new(
        &format!("R_WRAPPERS_RDATA_{}", name_upper),
        Span::call_site(),
    );
    let source_location_doc = crate::source_location_doc(name.span());

    // For S7 class systems, emit MX_S7_SIDECAR_PROPS entries so the S7 codegen
    // can substitute @prop lines for sidecar properties at write time.
    let sidecar_prop_entries = if info.class_system == ClassSystem::S7 {
        let entries: Vec<_> = pub_slots
            .iter()
            .map(|slot| {
                let field_str = crate::naming::ident_name(&slot.name);
                let doc_str = slot
                    .prop_doc
                    .as_deref()
                    .unwrap_or("(undocumented sidecar property)");
                let entry_ident = Ident::new(
                    &format!(
                        "__MX_S7_SIDECAR_PROP_{}_{}",
                        name_upper,
                        field_str.to_uppercase()
                    ),
                    Span::call_site(),
                );
                quote::quote! {
                    #[doc(hidden)]
                    #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_S7_SIDECAR_PROPS), linkme(crate = ::miniextendr_api::linkme))]
                    static #entry_ident: ::miniextendr_api::registry::SidecarPropEntry =
                        ::miniextendr_api::registry::SidecarPropEntry {
                            rust_type: #name_str,
                            field_name: #field_str,
                            prop_doc: #doc_str,
                        };
                }
            })
            .collect();
        quote::quote! { #(#entries)* }
    } else {
        quote::quote! {}
    };

    Ok(quote::quote! {
        #(#c_functions)*

        /// Sidecar accessor R wrapper code via distributed slice.
        #[doc = #source_location_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[doc(hidden)]
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_R_WRAPPERS), linkme(crate = ::miniextendr_api::linkme))]
        static #const_name_wrappers: ::miniextendr_api::registry::RWrapperEntry =
            ::miniextendr_api::registry::RWrapperEntry {
                priority: ::miniextendr_api::registry::RWrapperPriority::Sidecar,
                source_file: file!(),
                content: #r_wrappers,
            };

        #sidecar_prop_entries
    })
}

/// Generate the `TypedExternal` trait implementation for the derive target.
///
/// Produces three associated constants:
/// - `TYPE_NAME`: the struct name as a `&'static str`
/// - `TYPE_NAME_CSTR`: null-terminated byte string of the struct name
/// - `TYPE_ID_CSTR`: globally unique ID in the format
///   `"<crate_name>@<crate_version>::<module_path>::<type_name>\0"`,
///   using `CARGO_PKG_NAME`, `CARGO_PKG_VERSION`, and `module_path!()`.
///
/// Supports generic structs (generics are forwarded to the impl).
fn generate_typed_external(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let name_str = name.to_string();
    let name_lit = syn::LitStr::new(&name_str, name.span());
    let name_cstr = syn::LitByteStr::new(format!("{}\0", name_str).as_bytes(), name.span());

    // TYPE_ID_CSTR format: "<crate_name>@<crate_version>::<module_path>::<type_name>\0"
    //
    // Uses env!("CARGO_PKG_NAME") and env!("CARGO_PKG_VERSION") for the crate identifier,
    // ensuring two packages with the same type name from the same crate+version are compatible,
    // while different crate versions are considered distinct types.
    //
    // The module_path!() may include "crate::" prefix when compiled within the crate,
    // but combined with the explicit crate@version prefix, this is unambiguous.
    quote::quote! {
        impl #impl_generics ::miniextendr_api::externalptr::TypedExternal for #name #ty_generics #where_clause {
            const TYPE_NAME: &'static str = #name_lit;
            const TYPE_NAME_CSTR: &'static [u8] = #name_cstr;
            const TYPE_ID_CSTR: &'static [u8] =
                concat!(
                    env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION"),
                    "::", module_path!(), "::", #name_lit, "\0"
                ).as_bytes();
        }
    }
}

/// Generate the `IntoExternalPtr` marker trait impl.
///
/// This marker trait enables the blanket `impl<T: IntoExternalPtr> IntoR for T`
/// in miniextendr-api, allowing the type to be returned directly from functions.
fn generate_into_external_ptr(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote::quote! {
        impl #impl_generics ::miniextendr_api::externalptr::IntoExternalPtr for #name #ty_generics #where_clause {}
    }
}

/// Generate the concrete `IntoRVecElement` impl (issue #1284).
///
/// Lights up the single `impl<T: IntoRVecElement> IntoR for Vec<T>` blanket in
/// miniextendr-api, so `-> Vec<MyType>` returns compile and produce a `VECSXP`
/// of external pointers. The impl must be concrete (not a blanket over
/// `IntoExternalPtr`) because the `MatchArg` bridge already occupies a blanket
/// on `IntoRVecElement` and two blankets would collide (E0119) — the same
/// funnel design `#[derive(IntoR)]` newtypes use (see
/// `miniextendr_api::newtype` module docs).
///
/// GC discipline: `ExternalPtr::collect_into_r_list` allocates each
/// `EXTPTRSXP` directly into the already-protected destination list, so no
/// element is ever live-but-unrooted across a later allocation.
fn generate_into_r_vec_element(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote::quote! {
        impl #impl_generics ::miniextendr_api::IntoRVecElement for #name #ty_generics #where_clause {
            #[inline]
            fn elements_into_sexp(values: ::std::vec::Vec<Self>) -> ::miniextendr_api::SEXP {
                ::miniextendr_api::externalptr::ExternalPtr::<Self>::collect_into_r_list(values)
            }
        }
    }
}

/// Main entry point for `#[derive(ExternalPtr)]`.
///
/// Orchestrates the full derive expansion:
/// 1. Parses `#[externalptr(...)]` attributes for class system selection.
/// 2. Analyzes struct fields for `#[r_data]` sidecar slots.
/// 3. Generates `TypedExternal` impl (type identity for `ExternalPtr<T>`).
/// 4. Generates `IntoExternalPtr` marker impl (enables `IntoR` blanket impl)
///    and a concrete `IntoRVecElement` impl (enables the `IntoR for Vec<T>`
///    blanket, so `-> Vec<MyType>` returns a `VECSXP` of external pointers —
///    #1284). Both are suppressed when `emit_into_r_marker` is `false`, which
///    the struct-level `#[miniextendr(prefer = "native")]` path uses so its
///    concrete `AsRNative` `IntoR` does not collide with the blanket
///    `IntoExternalPtr` `IntoR` (E0119, #1283).
/// 5. Generates sidecar accessor FFI functions, registration constants, and R wrappers.
///
/// Returns the combined token stream of all generated items.
pub fn derive_external_ptr(
    input: DeriveInput,
    emit_into_r_marker: bool,
) -> syn::Result<TokenStream> {
    // Parse class system from #[externalptr(...)] attribute
    let class_system = parse_externalptr_attrs(&input)?;

    // Parse sidecar information from struct fields
    let sidecar_info = parse_sidecar_info(&input, class_system)?;

    let typed_external = generate_typed_external(&input);
    let (into_external_ptr, into_r_vec_element) = if emit_into_r_marker {
        (
            generate_into_external_ptr(&input),
            generate_into_r_vec_element(&input),
        )
    } else {
        (
            proc_macro2::TokenStream::new(),
            proc_macro2::TokenStream::new(),
        )
    };
    let sidecar_accessors = generate_sidecar_accessors(&input, &sidecar_info)?;
    let erased_wrapper = generate_erased_wrapper(&input);

    Ok(quote::quote! {
        #typed_external
        #into_external_ptr
        #into_r_vec_element
        #sidecar_accessors
        #erased_wrapper
    })
}

/// Generate the type-erased wrapper infrastructure for trait ABI dispatch.
///
/// For `#[derive(ExternalPtr)] struct MyCounter { ... }` generates:
/// - `__MxWrapperMyCounter` - repr(C) wrapper with mx_erased header + data
/// - `__MX_TAG_MYCOUNTER` - concrete type tag (FNV-1a hash of module_path + type)
/// - `__mx_drop_mycounter` - destructor for R GC
/// - `__MX_BASE_VTABLE_MYCOUNTER` - base vtable with universal_query
/// - `__mx_wrap_mycounter` - constructor returning `*mut mx_erased`
fn generate_erased_wrapper(input: &DeriveInput) -> TokenStream {
    let type_ident = &input.ident;

    // Only for non-generic types (generics can't participate in trait dispatch)
    if !input.generics.params.is_empty() {
        return quote::quote! {};
    }

    let type_upper = type_ident.to_string().to_uppercase();
    let type_lower = type_ident.to_string().to_lowercase();

    let wrapper_name = quote::format_ident!("__MxWrapper{}", type_ident);
    let base_vtable_name = quote::format_ident!("__MX_BASE_VTABLE_{}", type_upper);
    let concrete_tag_name = quote::format_ident!("__MX_TAG_{}", type_upper);
    let drop_fn_name = quote::format_ident!("__mx_drop_{}", type_lower);
    let wrap_fn_name = quote::format_ident!("__mx_wrap_{}", type_lower);
    let source_loc_doc = crate::source_location_doc(type_ident.span());
    let tag_path = format!("::{}", type_ident);

    quote::quote! {
        #[doc = concat!(
            "Type-erased wrapper for `",
            stringify!(#type_ident),
            "` with trait dispatch support."
        )]
        #[doc = "Generated by `#[derive(ExternalPtr)]`."]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[repr(C)]
        #[doc(hidden)]
        struct #wrapper_name {
            pub erased: ::miniextendr_api::abi::mx_erased,
            pub data: #type_ident,
        }

        #[doc(hidden)]
        const #concrete_tag_name: ::miniextendr_api::abi::mx_tag =
            ::miniextendr_api::abi::mx_tag_from_path(concat!(module_path!(), #tag_path));

        #[doc(hidden)]
        unsafe extern "C" fn #drop_fn_name(ptr: *mut ::miniextendr_api::abi::mx_erased) {
            if ptr.is_null() {
                return;
            }
            let wrapper = ptr.cast::<#wrapper_name>();
            // A panicking Drop impl must not unwind across the C-ABI boundary.
            // `drop_catching_panic` catches any panic and aborts instead.
            ::miniextendr_api::externalptr::drop_catching_panic(|| {
                unsafe { drop(Box::from_raw(wrapper)); }
            });
        }

        #[doc(hidden)]
        static #base_vtable_name: ::miniextendr_api::abi::mx_base_vtable =
            ::miniextendr_api::abi::mx_base_vtable {
                drop: #drop_fn_name,
                concrete_tag: #concrete_tag_name,
                query: ::miniextendr_api::registry::universal_query,
                data_offset: ::std::mem::offset_of!(#wrapper_name, data),
            };

        #[doc(hidden)]
        fn #wrap_fn_name(data: #type_ident) -> *mut ::miniextendr_api::abi::mx_erased {
            let wrapper = Box::new(#wrapper_name {
                erased: ::miniextendr_api::abi::mx_erased {
                    base: &#base_vtable_name,
                },
                data,
            });
            Box::into_raw(wrapper).cast::<::miniextendr_api::abi::mx_erased>()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ClassSystem, generate_class_integration_r_code, generate_r_wrapper_for_slot};

    const ALL_CLASS_SYSTEMS: [ClassSystem; 6] = [
        ClassSystem::Env,
        ClassSystem::R6,
        ClassSystem::S3,
        ClassSystem::S4,
        ClassSystem::S7,
        ClassSystem::Vctrs,
    ];

    /// Sidecar accessor C functions take only `x` (getter) or `x, value` (setter) —
    /// no `__miniextendr_call` parameter. The R wrappers must NOT pass `.call = match.call()`
    /// because that would be counted as an extra positional argument by `.Call()`, causing
    /// "Incorrect number of arguments" errors at runtime. Cover every class system variant.
    #[test]
    fn sidecar_accessors_do_not_pass_match_call() {
        let getter_c = "C__mx_rdata_get_T_f";
        let setter_c = "C__mx_rdata_set_T_f";

        for cs in ALL_CLASS_SYSTEMS {
            let out = generate_r_wrapper_for_slot(cs, "T", "f", getter_c, setter_c);
            // Correct form: no .call argument — the C function only accepts x (getter) or x, value (setter).
            assert!(
                out.contains(&format!(".Call({getter_c}, x)")),
                "{cs:?} getter should call without .call:\n{out}"
            );
            assert!(
                out.contains(&format!(".Call({setter_c}, x, value)")),
                "{cs:?} setter should call without .call:\n{out}"
            );
            // Must NOT include the erroneous .call = match.call() form.
            assert!(
                !out.contains(".call = match.call()"),
                "{cs:?} sidecar wrapper must not pass .call = match.call():\n{out}"
            );
        }
    }

    /// Every sidecar R wrapper (getter and setter, all class systems) must
    /// carry the tagged-condition guard so Rust panics / conversion failures
    /// transported by `with_r_unwind_protect(…, None)` are re-raised as
    /// structured R conditions instead of leaking the tagged SEXP to the user.
    #[test]
    fn sidecar_accessors_reraise_tagged_conditions() {
        let getter_c = "C__mx_rdata_get_T_f";
        let setter_c = "C__mx_rdata_set_T_f";

        for cs in ALL_CLASS_SYSTEMS {
            let out = generate_r_wrapper_for_slot(cs, "T", "f", getter_c, setter_c);
            assert_eq!(
                out.matches(".miniextendr_raise_condition(.val, sys.call())")
                    .count(),
                2,
                "{cs:?} getter+setter should each carry the condition guard:\n{out}"
            );
        }
    }

    /// The R6 active-binding and S7 property integration code also call the
    /// sidecar C entrypoints directly; they need the same guard (with
    /// `sys.call()` fallback attribution) and must not pass `.call =`.
    #[test]
    fn sidecar_class_integration_reraises_tagged_conditions() {
        let slot = super::SidecarSlot {
            name: syn::Ident::new("f", proc_macro2::Span::call_site()),
            ty: syn::parse_quote!(i32),
            index: 0,
            is_public: true,
            kind: super::SlotKind::ScalarInt,
            prop_doc: None,
        };
        let slots = [&slot];

        for cs in [ClassSystem::R6, ClassSystem::S7] {
            let out = generate_class_integration_r_code(cs, "T", &slots);
            assert_eq!(
                out.matches(".miniextendr_raise_condition(.val, sys.call())")
                    .count(),
                2,
                "{cs:?} integration getter+setter should each carry the condition guard:\n{out}"
            );
            assert!(
                !out.contains(".call = match.call()"),
                "{cs:?} integration code must not pass .call = match.call():\n{out}"
            );
        }
    }
}
