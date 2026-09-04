//! miniextendr-api: core runtime for Rust <-> R interop.
//!
//! This crate provides the FFI surface, safety wrappers, and macro re-exports
//! used by most miniextendr users. It is the primary dependency for building
//! Rust-powered R packages and exposing Rust types to R.
//!
//! At a glance:
//! - FFI bindings + checked wrappers for R's C API (`sys`, `r_ffi_checked`).
//! - Conversions between Rust and R types (`IntoR`, `TryFromSexp`, `Coerce`).
//! - ALTREP traits, registration helpers, and iterator-backed ALTREP data types.
//! - Wrapper generation from Rust signatures (`#[miniextendr]`, automatic registration via linkme).
//! - Main-thread unwind protection plus optional worker dispatch (`worker`).
//! - Class system support (S3, S4, S7, R6, env-style impl blocks).
//! - Cross-package trait ABI for type-erased dispatch (`trait_abi`).
//!
//! Most users should depend on this crate directly. For embedding R in
//! standalone binaries or integration tests, see `miniextendr-engine`.
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
//! That's it — `#[miniextendr]` handles everything. Items self-register
//! at link time; `miniextendr_init!` generates the `R_init_*` function
//! that calls `package_init()` to register all routines with R.
//! Wrapper R code is produced from Rust doc comments (roxygen tags are
//! extracted) by loading the freshly linked package library and calling the
//! registry writer. `R/miniextendr-wrappers.R` is regenerated during
//! development installs, kept out of git, and shipped from disk in release
//! tarballs; roxygen2 derives the tracked `NAMESPACE` and `man/*.Rd` files.
//!
//! ## Choosing the right API
//!
//! miniextendr has several places where two or more APIs reach the same
//! goal with different tradeoffs — a stricter / safer / more validated
//! option, and a looser / easier / less protective one. The most common
//! pairs are:
//!
//! | I'm reaching for... | Consider also | Why |
//! |---|---|---|
//! | default `IntoR` for `i64` / `u64` / `isize` / `usize` (silently widens to `REALSXP` on overflow) | `#[miniextendr(strict)]` → [`crate::strict`] helpers (panic on overflow) | strict catches the truncation bugs caused by R having no native 64-bit integer type |
//! | [`Coerce`] (infallible widening) | [`TryCoerce`] (fallible) | the source range can exceed the target type |
//! | `Rf_*_unchecked` FFI | checked variants | unchecked is only safe inside ALTREP callbacks, `with_r_unwind_protect`, or `with_r_thread` — MXL301 lint enforces |
//! | `panic!(msg)` | `miniextendr_api::error!("msg", class = "...")` | typed conditions let R-side `tryCatch` handlers route by class |
//! | raw `_dots: &Dots` | `#[miniextendr(dots = typed_list!(...))]` | validation moves from runtime to macro call site |
//! | `#[derive(AltrepInteger)]` field-based | `#[altrep(manual)]` + handwritten traits | when custom storage or computed-on-access can't fit the derive |
//! | hand-rolled [`TryFromSexp`] + [`IntoR`] | `#[derive(RSerializeNative)]` (serde feature) | serde is ergonomic for nested structs; hand-rolled is zero-overhead and fully controlled |
//!
//! Project-wide defaults are controlled by mutually-exclusive cargo
//! features — see the "Project-wide Defaults" feature table below.
//!
//! ### Default opinion
//!
//! When in doubt, pick the **stricter** path. The framework's default
//! stance is "fail loudly, leave a trail." The looser variants exist for
//! cases where the cost is measured or the looser semantics are correct
//! for your data — they are not the default.
//!
//! ## GC protection and ownership
//!
//! R's garbage collector can reclaim any SEXP that isn't protected. miniextendr
//! provides three complementary protection mechanisms:
//!
//! | Strategy | Module | Lifetime | Release Order | Use Case |
//! |----------|--------|----------|---------------|----------|
//! | **PROTECT stack** | [`gc_protect`] | Within `.Call` | LIFO (stack) | Temporary allocations |
//! | **VECSXP pool** | [`protect_pool`] | Across `.Call`s | Any order | Long-lived R objects |
//! | **R ownership** | [`ExternalPtr`](struct@ExternalPtr) | Until R GCs | R decides | Rust data owned by R |
//!
//! Quick guide:
//!
//! **Temporary allocations during computation** -> [`ProtectScope`]
//! ```ignore
//! unsafe fn compute(x: SEXP) -> SEXP {
//!     let scope = ProtectScope::new();
//!     let temp = scope.protect(Rf_allocVector(REALSXP, 100));
//!     // ... work with temp ...
//!     result.into_raw()
//! } // UNPROTECT(n) called automatically
//! ```
//!
//! **R objects surviving across `.Call`s** -> [`ProtectPool`] or `R_PreserveObject`
//! ```ignore
//! // ProtectPool: O(1) insert/release with generational keys
//! let mut pool = unsafe { ProtectPool::new(16) };
//! let key = unsafe { pool.insert(backing_vec) };
//! // ... use across multiple .Calls ...
//! unsafe { pool.release(key) };
//! ```
//!
//! **Rust data owned by R** -> [`ExternalPtr`](struct@ExternalPtr)
//! ```ignore
//! #[miniextendr]
//! fn create_model() -> ExternalPtr<MyModel> {
//!     ExternalPtr::new(MyModel::new())
//! } // R owns it; Drop runs when R GCs
//! ```
//!
//! Note: ALTREP trait methods receive raw SEXP pointers from R's runtime.
//! These are safe to dereference because R guarantees valid SEXPs in ALTREP callbacks.
//!
//! ## Threading and safety
//!
//! R uses `longjmp` for errors, which can bypass Rust destructors. Generated
//! wrappers run inline on R's main thread inside `R_UnwindProtect` by default.
//! Opt-in `#[miniextendr(worker)]` functions (or crates using
//! `worker-default`) run Rust logic on the worker and marshal R API calls back
//! through `with_r_thread`. Most FFI wrappers are main-thread routed via
//! `#[r_ffi_checked]`. Use unchecked variants only when you have arranged a
//! safe context.
//!
//! The `nonapi` feature exposes legacy controls for R's process-global stack
//! bounds. Disabling that check does **not** make R's API safe off the main
//! thread; package code must keep R API work on the recorded main thread.
//!
//! ## Feature Flags
//!
//! ### Core Features
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `nonapi` | Non-API R symbols (stack controls, mutable `DATAPTR`). May break with R updates. |
//! | `rayon` | Parallel iterators via Rayon. Adds `RParallelIterator`, `RParallelExtend`. |
//! | `connections` | Experimental R connection framework. **Unstable R API.** |
//! | `indicatif` | Progress bars routed through R connections. Requires `nonapi` + `connections`. |
//! | `vctrs` | vctrs class construction (`new_vctr`, `new_rcrd`, `new_list_of`) and `#[derive(Vctrs)]`. |
//! | `worker-thread` | Worker dispatch infrastructure. It does not change proc-macro defaults by itself; without it, worker APIs are inline stubs. |
//!
//! ### Type Conversions (Scalars & Vectors)
//!
//! | Feature | Rust Type | R Type | Notes |
//! |---------|-----------|--------|-------|
//! | `either` | `Either<L, R>` | Tries L then R | Union-like dispatch |
//! | `uuid` | `Uuid`, `Vec<Uuid>` | `character` | UUID ↔ string |
//! | `regex` | `Regex` | `character(1)` | Compiles pattern from R |
//! | `url` | `Url`, `Vec<Url>` | `character` | Validated URLs |
//! | `time` | `OffsetDateTime`, `Date` | `POSIXct`, `Date` | Date/time conversions |
//! | `jiff` | `Timestamp`, `Zoned`, civil types | R date/time classes | IANA timezone-aware conversions |
//! | `ordered-float` | `OrderedFloat<f64>` | `numeric` | NaN-orderable floats |
//! | `num-bigint` | `BigInt`, `BigUint` | `character` | Arbitrary precision via strings |
//! | `rust_decimal` | `Decimal` | `character` | Fixed-point decimals |
//! | `num-complex` | `Complex<f64>` | `complex` | Native R complex support |
//! | `indexmap` | `IndexMap<String, T>` | named `list` | Preserves insertion order |
//! | `bitflags` | `RFlags<T>` | `integer` | Bitflags ↔ integer |
//! | `bitvec` | `RBitVec` | `logical` | Bit vectors ↔ logical |
//! | `tinyvec` | `TinyVec<[T; N]>`, `ArrayVec<[T; N]>` | vectors | Small-vector optimization |
//!
//! ### Matrix & Array Libraries
//!
//! | Feature | Types | Conversions |
//! |---------|-------|-------------|
//! | `ndarray` | `Array1`–`Array6`, `ArrayD`, views | R vectors/matrices ↔ ndarray |
//! | `nalgebra` | `DVector`, `DMatrix` | R vectors/matrices ↔ nalgebra |
//!
//! ### Serialization
//!
//! | Feature | Traits/Modules | Description |
//! |---------|----------------|-------------|
//! | `serde` | `RSerializeNative`, `RDeserializeNative` | Direct Rust ↔ R native serialization |
//! | `serde_json` | `RSerialize`, `RDeserialize` | JSON string serialization (includes `serde`) |
//! | `borsh` | `Borsh<T>` | Binary serialization ↔ raw vectors via Borsh |
//!
//! ### Adapter Traits (Generic Operations)
//!
//! | Feature | Traits | Use Case |
//! |---------|--------|----------|
//! | `num-traits` | `RNum`, `RSigned`, `RFloat` | Generic numeric operations |
//! | `bytes` | `RBuf`, `RBufMut` | Byte buffer operations |
//!
//! ### Text & Data Processing
//!
//! | Feature | Types/Functions | Description |
//! |---------|-----------------|-------------|
//! | `aho-corasick` | `AhoCorasick`, `aho_compile` | Fast multi-pattern string search |
//! | `toml` | `TomlValue`, `toml_from_str` | TOML parsing and serialization |
//! | `tabled` | `table_to_string` | ASCII/Unicode table formatting |
//! | `sha2` | `sha256_str`, `sha512_bytes` | Cryptographic hashing |
//! | `blake3` | `blake3_str`, `blake3_bytes` | BLAKE3 hashing (fast, 32-byte digests) |
//! | `md5` | `md5_str`, `md5_bytes` | MD5 hashing (interop only — broken crypto) |
//! | `globset` | `GlobSet`, `build_globset` | Shell-style glob matching (path-aware) |
//! | `zstd` | `zstd_compress`, `zstd_decompress` | zstd whole-buffer compression |
//!
//! ### Random Number Generation
//!
//! | Feature | Types | Description |
//! |---------|-------|-------------|
//! | `rand` | `RRng`, `RDistributions` | Wraps R's RNG with `rand` traits |
//! | `rand_distr` | Re-exports `rand_distr` | Additional distributions (Normal, Exp, etc.) |
//!
//! ### Binary Data
//!
//! | Feature | Types | Description |
//! |---------|-------|-------------|
//! | `raw_conversions` | `Raw<T>`, `RawSlice<T>` | POD types ↔ raw vectors via bytemuck |
//!
//! ### Project-wide Defaults (mutually exclusive where noted)
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `r6-default` | Default class system: R6 (mutually exclusive with `s7-default`) |
//! | `s7-default` | Default class system: S7 (mutually exclusive with `r6-default`) |
//! | `worker-default` | Default to worker thread dispatch (implies `worker-thread`) |
//! | `strict-default` | Default to strict mode for lossy integer conversions |
//! | `coerce-default` | Default to coerce mode for type conversions |
//! | `fast-default` | Default to fast wrappers and unchecked conversion paths |
//!
//! ### Development / Diagnostics
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `doc-lint` | Warn on roxygen doc comment mismatches (enabled by default) |
//! | `macro-coverage` | Expose macro coverage test module for `cargo expand` auditing |
//! | `growth-debug` | Track and report collection growth events (zero-cost when off) |
//! | `full-integrations` | Every optional dependency integration (maintenance aggregate) |
//! | `full-codegen` | Codegen/runtime selectors using `r6-default` (maintenance aggregate) |
//! | `full-codegen-s7` | Codegen/runtime selectors using `s7-default` (maintenance aggregate) |
//! | `full` | Integrations + `full-codegen` + diagnostics for local checks |
// Re-export linkme for use by generated code (distributed_slice entries)
#[doc(hidden)]
pub use linkme;

// Procedural macros (re-exported from miniextendr-macros)
#[doc(hidden)]
pub use miniextendr_macros::__mx_trait_impl_expand;
#[doc(inline)]
pub use miniextendr_macros::ExternalPtr;
#[doc(inline)]
pub use miniextendr_macros::RNativeType;
#[doc(inline)]
pub use miniextendr_macros::impl_typed_external;
#[doc(inline)]
pub use miniextendr_macros::list;
#[doc(inline)]
pub use miniextendr_macros::miniextendr;
#[doc(inline)]
pub use miniextendr_macros::miniextendr_init;
#[doc(inline)]
pub use miniextendr_macros::r_ffi_checked;
#[doc(inline)]
pub use miniextendr_macros::typed_dataframe;
#[doc(inline)]
pub use miniextendr_macros::typed_list;
// Note: RFactor derive macro is re-exported - it shares the name with the RFactor trait
// but they're in different namespaces (derive macros vs types/traits)
#[cfg(feature = "vctrs")]
#[doc(inline)]
pub use miniextendr_macros::{PreferVctrs, Vctrs};
// Note: MatchArg derive macro is re-exported - it shares the name with the MatchArg trait
// but they're in different namespaces (derive macros vs types/traits), same as RFactor.
// The same applies to the `IntoR` and `TryFromSexp` derive macros, which share names with
// the `IntoR` / `TryFromSexp` traits — `#[derive(IntoR)]` (macro namespace) and the trait
// (type namespace) coexist, exactly like serde's `Serialize`.
#[doc(inline)]
pub use miniextendr_macros::{
    Altrep, AltrepComplex, AltrepInteger, AltrepList, AltrepLogical, AltrepRaw, AltrepReal,
    AltrepString, DataFrameRow, IntoList, IntoR, MatchArg, PreferDataFrame, PreferExternalPtr,
    PreferList, PreferRNativeType, RFactor, TryFromList, TryFromSexp,
};

pub mod altrep;
pub mod altrep_bridge;
pub mod altrep_data;
pub mod altrep_ext;
pub mod altrep_impl;
pub mod altrep_sexp;
pub mod altrep_traits;

// Re-export for backward compatibility - RegisterAltrep was moved from altrep_registration to altrep
#[doc(hidden)]
pub mod altrep_registration {
    pub use crate::altrep::RegisterAltrep;
}
/// Core type vocabulary for R values: `SEXPTYPE`, `R_xlen_t`, `Rcomplex`,
/// `RLogical`, `Rboolean`, `RNativeType`, `cetype_t`.
pub mod sexp_types;

/// The `SEXP` newtype and inherent methods.
pub mod sexp;

/// `SexpExt` — the ergonomic extension trait on `SEXP`.
pub mod sexp_ext;

/// Raw R FFI bindings — `extern "C-unwind"` blocks for `Rf_*` / `R_*` /
/// `INTEGER` / etc., plus the ALTREP method type aliases under
/// [`sys::altrep`]. The escape hatch for code that genuinely needs the raw
/// R C API.
///
/// **Almost no user code should reach into this module.** The framework
/// expects you to use the crate-root re-exports for the type vocabulary
/// ([`SEXP`], [`SexpExt`], [`SEXPTYPE`], [`R_xlen_t`], …) and the
/// higher-level safe wrappers ([`IntoR`], [`TryFromSexp`],
/// [`worker::with_r_thread`], [`unwind_protect::with_r_unwind_protect`],
/// the `#[miniextendr]` proc-macro). `sys::` is only here so those
/// wrappers — and the rare hand-rolled FFI in tests or benchmarks — have
/// something to call.
pub mod sys;

// Crate-root re-exports for the ergonomic API surface.
pub use sexp::{SEXP, SEXPREC};
pub use sexp_ext::SexpExt;
pub use sexp_types::{
    R_CFinalizer_t, R_xlen_t, RLogical, RNativeType, Rboolean, Rbyte, Rcomplex, SEXPTYPE, cetype_t,
};

/// Automatic registration internals.
///
/// Items annotated with `#[miniextendr]` self-register at link time.
/// The C entrypoint calls [`registry::miniextendr_register_routines`] to
/// finalize registration with R. Users don't interact with this module.
pub mod registry;

/// Host-time generator of `wasm_registry.rs` — the WASM-side replacement for
/// linkme. See the module for full rationale.
///
/// Host-only — the writer reads the live linkme distributed slices to format
/// `wasm_registry.rs`, and linkme isn't available on wasm32 anyway.
#[cfg(not(target_arch = "wasm32"))]
pub mod wasm_registry_writer;

// Re-export high-level ALTREP data traits
pub use altrep_data::{
    AltComplexData,
    AltIntegerData,
    AltListData,
    AltLogicalData,
    AltRawData,
    AltRealData,
    AltStringData,
    AltrepDataptr,
    AltrepExtract,
    AltrepLen,
    // Iterator-backed ALTREP types (R-native)
    IterComplexData,
    // Iterator-backed ALTREP types (with Coerce support)
    IterIntCoerceData,
    IterIntData,
    IterIntFromBoolData,
    IterListData,
    IterLogicalData,
    IterRawData,
    IterRealCoerceData,
    IterRealData,
    IterState,
    IterStringData,
    Logical,
    Sortedness,
    // Sparse iterator-backed ALTREP types (compute-on-access)
    SparseIterComplexData,
    SparseIterIntData,
    SparseIterLogicalData,
    SparseIterRawData,
    SparseIterRealData,
    SparseIterState,
    // Streaming ALTREP types (chunk-cached reader closures)
    StreamingIntData,
    StreamingRealData,
    // Windowed iterator-backed ALTREP types
    WindowedIterIntData,
    WindowedIterRealData,
    WindowedIterState,
};
// Re-export RBase enum, AltrepGuard, and AltrepSexp
pub use altrep::RBase;
pub use altrep_sexp::{AltrepSexp, ensure_materialized};
pub use altrep_traits::AltrepGuard;

// ALTREP package name global - set by C entrypoint before ALTREP registration
// This is a pointer to a null-terminated C string provided by C code.
// Default: c"unknown" for safety if not set.
use std::sync::atomic::{AtomicPtr, Ordering};
static ALTREP_PKG_NAME_PTR: AtomicPtr<std::ffi::c_char> =
    AtomicPtr::new(c"unknown".as_ptr().cast_mut());

/// Returns the current ALTREP package name as a C string pointer.
/// This is set by the C entrypoint before ALTREP registration.
#[doc(hidden)]
pub struct AltrepPkgName;

impl AltrepPkgName {
    /// Get the package name pointer.
    #[inline]
    pub fn as_ptr() -> *const std::ffi::c_char {
        ALTREP_PKG_NAME_PTR.load(Ordering::Acquire)
    }
}

/// Opaque handle for ALTREP package name.
/// Use `ALTREP_PKG_NAME.as_ptr()` to get the C string pointer.
#[doc(hidden)]
pub static ALTREP_PKG_NAME: AltrepPkgName = AltrepPkgName;

/// Set the ALTREP package name. Called from C entrypoint.
/// # Safety
/// The provided pointer must point to a valid null-terminated C string
/// that lives for the duration of the R session.
///
/// The strict requirement is narrower: R copies the bytes via `install()`
/// inside each `R_make_alt*_class` call that consults this global (see
/// `RegisterClass` in R's `src/main/altrep.c`), so the pointer only has to
/// remain valid across those calls. We keep the session-lifetime contract
/// because we don't track which registrations are still pending; a string
/// literal passed from C satisfies both.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn miniextendr_set_altrep_pkg_name(name: *const std::ffi::c_char) {
    let name = if name.is_null() {
        c"unknown".as_ptr()
    } else {
        name
    };
    ALTREP_PKG_NAME_PTR.store(name.cast_mut(), Ordering::Release);
}

// DllInfo global — stored during package_init, used by ALTREP class registration.
// R needs DllInfo to associate ALTREP classes with their package for serialization.
// Without it, readRDS in a fresh session can't find the class.
static ALTREP_DLL_INFO: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());

/// Get the stored DllInfo pointer for ALTREP class registration.
#[doc(hidden)]
pub fn altrep_dll_info() -> *mut sys::DllInfo {
    ALTREP_DLL_INFO.load(Ordering::Acquire).cast()
}

/// Store the DllInfo pointer during package init.
#[doc(hidden)]
pub fn set_altrep_dll_info(dll: *mut sys::DllInfo) {
    ALTREP_DLL_INFO.store(dll.cast(), Ordering::Release);
}

// Note: SexpExt is pub(crate), imported directly in modules that need it
pub mod from_r;
pub mod into_r;
pub mod into_r_error;
pub use into_r::{Altrep, IntoR, IntoRAltrep};
/// Container conversions for forwarding newtypes (`#[derive(TryFromSexp)]` /
/// `#[derive(IntoR)]`). See the module docs and issue #844.
pub mod newtype;
pub use into_r_error::IntoRError;
pub use newtype::{FromRNewtype, IntoRNewtype, IntoRVecElement};
pub mod into_r_as;
pub use into_r_as::{IntoRAs, StorageCoerceError};
pub mod pump;
pub mod unwind_protect;
pub mod worker;

// Re-export commonly used worker items at root for convenience
pub use worker::{Sendable, is_r_main_thread, with_r_thread};

// Required exports for generated code and initialization
pub use worker::miniextendr_runtime_init;

// Advanced stack-configuration utilities; not permission for off-main R API use
pub mod thread;

// Collection growth debug instrumentation (diagnostics)
#[cfg(feature = "growth-debug")]
pub mod growth_debug;

/// Track a collection growth (reallocation) event.
///
/// When the `growth-debug` feature is enabled, increments a thread-local counter
/// for the named collection. When disabled, compiles to a no-op.
///
/// # Example
///
/// ```ignore
/// let old_cap = vec.capacity();
/// vec.push(item);
/// if vec.capacity() != old_cap {
///     track_growth!("my_vec");
/// }
/// ```
#[cfg(feature = "growth-debug")]
#[macro_export]
macro_rules! track_growth {
    ($name:expr) => {
        $crate::growth_debug::record_growth($name)
    };
}

/// Track a collection growth (reallocation) event.
///
/// No-op when `growth-debug` feature is disabled.
#[cfg(not(feature = "growth-debug"))]
#[macro_export]
macro_rules! track_growth {
    ($name:expr) => {};
}

/// Print and reset all growth counters.
///
/// When the `growth-debug` feature is enabled, prints all tracked growth events
/// to stderr and resets the counters. When disabled, compiles to a no-op.
#[cfg(feature = "growth-debug")]
#[macro_export]
macro_rules! report_growth {
    () => {
        $crate::growth_debug::report_and_reset()
    };
}

/// Print and reset all growth counters.
///
/// No-op when `growth-debug` feature is disabled.
#[cfg(not(feature = "growth-debug"))]
#[macro_export]
macro_rules! report_growth {
    () => {};
}

/// Parse and evaluate **runtime** R source from Rust.
///
/// `r_str!` is sugar around [`expression::r_eval_str`]. It is the right tool
/// when the R code is genuinely dynamic — built with `format!`, derived from
/// user input, or otherwise not known at compile time. For code you can write
/// literally, prefer [`r!`](crate::r), which gives a cheap compile-time
/// sanity check on the token stream.
///
/// The argument is any expression evaluating to something `AsRef<str>`
/// (`&str`, `String`, `&String`, …). It is parsed with `R_ParseVector` and
/// evaluated with `Rf_eval`, with every intermediate SEXP protected and the
/// parse status checked, so a syntax error becomes an `Err`, never a segfault
/// or silent wrong answer.
///
/// # Forms
///
/// - `r_str!(code)` — evaluate in `R_GlobalEnv`.
/// - `r_str!(code, env = e)` — evaluate in the environment SEXP `e`.
///
/// Both forms evaluate to `Result<SEXP, String>`; the `SEXP` is **unprotected**
/// (protect it before further allocations).
///
/// # Safety
///
/// Expands to an `unsafe` block. Must be reached from (or routed to) the R
/// main thread — the underlying FFI is `#[r_ffi_checked]`, so calls from a
/// worker thread are serialized onto the R thread.
///
/// # Example
///
/// ```ignore
/// let obj = "mtcars";
/// let code = format!("summary({obj})");
/// let summary = r_str!(&code)?;          // in R_GlobalEnv
/// let three = r_str!("1L + 2L")?;
/// let in_env = r_str!("x + 1", env = my_env)?;
/// ```
#[macro_export]
macro_rules! r_str {
    ($code:expr $(,)?) => {
        unsafe {
            $crate::expression::r_eval_str(
                ::core::convert::AsRef::<str>::as_ref(&$code),
                $crate::sys::R_GlobalEnv,
            )
        }
    };
    ($code:expr, env = $env:expr $(,)?) => {
        unsafe {
            $crate::expression::r_eval_str(::core::convert::AsRef::<str>::as_ref(&$code), $env)
        }
    };
}

/// Evaluate R code written as **Rust tokens**, validated at compile time.
///
/// `r!` takes a single R expression as a token stream, `stringify!`s it into a
/// static R source string at build time, and evaluates it via
/// [`expression::r_eval_str`] (the same protect-safe parse + eval path as
/// [`r_str!`](crate::r_str)).
///
/// # What you get today
///
/// Because the argument is a Rust token tree, the Rust front-end already
/// rejects **unbalanced delimiters** (`r!(f(1, 2)` won't compile) and
/// lexically invalid tokens before R ever sees the string — a cheap
/// compile-time guard over the pure-runtime [`r_str!`](crate::r_str). The
/// source is lowered to a `&'static str` (`stringify!`), so there is no
/// `format!` allocation at the call site.
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
/// is tracked as a follow-up — see issue #938 (item 2). Until then `r!` parses
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
///   leading `env: <expr> ;` is consumed as Rust, the rest is R source. (The
///   `;` separator is used instead of a trailing `, env =` because R source is
///   a free token stream — a trailing keyword can't be reliably split off it.)
///
/// Both evaluate to `Result<SEXP, String>`; the `SEXP` is **unprotected**.
///
/// For genuinely dynamic code, use [`r_str!`](crate::r_str) instead.
///
/// # Note on `stringify!` spacing
///
/// `stringify!` normalises whitespace (`a+b` and `a + b` both stringify to
/// `a + b`) but preserves token order and literal contents, which is all R's
/// parser needs. String literals keep their quotes.
///
/// # Safety
///
/// Expands to an `unsafe` block; see [`r_str!`](crate::r_str).
///
/// # Example
///
/// ```ignore
/// let three = r!(1L + 2L)?;
/// let rows = r!(getFromNamespace(".theoph_rows", "dataframeflows")())?;
/// let in_env = r!(env: my_env; x + 1)?;
/// ```
#[doc(inline)]
pub use miniextendr_macros::r;

// `indicatif` progress integration (R console)
#[cfg(feature = "indicatif")]
pub mod progress;
#[cfg(feature = "indicatif")]
pub use indicatif;

// Legacy R-oriented stack sizing (always available; not an off-main R bridge)
#[cfg(windows)]
pub use thread::WINDOWS_R_STACK_SIZE;
pub use thread::{DEFAULT_R_STACK_SIZE, RThreadBuilder};

// Legacy process-global stack controls (requires nonapi; removal tracked in #1352)
#[cfg(feature = "nonapi")]
pub use thread::{StackCheckGuard, scope_with_r, spawn_with_r, with_stack_checking_disabled};

// Panic telemetry hook for structured panic→R-error diagnostics
pub mod panic_telemetry;

// Unified FFI guard for catching panics at Rust-R boundaries
pub mod ffi_guard;
pub use ffi_guard::{GuardMode, guarded_ffi_call, guarded_ffi_call_with_fallback};

// The unified owned data.frame type + conversion trait family
pub mod dataframe;
pub use dataframe::{
    BuiltDataFrame, ColumnarFrame, DataFrame, DataFrameError, FromDataFrame, GroupKey,
    GroupedDataFrame, IntoDataFrame, IntoDataFrameSplit, NamedDataFrameListBuilder, group_rows,
};

// Closure-per-column DataFrame builder (parallel fill with `rayon`, serial
// otherwise). Available regardless of the `rayon` feature (#1055).
pub mod dataframe_builder;
pub use dataframe_builder::RDataFrameBuilder;

// Strict conversion helpers for #[miniextendr(strict)]
pub mod strict;

// Cached R class attribute SEXPs (POSIXct, Date, data.frame, etc.)
pub mod cached_class;

// Tagged condition value transport for the Rust→R boundary
pub mod error_value;

// Error handling helpers (r_warning, r_print!, r_println! macros)
pub mod error;
pub use error::r_warning;

// RNG (random number generation) utilities
pub mod rng;
pub use rng::{RngGuard, with_rng};

// Re-export from_r
pub use from_r::{SexpError, SexpLengthError, SexpNaError, SexpTypeError, TryFromSexp};

// Encoding / locale probing (mainly for debugging). The module is always
// compiled; the symbols that reference non-API locale state from R's `Defn.h`
// (`utf8locale`, `mbcslocale`, `known_to_be_latin1`) are gated inside the
// module behind `#[cfg(feature = "nonapi")]` so a default build never links
// them. Only R-exported symbols may be declared there — hidden ones abort
// dyn.load (see sys::nonapi_encoding).
pub mod encoding;

// Expression evaluation helpers (RSymbol, RCall, REnv)
pub mod expression;
pub use expression::{RCall, REnv, RSymbol, r_eval_str, r_eval_str_global};

// S4 slot access and class checking helpers
pub mod s4_helpers;

// Note: RNativeType is pub(crate), imported directly in modules that need it

pub mod backtrace;

/// Shared truthiness parsing for boolean-ish environment-variable flags.
pub(crate) mod env_flag;

pub mod coerce;
pub use coerce::{Coerce, CoerceError, Coerced, TryCoerce};

/// Traits for R's `as.<class>()` coercion functions.
///
/// This module provides traits for implementing R's generic coercion methods
/// (`as.data.frame()`, `as.list()`, `as.character()`, etc.) for Rust types
/// wrapped in [`ExternalPtr`](struct@ExternalPtr).
///
/// See the [`r_coerce`] module documentation for usage examples.
pub mod r_coerce;
pub use r_coerce::{
    // Core coercion traits (R's `as.<class>()` generics)
    RCoerceCharacter,
    RCoerceComplex,
    RCoerceDataFrame,
    RCoerceDate,
    RCoerceEnvironment,
    // Error type
    RCoerceError,
    RCoerceFactor,
    RCoerceFunction,
    RCoerceInteger,
    RCoerceList,
    RCoerceLogical,
    RCoerceMatrix,
    RCoerceNumeric,
    RCoercePOSIXct,
    RCoerceRaw,
    RCoerceVector,
    // Helpers
    SUPPORTED_AS_GENERICS,
    is_supported_as_generic,
};

pub mod condition;
pub use condition::{AsRError, RCondition};
pub mod convert;
/// Support for R `...` arguments represented as a validated list.
pub mod dots;
pub mod list;
pub mod missing;
pub mod named_vector;
pub mod rcow;
pub mod rvalue;
pub use rvalue::RValue;
pub mod strvec;
pub mod typed_list;
pub use convert::{
    AsDataFrame, AsDataFrameExt, AsDisplay, AsDisplayVec, AsExternalPtr, AsExternalPtrExt,
    AsFromStr, AsFromStrVec, AsList, AsListExt, AsNamedList, AsNamedListExt, AsNamedVector,
    AsNamedVectorExt, AsRNative, AsRNativeExt, Collect, CollectNA, CollectNAInt, CollectStrings,
    ColumnSource,
};
#[cfg(feature = "vctrs")]
pub use convert::{AsVctrs, AsVctrsExt};
pub use into_r::Lazy;
pub use list::{
    IntoList, List, ListAccumulator, ListBuilder, ListMut, NamedList, TryFromList, collect_list,
};
pub use missing::{Missing, is_missing_arg};
pub use named_vector::{AtomicElement, NamedVector};
pub use rcow::{RBorrow, RCow};
pub use strvec::{
    ProtectedStrVec, ProtectedStrVecCowIter, ProtectedStrVecIter, StrVec, StrVecBuilder,
    StrVecCowIter, StrVecIter,
};
pub use typed_list::{
    TypeSpec, TypedEntry, TypedList, TypedListError, TypedListSpec, actual_type_string,
    sexptype_name, validate_list,
};

// External pointer module - Box-like owned pointer wrapping R's EXTPTRSXP
pub mod externalptr;

// Connection framework (unstable R API - use with caution)
#[cfg(feature = "connections")]
pub mod connection;
// txtProgressBar handle — drives utils::txtProgressBar from Rust
#[cfg(feature = "connections")]
pub mod txt_progress_bar;
pub use externalptr::{
    ErasedExternalPtr,
    ExternalPtr,
    ExternalSlice,
    IntoExternalPtr,
    TypedExternal,
    // ALTREP helpers (checked)
    altrep_data1_as,
    // ALTREP helpers (unchecked - for performance-critical callbacks)
    altrep_data1_as_unchecked,
    altrep_data1_mut,
    altrep_data1_mut_unchecked,
    altrep_data2_as,
    altrep_data2_as_unchecked,
};

// TypedExternal implementations for std types
pub mod externalptr_std;

// GC protection toolkit (PROTECT stack RAII wrappers)
pub mod gc_protect;
pub use gc_protect::{
    OwnedProtect, ProtectIndex, ProtectScope, Protected, Protector, ReprotectSlot, Root,
};

// VECSXP pool with generational keys (slotmap-backed)
pub mod protect_pool;
pub use protect_pool::{ProtectKey, ProtectPool};

// Reference-counted GC protection (BTreeMap + VECSXP backing)
pub mod refcount_protect;
pub use refcount_protect::{ArenaGuard, RefCountedArena, ThreadLocalArena};

pub mod allocator;
pub use allocator::RAllocator;

pub mod r_memory;

// region: Trait ABI Support
//
// Cross-package trait dispatch using a stable C ABI.
// See `trait_abi` module docs for details.

/// ABI types for cross-package trait dispatch.
///
/// This module defines the stable, C-compatible types used for runtime trait
/// dispatch across R package boundaries.
pub mod abi;

/// C-callable mx_abi functions (mx_wrap, mx_get, mx_query, mx_abi_register).
///
/// These are registered via `R_RegisterCCallable` during package init and
/// loaded by consumer packages via `R_GetCCallable`.
pub mod mx_abi;

/// Package initialization (`miniextendr_init!` support).
///
/// Consolidates all init steps into [`init::package_init`].
pub mod init;

/// Runtime support for trait ABI operations.
///
/// Provides C-callable loading and type conversion helpers for trait ABI support.
pub mod trait_abi;

/// vctrs class construction and trait support.
///
/// Provides helpers for building vctrs-compatible R objects and traits
/// for describing vctrs class metadata from Rust types.
///
/// Enable with `features = ["vctrs"]`.
#[cfg(feature = "vctrs")]
pub mod vctrs;
#[cfg(feature = "vctrs")]
pub use vctrs::{
    IntoVctrs, VctrsBuildError, VctrsClass, VctrsKind, VctrsListOf, VctrsRecord, new_list_of,
    new_rcrd, new_vctr,
};

// Re-export key ABI types at crate root for convenience
pub use abi::{mx_base_vtable, mx_erased, mx_meth, mx_tag};
pub use trait_abi::TraitView;
// endregion

// region: Marker Traits
//
// Marker traits for types derived with proc-macros.
// These enable compile-time identification and blanket implementations.

/// Marker traits for proc-macro derived types.
pub mod markers;
// endregion

// region: Adapter Traits
//
// Built-in adapter traits with blanket implementations for standard library traits.
// These allow any Rust type implementing Debug, Display, Hash, Ord, etc. to be
// exposed to R without boilerplate.

/// Built-in adapter traits for std library traits.
///
/// Provides [`RDebug`], [`RDisplay`], [`RHash`], [`ROrd`], [`RPartialOrd`],
/// [`RError`], [`RFromStr`], [`RClone`], [`RCopy`], [`RDefault`], [`RIterator`],
/// [`RExtend`], and [`RFromIter`] with blanket implementations where possible.
/// See module docs for usage.
///
/// [`RDebug`]: adapter_traits::RDebug
/// [`RDisplay`]: adapter_traits::RDisplay
/// [`RHash`]: adapter_traits::RHash
/// [`ROrd`]: adapter_traits::ROrd
/// [`RPartialOrd`]: adapter_traits::RPartialOrd
/// [`RError`]: adapter_traits::RError
/// [`RFromStr`]: adapter_traits::RFromStr
/// [`RClone`]: adapter_traits::RClone
/// [`RCopy`]: adapter_traits::RCopy
/// [`RDefault`]: adapter_traits::RDefault
/// [`RIterator`]: adapter_traits::RIterator
/// [`RExtend`]: adapter_traits::RExtend
/// [`RFromIter`]: adapter_traits::RFromIter
pub mod adapter_traits;
pub use adapter_traits::{
    RClone, RCopy, RDebug, RDefault, RDisplay, RError, RExtend, RFromIter, RFromStr, RHash,
    RIterator, RMakeIter, ROrd, RPartialOrd, RToVec,
};

/// This is used to ensure the macros of `miniextendr-macros` treat this crate as a "user crate"
/// atleast in the `macro_coverage`
#[doc(hidden)]
extern crate self as miniextendr_api;

#[cfg(feature = "macro-coverage")]
#[doc(hidden)]
pub mod macro_coverage;
// endregion

// region: Optional integrations with external crates (feature-gated)
//
// All optional feature integrations are organized in the `optionals` module.
// Types are re-exported at crate root for backwards compatibility.

/// Optional feature integrations with third-party crates.
///
/// This module contains all feature-gated integrations with external crates.
/// Each submodule is only compiled when its corresponding feature is enabled.
/// See the [module documentation][optionals] for a complete list of available features.
pub mod optionals;

// Re-export optional types at crate root for backwards compatibility
#[cfg(feature = "rayon")]
pub use optionals::parallel;
#[cfg(feature = "rayon")]
pub use optionals::rayon_bridge;
#[cfg(feature = "rayon")]
pub use optionals::{RParallelExtend, RParallelIterator};

#[cfg(feature = "rand")]
pub use optionals::rand;
#[cfg(feature = "rand_distr")]
pub use optionals::rand_distr;
#[cfg(feature = "rand")]
pub use optionals::rand_impl;
#[cfg(feature = "rand")]
pub use optionals::{RDistributionOps, RDistributions, RRng, RRngOps};

#[cfg(feature = "either")]
pub use optionals::either_impl;
#[cfg(feature = "either")]
pub use optionals::{Either, Left, Right};

#[cfg(feature = "ndarray")]
pub use optionals::ndarray_impl;
#[cfg(feature = "ndarray")]
pub use optionals::{
    ArcArray1, ArcArray2, Array0, Array1, Array2, Array3, Array4, Array5, Array6, ArrayD,
    ArrayView0, ArrayView1, ArrayView2, ArrayView3, ArrayView4, ArrayView5, ArrayView6, ArrayViewD,
    ArrayViewMut0, ArrayViewMut1, ArrayViewMut2, ArrayViewMut3, ArrayViewMut4, ArrayViewMut5,
    ArrayViewMut6, ArrayViewMutD, Ix0, Ix1, Ix2, Ix3, Ix4, Ix5, Ix6, IxDyn, RNdArrayOps, RNdIndex,
    RNdSlice, RNdSlice2D, ShapeBuilder,
};

#[cfg(feature = "nalgebra")]
pub use optionals::nalgebra_impl;
#[cfg(feature = "nalgebra")]
pub use optionals::{DMatrix, DVector, RMatrixOps, RVectorOps, SMatrix, SVector};

#[cfg(feature = "num-bigint")]
pub use optionals::num_bigint_impl;
#[cfg(feature = "num-bigint")]
pub use optionals::{BigInt, BigUint, RBigIntBitOps, RBigIntOps, RBigUintBitOps, RBigUintOps};

#[cfg(feature = "rust_decimal")]
pub use optionals::rust_decimal_impl;
#[cfg(feature = "rust_decimal")]
pub use optionals::{Decimal, RDecimalOps};

#[cfg(feature = "ordered-float")]
pub use optionals::ordered_float_impl;
#[cfg(feature = "ordered-float")]
pub use optionals::{OrderedFloat, ROrderedFloatOps};

#[cfg(feature = "num-complex")]
pub use optionals::num_complex_impl;
#[cfg(feature = "num-complex")]
pub use optionals::{Complex, RComplexOps};

#[cfg(feature = "num-traits")]
pub use optionals::num_traits_impl;
#[cfg(feature = "num-traits")]
pub use optionals::{RFloat, RNum, RSigned};

#[cfg(feature = "uuid")]
pub use optionals::uuid_impl;
#[cfg(feature = "uuid")]
pub use optionals::{RUuidOps, Uuid, uuid_helpers};

#[cfg(feature = "regex")]
pub use optionals::regex_impl;
#[cfg(feature = "regex")]
pub use optionals::{CaptureGroups, RCaptureGroups, RRegexOps, Regex};

#[cfg(feature = "url")]
pub use optionals::url_impl;
#[cfg(feature = "url")]
pub use optionals::{RUrlOps, Url, url_helpers};

#[cfg(feature = "aho-corasick")]
pub use optionals::aho_corasick_impl;
#[cfg(feature = "aho-corasick")]
pub use optionals::{
    AhoCorasick, RAhoCorasickOps, aho_compile, aho_count_matches, aho_find_all, aho_find_all_flat,
    aho_find_first, aho_is_match, aho_replace_all,
};

#[cfg(feature = "indexmap")]
pub use optionals::indexmap_impl;
#[cfg(feature = "indexmap")]
pub use optionals::{IndexMap, RIndexMapOps};

#[cfg(feature = "time")]
pub use optionals::time_impl;
#[cfg(feature = "time")]
pub use optionals::{Date, Duration, OffsetDateTime, RDateTimeFormat, RDuration};
#[cfg(feature = "time")]
pub use time;

#[cfg(feature = "jiff")]
pub use jiff;
#[cfg(feature = "jiff")]
pub use optionals::jiff_impl;
#[cfg(feature = "jiff")]
pub use optionals::{
    JiffDate, JiffDateTime, JiffTime, JiffTimestampVec, JiffZonedVec, RDate, RDateTime,
    RSignedDuration, RSpan, RTime, RTimestamp, RZoned, SignedDuration, Span, Timestamp, Zoned,
};

#[cfg(feature = "serde_json")]
pub use optionals::serde_impl;
#[cfg(feature = "toml")]
pub use optionals::toml_impl;
#[cfg(feature = "serde_json")]
pub use optionals::{
    FactorHandling, JsonOptions, JsonValue, NaHandling, RDeserialize, RJsonBridge, RJsonValueOps,
    RSerialize, SpecialFloatHandling, json_from_sexp, json_from_sexp_permissive,
    json_from_sexp_strict, json_from_sexp_with, json_into_sexp,
};
#[cfg(feature = "toml")]
pub use optionals::{RTomlOps, TomlValue, toml_from_str, toml_to_string, toml_to_string_pretty};

#[cfg(feature = "bytes")]
pub use optionals::bytes_impl;
#[cfg(feature = "bytes")]
pub use optionals::{Buf, BufMut, Bytes, BytesMut, RBuf, RBufMut};

#[cfg(feature = "sha2")]
pub use optionals::sha2_impl;
#[cfg(feature = "sha2")]
pub use optionals::{sha256_bytes, sha256_str, sha512_bytes, sha512_str};

#[cfg(feature = "blake3")]
pub use optionals::blake3_impl;
#[cfg(feature = "blake3")]
pub use optionals::{blake3_bytes, blake3_hex, blake3_str};

#[cfg(feature = "md5")]
pub use optionals::md5_impl;
#[cfg(feature = "md5")]
pub use optionals::{md5_bytes, md5_hex, md5_str};

#[cfg(feature = "globset")]
pub use optionals::globset_impl;
#[cfg(feature = "globset")]
pub use optionals::{
    Glob, GlobBuilder, GlobOptions, GlobSet, GlobSetBuilder, build_globset, globset_is_match,
    globset_matches,
};

#[cfg(feature = "zstd")]
pub use optionals::zstd_impl;
#[cfg(feature = "zstd")]
pub use optionals::{zstd_compress, zstd_decompress};

#[cfg(feature = "borsh")]
pub use optionals::borsh_impl;
#[cfg(feature = "borsh")]
pub use optionals::{Borsh, RBorshOps, borsh_from_raw, borsh_to_raw};

#[cfg(feature = "bitflags")]
pub use bitflags;
#[cfg(feature = "bitflags")]
pub use optionals::bitflags_impl;
#[cfg(feature = "bitflags")]
pub use optionals::{Flags, RFlags};

#[cfg(feature = "bitvec")]
pub use optionals::bitvec_impl;
#[cfg(feature = "bitvec")]
pub use optionals::{BitVec, Lsb0, Msb0, RBitVec};

#[cfg(feature = "tabled")]
pub use optionals::tabled_impl;
#[cfg(feature = "tabled")]
pub use optionals::{
    Builder, Table, Tabled, builder_to_string, table_from_vecs, table_to_string,
    table_to_string_opts, table_to_string_styled,
};

#[cfg(feature = "tinyvec")]
pub use optionals::tinyvec_impl;
#[cfg(feature = "tinyvec")]
pub use optionals::{Array, ArrayVec, TinyVec};

#[cfg(feature = "arrow")]
pub use optionals::arrow_impl;
#[cfg(feature = "arrow")]
pub use optionals::{
    ArrayRef, ArrowArray, BooleanArray, DataType, Date32Array, DictionaryArray, Field,
    Float64Array, Int32Array, RecordBatch, Schema, StringArray, StringDictionaryArray,
    TimestampSecondArray, UInt8Array,
};

#[cfg(feature = "datafusion")]
pub use optionals::RSessionContext;
#[cfg(feature = "datafusion")]
pub use optionals::datafusion_impl;

/// N-dimensional R arrays with const generic dimension count.
pub mod rarray;
pub use rarray::{RArray, RArray3D, RMatrix, RVector};

/// Direct R serialization via serde (no JSON intermediate).
///
/// Provides efficient type-preserving conversions between Rust types and native R objects:
/// - [`AsSerialize<T>`][serde::AsSerialize] - Wrapper for returning `Serialize` types from `#[miniextendr]` functions
/// - [`RSerializeNative`][serde::RSerializeNative] - Convert Rust → R (struct → named list)
/// - [`RDeserializeNative`][serde::RDeserializeNative] - Convert R → Rust (named list → struct)
///
/// Enable with `features = ["serde"]`.
///
/// See the [`serde`] module documentation for type mappings and examples.
#[cfg(feature = "serde")]
pub mod serde;
/// Re-export the upstream `serde` crate (aliased to avoid conflict with [`mod serde`]).
///
/// Downstream crates can use `miniextendr_api::serde_crate::{Serialize, Deserialize}`
/// and `#[serde(crate = "miniextendr_api::serde_crate")]` to avoid a direct `serde` dep.
#[cfg(feature = "serde")]
pub use ::serde as serde_crate;

/// Integration with the `bytemuck` crate for POD type conversions.
///
/// Provides explicit, safe conversions between Rust POD (Plain Old Data) types
/// and R raw vectors:
/// - `Raw<T>` - Single POD value (headerless, native layout)
/// - `RawSlice<T>` - Sequence of POD values (headerless)
/// - `RawTagged<T>` / `RawSliceTagged<T>` - With header metadata
///
/// Enable with `features = ["raw_conversions"]`.
///
/// ```ignore
/// use bytemuck::{Pod, Zeroable};
/// use miniextendr_api::raw_conversions::{Raw, RawSlice};
///
/// #[derive(Copy, Clone, Pod, Zeroable)]
/// #[repr(C)]
/// struct Vec3 { x: f32, y: f32, z: f32 }
///
/// #[miniextendr]
/// fn encode(x: f64, y: f64, z: f64) -> Raw<Vec3> {
///     Raw(Vec3 { x: x as f32, y: y as f32, z: z as f32 })
/// }
/// ```
#[cfg(feature = "raw_conversions")]
pub mod raw_conversions;
#[cfg(feature = "raw_conversions")]
pub use raw_conversions::{
    Pod, Raw, RawError, RawHeader, RawSlice, RawSliceTagged, RawTagged, Zeroable, raw_from_bytes,
    raw_slice_from_bytes, raw_slice_to_bytes, raw_to_bytes,
};

/// `match.arg`-style string conversion for enums.
///
/// Provides the [`MatchArg`] trait for converting Rust enums to/from R character
/// strings with partial matching, like R's `match.arg()`.
/// Use `#[derive(MatchArg)]` on C-style enums to auto-generate the implementation.
pub mod match_arg;
pub use match_arg::{
    MatchArg, MatchArgError, choices_sexp, match_arg_from_sexp, match_arg_vec_from_sexp,
    match_arg_vec_into_sexp,
};

/// Factor support for enum ↔ R factor conversions.
///
/// Provides the [`RFactor`] trait for converting Rust enums to/from R factors.
/// Use `#[derive(RFactor)]` on C-style enums to auto-generate the implementation.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::RFactor;
///
/// #[derive(Copy, Clone, RFactor)]
/// enum Color { Red, Green, Blue }
///
/// #[miniextendr]
/// fn color_name(c: Color) -> &'static str {
///     match c {
///         Color::Red => "red",
///         Color::Green => "green",
///         Color::Blue => "blue",
///     }
/// }
/// ```
pub mod factor;
pub use factor::{
    FactorOptionVec, FactorVec, RFactor, UnitEnumFactor, build_factor, build_levels_sexp,
    build_levels_sexp_cached, factor_from_sexp,
};

/// Convenience re-exports for common miniextendr items.
///
/// A single `use miniextendr_api::prelude::*;` brings into scope the most
/// commonly used macros, traits, types, and helpers.
pub mod prelude;
// endregion
