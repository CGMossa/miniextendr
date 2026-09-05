//! Safe wrappers for R expression evaluation.
//!
//! This module provides ergonomic types for building and evaluating R function
//! calls from Rust, handling GC protection and error propagation automatically.
//!
//! # Types
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`RSymbol`] | Interned R symbol (SYMSXP) |
//! | [`RCall`] | Builder for R function calls (LANGSXP) |
//! | [`REnv`] | Well-known R environments |
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::expression::{RCall, REnv};
//!
//! unsafe {
//!     // Call paste0("hello", " world") in the base environment
//!     let result = RCall::new("paste0")
//!         .arg(Rf_mkString(c"hello".as_ptr()))
//!         .arg(Rf_mkString(c" world".as_ptr()))
//!         .eval(REnv::base().as_sexp())?;
//! }
//! ```

use crate::gc_protect::{OwnedProtect, ProtectScope};
use crate::protect_pool::ProtectPool;
use crate::sexp_ext::PairListExt;
use crate::sys::{
    self, ParseStatus, R_BaseEnv, R_EmptyEnv, R_GlobalEnv, R_ParseVector, R_tryEvalSilent,
    Rf_install,
};
use crate::{SEXP, SexpExt};
use std::ffi::{CStr, CString};

// region: RSymbol

/// A safe wrapper around R symbols (SYMSXP).
///
/// R symbols are interned strings used as variable and function names.
/// They are never garbage collected, so `RSymbol` does not need GC protection.
///
/// # Example
///
/// ```ignore
/// let sym = RSymbol::new("paste0");
/// // sym.as_sexp() is a SYMSXP that can be used in call construction
/// ```
pub struct RSymbol {
    sexp: SEXP,
}

impl RSymbol {
    /// Create or retrieve an interned R symbol.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    ///
    /// # Panics
    ///
    /// Panics if `name` contains a null byte.
    #[inline]
    pub unsafe fn new(name: &str) -> Self {
        let c_name = CString::new(name).expect("symbol name must not contain null bytes");
        RSymbol {
            sexp: unsafe { Rf_install(c_name.as_ptr()) },
        }
    }

    /// Create a symbol from a C string literal.
    ///
    /// This avoids the allocation needed by [`new`](Self::new) when you have
    /// a `&CStr` available (e.g., from `c"name"` literals).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn from_cstr(name: &CStr) -> Self {
        RSymbol {
            sexp: unsafe { Rf_install(name.as_ptr()) },
        }
    }

    /// Get the underlying SEXP.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.sexp
    }
}
// endregion

// region: REnv

/// Handle to a well-known R environment.
///
/// Provides access to R's standard environments without raw FFI calls.
pub struct REnv {
    sexp: SEXP,
}

impl REnv {
    /// The global environment (`R_GlobalEnv`).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn global() -> Self {
        REnv {
            sexp: unsafe { R_GlobalEnv },
        }
    }

    /// The base environment (`R_BaseEnv`).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn base() -> Self {
        REnv {
            sexp: unsafe { R_BaseEnv },
        }
    }

    /// The empty environment (`R_EmptyEnv`).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn empty() -> Self {
        REnv {
            sexp: unsafe { R_EmptyEnv },
        }
    }

    /// The base namespace (`SEXP::base_namespace()`).
    ///
    /// Unlike [`base()`](Self::base) which is the base *environment* (exported
    /// functions visible to users), this is the base *namespace* (includes
    /// internal helpers). Rarely needed — prefer [`base()`](Self::base) unless
    /// you specifically need unexported base internals.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub fn base_namespace() -> Self {
        REnv {
            sexp: SEXP::base_namespace(),
        }
    }

    /// A package's namespace environment.
    ///
    /// Finds the namespace for a loaded package. Use this to evaluate functions
    /// that live in a specific package (e.g., `slot()` from `methods`).
    ///
    /// This is a safe wrapper around `R_FindNamespace` — it uses
    /// `R_tryEvalSilent` internally so that a missing namespace returns
    /// `Err` instead of longjmping through Rust frames.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the package namespace is not found (package not loaded).
    pub unsafe fn package_namespace(name: &str) -> Result<Self, String> {
        unsafe {
            let name_sexp = OwnedProtect::new(SEXP::scalar_string_from_str(name));
            RCall::new("getNamespace")
                .arg(name_sexp.get())
                .eval(R_BaseEnv)
                .map(|sexp| REnv { sexp })
        }
    }

    /// The current execution environment.
    ///
    /// Returns the environment of the innermost active closure on R's call
    /// stack, or the global environment if no closure is active.
    ///
    /// Useful when you need to evaluate an expression in the caller's context
    /// rather than a fixed well-known environment.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn caller() -> Self {
        REnv {
            sexp: unsafe { sys::R_GetCurrentEnv() },
        }
    }

    /// Wrap an arbitrary environment SEXP.
    ///
    /// # Safety
    ///
    /// `sexp` must be a valid ENVSXP.
    #[inline]
    pub unsafe fn from_sexp(sexp: SEXP) -> Self {
        REnv { sexp }
    }

    /// Get the underlying SEXP.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.sexp
    }
}
// endregion

// region: RCall

/// Builder for constructing and evaluating R function calls.
///
/// `RCall` constructs a LANGSXP (R language object) from a function name or
/// SEXP and a sequence of arguments (optionally named). It handles GC
/// protection during construction and evaluation.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::expression::RCall;
/// use miniextendr_api::sys;
///
/// unsafe {
///     // seq_len(10)
///     let result = RCall::new("seq_len")
///         .arg(SEXP::scalar_integer(10))
///         .eval_base()?;
///
///     // paste(x, collapse = ", ")
///     let result = RCall::new("paste")
///         .arg(some_sexp)
///         .named_arg("collapse", sys::Rf_mkString(c", ".as_ptr()))
///         .eval_base()?;
/// }
/// ```
pub struct RCall {
    /// Function symbol or SEXP.
    fun: SEXP,
    /// Arguments as (optional_name, value) pairs.
    args: Vec<(Option<CString>, SEXP)>,
    /// Roots the callable and every argument for the builder's lifetime.
    ///
    /// Besides keeping inline allocations alive, `ProtectPool` makes `RCall`
    /// `!Send + !Sync`: construction, mutation, and drop stay on R's thread.
    roots: ProtectPool,
}

impl RCall {
    /// Root a callable and initialize an empty builder.
    ///
    /// `fun` may be a freshly allocated, unprotected closure. The temporary
    /// stack root keeps it alive while the pool allocates its backing VECSXP.
    #[inline]
    unsafe fn from_callable(fun: SEXP) -> Self {
        unsafe {
            let fun_guard = OwnedProtect::new(fun);
            let mut roots = ProtectPool::new(4);
            roots.insert(fun_guard.get());
            RCall {
                fun,
                args: Vec::new(),
                roots,
            }
        }
    }

    /// Start building a call to a named R function.
    ///
    /// The function is looked up via `Rf_install`, which returns an interned symbol.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    ///
    /// # Panics
    ///
    /// Panics if `fun_name` contains a null byte.
    #[inline]
    pub unsafe fn new(fun_name: &str) -> Self {
        let c_name = CString::new(fun_name).expect("function name must not contain null bytes");
        unsafe { Self::from_callable(Rf_install(c_name.as_ptr())) }
    }

    /// Start building a call to a function given as a C string literal.
    ///
    /// More efficient than [`new`](Self::new) when a `&CStr` is available.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn from_cstr(fun_name: &CStr) -> Self {
        unsafe { Self::from_callable(Rf_install(fun_name.as_ptr())) }
    }

    /// Start building a call with a function SEXP (closure, builtin, etc.).
    ///
    /// # Safety
    ///
    /// `fun` must be a valid SEXP representing a callable R object.
    #[inline]
    pub unsafe fn from_sexp(fun: SEXP) -> Self {
        unsafe { Self::from_callable(fun) }
    }

    /// Add a positional argument.
    #[inline]
    pub fn arg(mut self, value: SEXP) -> Self {
        // The temporary stack root covers `ProtectPool::insert` if growing the
        // backing VECSXP allocates. The pool then owns the lasting root.
        unsafe {
            let value_guard = OwnedProtect::new(value);
            self.roots.insert(value_guard.get());
        }
        self.args.push((None, value));
        self
    }

    /// Add a named argument.
    ///
    /// # Panics
    ///
    /// Panics if `name` contains a null byte.
    #[inline]
    pub fn named_arg(mut self, name: &str, value: SEXP) -> Self {
        let c_name = CString::new(name).expect("argument name must not contain null bytes");
        unsafe {
            let value_guard = OwnedProtect::new(value);
            self.roots.insert(value_guard.get());
        }
        self.args.push((Some(c_name), value));
        self
    }

    /// Build the LANGSXP without evaluating it.
    ///
    /// The returned SEXP is **unprotected**. The caller must protect it if
    /// further allocations will occur before use.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. The builder keeps the callable
    /// and every argument rooted until it is dropped.
    pub unsafe fn build(&self) -> SEXP {
        unsafe {
            // Build the argument pairlist from back to front using Rf_cons.
            // ProtectScope tracks all intermediate cons cells and the final
            // LANGSXP head, then unprotects them all on drop. The returned
            // call is unprotected — caller protects it if needed.
            let scope = ProtectScope::new();

            let mut tail = SEXP::nil();
            for (name, value) in self.args.iter().rev() {
                tail = scope.protect_raw(value.cons(tail));
                if let Some(c_name) = name {
                    tail.set_tag(Rf_install(c_name.as_ptr()));
                }
            }

            // Prepend the function as LANGSXP head.
            // ProtectScope drops here; the call is unprotected on return
            // (callers re-protect via OwnedProtect before invoking eval).
            scope.protect_raw(self.fun.lcons(tail))
        }
    }

    /// Evaluate the call in the given environment.
    ///
    /// Uses `R_tryEvalSilent` so that R errors are captured as `Err(String)`
    /// rather than causing a longjmp through Rust frames.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread.
    /// - `env` must be a valid ENVSXP.
    /// - The builder must not outlive the active R session.
    ///
    /// # Returns
    ///
    /// - `Ok(SEXP)` with the result (unprotected — caller should protect if needed)
    /// - `Err(String)` with the R error message on failure
    pub unsafe fn eval(&self, env: SEXP) -> Result<SEXP, String> {
        unsafe {
            let call = OwnedProtect::new(self.build());

            let mut error_occurred: std::os::raw::c_int = 0;
            let result = R_tryEvalSilent(call.get(), env, &mut error_occurred);

            if error_occurred != 0 {
                Err(get_r_error_message())
            } else {
                Ok(result)
            }
        }
    }

    /// Evaluate in `R_BaseEnv`.
    ///
    /// # Safety
    ///
    /// Same as [`eval`](Self::eval).
    #[inline]
    pub unsafe fn eval_base(&self) -> Result<SEXP, String> {
        unsafe { self.eval(R_BaseEnv) }
    }

    /// Start building a namespaced call: `pkg::fun(args…)`.
    ///
    /// Looks up `pkg::fun_name` in the base environment and uses the resolved
    /// function closure as the call target. This respects R's namespace lookup
    /// rules (exported + non-exported via `::` / `:::`).
    ///
    /// This is the runtime counterpart of the lowered `pkg::fn(args…)` form
    /// in `r!(pkg::fn(args…))`.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    ///
    /// # Panics
    ///
    /// Panics if `pkg` or `fun_name` contain a null byte.
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` if the namespace lookup fails (package not loaded
    /// or function not found).
    pub unsafe fn namespaced(pkg: &str, fun_name: &str) -> Result<Self, String> {
        unsafe {
            // Build (:: pkg fun_name) as a LANGSXP and evaluate it to get the closure.
            let ns_op = Rf_install(c"::".as_ptr());
            let pkg_sym = {
                let c = CString::new(pkg).expect("pkg name must not contain null bytes");
                Rf_install(c.as_ptr())
            };
            let fun_sym = {
                let c = CString::new(fun_name).expect("fun name must not contain null bytes");
                Rf_install(c.as_ptr())
            };

            // Build `(:: pkg fun_name)` pairlist from back to front.
            let scope = crate::gc_protect::ProtectScope::new();
            let nil = crate::sys::R_NilValue;
            let fun_cons = scope.protect_raw(fun_sym.cons(nil));
            let pkg_cons = scope.protect_raw(pkg_sym.cons(fun_cons));
            let ns_call = scope.protect_raw(ns_op.lcons(pkg_cons));

            // Evaluate to resolve the function closure.
            let mut err: std::os::raw::c_int = 0;
            let fun_sexp = R_tryEvalSilent(ns_call, R_BaseEnv, &mut err);
            if err != 0 {
                return Err(get_r_error_message());
            }

            // Root the resolved closure for the builder lifetime. This is
            // load-bearing even for closures that are not namespace bindings
            // (the same constructor backs `RCall::from_sexp`).
            Ok(RCall::from_sexp(fun_sexp))
        }
    }
}

/// Build and evaluate `target$name` — the R `$` extraction operator.
///
/// This is a convenience wrapper that avoids hand-rolling
/// `Rf_install("$") + Rf_lang3(...) + R_tryEvalSilent(...)` ladders.
/// Equivalent to:
///
/// ```ignore
/// RCall::new("$")
///     .arg(target)
///     .arg(SEXP::scalar_string_from_str(name))
///     .eval_base()
/// ```
///
/// but uses the more direct LANGSXP form internally and protects all
/// intermediate allocations via RAII.
///
/// # Safety
///
/// - Must be called from the R main thread.
/// - `target` must be a valid SEXP (typically a list, environment, or S4
///   object that supports `$` extraction).
///
/// # Returns
///
/// - `Ok(SEXP)` with the extracted value (unprotected — caller should protect if needed).
/// - `Err(String)` with the R error message if `$` extraction fails or the
///   evaluation errors.
pub unsafe fn dollar_extract(target: SEXP, name: &str) -> Result<SEXP, String> {
    unsafe {
        let name_sexp = OwnedProtect::new(SEXP::scalar_string_from_str(name));
        RCall::new("$").arg(target).arg(name_sexp.get()).eval_base()
    }
}
// endregion

// region: r_eval_str (runtime string parse + eval)

/// Parse a string of R source and evaluate it in `env`.
///
/// This is the runtime workhorse behind the [`r_str!`](crate::r_str) and
/// [`r!`](crate::r) macros. It performs the full
/// `R_ParseVector` → check status → `Rf_eval` ladder with correct GC
/// protection on every intermediate SEXP, so callers never have to hand-roll
/// `OwnedProtect` around the parse tree.
///
/// Only the **last** top-level expression's value is returned (matching R's
/// `eval(parse(text = ...))` semantics): each parsed expression is evaluated in
/// order so that side effects (assignments, `library()`, …) take effect, and
/// the value of the final one is returned. An empty / whitespace-only string
/// yields `R_NilValue`.
///
/// # Safety
///
/// - Must be called from (or routed to) the R main thread. The parse and eval
///   FFI calls go through the checked `#[r_ffi_checked]` variants, which
///   serialize onto the R thread via `with_r_thread`, so calling from a
///   worker thread is sound — but the returned SEXP must not outlive the R
///   session.
/// - `env` must be a valid ENVSXP.
///
/// # Returns
///
/// - `Ok(SEXP)` with the value of the last expression (**unprotected** — the
///   caller should protect it if further allocations will occur before use).
/// - `Err(String)` if parsing fails (syntax error / incomplete input) or if
///   evaluation raises an R error. The error is captured via
///   `R_tryEvalSilent` + `geterrmessage()`, so it never longjmps through Rust
///   frames.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::expression::r_eval_str;
/// use miniextendr_api::sys::R_GlobalEnv;
///
/// unsafe {
///     let three = r_eval_str("1L + 2L", R_GlobalEnv)?;
///     // three is an INTSXP holding 3
/// }
/// ```
pub unsafe fn r_eval_str(code: &str, env: SEXP) -> Result<SEXP, String> {
    unsafe {
        // 1. Wrap the source in a length-1 STRSXP. scalar_string_from_str
        //    allocates a CHARSXP + STRSXP; protect it across the parse, which
        //    allocates again.
        let code_sexp = OwnedProtect::new(SEXP::scalar_string_from_str(code));

        // 2. Parse. R_ParseVector returns an EXPRSXP (a vector of expressions).
        //    Protect it across the subsequent VECTOR_ELT / Rf_eval allocations.
        let mut status = ParseStatus::PARSE_NULL;
        let parsed = R_ParseVector(code_sexp.get(), -1, &mut status, sys::R_NilValue);

        match status {
            ParseStatus::PARSE_OK => {}
            ParseStatus::PARSE_INCOMPLETE => {
                return Err(format!(
                    "incomplete R expression (unbalanced delimiter?): {code}"
                ));
            }
            ParseStatus::PARSE_ERROR => {
                return Err(format!("R syntax error while parsing: {code}"));
            }
            ParseStatus::PARSE_EOF => {
                return Err(format!("unexpected end of input while parsing: {code}"));
            }
            ParseStatus::PARSE_NULL => {
                return Err(format!("R_ParseVector returned PARSE_NULL for: {code}"));
            }
        }

        let parsed = OwnedProtect::new(parsed);

        // 3. Evaluate each parsed expression in order; return the value of the
        //    last one. An empty EXPRSXP (blank source) yields R_NilValue.
        let n = parsed.get().xlength();
        let mut result = sys::R_NilValue;
        for i in 0..n {
            // VECTOR_ELT borrows from `parsed` (still protected). The element
            // is part of the protected EXPRSXP, so it stays reachable.
            let expr = parsed.get().vector_elt(i);

            let mut error_occurred: std::os::raw::c_int = 0;
            result = R_tryEvalSilent(expr, env, &mut error_occurred);
            if error_occurred != 0 {
                return Err(get_r_error_message());
            }
        }

        Ok(result)
    }
}

/// Parse and evaluate a string of R source in `R_GlobalEnv`.
///
/// Convenience wrapper over [`r_eval_str`] for the common case. See that
/// function for safety and return semantics.
///
/// # Safety
///
/// Same as [`r_eval_str`].
#[inline]
pub unsafe fn r_eval_str_global(code: &str) -> Result<SEXP, String> {
    unsafe { r_eval_str(code, R_GlobalEnv) }
}
// endregion

// region: Error message extraction

/// Extract the most recent R error message.
///
/// Uses `geterrmessage()` which is public R API (unlike `R_curErrorBuf`
/// which is non-API). Falls back to a generic message if extraction fails.
unsafe fn get_r_error_message() -> String {
    unsafe {
        // Call geterrmessage() — a public R function that returns the last
        // error message as a character(1) string.
        let call = OwnedProtect::new(sys::Rf_lang1(Rf_install(c"geterrmessage".as_ptr())));

        let mut err: std::os::raw::c_int = 0;
        let msg_sexp = R_tryEvalSilent(call.get(), R_BaseEnv, &mut err);

        if err != 0 || msg_sexp.is_null() {
            return "R error occurred (could not retrieve message)".to_string();
        }

        let _msg_guard = OwnedProtect::new(msg_sexp);

        // geterrmessage() returns character(1)
        if msg_sexp.xlength() > 0 {
            let charsxp = msg_sexp.string_elt(0);
            if let Some(msg) = crate::from_r::charsxp_to_string_lossy(charsxp) {
                return msg.trim_end().to_string();
            }
        }
        "R error occurred".to_string()
    }
}
// endregion

// region: Tests

#[cfg(test)]
mod tests {
    use super::*;

    // These tests verify compilation and basic invariants.
    // Full integration tests require the R runtime.

    #[test]
    fn renv_types_are_sized() {
        // Just verify types compile and are sized
        fn assert_sized<T: Sized>() {}
        assert_sized::<RSymbol>();
        assert_sized::<RCall>();
        assert_sized::<REnv>();
    }

    #[test]
    fn renv_constructors_compile() {
        // Verify all REnv constructor signatures compile.
        // Actual testing requires the R runtime.
        fn assert_env_fn<F: FnOnce() -> REnv>(_f: F) {}
        fn assert_env_result_fn<F: FnOnce() -> Result<REnv, String>>(_f: F) {}

        assert_env_fn(|| unsafe { REnv::global() });
        assert_env_fn(|| unsafe { REnv::base() });
        assert_env_fn(|| unsafe { REnv::empty() });
        assert_env_fn(REnv::base_namespace);
        assert_env_fn(|| unsafe { REnv::caller() });
        assert_env_result_fn(|| unsafe { REnv::package_namespace("base") });
    }
}
// endregion
