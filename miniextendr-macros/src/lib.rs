//! # miniextendr-macros - Procedural macros for Rust <-> R interop
//!
//! This crate provides the procedural macros that power miniextendr's code
//! generation. Most users should depend on `miniextendr-api` and use its
//! re-exports, but this crate can be used directly when you only need macros.
//!
//! Primary macros and derives:
//! - `#[miniextendr]` on functions, impl blocks, trait defs, and trait impls.
//! - `#[r_ffi_checked]` for main-thread routing of C-ABI wrappers.
//! - Derives: `ExternalPtr`, `RNativeType`, ALTREP derives, `RFactor`.
//! - Helpers: `typed_list` for typed list builders.
//!
//! R wrapper generation is driven by Rust doc comments (roxygen tags are
//! extracted). During package build, the wrapper-gen pass loads the installed
//! shared object into R and calls `miniextendr_write_wrappers`, which walks the
//! linkme `#[distributed_slice]` tables and writes `R/<pkg>-wrappers.R`.
//!
//! ## Quick start
//!
//! ```ignore
//! use miniextendr_api::miniextendr;
//!
//! #[miniextendr]
//! fn add(a: i32, b: i32) -> i32 {
//!     a + b
//! }
//! ```
//!
//! ## Macro expansion pipeline
//!
//! ### Overview
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │                         #[miniextendr] on fn                             │
//! │                                                                          │
//! │  1. Parse: syn::ItemFn → MiniextendrFunctionParsed                       │
//! │  2. Analyze return type (Result<T>, Option<T>, raw SEXP, etc.)           │
//! │  3. Generate:                                                            │
//! │     ├── C wrapper: extern "C-unwind" fn C_<crate>_<name>(...) → SEXP     │
//! │     ├── R wrapper: const R_WRAPPER_<NAME>: &str = "..."                  │
//! │     └── Registration: const call_method_def_<name>: R_CallMethodDef      │
//! │  4. Original function preserved (with added attributes)                  │
//! └──────────────────────────────────────────────────────────────────────────┘
//!
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │                    #[miniextendr(env|r6|s3|s4|s7)] on impl               │
//! │                                                                          │
//! │  1. Parse: syn::ItemImpl → extract methods                               │
//! │  2. For each method:                                                     │
//! │     ├── Generate C wrapper (handles self parameter)                      │
//! │     ├── Generate R method wrapper string                                 │
//! │     └── Generate registration entry                                      │
//! │  3. Generate class definition code per class system:                     │
//! │     ├── env: new.env() + method assignment                               │
//! │     ├── r6: R6Class() definition                                         │
//! │     ├── s3: S3 generics + methods                                        │
//! │     ├── s4: setClass() + setMethod()                                     │
//! │     └── s7: new_class() definition                                       │
//! │  4. Emit const with combined R code                                      │
//! └──────────────────────────────────────────────────────────────────────────┘
//!
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │                         #[miniextendr] on trait                          │
//! │                                                                          │
//! │  1. Parse: syn::ItemTrait → extract method signatures                    │
//! │  2. Generate:                                                            │
//! │     ├── Trait tag constant: const TAG_<TRAIT>: mx_tag = ...              │
//! │     ├── Vtable struct: struct __vtable_<Trait> { ... }                   │
//! │     └── CCalls table: static MX_CCALL_<TRAIT>: [...] = ...               │
//! │  3. Original trait preserved                                             │
//! └──────────────────────────────────────────────────────────────────────────┘
//!
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │                    #[miniextendr] impl Trait for Type                    │
//! │                                                                          │
//! │  1. Parse: syn::ItemImpl (trait impl)                                    │
//! │  2. Generate:                                                            │
//! │     ├── Vtable instance: static __VTABLE_<CRATE>_<TRAIT>_FOR_<TYPE>     │
//! │     ├── Wrapper struct: struct __MxWrapper<Type> { erased, data }        │
//! │     ├── Query function: fn __mx_query_<type>(tag) → vtable ptr           │
//! │     └── Base vtable: static __MX_BASE_VTABLE_<TYPE>: ...                 │
//! │  3. Original impl preserved                                              │
//! └──────────────────────────────────────────────────────────────────────────┘
//!
//! ```
//!
//! ### Key Modules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | `miniextendr_fn` | Function parsing and attribute handling |
//! | `c_wrapper_builder` | C wrapper generation (`extern "C-unwind"`) |
//! | `r_wrapper_builder` | R wrapper code generation |
//! | `rust_conversion_builder` | Rust→SEXP return value conversion |
//! | `miniextendr_impl` | `impl Type` block processing |
//! | `r_class_formatter` | Class system code generation (env/r6/s3/s4/s7) |
//! | `miniextendr_trait` | Trait ABI metadata generation |
//! | `miniextendr_impl_trait` | `impl Trait for Type` vtable generation |
//! | `altrep` / `altrep_derive` | ALTREP struct derivation |
//! | `externalptr_derive` | `#[derive(ExternalPtr)]` |
//! | `roxygen` | Roxygen doc comment handling |
//!
//! ### Generated Symbol Naming
//!
//! Every `#[no_mangle]` symbol is prefixed with the consuming crate's name
//! (from `CARGO_CRATE_NAME`) so two packages loaded into one webR session
//! can't collide on a C symbol (#1273; helpers in `naming.rs`). For a
//! function `my_func` in a crate `mypkg`:
//! - C wrapper: `C_mypkg_my_func`
//! - R wrapper const: `R_WRAPPER_MY_FUNC`
//! - Registration: `call_method_def_my_func`
//!
//! For a type `MyType` with trait `Counter` in a crate `mypkg`:
//! - Vtable: `__VTABLE_MYPKG_COUNTER_FOR_MYTYPE`
//! - Wrapper: `__MxWrapperMyType`
//! - Query: `__mx_query_mytype`
//!
//! ## Return Type Handling
//!
//! The `return_type_analysis` module determines how to convert Rust returns to SEXP:
//!
//! | Rust Type | Strategy | R Result |
//! |-----------|----------|----------|
//! | `T: IntoR` | `.into_sexp()` | Converted value |
//! | `Result<T, E>` | Unwrap or R error | Value or error |
//! | `Option<T>` | `Some` → value, `None` → `NULL` | Value or NULL |
//! | `SEXP` | Pass through | Raw SEXP |
//! | `()` | Invisible NULL | `invisible(NULL)` |
//!
//! Use `#[miniextendr(unwrap_in_r)]` to return `Result<T, E>` to R without unwrapping.
//!
//! ## Thread Strategy
//!
//! By default, `#[miniextendr]` functions run on R's main thread. Opt into
//! worker-thread execution with `#[miniextendr(worker)]` (requires the
//! `worker-thread` feature on `miniextendr-api`). A worker opt-in is ignored
//! when the signature requires main-thread execution (returns/takes `SEXP`,
//! uses variadic dots, or sets `check_interrupt`).
//!
//! **Note**: `ExternalPtr<T>` is `Send` — it can be returned from worker
//! thread functions. All R API operations on `ExternalPtr` are serialized
//! through `with_r_thread`.
//!
//! ## Class Systems
//!
//! The `r_class_formatter` module generates R code for different class systems:
//!
//! | System | Generated R Code | Self Parameter |
//! |--------|------------------|----------------|
//! | `env` | `new.env()` with methods | `self` environment |
//! | `r6` | `R6Class()` | `self` environment |
//! | `s3` | `structure()` + generics | First argument |
//! | `s4` | `setClass()` + `setMethod()` | First argument |
//! | `s7` | `new_class()` | `self` property |

// miniextendr-macros procedural macros

mod altrep;
mod c_wrapper_builder;
mod list_macro;
mod match_arg_keys;
mod miniextendr_fn;
mod type_inspect;
mod typed_dataframe;
mod typed_list;
mod util;
use crate::miniextendr_fn::{MiniextendrFnAttrs, MiniextendrFunctionParsed};
mod miniextendr_impl;
mod r_wrapper_builder;
/// Builder utilities for formatting R wrapper arguments and calls.
pub(crate) use r_wrapper_builder::RArgumentBuilder;
mod rust_conversion_builder;
/// Helper for generating Rust→R conversion code for return values.
pub(crate) use rust_conversion_builder::RustConversionBuilder;
mod method_return_builder;
/// Helpers for shaping method return handling (R vs Rust wrapper code).
pub(crate) use method_return_builder::{MethodReturnBuilder, ReturnStrategy};
mod altrep_derive;
mod dataframe_derive;
mod lifecycle;
mod list_derive;
mod r_class_formatter;
mod r_preconditions;
mod return_type_analysis;
mod roxygen;

// Trait ABI support modules
mod externalptr_derive;
mod miniextendr_impl_trait;
mod miniextendr_trait;
mod typed_external_macro;

// Factor support
mod factor_derive;
mod match_arg_derive;
mod newtype_derive;

// Struct/enum dispatch for #[miniextendr] on structs and enums
mod struct_enum_dispatch;

// r! proc-macro implementation
mod r_macro;

// vctrs support
#[cfg(feature = "vctrs")]
mod vctrs_derive;
mod vctrs_generics;

mod naming;
pub(crate) use naming::r_wrapper_const_ident_for;

// Feature default mutual exclusivity guards
#[cfg(all(feature = "r6-default", feature = "s7-default"))]
compile_error!(
    "features \"r6-default\" and \"s7-default\" are mutually exclusive — \
     enable exactly one, or omit both to fall back to the unspecified default"
);
// Note: default-main-thread was removed — main thread is now the hardcoded default.
// worker-default still opts into worker thread execution.

pub(crate) use type_inspect::{
    SeveralOkContainer, classify_several_ok_container, first_type_argument,
    is_main_thread_bound_input, is_main_thread_bound_return, is_sexp_type, second_type_argument,
};
pub(crate) use util::{extract_cfg_attrs, r_wrapper_raw_literal, source_location_doc};

/// Validate the signature of an `extern "C-unwind"` fn exported via `#[miniextendr]`.
///
/// R's `.Call` interface passes all arguments as `SEXP` and expects a `SEXP`
/// return value. For `extern "C-unwind"` functions the user writes the C symbol
/// directly, so the signature must satisfy those invariants statically —
/// otherwise the generated registration produces UB at runtime.
///
/// Called before any codegen so we fail fast on an invalid extern signature
/// rather than emitting a wrapper that would only matter after the error.
fn validate_extern_signature(
    abi: &syn::Abi,
    attrs: &[syn::Attribute],
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    output: &syn::ReturnType,
) -> syn::Result<()> {
    use syn::spanned::Spanned;

    // Reject `self` receivers up front — they are never valid for `.Call`
    // exports, and a missing-return-type error would only hide the real
    // problem (users write `fn foo(self)` to ask "can I export a method?").
    for input in inputs.iter() {
        if let syn::FnArg::Receiver(recv) = input {
            return Err(syn::Error::new_spanned(
                recv,
                "self parameter not allowed in standalone functions; \
                 use #[miniextendr(env|r6|s3|s4|s7)] on impl blocks instead",
            ));
        }
    }

    // Require one of #[no_mangle] / #[unsafe(no_mangle)] / #[export_name].
    let has_no_mangle = attrs.iter().any(|attr| {
        attr.path().is_ident("no_mangle")
            || attr
                .parse_nested_meta(|meta| {
                    if meta.path.is_ident("no_mangle") {
                        Err(meta.error("found #[no_mangle]"))
                    } else {
                        Ok(())
                    }
                })
                .is_err()
    });
    let has_export_name = attrs.iter().any(|attr| attr.path().is_ident("export_name"));
    if !has_no_mangle && !has_export_name {
        return Err(syn::Error::new(
            attrs
                .first()
                .map(|attr| attr.span())
                .unwrap_or_else(|| abi.span()),
            "extern \"C-unwind\" functions need a visible C symbol for R's .Call interface. \
             Add one of:\n  \
             - `#[unsafe(no_mangle)]` (Rust 2024 edition)\n  \
             - `#[no_mangle]` (Rust 2021 edition)\n  \
             - `#[export_name = \"my_symbol\"]` (custom symbol name)",
        ));
    }

    // Return type must be SEXP.
    match output {
        non_return_type @ syn::ReturnType::Default => {
            return Err(syn::Error::new(
                non_return_type.span(),
                "extern \"C-unwind\" functions used with #[miniextendr] must return SEXP. \
                 Add `-> miniextendr_api::SEXP` as the return type. \
                 If you want automatic type conversion, remove `extern \"C-unwind\"` and let \
                 the macro generate the C wrapper.",
            ));
        }
        syn::ReturnType::Type(_rarrow, output_type) => match output_type.as_ref() {
            syn::Type::Path(type_path) => {
                if let Some(path_to_sexp) = type_path.path.segments.last().map(|x| &x.ident)
                    && path_to_sexp != "SEXP"
                {
                    return Err(syn::Error::new(
                        path_to_sexp.span(),
                        format!(
                            "extern \"C-unwind\" functions must return SEXP, found `{path_to_sexp}`. \
                             R's .Call interface expects SEXP return values. \
                             Change the return type to `miniextendr_api::SEXP`, or remove \
                             `extern \"C-unwind\"` to let the macro handle type conversion.",
                        ),
                    ));
                }
            }
            _ => {
                return Err(syn::Error::new(
                    output_type.span(),
                    "extern \"C-unwind\" functions must return SEXP. \
                     R's .Call interface expects SEXP return values. \
                     Change the return type to `miniextendr_api::SEXP`, or remove \
                     `extern \"C-unwind\"` to let the macro handle type conversion.",
                ));
            }
        },
    }

    // Every input must be SEXP. Reject variadics and receivers.
    for input in inputs.iter() {
        match input {
            syn::FnArg::Receiver(recv) => {
                return Err(syn::Error::new_spanned(
                    recv,
                    "extern \"C-unwind\" functions cannot have a `self` parameter. \
                     R's .Call interface only accepts SEXP arguments. \
                     Use `#[miniextendr(env|r6|s3|s4|s7)]` on an impl block for methods.",
                ));
            }
            syn::FnArg::Typed(pat_type) => {
                if let syn::Pat::Rest(_) = pat_type.pat.as_ref() {
                    return Err(syn::Error::new_spanned(
                        pat_type,
                        "extern functions cannot use variadic (...) - .Call passes fixed arguments",
                    ));
                }
                let is_sexp = match pat_type.ty.as_ref() {
                    syn::Type::Path(type_path) => type_path
                        .path
                        .segments
                        .last()
                        .is_some_and(|seg| seg.ident == "SEXP"),
                    _ => false,
                };
                if !is_sexp {
                    let is_dots_type = if let syn::Type::Reference(type_ref) = pat_type.ty.as_ref()
                    {
                        if let syn::Type::Path(inner) = type_ref.elem.as_ref() {
                            inner
                                .path
                                .segments
                                .last()
                                .is_some_and(|seg| seg.ident == "Dots")
                        } else {
                            false
                        }
                    } else if let syn::Type::Path(type_path) = pat_type.ty.as_ref() {
                        type_path
                            .path
                            .segments
                            .last()
                            .is_some_and(|seg| seg.ident == "Dots")
                    } else {
                        false
                    };
                    let msg = if is_dots_type {
                        "extern functions cannot use Dots; use `...` syntax in non-extern #[miniextendr] functions instead"
                    } else {
                        "extern function parameters must be SEXP - .Call passes all arguments as SEXP"
                    };
                    return Err(syn::Error::new_spanned(&pat_type.ty, msg));
                }
            }
        }
    }

    Ok(())
}

/// Builds the `let dots_typed = <dots>.typed(<spec>)...` statement injected
/// at the start of a function body when `#[miniextendr(dots =
/// typed_list!(...))]` is used.
///
/// Uses `unwrap_or_else(|e| panic!("...: {e}"))` rather than
/// `Result::expect`, which formats the error with `Debug` instead of
/// `Display` — for `TypedListError` that leaks PascalCase enum-variant names
/// (`Missing { name: "x" }`) into the R-visible message instead of the
/// human-phrased text (`missing required field: "x"`) that the direct
/// `typed_list!` path already produces via `Display`. See audit A8.
pub(crate) fn build_dots_validation_stmt(
    dots_param: &syn::Ident,
    spec_tokens: &proc_macro2::TokenStream,
) -> syn::Stmt {
    syn::parse_quote! {
        let dots_typed = #dots_param.typed(#spec_tokens)
            .unwrap_or_else(|e| panic!("dots validation failed: {e}"));
    }
}

/// Emit the `extern "C-unwind"` helper + `R_CallMethodDef` registration for
/// each standalone-fn `match_arg` param.
///
/// Each helper returns the enum's `CHOICES` wrapped in a STRSXP so the R
/// wrapper's prelude can call `match.arg(x, .Call(helper, ...))`. Factored
/// out of the `miniextendr` fn body so the entry point doesn't own the
/// `quote!` scaffolding.
fn build_match_arg_helpers(
    match_arg_param_info: &[(String, String, &syn::Type)],
    parsed: &miniextendr_fn::MiniextendrFunctionParsed,
    c_ident_str: &str,
    cfg_attrs: &[syn::Attribute],
) -> Vec<proc_macro2::TokenStream> {
    match_arg_param_info
        .iter()
        .map(|(r_param, rust_name, param_ty)| {
            // For several_ok, the param type is e.g. Vec<Mode>/Box<[Mode]>/[Mode; N]/&[Mode]
            // — extract inner Mode for choices_sexp.
            let choices_ty: &syn::Type = if parsed.has_several_ok(rust_name) {
                classify_several_ok_container(param_ty)
                    .map(|(_, t)| t)
                    .unwrap_or(param_ty)
            } else {
                param_ty
            };
            let helper_fn_name = crate::match_arg_keys::choices_helper_c_name(c_ident_str, r_param);
            let helper_fn_ident = syn::Ident::new(&helper_fn_name, proc_macro2::Span::call_site());
            let helper_def_ident =
                crate::match_arg_keys::choices_helper_def_ident(c_ident_str, r_param);
            let helper_c_name = syn::LitCStr::new(
                std::ffi::CString::new(helper_fn_name.clone())
                    .expect("valid C string")
                    .as_c_str(),
                proc_macro2::Span::call_site(),
            );
            quote::quote! {
                #(#cfg_attrs)*
                #[allow(non_snake_case)]
                #[unsafe(no_mangle)]
                pub extern "C-unwind" fn #helper_fn_ident(
                    __miniextendr_call: ::miniextendr_api::SEXP,
                ) -> ::miniextendr_api::SEXP {
                    ::miniextendr_api::choices_sexp::<#choices_ty>()
                }

                #(#cfg_attrs)*
                #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS), linkme(crate = ::miniextendr_api::linkme))]
                #[allow(non_upper_case_globals)]
                #[allow(non_snake_case)]
                static #helper_def_ident: ::miniextendr_api::sys::R_CallMethodDef = unsafe {
                    ::miniextendr_api::sys::R_CallMethodDef {
                        name: #helper_c_name.as_ptr(),
                        fun: Some(std::mem::transmute::<
                            unsafe extern "C-unwind" fn(
                                ::miniextendr_api::SEXP,
                            ) -> ::miniextendr_api::SEXP,
                            unsafe extern "C-unwind" fn() -> *mut ::std::os::raw::c_void,
                        >(#helper_fn_ident)),
                        numArgs: 1i32,
                    }
                };
            }
        })
        .collect()
}

/// Export Rust items to R.
///
/// `#[miniextendr]` can be applied to:
/// - `fn` items (generate C + R wrappers)
/// - `impl` blocks (generate R class methods)
/// - `trait` items (generate trait ABI metadata)
/// - ALTREP wrapper structs (generate `RegisterAltrep` impls)
///
/// # Functions
///
/// ```ignore
/// use miniextendr_api::miniextendr;
///
/// #[miniextendr]
/// fn add(a: i32, b: i32) -> i32 { a + b }
/// ```
///
/// This produces a C wrapper `C_<crate>_add` and an R wrapper `add()`.
/// Registration is automatic via linkme distributed slices.
///
/// ## `extern "C-unwind"`
///
/// If the function is declared `extern "C-unwind"` and exported with
/// `#[no_mangle]` (2021), `#[unsafe(no_mangle)]` (2024), or `#[export_name = "..."]`,
/// the function itself is the C symbol and the R wrapper is prefixed with
/// `unsafe_` to signal bypassed safety (no worker isolation or conversion).
///
/// ## Variadics (`...`)
///
/// Use `...` as the last argument. The Rust parameter becomes `_dots: &Dots`.
/// Use `name: ...` to give it a custom name (e.g., `args: ...` → `args: &Dots`).
///
/// ### Typed Dots Validation
///
/// Use `#[miniextendr(dots = typed_list!(...))]` to automatically validate dots
/// and create a `dots_typed` variable with typed accessors:
///
/// ```ignore
/// #[miniextendr(dots = typed_list!(x => numeric(), y => integer(), z? => character()))]
/// pub fn my_func(...) -> String {
///     let x: f64 = dots_typed.get("x").expect("x");
///     let y: i32 = dots_typed.get("y").expect("y");
///     let z: Option<String> = dots_typed.get_opt("z").expect("z");
///     format!("x={}, y={}", x, y)
/// }
/// ```
///
/// Type specs: `numeric()`, `integer()`, `logical()`, `character()`, `list()`,
/// `raw()`, `complex()`, or `"class_name"` for class inheritance checks.
/// Add `(n)` for exact length: `numeric(4)`. Use `?` suffix for optional fields.
/// Use `@exact;` prefix for strict mode (reject extra fields).
///
/// ## Attributes
///
/// - `#[miniextendr(worker)]` — opt into worker-thread execution
/// - `#[miniextendr(invisible)]` / `#[miniextendr(visible)]` — control return visibility
/// - `#[miniextendr(check_interrupt)]` — check for user interrupt after call
/// - `#[miniextendr(coerce)]` — coerce R type before conversion (also usable per-parameter)
/// - `#[miniextendr(strict)]` — reject lossy conversions for i64/u64/isize/usize
/// - `#[miniextendr(unwrap_in_r)]` — return `Result<T, E>` to R without unwrapping
/// - `#[miniextendr(dots = typed_list!(...))]` — validate dots, create `dots_typed`
/// - `#[miniextendr(internal)]` — adds `@keywords internal` to R wrapper
/// - `#[miniextendr(noexport)]` — suppresses `@export` from R wrapper
///
/// # Impl blocks (class systems)
///
/// Apply `#[miniextendr(env|r6|s7|s3|s4)]` to an `impl Type` block.
/// Use `#[miniextendr(label = "...")]` to disambiguate multiple impl blocks
/// on the same type.
/// Registration is automatic.
///
/// ## R6 Active Bindings
///
/// For R6 classes, use `#[miniextendr(r6(active))]` on methods to create
/// active bindings (computed properties accessed without parentheses):
///
/// ```ignore
/// use miniextendr_api::miniextendr;
///
/// pub struct Rectangle {
///     width: f64,
///     height: f64,
/// }
///
/// #[miniextendr(r6)]
/// impl Rectangle {
///     pub fn new(width: f64, height: f64) -> Self {
///         Self { width, height }
///     }
///
///     /// Returns the area (width * height).
///     #[miniextendr(r6(active))]
///     pub fn area(&self) -> f64 {
///         self.width * self.height
///     }
///
///     /// Regular method (requires parentheses).
///     pub fn scale(&mut self, factor: f64) {
///         self.width *= factor;
///         self.height *= factor;
///     }
/// }
/// ```
///
/// In R:
/// ```r
/// r <- Rectangle$new(3, 4)
/// r$area        # 12 (active binding - no parentheses!)
/// r$scale(2)    # Regular method call
/// r$area        # 24
/// ```
///
/// Active bindings must be getter-only methods taking only `&self`.
///
/// ## S7 Properties
///
/// For S7 classes, use `#[miniextendr(s7(getter))]` and `#[miniextendr(s7(setter))]`
/// to create computed properties accessed via `@`:
///
/// ```ignore
/// use miniextendr_api::{miniextendr, ExternalPtr};
///
/// #[derive(ExternalPtr)]
/// pub struct Range {
///     start: f64,
///     end: f64,
/// }
///
/// #[miniextendr(s7)]
/// impl Range {
///     pub fn new(start: f64, end: f64) -> Self {
///         Self { start, end }
///     }
///
///     /// Computed property (read-only): length of the range.
///     #[miniextendr(s7(getter))]
///     pub fn length(&self) -> f64 {
///         self.end - self.start
///     }
///
///     /// Dynamic property getter.
///     #[miniextendr(s7(getter, prop = "midpoint"))]
///     pub fn get_midpoint(&self) -> f64 {
///         (self.start + self.end) / 2.0
///     }
///
///     /// Dynamic property setter.
///     #[miniextendr(s7(setter, prop = "midpoint"))]
///     pub fn set_midpoint(&mut self, value: f64) {
///         let half = self.length() / 2.0;
///         self.start = value - half;
///         self.end = value + half;
///     }
/// }
/// ```
///
/// In R:
/// ```r
/// r <- Range(0, 10)
/// r@length     # 10 (computed, read-only)
/// r@midpoint   # 5 (dynamic property)
/// r@midpoint <- 20  # Adjusts start/end to center at 20
/// ```
///
/// ### Property Attributes
///
/// - `#[miniextendr(s7(getter))]` - Read-only computed property
/// - `#[miniextendr(s7(getter, prop = "name"))]` - Named property getter
/// - `#[miniextendr(s7(setter, prop = "name"))]` - Named property setter
/// - `#[miniextendr(s7(getter, default = "0.0"))]` - Property with default value
/// - `#[miniextendr(s7(getter, required))]` - Required property (error if not provided)
/// - `#[miniextendr(s7(getter, frozen))]` - Property that can only be set once
/// - `#[miniextendr(s7(getter, deprecated = "Use X instead"))]` - Deprecated property
/// - `#[miniextendr(s7(validate))]` - Validator function for property
///
/// ## S7 Generic Dispatch Control
///
/// Control how S7 generics are created:
///
/// - `#[miniextendr(s7(no_dots))]` - Create strict generic without `...`
/// - `#[miniextendr(s7(dispatch = "x,y"))]` - Multi-dispatch on multiple arguments
/// - `#[miniextendr(s7(fallback))]` - Register method for `class_any` (catch-all).
///   The generated R wrapper uses `tryCatch(x@.ptr, error = function(e) x)` to
///   safely extract the self argument, so non-miniextendr objects won't crash with
///   a slot-access error. Instead, incompatible objects produce a Rust type-conversion
///   error when the method tries to interpret the argument as `&Self`.
///
/// ```ignore
/// #[miniextendr(s7)]
/// impl MyClass {
///     /// Strict generic: function(x) instead of function(x, ...)
///     #[miniextendr(s7(no_dots))]
///     pub fn strict_method(&self) -> i32 { 42 }
///
///     /// Fallback method dispatched on class_any.
///     /// Calling this on a non-MyClass object produces a type-conversion error,
///     /// not a slot-access crash.
///     #[miniextendr(s7(fallback))]
///     pub fn describe(&self) -> String { "generic description".into() }
/// }
/// ```
///
/// ## S7 Type Conversion (`convert`)
///
/// Use `convert_from` and `convert_to` to enable S7's `convert()` for type coercion:
///
/// ```ignore
/// use miniextendr_api::{miniextendr, ExternalPtr};
///
/// #[derive(ExternalPtr)]
/// pub struct Celsius { value: f64 }
///
/// #[derive(ExternalPtr)]
/// pub struct Fahrenheit { value: f64 }
///
/// #[miniextendr(s7)]
/// impl Fahrenheit {
///     pub fn new(value: f64) -> Self { Self { value } }
///
///     /// Convert FROM Celsius TO Fahrenheit.
///     /// Usage: S7::convert(celsius_obj, Fahrenheit)
///     #[miniextendr(s7(convert_from = "Celsius"))]
///     pub fn from_celsius(c: ExternalPtr<Celsius>) -> Self {
///         Fahrenheit { value: c.value * 9.0 / 5.0 + 32.0 }
///     }
///
///     /// Convert FROM Fahrenheit TO Celsius.
///     /// Usage: S7::convert(fahrenheit_obj, Celsius)
///     #[miniextendr(s7(convert_to = "Celsius"))]
///     pub fn to_celsius(&self) -> Celsius {
///         Celsius { value: (self.value - 32.0) * 5.0 / 9.0 }
///     }
/// }
/// ```
///
/// In R:
/// ```r
/// c <- Celsius(100)
/// f <- S7::convert(c, Fahrenheit)  # Uses convert_from
/// c2 <- S7::convert(f, Celsius)    # Uses convert_to
/// ```
///
/// **Note:** Classes must be defined before they can be referenced in convert methods.
/// Define the "from" class before the "to" class to avoid forward reference issues.
///
/// # Traits (ABI)
///
/// Apply `#[miniextendr]` to a trait to generate ABI metadata, then use
/// `#[miniextendr] impl Trait for Type`. Registration is automatic.
///
/// # ALTREP
///
/// `#[miniextendr]` no longer generates ALTREP classes — `class`/`base`
/// attributes on a one-field struct are a compile error. Use the per-family
/// derives (`#[derive(AltrepInteger)]`, …) with `#[altrep(class = "...")]`
/// instead; registration is automatic there.
#[proc_macro_attribute]
pub fn miniextendr(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Try to parse as function first
    if syn::parse::<syn::ItemFn>(item.clone()).is_ok() {
        // Continue with function handling below
    } else if syn::parse::<syn::ItemImpl>(item.clone()).is_ok() {
        // Delegate to impl block parser
        return miniextendr_impl::expand_impl(attr, item);
    } else if syn::parse::<syn::ItemTrait>(item.clone()).is_ok() {
        // Delegate to trait ABI generator
        return miniextendr_trait::expand_trait(attr, item);
    } else {
        // Delegate to struct/enum dispatch (handles ALTREP, ExternalPtr, list, dataframe, factor, match_arg)
        return struct_enum_dispatch::expand_struct_or_enum(attr, item);
    }

    let MiniextendrFnAttrs {
        force_worker,
        force_invisible,
        check_interrupt,
        coerce_all,
        rng,
        unwrap_in_r,
        no_preconditions,
        no_call_attribution,
        call_caller,
        return_pref,
        return_pref_span,
        s3_generic,
        s3_class,
        dots_spec,
        dots_span,
        lifecycle,
        strict,
        internal,
        noexport,
        export,
        doc,
        c_symbol,
        r_name: fn_r_name,
        postfix: fn_postfix,
        r_entry,
        r_post_checks,
        r_on_exit,
    } = syn::parse_macro_input!(attr as MiniextendrFnAttrs);

    let mut parsed = syn::parse_macro_input!(item as MiniextendrFunctionParsed);

    // Reject async functions
    if let Some(asyncness) = &parsed.item().sig.asyncness {
        return syn::Error::new_spanned(
            asyncness,
            "async functions are not supported by #[miniextendr]; \
             R's C API is synchronous and incompatible with async executors",
        )
        .into_compile_error()
        .into();
    }

    // Validate: reject type/const generic functions.
    // Lifetime params ARE allowed — they are erased at codegen and produce a single monomorphic
    // symbol, so `#[no_mangle] extern "C-unwind" fn f<'a>(...)` is valid.
    // Type and const params require monomorphization → multiple symbols → cannot be #[no_mangle].
    {
        let params = &parsed.item().sig.generics.params;
        let has_type_or_const = params
            .iter()
            .any(|p| matches!(p, syn::GenericParam::Type(_) | syn::GenericParam::Const(_)));

        if has_type_or_const {
            let err = syn::Error::new_spanned(
                &parsed.item().sig.generics,
                "#[miniextendr] functions cannot have generic type or const parameters. \
                 Generic functions are incompatible with `extern \"C-unwind\"` and `#[no_mangle]` \
                 required for R FFI. Consider using trait objects or monomorphization instead. \
                 Explicit lifetime parameters are allowed (lifetimes are erased at codegen).",
            );
            return err.into_compile_error().into();
        }
    }

    // NB: no automatic `#[track_caller]` (dropped in #1121). It made a *direct*
    // panic's hook-location resolve to the generated wrapper glue instead of the
    // user's `panic!` line, which defeats the `(at file:line)` suffix now folded
    // into the R error message from the panic hook. See docs/TRACK_CALLER.md.
    parsed.add_inline_never_if_needed();

    // Extract commonly used values
    let uses_internal_c_wrapper = parsed.uses_internal_c_wrapper();
    let c_ident = if let Some(ref sym) = c_symbol {
        syn::Ident::new(sym, parsed.c_wrapper_ident().span())
    } else {
        parsed.c_wrapper_ident()
    };
    let r_wrapper_generator = parsed.r_wrapper_const_ident();

    // Extract references to parsed components
    let rust_ident = parsed.ident();
    let inputs = parsed.inputs();
    let output = parsed.output();
    let abi = parsed.abi();
    let attrs = parsed.attrs();
    let vis = parsed.vis();
    let generics = parsed.generics();
    let has_dots = parsed.has_dots();
    let named_dots = parsed.named_dots().cloned();

    // Fail fast on invalid extern "C-unwind" signatures *before* any codegen,
    // so we never emit a wrapper that would be discarded by the surfaced error.
    if let Some(user_abi) = abi
        && let Err(e) = validate_extern_signature(user_abi, attrs, inputs, output)
    {
        return e.into_compile_error().into();
    }

    // Check for @title/@description conflicts with implicit values (doc-lint feature)
    // Skip when `doc` attribute overrides the roxygen — implicit docs are irrelevant then.
    let doc_lint_warnings = if doc.is_some() {
        proc_macro2::TokenStream::new()
    } else {
        crate::roxygen::doc_conflict_warnings(attrs, rust_ident.span())
    };

    // calling the rust function with
    let rust_inputs: Vec<syn::Ident> = inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(p) = pt.pat.as_ref()
            {
                return Some(p.ident.clone());
            }
            None
        })
        .collect();
    // dbg!(&rust_inputs);

    // Validate dots_spec usage (actual injection happens later in the function body)
    if dots_spec.is_some() && !has_dots {
        let err = syn::Error::new(
            dots_span.unwrap_or_else(proc_macro2::Span::call_site),
            "#[miniextendr(dots = typed_list!(...))] requires a `...` parameter in the function signature",
        );
        return err.into_compile_error().into();
    }

    // Analyze return type to determine:
    // - Whether it returns SEXP (affects thread strategy)
    // - Whether result should be invisible
    let rust_result_ident =
        syn::Ident::new("__miniextendr_rust_result", proc_macro2::Span::mixed_site());
    let return_analysis = return_type_analysis::analyze_return_type(
        output,
        &rust_result_ident,
        rust_ident,
        unwrap_in_r,
        strict,
    );

    let returns_sexp = return_analysis.returns_sexp;
    let is_invisible_return_type = return_analysis.is_invisible;

    // Apply explicit visibility override from #[miniextendr(invisible)] or #[miniextendr(visible)]
    let is_invisible_return_type = force_invisible.unwrap_or(is_invisible_return_type);

    // Check if any input parameter is main-thread-bound (SEXP or a !Send
    // framework wrapper like AltrepSexp — neither can move into the worker
    // closure, so the function must stay on the main thread)
    let has_sexp_inputs = inputs.iter().any(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            is_main_thread_bound_input(pat_type.ty.as_ref())
        } else {
            false
        }
    });

    // Check if the return type is main-thread-bound (BuiltDataFrame /
    // DataFrameShape / an R-backed view — all hold R memory and are !Send, so
    // they cannot cross back from the worker thread; `run_on_worker` requires
    // `T: Send`). Walks nested positions: Result<BuiltDataFrame, E>,
    // Option<DataFrameShape>, Vec<...>, tuples.
    let has_main_thread_bound_return = match output {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => is_main_thread_bound_return(ty),
    };

    // ═══════════════════════════════════════════════════════════════════════════
    // Thread Strategy Selection
    // ═══════════════════════════════════════════════════════════════════════════
    //
    // miniextendr supports two execution strategies:
    //
    // 1. **Main Thread Strategy** (with_r_unwind_protect) — DEFAULT
    //    - All code runs on R's main thread
    //    - Required when SEXP types are involved (not Send)
    //    - Required for R API calls (Rf_*, R_*)
    //    - Panic handling via R_UnwindProtect (Rust destructors run correctly)
    //    - Errors are always returned as tagged SEXP values; the R wrapper
    //      inspects the tag and raises a structured condition (`rust_*` class
    //      layering) past the Rust boundary.
    //    - Simpler execution model, better R integration
    //
    // 2. **Worker Thread Strategy** (run_on_worker + catch_unwind) — OPT-IN
    //    - Argument conversion on main thread (SEXP → Rust types)
    //    - Function execution on dedicated worker thread (clean panic isolation)
    //    - Result conversion on main thread (Rust types → SEXP)
    //    - Panic handling via catch_unwind (prevents unwinding across FFI boundary)
    //    - Opt in with #[miniextendr(worker)]
    //    - ExternalPtr<T> is Send: can be returned from worker thread functions
    //    - R API calls from worker use with_r_thread (serialized to main thread)
    //
    // Default: Main thread (simpler execution model, compatible with the
    // tagged-SEXP error transport)
    // Override: Use worker thread with #[miniextendr(worker)]
    //
    // Thread strategy:
    // - Main thread is always used unless force_worker is set
    // - force_worker cannot override hard requirements for main thread
    // - Hard requirements: returns_sexp, has_sexp_inputs,
    //   has_main_thread_bound_return, has_dots, check_interrupt
    let requires_main_thread = returns_sexp
        || has_sexp_inputs
        || has_main_thread_bound_return
        || has_dots
        || check_interrupt;
    let use_main_thread = !force_worker || requires_main_thread;

    // Extract cfg attributes to apply to generated items (needed by CWrapperContext)
    let cfg_attrs = extract_cfg_attrs(parsed.attrs());

    let source_loc_doc = source_location_doc(rust_ident.span());

    // Build individual per-parameter coerce and match_arg_several_ok lists
    let mut coerce_params_list: Vec<String> = Vec::new();
    let mut match_arg_several_ok_params_list: Vec<String> = Vec::new();
    for input in inputs.iter() {
        if let syn::FnArg::Typed(pt) = input
            && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
        {
            let param_name = pat_ident.ident.to_string();
            if parsed.has_coerce_attr(&param_name) {
                coerce_params_list.push(param_name.clone());
            }
            if parsed.has_match_arg_attr(&param_name) && parsed.has_several_ok(&param_name) {
                match_arg_several_ok_params_list.push(param_name);
            }
        }
    }

    // Build the call expression: rust_ident(rust_input_1, rust_input_2, ...)
    let fn_call_expr = quote::quote! { #rust_ident(#(#rust_inputs),*) };

    // Determine return handling: use standalone-fn semantics (OptionIntoR for Option<T>)
    // and handle unwrap_in_r (Result<T, E> → IntoR to pass result list to R).
    let return_pref_is_set = !matches!(return_pref, crate::miniextendr_fn::ReturnPref::Auto);
    let fn_return_handling = if unwrap_in_r && crate::return_type_analysis::output_is_result(output)
    {
        // In unwrap_in_r mode the IntoR here operates on the whole Result<T,E> (the
        // framework's IntoR for Result encodes it as a tagged list for R to decode), NOT on
        // the inner T. prefer= always targets that inner T, so there is nothing for it to
        // wrap here — reject rather than silently drop it (see apply_return_pref docstring
        // and the BUG4 audit finding).
        if return_pref_is_set {
            let span = return_pref_span.unwrap_or_else(proc_macro2::Span::call_site);
            return syn::Error::new(
                span,
                "`prefer = ...` cannot be combined with `unwrap_in_r` on a function returning \
                 `Result<T, E>`: `unwrap_in_r` converts the whole `Result<T, E>` via a single \
                 `IntoR` impl (a tagged list for R to decode), not the inner `T` alone, so \
                 there is no plain `T` for `prefer=` to wrap. Remove `prefer=`.",
            )
            .into_compile_error()
            .into();
        }
        c_wrapper_builder::ReturnHandling::IntoR
    } else {
        let auto = c_wrapper_builder::detect_return_handling_standalone_fn(output);
        // Apply return_pref override: wraps the result in AsList/AsExternalPtr/AsRNative.
        // Only applies to the plain IntoR variant — Option*/Result*/Unit/RawSexp/ExternalPtr
        // variants have no bare T to wrap and now hard-error instead of silently ignoring
        // prefer= (see apply_return_pref docstring).
        match apply_return_pref(auto, return_pref, return_pref_span) {
            Ok(handling) => handling,
            Err(err) => return err.into_compile_error().into(),
        }
    };

    let thread_strategy = if use_main_thread {
        c_wrapper_builder::ThreadStrategy::MainThread
    } else {
        c_wrapper_builder::ThreadStrategy::WorkerThread
    };

    // Build CWrapperContext for the standalone fn
    let mut c_wrapper_builder =
        c_wrapper_builder::CWrapperContext::builder(rust_ident.clone(), c_ident.clone())
            .r_wrapper_const(r_wrapper_generator.clone())
            .inputs(inputs.iter().cloned().collect())
            .output(output.clone())
            .call_expr(fn_call_expr)
            .thread_strategy(thread_strategy)
            .return_handling(fn_return_handling)
            .cfg_attrs(cfg_attrs.clone())
            .vis(vis.clone())
            .generics(generics.clone())
            .preserve_param_names();

    if uses_internal_c_wrapper {
        // Normal Rust fn: generate full C wrapper
    } else {
        // extern "C-unwind" fn: user wrote the C symbol; only emit R_CallMethodDef
        c_wrapper_builder = c_wrapper_builder.skip_wrapper();
    }

    if coerce_all {
        c_wrapper_builder = c_wrapper_builder.coerce_all();
    }
    for param in &coerce_params_list {
        c_wrapper_builder = c_wrapper_builder.with_coerce_param(param.clone());
    }
    for param in match_arg_several_ok_params_list {
        c_wrapper_builder = c_wrapper_builder.match_arg_several_ok(param);
    }
    if check_interrupt {
        c_wrapper_builder = c_wrapper_builder.check_interrupt();
    }
    if rng {
        c_wrapper_builder = c_wrapper_builder.rng();
    }
    if strict {
        c_wrapper_builder = c_wrapper_builder.strict();
    }

    let c_wrapper = c_wrapper_builder.build().generate();

    // region: R wrappers generation in `fn`
    // Build R formal parameters and call arguments using shared builder
    let mut arg_builder = RArgumentBuilder::new(inputs);
    if has_dots {
        arg_builder = arg_builder.with_dots(named_dots.clone().map(|id| id.to_string()));
    }
    // Add user-specified parameter defaults (Missing<T> defaults handled via body prelude)
    let mut merged_defaults = parsed.param_defaults();
    // For match_arg params, always use the choices placeholder as the formal
    // default — the write-time pass replaces it with `c("a", "b", ...)`.
    // If the user supplied `default = "\"X\""`, capture X as `preferred_default`
    // so the write-time pass rotates X to position 1; the user's literal does
    // NOT become the formal (otherwise R's `match.arg` would only see one
    // choice).
    //
    // Tuple: (placeholder, rust_param, preferred_default_unquoted_or_empty)
    let mut match_arg_placeholders: Vec<(String, String, String)> = Vec::new();
    // (r_name, rust_param) pairs — used later to build @param doc placeholders
    let mut match_arg_r_names: Vec<(String, String)> = Vec::new();
    for match_arg_param in parsed.match_arg_params() {
        let r_name = r_wrapper_builder::normalize_r_arg_string(match_arg_param);
        match_arg_r_names.push((r_name.clone(), match_arg_param.clone()));
        let preferred = match merged_defaults.get(&r_name) {
            Some(raw) => crate::match_arg_keys::extract_match_arg_default(raw),
            None => String::new(),
        };
        let placeholder = crate::match_arg_keys::choices_placeholder(&c_ident.to_string(), &r_name);
        merged_defaults.insert(r_name.clone(), placeholder.clone());
        match_arg_placeholders.push((placeholder, match_arg_param.clone(), preferred));
    }
    // Add c("a", "b", "c") default for choices params (idiomatic R match.arg pattern)
    for (param_name, choices) in parsed.choices_params() {
        let r_name = r_wrapper_builder::normalize_r_arg_string(param_name);
        let quoted: Vec<String> = choices.iter().map(|c| format!("\"{}\"", c)).collect();
        merged_defaults
            .entry(r_name)
            .or_insert_with(|| format!("c({})", quoted.join(", ")));
    }
    arg_builder = arg_builder.with_defaults(merged_defaults);

    let r_formals = arg_builder.build_formals();
    let mut r_call_args_strs = arg_builder.build_call_args_vec();

    // Prepend .call parameter if using internal C wrapper.
    // `#[miniextendr(no_call_attribution)]` / `fast` emits `.call = NULL`
    // instead of `match.call()` — saves ~1200 ns/call. The R-side
    // .miniextendr_raise_condition helper falls back to sys.call() so the
    // error UX is preserved (positional args instead of named).
    let call_attribution = if no_call_attribution {
        r_wrapper_builder::CallAttribution::None
    } else if call_caller {
        r_wrapper_builder::CallAttribution::Caller
    } else {
        r_wrapper_builder::CallAttribution::Wrapper
    };
    if uses_internal_c_wrapper {
        r_call_args_strs.insert(0, call_attribution.dot_call_arg().to_string());
    } else if call_caller {
        // `extern "C-unwind"` fns have no generated call slot to redirect.
        return syn::Error::new_spanned(
            &parsed.item().sig.ident,
            "`call = caller` needs the generated call slot; an `extern \"C-unwind\"` function \
             has no `.call` argument to attribute",
        )
        .into_compile_error()
        .into();
    }

    // Build the R body string consistently
    let c_ident_str = c_ident.to_string();
    let call_args_joined = r_call_args_strs.join(", ");
    let call_expr = if r_call_args_strs.is_empty() {
        format!(".Call({})", c_ident_str)
    } else {
        format!(".Call({}, {})", c_ident_str, call_args_joined)
    };
    let r_wrapper_return_str = {
        // Capture result, check for tagged condition value, raise R condition if present.
        let final_return = if is_invisible_return_type {
            "invisible(.val)"
        } else {
            ".val"
        };
        let body = crate::method_return_builder::standalone_body_with_call_default(
            &call_expr,
            final_return,
            "  ",
            call_attribution.raise_default(),
        );
        // `call = caller` binds `.mx_call` in the wrapper's own frame first.
        format!("{}{body}", call_attribution.prelude("  "))
    };
    // Determine R function name and S3-specific comments
    let is_s3_method = s3_generic.is_some() || s3_class.is_some();
    let r_wrapper_ident_str: String;
    let s3_method_comment: String;

    if is_s3_method {
        // For S3 methods, function name is generic.class
        // generic defaults to Rust function name if not specified
        let generic = s3_generic.clone().unwrap_or_else(|| rust_ident.to_string());
        // s3_class is guaranteed to be Some here because MiniextendrFnAttrs::parse
        // validates that s3(...) always has class specified
        let class = s3_class.as_ref().expect("s3_class validated at parse time");
        r_wrapper_ident_str = format!("{}.{}", generic, class);
        // Add @importFrom for vctrs generics so roxygen registers the dependency
        let import_comment = if crate::vctrs_generics::is_vctrs_generic(&generic) {
            format!("#' @importFrom vctrs {}\n", generic)
        } else {
            String::new()
        };
        s3_method_comment = format!("{}#' @method {} {}\n", import_comment, generic, class);
    } else if let Some(ref custom_name) = fn_r_name {
        r_wrapper_ident_str = custom_name.clone();
        s3_method_comment = String::new();
    } else if let Some(ref postfix) = fn_postfix {
        // `postfix = "_impl"`: the R wrapper is `<rust_name><postfix>`; the C
        // symbol keeps the Rust name.
        r_wrapper_ident_str = format!("{rust_ident}{postfix}");
        s3_method_comment = String::new();
    } else if abi.is_some() {
        r_wrapper_ident_str = format!("unsafe_{}", rust_ident);
        s3_method_comment = String::new();
    } else {
        r_wrapper_ident_str = rust_ident.to_string();
        s3_method_comment = String::new();
    };

    // Stable, consistent R formatting style: brace on same line, body indented, closing brace on its own line
    // r_formals is already a joined string from build_formals()
    let formals_joined = r_formals;
    let mut roxygen_tags = if let Some(ref doc_text) = doc {
        // Custom doc override: each line becomes a separate roxygen tag entry
        doc_text.lines().map(|l| l.to_string()).collect()
    } else {
        crate::roxygen::roxygen_tags_from_attrs(attrs)
    };

    // Determine lifecycle: explicit attr > #[deprecated] extraction
    let lifecycle_spec = lifecycle.or_else(|| {
        attrs
            .iter()
            .find_map(crate::lifecycle::parse_rust_deprecated)
    });

    // Inject lifecycle badge into roxygen tags if present
    if let Some(ref spec) = lifecycle_spec {
        crate::lifecycle::inject_lifecycle_badge(&mut roxygen_tags, spec);
    }

    // Auto-generate @param tags for every non-dots parameter the user didn't
    // already document. Priority, per param:
    //   1. choices(...)      — quoted list, "One of ..." / "One or more of ..."
    //   2. match_arg         — placeholder resolved at write time (#210)
    //   3. everything else   — "(no documentation available)" fallback
    //
    // Collect (doc_placeholder, rust_param) as we go so the write-time resolver
    // registry gets an MX_MATCH_ARG_PARAM_DOCS entry for every match_arg param.
    let mut match_arg_param_doc_placeholders: Vec<(String, String)> = Vec::new();
    for arg in inputs.iter() {
        let syn::FnArg::Typed(pt) = arg else {
            continue;
        };
        let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() else {
            continue;
        };
        if parsed.is_dots_param(&pat_ident.ident) {
            continue;
        }
        let rust_name = pat_ident.ident.to_string();
        let r_name = r_wrapper_builder::normalize_r_arg_ident(&pat_ident.ident).to_string();
        let already_documented = crate::roxygen::param_documented(&roxygen_tags, &r_name);
        if already_documented {
            continue;
        }

        if let Some(choices) = parsed.choices_for_param(&rust_name) {
            let quoted: Vec<String> = choices.iter().map(|c| format!("\"{}\"", c)).collect();
            let prefix = if parsed.has_several_ok(&rust_name) {
                "One or more of"
            } else {
                "One of"
            };
            roxygen_tags.push(format!("@param {r_name} {prefix} {}.", quoted.join(", ")));
        } else if parsed.has_match_arg_attr(&rust_name) {
            let doc_placeholder =
                crate::match_arg_keys::param_doc_placeholder(&c_ident.to_string(), &r_name);
            roxygen_tags.push(format!("@param {r_name} {doc_placeholder}"));
            match_arg_param_doc_placeholders.push((doc_placeholder, rust_name));
        } else {
            roxygen_tags.push(format!("@param {r_name} (no documentation available)"));
        }
    }

    // A standalone function's reference page is titled by its R wrapper name — never
    // the doc-comment prose. rustdoc summaries are markdown (intra-doc links, code
    // spans) that roxygen2 can't resolve as a `\title`; the prose is promoted to
    // `@description` by `roxygen_tags_from_attrs` instead. Without a `@title`, roxygen2
    // skips the `.Rd` entirely (#1054), so inject the wrapper name when none exists.
    if !roxygen_tags.is_empty() && !crate::roxygen::has_roxygen_tag(&roxygen_tags, "title") {
        roxygen_tags.insert(0, format!("@title {}", r_wrapper_ident_str));
    }

    let roxygen_tags_str = crate::roxygen::format_roxygen_tags(&roxygen_tags);
    let has_export_tag = crate::roxygen::has_roxygen_tag(&roxygen_tags, "export");
    let has_no_rd_tag = crate::roxygen::has_roxygen_tag(&roxygen_tags, "noRd");
    let has_internal_tag = crate::roxygen::has_roxygen_tag(&roxygen_tags, "keywords internal");
    // Add roxygen comments: @source for traceability, @export if public
    let source_comment = format!(
        "#' @source Generated by miniextendr from Rust fn `{}`\n",
        rust_ident
    );
    // Inject @keywords internal if #[miniextendr(internal)] and not already present
    let internal_comment = if internal && !has_internal_tag {
        "#' @keywords internal\n"
    } else {
        ""
    };
    // S3 methods need both @method (for registration) AND @export (for NAMESPACE)
    // Don't auto-export functions marked with @noRd, @keywords internal, or attr flags
    // #[miniextendr(export)] forces @export even on non-pub functions
    let export_comment = if (matches!(vis, syn::Visibility::Public(_)) || export)
        && !has_export_tag
        && !has_no_rd_tag
        && !has_internal_tag
        && !internal
        && !noexport
    {
        "#' @export\n".to_string()
    } else {
        String::new()
    };
    // `noexport` means no man page at all (docs/CLASS_SYSTEMS.md export-control
    // table): inject @noRd so roxygen2 skips the Rd and the write-time
    // @rdname-by-file-stem grouping (registry.rs) leaves the fn out of shared
    // pages. Without this, an unexported fn keeps a \usage entry in man/,
    // which R CMD check flags as a code/documentation mismatch.
    let no_rd_comment = if noexport && !has_no_rd_tag {
        "#' @noRd\n"
    } else {
        ""
    };
    // Generate match.arg prelude for parameters with #[miniextendr(match_arg)]
    // Collect (r_param_name, rust_name, rust_type) for each match_arg param
    let match_arg_param_info: Vec<(String, String, &syn::Type)> = inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                let rust_name = pat_ident.ident.to_string();
                if parsed.has_match_arg_attr(&rust_name) {
                    let r_name =
                        r_wrapper_builder::normalize_r_arg_ident(&pat_ident.ident).to_string();
                    return Some((r_name, rust_name, pt.ty.as_ref()));
                }
            }
            None
        })
        .collect();

    let match_arg_prelude = if match_arg_param_info.is_empty() {
        String::new()
    } else {
        let mut lines = Vec::new();
        for (r_param, rust_name, _) in &match_arg_param_info {
            // factor → character normalization
            lines.push(format!(
                "{param} <- if (is.factor({param})) as.character({param}) else {param}",
                param = r_param,
            ));
            // match.arg pulls the choice list off the formal default (which the
            // write-time pass has populated as `c("a", "b", ...)`), so no
            // explicit second arg is needed.
            if parsed.has_several_ok(rust_name) {
                lines.push(format!(
                    "{param} <- base::match.arg({param}, several.ok = TRUE)",
                    param = r_param,
                ));
            } else {
                lines.push(format!(
                    "{param} <- base::match.arg({param})",
                    param = r_param,
                ));
            }
        }
        lines.join("\n  ")
    };

    // Generate idiomatic match.arg prelude for choices params
    // These use the simpler pattern: `param <- match.arg(param)` (no C helper call needed)
    // With `several_ok`, emit `match.arg(param, several.ok = TRUE)` for multi-value selection
    let choices_prelude = {
        let mut lines = Vec::new();
        for arg in inputs.iter() {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                let rust_name = pat_ident.ident.to_string();
                if parsed.choices_for_param(&rust_name).is_some() {
                    let r_name =
                        r_wrapper_builder::normalize_r_arg_ident(&pat_ident.ident).to_string();
                    if parsed.has_several_ok(&rust_name) {
                        lines.push(format!(
                            "{r_name} <- match.arg({r_name}, several.ok = TRUE)"
                        ));
                    } else {
                        lines.push(format!("{r_name} <- match.arg({r_name})"));
                    }
                }
            }
        }
        if lines.is_empty() {
            String::new()
        } else {
            lines.join("\n  ")
        }
    };

    // Generate lifecycle prelude if needed
    let lifecycle_prelude = lifecycle_spec
        .as_ref()
        .and_then(|spec| spec.r_prelude(&r_wrapper_ident_str));

    // Generate R-side precondition checks (stopifnot + fallback precheck calls)
    // Skip both match_arg and choices params (already validated by match.arg)
    let mut skip_params: std::collections::HashSet<String> =
        parsed.match_arg_params().cloned().collect();
    for (param_name, _) in parsed.choices_params() {
        skip_params.insert(r_wrapper_builder::normalize_r_arg_string(param_name));
    }
    // `#[miniextendr(no_preconditions)]` / `fast` opts out of the stopifnot
    // prelude entirely. TryFromSexp still raises a typed Rust error on
    // mismatched input — see analysis/scaffolding-deep-findings-2026-05-20.md
    // for why this is ~1230 ns / 1-arg or ~3900 ns / 5-arg of savings.
    let precondition_prelude = if no_preconditions {
        String::new()
    } else {
        // A coerced integer-element vector reads via `&[i32]` (INTSXP-only), so its
        // precondition tightens to `is.integer` (issue #616). `coerce_params_list`
        // holds Rust names; normalize to R names.
        let precondition_opts = r_preconditions::PreconditionOptions {
            coerce_all,
            coerce_params: coerce_params_list
                .iter()
                .map(|p| r_wrapper_builder::normalize_r_arg_string(p))
                .collect(),
        };
        let precondition_output =
            r_preconditions::build_precondition_checks(inputs, &skip_params, &precondition_opts);
        if precondition_output.static_checks.is_empty() {
            String::new()
        } else {
            precondition_output.static_checks.join("\n  ")
        }
    };

    // Combine all preludes: r_entry, on.exit, lifecycle, static preconditions, match.arg, choices, r_post_checks
    // (Missing<T> forwarding lives inline in the `.Call()` args — see
    // `build_call_args_vec` — because a prelude binding of the missing
    // sentinel errors on lookup.)
    let on_exit_str = r_on_exit.as_ref().map(|oe| oe.to_r_code());
    let combined_prelude = {
        let mut parts = Vec::new();
        if let Some(ref entry) = r_entry {
            parts.push(entry.as_str());
        }
        if let Some(ref s) = on_exit_str {
            parts.push(s.as_str());
        }
        if let Some(ref lc) = lifecycle_prelude {
            parts.push(lc.as_str());
        }
        if !precondition_prelude.is_empty() {
            parts.push(&precondition_prelude);
        }
        if !match_arg_prelude.is_empty() {
            parts.push(&match_arg_prelude);
        }
        if !choices_prelude.is_empty() {
            parts.push(&choices_prelude);
        }
        if let Some(ref post) = r_post_checks {
            parts.push(post.as_str());
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n  "))
        }
    };

    let r_wrapper_string = if let Some(prelude) = combined_prelude {
        format!(
            "{}{}{}{}{}{}{} <- function({}) {{\n  {}\n  {}\n}}",
            roxygen_tags_str,
            source_comment,
            s3_method_comment,
            internal_comment,
            no_rd_comment,
            export_comment,
            r_wrapper_ident_str,
            formals_joined,
            prelude,
            r_wrapper_return_str
        )
    } else {
        format!(
            "{}{}{}{}{}{}{} <- function({}) {{\n  {}\n}}",
            roxygen_tags_str,
            source_comment,
            s3_method_comment,
            internal_comment,
            no_rd_comment,
            export_comment,
            r_wrapper_ident_str,
            formals_joined,
            r_wrapper_return_str
        )
    };
    // Use a raw string literal for better readability in macro expansion
    let r_wrapper_str = r_wrapper_raw_literal(&r_wrapper_string);

    // endregion

    // Generate doc strings with links
    let r_wrapper_doc = format!(
        "R wrapper code for [`{}`], calls [`{}`].",
        rust_ident, c_ident
    );
    let source_start = rust_ident.span().start();
    let source_line_lit = syn::LitInt::new(&source_start.line.to_string(), rust_ident.span());
    let source_col_lit =
        syn::LitInt::new(&(source_start.column + 1).to_string(), rust_ident.span());

    // Get the normalized item for output, with roxygen tags stripped from docs.
    // Roxygen tags are for R documentation and shouldn't appear in rustdoc.
    let mut original_item = parsed.item_without_roxygen();
    // Strip only the miniextendr attributes; keep everything else.
    original_item
        .attrs
        .retain(|attr| !attr.path().is_ident("miniextendr"));

    // Inject dots_typed binding into function body if dots = typed_list!(...) was specified
    if let Some(ref spec_tokens) = dots_spec {
        let dots_param = named_dots.clone().unwrap_or_else(|| {
            syn::Ident::new("__miniextendr_dots", proc_macro2::Span::call_site())
        });
        let validation_stmt = build_dots_validation_stmt(&dots_param, spec_tokens);
        original_item.block.stmts.insert(0, validation_stmt);
    }

    let original_item = original_item;

    // Generate match_arg choices helper C wrappers and R_CallMethodDef entries
    let match_arg_helpers = build_match_arg_helpers(
        &match_arg_param_info,
        &parsed,
        &c_ident.to_string(),
        &cfg_attrs,
    );

    // Generate MX_MATCH_ARG_CHOICES entries for placeholder → choices replacement
    // Resolve the `MatchArg`-bound type used in the choices_str closure: for
    // `several_ok` params that's the inner element of the container, otherwise
    // it's the param type directly.
    let choices_ty_for = |rust_param: &str| -> Option<&syn::Type> {
        let (_, _, param_ty) = match_arg_param_info
            .iter()
            .find(|(_, rn, _)| rn == rust_param)?;
        let ty: &syn::Type = if parsed.has_several_ok(rust_param) {
            classify_several_ok_container(param_ty)
                .map(|(_, t)| t)
                .unwrap_or(param_ty)
        } else {
            param_ty
        };
        Some(ty)
    };

    let match_arg_choices_entries: Vec<proc_macro2::TokenStream> = match_arg_placeholders
        .iter()
        .filter_map(|(placeholder, rust_param, preferred_default)| {
            let choices_ty = choices_ty_for(rust_param)?;
            let entry_ident = syn::Ident::new(
                &format!(
                    "match_arg_choices_entry_{}",
                    crate::match_arg_keys::placeholder_ident_suffix(placeholder)
                ),
                proc_macro2::Span::call_site(),
            );
            Some(crate::match_arg_keys::choices_entry_tokens(
                &cfg_attrs,
                &entry_ident,
                placeholder,
                choices_ty,
                preferred_default,
            ))
        })
        .collect();

    // Generate MX_MATCH_ARG_PARAM_DOCS entries for @param doc placeholder → choice description
    let match_arg_param_doc_entries: Vec<proc_macro2::TokenStream> =
        match_arg_param_doc_placeholders
            .iter()
            .filter_map(|(doc_placeholder, rust_param)| {
                let choices_ty = choices_ty_for(rust_param)?;
                let several_ok_lit = parsed.has_several_ok(rust_param);
                let entry_ident = syn::Ident::new(
                    &format!(
                        "match_arg_param_doc_entry_{}",
                        crate::match_arg_keys::placeholder_ident_suffix(doc_placeholder)
                    ),
                    proc_macro2::Span::call_site(),
                );
                Some(crate::match_arg_keys::param_doc_entry_tokens(
                    &cfg_attrs,
                    &entry_ident,
                    doc_placeholder,
                    several_ok_lit,
                    choices_ty,
                ))
            })
            .collect();

    // Generate doc comment linking to C wrapper and R wrapper constant
    let fn_r_wrapper_doc = format!(
        "See [`{}`] for C wrapper, [`{}`] for R wrapper.",
        c_ident, r_wrapper_generator
    );

    let expanded: proc_macro::TokenStream = quote::quote! {
        // rust function with doc link to R wrapper
        #[doc = #fn_r_wrapper_doc]
        #original_item

        // C wrapper
        #(#cfg_attrs)*
        #c_wrapper

        // R wrapper (self-registers via distributed_slice)
        #(#cfg_attrs)*
        #[doc = #r_wrapper_doc]
        #[doc = concat!("Wraps Rust function `", stringify!(#rust_ident), "`.")]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_R_WRAPPERS), linkme(crate = ::miniextendr_api::linkme))]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        static #r_wrapper_generator: ::miniextendr_api::registry::RWrapperEntry =
            ::miniextendr_api::registry::RWrapperEntry {
                priority: ::miniextendr_api::registry::RWrapperPriority::Function,
                source_file: file!(),
                content: concat!(
                    "# Generated from Rust fn `",
                    stringify!(#rust_ident),
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

        // match_arg choices helpers (C wrappers + R_CallMethodDef entries)
        // Each helper's call_method_def self-registers via distributed_slice
        #(#match_arg_helpers)*

        // match_arg choices entries for R wrapper default replacement
        #(#match_arg_choices_entries)*

        // match_arg @param doc entries for R wrapper roxygen doc replacement
        #(#match_arg_param_doc_entries)*

        // doc-lint warnings (if any)
        #doc_lint_warnings
    }
    .into();

    expanded
}

/// Maps a `ReturnPref` attribute value onto an auto-detected `ReturnHandling`.
///
/// Only the plain `IntoR` variant has a bare `T` for `prefer=` to wrap, so it is the
/// only variant substituted with its pref-specific counterpart
/// (`AsListOf`/`AsExternalPtrOf`/`AsNativeOf`). Every other variant (`Unit`, `RawSexp`,
/// `ExternalPtr`, `Option*`, `Result*`) has its own fixed SEXP-shape rule that
/// `prefer=` cannot compose with — returning a compile error for those is better than
/// silently dropping the attribute (see the BUG4 audit finding: `prefer = "list"` on an
/// `Option<T>` return used to be accepted and silently ignored).
fn apply_return_pref(
    auto: c_wrapper_builder::ReturnHandling,
    pref: crate::miniextendr_fn::ReturnPref,
    pref_span: Option<proc_macro2::Span>,
) -> syn::Result<c_wrapper_builder::ReturnHandling> {
    use crate::miniextendr_fn::ReturnPref;
    use c_wrapper_builder::ReturnHandling;

    let wrapped = match pref {
        ReturnPref::Auto => return Ok(auto),
        ReturnPref::List => match auto {
            ReturnHandling::IntoR => Some(ReturnHandling::AsListOf),
            _ => None,
        },
        ReturnPref::ExternalPtr => match auto {
            ReturnHandling::IntoR => Some(ReturnHandling::AsExternalPtrOf),
            _ => None,
        },
        ReturnPref::Native => match auto {
            ReturnHandling::IntoR => Some(ReturnHandling::AsNativeOf),
            _ => None,
        },
    };

    wrapped.ok_or_else(|| {
        let (pref_name, wrapper_name) = return_pref_names(pref);
        let span = pref_span.unwrap_or_else(proc_macro2::Span::call_site);
        syn::Error::new(
            span,
            format!(
                "`prefer = \"{pref_name}\"` cannot be honored on this return type. \
                 `prefer=` only applies to a function returning a plain `T: IntoR` value, \
                 which it wraps in `{wrapper_name}` before conversion. This function's return \
                 type falls into a different codegen category ({}) with its own fixed \
                 SEXP-shape rule, so there is no plain `T` for `prefer=` to wrap. Remove \
                 `prefer=`, or change the return type to a plain `T`.",
                return_handling_category_description(&auto),
            ),
        )
    })
}

/// Human-readable `(attribute value, wrapper type)` pair for a [`ReturnPref`](crate::miniextendr_fn::ReturnPref),
/// used to phrase the `apply_return_pref` compile error.
fn return_pref_names(pref: crate::miniextendr_fn::ReturnPref) -> (&'static str, &'static str) {
    use crate::miniextendr_fn::ReturnPref;
    match pref {
        ReturnPref::Auto => ("auto", ""),
        ReturnPref::List => ("list", "AsList"),
        ReturnPref::ExternalPtr => ("externalptr", "AsExternalPtr"),
        ReturnPref::Native => ("native", "AsRNative"),
    }
}

/// Human-readable description of a [`ReturnHandling`](c_wrapper_builder::ReturnHandling)
/// category, for the `apply_return_pref` compile error. Only describes the categories
/// [`c_wrapper_builder::detect_return_handling_standalone_fn`] can actually produce;
/// the wildcard arm covers variants that never reach `apply_return_pref` as `auto`
/// (`IntoR` itself, method-only `SelfHandle`, and the `As*Of` variants `apply_return_pref`
/// produces as *output*, never takes as input).
fn return_handling_category_description(rh: &c_wrapper_builder::ReturnHandling) -> &'static str {
    use c_wrapper_builder::ReturnHandling;
    match rh {
        ReturnHandling::Unit => "the unit return type `()`",
        ReturnHandling::RawSexp => "a raw `SEXP` return type",
        ReturnHandling::ExternalPtr => {
            "a `Self`-returning constructor, already converted via `ExternalPtr::new`"
        }
        ReturnHandling::OptionUnit => "`Option<()>`",
        ReturnHandling::OptionSexp => "`Option<SEXP>`",
        ReturnHandling::OptionIntoR | ReturnHandling::OptionIntoRUnwrap => "`Option<T>`",
        ReturnHandling::ResultUnit => "`Result<(), E>`",
        ReturnHandling::ResultSexp => "`Result<SEXP, E>`",
        ReturnHandling::ResultIntoR => "`Result<T, E>`",
        ReturnHandling::ResultNullOnErr => "`Result<T, ()>`",
        _ => "this return type",
    }
}

/// Generate thread-safe wrappers for R FFI functions.
///
/// Apply this to an `extern "C-unwind"` block to generate, **for each
/// non-variadic function**, a pair of entry points:
///
/// - The original name (e.g. `Rf_allocVector`) — a safe Rust wrapper that
///   runs directly on R's main thread, routes through
///   `miniextendr_api::worker::with_r_thread` from an active miniextendr
///   worker context, and panics for arbitrary off-main callers.
/// - A `*_unchecked` sibling (`Rf_allocVector_unchecked`) — the raw
///   `extern "C-unwind"` declaration with no main-thread assertion and no
///   worker round-trip.
///
/// User code should reach for the checked variant by default; the unchecked
/// sibling exists for three known-safe contexts:
///
/// 1. **Inside ALTREP callbacks** — R is already calling us on the main
///    thread, so the assertion would always pass and the route would
///    deadlock the call back to R.
/// 2. **Inside a `with_r_unwind_protect` body** — the guard has established
///    main-thread context, and re-entering `with_r_thread` would nest two
///    `R_UnwindProtect` frames (paying the longjmp-leak cost twice).
/// 3. **Inside a `with_r_thread` body** — the assertion is redundant; you
///    are already where you needed to be.
///
/// The build-time lint **MXL301** enforces this: calling `*_unchecked`
/// outside one of those three contexts is a compile-time error. Without the
/// `worker-thread` feature, the checked variant still enforces the recorded
/// main-thread contract; it simply has no worker route available.
///
/// # Tradeoffs at a glance
///
/// | Variant | Asserts main thread | Routes to main | When to use |
/// |---|---|---|---|
/// | `Rf_foo` (checked) | yes (debug) | yes (from worker) | default |
/// | `Rf_foo_unchecked` | no | no | ALTREP callbacks, `with_r_unwind_protect`, `with_r_thread` |
///
/// # Behavior
///
/// All non-variadic functions are routed to the main thread via `with_r_thread`
/// when called from a worker thread. The return value is wrapped in `Sendable`
/// and sent back to the caller. This applies to both value-returning functions
/// (SEXP, i32, etc.) and pointer-returning functions (`*const T`, `*mut T`).
///
/// Pointer-returning functions (like `INTEGER`, `REAL`) are safe to route because
/// the underlying SEXP must be GC-protected by the caller, and R's GC only runs
/// during R API calls which are serialized through `with_r_thread`.
///
/// # Initialization Requirement
///
/// `miniextendr_runtime_init()` must be called before using any wrapped function.
/// Calling before initialization will panic with a descriptive error message.
///
/// # Limitations
///
/// - Variadic functions are passed through unchanged (no wrapper)
/// - Statics are passed through unchanged
/// - Functions with `#[link_name]` are passed through unchanged
///
/// # Example
///
/// ```ignore
/// #[r_ffi_checked]
/// unsafe extern "C-unwind" {
///     // Routed to main thread via with_r_thread when called from worker
///     pub fn Rf_ScalarInteger(arg1: i32) -> SEXP;
///     pub fn INTEGER(x: SEXP) -> *mut i32;
/// }
/// ```
#[proc_macro_attribute]
pub fn r_ffi_checked(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let foreign_mod = syn::parse_macro_input!(item as syn::ItemForeignMod);

    let foreign_mod_attrs = &foreign_mod.attrs;
    let abi = &foreign_mod.abi;
    let mut unchecked_items = Vec::new();
    let mut checked_wrappers = Vec::new();

    for item in &foreign_mod.items {
        match item {
            syn::ForeignItem::Fn(fn_item) => {
                let is_variadic = fn_item.sig.variadic.is_some();

                // Check if function already has #[link_name] - if so, pass through unchanged
                let has_link_name = fn_item
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("link_name"));

                if is_variadic || has_link_name {
                    // Pass through variadic functions and functions with explicit link_name unchanged
                    unchecked_items.push(item.clone());
                } else {
                    // Generate checked wrapper for non-variadic functions
                    let vis = &fn_item.vis;
                    let fn_name = &fn_item.sig.ident;
                    let fn_name_str = fn_name.to_string();
                    let unchecked_name = quote::format_ident!("{}_unchecked", fn_name);
                    let unchecked_name_str = unchecked_name.to_string();
                    let inputs = &fn_item.sig.inputs;
                    let output = &fn_item.sig.output;
                    // Filter out link_name attributes (already checked above, but be safe)
                    let attrs: Vec<_> = fn_item
                        .attrs
                        .iter()
                        .filter(|attr| !attr.path().is_ident("link_name"))
                        .collect();
                    let checked_doc = format!(
                        "Checked wrapper for `{}`. Calls `{}` and routes through `with_r_thread`.",
                        fn_name_str, unchecked_name_str
                    );
                    let checked_doc_lit = syn::LitStr::new(&checked_doc, fn_name.span());
                    let source_loc_doc = crate::source_location_doc(fn_name.span());
                    let source_loc_doc_lit = syn::LitStr::new(&source_loc_doc, fn_name.span());

                    // Generate the unchecked FFI binding with #[link_name]
                    // Same visibility as the checked variant
                    let link_name = syn::LitStr::new(&fn_name_str, fn_name.span());
                    let unchecked_fn: syn::ForeignItem = syn::parse_quote! {
                        #(#attrs)*
                        #[doc = concat!("Unchecked FFI binding for `", stringify!(#fn_name), "`.")]
                        #[doc = #source_loc_doc_lit]
                        #[doc = concat!("Generated from source file `", file!(), "`.")]
                        #[link_name = #link_name]
                        #vis fn #unchecked_name(#inputs) #output;
                    };
                    unchecked_items.push(unchecked_fn);

                    // Generate a checked wrapper function
                    let arg_names: Vec<_> = inputs
                        .iter()
                        .filter_map(|arg| {
                            if let syn::FnArg::Typed(pat_type) = arg
                                && let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref()
                            {
                                Some(pat_ident.ident.clone())
                            } else {
                                None
                            }
                        })
                        .collect();

                    let is_never = matches!(output, syn::ReturnType::Type(_, ty) if matches!(**ty, syn::Type::Never(_)));

                    let wrapper = if is_never {
                        // Never-returning functions (like Rf_error)
                        quote::quote! {
                            #(#attrs)*
                            #[doc = #checked_doc_lit]
                            #[doc = #source_loc_doc_lit]
                            #[doc = concat!("Generated from source file `", file!(), "`.")]
                            #[inline(always)]
                            #[allow(non_snake_case)]
                            #vis unsafe fn #fn_name(#inputs) #output {
                                ::miniextendr_api::worker::with_r_thread(move || unsafe {
                                    #unchecked_name(#(#arg_names),*)
                                })
                            }
                        }
                    } else {
                        // Normal functions - route via with_r_thread
                        quote::quote! {
                            #(#attrs)*
                            #[doc = #checked_doc_lit]
                            #[doc = #source_loc_doc_lit]
                            #[doc = concat!("Generated from source file `", file!(), "`.")]
                            #[inline(always)]
                            #[allow(non_snake_case)]
                            #vis unsafe fn #fn_name(#inputs) #output {
                                let result = ::miniextendr_api::worker::with_r_thread(move || {
                                    ::miniextendr_api::worker::Sendable(unsafe {
                                        #unchecked_name(#(#arg_names),*)
                                    })
                                });
                                result.0
                            }
                        }
                    };
                    checked_wrappers.push(wrapper);
                }
            }
            _ => {
                // Pass through statics and other items unchanged
                unchecked_items.push(item.clone());
            }
        }
    }

    let expanded = quote::quote! {
        #(#foreign_mod_attrs)*
        unsafe #abi {
            #(#unchecked_items)*
        }

        #(#checked_wrappers)*
    };

    expanded.into()
}

/// Derive macro for implementing `RNativeType` on a newtype wrapper.
///
/// This allows newtype wrappers around R native types to work with `Vec<T>`,
/// `&[T]` conversions and the `Coerce<R>` traits.
/// The inner type must implement `RNativeType`.
///
/// # Supported Struct Forms
///
/// Both tuple structs and single-field named structs are supported:
///
/// ```ignore
/// use miniextendr_api::RNativeType;
///
/// // Tuple struct (most common)
/// #[derive(Clone, Copy, RNativeType)]
/// struct UserId(i32);
///
/// // Named single-field struct
/// #[derive(Clone, Copy, RNativeType)]
/// struct Temperature { celsius: f64 }
/// ```
///
/// # Generated Code
///
/// For `struct UserId(i32)`, this generates:
///
/// ```ignore
/// impl RNativeType for UserId {
///     const SEXP_TYPE: SEXPTYPE = <i32 as RNativeType>::SEXP_TYPE;
///     const R_NA: Self = UserId(<i32 as RNativeType>::R_NA);
///
///     unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
///         <i32 as RNativeType>::dataptr_mut(sexp).cast()
///     }
/// }
/// ```
///
/// # Using the Newtype with Coerce
///
/// Once `RNativeType` is derived, you can implement `Coerce` to/from the newtype:
///
/// ```ignore
/// impl Coerce<UserId> for i32 {
///     fn coerce(self) -> UserId { UserId(self) }
/// }
///
/// let id: UserId = 42.coerce();
/// ```
///
/// # Requirements
///
/// - Must be a newtype struct (exactly one field, tuple or named)
/// - The inner type must implement `RNativeType` (`i32`, `f64`, `RLogical`, `u8`, `Rcomplex`)
/// - Should also derive `Copy` (required by `RNativeType: Copy`)
#[proc_macro_derive(RNativeType)]
pub fn derive_rnative_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Extract inner type and constructor — must be a newtype (single field)
    let (inner_ty, elt_ctor): (syn::Type, proc_macro2::TokenStream) = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let ty = fields.unnamed.first().unwrap().ty.clone();
                let ctor = quote::quote! { Self(val) };
                (ty, ctor)
            }
            syn::Fields::Named(fields) if fields.named.len() == 1 => {
                let field = fields.named.first().unwrap();
                let ty = field.ty.clone();
                let field_name = field.ident.as_ref().unwrap();
                let ctor = quote::quote! { Self { #field_name: val } };
                (ty, ctor)
            }
            _ => {
                return syn::Error::new_spanned(
                    name,
                    "#[derive(RNativeType)] requires a newtype struct with exactly one field",
                )
                .into_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "#[derive(RNativeType)] only works on structs")
                .into_compile_error()
                .into();
        }
    };

    let expanded = quote::quote! {
        impl #impl_generics ::miniextendr_api::RNativeType for #name #ty_generics #where_clause {
            const SEXP_TYPE: ::miniextendr_api::SEXPTYPE =
                <#inner_ty as ::miniextendr_api::RNativeType>::SEXP_TYPE;

            const R_NA: Self = {
                let val = <#inner_ty as ::miniextendr_api::RNativeType>::R_NA;
                #elt_ctor
            };

            #[inline]
            unsafe fn dataptr_mut(sexp: ::miniextendr_api::SEXP) -> *mut Self {
                // Newtype is repr(transparent), so we can cast the pointer
                unsafe {
                    <#inner_ty as ::miniextendr_api::RNativeType>::dataptr_mut(sexp).cast()
                }
            }

            #[inline]
            fn elt(sexp: ::miniextendr_api::SEXP, i: isize) -> Self {
                let val = <#inner_ty as ::miniextendr_api::RNativeType>::elt(sexp, i);
                #elt_ctor
            }
        }

    };

    expanded.into()
}

/// Derive macro for implementing `TypedExternal` on a type.
///
/// This makes the type compatible with `ExternalPtr<T>` for storing in R's external pointers.
///
/// # Basic Usage
///
/// ```ignore
/// use miniextendr_api::TypedExternal;
///
/// #[derive(ExternalPtr)]
/// struct MyData {
///     value: i32,
/// }
///
/// // Now you can use ExternalPtr<MyData>
/// let ptr = ExternalPtr::new(MyData { value: 42 });
/// ```
///
/// # Trait ABI
///
/// Trait dispatch wrappers are automatically generated:
///
/// ```ignore
/// use miniextendr_api::miniextendr;
///
/// #[derive(ExternalPtr)]
/// struct MyCounter {
///     value: i32,
/// }
///
/// #[miniextendr]
/// impl Counter for MyCounter {
///     fn value(&self) -> i32 { self.value }
///     fn increment(&mut self) { self.value += 1; }
/// }
/// ```
///
/// This generates additional infrastructure for type-erased trait dispatch:
/// - `__MxWrapperMyCounter` - Type-erased wrapper struct
/// - `__MX_BASE_VTABLE_MYCOUNTER` - Base vtable with drop/query
/// - `__mx_wrap_mycounter()` - Constructor returning `*mut mx_erased`
///
/// # Generated Code (Basic)
///
/// For a type `MyData` without traits:
///
/// ```ignore
/// impl TypedExternal for MyData {
///     const TYPE_NAME: &'static str = "MyData";
///     const TYPE_NAME_CSTR: &'static [u8] = b"MyData\0";
/// }
/// ```
#[proc_macro_derive(ExternalPtr, attributes(externalptr, r_data))]
pub fn derive_external_ptr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    // Standalone `#[derive(ExternalPtr)]` always emits the `IntoExternalPtr`
    // marker (enabling the blanket `IntoR`); only the struct-level
    // `prefer = "native"` dispatch path suppresses it (#1283).
    externalptr_derive::derive_external_ptr(input, true)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP integer vector data types.
///
/// Auto-implements `AltrepLen`, `AltIntegerData`, and the low-level ALTREP
/// trait impls (`Altrep`, `AltVec`, `AltInteger`, `InferBase`).
///
/// # Attributes
///
/// - `#[altrep(len = "field_name")]` - Specify length field (auto-detects "len" or "length")
/// - `#[altrep(elt = "field_name")]` - For constant vectors, specify which field provides elements
/// - `#[altrep(dataptr)]` - Enable direct data-pointer access
/// - `#[altrep(serialize)]` - Enable ALTREP serialization support
/// - `#[altrep(subset)]` - Enable `Extract_subset` optimization
/// - `#[altrep(no_lowlevel)]` - Skip the automatic low-level trait impls
///
/// # Example (Constant Vector - Zero Boilerplate!)
///
/// ```ignore
/// #[derive(ExternalPtr, AltrepInteger)]
/// #[altrep(elt = "value")]  // All elements return this field
/// pub struct ConstantIntData {
///     value: i32,
///     len: usize,
/// }
///
/// // That's it! 3 lines instead of 30!
/// // AltrepLen, AltIntegerData, and low-level impls are auto-generated
///
/// #[miniextendr(class = "ConstantInt")]
/// pub struct ConstantIntClass(pub ConstantIntData);
/// ```
///
/// # Example (Custom elt() - Override One Method)
///
/// ```ignore
/// #[derive(ExternalPtr, AltrepInteger)]
/// pub struct ArithSeqData {
///     start: i32,
///     step: i32,
///     len: usize,
/// }
///
/// // Auto-generates AltrepLen and stub AltIntegerData
/// // Just override elt() for custom logic:
/// impl AltIntegerData for ArithSeqData {
///     fn elt(&self, i: usize) -> i32 {
///         self.start + (i as i32) * self.step
///     }
/// }
/// ```
#[proc_macro_derive(AltrepInteger, attributes(altrep))]
pub fn derive_altrep_integer(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_integer(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP real vector data types.
///
/// Auto-implements `AltrepLen` and `AltRealData` traits.
/// Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).
#[proc_macro_derive(AltrepReal, attributes(altrep))]
pub fn derive_altrep_real(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_real(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP logical vector data types.
///
/// Auto-implements `AltrepLen` and `AltLogicalData` traits.
/// Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).
#[proc_macro_derive(AltrepLogical, attributes(altrep))]
pub fn derive_altrep_logical(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_logical(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP raw vector data types.
///
/// Auto-implements `AltrepLen` and `AltRawData` traits.
/// Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).
#[proc_macro_derive(AltrepRaw, attributes(altrep))]
pub fn derive_altrep_raw(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_raw(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP string vector data types.
///
/// Auto-implements `AltrepLen` and `AltStringData` traits.
/// Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).
#[proc_macro_derive(AltrepString, attributes(altrep))]
pub fn derive_altrep_string(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_string(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP complex vector data types.
///
/// Auto-implements `AltrepLen` and `AltComplexData` traits.
/// Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).
#[proc_macro_derive(AltrepComplex, attributes(altrep))]
pub fn derive_altrep_complex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_complex(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP list vector data types.
///
/// Auto-implements `AltrepLen` and `AltListData` traits.
/// Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger),
/// except `dataptr` and `subset` which are not supported for list ALTREP.
#[proc_macro_derive(AltrepList, attributes(altrep))]
pub fn derive_altrep_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_list(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive ALTREP registration for a data struct.
///
/// Generates `TypedExternal`, `AltrepClass`, `RegisterAltrep`, `IntoR`,
/// linkme registration entry, and `Ref`/`Mut` accessor types.
///
/// The struct must already have low-level ALTREP traits implemented.
/// For most use cases, prefer a family-specific derive:
/// `#[derive(AltrepInteger)]`, `#[derive(AltrepReal)]`, etc.
/// Use `#[altrep(manual)]` on a family derive to skip data trait generation
/// when you provide your own `AltrepLen` + `Alt*Data` impls.
///
/// # Attributes
///
/// - `#[altrep(class = "Name")]` — custom ALTREP class name (defaults to struct name)
///
/// # Example
///
/// ```ignore
/// // Prefer family derives with manual:
/// #[derive(AltrepInteger)]
/// #[altrep(manual, class = "MyCustom", serialize)]
/// struct MyData { ... }
///
/// impl AltrepLen for MyData { ... }
/// impl AltIntegerData for MyData { ... }
/// ```
#[proc_macro_derive(Altrep, attributes(altrep))]
pub fn derive_altrep(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep::derive_altrep(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `IntoList` for a struct (Rust → R list).
///
/// - Named structs → named R list: `list(x = 1L, y = 2L)`
/// - Tuple structs → unnamed R list: `list(1L, 2L)`
/// - Fields annotated `#[into_list(ignore)]` are skipped
#[proc_macro_derive(IntoList, attributes(into_list))]
pub fn derive_into_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_into_list(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `TryFromList` for a struct (R list → Rust).
///
/// - Named structs: extract by field name
/// - Tuple structs: extract by position (0, 1, 2, ...)
/// - Fields annotated `#[into_list(ignore)]` are not read and are initialized with `Default::default()`
#[proc_macro_derive(TryFromList, attributes(into_list))]
pub fn derive_try_from_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_try_from_list(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PreferList`: emits an `IntoR` impl selecting list as the type's default
/// Rust→R conversion (via `IntoList::into_list`).
///
/// A type carries exactly one representation default: stacking two `Prefer*`
/// derives is a compile error. Each `Prefer*` derive emits a fixed-name marker
/// const, so a second one triggers a guided `duplicate definitions with name
/// __miniextendr_conflicting_Prefer_derives__keep_ONE_or_use_call_site_As_wrappers`
/// error (alongside the raw conflicting-`IntoR`-impl error) — keep one `Prefer*`,
/// or drop them all and choose a representation per return value at the call site
/// with an `As*` wrapper (`AsList`, `AsExternalPtr`, `AsDataFrame`, ...).
///
/// # Example
///
/// ```ignore
/// #[derive(IntoList, PreferList)]
/// struct Config { verbose: bool, threads: i32 }
/// // IntoR produces list(verbose = TRUE, threads = 4L)
/// ```
#[proc_macro_derive(PreferList)]
pub fn derive_prefer_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_list(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PreferDataFrame`: when a type implements both `IntoDataFrame` (via `DataFrameRow`)
/// and other conversion paths, this selects data.frame as the default `IntoR` conversion.
///
/// # Example
///
/// ```ignore
/// #[derive(DataFrameRow, PreferDataFrame)]
/// struct Obs { time: f64, value: f64 }
/// // IntoR produces data.frame(time = ..., value = ...)
/// ```
#[proc_macro_derive(PreferDataFrame)]
pub fn derive_prefer_data_frame(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_data_frame(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PreferExternalPtr`: when a type implements both `ExternalPtr` and
/// other conversion paths (e.g., `IntoList`), this selects `ExternalPtr` wrapping
/// as the default `IntoR` conversion.
///
/// # Example
///
/// ```ignore
/// #[derive(ExternalPtr, IntoList, PreferExternalPtr)]
/// struct Model { weights: Vec<f64> }
/// // IntoR wraps as ExternalPtr (opaque R object), not list
/// ```
#[proc_macro_derive(PreferExternalPtr)]
pub fn derive_prefer_externalptr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_externalptr(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PreferRNativeType`: when a newtype wraps an `RNativeType` and also
/// implements other conversions, this selects the native R vector conversion
/// as the default `IntoR` path.
///
/// # Example
///
/// ```ignore
/// #[derive(Copy, Clone, RNativeType, PreferRNativeType)]
/// struct Meters(f64);
/// // IntoR produces a numeric scalar, not an ExternalPtr
/// ```
#[proc_macro_derive(PreferRNativeType)]
pub fn derive_prefer_rnative(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_rnative(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PreferVctrs`: emits an `IntoR` impl converting the type to its R vctrs object via
/// `IntoVctrs::into_vctrs`.
///
/// Pair with `#[derive(Vctrs)]` (which supplies `IntoVctrs`) so the type can be returned
/// directly from `#[miniextendr]` functions instead of `value.into_vctrs().map_err(...)`.
///
/// # Example
///
/// ```ignore
/// #[derive(Vctrs, PreferVctrs)]
/// #[vctrs(class = "percent", base = "double")]
/// struct Percent { #[vctrs(data)] values: Vec<f64> }
/// // IntoR builds the `percent` vctrs vector; a build failure becomes an R error.
/// ```
#[cfg(feature = "vctrs")]
#[proc_macro_derive(PreferVctrs)]
pub fn derive_prefer_vctrs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_vctrs(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `DataFrameRow`: generates a companion `*DataFrame` type with collection fields,
/// plus `IntoR` / `TryFromSexp` / `IntoDataFrame` impls for seamless R data.frame conversion.
///
/// # Example
///
/// ```ignore
/// #[derive(DataFrameRow)]
/// struct Measurement {
///     time: f64,
///     value: f64,
/// }
///
/// // Generates MeasurementDataFrame { time: Vec<f64>, value: Vec<f64> }
/// // plus conversion impls
/// ```
///
/// # Struct-level attributes
///
/// - `#[dataframe(name = "CustomDf")]` — custom name for the generated DataFrame type
/// - `#[dataframe(align)]` — pad shorter columns with NA to match longest
/// - `#[dataframe(tag = "my_tag")]` — attach a tag attribute to the data.frame
/// - `#[dataframe(conflicts = "string")]` — resolve conflicting column types as strings
///
/// # Field-level attributes
///
/// - `#[dataframe(skip)]` — omit this field from the DataFrame
/// - `#[dataframe(rename = "col")]` — custom column name
/// - `#[dataframe(as_list)]` — keep collection as single list column (no expansion)
/// - `#[dataframe(expand)]` / `#[dataframe(unnest)]` — expand collection into suffixed columns
/// - `#[dataframe(width = N)]` — pin expansion width (shorter rows get NA)
///
/// # Public surface (which verbs to call)
///
/// Every capability the derive provides has a documented, trait-based (or `std`)
/// verb — reach for these, not any incidental inherent plumbing:
///
/// - **Rows → R `data.frame`**: `rows.into_dataframe()?` (owned, GC-rooted
///   `BuiltDataFrame`) or `rows.wrap_data_frame()` (deferred `IntoR` wrapper);
///   parallel variant `rows.into_dataframe_par()?`. From the `IntoDataFrame` /
///   `AsDataFrameExt` traits (both re-exported from `miniextendr_api::prelude`).
/// - **R `data.frame` → rows**: `Vec::<Row>::from_dataframe(&df)?` (parallel:
///   `Vec::<Row>::from_dataframe_par(&df)?`), from the `FromDataFrame` trait — or
///   the one-call `Row::try_from_dataframe(sexp)` reader on the row type.
/// - **Rows ↔ the pure-Rust columnar companion** (`<Row>DataFrame`, `Vec`-columns,
///   no R involved): the `ColumnarFrame` trait (in the prelude) —
///   `<Row>DataFrame::from_rows(rows)` / `from_rows_par(rows)` (parallel build of
///   the *companion*, which `into_dataframe_par` does not give you) and, for
///   row-iterable companions, `companion.into_rows()`. `Vec<Row>: Into<companion>`
///   and the companion's `IntoIterator` are the equivalent `std` verbs.
/// - **Enum split representation**: `rows.into_dataframe_split()` returns one
///   `data.frame` per variant as an R list (only that variant's columns — no NA
///   fill), from the `IntoDataFrameSplit` trait (in the prelude). Enum rows
///   only; struct derives don't partition.
///
/// The generated `<Row>DataFrame` / `<Row>DataFrameIter` types are intermediate
/// column-oriented companions; you rarely name them directly.
#[proc_macro_derive(DataFrameRow, attributes(dataframe))]
pub fn derive_dataframe_row(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    dataframe_derive::derive_dataframe_row(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `RFactor`: enables conversion between Rust enums and R factors.
///
/// # Usage
///
/// ```ignore
/// #[derive(Copy, Clone, RFactor)]
/// enum Color {
///     Red,
///     Green,
///     Blue,
/// }
/// ```
///
/// # Attributes
///
/// - `#[r_factor(rename = "name")]` - Rename a variant's level string
/// - `#[r_factor(rename_all = "snake_case")]` - Rename all variants (snake_case, kebab-case, lower, upper)
#[proc_macro_derive(RFactor, attributes(r_factor))]
pub fn derive_r_factor(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    factor_derive::derive_r_factor(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `MatchArg`: enables conversion between Rust enums and R character strings
/// with `match.arg` semantics (partial matching, informative errors).
///
/// # Usage
///
/// ```ignore
/// #[derive(Copy, Clone, MatchArg)]
/// enum Mode {
///     Fast,
///     Safe,
///     Debug,
/// }
/// ```
///
/// # Attributes
///
/// - `#[match_arg(rename = "name")]` - Rename a variant's choice string
/// - `#[match_arg(rename_all = "snake_case")]` - Rename all variants (snake_case, kebab-case, lower, upper)
///
/// # Generated Implementations
///
/// - `MatchArg` - Choice metadata and bidirectional conversion
/// - `TryFromSexp` - Convert R STRSXP/factor to enum (with partial matching)
/// - `IntoR` - Convert enum to R character scalar
#[proc_macro_derive(MatchArg, attributes(match_arg))]
pub fn derive_match_arg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match_arg_derive::derive_match_arg(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `TryFromSexp` for a single-field newtype: forward the R → Rust
/// conversion to the inner type.
///
/// Generates a scalar `TryFromSexp` impl that delegates to the inner type (so the
/// newtype inherits its exact SEXPTYPE checks, NA policy, and error text), plus a
/// `FromRNewtype` marker impl. The marker lets `miniextendr-api`'s container
/// blankets light up `Vec<T>` / `Option<T>` / `Vec<Option<T>>` automatically.
///
/// # Usage
///
/// ```ignore
/// use uuid::Uuid;
///
/// #[derive(TryFromSexp)]            // R -> Rust only
/// struct Pattern(regex::Regex);
///
/// #[derive(TryFromSexp, IntoR)]     // round-trip; Vec/Option containers work too
/// struct UserId(Uuid);
/// ```
///
/// Direction is chosen by which derive you list — derive only `TryFromSexp` for
/// inner types that read from R but cannot be written back (e.g. `regex::Regex`).
#[proc_macro_derive(TryFromSexp)]
pub fn derive_try_from_sexp(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    newtype_derive::derive_try_from_sexp(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `IntoR` for a single-field newtype: forward the Rust → R conversion to
/// the inner type.
///
/// Generates a scalar `IntoR` impl that delegates to the inner type, plus an
/// `IntoRNewtype` marker (powering the `Option<T>` / `Vec<Option<T>>` container
/// blankets) and a concrete `IntoRVecElement` impl (powering `Vec<T>`). See
/// `#[derive(TryFromSexp)]` for usage.
///
/// Do not derive both `IntoR` and `MatchArg` on the same type: both feed the
/// single `IntoR for Vec<T>` blanket slot and would collide (E0119).
#[proc_macro_derive(IntoR)]
pub fn derive_into_r(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    newtype_derive::derive_into_r(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `Vctrs`: enables creating vctrs-compatible S3 vector classes from Rust structs.
///
/// # Usage
///
/// ```ignore
/// #[derive(Vctrs)]
/// #[vctrs(class = "percent", base = "double")]
/// pub struct Percent {
///     data: Vec<f64>,
/// }
/// ```
///
/// # Attributes
///
/// - `#[vctrs(class = "name")]` - R class name (required)
/// - `#[vctrs(base = "type")]` - Base type: double, integer, logical, character, raw, list, record
/// - `#[vctrs(abbr = "abbr")]` - Abbreviation for `vec_ptype_abbr`
/// - `#[vctrs(inherit_base = true|false)]` - Whether to include base type in class vector
///
/// # Generated Implementations
///
/// - `VctrsClass` - Metadata trait for vctrs class information
/// - `VctrsRecord` (for `base = "record"`) - Field names for record types
#[cfg(feature = "vctrs")]
#[proc_macro_derive(Vctrs, attributes(vctrs))]
pub fn derive_vctrs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    vctrs_derive::derive_vctrs(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Create a `TypedListSpec` for validating `...` arguments or lists.
///
/// This macro provides ergonomic syntax for defining typed list specifications
/// that can be used with `Dots::typed()` to validate the structure of
/// `...` arguments passed from R.
///
/// # Syntax
///
/// ```text
/// typed_list!(
///     name => type_spec,    // required field with type
///     name? => type_spec,   // optional field with type
///     name,                 // required field, any type
///     name?,                // optional field, any type
/// )
/// ```
///
/// For strict mode (no extra fields allowed):
/// ```text
/// typed_list!(@exact; name => type_spec, ...)
/// ```
///
/// # Type Specifications
///
/// ## Base types (with optional length)
/// - `numeric()` / `numeric(4)` - Real/double vector
/// - `integer()` / `integer(4)` - Integer vector
/// - `logical()` / `logical(4)` - Logical vector
/// - `character()` / `character(4)` - Character vector
/// - `raw()` / `raw(4)` - Raw vector
/// - `complex()` / `complex(4)` - Complex vector
/// - `list()` / `list(4)` - List (VECSXP)
///
/// ## Special types
/// - `data_frame()` - Data frame
/// - `factor()` - Factor
/// - `matrix()` - Matrix
/// - `array()` - Array
/// - `function()` - Function
/// - `environment()` - Environment
/// - `null()` - NULL only
/// - `any()` - Any type
///
/// ## String literals
/// - `"numeric"`, `"integer"`, etc. - Same as call syntax
/// - `"data.frame"` - Data frame (alias)
/// - `"MyClass"` - Any other string is treated as a class name (uses `Rf_inherits`)
///
/// # Examples
///
/// ## Basic usage
///
/// ```ignore
/// use miniextendr_api::{miniextendr, typed_list, Dots};
///
/// #[miniextendr]
/// pub fn process_args(dots: ...) -> Result<i32, String> {
///     let args = dots.typed(typed_list!(
///         alpha => numeric(4),
///         beta => list(),
///         gamma? => "character",
///     )).map_err(|e| e.to_string())?;
///
///     let alpha: Vec<f64> = args.get("alpha").map_err(|e| e.to_string())?;
///     Ok(alpha.len() as i32)
/// }
/// ```
///
/// ## Strict mode
///
/// ```ignore
/// // Reject any extra named fields
/// let args = dots.typed(typed_list!(@exact;
///     x => numeric(),
///     y => numeric(),
/// ))?;
/// ```
///
/// ## Class checking
///
/// ```ignore
/// // Check for specific R class (uses Rf_inherits semantics)
/// let args = dots.typed(typed_list!(
///     data => "data.frame",
///     model => "lm",
/// ))?;
/// ```
///
/// ## Attribute sugar
///
/// Instead of calling `.typed()` manually, you can use `typed_list!` directly in the
/// `#[miniextendr]` attribute for automatic validation:
///
/// ```ignore
/// #[miniextendr(dots = typed_list!(x => numeric(), y => numeric()))]
/// pub fn my_func(...) -> String {
///     // `dots_typed` is automatically created and validated
///     let x: f64 = dots_typed.get("x").expect("x");
///     let y: f64 = dots_typed.get("y").expect("y");
///     format!("x={}, y={}", x, y)
/// }
/// ```
///
/// This injects validation at the start of the function body:
/// ```ignore
/// let dots_typed = _dots.typed(typed_list!(...))
///     .unwrap_or_else(|e| panic!("dots validation failed: {e}"));
/// ```
///
/// See the [`#[miniextendr]`](macro@miniextendr) attribute documentation for more details.
///
#[proc_macro]
pub fn typed_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(input as typed_list::TypedListInput);
    typed_list::expand_typed_list(parsed).into()
}

/// Define a compile-time-validated wrapper for an R `data.frame` input.
///
/// `typed_dataframe!` mirrors [`typed_list!`] for the data.frame shape:
/// declare the columns once, get a struct that implements `TryFromSexp`
/// (validating both the `data.frame` class and per-column SEXPTYPE) plus
/// per-column borrowed accessors that return `&[T]`.
///
/// # Syntax
///
/// ```ignore
/// typed_dataframe! {
///     /// The shape we accept for the Theoph PK dataset.
///     pub TheophDf {
///         subject: i32,
///         weight: f64,
///         dose: f64,
///         flag: Option<i32>,   // optional column
///     }
/// }
/// ```
///
/// For strict mode (reject any column not declared):
/// ```ignore
/// typed_dataframe! {
///     @exact;
///     pub Strict { x: i32 }
/// }
/// ```
///
/// # Supported element types
///
/// v1 supports column element types that implement
/// `miniextendr_api::RNativeType`:
///
/// - `i32` — `INTSXP`
/// - `f64` — `REALSXP`
/// - `u8` — `RAWSXP`
/// - `miniextendr_api::RLogical` — `LGLSXP`
/// - `miniextendr_api::Rcomplex` — `CPLXSXP`
///
/// `String`/`&str` column types are not yet supported (character vectors
/// don't expose a contiguous slice). `bool` is also not yet supported as
/// a direct field type — use `RLogical` and convert per-element, or
/// follow the open follow-up issues from PR #698.
///
/// # Generated API
///
/// For each `name: T` column the macro emits:
/// - `pub fn name(&self) -> &[T]` (required)
/// - `pub fn name(&self) -> Option<&[T]>` (optional, `Option<T>`)
///
/// Plus housekeeping:
/// - `pub fn nrow(&self) -> usize`
/// - `pub fn ncol(&self) -> usize` (count of *declared* columns)
/// - `pub fn as_sexp(&self) -> SEXP`
///
/// All borrowed accessors are bound to `&self`; the SEXP is protected
/// by the surrounding `#[miniextendr]` call wrapper while the struct is
/// alive.
///
/// # Error reporting
///
/// `TryFromSexp::try_from_sexp` batches every per-column error into a
/// single `SexpError::InvalidValue`, so the R user sees one diagnostic
/// covering all missing or wrong-typed columns rather than a sequence of
/// stop-on-first-failure messages.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{miniextendr, typed_dataframe};
///
/// typed_dataframe! {
///     pub TheophDf {
///         subject: i32,
///         weight: f64,
///         dose: f64,
///     }
/// }
///
/// #[miniextendr]
/// pub fn theoph_nrow(df: TheophDf) -> i32 {
///     // df.subject() -> &[i32], df.weight() -> &[f64]
///     // Lengths are guaranteed equal across columns (data.frame invariant).
///     df.nrow() as i32
/// }
/// ```
///
/// [`typed_list!`]: macro@typed_list
#[proc_macro]
pub fn typed_dataframe(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(input as typed_dataframe::TypedDataframeInput);
    typed_dataframe::expand_typed_dataframe(parsed).into()
}

/// Construct an R list from Rust values.
///
/// This macro provides a convenient way to create R lists in Rust code,
/// using R-like syntax. Values are converted to R objects via the [`IntoR`] trait.
///
/// # Syntax
///
/// ```ignore
/// // Named entries (like R's list())
/// list!(
///     alpha = 1,
///     beta = "hello",
///     "my-name" = vec![1, 2, 3],
/// )
///
/// // Unnamed entries
/// list!(1, "hello", vec![1, 2, 3])
///
/// // Mixed (unnamed entries get empty string names)
/// list!(alpha = 1, 2, beta = "hello")
///
/// // Empty list
/// list!()
/// ```
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::{list, IntoR};
///
/// // Create a named list
/// let my_list = list!(
///     x = 42,
///     y = "hello world",
///     z = vec![1.0, 2.0, 3.0],
/// );
///
/// // In R this is equivalent to:
/// // list(x = 42L, y = "hello world", z = c(1, 2, 3))
/// ```
///
/// [`IntoR`]: https://docs.rs/miniextendr-api/latest/miniextendr_api/into_r/trait.IntoR.html
#[proc_macro]
pub fn list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(input as list_macro::ListInput);
    list_macro::expand_list(parsed).into()
}

/// Evaluate R code written as **Rust tokens**, validated at compile time.
///
/// `r!` takes a single R expression as a token stream, `stringify!`s it into a
/// static R source string at build time, and evaluates it via
/// `miniextendr_api::expression::r_eval_str` (the same protect-safe parse + eval
/// path as `r_str!`).
///
/// # What you get today
///
/// Because the argument is a Rust token tree, the Rust front-end already
/// rejects **unbalanced delimiters** (`r!(f(1, 2)` won't compile) and
/// lexically invalid tokens before R ever sees the string — a cheap
/// compile-time guard over the pure-runtime `r_str!`. The source is lowered to
/// a `&'static str` (`stringify!`), so there is no `format!` allocation at the
/// call site.
///
/// This proc-macro additionally validates a conservative subset of known-bad
/// R syntax constructs (trailing binary operators, consecutive non-unary
/// binary operators, bare `if`/`while`/`for` without a body, etc.) and emits
/// a precise compile error pointing at the offending token. Empty (missing)
/// call arguments — `f(, x)`, `matrix(, 2, 2)` — are valid R and pass.
///
/// # What is deferred
///
/// Direct `Rf_lang*` call-tree lowering (skipping the runtime parser entirely)
/// is tracked as a follow-up in issue #938 (item 2). Until then `r!` parses
/// its static string at first evaluation, exactly like `r_str!`.
///
/// # Non-goals
///
/// A complete R grammar validator is not achievable over Rust tokens:
/// - Single-quoted strings (`'hello'`) and backtick-quoted names (`` `foo` ``)
///   already die at the Rust lexer — nothing to validate.
/// - `%op%` tokenises as `%`, ident, `%` and is accepted without analysis.
/// - Anything the validator cannot confidently classify as wrong passes through
///   unvalidated (conservative reject-only-known-bad design).
///
/// # Forms
///
/// - `r!(R tokens…)` — evaluate in `R_GlobalEnv`.
/// - `r!(env: e; R tokens…)` — evaluate in the environment SEXP `e`. The
///   leading `env: <expr> ;` is consumed as Rust, the rest is R source.
///
/// Both evaluate to `Result<SEXP, String>`; the `SEXP` is **unprotected**.
///
/// # Safety
///
/// Expands to an `unsafe` block; the underlying FFI is `#[r_ffi_checked]`, so
/// calls from a worker thread are serialized onto the R thread.
///
/// # Example
///
/// ```ignore
/// let three = r!(1L + 2L)?;
/// let rows = r!(getFromNamespace(".theoph_rows", "dataframeflows")())?;
/// let in_env = r!(env: my_env; x + 1)?;
/// ```
#[proc_macro]
pub fn r(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    r_macro::expand(input)
}

/// Internal proc macro used by TPIE (Trait-Provided Impl Expansion).
///
/// Called by `__mx_impl_<Trait>!` macro_rules macros generated by `#[miniextendr]` on traits.
/// Do not call directly.
#[proc_macro]
#[doc(hidden)]
pub fn __mx_trait_impl_expand(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    miniextendr_impl_trait::expand_tpie(input)
}

/// Generate `TypedExternal` and `IntoExternalPtr` impls for a concrete monomorphization
/// of a generic type.
///
/// Since `#[derive(ExternalPtr)]` rejects generic types, use this macro to generate
/// the necessary impls for a specific type instantiation.
///
/// # Example
///
/// ```ignore
/// struct Wrapper<T> { inner: T }
///
/// impl_typed_external!(Wrapper<i32>);
/// impl_typed_external!(Wrapper<String>);
/// ```
#[proc_macro]
pub fn impl_typed_external(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match typed_external_macro::impl_typed_external(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

/// Generate the `R_init_*` entry point for a miniextendr R package.
///
/// This macro consolidates all package initialization into a single line.
/// It generates an `extern "C-unwind"` function that R calls when loading
/// the shared library.
///
/// # Usage
///
/// ```ignore
/// // Auto-detects package name from CARGO_CRATE_NAME (recommended):
/// miniextendr_api::miniextendr_init!();
///
/// // Or specify explicitly (for edge cases):
/// miniextendr_api::miniextendr_init!(mypkg);
/// ```
///
/// The generated function calls `miniextendr_api::init::package_init` which
/// handles panic hooks, runtime init, locale assertion, ALTREP setup, trait ABI
/// registration, routine registration, and symbol locking.
#[proc_macro]
pub fn miniextendr_init(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let pkg_name: syn::Ident = if input.is_empty() {
        // Auto-detect from CARGO_CRATE_NAME (set by cargo during compilation).
        // Cargo normalizes hyphens → underscores, so this is almost always a
        // valid Rust/C identifier. Still parse through syn so malformed values
        // surface as a compile error rather than an ICE.
        let name = match std::env::var("CARGO_CRATE_NAME") {
            Ok(n) => n,
            Err(_) => {
                return syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "CARGO_CRATE_NAME not set. Either pass the package name explicitly: \
                     miniextendr_init!(mypkg), or ensure you're building with cargo.",
                )
                .into_compile_error()
                .into();
            }
        };
        match syn::parse_str::<syn::Ident>(&name) {
            Ok(id) => id,
            Err(_) => {
                return syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!(
                        "CARGO_CRATE_NAME `{name}` is not a valid C identifier; \
                         R_init_<pkg> must match `[A-Za-z_][A-Za-z0-9_]*`. \
                         Pass the name explicitly: miniextendr_init!(my_pkg)."
                    ),
                )
                .into_compile_error()
                .into();
            }
        }
    } else {
        syn::parse_macro_input!(input as syn::Ident)
    };
    let fn_name = syn::Ident::new(&format!("R_init_{}", pkg_name), pkg_name.span());
    let unload_name = syn::Ident::new(&format!("R_unload_{}", pkg_name), pkg_name.span());

    // Build a byte string literal with NUL terminator for the package name.
    let mut name_bytes = pkg_name.to_string().into_bytes();
    name_bytes.push(0);
    let byte_lit = syn::LitByteStr::new(&name_bytes, pkg_name.span());

    let expanded = quote::quote! {
        // wasm32: pull in the host-generated `wasm_registry.rs` snapshot
        // (committed by the user crate, regenerated by `just wasm-prepare`).
        // The path is relative to the file invoking `miniextendr_init!()` —
        // by convention `<crate>/src/rust/lib.rs`, so the snapshot sits at
        // `<crate>/src/rust/wasm_registry.rs`. Module is `#[doc(hidden)]`
        // because it's purely an internal bridge between the user crate's
        // wrapper / vtable / register-fn `#[no_mangle]` exports and
        // `miniextendr_api::registry::install_wasm_runtime_slices`.
        #[cfg(target_arch = "wasm32")]
        #[path = "wasm_registry.rs"]
        #[doc(hidden)]
        mod __miniextendr_wasm_registry;

        #[unsafe(no_mangle)]
        pub unsafe extern "C-unwind" fn #fn_name(
            dll: *mut ::miniextendr_api::sys::DllInfo,
        ) {
            // wasm32: install the pre-generated runtime tables before
            // package_init runs. linkme didn't gather anything (the slices
            // are OnceLock-backed on wasm32), so register_routines /
            // universal_query would otherwise see empty slices.
            #[cfg(target_arch = "wasm32")]
            ::miniextendr_api::registry::install_wasm_runtime_slices(
                __miniextendr_wasm_registry::MX_CALL_DEFS_WASM,
                __miniextendr_wasm_registry::MX_ALTREP_REGISTRATIONS_WASM,
                __miniextendr_wasm_registry::MX_TRAIT_DISPATCH_WASM,
            );

            unsafe {
                // SAFETY: byte literal is a valid NUL-terminated string produced by the macro.
                let pkg_name = ::std::ffi::CStr::from_bytes_with_nul_unchecked(#byte_lit);
                ::miniextendr_api::init::package_init(dll, pkg_name);
            }
        }

        /// R_unload_<pkg> entry point — R calls this on `detach(unload=TRUE)` /
        /// `dyn.unload()`. Signals the miniextendr worker thread (if enabled)
        /// to exit cleanly. See `#103`.
        #[unsafe(no_mangle)]
        pub unsafe extern "C-unwind" fn #unload_name(
            _dll: *mut ::miniextendr_api::sys::DllInfo,
        ) {
            ::miniextendr_api::worker::miniextendr_runtime_shutdown();
        }

        /// Linker anchor: stub.c takes the address of this symbol to force the
        /// linker to pull in the user crate's archive member from the staticlib.
        /// With codegen-units = 1, this single member contains all linkme
        /// distributed_slice entries. The name is package-independent so stub.c
        /// doesn't need configure substitution.
        ///
        /// Defined as a function rather than a static so it stays exported under
        /// the webR wasm RUSTFLAG -Zdefault-visibility=hidden, which keeps
        /// no_mangle functions exported (like the R_init entry point) but hides
        /// no_mangle statics. A hidden anchor breaks wasm side-module dlopen
        /// (bad export type, undefined). See miniextendr webR notes (#494).
        #[unsafe(no_mangle)]
        pub extern "C" fn miniextendr_force_link() {}
    };

    expanded.into()
}

#[cfg(test)]
mod tests;
