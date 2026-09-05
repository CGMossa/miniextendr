//! Unified C wrapper generation for standalone functions and impl methods.
//!
//! This module provides shared infrastructure for generating C wrappers that:
//! - Handle worker thread vs main thread execution strategies
//! - Perform parameter conversion from SEXP to Rust types
//! - Convert Rust return values back to SEXP
//! - Properly handle panics and R errors
//!
//! The same infrastructure is used by both `#[miniextendr]` on standalone functions
//! and `#[miniextendr(env|r6|s3|s4|s7)]` on impl blocks.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// How a `#[miniextendr]` parameter type borrows R's data pointer, if it does.
///
/// `&[T]` / `&mut [T]` (and their `Option<_>` wrappers) are the `TryFromSexp`
/// impls in `miniextendr-api/src/from_r.rs` that return a *view* over R's
/// `DATAPTR` without copying (`r_slice` / `r_slice_mut`). Two such parameters
/// bound to the same SEXP alias one buffer; that is undefined behavior whenever
/// at least one of the two borrows is mutable (#1104).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SliceBorrow {
    /// `&mut [T]` / `Option<&mut [T]>` — exclusive borrow via `r_slice_mut`.
    Mut,
    /// `&[T]` / `Option<&[T]>` — shared borrow via `r_slice`.
    Shared,
}

/// Classify `ty` as a zero-copy slice borrow of R's data pointer, if it is one.
///
/// `Vec<T>` / `Box<[T]>` copy the R vector and never alias R's buffer, so they
/// are not classified. `match_arg` + `several_ok` `&mut [T]` params get their
/// own owned `Vec<T>` storage and are excluded by the caller, not here.
fn slice_borrow_kind(ty: &syn::Type) -> Option<SliceBorrow> {
    match ty {
        // &[T] / &mut [T]
        syn::Type::Reference(r) if matches!(r.elem.as_ref(), syn::Type::Slice(_)) => {
            Some(if r.mutability.is_some() {
                SliceBorrow::Mut
            } else {
                SliceBorrow::Shared
            })
        }
        // Option<&[T]> / Option<&mut [T]>
        syn::Type::Path(tp) => {
            let seg = tp.path.segments.last()?;
            if seg.ident != "Option" {
                return None;
            }
            let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
                return None;
            };
            args.args.iter().find_map(|a| match a {
                syn::GenericArgument::Type(inner) => slice_borrow_kind(inner),
                _ => None,
            })
        }
        _ => None,
    }
}

/// Thread execution strategy for C wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStrategy {
    /// Execute on main R thread with `with_r_unwind_protect`. **Default.**
    ///
    /// All code runs on R's main thread. Errors are returned as tagged SEXP values
    /// (via `make_rust_condition_value`) and the R wrapper raises structured
    /// condition objects. Simpler execution model with better R integration.
    ///
    /// Also required when:
    /// - Function takes SEXP inputs (not Send)
    /// - Function returns raw SEXP
    /// - Instance method (self_ptr isn't Send)
    /// - Function uses variadic dots (Dots type isn't Send)
    /// - `#[miniextendr(check_interrupt)]` used
    MainThread,

    /// Execute on worker thread with panic catching. **Opt-in via `#[miniextendr(worker)]`.**
    ///
    /// Structure:
    /// 1. Argument conversion on main thread
    /// 2. Function execution on worker thread via `run_on_worker`
    /// 3. SEXP conversion on main thread with `with_r_unwind_protect`
    WorkerThread,
}

/// Strategy for converting a Rust return value into an R `SEXP`.
///
/// Determined automatically by [`detect_return_handling`] from the function's return type,
/// or set explicitly via [`CWrapperContextBuilder::return_handling`]. Each variant
/// handles a different return type pattern, controlling how the C wrapper converts
/// the Rust value back to R and how errors/None values are surfaced.
#[derive(Debug, Clone)]
pub enum ReturnHandling {
    /// Returns unit type `()` -- emits `R_NilValue`.
    Unit,
    /// Returns raw `SEXP` -- passes the value through unchanged (no conversion).
    RawSexp,
    /// Returns `Self` -- wraps the value in an `ExternalPtr` via `ExternalPtr::new`.
    ExternalPtr,
    /// Returns `&Self` / `&mut Self` (an in-place builder) -- evaluates the call
    /// for its side effect (the mutation already happened through `&mut self`),
    /// discards the returned borrow, and returns the *same* `self_sexp` handle
    /// unchanged. This gives R in-place value semantics with no clone: the R
    /// object the user piped in is returned verbatim, wrapping the now-mutated
    /// Rust value. Only valid for instance methods (requires `self_sexp`).
    SelfHandle,
    /// Fallible in-place step whose call expression yields `Result<(), E>`
    /// (`&mut self -> Result<&mut Self, E>`, or `self -> Result<Self, E>` after
    /// the write-back closure): raise on `Err`, return the same `self_sexp`
    /// handle on `Ok` (#1432, #1433).
    SelfHandleResult,
    /// `Option` sibling of [`SelfHandleResult`](Self::SelfHandleResult): the
    /// call yields `Option<()>`; raise the absence error on `None`, return the
    /// same handle on `Some`.
    SelfHandleOption,
    /// Returns an arbitrary type `T: IntoR` -- converts via `IntoR::into_sexp`.
    IntoR,
    /// Returns `Option<()>` -- raises an error on `None`, otherwise emits `R_NilValue`.
    OptionUnit,
    /// Returns `Option<SEXP>` -- raises an error on `None`, otherwise passes through.
    OptionSexp,
    /// Returns `Option<T>` where `Option<T>: IntoR` -- calls `IntoR::into_sexp` on the whole
    /// `Option` value. Suitable when the type has a direct `impl IntoR for Option<T>` (e.g.,
    /// `Option<&T>`, `Option<Vec<T>>`, `Option<i32>`). `None` maps to whatever the `IntoR`
    /// impl returns (typically NULL or NA).
    ///
    /// Use this variant explicitly via [`CWrapperContextBuilder::return_handling`] when
    /// the type has a direct `IntoR` impl for the whole `Option`. The auto-detector
    /// `detect_return_handling` conservatively returns `OptionIntoRUnwrap` instead
    /// since it cannot resolve trait impls at macro expansion time.
    #[allow(dead_code)]
    // Used via explicit return_handling() call; auto-detect uses OptionIntoRUnwrap
    OptionIntoR,
    /// Returns `Option<T>` where `T: IntoR` -- unwraps the option first, then converts the
    /// inner value via `IntoR::into_sexp`. Raises an error on `None`. Suitable when `T: IntoR`
    /// but `Option<T>` doesn't have a direct `IntoR` impl (e.g., `Option<SomeExternalPtr>`).
    OptionIntoRUnwrap,
    /// Returns `Option<Self>` -- a lookup-shaped fallible constructor (e.g. `try_find`).
    /// Raises an error on `None` (same as [`OptionIntoRUnwrap`](Self::OptionIntoRUnwrap)),
    /// but wraps `Some(Self)` in an `ExternalPtr` via `ExternalPtr::new` (same as
    /// [`ExternalPtr`](Self::ExternalPtr)) instead of routing it through `IntoR` (which
    /// `Self` generally does not implement). The R-side wrapper mirrors this: a successful
    /// return is treated exactly like a bare `Self` return (wrapped class object via
    /// `ReturnStrategy::for_method` — see `ParsedMethod::returns_option_self`). Symmetric
    /// with [`ResultExternalPtr`](Self::ResultExternalPtr).
    OptionExternalPtr,
    /// Returns `Result<(), E>` -- raises an error on `Err`, otherwise emits `R_NilValue`.
    ResultUnit,
    /// Returns `Result<SEXP, E>` -- raises an error on `Err`, otherwise passes through.
    ResultSexp,
    /// Returns `Result<T, E>` -- raises an error on `Err`, otherwise converts via `IntoR::into_sexp`.
    ResultIntoR,
    /// Returns `Result<T, ()>` -- maps `Err(())` to `Err(NullOnErr)` then converts via `IntoR`.
    /// `None`/`Err` maps to R `NULL` (unit error is a deliberate sentinel, not a Rust failure).
    ResultNullOnErr,
    /// Returns `Result<Self, E>` -- a fallible constructor-shaped method (e.g. `from_r`,
    /// `try_new`). Raises an error on `Err` (same as [`ResultIntoR`](Self::ResultIntoR)), but
    /// wraps `Ok(Self)` in an `ExternalPtr` via `ExternalPtr::new` (same as
    /// [`ExternalPtr`](Self::ExternalPtr)) instead of routing it through `IntoR` (which `Self`
    /// generally does not implement). The R-side wrapper mirrors this: a successful return is
    /// treated exactly like a bare `Self` return (wrapped class object via
    /// `ReturnStrategy::for_method` — see `ParsedMethod::returns_result_self`).
    ResultExternalPtr,
    /// Returns `T` where `T: IntoList` -- wraps in `AsList(result)` then calls `IntoR::into_sexp`.
    ///
    /// Produced when `#[miniextendr(prefer = "list")]` is used on a function returning `T: IntoList`.
    /// Distinct from returning `AsList<T>` explicitly only in that the wrapping is generated
    /// by the macro rather than written by the user.
    AsListOf,
    /// Returns `T` where `T: IntoExternalPtr` -- wraps in `AsExternalPtr(result)` then calls `IntoR::into_sexp`.
    ///
    /// Produced when `#[miniextendr(prefer = "externalptr")]` is used on a function returning
    /// `T: IntoExternalPtr`. Distinct from the existing `ExternalPtr` variant (which boxes `Self`
    /// via `ExternalPtr::new`): this variant calls `IntoExternalPtr::into_external_ptr()` on `T`.
    AsExternalPtrOf,
    /// Returns `T` where `T: RNativeType` -- wraps in `AsRNative(result)` then calls `IntoR::into_sexp`.
    ///
    /// Produced when `#[miniextendr(prefer = "native")]` is used on a function returning `T: RNativeType`.
    AsNativeOf,
}

/// How the generated `Err` arm turns an error value into condition parts
/// (message, class vector, structured data).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ErrPartsMode {
    /// Autoref-specialisation probe (`__mx_result_err_parts!`): `RConditionError`
    /// when the error type implements it, `Debug` otherwise.
    #[default]
    Probe,
    /// `#[miniextendr(serde_error)]`: serialize the error with serde; the enum
    /// variant becomes the member class `<prefix>_<variant>`, the payload fields
    /// become the condition's data. `tag` is the internally-tagged discriminator
    /// field consumed as the variant (`#[serde(tag = "kind")]`); `skip` and
    /// `rename` are the payload-field controls (`skip(...)` / `rename(...)`).
    Serde {
        tag: String,
        prefix: String,
        skip: Vec<String>,
        rename: Vec<(String, String)>,
    },
}

impl ErrPartsMode {
    /// Resolve a parsed `serde_error` spec (or its absence) into a mode.
    pub fn from_spec(spec: Option<&crate::miniextendr_fn::SerdeErrorSpec>) -> Self {
        match spec {
            None => ErrPartsMode::Probe,
            Some(spec) => ErrPartsMode::Serde {
                tag: spec.tag().to_string(),
                prefix: spec.prefix(),
                skip: spec.skip.clone(),
                rename: spec.rename.clone(),
            },
        }
    }

    /// The expression yielding an `ErrParts` from the bound error `e`.
    pub fn expr(&self) -> TokenStream {
        match self {
            ErrPartsMode::Probe => quote! { ::miniextendr_api::__mx_result_err_parts!(e) },
            ErrPartsMode::Serde {
                tag,
                prefix,
                skip,
                rename,
            } => {
                let (from, to): (Vec<&String>, Vec<&String>) =
                    rename.iter().map(|(f, t)| (f, t)).unzip();
                quote! {
                    ::miniextendr_api::condition::serde_err_parts(
                        &e,
                        #tag,
                        #prefix,
                        &[#(#skip),*],
                        &[#((#from, #to)),*],
                    )
                }
            }
        }
    }
}

/// All information needed to generate a C wrapper function for an R-exported Rust item.
///
/// This struct abstracts over the differences between standalone `#[miniextendr]` functions
/// and `impl` block methods (R6, S3, S4, S7, Env). It is constructed via
/// [`CWrapperContextBuilder`] and consumed by [`CWrapperContext::generate`], which emits
/// both the `extern "C-unwind"` wrapper and the corresponding `R_CallMethodDef` constant.
pub struct CWrapperContext {
    /// Identifier of the original Rust function or method being wrapped.
    pub fn_ident: syn::Ident,
    /// Identifier for the generated C wrapper (e.g., `C_<crate>_foo` or `C_<crate>_Type__method`).
    pub c_ident: syn::Ident,
    /// Identifier of the `R_WRAPPER_*` or `R_WRAPPERS_IMPL_*` const that holds the
    /// generated R wrapper code string. Used for rustdoc cross-references.
    pub r_wrapper_const: syn::Ident,
    /// Function parameters (excluding the `self` receiver for methods).
    /// Each parameter becomes a `SEXP` argument in the C wrapper signature.
    pub inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    /// The original Rust return type. Used by strict-mode to inspect whether the inner
    /// type is lossy (e.g., `i64`, `u64`) and needs checked conversion.
    pub output: syn::ReturnType,
    /// Statements emitted before the call expression. For instance methods, this
    /// includes extracting `self` from the `ExternalPtr` SEXP.
    pub pre_call: Vec<TokenStream>,
    /// The actual Rust call expression (e.g., `my_func(arg0, arg1)` or
    /// `self_ref.method(arg0)`). Inserted into the wrapper body after conversions.
    pub call_expr: TokenStream,
    /// Whether to run on the main R thread or dispatch to the worker thread.
    pub thread_strategy: ThreadStrategy,
    /// How to convert the Rust return value into a `SEXP` for R.
    pub return_handling: ReturnHandling,
    /// When `true`, all parameters use coercing conversion (`Rf_coerceVector`) instead
    /// of strict type-matching. Set by `#[miniextendr(coerce)]`.
    pub coerce_all: bool,
    /// Names of individual parameters that use coercing conversion.
    /// Set by `#[miniextendr(coerce = "param_name")]`.
    pub coerce_params: Vec<String>,
    /// When `true`, emits `R_CheckUserInterrupt()` before the call expression.
    /// Set by `#[miniextendr(check_interrupt)]`.
    pub check_interrupt: bool,
    /// When `true`, wraps the call in `GetRNGstate()`/`PutRNGstate()` for R's
    /// random number generator state management. Set by `#[miniextendr(rng)]`.
    pub rng: bool,
    /// `#[cfg(...)]` attributes from the original item, propagated to the C wrapper
    /// and `call_method_def` constant so they are conditionally compiled.
    pub cfg_attrs: Vec<syn::Attribute>,
    /// For methods: the type identifier (e.g., `MyStruct`). Used in doc comments
    /// and default `call_method_def` naming. `None` for standalone functions.
    pub type_context: Option<syn::Ident>,
    /// Whether the original method has a `self` receiver. When `true`, the C wrapper
    /// includes a `self_sexp` parameter before the regular arguments.
    pub has_self: bool,
    /// Override for the `call_method_def` constant name. If `None`, defaults to
    /// `call_method_def_{type}_{method}` (methods) or `call_method_def_{fn}` (standalone).
    pub call_method_def_ident: Option<syn::Ident>,
    /// When `true`, uses `checked_into_sexp_*` for lossy return types (`i64`, `u64`,
    /// `isize`, `usize` and their `Vec` variants) instead of regular `IntoR::into_sexp`.
    /// Set by `#[miniextendr(strict)]`.
    pub strict: bool,
    /// How `Err` values become condition parts. Set by `#[miniextendr(serde_error)]`.
    pub err_parts: ErrPartsMode,
    /// Parameter names with `#[miniextendr(match_arg, several_ok)]` — use
    /// `match_arg_vec_from_sexp` (instead of `TryFromSexp`) for the `Vec<T>` conversion
    /// so each element is validated against the enum's `MatchArg::CHOICES`.
    ///
    /// Scalar `match_arg` doesn't need entries here because R-side `match.arg()`
    /// already narrowed the SEXP to a valid choice; the default `TryFromSexp for Enum`
    /// (generated by `#[derive(MatchArg)]`) decodes it.
    pub match_arg_several_ok_params: Vec<String>,
    /// When `true`, preserve original parameter names from `inputs` in the C wrapper
    /// signature instead of renaming to `arg_0`, `arg_1`, ... The fn path preserves
    /// user identifiers for rustdoc visibility; impl method path uses `arg_N` for safety.
    pub preserve_param_names: bool,
    /// Visibility of the generated `extern "C-unwind"` wrapper function.
    /// Default: [`syn::Visibility::Inherited`] (no visibility keyword).
    /// Standalone `#[miniextendr]` fns forward the user's visibility (`pub`, `pub(crate)`, etc.).
    pub vis: syn::Visibility,
    /// Generic parameters of the wrapped function, emitted on the C wrapper signature
    /// as `fn #c_ident #generics(...)`. Default: empty (no generics).
    pub generics: syn::Generics,
    /// When `true`, the original Rust fn is already an `extern "C-unwind"` symbol (user-written).
    /// Skip generating the wrapper body but still emit the `R_CallMethodDef` for registration.
    /// The `numArgs` count excludes the synthetic `__miniextendr_call` SEXP parameter since
    /// the user-written fn doesn't have it.
    pub skip_wrapper: bool,
}

impl CWrapperContext {
    /// Creates a new [`CWrapperContextBuilder`] with the given function and C wrapper identifiers.
    ///
    /// All other fields start at their defaults (empty/false/None). Use the builder methods
    /// to configure the context, then call [`CWrapperContextBuilder::build`] to finalize.
    pub fn builder(fn_ident: syn::Ident, c_ident: syn::Ident) -> CWrapperContextBuilder {
        CWrapperContextBuilder {
            fn_ident,
            c_ident,
            r_wrapper_const: None,
            inputs: syn::punctuated::Punctuated::new(),
            output: syn::ReturnType::Default,
            pre_call: Vec::new(),
            call_expr: None,
            thread_strategy: None,
            return_handling: None,
            coerce_all: false,
            coerce_params: Vec::new(),
            check_interrupt: false,
            rng: false,
            cfg_attrs: Vec::new(),
            type_context: None,
            has_self: false,
            call_method_def_ident: None,
            strict: false,
            err_parts: ErrPartsMode::Probe,
            match_arg_several_ok_params: Vec::new(),
            preserve_param_names: false,
            vis: syn::Visibility::Inherited,
            generics: syn::Generics::default(),
            skip_wrapper: false,
        }
    }

    /// Generates the complete output for this wrapper: an `extern "C-unwind"` function
    /// and an `R_CallMethodDef` constant, both decorated with `#[cfg(...)]` attributes
    /// if present.
    ///
    /// When `skip_wrapper` is set (for user-written `extern "C-unwind"` fns), only the
    /// `R_CallMethodDef` is emitted — the fn body itself is already the C symbol.
    ///
    /// Dispatches to [`generate_main_thread_wrapper`](Self::generate_main_thread_wrapper) or
    /// [`generate_worker_thread_wrapper`](Self::generate_worker_thread_wrapper) based on
    /// [`thread_strategy`](Self::thread_strategy).
    pub fn generate(&self) -> TokenStream {
        let call_method_def = self.generate_call_method_def();

        let cfg_attrs = &self.cfg_attrs;

        if self.skip_wrapper {
            // User-written extern "C-unwind" fn — only emit the registration entry
            quote! {
                #(#cfg_attrs)*
                #call_method_def
            }
        } else {
            let c_wrapper = match self.thread_strategy {
                ThreadStrategy::MainThread => self.generate_main_thread_wrapper(),
                ThreadStrategy::WorkerThread => self.generate_worker_thread_wrapper(),
            };

            quote! {
                #(#cfg_attrs)*
                #c_wrapper

                #(#cfg_attrs)*
                #call_method_def
            }
        }
    }

    /// Builds the C wrapper's parameter list from the Rust function signature.
    ///
    /// Returns a tuple of:
    /// - `c_params`: `SEXP` parameter declarations for the C wrapper signature. Always
    ///   starts with `__miniextendr_call` (the R call object for error context), followed
    ///   by `self_sexp` for instance methods, then `arg_0`, `arg_1`, ... for each input.
    /// - `rust_args`: The original Rust parameter identifiers (used in the call expression).
    /// - `sexp_idents`: The generated `arg_N` identifiers (used in SEXP-to-Rust conversions).
    fn build_c_params(&self) -> (Vec<TokenStream>, Vec<syn::Ident>, Vec<syn::Ident>) {
        let mut c_params: Vec<TokenStream> = Vec::new();
        let mut rust_args: Vec<syn::Ident> = Vec::new();
        let mut sexp_idents: Vec<syn::Ident> = Vec::new();

        // First param is always __miniextendr_call for error context
        c_params.push(quote!(__miniextendr_call: ::miniextendr_api::SEXP));

        // For instance methods, add self_sexp parameter
        if self.has_self {
            c_params.push(quote!(self_sexp: ::miniextendr_api::SEXP));
        }

        // Add regular parameters
        for (idx, arg) in self.inputs.iter().enumerate() {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                let ident = &pat_ident.ident;
                // When preserve_param_names is set, use the original parameter name
                // (visible in rustdoc). Otherwise use arg_N for predictable mangling.
                let param_ident = if self.preserve_param_names {
                    ident.clone()
                } else {
                    format_ident!("arg_{}", idx)
                };

                c_params.push(quote!(#param_ident: ::miniextendr_api::SEXP));
                rust_args.push(ident.clone());
                sexp_idents.push(param_ident);
            }
        }

        (c_params, rust_args, sexp_idents)
    }

    /// Generates `TryFromSexp` conversion statements for each parameter.
    ///
    /// Each statement converts an `arg_N: SEXP` into the corresponding Rust type
    /// declared in the original function signature. Respects `strict` and `coerce` settings.
    ///
    /// Used by the main-thread wrapper where all conversions happen inline.
    fn build_conversion_stmts(&self, sexp_idents: &[syn::Ident]) -> Vec<TokenStream> {
        let mut builder = crate::RustConversionBuilder::new();
        if self.strict {
            builder = builder.with_strict();
        }
        if self.coerce_all {
            builder = builder.with_coerce_all();
        }
        for param in &self.coerce_params {
            builder = builder.with_coerce_param(param.clone());
        }
        for param in &self.match_arg_several_ok_params {
            builder = builder.with_match_arg_several_ok(param.clone());
        }
        builder.build_conversions(&self.inputs, sexp_idents)
    }

    /// Build conversion statements split for worker thread execution.
    ///
    /// Returns (pre_closure, in_closure) statements:
    /// - pre_closure: Run on main thread, produce owned values to move
    /// - in_closure: Run inside worker closure, create borrows
    fn build_conversion_stmts_split(
        &self,
        sexp_idents: &[syn::Ident],
    ) -> (Vec<TokenStream>, Vec<TokenStream>) {
        let mut builder = crate::RustConversionBuilder::new();
        if self.strict {
            builder = builder.with_strict();
        }
        if self.coerce_all {
            builder = builder.with_coerce_all();
        }
        for param in &self.coerce_params {
            builder = builder.with_coerce_param(param.clone());
        }
        for param in &self.match_arg_several_ok_params {
            builder = builder.with_match_arg_several_ok(param.clone());
        }

        let mut all_pre = Vec::new();
        let mut all_in = Vec::new();

        for (arg, sexp_ident) in self.inputs.iter().zip(sexp_idents.iter()) {
            if let syn::FnArg::Typed(pat_type) = arg {
                let (owned, borrowed) = builder.build_conversion_split(pat_type, sexp_ident);
                all_pre.extend(owned);
                all_in.extend(borrowed);
            }
        }

        (all_pre, all_in)
    }

    /// Emit a debug-only guard against aliasing zero-copy slice arguments (#1104).
    ///
    /// `impl TryFromSexp for &mut [T]` (and `Option<&mut [T]>`) hands out a
    /// mutable slice over R's data pointer without copying, and `&[T]` /
    /// `Option<&[T]>` hand out a shared one. When R binds the same vector to two
    /// such parameters (`f(x, x)`), the wrapper would produce two aliasing slices
    /// over one buffer. That is undefined behavior whenever at least one of the
    /// two borrows is mutable — two mutable slices, or one mutable and one shared
    /// (`&mut [T]` + `&[T]`). Two shared borrows do not conflict and are allowed.
    ///
    /// Since the wrapper knows the parameter shapes, we compare the raw SEXP
    /// identities pairwise before any conversion and panic (converted to an R
    /// error by the surrounding unwind guard) if an offending pair shares a SEXP,
    /// naming both parameters. Comparing SEXP identity (not the data pointer) is
    /// deliberate: two *distinct* empty vectors share R's `0x1` sentinel data
    /// pointer but are different SEXPs, so identity avoids a false positive there.
    ///
    /// `debug_assert!` compiles to nothing in release builds, so this is a
    /// zero-cost debugging aid. `match_arg` + `several_ok` `&mut [T]` params are
    /// excluded: they get their own owned `Vec<T>` storage and never alias R.
    fn build_alias_guard(&self, sexp_idents: &[syn::Ident]) -> TokenStream {
        // Collect (sexp_ident, param_name, borrow_kind) for every slice-family
        // parameter that borrows R's data pointer directly.
        let mut slice_params: Vec<(&syn::Ident, String, SliceBorrow)> = Vec::new();
        for (arg, sexp_ident) in self.inputs.iter().zip(sexp_idents.iter()) {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                let param_name = pat_ident.ident.to_string();
                if self.match_arg_several_ok_params.contains(&param_name) {
                    continue;
                }
                if let Some(kind) = slice_borrow_kind(pt.ty.as_ref()) {
                    slice_params.push((sexp_ident, param_name, kind));
                }
            }
        }

        let mut checks = Vec::new();
        for i in 0..slice_params.len() {
            for j in (i + 1)..slice_params.len() {
                let (id_a, name_a, kind_a) = &slice_params[i];
                let (id_b, name_b, kind_b) = &slice_params[j];
                // Two shared `&[T]` reads over one buffer are sound; only a pair
                // where at least one borrow is mutable is undefined behavior.
                if *kind_a != SliceBorrow::Mut && *kind_b != SliceBorrow::Mut {
                    continue;
                }
                let msg = format!(
                    "aliasing slice arguments: parameters `{name_a}` and `{name_b}` are bound to \
                     the same R object, and at least one borrows it mutably (`&mut [T]`), so they \
                     would produce aliasing slices over one vector (undefined behavior). Pass \
                     distinct vectors."
                );
                checks.push(quote! {
                    ::core::debug_assert!(#id_a != #id_b, #msg);
                });
            }
        }

        quote! { #(#checks)* }
    }

    /// Generates an `extern "C-unwind"` wrapper that runs entirely on the R main thread.
    ///
    /// The wrapper body is enclosed in `with_r_unwind_protect`, which catches both Rust
    /// panics and R longjmps and returns a tagged-condition SEXP on failure (the R-side
    /// wrapper raises a structured condition). When `rng` is enabled, the call is
    /// additionally wrapped in `catch_unwind` so that `PutRNGstate()` runs even on panic.
    fn generate_main_thread_wrapper(&self) -> TokenStream {
        let c_ident = &self.c_ident;
        let vis = &self.vis;
        let generics = &self.generics;
        let (c_params, _, sexp_idents) = self.build_c_params();
        let conversion_stmts = self.build_conversion_stmts(&sexp_idents);
        let alias_guard = self.build_alias_guard(&sexp_idents);
        let pre_call = &self.pre_call;
        let call_expr = &self.call_expr;

        let pre_call_checks = if self.check_interrupt {
            quote! {
                unsafe { ::miniextendr_api::sys::R_CheckUserInterrupt(); }
            }
        } else {
            TokenStream::new()
        };

        let return_handling = self.generate_return_handling(call_expr);

        let doc = self.generate_doc_comment("main thread");
        let source_loc_doc = crate::source_location_doc(self.fn_ident.span());

        // Unwind protection returns tagged condition SEXP on panic; the R-side wrapper raises.
        let unwind_protect_fn = quote! { ::miniextendr_api::unwind_protect::with_r_unwind_protect };

        if self.rng {
            // RNG variant: wrap in catch_unwind so we can call PutRNGstate before error handling.
            // The wrapper always returns a tagged condition SEXP on panic; the R-side wrapper raises.
            let rng_panic_handler = quote! {
                unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                    &::miniextendr_api::unwind_protect::panic_payload_to_string(&*payload),
                    ::miniextendr_api::error_value::kind::PANIC,
                    ::core::option::Option::None,
                    Some(__miniextendr_call),
                ) }
            };
            quote! {
                #[doc = #doc]
                #[doc = #source_loc_doc]
                #[doc = concat!("Generated from source file `", file!(), "`.")]
                #[unsafe(no_mangle)]
                #vis extern "C-unwind" fn #c_ident #generics(#(#c_params),*) -> ::miniextendr_api::SEXP {
                    unsafe { ::miniextendr_api::sys::GetRNGstate(); }
                    let __result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                        #unwind_protect_fn(
                            || {
                                #alias_guard
                                #pre_call_checks
                                #(#pre_call)*
                                #(#conversion_stmts)*
                                #return_handling
                            },
                            Some(__miniextendr_call),
                        )
                    }));
                    // PutRNGstate runs after catch_unwind, before error handling
                    unsafe { ::miniextendr_api::sys::PutRNGstate(); }
                    match __result {
                        Ok(sexp) => sexp,
                        Err(payload) => { #rng_panic_handler },
                    }
                }
            }
        } else {
            // Non-RNG variant: direct call to with_r_unwind_protect
            quote! {
                #[doc = #doc]
                #[doc = #source_loc_doc]
                #[doc = concat!("Generated from source file `", file!(), "`.")]
                #[unsafe(no_mangle)]
                #vis extern "C-unwind" fn #c_ident #generics(#(#c_params),*) -> ::miniextendr_api::SEXP {
                    #unwind_protect_fn(
                        || {
                            #alias_guard
                            #pre_call_checks
                            #(#pre_call)*
                            #(#conversion_stmts)*
                            #return_handling
                        },
                        Some(__miniextendr_call),
                    )
                }
            }
        }
    }

    /// Generates an `extern "C-unwind"` wrapper that dispatches to the worker thread.
    ///
    /// Structure:
    /// 1. `GetRNGstate()` (if `rng` enabled)
    /// 2. `catch_unwind` around the entire body
    /// 3. Pre-closure conversions on the main thread (produces owned values)
    /// 4. `run_on_worker` (returns `Result<T, String>`) with a
    ///    `move` closure containing in-closure conversions and the call expression
    /// 5. Return conversion back on the main thread via `with_r_unwind_protect`
    /// 6. `PutRNGstate()` (if `rng` enabled)
    /// 7. Panic handling: either tagged error value or `Rf_errorcall`
    fn generate_worker_thread_wrapper(&self) -> TokenStream {
        let c_ident = &self.c_ident;
        let vis = &self.vis;
        let generics = &self.generics;
        let (c_params, _, sexp_idents) = self.build_c_params();
        let (pre_closure_stmts, in_closure_stmts) = self.build_conversion_stmts_split(&sexp_idents);
        let alias_guard = self.build_alias_guard(&sexp_idents);
        let pre_call = &self.pre_call;
        let call_expr = &self.call_expr;

        // Compile-time check: worker dispatch requires the `worker-thread` feature.
        // Check both `worker-thread` (direct) and `worker-default` (implies worker-thread
        // via miniextendr-api, but the user crate may only have the latter in its features).
        let fn_name = self.fn_ident.to_string();
        let feature_msg = format!(
            "`#[miniextendr(worker)]` on `{fn_name}` requires the `worker-thread` cargo feature. \
             Add `worker-thread = [\"miniextendr-api/worker-thread\"]` to your [features] in Cargo.toml."
        );
        let worker_feature_check = quote! {
            #[cfg(not(any(feature = "worker-thread", feature = "worker-default")))]
            compile_error!(#feature_msg);
        };

        let pre_call_checks = if self.check_interrupt {
            quote! {
                unsafe { ::miniextendr_api::sys::R_CheckUserInterrupt(); }
            }
        } else {
            TokenStream::new()
        };

        let (worker_body, return_conversion) = self.generate_worker_return_handling(call_expr);

        let doc = self.generate_doc_comment("worker thread");
        let source_loc_doc = crate::source_location_doc(self.fn_ident.span());

        // RNG state management: GetRNGstate at start, PutRNGstate before returning/error handling
        let (rng_get, rng_put) = if self.rng {
            (
                quote! { unsafe { ::miniextendr_api::sys::GetRNGstate(); } },
                quote! { unsafe { ::miniextendr_api::sys::PutRNGstate(); } },
            )
        } else {
            (TokenStream::new(), TokenStream::new())
        };

        // Panic error handling: return tagged error value (the only mode).
        //
        // #1245 Gap 2 (not fixed here): this block is reused below for the
        // OUTER `Err(payload) => #panic_error_handling` arm — the defensive
        // case where the whole worker-dispatch closure panics directly
        // (rather than `run_on_worker` returning `Err`, which is handled
        // separately just above and already carries a location-folded
        // message per #1245 Gap 1). This site stringifies via
        // `panic_payload_to_string` (no location fold), so a panic reaching
        // it loses its `(at file:line)` suffix. Near-no-op in practice — the
        // panics that actually reach it are framework-internal (e.g.
        // re-entrant `run_on_worker`), not user code. Fixing it properly
        // would need `panic_message_with_location` made `pub`.
        let panic_error_handling = quote! {
            unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                &::miniextendr_api::unwind_protect::panic_payload_to_string(&*payload),
                ::miniextendr_api::error_value::kind::PANIC,
                ::core::option::Option::None,
                Some(__miniextendr_call),
            ) }
        };

        // run_on_worker returns Result; Err → tagged error value.
        quote! {
            #worker_feature_check

            #[doc = #doc]
            #[doc = #source_loc_doc]
            #[doc = concat!("Generated from source file `", file!(), "`.")]
            #[unsafe(no_mangle)]
            #vis extern "C-unwind" fn #c_ident #generics(#(#c_params),*) -> ::miniextendr_api::SEXP {
                #rng_get
                let __miniextendr_panic_result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(move || {
                    #alias_guard
                    #pre_call_checks
                    #(#pre_call)*
                    #(#pre_closure_stmts)*

                    match ::miniextendr_api::worker::run_on_worker(move || {
                        #(#in_closure_stmts)*
                        #worker_body
                    }) {
                        Ok(__miniextendr_result) => {
                            #return_conversion
                        }
                        Err(__panic_msg) => {
                            unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                                &__panic_msg, ::miniextendr_api::error_value::kind::PANIC, ::core::option::Option::None, Some(__miniextendr_call),
                            ) }
                        }
                    }
                }));
                #rng_put
                match __miniextendr_panic_result {
                    Ok(sexp) => sexp,
                    Err(payload) => {
                        #panic_error_handling
                    },
                }
            }
        }
    }

    /// Generates the inline return-handling code for the main-thread wrapper.
    ///
    /// Emits the call expression followed by conversion logic based on [`ReturnHandling`].
    /// For `Option`/`Result` variants, also emits error-path code that returns a
    /// tagged condition SEXP (which the R-side wrapper raises).
    fn generate_return_handling(&self, call_expr: &TokenStream) -> TokenStream {
        let fn_ident = &self.fn_ident;
        let err_parts = self.err_parts.expr();

        match &self.return_handling {
            ReturnHandling::Unit => {
                quote! {
                    #call_expr;
                    ::miniextendr_api::SEXP::nil()
                }
            }
            ReturnHandling::RawSexp => {
                quote! {
                    #call_expr
                }
            }
            ReturnHandling::ExternalPtr => {
                quote! {
                    let __result = #call_expr;
                    ::miniextendr_api::into_r::IntoR::into_sexp(
                        ::miniextendr_api::externalptr::ExternalPtr::new(__result)
                    )
                }
            }
            ReturnHandling::SelfHandle => {
                // In-place builder: the `&mut self` method mutated the value
                // pointed to by `self_sexp`. Evaluate the call only for that
                // side effect, drop the returned `&Self`/`&mut Self` borrow
                // immediately (the trailing `;` ends its lifetime), and hand
                // back the SAME ExternalPtr handle. No clone, no rewrap.
                quote! {
                    let _ = #call_expr;
                    self_sexp
                }
            }
            ReturnHandling::SelfHandleResult => {
                quote! {
                    let __result: ::core::result::Result<(), _> = #call_expr;
                    if let Err(e) = __result {
                        return unsafe { ::miniextendr_api::error_value::result_err_condition_value(
                            #err_parts, Some(__miniextendr_call),
                        ) };
                    }
                    self_sexp
                }
            }
            ReturnHandling::SelfHandleOption => {
                let error_msg = format!("`{}()` returned no value", fn_ident);
                quote! {
                    let __result: ::core::option::Option<()> = #call_expr;
                    if __result.is_none() {
                        return unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                            #error_msg, ::miniextendr_api::error_value::kind::NONE_ERR, ::core::option::Option::None, Some(__miniextendr_call),
                        ) };
                    }
                    self_sexp
                }
            }
            ReturnHandling::IntoR => {
                let result_ident = format_ident!("__result");
                let conversion = self.sexp_conversion_expr(&result_ident, true);
                quote! {
                    let #result_ident = #call_expr;
                    #conversion
                }
            }
            ReturnHandling::OptionUnit => {
                let error_msg = format!("`{}()` returned no value", fn_ident);
                quote! {
                    let __result = #call_expr;
                    if __result.is_none() {
                        return unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                            #error_msg, ::miniextendr_api::error_value::kind::NONE_ERR, ::core::option::Option::None, Some(__miniextendr_call),
                        ) };
                    }
                    ::miniextendr_api::SEXP::nil()
                }
            }
            ReturnHandling::OptionSexp => {
                let error_msg = format!("`{}()` returned no value", fn_ident);
                quote! {
                    let __result = #call_expr;
                    match __result {
                        Some(v) => v,
                        None => return unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                            #error_msg, ::miniextendr_api::error_value::kind::NONE_ERR, ::core::option::Option::None, Some(__miniextendr_call),
                        ) },
                    }
                }
            }
            ReturnHandling::OptionIntoR => {
                // For Option<T> where Option<T>: IntoR (e.g. Option<&T>, Option<Vec<T>>).
                // Call into_sexp on the whole Option — IntoR impl handles None → NULL/NA.
                // result_ident holds the full Option<T>, so the strict lookup must
                // see the full type (checked_option_* helpers).
                let result_ident = format_ident!("__result");
                let conversion = self.sexp_conversion_expr(&result_ident, false);
                quote! {
                    let #result_ident = #call_expr;
                    #conversion
                }
            }
            ReturnHandling::OptionIntoRUnwrap => {
                // For Option<T> where T: IntoR but Option<T>: IntoR is not available.
                // Unwraps first (raises error on None), then converts T via IntoR.
                let error_msg = format!("`{}()` returned no value", fn_ident);
                let result_ident = format_ident!("__result");
                let conversion = self.sexp_conversion_expr(&result_ident, true);
                quote! {
                    let __result = #call_expr;
                    let #result_ident = match __result {
                        Some(v) => v,
                        None => return unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                            #error_msg, ::miniextendr_api::error_value::kind::NONE_ERR, ::core::option::Option::None, Some(__miniextendr_call),
                        ) },
                    };
                    #conversion
                }
            }
            // Option<Self>: raise on None like OptionIntoRUnwrap, but wrap Some(Self) in an
            // ExternalPtr like the bare-Self path rather than routing through IntoR.
            ReturnHandling::OptionExternalPtr => {
                let error_msg = format!("`{}()` returned no value", fn_ident);
                quote! {
                    let __result = #call_expr;
                    let __result = match __result {
                        Some(v) => v,
                        None => return unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                            #error_msg, ::miniextendr_api::error_value::kind::NONE_ERR, ::core::option::Option::None, Some(__miniextendr_call),
                        ) },
                    };
                    ::miniextendr_api::into_r::IntoR::into_sexp(
                        ::miniextendr_api::externalptr::ExternalPtr::new(__result)
                    )
                }
            }
            ReturnHandling::ResultUnit => {
                quote! {
                    let __result = #call_expr;
                    if let Err(e) = __result {
                        return unsafe { ::miniextendr_api::error_value::result_err_condition_value(
                            #err_parts, Some(__miniextendr_call),
                        ) };
                    }
                    ::miniextendr_api::SEXP::nil()
                }
            }
            ReturnHandling::ResultSexp => {
                quote! {
                    let __result = #call_expr;
                    match __result {
                        Ok(v) => v,
                        Err(e) => return unsafe { ::miniextendr_api::error_value::result_err_condition_value(
                            #err_parts, Some(__miniextendr_call),
                        ) },
                    }
                }
            }
            ReturnHandling::ResultIntoR => {
                let result_ident = format_ident!("__result");
                let conversion = self.sexp_conversion_expr(&result_ident, true);
                quote! {
                    let __result = #call_expr;
                    let #result_ident = match __result {
                        Ok(v) => v,
                        Err(e) => return unsafe { ::miniextendr_api::error_value::result_err_condition_value(
                            #err_parts, Some(__miniextendr_call),
                        ) },
                    };
                    #conversion
                }
            }
            // Result<T, ()>: unit error is a deliberate sentinel — always return NULL on Err.
            ReturnHandling::ResultNullOnErr => {
                let result_ident = format_ident!("__result");
                let conversion = self.sexp_conversion_expr(&result_ident, true);
                quote! {
                    let __result = #call_expr;
                    match __result {
                        Ok(#result_ident) => #conversion,
                        Err(()) => ::miniextendr_api::SEXP::nil(),
                    }
                }
            }
            // Result<Self, E>: raise on Err like ResultIntoR, but wrap Ok(Self) in an
            // ExternalPtr like the bare-Self path rather than routing through IntoR.
            ReturnHandling::ResultExternalPtr => {
                quote! {
                    let __result = #call_expr;
                    let __result = match __result {
                        Ok(v) => v,
                        Err(e) => return unsafe { ::miniextendr_api::error_value::result_err_condition_value(
                            #err_parts, Some(__miniextendr_call),
                        ) },
                    };
                    ::miniextendr_api::into_r::IntoR::into_sexp(
                        ::miniextendr_api::externalptr::ExternalPtr::new(__result)
                    )
                }
            }
            ReturnHandling::AsListOf => {
                quote! {
                    let __result = #call_expr;
                    ::miniextendr_api::into_r::IntoR::into_sexp(
                        ::miniextendr_api::convert::AsList(__result)
                    )
                }
            }
            ReturnHandling::AsExternalPtrOf => {
                quote! {
                    let __result = #call_expr;
                    ::miniextendr_api::into_r::IntoR::into_sexp(
                        ::miniextendr_api::convert::AsExternalPtr(__result)
                    )
                }
            }
            ReturnHandling::AsNativeOf => {
                quote! {
                    let __result = #call_expr;
                    ::miniextendr_api::into_r::IntoR::into_sexp(
                        ::miniextendr_api::convert::AsRNative(__result)
                    )
                }
            }
        }
    }

    /// Generates return-handling code split between worker and main threads.
    ///
    /// Returns `(worker_body, return_conversion)`:
    /// - `worker_body`: Runs inside the `run_on_worker` closure. Contains just the call
    ///   expression (the worker returns the raw `Option`/`Result` for the main thread
    ///   to inspect).
    /// - `return_conversion`: Runs back on the main thread after the worker returns.
    ///   Converts the Rust value to SEXP (via `with_r_unwind_protect`). For `Option`
    ///   and `Result` variants, error checking happens here and produces a tagged
    ///   condition SEXP that the R-side wrapper raises.
    fn generate_worker_return_handling(
        &self,
        call_expr: &TokenStream,
    ) -> (TokenStream, TokenStream) {
        let fn_ident = &self.fn_ident;
        let err_parts = self.err_parts.expr();

        match &self.return_handling {
            ReturnHandling::Unit => {
                let worker = quote! {
                    #call_expr;
                };
                let convert = quote! {
                    ::miniextendr_api::SEXP::nil()
                };
                (worker, convert)
            }
            ReturnHandling::RawSexp => {
                // Raw SEXP can't use worker thread - this shouldn't happen
                // but handle it gracefully
                let worker = quote! {
                    #call_expr
                };
                let convert = quote! {
                    __miniextendr_result
                };
                (worker, convert)
            }
            ReturnHandling::ExternalPtr => {
                let worker = quote! {
                    #call_expr
                };
                let unwind_fn = self.worker_conversion_unwind_fn();
                let convert = quote! {
                    #unwind_fn(
                        || ::miniextendr_api::into_r::IntoR::into_sexp(
                            ::miniextendr_api::externalptr::ExternalPtr::new(__miniextendr_result)
                        ),
                        None,
                    )
                };
                (worker, convert)
            }
            ReturnHandling::IntoR => {
                let worker = quote! {
                    #call_expr
                };
                let result_ident = format_ident!("__miniextendr_result");
                let conversion = self.sexp_conversion_expr(&result_ident, true);
                let unwind_fn = self.worker_conversion_unwind_fn();
                let convert = quote! {
                    #unwind_fn(
                        || #conversion,
                        None,
                    )
                };
                (worker, convert)
            }
            ReturnHandling::OptionUnit => {
                let error_msg = format!("`{}()` returned no value", fn_ident);
                // Return the Option from worker, check on main thread.
                let worker = quote! { #call_expr };
                let convert = quote! {
                    if __miniextendr_result.is_none() {
                        unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                            #error_msg, ::miniextendr_api::error_value::kind::NONE_ERR, ::core::option::Option::None, Some(__miniextendr_call),
                        ) }
                    } else {
                        ::miniextendr_api::SEXP::nil()
                    }
                };
                (worker, convert)
            }
            ReturnHandling::OptionSexp => {
                let error_msg = format!("`{}()` returned no value", fn_ident);
                let worker = quote! { #call_expr };
                let convert = quote! {
                    match __miniextendr_result {
                        Some(v) => v,
                        None => unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                            #error_msg, ::miniextendr_api::error_value::kind::NONE_ERR, ::core::option::Option::None, Some(__miniextendr_call),
                        ) },
                    }
                };
                (worker, convert)
            }
            ReturnHandling::OptionIntoR => {
                // For Option<T> where Option<T>: IntoR, call into_sexp on the whole Option.
                // The worker returns the raw Option<T>; the main thread converts via IntoR.
                // None maps to whatever IntoR for Option<T> returns (NULL/NA) — not an error.
                // result_ident holds the full Option<T>, so the strict lookup must
                // see the full type (checked_option_* helpers).
                let worker = quote! { #call_expr };
                let result_ident = format_ident!("__miniextendr_result");
                let conversion = self.sexp_conversion_expr(&result_ident, false);
                let unwind_fn = self.worker_conversion_unwind_fn();
                let convert = quote! {
                    {
                        let #result_ident = __miniextendr_result;
                        #unwind_fn(|| #conversion, None)
                    }
                };
                (worker, convert)
            }
            ReturnHandling::OptionIntoRUnwrap => {
                // For Option<T> where T: IntoR but Option<T>: IntoR is not available.
                // Unwraps first (raises error on None), then converts T via IntoR.
                let error_msg = format!("`{}()` returned no value", fn_ident);
                // Return the Option from worker, check on main thread.
                let worker = quote! { #call_expr };
                let result_ident = format_ident!("__miniextendr_result");
                let conversion = self.sexp_conversion_expr(&result_ident, true);
                let unwind_fn = self.worker_conversion_unwind_fn();
                let convert = quote! {
                    match __miniextendr_result {
                        Some(#result_ident) => #unwind_fn(|| #conversion, None),
                        None => unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                            #error_msg, ::miniextendr_api::error_value::kind::NONE_ERR, ::core::option::Option::None, Some(__miniextendr_call),
                        ) },
                    }
                };
                (worker, convert)
            }
            // Option<Self>: raise on None like OptionIntoRUnwrap, but wrap Some(Self) in an
            // ExternalPtr like the bare-Self path rather than routing through IntoR.
            ReturnHandling::OptionExternalPtr => {
                let error_msg = format!("`{}()` returned no value", fn_ident);
                let worker = quote! { #call_expr };
                let unwind_fn = self.worker_conversion_unwind_fn();
                let convert = quote! {
                    match __miniextendr_result {
                        Some(v) => #unwind_fn(
                            || ::miniextendr_api::into_r::IntoR::into_sexp(
                                ::miniextendr_api::externalptr::ExternalPtr::new(v)
                            ),
                            None,
                        ),
                        None => unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                            #error_msg, ::miniextendr_api::error_value::kind::NONE_ERR, ::core::option::Option::None, Some(__miniextendr_call),
                        ) },
                    }
                };
                (worker, convert)
            }
            ReturnHandling::ResultUnit => {
                let worker = quote! { #call_expr };
                let convert = quote! {
                    match __miniextendr_result {
                        Ok(()) => ::miniextendr_api::SEXP::nil(),
                        Err(e) => unsafe { ::miniextendr_api::error_value::result_err_condition_value(
                            #err_parts, Some(__miniextendr_call),
                        ) },
                    }
                };
                (worker, convert)
            }
            ReturnHandling::ResultSexp => {
                let worker = quote! { #call_expr };
                let convert = quote! {
                    match __miniextendr_result {
                        Ok(v) => v,
                        Err(e) => unsafe { ::miniextendr_api::error_value::result_err_condition_value(
                            #err_parts, Some(__miniextendr_call),
                        ) },
                    }
                };
                (worker, convert)
            }
            ReturnHandling::ResultIntoR => {
                let worker = quote! { #call_expr };
                let result_ident = format_ident!("__miniextendr_result");
                let conversion = self.sexp_conversion_expr(&result_ident, true);
                let unwind_fn = self.worker_conversion_unwind_fn();
                let convert = quote! {
                    match __miniextendr_result {
                        Ok(#result_ident) => #unwind_fn(
                            || #conversion,
                            None,
                        ),
                        Err(e) => unsafe { ::miniextendr_api::error_value::result_err_condition_value(
                            #err_parts, Some(__miniextendr_call),
                        ) },
                    }
                };
                (worker, convert)
            }
            // Result<T, ()>: unit error is a deliberate sentinel — always map to NULL.
            // Convert via NullOnErr so IntoR returns R NULL on Err.
            ReturnHandling::ResultNullOnErr => {
                let result_ident = format_ident!("__miniextendr_result");
                let unwind_fn = self.worker_conversion_unwind_fn();
                let worker = quote! { #call_expr };
                let conversion = self.sexp_conversion_expr(&result_ident, true);
                let convert = quote! {
                    match __miniextendr_result {
                        Ok(#result_ident) => #unwind_fn(|| #conversion, None),
                        Err(()) => ::miniextendr_api::SEXP::nil(),
                    }
                };
                (worker, convert)
            }
            // Result<Self, E>: raise on Err like ResultIntoR, but wrap Ok(Self) in an
            // ExternalPtr like the bare-Self path rather than routing through IntoR.
            ReturnHandling::ResultExternalPtr => {
                let worker = quote! { #call_expr };
                let unwind_fn = self.worker_conversion_unwind_fn();
                let convert = quote! {
                    match __miniextendr_result {
                        Ok(v) => #unwind_fn(
                            || ::miniextendr_api::into_r::IntoR::into_sexp(
                                ::miniextendr_api::externalptr::ExternalPtr::new(v)
                            ),
                            None,
                        ),
                        Err(e) => unsafe { ::miniextendr_api::error_value::result_err_condition_value(
                            #err_parts, Some(__miniextendr_call),
                        ) },
                    }
                };
                (worker, convert)
            }
            ReturnHandling::AsListOf => {
                let worker = quote! { #call_expr };
                let unwind_fn = self.worker_conversion_unwind_fn();
                let convert = quote! {
                    #unwind_fn(
                        || ::miniextendr_api::into_r::IntoR::into_sexp(
                            ::miniextendr_api::convert::AsList(__miniextendr_result)
                        ),
                        None,
                    )
                };
                (worker, convert)
            }
            ReturnHandling::AsExternalPtrOf => {
                let worker = quote! { #call_expr };
                let unwind_fn = self.worker_conversion_unwind_fn();
                let convert = quote! {
                    #unwind_fn(
                        || ::miniextendr_api::into_r::IntoR::into_sexp(
                            ::miniextendr_api::convert::AsExternalPtr(__miniextendr_result)
                        ),
                        None,
                    )
                };
                (worker, convert)
            }
            ReturnHandling::AsNativeOf => {
                let worker = quote! { #call_expr };
                let unwind_fn = self.worker_conversion_unwind_fn();
                let convert = quote! {
                    #unwind_fn(
                        || ::miniextendr_api::into_r::IntoR::into_sexp(
                            ::miniextendr_api::convert::AsRNative(__miniextendr_result)
                        ),
                        None,
                    )
                };
                (worker, convert)
            }
            ReturnHandling::SelfHandle
            | ReturnHandling::SelfHandleResult
            | ReturnHandling::SelfHandleOption => {
                // The self-handle strategies are only assigned to instance
                // methods, which always run on the main thread (the receiver
                // borrow / moved value can't cross to the worker). They never
                // reach the worker return-handling path.
                unreachable!(
                    "ReturnHandling::SelfHandle* is instance-only and always uses the main thread"
                )
            }
        }
    }

    /// Returns the unwind protection function for worker-thread conversion steps.
    /// Always returns tagged condition SEXP on conversion panics; the R-side wrapper raises.
    fn worker_conversion_unwind_fn(&self) -> TokenStream {
        quote! { ::miniextendr_api::unwind_protect::with_r_unwind_protect }
    }

    /// Returns the SEXP conversion expression for `result_ident`, using strict
    /// checked conversion if strict mode is on and the inner return type is lossy,
    /// otherwise falling back to `IntoR::into_sexp()`.
    ///
    /// `result_holds_unwrapped` says what `result_ident` is bound to at runtime:
    /// `true` when the arm already unwrapped the declared `Option<T>` /
    /// `Result<T, E>` wrapper (so the strict lookup must see the inner `T`),
    /// `false` when `result_ident` holds the full declared return type (so
    /// `Option<lossy>` must resolve to the `checked_option_*` helpers — passing
    /// the stripped inner type there emits a scalar helper call on an `Option`
    /// value, which does not compile; caught by the strict-default CI leg).
    fn sexp_conversion_expr(
        &self,
        result_ident: &syn::Ident,
        result_holds_unwrapped: bool,
    ) -> TokenStream {
        if self.strict {
            // Extract the type `result_ident` actually holds from the output type
            let inner_ty = match &self.output {
                syn::ReturnType::Type(_, ty) => {
                    let ty = ty.as_ref();
                    // Strip an Option<T> / Result<T, E> wrapper only when the
                    // calling arm bound the unwrapped value.
                    if result_holds_unwrapped
                        && let syn::Type::Path(p) = ty
                        && let Some(seg) = p.path.segments.last()
                    {
                        let name = seg.ident.to_string();
                        if (name == "Option" || name == "Result")
                            && let Some(inner) = first_type_argument(seg)
                        {
                            Some(inner)
                        } else {
                            Some(ty)
                        }
                    } else {
                        Some(ty)
                    }
                }
                syn::ReturnType::Default => None,
            };

            if let Some(inner_ty) = inner_ty.and_then(|ty| {
                crate::return_type_analysis::strict_conversion_for_type(ty, result_ident)
            }) {
                return inner_ty;
            }
        }

        quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#result_ident) }
    }

    /// Generates the `R_CallMethodDef` constant for R's `.Call` interface registration.
    ///
    /// The constant contains the C symbol name, a `DL_FUNC` pointer to the wrapper
    /// (obtained via `transmute`), and the argument count. R uses this at package load
    /// time (via `R_registerRoutines`) to register the native routine.
    fn generate_call_method_def(&self) -> TokenStream {
        let fn_ident = &self.fn_ident;
        let c_ident = &self.c_ident;
        // When skip_wrapper is set, the user-written fn doesn't have the synthetic
        // __miniextendr_call SEXP param — use the actual input count. Otherwise
        // use build_c_params() which includes __miniextendr_call + self_sexp.
        let num_args = if self.skip_wrapper {
            self.inputs
                .iter()
                .filter(|arg| matches!(arg, syn::FnArg::Typed(_)))
                .count()
        } else {
            let (c_params, _, _) = self.build_c_params();
            c_params.len()
        };
        let num_args_lit = syn::LitInt::new(&num_args.to_string(), proc_macro2::Span::call_site());

        let c_ident_name = syn::LitCStr::new(
            std::ffi::CString::new(c_ident.to_string())
                .expect("valid C string")
                .as_c_str(),
            c_ident.span(),
        );

        // Use custom call_method_def_ident if set, otherwise use default naming
        let call_method_def_ident = self.call_method_def_ident.clone().unwrap_or_else(|| {
            if let Some(ref type_ident) = self.type_context {
                format_ident!("call_method_def_{}_{}", type_ident, fn_ident)
            } else {
                format_ident!("call_method_def_{}", fn_ident)
            }
        });

        // Build func_ptr_def for transmute
        let func_ptr_def: Vec<syn::Type> = (0..num_args)
            .map(|_| syn::parse_quote!(::miniextendr_api::SEXP))
            .collect();

        let item_label = if let Some(ref type_ident) = self.type_context {
            format!("`{}::{}`", type_ident, fn_ident)
        } else {
            format!("`{}`", fn_ident)
        };
        let doc = format!(
            "R call method definition for {} (C wrapper: [`{}`]).",
            item_label, c_ident
        );
        let doc_example = format!(
            "Value: `R_CallMethodDef {{ name: \"{}\", numArgs: {}, fun: <DL_FUNC> }}`",
            c_ident, num_args
        );
        let source_loc_doc = crate::source_location_doc(self.fn_ident.span());

        quote! {
            #[doc = #doc]
            #[doc = #doc_example]
            #[doc = #source_loc_doc]
            #[doc = concat!("Generated from source file `", file!(), "`.")]
            #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS), linkme(crate = ::miniextendr_api::linkme))]
            #[allow(non_upper_case_globals)]
            #[allow(non_snake_case)]
            static #call_method_def_ident: ::miniextendr_api::sys::R_CallMethodDef = unsafe {
                ::miniextendr_api::sys::R_CallMethodDef {
                    name: #c_ident_name.as_ptr(),
                    fun: Some(std::mem::transmute::<
                        unsafe extern "C-unwind" fn(#(#func_ptr_def),*) -> ::miniextendr_api::SEXP,
                        unsafe extern "C-unwind" fn() -> *mut ::std::os::raw::c_void
                    >(#c_ident)),
                    numArgs: #num_args_lit,
                }
            };
        }
    }

    /// Generates a rustdoc comment string for the C wrapper function.
    ///
    /// Includes the original function/method name, thread strategy label, and a
    /// cross-reference to the R wrapper constant.
    fn generate_doc_comment(&self, thread_info: &str) -> String {
        if let Some(ref type_ident) = self.type_context {
            format!(
                "C wrapper for [`{}::{}`] ({}). See [`{}`] for R wrapper.",
                type_ident, self.fn_ident, thread_info, self.r_wrapper_const
            )
        } else {
            format!(
                "C wrapper for [`{}`] ({}). See [`{}`] for R wrapper.",
                self.fn_ident, thread_info, self.r_wrapper_const
            )
        }
    }
}

/// Builder for [`CWrapperContext`].
///
/// Created via [`CWrapperContext::builder`]. All fields except `fn_ident` and `c_ident`
/// (provided at construction) default to empty/false/None. Required fields (`call_expr`,
/// `r_wrapper_const`) must be set before calling [`build`](Self::build) or it will panic.
///
/// Optional fields like `thread_strategy` and `return_handling` are auto-detected from
/// the function signature if not explicitly set.
pub struct CWrapperContextBuilder {
    /// Rust function/method identifier (set at construction).
    fn_ident: syn::Ident,
    /// C wrapper function identifier (set at construction).
    c_ident: syn::Ident,
    /// R wrapper constant identifier for doc cross-references. **Required.**
    r_wrapper_const: Option<syn::Ident>,
    /// Function parameters (excluding `self`). Defaults to empty.
    inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    /// Rust return type. Defaults to `()` (no return type annotation).
    output: syn::ReturnType,
    /// Pre-call statements emitted before the call expression. Defaults to empty.
    pre_call: Vec<TokenStream>,
    /// The Rust call expression. **Required.**
    call_expr: Option<TokenStream>,
    /// Thread strategy override. If `None`, defaults to [`ThreadStrategy::MainThread`].
    thread_strategy: Option<ThreadStrategy>,
    /// Return handling override. If `None`, auto-detected from `output` via [`detect_return_handling`].
    return_handling: Option<ReturnHandling>,
    /// Enable coercing conversion for all parameters.
    coerce_all: bool,
    /// Names of individual parameters with coercing conversion enabled.
    coerce_params: Vec<String>,
    /// Emit `R_CheckUserInterrupt()` before the call.
    check_interrupt: bool,
    /// Wrap call in `GetRNGstate()`/`PutRNGstate()`.
    rng: bool,
    /// `#[cfg(...)]` attributes to propagate to generated items.
    cfg_attrs: Vec<syn::Attribute>,
    /// Type identifier for method context (e.g., `MyStruct`). `None` for standalone functions.
    type_context: Option<syn::Ident>,
    /// Whether the original method has a `self` receiver.
    has_self: bool,
    /// Custom `call_method_def` constant name override.
    call_method_def_ident: Option<syn::Ident>,
    /// Enable strict checked conversions for lossy return types.
    strict: bool,
    /// How `Err` values become condition parts.
    err_parts: ErrPartsMode,
    /// Parameter names with `match_arg + several_ok` — forwarded to
    /// `RustConversionBuilder::with_match_arg_several_ok` so each element of the
    /// Vec is decoded via `match_arg_vec_from_sexp` (enum's `MatchArg::CHOICES`).
    match_arg_several_ok_params: Vec<String>,
    /// When `true`, use original parameter names in C wrapper signature (for rustdoc).
    preserve_param_names: bool,
    /// Visibility of the generated `extern "C-unwind"` wrapper.
    vis: syn::Visibility,
    /// Generic parameters for the C wrapper signature.
    generics: syn::Generics,
    /// When `true`, skip wrapper body but still emit `R_CallMethodDef`.
    skip_wrapper: bool,
}

impl CWrapperContextBuilder {
    /// Sets the R wrapper constant identifier (e.g., `R_WRAPPER_my_func`).
    /// **Required** -- [`build`](Self::build) panics if not set.
    pub fn r_wrapper_const(mut self, ident: syn::Ident) -> Self {
        self.r_wrapper_const = Some(ident);
        self
    }

    /// Sets the function parameters (excluding `self` receiver).
    /// Each input becomes a `SEXP` argument in the C wrapper.
    pub fn inputs(
        mut self,
        inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    ) -> Self {
        self.inputs = inputs;
        self
    }

    /// Sets the Rust return type. Used for auto-detecting [`ReturnHandling`]
    /// and for strict-mode type inspection.
    pub fn output(mut self, output: syn::ReturnType) -> Self {
        self.output = output;
        self
    }

    /// Sets pre-call statements emitted before the call expression.
    /// Typically used for self-extraction in instance methods.
    pub fn pre_call(mut self, stmts: Vec<TokenStream>) -> Self {
        self.pre_call = stmts;
        self
    }

    /// Sets the Rust call expression (e.g., `my_func(arg0)` or `self_ref.method(arg0)`).
    /// **Required** -- [`build`](Self::build) panics if not set.
    pub fn call_expr(mut self, expr: TokenStream) -> Self {
        self.call_expr = Some(expr);
        self
    }

    /// Overrides the thread strategy. If not called, defaults to [`ThreadStrategy::MainThread`].
    pub fn thread_strategy(mut self, strategy: ThreadStrategy) -> Self {
        self.thread_strategy = Some(strategy);
        self
    }

    /// Overrides the return handling strategy. If not called, auto-detected from `output`
    /// via [`detect_return_handling`].
    pub fn return_handling(mut self, handling: ReturnHandling) -> Self {
        self.return_handling = Some(handling);
        self
    }

    /// Enables coercing conversion for all parameters via `Rf_coerceVector`.
    pub fn coerce_all(mut self) -> Self {
        self.coerce_all = true;
        self
    }

    /// Enables coercing conversion for a specific named parameter.
    pub fn with_coerce_param(mut self, param_name: String) -> Self {
        self.coerce_params.push(param_name);
        self
    }

    /// Enables `R_CheckUserInterrupt()` before the call expression.
    pub fn check_interrupt(mut self) -> Self {
        self.check_interrupt = true;
        self
    }

    /// Enable RNG state management (GetRNGstate/PutRNGstate).
    pub fn rng(mut self) -> Self {
        self.rng = true;
        self
    }

    /// Sets `#[cfg(...)]` attributes to propagate to the C wrapper and `call_method_def`.
    pub fn cfg_attrs(mut self, attrs: Vec<syn::Attribute>) -> Self {
        self.cfg_attrs = attrs;
        self
    }

    /// Sets the type context for methods (e.g., `MyStruct`). Used in doc comments
    /// and default `call_method_def` naming.
    pub fn type_context(mut self, type_ident: syn::Ident) -> Self {
        self.type_context = Some(type_ident);
        self
    }

    /// Marks this as an instance method with a `self` receiver.
    /// Causes the C wrapper to include a `self_sexp` parameter.
    pub fn has_self(mut self) -> Self {
        self.has_self = true;
        self
    }

    /// Enables strict checked conversions for lossy return types (`i64`, `u64`, `isize`,
    /// `usize` and their `Vec` variants).
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Choose how `Err` values become condition parts (default: the
    /// `RConditionError`/`Debug` probe). See [`ErrPartsMode`].
    pub fn err_parts(mut self, mode: ErrPartsMode) -> Self {
        self.err_parts = mode;
        self
    }

    /// Record a parameter as `match_arg + several_ok`.
    ///
    /// Passed through to `RustConversionBuilder::with_match_arg_several_ok`, which
    /// switches that parameter's conversion from `TryFromSexp` to
    /// `match_arg_vec_from_sexp::<Inner>` so each STRSXP element is validated against
    /// the enum's `MatchArg::CHOICES`.
    pub fn match_arg_several_ok(mut self, param_name: String) -> Self {
        self.match_arg_several_ok_params.push(param_name);
        self
    }

    /// Set a custom call_method_def identifier.
    ///
    /// If not set, the default naming is used:
    /// - With type_context: `call_method_def_{type}_{method}`
    /// - Without: `call_method_def_{method}`
    pub fn call_method_def_ident(mut self, ident: syn::Ident) -> Self {
        self.call_method_def_ident = Some(ident);
        self
    }

    /// Preserve original parameter names in the C wrapper signature.
    ///
    /// When `true`, `build_c_params` uses the original identifier from `inputs` instead
    /// of renaming to `arg_N`. Enables rustdoc to show descriptive parameter names.
    /// Used by the standalone-fn path; impl methods use the default `arg_N` form.
    pub fn preserve_param_names(mut self) -> Self {
        self.preserve_param_names = true;
        self
    }

    /// Set the visibility of the generated `extern "C-unwind"` wrapper.
    ///
    /// Defaults to [`syn::Visibility::Inherited`]. Standalone fns forward the user's
    /// declared visibility (`pub`, `pub(crate)`, etc.).
    pub fn vis(mut self, vis: syn::Visibility) -> Self {
        self.vis = vis;
        self
    }

    /// Set the generic parameters for the C wrapper function signature.
    ///
    /// Defaults to empty generics. Standalone fns with generic parameters
    /// must forward them so the generated wrapper is also generic.
    pub fn generics(mut self, generics: syn::Generics) -> Self {
        self.generics = generics;
        self
    }

    /// Skip generating the wrapper body and only emit the `R_CallMethodDef`.
    ///
    /// Use this when the Rust fn is already `extern "C-unwind"` with `#[no_mangle]` or
    /// `#[unsafe(no_mangle)]` (the user wrote the C symbol directly). The function still
    /// needs to be registered with R via `R_CallMethodDef`.
    ///
    /// When set, `numArgs` is computed from `inputs` directly (no synthetic
    /// `__miniextendr_call` param).
    pub fn skip_wrapper(mut self) -> Self {
        self.skip_wrapper = true;
        self
    }

    /// Consumes the builder and returns a fully configured [`CWrapperContext`].
    ///
    /// If `thread_strategy` was not set, defaults to [`ThreadStrategy::MainThread`].
    /// If `return_handling` was not set, auto-detects from the `output` type via
    /// [`detect_return_handling`].
    ///
    /// # Panics
    ///
    /// Panics if `call_expr` or `r_wrapper_const` was not set.
    pub fn build(self) -> CWrapperContext {
        let call_expr = self
            .call_expr
            .expect("call_expr is required for CWrapperContext");
        let r_wrapper_const = self
            .r_wrapper_const
            .expect("r_wrapper_const is required for CWrapperContext");

        // Detect thread strategy if not explicitly set.
        // Main thread is the default for all methods (safer, simpler execution model).
        let thread_strategy = self.thread_strategy.unwrap_or(ThreadStrategy::MainThread);

        // Detect return handling if not explicitly set
        let return_handling = self
            .return_handling
            .unwrap_or_else(|| detect_return_handling(&self.output));

        CWrapperContext {
            fn_ident: self.fn_ident,
            c_ident: self.c_ident,
            r_wrapper_const,
            inputs: self.inputs,
            output: self.output,
            pre_call: self.pre_call,
            call_expr,
            thread_strategy,
            return_handling,
            coerce_all: self.coerce_all,
            coerce_params: self.coerce_params,
            check_interrupt: self.check_interrupt,
            rng: self.rng,
            cfg_attrs: self.cfg_attrs,
            type_context: self.type_context,
            has_self: self.has_self,
            call_method_def_ident: self.call_method_def_ident,
            strict: self.strict,
            err_parts: self.err_parts,
            match_arg_several_ok_params: self.match_arg_several_ok_params,
            preserve_param_names: self.preserve_param_names,
            vis: self.vis,
            generics: self.generics,
            skip_wrapper: self.skip_wrapper,
        }
    }
}

/// Detects the appropriate [`ReturnHandling`] strategy from a function's return type.
///
/// Inspects the `syn::ReturnType`:
/// - No return type annotation (`Default`) maps to [`ReturnHandling::Unit`].
/// - An explicit type is analyzed by [`detect_return_handling_from_type`].
pub fn detect_return_handling(output: &syn::ReturnType) -> ReturnHandling {
    match output {
        syn::ReturnType::Default => ReturnHandling::Unit,
        syn::ReturnType::Type(_, ty) => detect_return_handling_from_type(ty),
    }
}

/// Detects [`ReturnHandling`] for the standalone-`#[miniextendr]`-fn path.
///
/// Identical to [`detect_return_handling`] except that general `Option<T>` maps to
/// [`ReturnHandling::OptionIntoR`] (call `into_sexp` on the whole Option, matching the
/// historical `analyze_return_type` behavior) rather than [`ReturnHandling::OptionIntoRUnwrap`]
/// (the default that preserves impl-method behavior). Use this when building a
/// [`CWrapperContext`] for a standalone function.
pub fn detect_return_handling_standalone_fn(output: &syn::ReturnType) -> ReturnHandling {
    let handling = detect_return_handling(output);
    // Standalone fns' old path called into_sexp(whole_option), which is OptionIntoR semantics.
    match handling {
        ReturnHandling::OptionIntoRUnwrap => ReturnHandling::OptionIntoR,
        other => other,
    }
}

/// Determines the [`ReturnHandling`] variant for a concrete `syn::Type`.
///
/// Recognition rules:
/// - `()` -> [`Unit`](ReturnHandling::Unit)
/// - `Self` -> [`ExternalPtr`](ReturnHandling::ExternalPtr)
/// - `SEXP` -> [`RawSexp`](ReturnHandling::RawSexp)
/// - `Option<T>` -> recurses into `T` for `OptionUnit`, `OptionSexp`, `OptionExternalPtr`
///   (`T = Self`), or `OptionIntoRUnwrap`
/// - `Result<T, E>` -> recurses into `T` for `ResultUnit`, `ResultSexp`, `ResultExternalPtr`
///   (`T = Self`), or `ResultIntoR`
/// - Anything else -> [`IntoR`](ReturnHandling::IntoR)
///
/// Note: The default for `Option<T>` (non-unit, non-SEXP) is `OptionIntoRUnwrap` (unwrap
/// first, error on `None`), which preserves the historical behavior for impl methods.
/// Use [`ReturnHandling::OptionIntoR`] explicitly when the type has a direct
/// `impl IntoR for Option<T>` (e.g., `Option<&T>`, `Option<Vec<T>>`, `Option<i32>`).
fn detect_return_handling_from_type(ty: &syn::Type) -> ReturnHandling {
    match ty {
        // Unit tuple ()
        syn::Type::Tuple(t) if t.elems.is_empty() => ReturnHandling::Unit,

        // Self - wrap in ExternalPtr
        syn::Type::Path(p)
            if p.path
                .segments
                .last()
                .map(|s| s.ident == "Self")
                .unwrap_or(false) =>
        {
            ReturnHandling::ExternalPtr
        }

        // SEXP - pass through
        syn::Type::Path(p)
            if p.path
                .segments
                .last()
                .map(|s| s.ident == "SEXP")
                .unwrap_or(false) =>
        {
            ReturnHandling::RawSexp
        }

        // Option<T>
        syn::Type::Path(p)
            if p.path
                .segments
                .last()
                .map(|s| s.ident == "Option")
                .unwrap_or(false) =>
        {
            if let Some(inner_ty) = first_type_argument(p.path.segments.last().unwrap()) {
                match inner_ty {
                    syn::Type::Tuple(t) if t.elems.is_empty() => ReturnHandling::OptionUnit,
                    syn::Type::Path(ip)
                        if ip
                            .path
                            .segments
                            .last()
                            .map(|s| s.ident == "SEXP")
                            .unwrap_or(false) =>
                    {
                        ReturnHandling::OptionSexp
                    }
                    // Option<Self>: a lookup-shaped fallible constructor. Wrap
                    // `Some(Self)` in an ExternalPtr like the bare-`Self` path, rather
                    // than routing it through `IntoR` (which `Self` generally lacks).
                    // Symmetric with `Result<Self, E>` -> `ResultExternalPtr` below.
                    syn::Type::Path(ip)
                        if ip
                            .path
                            .segments
                            .last()
                            .map(|s| s.ident == "Self")
                            .unwrap_or(false) =>
                    {
                        ReturnHandling::OptionExternalPtr
                    }
                    _ => ReturnHandling::OptionIntoRUnwrap,
                }
            } else {
                ReturnHandling::OptionIntoRUnwrap
            }
        }

        // Result<T, E>
        syn::Type::Path(p)
            if p.path
                .segments
                .last()
                .map(|s| s.ident == "Result")
                .unwrap_or(false) =>
        {
            let seg = p.path.segments.last().unwrap();
            // Special case: Result<T, ()> — unit error is a deliberate sentinel that maps to
            // R NULL, not a failure.
            let err_is_unit = crate::second_type_argument(seg)
                .is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));
            if err_is_unit {
                return ReturnHandling::ResultNullOnErr;
            }
            if let Some(ok_ty) = first_type_argument(seg) {
                match ok_ty {
                    syn::Type::Tuple(t) if t.elems.is_empty() => ReturnHandling::ResultUnit,
                    syn::Type::Path(ip)
                        if ip
                            .path
                            .segments
                            .last()
                            .map(|s| s.ident == "SEXP")
                            .unwrap_or(false) =>
                    {
                        ReturnHandling::ResultSexp
                    }
                    // Result<Self, E>: a fallible constructor-shaped method. Wrap
                    // `Ok(Self)` in an ExternalPtr like the bare-`Self` path, rather
                    // than routing it through `IntoR` (which `Self` generally lacks).
                    syn::Type::Path(ip)
                        if ip
                            .path
                            .segments
                            .last()
                            .map(|s| s.ident == "Self")
                            .unwrap_or(false) =>
                    {
                        ReturnHandling::ResultExternalPtr
                    }
                    _ => ReturnHandling::ResultIntoR,
                }
            } else {
                ReturnHandling::ResultIntoR
            }
        }

        // Everything else - use IntoR
        _ => ReturnHandling::IntoR,
    }
}

/// Extracts the first generic type argument from a path segment's angle-bracketed arguments.
///
/// For example, given `Option<String>`, returns `Some(&String)`.
/// Returns `None` if the segment has no angle-bracketed arguments or no type arguments.
fn first_type_argument(seg: &syn::PathSegment) -> Option<&syn::Type> {
    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
        for arg in ab.args.iter() {
            if let syn::GenericArgument::Type(ty) = arg {
                return Some(ty);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests;
