//! Safe API for R's `R_UnwindProtect`
//!
//! This module provides [`with_r_unwind_protect`] for handling R errors with Rust cleanup.
//! It automatically runs Rust destructors when R errors occur.
//!
//! **Important**: R uses `longjmp` for error handling, which normally bypasses Rust destructors.
//! Use this API to ensure cleanup happens even when R errors occur.
//!
//! ## When to reach for this
//!
//! - **Calling R APIs that can error from a body you wrote yourself**
//!   (custom ALTREP, custom connection trampoline, hand-rolled FFI shim).
//!   Wrap the R-calling section in [`with_r_unwind_protect`] so Rust
//!   destructors run if R longjmps.
//! - **Inside a [`with_r_unwind_protect`] body it is safe to use `*_unchecked`
//!   variants of the R FFI** — see the [`crate::sys`] module doc. The lint
//!   **MXL301** recognises this as one of the three contexts where bypassing
//!   the main-thread assertion is valid (the other two being ALTREP callbacks
//!   and [`crate::worker::with_r_thread`] bodies).
//!
//! ## You probably don't need this from a `#[miniextendr]` body
//!
//! The proc-macro already wraps every function and method in a guard that
//! converts panics into the tagged-condition transport ([`crate::error_value`]).
//! Returning `Result::Err`, `Option::None`, or calling `panic!()` /
//! [`crate::error!`] / [`crate::warning!`] / [`crate::message!`] is the
//! idiomatic path. Direct [`with_r_unwind_protect`] use inside that body is
//! almost always wrong — you'd be nesting an `R_UnwindProtect` inside another
//! `R_UnwindProtect`, paying the longjmp-leak cost twice (see "Leaks" below).
//!
//! ## Don't use `Rf_error`
//!
//! `Rf_error` and `Rf_errorcall` longjmp directly, skipping every Rust
//! destructor on the stack. The lint **MXL300** forbids them in user code.
//! Panic instead (or call [`crate::error!`]) and the framework raises the
//! corresponding R condition for you.
//!
//! ## Leaks
//!
//! On the R longjmp path (when R unwinds out of the protected body),
//! `with_r_unwind_protect` leaks ~8 bytes (an `RErrorMarker` + `Box` header)
//! because the cleanup handler can't reclaim them via
//! `Box::from_raw`. Regular Rust panics from inside the body don't leak.
//! This is the cost MXL300 is buying off: every direct `Rf_error()` would
//! incur the same leak with no observability.
//!
//! ## Log drain
//!
//! Every call to `with_r_unwind_protect` (and its variants) drains the
//! cross-thread log queue via the crate-private `drain_log_queue_if_available`
//! helper before returning or re-raising an R error. This ensures that records
//! buffered by worker threads are flushed to R's console on every FFI exit —
//! including error paths.
//!
//! ## Cross references
//!
//! - [`crate::worker::with_r_thread`] — routes a closure to R's main thread.
//! - [`crate::ffi_guard`] — unified panic-catching trampoline that consumes
//!   `with_r_unwind_protect_sourced` for ALTREP `RUnwind` mode.
//! - [`crate::error_value`] / [`mod@crate::condition`] — panic → R condition
//!   transport.
use std::{
    any::Any,
    borrow::Cow,
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::OnceLock,
};

// region: raise_rust_condition_via_stop — Approach 3 for ALTREP RUnwind path

/// Cached `stop` symbol (permanently interned via `Rf_install`).
fn stop_sym() -> crate::SEXP {
    static CACHE: OnceLock<crate::SEXP> = OnceLock::new();
    *CACHE.get_or_init(|| unsafe { crate::sys::Rf_install(c"stop".as_ptr()) })
}

/// Raise an R condition with `rust_*` class layering by evaluating
/// `stop(structure(list(message = msg, call = call, ...data), class = c(...)))`.
///
/// This is **Approach 3** from the issue-345 plan: the `Rf_eval(stop(...))` pattern
/// that works in any context where there is no outer R wrapper to inspect a tagged SEXP.
/// It is the only viable option for ALTREP callbacks, which are invoked directly by
/// R's runtime (no `.Call` frame, no R wrapper).
///
/// The `stop()` call longjmps, so this function never returns — declared `-> !`.
///
/// ## Class layering
///
/// - With `class = ["my_class"]`, the resulting R condition has class:
///   `c("my_class", "rust_error", "simpleError", "error", "condition")`; several
///   classes are prepended in order.
/// - Without a custom class: `c("rust_error", "simpleError", "error", "condition")`.
///
/// ## Structured `data` (issue #996 path 2)
///
/// When `data` is `Some`, each `(name, value)` pair is spliced directly into
/// the condition list *after* `message`/`call` — mirroring how the
/// tagged-transport path's `.miniextendr_raise_condition` R helper
/// (`utils::modifyList`) layers the macros' `data = ...` payload onto the base
/// condition fields (see `crate::error_value`). `message`/`call` are kept
/// first so `$`'s first-match semantics protect `conditionMessage()` /
/// `conditionCall()` even if a data field happens to share one of those names.
/// `None` produces the original 2-element `(message, call)` list.
///
/// ## MXL300 compliance
///
/// This function raises an R error via `Rf_eval(stop(...))`, not via direct
/// `Rf_error`/`Rf_errorcall`. MXL300 does not flag `Rf_eval`.
///
/// # Safety
///
/// Must be called from R's main thread inside an `R_UnwindProtect` cleanup
/// or equivalent context where R longjmps are safe. In practice, always called
/// from `with_r_unwind_protect_sourced` on the ALTREP guard path.
pub(crate) unsafe fn raise_rust_condition_via_stop(
    message: &str,
    class: &[String],
    call: Option<crate::SEXP>,
    data: Option<crate::condition::ConditionData>,
) -> ! {
    use crate::sexp_types::CE_UTF8;
    use crate::sys::{R_BaseEnv, Rf_allocVector, Rf_eval, Rf_lang2, Rf_mkCharCE, Rf_protect};
    use crate::{IntoR, SEXP, SEXPTYPE, SexpExt};

    unsafe {
        // Build the class vector: c(<custom classes…>, "rust_error", "simpleError", "error", "condition")
        let base_classes: &[&std::ffi::CStr] =
            &[c"rust_error", c"simpleError", c"error", c"condition"];
        let class_count = base_classes.len() + class.len();

        let class_vec = Rf_allocVector(SEXPTYPE::STRSXP, class_count as isize);
        Rf_protect(class_vec);

        let mut idx = 0isize;
        for custom in class {
            let custom_cstr = std::ffi::CString::new(custom.as_str())
                .unwrap_or_else(|_| std::ffi::CString::new("rust_error").unwrap());
            let custom_charsxp = Rf_mkCharCE(custom_cstr.as_ptr(), CE_UTF8);
            class_vec.set_string_elt(idx, custom_charsxp);
            idx += 1;
        }
        for base in base_classes {
            let charsxp = crate::cached_class::permanent_charsxp(base);
            class_vec.set_string_elt(idx, charsxp);
            idx += 1;
        }

        // Build the message SEXP
        let msg_cstr = std::ffi::CString::new(message)
            .unwrap_or_else(|_| std::ffi::CString::new("<invalid error message>").unwrap());
        let msg_charsxp = Rf_mkCharCE(msg_cstr.as_ptr(), CE_UTF8);
        let msg_sexp = SEXP::scalar_string(msg_charsxp);
        Rf_protect(msg_sexp);

        let call_sexp = call.unwrap_or(SEXP::nil());

        // Build a named list: list(message = msg, call = call_sexp, ...data).
        // `data_len` extra slots hold the spliced `data =` fields (issue #996
        // path 2); with no data this is the original 2-element list.
        let data_len = data.as_ref().map_or(0, |fields| fields.len());
        let total_len = 2 + data_len;

        let err_list = Rf_allocVector(SEXPTYPE::VECSXP, total_len as isize);
        Rf_protect(err_list);
        err_list.set_vector_elt(0, msg_sexp);
        err_list.set_vector_elt(1, call_sexp);

        // Set names: c("message", "call", <data field names>...)
        let names_vec = Rf_allocVector(SEXPTYPE::STRSXP, total_len as isize);
        Rf_protect(names_vec);
        names_vec.set_string_elt(0, crate::cached_class::permanent_charsxp(c"message"));
        names_vec.set_string_elt(1, crate::cached_class::permanent_charsxp(c"call"));

        // PROTECT discipline: err_list and names_vec are already protected
        // above. Each data field's materialised value is stored into the
        // protected err_list immediately (rooting it) before the next
        // allocation (its name CHARSXP) — same discipline as
        // `make_rust_condition_value_with_data` in `crate::error_value`.
        if let Some(fields) = data {
            for (i, (name, value)) in fields.into_iter().enumerate() {
                let idx = (2 + i) as isize;
                let value_sexp = value.into_sexp();
                err_list.set_vector_elt(idx, value_sexp);
                let name_cstr = std::ffi::CString::new(name)
                    .unwrap_or_else(|_| std::ffi::CString::new("<invalid name>").unwrap());
                let name_charsxp = Rf_mkCharCE(name_cstr.as_ptr(), CE_UTF8);
                names_vec.set_string_elt(idx, name_charsxp);
            }
        }
        err_list.set_names(names_vec);

        // Set the class attribute directly (no structure() call needed)
        err_list.set_class(class_vec);

        // Build stop(err_list) as a language object: lang2(stop_sym, err_list)
        // stop() accepts a condition object directly
        let stop_call = Rf_lang2(stop_sym(), err_list);
        Rf_protect(stop_call);

        // Rf_eval(stop_call, R_BaseEnv) longjmps — never returns
        // The protect stack is cleaned up by R's longjmp unwind
        Rf_eval(stop_call, R_BaseEnv);

        // Never reached — Rf_eval(stop(...), ...) always longjmps
        std::hint::unreachable_unchecked()
    }
}

// endregion

use crate::sys::{self, R_ContinueUnwind, R_UnwindProtect_C_unwind};
use crate::{Rboolean, SEXP};

/// Global continuation token for R_UnwindProtect.
///
/// Using a single global token instead of thread-local tokens avoids leaking
/// one token per thread that uses `with_r_unwind_protect`.
///
/// # Safety
///
/// The token is created and preserved once during first use. It remains valid
/// for the entire R session.
static R_CONTINUATION_TOKEN: OnceLock<SEXP> = OnceLock::new();

/// Get or create the global continuation token.
///
/// This is public for use by the worker module.
pub(crate) fn get_continuation_token() -> SEXP {
    *R_CONTINUATION_TOKEN.get_or_init(|| {
        // The continuation token must be created on R's main thread
        // (R_MakeUnwindCont is an R API call). OnceLock ensures it is
        // only created once and safely shared.
        unsafe {
            let token = sys::R_MakeUnwindCont();
            sys::R_PreserveObject(token);
            token
        }
    })
}

/// Panic payload whose message is already final (location folded, or
/// deliberately location-free): downstream folds must use it verbatim and
/// must NOT append the current thread's recorded panic location (#1245).
///
/// Produced by `worker::route_to_main_thread`'s re-panic when a `with_r_thread`
/// closure panics on the main thread: the main-thread stringify point already
/// folded the *true* origin location into the message before it crossed back
/// to the worker, so the worker's own re-panic (needed to unwind out of
/// `run_on_worker`) must carry that message forward untouched rather than
/// re-fold its own relay call site on top.
pub(crate) struct PreLocatedPanic(pub(crate) String);

/// Extract a message from a panic payload.
///
/// Handles `&str`, `String`, `&String`, and `PreLocatedPanic` payloads
/// consistently. The borrowed variants are returned as `Cow::Borrowed`, so the
/// common `panic!("literal")` case avoids the heap allocation that a `String`
/// return would force. Unrecognised payload types fall back to a
/// `Cow::Borrowed` static string.
///
/// Call `.into_owned()` (or `.to_string()`) at sites that need an owned
/// `String`.
pub fn panic_payload_to_string(payload: &(dyn Any + Send)) -> Cow<'_, str> {
    if let Some(&s) = payload.downcast_ref::<&str>() {
        Cow::Borrowed(s)
    } else if let Some(s) = payload.downcast_ref::<String>() {
        Cow::Borrowed(s.as_str())
    } else if let Some(s) = payload.downcast_ref::<&String>() {
        Cow::Borrowed(s.as_str())
    } else if let Some(pre) = payload.downcast_ref::<PreLocatedPanic>() {
        Cow::Borrowed(pre.0.as_str())
    } else {
        Cow::Borrowed("unknown panic")
    }
}

/// Stringify a panic payload and fold in the Rust source location the panic
/// hook recorded on the *current* thread, producing the final R-facing message.
///
/// Returns `panic_payload_to_string(payload)` with a `\n(at file:line)` suffix
/// when [`crate::backtrace::take_last_panic_location`] has a location for this
/// thread, otherwise the bare message. The location is captured in the process
/// panic hook (`backtrace.rs`), which fires on the panicking thread — so this
/// **must be called on the same thread that ran the panicking closure** (main
/// for main-thread `#[miniextendr]` fns; the worker for worker-dispatched fns).
///
/// Only the *generic panic* path uses this. `error!`/`warning!`/`message!`/
/// `condition!` (and `Result::Err` / `Option::None`) travel the typed
/// `RCondition` / tagged-value branches and are deliberately left byte-for-byte
/// unchanged — they carry no location suffix.
pub(crate) fn panic_message_with_location(payload: &(dyn Any + Send)) -> String {
    let msg = panic_payload_to_string(payload);
    match crate::backtrace::take_last_panic_location() {
        Some((file, line)) => format!("{msg}\n(at {file}:{line})"),
        None => msg.into_owned(),
    }
}

// region: Log drain integration

/// Drain the cross-thread log queue if the `log` feature is enabled.
///
/// This is called at every exit point of `run_r_unwind_protect` (normal
/// return, Rust panic, and immediately before `R_ContinueUnwind`) so that
/// worker-thread log records always reach R's console before the FFI call
/// returns or re-raises an R error.
///
/// When the `log` feature is disabled this compiles to a no-op; there is
/// no runtime overhead.
#[inline]
fn drain_log_queue_if_available() {
    #[cfg(feature = "log")]
    crate::optionals::log_impl::drain_log_queue();
}

// endregion

/// Core R_UnwindProtect wrapper. Returns `Ok(result)` on success,
/// `Err(payload)` on Rust panic, or diverges via `R_ContinueUnwind` on R longjmp.
///
/// Handles: CallData boxing, trampoline, cleanup handler, continuation token,
/// `Box::from_raw` reclamation on all non-diverging paths.
///
/// Drains the cross-thread log queue (when the `log` feature is enabled) at
/// each exit point so worker-thread records reach R's console before the FFI
/// boundary is crossed.
fn run_r_unwind_protect<F, R>(f: F) -> Result<R, Box<dyn Any + Send>>
where
    F: FnOnce() -> R,
{
    /// Marker type for R errors caught by R_UnwindProtect's cleanup handler.
    struct RErrorMarker;

    struct CallData<F, R> {
        f: Option<F>,
        result: Option<R>,
        panic_payload: Option<Box<dyn Any + Send>>,
    }

    unsafe extern "C-unwind" fn trampoline<F, R>(data: *mut c_void) -> SEXP
    where
        F: FnOnce() -> R,
    {
        assert!(!data.is_null(), "trampoline: data pointer is null");
        let data = unsafe { &mut *data.cast::<CallData<F, R>>() };
        let f = data.f.take().expect("trampoline: closure already consumed");

        match catch_unwind(AssertUnwindSafe(f)) {
            Ok(result) => {
                data.result = Some(result);
                crate::SEXP::nil()
            }
            Err(payload) => {
                data.panic_payload = Some(payload);
                crate::SEXP::nil()
            }
        }
    }

    unsafe extern "C-unwind" fn cleanup_handler(_data: *mut c_void, jump: Rboolean) {
        if jump != Rboolean::FALSE {
            // R is about to longjmp - trigger a Rust panic so we can unwind properly
            std::panic::panic_any(RErrorMarker);
        }
    }

    unsafe {
        let token = get_continuation_token();

        let data = Box::into_raw(Box::new(CallData::<F, R> {
            f: Some(f),
            result: None,
            panic_payload: None,
        }));

        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            R_UnwindProtect_C_unwind(
                Some(trampoline::<F, R>),
                data.cast(),
                Some(cleanup_handler),
                std::ptr::null_mut(),
                token,
            )
        }));

        let mut data = Box::from_raw(data);

        match panic_result {
            Ok(_) => {
                // Check if trampoline caught a panic
                if let Some(payload) = data.panic_payload.take() {
                    drop(data);
                    // Drain worker-thread log records before returning the panic
                    // payload to the caller (which will convert it to an R error).
                    drain_log_queue_if_available();
                    Err(payload)
                } else {
                    // Normal completion - return the result
                    let result = data
                        .result
                        .take()
                        .expect("result not set after successful completion");
                    drop(data);
                    // Drain worker-thread log records on the normal success path.
                    drain_log_queue_if_available();
                    Ok(result)
                }
            }
            Err(payload) => {
                // Drop data first to run destructors
                drop(data);
                // Check if this was an R error or a Rust panic
                if payload.downcast_ref::<RErrorMarker>().is_some() {
                    // R error - drain log records before re-raising so worker
                    // thread output is not lost even on error exits.
                    drain_log_queue_if_available();
                    // Continue R's unwind (diverges, never returns)
                    R_ContinueUnwind(token);
                } else {
                    // Rust panic — drain before returning the payload.
                    drain_log_queue_if_available();
                    Err(payload)
                }
            }
        }
    }
}

/// Execute a closure with R unwind protection, raising any Rust panic as an R
/// error via `Rf_eval(stop(structure(...)))`.
///
/// If the closure panics, the panic is caught and converted to an R error
/// (longjmp) with `rust_*` class layering. If R raises an error (longjmp), all
/// Rust RAII resources are properly dropped before R continues unwinding.
///
/// **This is NOT the user-facing path for `#[miniextendr]` functions.** That
/// path is [`with_r_unwind_protect`], which returns a tagged SEXP instead of
/// longjmping (the macro-generated R wrapper raises the structured condition).
///
/// This raising-variant exists for guard sites that have no R wrapper between
/// them and R's runtime:
/// - ALTREP `RUnwind` guard callbacks (via the crate-private
///   `with_r_unwind_protect_sourced`)
/// - FFI guard tests / benchmarks exercising the raw `R_UnwindProtect` mechanism
///
/// In those contexts there is no consumer-side R wrapper to inspect a tagged
/// SEXP. Panics are routed through `raise_rust_condition_via_stop` so they
/// still receive `rust_*` class layering (issue #345). Trait-ABI shims use a
/// separate SEXP-returning variant ([`with_r_unwind_protect_shim`]) that
/// re-panics at the View boundary.
///
/// # Arguments
///
/// * `f` - The closure to execute
/// * `call` - Optional R call SEXP for better error messages
pub fn with_r_unwind_protect_or_raise<F, R>(f: F, call: Option<SEXP>) -> R
where
    F: FnOnce() -> R,
{
    with_r_unwind_protect_sourced(f, call, crate::panic_telemetry::PanicSource::UnwindProtect)
}

/// Like [`with_r_unwind_protect_or_raise`], but reports panics with a custom
/// `PanicSource`.
///
/// Used by `guarded_altrep_call` so that panics inside ALTREP callbacks with
/// `AltrepGuard::RUnwind` are still attributed to `PanicSource::Altrep`.
///
/// Handles [`crate::condition::RCondition`] payloads:
///
/// - `RCondition::Error` — routes through [`raise_rust_condition_via_stop`] which
///   `Rf_eval`s `stop(structure(..., class = c("rust_error", ...)))`. This gives
///   full `rust_*` class layering even in ALTREP callback context where there is
///   no R wrapper to inspect a tagged SEXP (Approach 3 from the issue-345 plan).
///   Custom `class = "..."` from `error!()` is preserved in the class vector.
///
/// - `Warning`, `Message`, `Condition` — convert to a plain R error with a
///   diagnostic message. `warning!()`/`message!()` from ALTREP context cannot
///   suspend execution for non-fatal signals; documented limitation.
pub(crate) fn with_r_unwind_protect_sourced<F, R>(
    f: F,
    call: Option<SEXP>,
    source: crate::panic_telemetry::PanicSource,
) -> R
where
    F: FnOnce() -> R,
{
    match run_r_unwind_protect(f) {
        Ok(result) => result,
        Err(payload) => {
            // region: RCondition recognition for the raising-variant path
            if payload.is::<crate::condition::RCondition>() {
                // Take ownership so `data` (issue #996 path 2) can be moved into
                // `raise_rust_condition_via_stop` without cloning — same idiom as
                // `with_r_unwind_protect_shim`.
                let cond = *payload
                    .downcast::<crate::condition::RCondition>()
                    .expect("checked is::<RCondition> above");
                match cond {
                    crate::condition::RCondition::Error {
                        message,
                        class,
                        data,
                    } => {
                        // Approach 3 (issue-345): raise via Rf_eval(stop(structure(...)))
                        // so tryCatch(rust_error = h, ...) and tryCatch(my_class = h, ...)
                        // both match. No R wrapper needed. `data` fields are spliced in
                        // too (issue #996 path 2) — previously silently dropped here.
                        crate::panic_telemetry::fire(&message, source);
                        unsafe { raise_rust_condition_via_stop(&message, &class, call, data) }
                    }
                    crate::condition::RCondition::Warning { .. }
                    | crate::condition::RCondition::Message { .. }
                    | crate::condition::RCondition::Condition { .. } => {
                        // warning!/message!/condition! cannot be cleanly raised from ALTREP
                        // context (no mechanism to suspend execution for non-fatal signals).
                        // Documented degradation: convert to a plain R error with a fixed
                        // diagnostic, but route through `raise_rust_condition_via_stop` so
                        // the resulting error gets `rust_error` class layering — consistent
                        // with the generic-panic branch a few lines below (issue #366).
                        // The data fields (if any) are dropped along with everything else
                        // about the original kind — this branch already discards message.
                        let msg = "warning!/message!/condition! from ALTREP callback context \
                                   cannot be raised as non-fatal signals; use error!() instead. \
                                   This context has no R wrapper to handle signal restart.";
                        crate::panic_telemetry::fire(msg, source);
                        unsafe { raise_rust_condition_via_stop(msg, &[], call, None) }
                    }
                }
            } else {
                // Generic panic — no class layering, plain error string plus the
                // `(at file:line)` suffix folded from the panic hook (this branch
                // runs on the panicking thread for the ALTREP/FFI-guard path).
                // Fire telemetry and raise via Approach 3 with rust_error class so
                // tryCatch(rust_error = h, ...) matches even for plain panics.
                let msg = panic_message_with_location(payload.as_ref());
                crate::panic_telemetry::fire(&msg, source);
                unsafe { raise_rust_condition_via_stop(&msg, &[], call, None) }
            }
            // endregion
        }
    }
}

/// Like [`with_r_unwind_protect`], but tailored for trait-ABI vtable shims.
///
/// Same tagged-SEXP behaviour as [`with_r_unwind_protect`], but intended for
/// shim functions that have no R wrapper of their own. The tagged SEXP is
/// returned to the View method wrapper, which calls
/// [`crate::condition::repanic_if_rust_error`] to re-panic with the
/// reconstructed [`crate::condition::RCondition`]. The outer
/// `with_r_unwind_protect` in the consumer's C entry point then catches the
/// re-panic and builds the final tagged SEXP for the consumer's R wrapper.
///
/// R-origin errors (longjmp) still pass through via `R_ContinueUnwind` — the
/// outer guard will catch them.
///
/// # PROTECT note
///
/// The returned SEXP is unprotected. The View method wrapper must not call any
/// R API functions between receiving it and passing it to
/// `repanic_if_rust_error`. `repanic_if_rust_error` reads the message/kind/class
/// strings immediately and then panics (or returns), so the SEXP does not need
/// protection beyond that window.
pub fn with_r_unwind_protect_shim<F>(f: F) -> SEXP
where
    F: FnOnce() -> SEXP,
{
    match run_r_unwind_protect(f) {
        Ok(result) => result,
        Err(payload) => {
            // region: RCondition recognition — same as the tagged-SEXP path
            if payload.is::<crate::condition::RCondition>() {
                use crate::error_value::kind;
                // Take ownership of the payload so the `data` Vec can be moved
                // into `make_rust_condition_value` (consumed when materialised).
                let cond = *payload
                    .downcast::<crate::condition::RCondition>()
                    .expect("checked is::<RCondition> above");
                let (kind, message, class, data) = match cond {
                    crate::condition::RCondition::Error {
                        message,
                        class,
                        data,
                    } => (kind::ERROR, message, class, data),
                    crate::condition::RCondition::Warning {
                        message,
                        class,
                        data,
                    } => (kind::WARNING, message, class, data),
                    crate::condition::RCondition::Message { message, data } => {
                        (kind::MESSAGE, message, Vec::new(), data)
                    }
                    crate::condition::RCondition::Condition {
                        message,
                        class,
                        data,
                    } => (kind::CONDITION, message, class, data),
                };
                // SAFETY: on the R main thread inside R_UnwindProtect.
                return unsafe {
                    crate::error_value::make_rust_condition_value_with_data(
                        &message, kind, &class, None, data,
                    )
                };
            }
            // endregion

            // Generic panic path — fold the hook-captured `(at file:line)` into
            // the message (this shim runs on the panicking thread).
            let msg = panic_message_with_location(payload.as_ref());
            crate::panic_telemetry::fire(&msg, crate::panic_telemetry::PanicSource::UnwindProtect);
            // SAFETY: on the R main thread inside R_UnwindProtect.
            unsafe {
                crate::error_value::make_rust_condition_value(
                    &msg,
                    crate::error_value::kind::PANIC,
                    None,
                    None,
                )
            }
        }
    }
}

/// Run a closure under `R_UnwindProtect`, returning a tagged condition SEXP on
/// Rust panics instead of raising an R error.
///
/// This is **the** transport for all `#[miniextendr]` functions and methods.
/// The returned error/condition SEXP is inspected by the generated R wrapper
/// which raises a proper R condition past the Rust boundary, with `rust_*`
/// class layering.
///
/// Recognises [`crate::condition::RCondition`] payloads (from `error!()`,
/// `warning!()`, `message!()`, `condition!()`) before falling through to the
/// generic panic→string path.
///
/// R-origin errors (longjmp) still pass through via `R_ContinueUnwind`.
///
/// For guard sites that have no R wrapper to inspect a tagged SEXP (ALTREP
/// `RUnwind` callbacks, FFI guard tests) see [`with_r_unwind_protect_or_raise`];
/// for trait-ABI vtable shims see [`with_r_unwind_protect_shim`].
pub fn with_r_unwind_protect<F>(f: F, call: Option<SEXP>) -> SEXP
where
    F: FnOnce() -> SEXP,
{
    match run_r_unwind_protect(f) {
        Ok(result) => result,
        Err(payload) => {
            // region: RCondition recognition — must come before generic panic path
            if payload.is::<crate::condition::RCondition>() {
                use crate::error_value::kind;
                // Take ownership so the `data` payload can be moved into
                // `make_rust_condition_value` (consumed during materialisation).
                let cond = *payload
                    .downcast::<crate::condition::RCondition>()
                    .expect("checked is::<RCondition> above");
                let (kind, message, class, data) = match cond {
                    crate::condition::RCondition::Error {
                        message,
                        class,
                        data,
                    } => (kind::ERROR, message, class, data),
                    crate::condition::RCondition::Warning {
                        message,
                        class,
                        data,
                    } => (kind::WARNING, message, class, data),
                    crate::condition::RCondition::Message { message, data } => {
                        (kind::MESSAGE, message, Vec::new(), data)
                    }
                    crate::condition::RCondition::Condition {
                        message,
                        class,
                        data,
                    } => (kind::CONDITION, message, class, data),
                };
                // No panic telemetry for user-raised conditions — they are intentional.
                // SAFETY: on the R main thread inside R_UnwindProtect.
                return unsafe {
                    crate::error_value::make_rust_condition_value_with_data(
                        &message, kind, &class, call, data,
                    )
                };
            }
            // endregion

            // Generic panic path — the primary user-facing route for
            // `#[miniextendr]` fns running on the main thread. Fold the
            // hook-captured `(at file:line)` into the message. (The RCondition
            // branch above is deliberately untouched: error!/warning!/message!/
            // condition! and Err/None carry no location suffix.)
            let msg = panic_message_with_location(payload.as_ref());
            crate::panic_telemetry::fire(&msg, crate::panic_telemetry::PanicSource::UnwindProtect);
            // SAFETY: on the R main thread inside R_UnwindProtect.
            unsafe {
                crate::error_value::make_rust_condition_value(
                    &msg,
                    crate::error_value::kind::PANIC,
                    None,
                    call,
                )
            }
        }
    }
}
