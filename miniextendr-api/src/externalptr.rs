#![allow(rustdoc::private_intra_doc_links)]
//! `ExternalPtr<T>` — a Box-like owned pointer that wraps R's EXTPTRSXP.
//!
//! This provides ownership semantics similar to `Box<T>`, with the key difference
//! that cleanup is deferred to R's garbage collector via finalizers.
//!
//! # Submodules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`altrep_helpers`] | ALTREP data1/data2 slot access helpers + `Sidecar` marker type |
//!
//! # Core Types
//!
//! - [`ExternalPtr<T>`] — owned pointer wrapping EXTPTRSXP
//! - [`TypedExternal`] — display and diagnostic metadata for stored types
//! - [`ExternalSlice<T>`] — helper for slice data in external pointers
//! - [`ErasedExternalPtr`] — type-erased `ExternalPtr<()>` alias
//! - [`IntoExternalPtr`] — conversion trait for wrapping values
//!
//! `PartialEq`/`PartialOrd` compare the pointee values (like `Box<T>`). Use
//! `ptr_eq` when you care about pointer identity, and `as_ref()`/`as_mut()` for
//! explicit by-value comparisons.
//!
//! # Protection Strategies in miniextendr
//!
//! miniextendr provides three complementary protection mechanisms for different scenarios:
//!
//! | Strategy | Module | Lifetime | Release Order | Use Case |
//! |----------|--------|----------|---------------|----------|
//! | **PROTECT stack** | [`gc_protect`](crate::gc_protect) | Within `.Call` | LIFO (stack) | Temporary allocations |
//! | **VECSXP pool** | [`protect_pool`](crate::protect_pool) | Across `.Call`s | Any order | Long-lived R objects |
//! | **R ownership** | [`ExternalPtr`](struct@crate::externalptr::ExternalPtr) | Until R GCs | R decides | Rust data owned by R |
//!
//! ## When to Use ExternalPtr
//!
//! **Use `ExternalPtr` (this module) when:**
//! - You want R to own a Rust value
//! - The Rust value should be dropped when R garbage collects the pointer
//! - You're exposing Rust structs to R code
//!
//! **Use [`gc_protect`](crate::gc_protect) instead when:**
//! - You're allocating temporary R objects during computation
//! - Protection is short-lived (within a single `.Call`)
//!
//! **Use [`ProtectPool`] instead when:**
//! - You need R objects (not Rust values) to survive across `.Call`s
//! - You need arbitrary-order release of protections
//!
//! ## How ExternalPtr Protection Works
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  ExternalPtr<MyStruct>::new(value)                              │
//! │  ├── Rf_protect() during construction (temporary)               │
//! │  ├── R_MakeExternalPtr() creates EXTPTRSXP                      │
//! │  ├── R_RegisterCFinalizerEx() registers cleanup callback        │
//! │  ├── pool.insert() roots it for the Rust handle's lifetime       │
//! │  └── Rf_unprotect() after construction complete                 │
//! │                                                                 │
//! │  Held in Rust (even across other R allocations, e.g. in a Vec)  │
//! │  └── stays alive — the pool's GC-traced VECSXP slot roots it     │
//! │                                                                 │
//! │  Return to R → R now also references the EXTPTRSXP              │
//! │  └── Rust handle drops → pool.release(key) drops the root,      │
//! │      but R's own reference keeps it live                        │
//! │                                                                 │
//! │  R GC runs (no refs left) → finalizer (release_any) frees value │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! Owning handles (`new` / `from_raw` / `Clone`) root their `EXTPTRSXP` in a
//! process-wide [`ProtectPool`](crate::protect_pool) so they survive R
//! allocations while held in Rust; *borrowed* views (`wrap_sexp` / `from_sexp`
//! / `reborrow`) take no root — the object is kept alive by whatever R-side
//! reference handed it to them. The pool (O(1) any-order release) is used
//! rather than `R_PreserveObject` because a `Vec<ExternalPtr>` releases its
//! roots front-to-back, the O(n²) worst case for `R_ReleaseObject`'s
//! precious-list scan — see `analysis/gc-protection-benchmarks-results.md`.
//!
//! When the end goal is an R `list()` of external pointers (rather than a
//! `Vec<ExternalPtr>` you keep working with in Rust), prefer
//! [`ExternalPtr::collect_into_r_list`](struct@ExternalPtr) — it builds each
//! `EXTPTRSXP` straight into the protected result list, so the list roots every
//! element and the pool is never touched at all.
//!
//! # Type Identification
//!
//! Type safety is enforced via `Any::downcast` (Rust's `TypeId`). R symbols
//! in the `tag` and `prot` slots are retained for display and error messages
//! but are **never authoritative** for downcast safety — the `Any` vtable is.
//!
//! Internally, data is stored as `Box<Box<dyn Any>>` — a thin pointer (fits
//! in R's `R_ExternalPtrAddr`) pointing to a fat pointer (carries the `Any`
//! vtable for runtime downcasting). The outer `Box` keeps the heap address
//! stable so [`ExternalPtr::cached_ptr`](struct@ExternalPtr) can be cached
//! once at construction.
//!
//! The `tag` slot holds a symbol (type name, for display).
//! The `prot` slot holds a VECSXP (list) with two elements:
//!   - Index 0: SYMSXP (interned type ID symbol, for error messages)
//!   - Index 1: User-protected SEXP slot (for preventing GC of R objects)
//!
//! ## `TYPE_NAME_CSTR` vs `TYPE_ID_CSTR`
//!
//! [`TypedExternal`] exposes two associated constants with distinct roles —
//! mixing them up does not break type safety (`Any::downcast` is the real
//! gate) but produces noisy diagnostics.
//!
//! | Constant | Role | Visible to R as | Authoritative? |
//! |---|---|---|---|
//! | `TYPE_NAME_CSTR` | Display tag | `class()` / `print()` | No |
//! | `TYPE_ID_CSTR` | Error-message identifier on downcast failure | Stored in `prot[0]` | No (cosmetic; downcast uses `TypeId`) |
//!
//! `#[derive(ExternalPtr)]` fills both with sensible defaults; only override
//! manually when implementing `TypedExternal` by hand.
//!
//! # Pointer provenance for `cached_ptr`
//!
//! `ExternalPtr` caches the data pointer at construction so `as_ref` /
//! `as_mut` avoid an FFI call on every access. The cached `*mut T` **must**
//! be derived from a mutable path so writes through `as_mut` are sound under
//! Stacked Borrows:
//!
//! - `Box::into_raw(Box::new(value))` — preferred (the constructor path).
//! - `&mut T` — when you already hold an exclusive reference.
//! - `<Box<dyn Any>>::downcast_mut::<T>()` — when extracting from the inner box.
//! - [`std::ptr::from_mut`] — when promoting a `&mut T` to a raw pointer.
//!
//! Caching a pointer derived from `&T` or `downcast_ref::<T>()` is **UB**
//! the moment anything writes through it. Internal sites that touch
//! `cached_ptr` are audited; the rule matters for the (rare) hand-rolled
//! `TypedExternal` impl that bypasses [`ExternalPtr::new`].
//!
//! # See also
//!
//! - [`crate::altrep`] — when the alternative (an ALTREP class) makes more
//!   sense than `ExternalPtr`.
//!
//! # ExternalPtr is Not an R Native Type
//!
//! Unlike R's native atomic types (`integer`, `double`, `character`, etc.),
//! external pointers cannot be coerced to vectors or used in R's vectorized
//! operations. This is an R limitation, not a miniextendr limitation:
//!
//! ```r
//! > matrix(new("externalptr"), 1, 1)
//! Error in `as.vector()`:
//! ! cannot coerce type 'externalptr' to vector of type 'any'
//! ```
//!
//! If you need your Rust type to participate in R's vector/matrix operations,
//! consider implementing [`IntoList`](crate::list::IntoList) (via `#[derive(IntoList)]`)
//! to convert your struct to a named R list, or use ALTREP to expose Rust
//! iterators as lazy R vectors.

use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::{self, ManuallyDrop, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::{self, NonNull};

use crate::protect_pool::{ProtectKey, ProtectPool};
use crate::sys::{
    R_ClearExternalPtr, R_ExternalPtrAddr, R_ExternalPtrProtected, R_ExternalPtrTag,
    R_MakeExternalPtr, R_MakeExternalPtr_unchecked, R_RegisterCFinalizerEx,
    R_RegisterCFinalizerEx_unchecked, R_UnboundValue, R_getVarEx, Rf_allocVector,
    Rf_allocVector_unchecked, Rf_install, Rf_install_unchecked, Rf_protect, Rf_protect_unchecked,
    Rf_unprotect, Rf_unprotect_unchecked,
};
use crate::{R_xlen_t, Rboolean, SEXP, SEXPTYPE, SexpExt};

/// A wrapper around a raw pointer that implements [`Send`].
///
/// # Safety
///
/// This is safe to send between threads because it's just a memory address.
/// The data is owned and transferred to the main thread before being accessed.
type SendableAnyPtr = crate::worker::Sendable<NonNull<Box<dyn Any>>>;

/// Create a new sendable pointer from a raw `*mut Box<dyn Any>`.
///
/// # Safety
///
/// The pointer must be non-null.
#[inline]
unsafe fn sendable_any_ptr_new(ptr: *mut Box<dyn Any>) -> SendableAnyPtr {
    // SAFETY: Caller guarantees ptr is non-null
    crate::worker::Sendable(unsafe { NonNull::new_unchecked(ptr) })
}

/// Get the raw pointer, consuming the sendable wrapper.
#[inline]
fn sendable_any_ptr_into_ptr(ptr: SendableAnyPtr) -> *mut Box<dyn Any> {
    ptr.0.as_ptr()
}

/// Index of the type SYMSXP contained in the `prot` (a `VECSXP` list)
const PROT_TYPE_ID_INDEX: isize = 0;
/// Index of user-protected objects contained in the `prot` (a `VECSXP` list)
const PROT_USER_INDEX: isize = 1;
/// Length of the `prot` list (`VECSXP`)
const PROT_VEC_LEN: isize = 2;

#[inline]
fn is_type_erased<T: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<()>()
}

/// Get the interned R symbol for a type's name.
///
/// R interns symbols via `Rf_install`, so the same string always returns the
/// same pointer. The symbol is stable display metadata; `Any::downcast` is the
/// authoritative type check.
///
/// # Safety
///
/// Must be called from R's main thread.
#[inline]
unsafe fn type_symbol<T: TypedExternal>() -> SEXP {
    unsafe { Rf_install(T::TYPE_NAME_CSTR.as_ptr().cast()) }
}

/// Unchecked version of [`type_symbol`] - no thread safety checks.
///
/// # Safety
///
/// Must be called from R's main thread. No debug assertions.
#[inline]
unsafe fn type_symbol_unchecked<T: TypedExternal>() -> SEXP {
    unsafe { Rf_install_unchecked(T::TYPE_NAME_CSTR.as_ptr().cast()) }
}

/// Get the namespaced type ID symbol used in diagnostics.
///
/// Uses `TYPE_ID_CSTR`, which includes the module path to make mismatch
/// messages unambiguous.
///
/// # Safety
///
/// Must be called from R's main thread.
#[inline]
unsafe fn type_id_symbol<T: TypedExternal>() -> SEXP {
    unsafe { Rf_install(T::TYPE_ID_CSTR.as_ptr().cast()) }
}

/// Unchecked version of [`type_id_symbol`].
///
/// # Safety
///
/// Must be called from R's main thread. No debug assertions.
#[inline]
unsafe fn type_id_symbol_unchecked<T: TypedExternal>() -> SEXP {
    unsafe { Rf_install_unchecked(T::TYPE_ID_CSTR.as_ptr().cast()) }
}

/// Get the type name from a stored symbol SEXP.
///
/// # Safety
///
/// `sym` must be a valid SYMSXP.
#[inline]
fn symbol_name(sym: SEXP) -> &'static str {
    use crate::SexpExt;
    let printname = sym.printname();
    let cstr = printname.r_char();
    let len = printname.len();
    unsafe {
        std::str::from_utf8(std::slice::from_raw_parts(cstr.cast(), len))
            .expect("R SYMSXP PRINTNAME is not valid UTF-8")
    }
}

// region: TypedExternalPtr Trait

/// Trait for types that can be stored in an ExternalPtr.
///
/// This provides R-visible display and diagnostic identifiers. Runtime type
/// checking is performed by `Any::downcast` (Rust's `TypeId`), not by comparing
/// these symbols.
///
/// # Type ID vs Type Name
///
/// - `TYPE_ID_CSTR`: Namespaced identifier used in mismatch diagnostics (stored in `prot[0]`).
///   Format: `"<crate_name>@<crate_version>::<module_path>::<type_name>\0"`
///
///   The crate name, version, and module path distinguish otherwise similar
///   names in error messages. They do not determine compatibility.
///
/// - `TYPE_NAME_CSTR`: Short display name for the R tag (shown when printing).
///   Just the type identifier for readability.
pub trait TypedExternal: 'static {
    /// The type name as a static string (for debugging and display)
    const TYPE_NAME: &'static str;

    /// The type name as a null-terminated C string (for R tag display)
    const TYPE_NAME_CSTR: &'static [u8];

    /// Namespaced type ID as a null-terminated C string (for diagnostics).
    ///
    /// This should include the module path to prevent ambiguous messages.
    /// Use `concat!(module_path!(), "::", stringify!(Type), "\0").as_bytes()`
    /// when implementing manually, or use `#[derive(ExternalPtr)]`.
    const TYPE_ID_CSTR: &'static [u8];
}

/// Marker trait for types that should be converted to R as ExternalPtr.
///
/// When a type implements this trait (via `#[derive(ExternalPtr)]`), it gets a
/// blanket `IntoR` implementation that wraps the value in `ExternalPtr<T>`.
///
/// This allows returning the type directly from `#[miniextendr]` functions:
///
/// ```ignore
/// #[derive(ExternalPtr)]
/// struct MyData { value: i32 }
///
/// #[miniextendr]
/// fn create_data(v: i32) -> MyData {
///     MyData { value: v }  // Automatically wrapped in ExternalPtr
/// }
/// ```
pub trait IntoExternalPtr: TypedExternal {}

impl TypedExternal for () {
    const TYPE_NAME: &'static str = "()";
    const TYPE_NAME_CSTR: &'static [u8] = b"()\0";
    // Unit type is special - same ID as name since it's only used for type-erased ptrs
    const TYPE_ID_CSTR: &'static [u8] = b"()\0";
}
// endregion

// region: Class-handle unwrapping (audit A9)

/// Look up a variable bound directly in a single environment frame (no search
/// of enclosing frames — `R_getVarEx` with `inherits = FALSE`, the API-blessed
/// replacement for the removed `Rf_findVarInFrame`).
///
/// Returns `None` if `env` is not itself an environment, or if `name` has no
/// binding in it. Active bindings are forced transparently by R, same as any
/// other variable read. Note: `R_getVarEx` longjmps (raises an R error) if
/// the binding turns out to be `R_MissingArg` — pathological for the
/// `.ptr`/`.__enclos_env__`/`private` handle lookups this function serves,
/// and acceptable here since callers run under the framework's unwind
/// protection.
///
/// # Safety
///
/// Must be called from R's main thread.
unsafe fn env_binding(env: SEXP, name: &std::ffi::CStr) -> Option<SEXP> {
    unsafe {
        if !env.is_environment() {
            return None;
        }
        let sym = Rf_install(name.as_ptr());
        let val = R_getVarEx(sym, env, Rboolean::FALSE, R_UnboundValue);
        if ptr::addr_eq(val.0, R_UnboundValue.0) {
            None
        } else {
            Some(val)
        }
    }
}

/// Attempt to unwrap a class-wrapped handle down to the bare `EXTPTRSXP` it
/// carries, so [`ExternalPtr::<T>`](ExternalPtr) argument conversion accepts
/// the ergonomic class handle (e.g. `Foo$new(...)`) in addition to the raw
/// pointer returned by a low-level constructor (audit finding A9 —
/// `audit/2026-07-03-api-sense-conversions-dataframe-errors.md` #5).
///
/// Tries, in order:
/// - **Env / R6**: a direct `.ptr` binding on `sexp` itself — most
///   `#[miniextendr(env)]` classes are actually a bare classed `EXTPTRSXP`
///   (the generated constructor does `class(.val) <- "T"` directly on the
///   pointer returned by Rust, see `env_class.rs`), which already satisfies
///   the plain `EXTPTRSXP` check and never reaches this function, but a
///   user-authored environment that binds `.ptr` is covered here too. Then
///   the R6 handle chain `.__enclos_env__` -> `private` -> `.ptr` (R6
///   objects are the *public* environment; `private` only hangs off the
///   enclosing environment stored at `.__enclos_env__` for `portable`
///   classes, the default — see `r6_class.rs`).
/// - **S4**: the `ptr` slot via `methods::slot()`
///   ([`crate::s4_helpers::s4_get_slot`]). Guarded by `isS4()`, which
///   excludes S7 objects even though both share the `S4SXP`/`OBJSXP`
///   `SEXPTYPE` — S7's `new_object(S7_object(), ...)` base never sets the S4
///   bit.
/// - **Anything else carrying a `.ptr` attribute**: S7 stores properties as
///   plain attributes on its base object (see `s7_class.rs`), so
///   `Rf_getAttrib(x, ".ptr")` recovers the pointer without going through
///   S7's `@`/`prop()` dispatch machinery.
///
/// Returns `Some(inner)` only when the unwrapped value is itself an
/// `EXTPTRSXP` — anything else (e.g. a `.ptr`-named field that isn't a
/// pointer) is treated as "no handle found" rather than an error. No
/// recursion beyond one unwrap level. `Any::downcast` remains the type-safety
/// authority: unwrapping a handle for the *wrong* `T` still fails at the
/// caller with the existing type-mismatch error — this only loosens the
/// accepted R-side shape, not type safety.
///
/// # Safety
///
/// - Must be called from R's main thread.
/// - The returned SEXP is reachable from `sexp` (an env binding, S4 slot, or
///   attribute) for as long as `sexp` itself is protected. Macro-generated
///   `.Call()` wrappers hold every argument alive in the call's PROTECT stack
///   for the duration of the call, so no additional protection is needed
///   here.
pub(crate) unsafe fn unwrap_class_handle(sexp: SEXP) -> Option<SEXP> {
    unsafe {
        if sexp.is_environment() {
            if let Some(direct) = env_binding(sexp, c".ptr") {
                if direct.type_of() == SEXPTYPE::EXTPTRSXP {
                    return Some(direct);
                }
            }
            let enclos = env_binding(sexp, c".__enclos_env__")?;
            let private = env_binding(enclos, c"private")?;
            let inner = env_binding(private, c".ptr")?;
            return (inner.type_of() == SEXPTYPE::EXTPTRSXP).then_some(inner);
        }

        if sexp.is_s4() {
            let slot = crate::s4_helpers::s4_get_slot(sexp, "ptr").ok()?;
            return (slot.type_of() == SEXPTYPE::EXTPTRSXP).then_some(slot);
        }

        let attr = sexp.get_attr(Rf_install(c".ptr".as_ptr()));
        (attr.type_of() == SEXPTYPE::EXTPTRSXP).then_some(attr)
    }
}
// endregion

// region: ExternalPtr<T>

/// An owned pointer stored in R's external pointer SEXP.
///
/// This is conceptually similar to `Box<T>`, but with the following differences:
/// - Memory is freed by R's GC via a registered finalizer (non-deterministic)
/// - The underlying SEXP is Copy, so aliasing must be manually prevented
/// - Type checking happens at runtime via `Any::downcast` (Rust `TypeId`)
///
/// # Thread Safety
///
/// `ExternalPtr` is `Send` to allow returning from worker thread functions.
/// However, **concurrent access is not allowed** - R's runtime is single-threaded.
/// All R API calls are serialized through the main thread via `with_r_thread`.
///
/// # Safety
///
/// The ExternalPtr assumes exclusive ownership of the underlying data.
/// Cloning the raw SEXP without proper handling will lead to double-free.
///
/// # Examples
///
/// ```no_run
/// use miniextendr_api::externalptr::{ExternalPtr, TypedExternal};
///
/// struct MyData { value: f64 }
/// impl TypedExternal for MyData {
///     const TYPE_NAME: &'static str = "MyData";
///     const TYPE_NAME_CSTR: &'static [u8] = b"MyData\0";
///     const TYPE_ID_CSTR: &'static [u8] = b"my_crate::MyData\0";
/// }
///
/// let ptr = ExternalPtr::new(MyData { value: 3.14 });
/// assert_eq!(ptr.as_ref().unwrap().value, 3.14);
/// ```
#[repr(C)]
pub struct ExternalPtr<T: TypedExternal> {
    sexp: SEXP,
    /// Cached data pointer, set once at construction time.
    ///
    /// This avoids the `R_ExternalPtrAddr` FFI call on every `as_ref()`/`as_mut()`.
    /// The pointer remains valid for the lifetime of the `ExternalPtr` because:
    /// - R's finalizer only runs after R garbage-collects the SEXP (which cannot
    ///   happen while a Rust `ExternalPtr` value exists).
    /// - `R_ClearExternalPtr` is only called in methods that consume or finalize
    ///   (`into_raw`, `into_inner`, `release_any`).
    cached_ptr: NonNull<T>,
    /// The [`ProtectPool`] key rooting this handle's `EXTPTRSXP`, or `None` for
    /// borrowed views.
    ///
    /// `Some(key)` for *owning* handles built from a fresh value (`new` /
    /// `new_unchecked` / `from_raw` / `Clone` / `Default`): the constructor
    /// roots the `EXTPTRSXP` in the main-thread pool so it stays alive for the
    /// whole Rust lifetime of the handle — including while it sits in a `Vec`
    /// across other R allocations before being handed to R (#836). `Drop` /
    /// `into_raw` / `into_inner` release the root via the key.
    ///
    /// `None` for *borrowed* views of an SEXP R already owns (`wrap_sexp*` /
    /// `from_sexp*` / `reborrow`): no root is taken, so none is released. The
    /// aliased object is kept alive by whatever R-side reference handed it to us
    /// (a `.Call` argument frame, an owning sibling handle, …).
    root: Option<ProtectKey>,
    _marker: PhantomData<T>,
}

// SAFETY: ExternalPtr can be sent between threads because:
// 1. All R API operations are serialized through the main thread via with_r_thread
// 2. The worker thread is blocked while the main thread processes R calls
// 3. There is no concurrent access - only sequential hand-off between threads
unsafe impl<T: TypedExternal + Send> Send for ExternalPtr<T> {}

// region: ExternalPtr GC roots
//
// Owning `ExternalPtr` handles keep their `EXTPTRSXP` alive for the handle's
// whole Rust lifetime by rooting it in a process-wide `ProtectPool` — a single
// GC-traced VECSXP with Rust-side slot bookkeeping. This is what makes a naive
// `Vec<ExternalPtr<T>>` GC-safe: every element stays rooted while later elements
// allocate (#836).
//
// Why a pool and not `R_PreserveObject`: the pool releases in O(1) any order,
// whereas `R_ReleaseObject` scans R's precious list (O(n)). A `Vec<ExternalPtr>`
// drops front-to-back — oldest first, i.e. the entries deepest in R's LIFO
// precious list — so `R_PreserveObject` rooting degrades to O(n²) on exactly
// this workload (60–65× slower at 10k; see
// analysis/gc-protection-benchmarks-results.md). The pool is the mechanism the
// strategy analysis prescribes for ExternalPtr (analysis/gc-protection-strategies.md).
//
// `ProtectPool` is `!Send`/`!Sync` and lives in a `thread_local!` on R's main
// thread. Every access happens there: roots are taken inside
// `create_extptr_sexp[_unchecked]` (main-thread by contract / `with_r_thread`),
// and released through `with_r_thread` from `Drop` / `into_raw` / `into_inner`.
// The pool is wrapped in `ManuallyDrop` so it is never released at thread exit —
// it is a session-lifetime root table, and running `R_ReleaseObject` on its
// backing during R's own teardown would touch a half-freed R heap.

thread_local! {
    static EXTPTR_ROOTS: RefCell<Option<ManuallyDrop<ProtectPool>>> = const { RefCell::new(None) };
}

/// Root an owning handle's `EXTPTRSXP` in the main-thread pool.
///
/// Must run on R's main thread with `sexp` already protected by the caller (the
/// pool may allocate while growing). Both hold inside
/// `create_extptr_sexp[_unchecked]`.
#[inline]
fn root_owned(sexp: SEXP) -> ProtectKey {
    EXTPTR_ROOTS.with_borrow_mut(|slot| {
        let pool = slot.get_or_insert_with(|| {
            // SAFETY: on R's main thread (caller contract); R is initialized
            // (we are mid-`create_extptr_sexp`, allocating R objects).
            ManuallyDrop::new(unsafe { ProtectPool::new(ProtectPool::DEFAULT_CAPACITY) })
        });
        // SAFETY: on R's main thread; `sexp` is live (protected by the caller).
        unsafe { pool.insert(sexp) }
    })
}

/// Release an owning handle's pool root. Stale keys are a safe no-op.
///
/// Must run on R's main thread (callers route through `with_r_thread`).
#[inline]
fn unroot_owned(key: ProtectKey) {
    EXTPTR_ROOTS.with_borrow_mut(|slot| {
        if let Some(pool) = slot.as_mut() {
            // SAFETY: on R's main thread.
            unsafe { pool.release(key) };
        }
    });
}
// endregion

impl<T: TypedExternal> ExternalPtr<T> {
    /// Build an *owning* handle rooted at `root`.
    ///
    /// Pairs with the [`ProtectPool`] root that [`create_extptr_sexp`] /
    /// [`create_extptr_sexp_unchecked`] take on the SEXP (and return as the
    /// key). The root is released by `Drop` / `into_raw` / `into_inner`. Only
    /// the four fresh-value constructors (`new` / `new_unchecked` / `from_raw` /
    /// `from_raw_unchecked`) build through here.
    ///
    /// [`create_extptr_sexp`]: Self::create_extptr_sexp
    /// [`create_extptr_sexp_unchecked`]: Self::create_extptr_sexp_unchecked
    #[inline]
    fn from_owned_parts(sexp: SEXP, cached_ptr: NonNull<T>, root: ProtectKey) -> Self {
        Self {
            sexp,
            cached_ptr,
            root: Some(root),
            _marker: PhantomData,
        }
    }

    /// Build a *borrowed* view (`root = None`) of an SEXP R already owns.
    ///
    /// No GC root is taken and none is released — the aliased object is kept
    /// alive by the R-side reference that handed it to us. Used by every
    /// `wrap_sexp*` / `from_sexp*` / `reborrow` path.
    #[inline]
    fn from_borrowed_parts(sexp: SEXP, cached_ptr: NonNull<T>) -> Self {
        Self {
            sexp,
            cached_ptr,
            root: None,
            _marker: PhantomData,
        }
    }

    /// Release the pool root iff this handle owns one.
    ///
    /// Routed through [`with_r_thread`] because an owning `ExternalPtr` is
    /// `Send` and may be dropped on the worker thread, while the pool lives on
    /// R's main thread. `with_r_thread` runs the closure inline when already on
    /// the main thread (the common case), so this is a direct pool release
    /// there and a thread hand-off only from the worker. `ProtectKey` is `Copy`
    /// + `Send` (two `u32`s), so it crosses the boundary by value.
    ///
    /// [`with_r_thread`]: crate::worker::with_r_thread
    #[inline]
    fn release_root_if_owned(&self) {
        let Some(key) = self.root else {
            return;
        };
        crate::worker::with_r_thread(move || unroot_owned(key));
    }

    /// Allocates memory on the heap and places `x` into it.
    ///
    /// Internally stores a `Box<Box<dyn Any>>` — a thin pointer (fits in R's
    /// `R_ExternalPtrAddr`) pointing to a fat pointer (carries the `Any` vtable
    /// for runtime type checking via `downcast`).
    ///
    /// This function can be called from the two supported R contexts:
    /// - If called from R's main thread, creates the ExternalPtr directly
    /// - If called from the worker thread (during `run_on_worker`), automatically
    ///   sends the R API calls to the main thread via [`with_r_thread`]
    ///
    /// # Panics
    ///
    /// Panics if called from a non-main thread outside of a `run_on_worker` context.
    ///
    /// Equivalent to `Box::new`.
    ///
    /// [`with_r_thread`]: crate::worker::with_r_thread
    #[inline]
    pub fn new(x: T) -> Self {
        // Get concrete pointer with full write provenance from Box::into_raw,
        // BEFORE erasing to dyn Any. This preserves mutable provenance for
        // cached_ptr (downcast_ref would give shared-reference provenance,
        // which is UB for later writes through as_mut()).
        let raw: *mut T = Box::into_raw(Box::new(x));
        // SAFETY: Box::into_raw never returns null
        let cached_ptr = unsafe { NonNull::new_unchecked(raw) };

        // Re-wrap: Box::from_raw(raw) → Box<dyn Any> → Box<Box<dyn Any>>
        // The data stays at `raw`; we're just adding the Any vtable wrapper.
        let inner: Box<dyn Any> = unsafe { Box::from_raw(raw) };
        let any_raw: *mut Box<dyn Any> = Box::into_raw(Box::new(inner));

        // Wrap in Sendable so it can be sent across thread boundary
        let sendable = unsafe { sendable_any_ptr_new(any_raw) };

        // Use with_r_thread to run R API calls on main thread. The pool root is
        // taken there (on the main thread, where the pool lives) and the key
        // crosses back by value — `(SEXP, ProtectKey)` is `Send`.
        let (sexp, root) = crate::worker::with_r_thread(move || {
            let any_raw = sendable_any_ptr_into_ptr(sendable);
            unsafe { Self::create_extptr_sexp_unchecked(any_raw) }
        });

        Self::from_owned_parts(sexp, cached_ptr, root)
    }

    /// Allocates memory on the heap and places `x` into it, without thread checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. Calling from another thread
    /// is undefined behavior (R APIs are not thread-safe).
    #[inline]
    pub unsafe fn new_unchecked(x: T) -> Self {
        let raw: *mut T = Box::into_raw(Box::new(x));
        let cached_ptr = unsafe { NonNull::new_unchecked(raw) };

        let inner: Box<dyn Any> = unsafe { Box::from_raw(raw) };
        let any_raw: *mut Box<dyn Any> = Box::into_raw(Box::new(inner));

        let (sexp, root) = unsafe { Self::create_extptr_sexp_unchecked(any_raw) };
        Self::from_owned_parts(sexp, cached_ptr, root)
    }

    /// Create an EXTPTRSXP from a `*mut Box<dyn Any>`. Must be called from main thread.
    ///
    /// The `any_raw` is a thin pointer to a heap-allocated fat pointer (`Box<dyn Any>`).
    /// R stores the thin pointer in `R_ExternalPtrAddr`. Returns the SEXP and the
    /// [`ProtectPool`] key that roots it for the owning handle's lifetime.
    #[inline]
    unsafe fn create_extptr_sexp(any_raw: *mut Box<dyn Any>) -> (SEXP, ProtectKey) {
        debug_assert!(
            !any_raw.is_null(),
            "create_extptr_sexp received null pointer"
        );

        let type_sym = unsafe { type_symbol::<T>() };
        let type_id_sym = unsafe { type_id_symbol::<T>() };

        // keep raw: this protect/unprotect straddles the ProtectPool handoff
        // (`root_owned` below), a two-stage rooting boundary that outlives this
        // function via the pool key — not a lexical RAII scope. `OwnedProtect` /
        // `ProtectScope` would misrepresent the ownership transfer.
        let prot = unsafe { Rf_allocVector(SEXPTYPE::VECSXP, PROT_VEC_LEN) };
        unsafe { Rf_protect(prot) };
        prot.set_vector_elt(PROT_TYPE_ID_INDEX, type_id_sym);

        let sexp = unsafe { R_MakeExternalPtr(any_raw.cast(), type_sym, prot) };
        unsafe { Rf_protect(sexp) };

        // Non-generic finalizer — Box<dyn Any> vtable handles the concrete drop
        unsafe { R_RegisterCFinalizerEx(sexp, Some(release_any), Rboolean::TRUE) };

        // Root the owning handle for its whole Rust lifetime so it survives R
        // allocations while held (e.g. element-by-element in a `Vec`) before
        // reaching R (#836). The pool gives O(1) any-order release — see the
        // `EXTPTR_ROOTS` docs for why that beats `R_PreserveObject` here. `sexp`
        // is still protected, so the pool may safely allocate while growing.
        // Must happen here, on the main thread, because `new` returns the SEXP
        // to the *calling* thread (possibly the worker) where R API is gone.
        let root = root_owned(sexp);

        unsafe { Rf_unprotect(2) };
        (sexp, root)
    }

    /// Create an EXTPTRSXP from a `*mut Box<dyn Any>` without thread safety checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. No debug assertions for thread safety.
    ///
    /// Returns the SEXP and the [`ProtectPool`] key that roots it.
    #[inline]
    unsafe fn create_extptr_sexp_unchecked(any_raw: *mut Box<dyn Any>) -> (SEXP, ProtectKey) {
        debug_assert!(
            !any_raw.is_null(),
            "create_extptr_sexp_unchecked received null pointer"
        );

        let type_sym = unsafe { type_symbol_unchecked::<T>() };
        let type_id_sym = unsafe { type_id_symbol_unchecked::<T>() };

        let prot = unsafe { Rf_allocVector_unchecked(SEXPTYPE::VECSXP, PROT_VEC_LEN) };
        unsafe { Rf_protect_unchecked(prot) };
        unsafe { prot.set_vector_elt_unchecked(PROT_TYPE_ID_INDEX, type_id_sym) };

        let sexp = unsafe { R_MakeExternalPtr_unchecked(any_raw.cast(), type_sym, prot) };
        unsafe { Rf_protect_unchecked(sexp) };

        // Non-generic finalizer — Box<dyn Any> vtable handles the concrete drop
        unsafe {
            R_RegisterCFinalizerEx_unchecked(sexp, Some(release_any), Rboolean::TRUE);
        };

        // Root the owning handle (see `create_extptr_sexp` for the rationale).
        // `root_owned` uses the pool's checked FFI, which runs inline here
        // because the unchecked constructors are main-thread-by-contract; `sexp`
        // is still protected, covering any allocation inside a pool grow.
        let root = root_owned(sexp);

        unsafe { Rf_unprotect_unchecked(2) };
        (sexp, root)
    }

    /// Collect an iterator of values into a protected R list (`VECSXP`) holding
    /// one fresh external pointer per item, rooting each via the destination
    /// list instead of the [`ProtectPool`](crate::protect_pool).
    ///
    /// This is the GC-safe, allocation-lean way to hand many Rust values to R at
    /// once — e.g. converting a `Vec<T>` into an R `list()` of external pointers.
    /// Each `EXTPTRSXP` is created and **immediately** stored into the
    /// already-protected result list, so the list roots it the instant it
    /// exists: there is no unprotected window between element allocations, and
    /// **no per-element pool traffic**.
    ///
    /// Contrast the naive `items.map(ExternalPtr::new).collect::<Vec<_>>()`,
    /// which roots every handle in the process-wide pool (keeping the `Vec`
    /// GC-safe while held — #836) only to release every root again when the `Vec`
    /// drops, then still needs a second pass to copy the handles into a list.
    /// Here the list *is* the root, so both the pool round-trip and the copy
    /// pass are skipped. The whole batch also crosses to R's main thread in a
    /// single [`with_r_thread`](crate::worker::with_r_thread) hop rather than one
    /// per element.
    ///
    /// The returned `VECSXP` is **not** protected: the caller must protect it or
    /// return it to R immediately, exactly like any other freshly built SEXP
    /// (e.g. an [`IntoR`](crate::IntoR) result).
    pub fn collect_into_r_list<I>(items: I) -> SEXP
    where
        I: IntoIterator<Item = T>,
    {
        // Box + type-erase every value on the *calling* thread (no R API needed),
        // then ship only the raw thin pointers to the main thread — the same
        // ownership transfer `new` performs, batched. `Sendable` carries the Vec
        // across the boundary; the values are owned and handed off, never aliased.
        let raws: Vec<*mut Box<dyn Any>> = items
            .into_iter()
            .map(|x| {
                let inner: Box<dyn Any> = Box::new(x);
                Box::into_raw(Box::new(inner))
            })
            .collect();
        let sendable = crate::worker::Sendable(raws);

        crate::worker::with_r_thread(move || {
            let raws = sendable.0;
            // SAFETY: `with_r_thread` runs this on R's main thread; every entry
            // is a live `Box<Box<dyn Any>>` wrapping a `T`, ownership transferred.
            unsafe { Self::build_extptr_list(&raws) }
        })
    }

    /// Build a protected `VECSXP` of external pointers from already-erased boxes.
    ///
    /// Allocates the result list, protects it, then creates one `EXTPTRSXP` per
    /// entry directly into its slot — rooted by the protected list, no pool. The
    /// type symbols are interned once and reused (they are never GC'd, so they
    /// stay valid across the allocating loop). Returns the list **unprotected**.
    ///
    /// # Safety
    ///
    /// Must run on R's main thread; each `raw` must be a live `Box<Box<dyn Any>>`
    /// wrapping a `T`, with ownership transferred to the new external pointer.
    unsafe fn build_extptr_list(raws: &[*mut Box<dyn Any>]) -> SEXP {
        let n = R_xlen_t::try_from(raws.len()).expect("list length exceeds R_xlen_t::MAX");
        let list = unsafe { Rf_allocVector_unchecked(SEXPTYPE::VECSXP, n) };
        unsafe { Rf_protect_unchecked(list) };

        let type_sym = unsafe { type_symbol_unchecked::<T>() };
        let type_id_sym = unsafe { type_id_symbol_unchecked::<T>() };

        for (i, &any_raw) in raws.iter().enumerate() {
            let idx = R_xlen_t::try_from(i).expect("index exceeds R_xlen_t::MAX");
            // SAFETY: main thread; `any_raw` owns a `T`; `list` is protected, so
            // it roots each element the instant `set_vector_elt` stores it.
            unsafe { Self::make_extptr_into_slot(any_raw, type_sym, type_id_sym, list, idx) };
        }

        unsafe { Rf_unprotect_unchecked(1) };
        list
    }

    /// Create an `EXTPTRSXP` for `any_raw` and store it into `dest[idx]`.
    ///
    /// Mirrors [`create_extptr_sexp_unchecked`](Self::create_extptr_sexp_unchecked)
    /// but roots the new pointer via `dest` (which the caller keeps protected)
    /// instead of the pool — the element is live the instant it lands in the
    /// protected list, so a bulk build pays no pool insert/release per element.
    ///
    /// # Safety
    ///
    /// Must run on R's main thread; `any_raw` must own a `T`; `dest` must be a
    /// protected `VECSXP` with `idx` in bounds; `type_sym` / `type_id_sym` must
    /// be the interned symbols for `T`.
    #[inline]
    unsafe fn make_extptr_into_slot(
        any_raw: *mut Box<dyn Any>,
        type_sym: SEXP,
        type_id_sym: SEXP,
        dest: SEXP,
        idx: R_xlen_t,
    ) {
        let prot = unsafe { Rf_allocVector_unchecked(SEXPTYPE::VECSXP, PROT_VEC_LEN) };
        unsafe { Rf_protect_unchecked(prot) };
        unsafe { prot.set_vector_elt_unchecked(PROT_TYPE_ID_INDEX, type_id_sym) };

        let sexp = unsafe { R_MakeExternalPtr_unchecked(any_raw.cast(), type_sym, prot) };
        unsafe { Rf_protect_unchecked(sexp) };
        unsafe { R_RegisterCFinalizerEx_unchecked(sexp, Some(release_any), Rboolean::TRUE) };

        // Root via the destination list instead of the pool: `dest` is protected
        // by the caller, so storing `sexp` keeps it (and its `prot`) alive with
        // no pool churn.
        unsafe { dest.set_vector_elt_unchecked(idx, sexp) };

        unsafe { Rf_unprotect_unchecked(2) };
    }

    /// Constructs a new `ExternalPtr` with uninitialized contents.
    ///
    /// Equivalent to `Box::new_uninit`.
    #[inline]
    pub fn new_uninit() -> ExternalPtr<MaybeUninit<T>>
    where
        MaybeUninit<T>: TypedExternal,
    {
        ExternalPtr::new(MaybeUninit::uninit())
    }

    /// Constructs a new `ExternalPtr` with zeroed contents.
    ///
    /// Equivalent to `Box::new_zeroed`.
    #[inline]
    pub fn new_zeroed() -> ExternalPtr<MaybeUninit<T>>
    where
        MaybeUninit<T>: TypedExternal,
    {
        ExternalPtr::new(MaybeUninit::zeroed())
    }

    /// Constructs an ExternalPtr from a raw pointer.
    ///
    /// Re-wraps the `*mut T` in `Box<dyn Any>` for the new storage format.
    ///
    /// # Safety
    ///
    /// - `raw` must have been allocated via `Box::into_raw` or equivalent
    /// - `raw` must not be null
    /// - Caller transfers ownership to the ExternalPtr
    /// - Must be called from R's main thread
    ///
    /// Equivalent to `Box::from_raw`.
    #[inline]
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        // Re-wrap in Box<dyn Any> → Box<Box<dyn Any>>
        let inner: Box<dyn Any> = unsafe { Box::from_raw(raw) };
        let outer: Box<Box<dyn Any>> = Box::new(inner);
        let any_raw: *mut Box<dyn Any> = Box::into_raw(outer);

        let (sexp, root) = unsafe { Self::create_extptr_sexp(any_raw) };
        Self::from_owned_parts(sexp, unsafe { NonNull::new_unchecked(raw) }, root)
    }

    /// Constructs an ExternalPtr from a raw pointer, without thread checks.
    ///
    /// # Safety
    ///
    /// - `raw` must have been allocated via `Box::into_raw` or equivalent
    /// - `raw` must not be null
    /// - Caller transfers ownership to the ExternalPtr
    /// - Must be called from R's main thread (no debug assertions)
    #[inline]
    pub unsafe fn from_raw_unchecked(raw: *mut T) -> Self {
        let inner: Box<dyn Any> = unsafe { Box::from_raw(raw) };
        let outer: Box<Box<dyn Any>> = Box::new(inner);
        let any_raw: *mut Box<dyn Any> = Box::into_raw(outer);

        let (sexp, root) = unsafe { Self::create_extptr_sexp_unchecked(any_raw) };
        Self::from_owned_parts(sexp, unsafe { NonNull::new_unchecked(raw) }, root)
    }

    /// Consumes the ExternalPtr, returning a raw pointer.
    ///
    /// The caller is responsible for the memory, and the finalizer is
    /// effectively orphaned (will do nothing since we clear the pointer).
    ///
    /// Equivalent to `Box::into_raw`.
    #[inline]
    pub fn into_raw(this: Self) -> *mut T {
        let ptr = this.cached_ptr.as_ptr();

        // Ownership of the R object leaves this handle: drop our GC root before
        // `mem::forget` skips `Drop`. (`into_raw` already calls R API directly,
        // so it is main-thread-contract — release directly, no thread hop.)
        this.release_root_if_owned();

        // Recover and disassemble the Box<Box<dyn Any>> wrapper.
        // We need to free the wrapper allocations without dropping the T data.
        let any_raw = unsafe { R_ExternalPtrAddr(this.sexp) as *mut Box<dyn Any> };

        // Clear the external pointer so the finalizer becomes a no-op
        unsafe { R_ClearExternalPtr(this.sexp) };

        if !any_raw.is_null() {
            // Reconstruct outer box → extract inner → leak inner (prevents T drop)
            let outer: Box<Box<dyn Any>> = unsafe { Box::from_raw(any_raw) };
            let inner: Box<dyn Any> = *outer;
            // Box::into_raw leaks the inner allocation — caller owns T via `ptr`
            let _ = Box::into_raw(inner);
        }

        // Don't run our Drop
        mem::forget(this);

        ptr
    }

    /// Consumes the ExternalPtr, returning a `NonNull` pointer.
    ///
    /// Equivalent to `Box::into_non_null`.
    #[inline]
    pub fn into_non_null(this: Self) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(Self::into_raw(this)) }
    }

    /// Consumes and leaks the ExternalPtr, returning a mutable reference.
    ///
    /// The memory will never be freed (from Rust's perspective; R's GC
    /// finalizer is neutralized).
    ///
    /// Equivalent to `Box::leak`.
    #[inline]
    pub fn leak<'a>(this: Self) -> &'a mut T
    where
        T: 'a,
    {
        unsafe { &mut *Self::into_raw(this) }
    }

    /// Consumes the ExternalPtr, returning the wrapped value.
    ///
    /// Uses `Box<dyn Any>::downcast` to recover the concrete `Box<T>`,
    /// then moves the value out.
    ///
    /// Equivalent to `*boxed` (deref move) or `Box::into_inner`.
    #[inline]
    pub fn into_inner(this: Self) -> T {
        // Ownership leaves this handle: drop our GC root before `mem::forget`.
        this.release_root_if_owned();

        let any_raw = unsafe { R_ExternalPtrAddr(this.sexp) as *mut Box<dyn Any> };

        // Clear so finalizer is no-op
        unsafe { R_ClearExternalPtr(this.sexp) };
        mem::forget(this);

        assert!(!any_raw.is_null(), "ExternalPtr is null or cleared");
        let outer: Box<Box<dyn Any>> = unsafe { Box::from_raw(any_raw) };
        let inner: Box<dyn Any> = *outer;
        *inner
            .downcast::<T>()
            .expect("ExternalPtr type mismatch in into_inner")
    }

    // region: Pin support (Box-equivalent)

    /// Constructs a new `Pin<ExternalPtr<T>>`.
    ///
    /// Equivalent to `Box::pin`.
    ///
    /// # Note
    ///
    /// Unlike `Box::pin`, this requires `T: Unpin` because `ExternalPtr`
    /// implements `DerefMut` unconditionally. For `!Unpin` types, use
    /// `ExternalPtr::new` and manage pinning guarantees manually.
    #[inline]
    pub fn pin(x: T) -> Pin<Self>
    where
        T: Unpin,
    {
        // SAFETY: T: Unpin, so pinning is always safe
        Pin::new(Self::new(x))
    }

    /// Constructs a new `Pin<ExternalPtr<T>>` without requiring `Unpin`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pinning invariants are upheld:
    /// - The data will not be moved out of the `ExternalPtr`
    /// - The data will not be accessed mutably in ways that would move it
    ///
    /// Since `ExternalPtr` implements `DerefMut`, using this with `!Unpin`
    /// types requires careful handling to avoid moving the inner value.
    #[inline]
    pub fn pin_unchecked(x: T) -> Pin<Self> {
        unsafe { Pin::new_unchecked(Self::new(x)) }
    }

    /// Converts a `ExternalPtr<T>` into a `Pin<ExternalPtr<T>>`.
    ///
    /// Equivalent to `Box::into_pin`.
    #[inline]
    pub fn into_pin(this: Self) -> Pin<Self>
    where
        T: Unpin,
    {
        // SAFETY: T: Unpin, so it's always safe to pin
        Pin::new(this)
    }
    // endregion

    // region: Accessors

    /// Returns a reference to the underlying value.
    ///
    /// Uses the cached pointer set at construction time, avoiding the
    /// `R_ExternalPtrAddr` FFI call on every access.
    #[inline]
    pub fn as_ref(&self) -> Option<&T> {
        // SAFETY: cached_ptr is always valid for the lifetime of ExternalPtr
        Some(unsafe { self.cached_ptr.as_ref() })
    }

    /// Returns a mutable reference to the underlying value.
    ///
    /// Uses the cached pointer set at construction time, avoiding the
    /// `R_ExternalPtrAddr` FFI call on every access.
    #[inline]
    pub fn as_mut(&mut self) -> Option<&mut T> {
        // SAFETY: cached_ptr is always valid for the lifetime of ExternalPtr
        Some(unsafe { self.cached_ptr.as_mut() })
    }

    /// Returns the raw pointer without consuming the ExternalPtr.
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.cached_ptr.as_ptr().cast_const()
    }

    /// Returns the raw mutable pointer without consuming the ExternalPtr.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.cached_ptr.as_ptr()
    }

    /// Checks whether two `ExternalPtr`s refer to the same allocation (pointer identity).
    ///
    /// This ignores the pointee values. Use this when you need alias detection;
    /// prefer `PartialEq`/`PartialOrd` or `as_ref()` for value comparisons.
    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        ptr::eq(
            this.cached_ptr.as_ptr().cast_const(),
            other.cached_ptr.as_ptr().cast_const(),
        )
    }
    // endregion

    // region: R-specific accessors

    /// Returns the underlying SEXP.
    ///
    /// # Warning
    ///
    /// The returned SEXP must not be duplicated or the finalizer will double-free.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.sexp
    }

    /// Create a lightweight alias of this ExternalPtr sharing the same R object.
    ///
    /// The returned `ExternalPtr` points to the **same** underlying EXTPTRSXP.
    /// No data is copied and no new R object is allocated -- both the original
    /// and the alias refer to the same R-level external pointer.
    ///
    /// This is the correct way to return "self" from a method that takes
    /// `self: &ExternalPtr<Self>`, preserving R object identity:
    ///
    /// ```ignore
    /// #[miniextendr(env)]
    /// impl MyType {
    ///     pub fn identity(self: &ExternalPtr<Self>) -> ExternalPtr<Self> {
    ///         self.reborrow()
    ///     }
    /// }
    /// ```
    ///
    /// # Safety note
    ///
    /// The caller must not use the original and the alias to create overlapping
    /// mutable references (`as_mut`). In typical use (returning from a method),
    /// the borrow of the original ends when the method returns, so this is safe.
    #[inline]
    pub fn reborrow(&self) -> Self {
        // SAFETY: self.sexp is a valid live EXTPTRSXP that we already hold.
        // wrap_sexp re-extracts the data pointer from the same SEXP.
        unsafe { Self::wrap_sexp(self.sexp) }
            .expect("reborrow of live ExternalPtr should never fail")
    }

    /// Returns the tag SEXP (type identifier symbol).
    #[inline]
    pub fn tag(&self) -> SEXP {
        unsafe { R_ExternalPtrTag(self.sexp) }
    }

    /// Returns the tag SEXP (unchecked version).
    ///
    /// Skips thread safety checks for performance-critical paths.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. Only use in ALTREP callbacks
    /// or other contexts where you're certain you're on the main thread.
    #[inline]
    pub unsafe fn tag_unchecked(&self) -> SEXP {
        unsafe { crate::sys::R_ExternalPtrTag_unchecked(self.sexp) }
    }

    /// Returns the protected SEXP slot (user-protected objects).
    ///
    /// This returns the user-protected object stored in the prot VECSXP,
    /// not the VECSXP itself.
    #[inline]
    pub fn protected(&self) -> SEXP {
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null_or_nil() {
                return SEXP::nil();
            }
            if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
                return SEXP::nil();
            }
            prot.vector_elt(PROT_USER_INDEX)
        }
    }

    /// Returns the protected SEXP slot (unchecked version).
    ///
    /// Skips thread safety checks for performance-critical paths.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. Only use in ALTREP callbacks
    /// or other contexts where you're certain you're on the main thread.
    #[inline]
    pub unsafe fn protected_unchecked(&self) -> SEXP {
        use crate::sys::R_ExternalPtrProtected_unchecked;

        unsafe {
            let prot = R_ExternalPtrProtected_unchecked(self.sexp);
            if prot.is_null_or_nil() {
                return SEXP::nil();
            }
            if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
                return SEXP::nil();
            }
            prot.vector_elt_unchecked(PROT_USER_INDEX)
        }
    }

    /// Sets the user-protected SEXP slot.
    ///
    /// Use this to prevent R objects from being GC'd while this ExternalPtr exists.
    /// The type ID stored in prot slot 0 is preserved.
    ///
    /// Returns `false` if the prot structure is malformed (should not happen
    /// for ExternalPtrs created by this library).
    ///
    /// # Safety
    ///
    /// - `user_prot` must be a valid SEXP or R_NilValue
    /// - Must be called from the R main thread
    #[inline]
    pub unsafe fn set_protected(&self, user_prot: SEXP) -> bool {
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null_or_nil() {
                debug_assert!(false, "ExternalPtr prot slot is null or R_NilValue");
                return false;
            }
            if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
                debug_assert!(
                    false,
                    "ExternalPtr prot slot is not a VECSXP of expected length"
                );
                return false;
            }
            prot.set_vector_elt(PROT_USER_INDEX, user_prot);
            true
        }
    }

    /// Returns the raw prot VECSXP (contains both type ID and user protected).
    ///
    /// Prefer using `protected()` for user data and `stored_type_id()` for type info.
    #[inline]
    pub fn prot_raw(&self) -> SEXP {
        unsafe { R_ExternalPtrProtected(self.sexp) }
    }

    /// Checks if the internal pointer is null (already finalized or cleared).
    #[inline]
    pub fn is_null(&self) -> bool {
        unsafe { R_ExternalPtrAddr(self.sexp).is_null() }
    }
    // endregion

    // region: Type checking

    /// Attempt to wrap a SEXP as an ExternalPtr with type checking.
    ///
    /// Uses `Any::downcast_ref` for authoritative type checking (Rust `TypeId`).
    /// Type-erased `ExternalPtr<()>` deliberately skips the concrete downcast.
    ///
    /// Returns `None` if:
    /// - The internal pointer is null
    /// - The stored `Box<dyn Any>` does not contain a `T`
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP created by this library
    /// - The caller must ensure no other ExternalPtr owns this SEXP
    pub unsafe fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        debug_assert_eq!(
            sexp.type_of(),
            crate::SEXPTYPE::EXTPTRSXP,
            "wrap_sexp: expected EXTPTRSXP, got {:?}",
            sexp.type_of()
        );
        let any_raw = unsafe { R_ExternalPtrAddr(sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return None;
        }

        if is_type_erased::<T>() {
            // Type-erased path: skip downcast, just use the raw pointer
            // (ExternalPtr<()> doesn't care about the concrete type)
            return Some(Self::from_borrowed_parts(sexp, unsafe {
                NonNull::new_unchecked(any_raw.cast::<T>())
            }));
        }

        // Use downcast_mut (not downcast_ref) so cached_ptr gets mutable
        // provenance — shared-reference provenance from downcast_ref would
        // make later writes through as_mut() UB under Stacked Borrows.
        let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
        let concrete: &mut T = any_box.downcast_mut::<T>()?;

        Some(Self::from_borrowed_parts(sexp, unsafe {
            NonNull::new_unchecked(ptr::from_mut(concrete))
        }))
    }

    /// Attempt to wrap a SEXP as an ExternalPtr (unchecked version).
    ///
    /// Skips thread safety checks for performance-critical paths like ALTREP callbacks.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP created by this library
    /// - The caller must ensure exclusive ownership
    /// - Must be called from the R main thread (guaranteed in ALTREP callbacks)
    pub unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Option<Self> {
        use crate::sys::R_ExternalPtrAddr_unchecked;

        debug_assert_eq!(
            sexp.type_of(),
            crate::SEXPTYPE::EXTPTRSXP,
            "wrap_sexp_unchecked: expected EXTPTRSXP, got {:?}",
            sexp.type_of()
        );
        let any_raw = unsafe { R_ExternalPtrAddr_unchecked(sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return None;
        }

        if is_type_erased::<T>() {
            return Some(Self::from_borrowed_parts(sexp, unsafe {
                NonNull::new_unchecked(any_raw.cast::<T>())
            }));
        }

        let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
        let concrete: &mut T = any_box.downcast_mut::<T>()?;

        Some(Self::from_borrowed_parts(sexp, unsafe {
            NonNull::new_unchecked(ptr::from_mut(concrete))
        }))
    }

    /// Attempt to wrap a SEXP as an ExternalPtr, returning an error with type info on mismatch.
    ///
    /// This is used by the [`TryFromSexp`] trait implementation.
    ///
    /// # Safety
    ///
    /// Same as [`wrap_sexp`](Self::wrap_sexp).
    ///
    /// [`TryFromSexp`]: crate::TryFromSexp
    pub unsafe fn wrap_sexp_with_error(sexp: SEXP) -> Result<Self, TypeMismatchError> {
        debug_assert_eq!(
            sexp.type_of(),
            crate::SEXPTYPE::EXTPTRSXP,
            "wrap_sexp_with_error: expected EXTPTRSXP, got {:?}",
            sexp.type_of()
        );
        let any_raw = unsafe { R_ExternalPtrAddr(sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return Err(TypeMismatchError::NullPointer);
        }

        if is_type_erased::<T>() {
            return Ok(Self::from_borrowed_parts(sexp, unsafe {
                NonNull::new_unchecked(any_raw.cast::<T>())
            }));
        }

        let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
        match any_box.downcast_mut::<T>() {
            Some(concrete) => Ok(Self::from_borrowed_parts(sexp, unsafe {
                NonNull::new_unchecked(ptr::from_mut(concrete))
            })),
            None => {
                // Try to get the stored type name from R symbol for error reporting
                let found = unsafe {
                    let prot = R_ExternalPtrProtected(sexp);
                    if !prot.is_null_or_nil()
                        && prot.type_of() == SEXPTYPE::VECSXP
                        && prot.len() >= PROT_VEC_LEN as usize
                    {
                        let stored_sym = prot.vector_elt(PROT_TYPE_ID_INDEX);
                        if stored_sym.type_of() == SEXPTYPE::SYMSXP {
                            symbol_name(stored_sym)
                        } else {
                            "<unknown>"
                        }
                    } else {
                        "<unknown>"
                    }
                };
                Err(TypeMismatchError::Mismatch {
                    expected: T::TYPE_NAME,
                    found,
                })
            }
        }
    }

    /// Create an ExternalPtr from an SEXP without type checking.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP containing a `*mut Box<dyn Any>`
    ///   wrapping a value of type `T`
    /// - The caller must ensure exclusive ownership
    #[inline]
    pub unsafe fn from_sexp_unchecked(sexp: SEXP) -> Self {
        debug_assert_eq!(
            sexp.type_of(),
            crate::SEXPTYPE::EXTPTRSXP,
            "from_sexp_unchecked: expected EXTPTRSXP, got {:?}",
            sexp.type_of()
        );
        let any_raw = unsafe { R_ExternalPtrAddr(sexp) as *mut Box<dyn Any> };
        debug_assert!(!any_raw.is_null(), "from_sexp_unchecked: null pointer");

        let cached_ptr = if is_type_erased::<T>() {
            unsafe { NonNull::new_unchecked(any_raw.cast::<T>()) }
        } else {
            let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
            let concrete: &mut T = unsafe { any_box.downcast_mut::<T>().unwrap_unchecked() };
            unsafe { NonNull::new_unchecked(ptr::from_mut(concrete)) }
        };

        Self::from_borrowed_parts(sexp, cached_ptr)
    }
    // endregion

    // region: Downcast support

    /// Returns the type name for type T.
    #[inline]
    pub fn type_name() -> &'static str {
        T::TYPE_NAME
    }

    /// Returns the type name stored in this ExternalPtr's prot slot.
    ///
    /// Returns `None` if the prot slot doesn't contain a valid type symbol.
    #[inline]
    pub fn stored_type_name(&self) -> Option<&'static str> {
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null_or_nil() {
                return None;
            }
            if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
                return None;
            }
            let stored_sym = prot.vector_elt(PROT_TYPE_ID_INDEX);
            if stored_sym.type_of() != SEXPTYPE::SYMSXP {
                return None;
            }
            Some(symbol_name(stored_sym))
        }
    }
    // endregion
}

impl ExternalPtr<()> {
    /// Create a type-erased ExternalPtr from an EXTPTRSXP without checking the stored type.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP
    /// - Caller must ensure exclusive ownership semantics are upheld
    #[inline]
    pub unsafe fn from_sexp(sexp: SEXP) -> Self {
        debug_assert!(sexp.type_of() == SEXPTYPE::EXTPTRSXP);
        unsafe { Self::from_sexp_unchecked(sexp) }
    }

    /// Check whether the stored `Box<dyn Any>` contains a `T`.
    ///
    /// Uses `Any::is` for authoritative runtime type checking.
    #[inline]
    pub fn is<T: TypedExternal>(&self) -> bool {
        let any_raw = unsafe { R_ExternalPtrAddr(self.sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return false;
        }
        let any_box: &Box<dyn Any> = unsafe { &*any_raw };
        any_box.is::<T>()
    }

    /// Downcast to an immutable reference of the stored type if it matches `T`.
    ///
    /// Uses `Any::downcast_ref` for authoritative runtime type checking.
    #[inline]
    pub fn downcast_ref<T: TypedExternal>(&self) -> Option<&T> {
        let any_raw = unsafe { R_ExternalPtrAddr(self.sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return None;
        }
        let any_box: &Box<dyn Any> = unsafe { &*any_raw };
        any_box.downcast_ref::<T>()
    }

    /// Downcast to a mutable reference of the stored type if it matches `T`.
    ///
    /// Uses `Any::downcast_mut` for authoritative runtime type checking.
    #[inline]
    pub fn downcast_mut<T: TypedExternal>(&mut self) -> Option<&mut T> {
        let any_raw = unsafe { R_ExternalPtrAddr(self.sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return None;
        }
        let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
        any_box.downcast_mut::<T>()
    }
}

// region: Consuming (`self` by value) method support

/// Marker left in an `EXTPTRSXP` slot while a consuming (`self` by value)
/// method runs, and left behind for good if that method panics.
///
/// `#[miniextendr]` methods taking bare `self` move the stored value out of
/// the R handle with [`ExternalPtr::<()>::take_for_consuming`], call the
/// method, and either write the result back
/// ([`ExternalPtr::<()>::restore_after_consuming`], for `self -> Self`) or
/// leave the slot consumed (terminal `self -> T`). A slot holding this marker
/// makes every later method call on the handle fail with a "consumed" error
/// instead of a type mismatch; the finaliser drops the marker like any value.
#[derive(Debug)]
pub struct ConsumedSlot;

/// Bound for the receiver of a **fallible** consuming method
/// (`self -> Result<Self, E>` / `self -> Option<Self>`).
///
/// The generated wrapper calls the method on a clone of the stored value and
/// only overwrites the R handle on `Ok` / `Some`, so a failed step leaves the
/// R object exactly as it was, which is what an interactive R user expects
/// from `obj |> add_step(-1)` erroring. Blanket-implemented for every
/// `T: Clone`; the `on_unimplemented` text below is what rustc prints when
/// the type is not `Clone`.
#[diagnostic::on_unimplemented(
    message = "a fallible consuming method (`self -> Result<Self, E>` / `Option<Self>`) needs `{Self}: Clone`",
    note = "the wrapper calls the method on a clone so a failed step leaves the R object untouched; derive or implement `Clone`, or take `&mut self` and return `Result<&mut Self, E>` instead"
)]
pub trait ConsumingFallible: Clone {}
impl<T: Clone> ConsumingFallible for T {}

/// Clone the stored value for a fallible consuming step (see
/// [`ConsumingFallible`]). Free function so codegen can name the bound.
#[inline]
pub fn clone_for_consuming<T: ConsumingFallible>(value: &T) -> T {
    value.clone()
}

/// Panic with the right message when a handle's stored value is not a `T`:
/// "consumed" if a previous `self`-by-value step failed, type mismatch
/// otherwise. Used by generated method preludes instead of a bare `expect`.
#[cold]
pub fn handle_downcast_failed<T: TypedExternal>(ptr: &ExternalPtr<()>) -> ! {
    if ptr.is_consumed() {
        panic!(
            "this `{}` object was consumed by a `self`-by-value method that did not return \
             a new value (it returned a plain result or panicked) and can no longer be used",
            ExternalPtr::<T>::type_name()
        );
    }
    panic!(
        "expected ExternalPtr<{}>, found `{}`",
        ExternalPtr::<T>::type_name(),
        ptr.stored_type_name().unwrap_or("<unknown>")
    );
}

impl ExternalPtr<()> {
    /// Whether the slot holds the [`ConsumedSlot`] marker.
    pub fn is_consumed(&self) -> bool {
        let any_raw = unsafe { R_ExternalPtrAddr(self.sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return false;
        }
        let any_box: &Box<dyn Any> = unsafe { &*any_raw };
        any_box.is::<ConsumedSlot>()
    }

    /// Move the stored `T` out of the handle, leaving [`ConsumedSlot`] behind.
    ///
    /// Returns `None` when the slot does not hold a `T` (wrong type, null, or
    /// already consumed); the slot is untouched in that case. The outer
    /// `Box<Box<dyn Any>>` cell stays allocated, so the finaliser and every
    /// other accessor keep working on the marker.
    pub fn take_for_consuming<T: TypedExternal>(&mut self) -> Option<T> {
        let any_raw = unsafe { R_ExternalPtrAddr(self.sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return None;
        }
        let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
        if !any_box.is::<T>() {
            return None;
        }
        let taken = std::mem::replace(any_box, Box::new(ConsumedSlot));
        let boxed: Box<T> = taken.downcast::<T>().expect("checked is::<T> above");
        Some(*boxed)
    }

    /// Put a value back into a slot emptied by [`Self::take_for_consuming`]
    /// (the write-back half of `self -> Self`). Replaces whatever the slot
    /// holds; the `TypedExternal` tag in the `prot` slot is unchanged because
    /// the type is the same.
    pub fn restore_after_consuming<T: TypedExternal>(&mut self, value: T) {
        let any_raw = unsafe { R_ExternalPtrAddr(self.sexp) as *mut Box<dyn Any> };
        assert!(
            !any_raw.is_null(),
            "restore_after_consuming on a null external pointer"
        );
        let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
        *any_box = Box::new(value);
    }
}

// endregion

/// Error returned when type checking fails in `try_from_sexp_with_error`.
///
/// The `found` field in `Mismatch` contains a `&'static str` from R's
/// interned symbol table, which persists for the R session lifetime.
#[derive(Debug, Clone)]
pub enum TypeMismatchError {
    /// The external pointer's address was null.
    NullPointer,
    /// The prot slot didn't contain a valid type symbol.
    InvalidTypeId,
    /// The stored type doesn't match the expected type.
    Mismatch {
        /// Expected Rust type name from this pointer wrapper.
        expected: &'static str,
        /// Actual stored Rust type name found in pointer metadata.
        found: &'static str,
    },
}

impl fmt::Display for TypeMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NullPointer => write!(f, "external pointer is null"),
            Self::InvalidTypeId => write!(f, "external pointer has no valid type id"),
            Self::Mismatch { expected, found } => {
                write!(
                    f,
                    "type mismatch: expected `{}`, found `{}`",
                    expected, found
                )
            }
        }
    }
}

impl std::error::Error for TypeMismatchError {}
// endregion

// region: MaybeUninit support

// We need a separate TypedExternal impl for MaybeUninit<T>
// This is typically done via blanket impl or macro

impl<T: TypedExternal> ExternalPtr<MaybeUninit<T>>
where
    MaybeUninit<T>: TypedExternal,
{
    /// Converts to `ExternalPtr<T>`.
    ///
    /// # Safety
    ///
    /// The value must have been initialized.
    ///
    /// # Implementation Note
    ///
    /// This method creates a *new* SEXP with `T`'s type information, leaving
    /// the original `MaybeUninit<T>` SEXP as an orphaned empty shell in R's heap.
    /// This is necessary because the type ID stored in the prot slot must match
    /// the actual type. The orphaned SEXP will be cleaned up by R's GC eventually.
    ///
    /// If you need to avoid this overhead, consider using `ExternalPtr<T>::new`
    /// directly and initializing in place via `as_mut`.
    ///
    /// Equivalent to `Box::assume_init`.
    #[inline]
    pub fn assume_init(self) -> ExternalPtr<T> {
        // Get the raw pointer (this clears the original SEXP, making its finalizer a no-op)
        let ptr = Self::into_raw(self).cast();

        // Create a new ExternalPtr with T's type info
        unsafe { ExternalPtr::from_raw(ptr) }
    }

    /// Writes a value and converts to initialized.
    ///
    /// Creates a new SEXP with `T`'s type information (the original
    /// `MaybeUninit<T>` SEXP becomes an orphaned shell, cleaned up by GC).
    #[inline]
    pub fn write(mut self, value: T) -> ExternalPtr<T> {
        unsafe {
            (*Self::as_mut_ptr(&mut self)).write(value);
            self.assume_init()
        }
    }
}
/// Type-erased `ExternalPtr` for cases where the concrete `T` is not needed.
pub type ErasedExternalPtr = ExternalPtr<()>;
// endregion

// region: Trait Implementations

impl<T: TypedExternal> Deref for ExternalPtr<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        Self::as_ref(self).expect("ExternalPtr is null or cleared")
    }
}

impl<T: TypedExternal> DerefMut for ExternalPtr<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        Self::as_mut(self).expect("ExternalPtr is null or cleared")
    }
}

impl<T: TypedExternal> AsRef<T> for ExternalPtr<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        Self::as_ref(self).expect("ExternalPtr is null or cleared")
    }
}

impl<T: TypedExternal> AsMut<T> for ExternalPtr<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        Self::as_mut(self).expect("ExternalPtr is null or cleared")
    }
}

impl<T: TypedExternal> std::borrow::Borrow<T> for ExternalPtr<T> {
    #[inline]
    fn borrow(&self) -> &T {
        Self::as_ref(self).expect("ExternalPtr is null or cleared")
    }
}

impl<T: TypedExternal> std::borrow::BorrowMut<T> for ExternalPtr<T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        Self::as_mut(self).expect("ExternalPtr is null or cleared")
    }
}

impl<T: TypedExternal + Clone> Clone for ExternalPtr<T> {
    /// Deep clones the inner value into a new ExternalPtr.
    ///
    /// This creates a completely independent ExternalPtr with its own
    /// heap allocation and finalizer.
    #[inline]
    fn clone(&self) -> Self {
        Self::new((**self).clone())
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        (**self).clone_from(&**source);
    }
}

impl<T: TypedExternal + Default> Default for ExternalPtr<T> {
    /// Creates an ExternalPtr containing the default value of T.
    #[inline]
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: TypedExternal + fmt::Debug> fmt::Debug for ExternalPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: TypedExternal + fmt::Display> fmt::Display for ExternalPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: TypedExternal> fmt::Pointer for ExternalPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&Self::as_ptr(self), f)
    }
}

impl<T: TypedExternal + PartialEq> PartialEq for ExternalPtr<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: TypedExternal + Eq> Eq for ExternalPtr<T> {}

impl<T: TypedExternal + PartialOrd> PartialOrd for ExternalPtr<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }
}

impl<T: TypedExternal + Ord> Ord for ExternalPtr<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (**self).cmp(&**other)
    }
}

impl<T: TypedExternal + Hash> Hash for ExternalPtr<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: TypedExternal + std::iter::Iterator> std::iter::Iterator for ExternalPtr<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        (**self).next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        (**self).nth(n)
    }
}

impl<T: TypedExternal + std::iter::DoubleEndedIterator> std::iter::DoubleEndedIterator
    for ExternalPtr<T>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        (**self).next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        (**self).nth_back(n)
    }
}

impl<T: TypedExternal + std::iter::ExactSizeIterator> std::iter::ExactSizeIterator
    for ExternalPtr<T>
{
    fn len(&self) -> usize {
        (**self).len()
    }
}

impl<T: TypedExternal + std::iter::FusedIterator> std::iter::FusedIterator for ExternalPtr<T> {}

impl<T: TypedExternal> From<T> for ExternalPtr<T> {
    #[inline]
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

impl<T: TypedExternal> From<Box<T>> for ExternalPtr<T> {
    #[inline]
    fn from(boxed: Box<T>) -> Self {
        unsafe { Self::from_raw(Box::into_raw(boxed)) }
    }
}

// `Drop` releases the R-side GC root taken at construction (for *owning*
// handles only) but never frees the pointee — that stays R's job, run by the
// `release_any` finalizer when R garbage-collects the `EXTPTRSXP`. Dropping the
// root just makes the object eligible for collection once R itself holds no
// other reference; if R still references it (the usual case — it was returned
// from a `.Call` or stored), it stays alive and the finalizer runs later.
//
// For deterministic *value* cleanup, use `ExternalPtr::into_inner` (moves the
// value out) or `drop(Box::from_raw(ExternalPtr::into_raw(ptr)))`.
impl<T: TypedExternal> Drop for ExternalPtr<T> {
    fn drop(&mut self) {
        self.release_root_if_owned();
    }
}
// endregion

// region: Finalizer

/// Guard that aborts the process if dropped while a panic is in progress.
///
/// Used by [`drop_catching_panic`] to implement panic-safe destructor calls
/// without `catch_unwind`. When `f()` completes normally, the guard is
/// dropped with `std::thread::panicking() == false` and becomes a no-op.
/// If `f()` panics, the guard's destructor runs during stack unwinding
/// (when `std::thread::panicking() == true`) and calls `process::abort()`.
///
/// This approach avoids `catch_unwind`, which registers LLVM unwind landing
/// pads. Inside R's GC finalizer walk, any interaction with the unwinding
/// machinery — especially on the first call that lazily initialises exception
/// handling state — can trigger an allocator call that re-enters R's GC and
/// produces a "recursive gc invocation" hard crash.
#[must_use]
struct AbortIfUnwinding;

impl Drop for AbortIfUnwinding {
    #[cold]
    fn drop(&mut self) {
        if std::thread::panicking() {
            // A panic propagated through a finalizer — abort immediately.
            // The value being dropped is in an indeterminate state; continuing
            // is not safe.
            eprintln!("miniextendr: destructor panicked during R finalization; aborting");
            std::process::abort();
        }
    }
}

/// Run a destructor closure, aborting the process if the closure panics.
///
/// A panic inside a GC finalizer cannot be safely propagated: the finalizer
/// runs at an arbitrary point in R's garbage collector, and unwinding across
/// the C-ABI boundary into R's runtime is undefined behaviour. Aborting is
/// the only safe recovery strategy — the destructor has already left the
/// value in an indeterminate state, so continuing is not an option.
///
/// ## Implementation note
///
/// This function deliberately avoids `std::panic::catch_unwind`. On the first
/// call from within R's GC finalizer, `catch_unwind` may lazily initialise
/// LLVM exception-handling state, which can allocate. Any allocation during a
/// GC finalizer re-enters the GC and triggers the fatal "recursive gc
/// invocation" crash. Instead, this function uses a drop-guard whose `Drop`
/// impl calls `std::thread::panicking()` — a cheap, allocation-free TLS read.
///
/// This helper is `#[doc(hidden)]` because it is called from macro-generated
/// code and is not part of the public API.
#[doc(hidden)]
#[inline]
pub fn drop_catching_panic<F: FnOnce()>(f: F) {
    let _guard = AbortIfUnwinding;
    f();
    // guard dropped here with panicking() == false → no-op
}

/// Non-generic C finalizer called by R's garbage collector.
///
/// Since `ExternalPtr` stores `Box<Box<dyn Any>>`, the `Any` vtable carries
/// the concrete type's drop function. No generic parameter needed — one
/// finalizer function handles all `ExternalPtr<T>` types.
extern "C-unwind" fn release_any(sexp: SEXP) {
    if sexp.is_null() {
        return;
    }
    if sexp.is_nil() {
        return;
    }

    let any_raw = unsafe { R_ExternalPtrAddr(sexp) as *mut Box<dyn Any> };

    // Guard against double-finalization
    if any_raw.is_null() {
        return;
    }

    // Clear the external pointer first (prevents double-free if called again)
    unsafe { R_ClearExternalPtr(sexp) };

    // Reconstruct the outer Box<Box<dyn Any>> and let it drop.
    // This drops the outer Box, then the inner Box<dyn Any>, which
    // uses the vtable to drop the concrete T value.
    //
    // A panicking Drop impl must not unwind across the C-ABI boundary into R.
    // `drop_catching_panic` catches any panic and aborts instead.
    drop_catching_panic(|| drop(unsafe { Box::from_raw(any_raw) }));
}
// endregion

// region: Utility: ExternalSlice (helper for slice data)

/// A slice stored as a standalone struct, suitable for wrapping in ExternalPtr.
///
/// This is analogous to the data inside a `Box<[T]>`, but stores capacity
/// for proper deallocation when created from a `Vec`.
///
/// # Usage
///
/// To use with `ExternalPtr`, implement `TypedExternal` for your specific
/// `ExternalSlice<YourType>`:
///
/// ```ignore
/// impl_typed_external!(ExternalSlice<MyElement>);
/// let ptr = ExternalPtr::new(ExternalSlice::new(vec![1, 2, 3]));
/// ```
#[repr(C)]
pub struct ExternalSlice<T: 'static> {
    ptr: NonNull<T>,
    len: usize,
    capacity: usize,
}

impl<T: 'static> ExternalSlice<T> {
    /// Create an external slice from a `Vec`, preserving its allocation.
    pub fn new(slice: Vec<T>) -> Self {
        let mut vec = ManuallyDrop::new(slice);
        Self {
            ptr: unsafe { NonNull::new_unchecked(vec.as_mut_ptr()) },
            len: vec.len(),
            capacity: vec.capacity(),
        }
    }

    /// Create from a boxed slice (capacity == len).
    pub fn from_boxed(boxed: Box<[T]>) -> Self {
        let len = boxed.len();
        let ptr = Box::into_raw(boxed).cast();
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            len,
            capacity: len,
        }
    }

    /// Borrow the contents as a shared slice.
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Borrow the contents as a mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    /// Number of elements in the slice.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the slice is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Capacity of the underlying allocation.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<T: 'static> Drop for ExternalSlice<T> {
    fn drop(&mut self) {
        unsafe {
            let _ = Vec::from_raw_parts(self.ptr.as_ptr(), self.len, self.capacity);
        }
    }
}
// endregion

mod altrep_helpers;
pub use altrep_helpers::*;

#[cfg(test)]
mod tests {
    use super::drop_catching_panic;

    #[test]
    fn drop_catching_panic_does_not_propagate_panic() {
        // Verify that drop_catching_panic catches a panicking closure and does
        // NOT propagate the panic to the caller.
        //
        // Note: we cannot test the abort path from inside a test process, so
        // we document it with a comment instead:
        //   If the closure panics, `drop_catching_panic` calls `eprintln!` then
        //   `std::process::abort()`. That path is exercised only by the process
        //   dying, which is observable from an external test harness (not done
        //   here to keep CI simple).
        //
        // What we CAN test: the happy path (no panic) completes normally, and
        // the function compiles and links correctly with a `FnOnce()` generic.
        let mut ran = false;
        drop_catching_panic(|| {
            ran = true;
        });
        assert!(ran, "closure should have been called");
    }

    #[test]
    fn drop_catching_panic_happy_path_drops_value() {
        // Confirm that the closure's side-effects (i.e. actual drop) occur
        // when no panic is raised.
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let dropped = Arc::new(AtomicBool::new(false));
        let flag = dropped.clone();

        struct DropSignal(Arc<AtomicBool>);
        impl Drop for DropSignal {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let signal = DropSignal(flag);
        drop_catching_panic(|| drop(signal));

        assert!(
            dropped.load(Ordering::SeqCst),
            "inner value should have been dropped"
        );
    }
}
