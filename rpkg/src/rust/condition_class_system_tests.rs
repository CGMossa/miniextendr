//! Cross class-system condition fixtures.
//!
//! Each class system (R6, S3, S4, S7, Env, Vctrs) gets a small struct whose
//! methods raise every `RCondition` variant — bare and with a custom class. The
//! matrix exercises the per-class-system R wrapper shape (active bindings,
//! setMethod, `self`-default arg, S7 `tryCatch(x@.ptr)` lambda, vctrs S3
//! `UseMethod` dispatch, ...) which all dispatch through
//! `.miniextendr_raise_condition` after PR #382.
//!
//! The Vctrs fixture is feature-gated behind `vctrs`. R-side tests pair with
//! `skip_if_vctrs_disabled()` in
//! `rpkg/tests/testthat/test-conditions-comprehensive.R`.
//!
//! Tests live in `rpkg/tests/testthat/test-conditions-comprehensive.R`.

use miniextendr_api::externalptr::ExternalPtr;
use miniextendr_api::miniextendr;

type RCondition = miniextendr_api::condition::RCondition;

// region: shared helpers

/// Variable-class `error!` — the `error!` macro requires a literal class string.
fn raise_error_with_class(class: &str, msg: &str) -> ! {
    std::panic::panic_any(RCondition::Error {
        message: msg.to_string(),
        class: vec![class.to_string()],
        data: None,
    });
}

fn raise_warning_with_class(class: &str, msg: &str) -> ! {
    std::panic::panic_any(RCondition::Warning {
        message: msg.to_string(),
        class: vec![class.to_string()],
        data: None,
    });
}

fn raise_condition_with_class(class: &str, msg: &str) -> ! {
    std::panic::panic_any(RCondition::Condition {
        message: msg.to_string(),
        class: vec![class.to_string()],
        data: None,
    });
}

// endregion

// region: R6 — active bindings + $-dispatch

/// R6 fixture for raising every condition kind from instance methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6Raiser {
    id: i32,
}

#[miniextendr(r6)]
impl R6Raiser {
    pub fn new(id: i32) -> Self {
        R6Raiser { id }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn raise_error(&self, msg: &str) {
        miniextendr_api::error!("{msg}");
    }

    pub fn raise_error_classed(&self, class: &str, msg: &str) {
        raise_error_with_class(class, msg);
    }

    pub fn raise_warning(&self, msg: &str) {
        miniextendr_api::warning!("{msg}");
    }

    pub fn raise_warning_classed(&self, class: &str, msg: &str) {
        raise_warning_with_class(class, msg);
    }

    pub fn raise_message(&self, msg: &str) {
        miniextendr_api::message!("{msg}");
    }

    pub fn raise_condition(&self, msg: &str) {
        miniextendr_api::condition!("{msg}");
    }

    pub fn raise_condition_classed(&self, class: &str, msg: &str) {
        raise_condition_with_class(class, msg);
    }
}

// endregion

// region: S3 — setMethod-style

#[derive(miniextendr_api::ExternalPtr)]
pub struct S3Raiser {
    id: i32,
}

#[miniextendr(s3, internal)]
impl S3Raiser {
    /// @param id Numeric identifier for this raiser instance.
    pub fn new(id: i32) -> Self {
        S3Raiser { id }
    }

    pub fn s3_id(&self) -> i32 {
        self.id
    }

    pub fn s3_raise_error(&self, msg: &str) {
        miniextendr_api::error!("{msg}");
    }

    pub fn s3_raise_error_classed(&self, class: &str, msg: &str) {
        raise_error_with_class(class, msg);
    }

    pub fn s3_raise_warning(&self, msg: &str) {
        miniextendr_api::warning!("{msg}");
    }

    pub fn s3_raise_warning_classed(&self, class: &str, msg: &str) {
        raise_warning_with_class(class, msg);
    }

    pub fn s3_raise_message(&self, msg: &str) {
        miniextendr_api::message!("{msg}");
    }

    pub fn s3_raise_condition(&self, msg: &str) {
        miniextendr_api::condition!("{msg}");
    }

    pub fn s3_raise_condition_classed(&self, class: &str, msg: &str) {
        raise_condition_with_class(class, msg);
    }
}

// endregion

// region: S4 — setMethod with formal class

#[derive(miniextendr_api::ExternalPtr)]
pub struct S4Raiser {
    id: i32,
}

/// @aliases s4_raise_error,S4Raiser-method s4_raise_error_classed,S4Raiser-method s4_raise_warning,S4Raiser-method s4_raise_warning_classed,S4Raiser-method s4_raise_message,S4Raiser-method s4_raise_condition,S4Raiser-method s4_raise_condition_classed,S4Raiser-method s4_id,S4Raiser-method
//
// NB: s4 codegen auto-prepends `s4_` to each method name when generating the R
// generic; keep the Rust method names unprefixed (`raise_error`, not `s4_raise_error`)
// or you end up with `s4_s4_raise_error`.
#[miniextendr(s4, internal)]
impl S4Raiser {
    pub fn new(id: i32) -> Self {
        S4Raiser { id }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn raise_error(&self, msg: &str) {
        miniextendr_api::error!("{msg}");
    }

    pub fn raise_error_classed(&self, class: &str, msg: &str) {
        raise_error_with_class(class, msg);
    }

    pub fn raise_warning(&self, msg: &str) {
        miniextendr_api::warning!("{msg}");
    }

    pub fn raise_warning_classed(&self, class: &str, msg: &str) {
        raise_warning_with_class(class, msg);
    }

    pub fn raise_message(&self, msg: &str) {
        miniextendr_api::message!("{msg}");
    }

    pub fn raise_condition(&self, msg: &str) {
        miniextendr_api::condition!("{msg}");
    }

    pub fn raise_condition_classed(&self, class: &str, msg: &str) {
        raise_condition_with_class(class, msg);
    }
}

// endregion

// region: S7 — new_generic dispatch + tryCatch(x@.ptr) lambda

#[derive(miniextendr_api::ExternalPtr)]
pub struct S7Raiser {
    id: i32,
}

#[miniextendr(s7, internal)]
impl S7Raiser {
    pub fn new(id: i32) -> Self {
        S7Raiser { id }
    }

    pub fn s7_id(&self) -> i32 {
        self.id
    }

    pub fn s7_raise_error(&self, msg: &str) {
        miniextendr_api::error!("{msg}");
    }

    pub fn s7_raise_error_classed(&self, class: &str, msg: &str) {
        raise_error_with_class(class, msg);
    }

    pub fn s7_raise_warning(&self, msg: &str) {
        miniextendr_api::warning!("{msg}");
    }

    pub fn s7_raise_warning_classed(&self, class: &str, msg: &str) {
        raise_warning_with_class(class, msg);
    }

    pub fn s7_raise_message(&self, msg: &str) {
        miniextendr_api::message!("{msg}");
    }

    pub fn s7_raise_condition(&self, msg: &str) {
        miniextendr_api::condition!("{msg}");
    }

    pub fn s7_raise_condition_classed(&self, class: &str, msg: &str) {
        raise_condition_with_class(class, msg);
    }
}

// endregion

// region: Env — `self`-default-arg standalone wrapper

#[derive(miniextendr_api::ExternalPtr)]
pub struct EnvRaiser {
    id: i32,
}

#[miniextendr(env)]
impl EnvRaiser {
    pub fn new(id: i32) -> Self {
        EnvRaiser { id }
    }

    pub fn env_id(&self) -> i32 {
        self.id
    }

    pub fn env_raise_error(&self, msg: &str) {
        miniextendr_api::error!("{msg}");
    }

    pub fn env_raise_error_classed(&self, class: &str, msg: &str) {
        raise_error_with_class(class, msg);
    }

    pub fn env_raise_warning(&self, msg: &str) {
        miniextendr_api::warning!("{msg}");
    }

    pub fn env_raise_warning_classed(&self, class: &str, msg: &str) {
        raise_warning_with_class(class, msg);
    }

    pub fn env_raise_message(&self, msg: &str) {
        miniextendr_api::message!("{msg}");
    }

    pub fn env_raise_condition(&self, msg: &str) {
        miniextendr_api::condition!("{msg}");
    }

    pub fn env_raise_condition_classed(&self, class: &str, msg: &str) {
        raise_condition_with_class(class, msg);
    }
}

// endregion

// region: Vctrs — static methods + S3 protocol override, vector-payload constructor
//
// vctrs codegen forbids `&self` receivers (MXL120) because a vctrs object is an
// S3-classed base vector — there is no Rust `Self` stored in the SEXP. All
// methods are therefore static; the vctrs payload (`Vec<f64>`) is passed
// explicitly.
//
// Two emission shapes are exercised:
//   1. Static helpers — emitted as plain wrapped fns
//      (`vctrsraiser_vctrs_raise_error(values, msg)`); full `match.call()`
//      attribution like any `#[miniextendr]` free function.
//   2. vctrs protocol override (`#[miniextendr(vctrs(format))]`) — emitted as
//      the S3 method `format.VctrsRaiser(values, ...)`; dispatch goes through
//      `UseMethod("format")` in base R, the method body carries
//      `.call = match.call()` of the inner `.Call`. This is the closest analogue
//      to the S3 raise_*.S3Raiser path.

#[cfg(feature = "vctrs")]
mod vctrs_raiser {
    use super::*;

    /// Vctrs fixture — static methods raise every condition kind.
    ///
    /// `kind = "vctr"` + `base = "double"` means the constructor returns the
    /// numeric payload (`Vec<f64>`) and the R wrapper calls
    /// `vctrs::new_vctr(.data, class = "VctrsRaiser")`. Method names start with
    /// `vctrs_` to avoid colliding with the unprefixed `raise_*` static fns on
    /// other class systems.
    pub struct VctrsRaiser;

    #[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "raise"))]
    impl VctrsRaiser {
        /// Vctrs ctors return vector payload (not Self) — vctrs::new_vctr wraps it.
        /// @param values Numeric values to embed in the vctrs vector.
        #[allow(clippy::new_ret_no_self)]
        pub fn new(values: Vec<f64>) -> Vec<f64> {
            values
        }

        /// @param values The vctrs payload (unused; receiver stand-in for the S3 object).
        /// @param msg Error message.
        pub fn vctrs_raise_error(_values: Vec<f64>, msg: String) {
            miniextendr_api::error!("{msg}");
        }

        /// @param values The vctrs payload (unused).
        /// @param class Custom condition class.
        /// @param msg Error message.
        pub fn vctrs_raise_error_classed(_values: Vec<f64>, class: String, msg: String) {
            raise_error_with_class(&class, &msg);
        }

        /// @param values The vctrs payload (unused).
        /// @param msg Warning message.
        pub fn vctrs_raise_warning(_values: Vec<f64>, msg: String) {
            miniextendr_api::warning!("{msg}");
        }

        /// @param values The vctrs payload (unused).
        /// @param class Custom condition class.
        /// @param msg Warning message.
        pub fn vctrs_raise_warning_classed(_values: Vec<f64>, class: String, msg: String) {
            raise_warning_with_class(&class, &msg);
        }

        /// @param values The vctrs payload (unused).
        /// @param msg Message text.
        pub fn vctrs_raise_message(_values: Vec<f64>, msg: String) {
            miniextendr_api::message!("{msg}");
        }

        /// @param values The vctrs payload (unused).
        /// @param msg Condition message.
        pub fn vctrs_raise_condition(_values: Vec<f64>, msg: String) {
            miniextendr_api::condition!("{msg}");
        }

        /// @param values The vctrs payload (unused).
        /// @param class Custom condition class.
        /// @param msg Condition message.
        pub fn vctrs_raise_condition_classed(_values: Vec<f64>, class: String, msg: String) {
            raise_condition_with_class(&class, &msg);
        }

        /// vctrs protocol override — emits `format.VctrsRaiser(x, ...)` for
        /// S3 dispatch via `UseMethod("format")`. Always panics so the test can
        /// inspect `conditionCall` for the dispatched generic name. Note that
        /// `format`-protocol dispatch is what's being exercised here, not
        /// arbitrary `vctrs_*` S3 generics — `match.call()` inside
        /// `format.VctrsRaiser` captures the dispatched-method call frame, not
        /// the user's `format(obj)` call.
        ///
        /// The argument is named `x` (not `values`) so the generated S3 method
        /// signature `format.VctrsRaiser(x, ...)` matches the base S3 generic
        /// `format(x, ...)` — otherwise `R CMD check` raises an S3
        /// generic/method consistency WARNING.
        ///
        /// @param x The vctrs payload (unused; receiver stand-in for the S3 object).
        /// @return Never returns — always raises a `rust_error`.
        #[miniextendr(vctrs(format))]
        #[allow(clippy::needless_pass_by_value)]
        pub fn format_vctrsraiser(_x: Vec<f64>) -> Vec<String> {
            miniextendr_api::error!("format-protocol boom");
        }
    }
}

// endregion

// region: edge cases — non-RCondition panic, long/unicode messages

/// Panic with a non-`String`/non-`RCondition` payload — falls through to the
/// generic panic→string fallback.
#[miniextendr]
pub fn condition_panic_with_int_payload() {
    std::panic::panic_any(42_i32);
}

/// Long message exercises the `CString` + STRSXP encoding path.
#[miniextendr]
pub fn condition_error_long_message(n_chunks: i32) {
    let chunk = "abcdefghij_";
    let n = n_chunks.max(0) as usize;
    let msg: String = chunk.repeat(n);
    miniextendr_api::error!("{msg}");
}

/// Unicode + multibyte + embedded newline — tests UTF-8 round-trip.
#[miniextendr]
pub fn condition_error_unicode() {
    miniextendr_api::error!("rust ⚙️ panic\n日本語\nемоджи 🦀");
}

/// Empty error message — degenerate but should still produce a valid condition.
#[miniextendr]
pub fn condition_error_empty() {
    miniextendr_api::error!("");
}

// endregion

// region: audit A9 — ExternalPtr<T> arguments accept class-wrapped handles
//
// Reuses the S3/S4/S7 raisers above (already exported types) to prove a
// standalone fn declared as `ExternalPtr<T>` now accepts the ergonomic class
// handle, not just the bare EXTPTRSXP a low-level constructor would return.
// Env/R6 already had (or gained, for R6) coverage elsewhere:
// `ptr_identity`/`ptr_pick_larger` (Env, `externalptr_identity_tests.rs`) and
// `sidecar_consumer_panic` (R6, `condition_sidecar_tests.rs`). Vctrs is not
// covered here — vctrs objects in this framework are plain classed vectors
// with no Rust struct behind them (`VctrsRaiser` above doesn't even derive
// `ExternalPtr`), so there is no handle to unwrap: passing one to an
// `ExternalPtr<T>` parameter is an ordinary type mismatch, unrelated to A9.
//
// Tests live in `rpkg/tests/testthat/test-conditions-comprehensive.R`.

/// Unwraps an S3-wrapped `S3Raiser` handle (a classed `EXTPTRSXP` — already
/// worked pre-A9) and returns its `id`.
/// @param x A raiser handle (class object or bare external pointer).
#[miniextendr(noexport)]
pub fn s3_raiser_id_via_externalptr(x: ExternalPtr<S3Raiser>) -> i32 {
    x.id
}

/// Unwraps an S4-wrapped `S4Raiser` handle (the `ptr` slot, via
/// `methods::slot()`) and returns its `id`.
/// @param x A raiser handle (class object or bare external pointer).
#[miniextendr(noexport)]
pub fn s4_raiser_id_via_externalptr(x: ExternalPtr<S4Raiser>) -> i32 {
    x.id
}

/// Unwraps an S7-wrapped `S7Raiser` handle (the `.ptr` attribute) and
/// returns its `id`.
/// @param x A raiser handle (class object or bare external pointer).
#[miniextendr(noexport)]
pub fn s7_raiser_id_via_externalptr(x: ExternalPtr<S7Raiser>) -> i32 {
    x.id
}

// endregion
