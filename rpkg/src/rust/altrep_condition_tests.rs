//! ALTREP test fixtures for issue-345 SPIKE: rust_* class layering from RUnwind callbacks.
//!
//! These fixtures verify that panics and `error!()` from ALTREP `r_unwind` guard callbacks
//! produce R conditions that match `tryCatch(rust_error = h, ...)` and
//! `tryCatch(altrep_specific = h, ...)` respectively.
//!
//! Tests live in `rpkg/tests/testthat/test-altrep-conditions.R`.

use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen};
use miniextendr_api::miniextendr;
use miniextendr_api::{AltrepInteger, IntoR, SEXP};

// region: PanickingAltrep — plain panic from elt(), guard = RUnwind (default)

/// ALTREP integer that panics with a plain message when any element is accessed.
///
/// Uses the default `GUARD = AltrepGuard::RUnwind`, so the panic is caught by
/// `with_r_unwind_protect_sourced` → `raise_rust_condition_via_stop`.
/// Expected: `tryCatch(x[1L], rust_error = function(e) conditionClass(e))` matches.
#[derive(AltrepInteger)]
#[altrep(class = "PanickingAltrep", manual)]
pub struct PanickingAltrepData {
    pub len: usize,
    pub message: String,
}

impl AltrepLen for PanickingAltrepData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltIntegerData for PanickingAltrepData {
    fn elt(&self, _i: usize) -> i32 {
        panic!("{}", self.message);
    }
}

/// Create an ALTREP integer that panics on element access (plain panic).
///
/// Accessing any element will raise `rust_error` in R — catchable via
/// `tryCatch(x[1L], rust_error = function(e) e)`.
///
/// @param n Length of the vector.
/// @param message Panic message.
/// @return An ALTREP integer vector.
/// @export
#[miniextendr]
pub fn altrep_panic_on_elt(n: i32, message: &str) -> SEXP {
    let data = PanickingAltrepData {
        len: n.max(0) as usize,
        message: message.to_string(),
    };
    data.into_sexp()
}

// endregion

// region: ClassedErrorAltrep — error!(class = ...) from elt(), guard = RUnwind (default)

/// ALTREP integer that raises `error!(class = "altrep_specific", ...)` on element access.
///
/// Uses the default `GUARD = AltrepGuard::RUnwind`. Expected:
/// - `tryCatch(x[1L], altrep_specific = function(e) "caught!")` matches.
/// - `tryCatch(x[1L], rust_error = function(e) "caught!")` also matches (layered class).
#[derive(AltrepInteger)]
#[altrep(class = "ClassedErrorAltrep", manual)]
pub struct ClassedErrorAltrepData {
    pub len: usize,
    pub error_class: String,
    pub message: String,
}

impl AltrepLen for ClassedErrorAltrepData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltIntegerData for ClassedErrorAltrepData {
    fn elt(&self, _i: usize) -> i32 {
        // Can't use error!(class = ...) with a runtime variable directly.
        // Use the enum directly to set the class at runtime.
        std::panic::panic_any(miniextendr_api::condition::RCondition::Error {
            message: self.message.clone(),
            class: vec![self.error_class.clone()],
            data: None,
        });
    }
}

/// Create an ALTREP integer that raises `error!(class = ..., ...)` on element access.
///
/// Accessing any element will raise a classed R error — catchable via
/// `tryCatch(x[1L], altrep_specific = function(e) e)` or
/// `tryCatch(x[1L], rust_error = function(e) e)`.
///
/// @param n Length of the vector.
/// @param error_class Custom class for the error condition.
/// @param message Error message.
/// @return An ALTREP integer vector.
/// @export
#[miniextendr]
pub fn altrep_classed_error_on_elt(n: i32, error_class: &str, message: &str) -> SEXP {
    let data = ClassedErrorAltrepData {
        len: n.max(0) as usize,
        error_class: error_class.to_string(),
        message: message.to_string(),
    };
    data.into_sexp()
}

// endregion

// region: LoopStressAltrep — panic on specific index, used to test tight-loop re-entry

/// ALTREP integer that succeeds for most elements but panics on a specific index.
///
/// Used to test the open question from issue-345 plan: does `Rf_eval(stop(...))`
/// from ALTREP context risk re-entering ALTREP dispatch?
#[derive(AltrepInteger)]
#[altrep(class = "LoopStressAltrep", manual)]
pub struct LoopStressAltrepData {
    pub len: usize,
    pub panic_at: usize,
}

impl AltrepLen for LoopStressAltrepData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltIntegerData for LoopStressAltrepData {
    fn elt(&self, i: usize) -> i32 {
        if i == self.panic_at {
            panic!("deliberate panic at index {}", i);
        }
        i as i32
    }
}

/// Create an ALTREP integer that panics on element `panic_at` (0-indexed).
///
/// Used to stress-test ALTREP re-entry safety when `raise_rust_condition_via_stop`
/// fires `Rf_eval(stop(...))` from within an ALTREP callback.
///
/// @param n Length of the vector.
/// @param panic_at Zero-based index at which to panic.
/// @return An ALTREP integer vector.
/// @export
#[miniextendr]
pub fn altrep_panic_at_index(n: i32, panic_at: i32) -> SEXP {
    let data = LoopStressAltrepData {
        len: n.max(0) as usize,
        panic_at: panic_at.max(0) as usize,
    };
    data.into_sexp()
}

// endregion

// region: Non-error degradation — warning!/message!/condition! from elt() (issue #366)
//
// These three kinds cannot suspend execution from inside an ALTREP callback —
// no R wrapper can catch and resume. The unwind path documents this and
// degrades them to a plain R error with a diagnostic message. After the #366
// fix the degraded error inherits `rust_error` class layering; before the fix
// it produces a plain `simpleError`.

#[derive(AltrepInteger)]
#[altrep(class = "WarnAltrep", manual)]
pub struct WarnAltrepData {
    pub len: usize,
    pub message: String,
}

impl AltrepLen for WarnAltrepData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltIntegerData for WarnAltrepData {
    fn elt(&self, _i: usize) -> i32 {
        miniextendr_api::warning!("{}", self.message);
    }
}

/// Create an ALTREP integer that calls `warning!()` on element access.
///
/// In ALTREP RUnwind context the warning cannot suspend — it is degraded to
/// a `rust_error` with a fixed diagnostic message.
///
/// @param n Length of the vector.
/// @param message Warning message.
/// @return An ALTREP integer vector.
/// @export
#[miniextendr]
pub fn altrep_warn_on_elt(n: i32, message: &str) -> SEXP {
    let data = WarnAltrepData {
        len: n.max(0) as usize,
        message: message.to_string(),
    };
    data.into_sexp()
}

#[derive(AltrepInteger)]
#[altrep(class = "MessageAltrep", manual)]
pub struct MessageAltrepData {
    pub len: usize,
    pub message: String,
}

impl AltrepLen for MessageAltrepData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltIntegerData for MessageAltrepData {
    fn elt(&self, _i: usize) -> i32 {
        miniextendr_api::message!("{}", self.message);
    }
}

/// Create an ALTREP integer that calls `message!()` on element access.
///
/// Same degradation as `altrep_warn_on_elt`.
///
/// @param n Length of the vector.
/// @param message Message body.
/// @return An ALTREP integer vector.
/// @export
#[miniextendr]
pub fn altrep_message_on_elt(n: i32, message: &str) -> SEXP {
    let data = MessageAltrepData {
        len: n.max(0) as usize,
        message: message.to_string(),
    };
    data.into_sexp()
}

#[derive(AltrepInteger)]
#[altrep(class = "ConditionAltrep", manual)]
pub struct ConditionAltrepData {
    pub len: usize,
    pub message: String,
}

impl AltrepLen for ConditionAltrepData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltIntegerData for ConditionAltrepData {
    fn elt(&self, _i: usize) -> i32 {
        miniextendr_api::condition!("{}", self.message);
    }
}

/// Create an ALTREP integer that calls `condition!()` on element access.
///
/// Same degradation as `altrep_warn_on_elt`.
///
/// @param n Length of the vector.
/// @param message Condition message body.
/// @return An ALTREP integer vector.
/// @export
#[miniextendr]
pub fn altrep_condition_on_elt(n: i32, message: &str) -> SEXP {
    let data = ConditionAltrepData {
        len: n.max(0) as usize,
        message: message.to_string(),
    };
    data.into_sexp()
}

// endregion

// region: DataErrorAltrep — error!(data = ...) from elt(), guard = RUnwind (issue #996 path 2)
//
// `with_r_unwind_protect_sourced` (the ALTREP `RUnwind` guard path) routes
// `RCondition::Error` through `raise_rust_condition_via_stop`, which builds
// `stop(structure(list(message = ..., call = ...), class = ...))` directly —
// there is no R wrapper here to splice `.val$data` the way
// `.miniextendr_raise_condition` does for the primary transport. Before
// #996 path 2, this dropped the `data =` fields entirely: class layering and
// message survived, `e$field_a` did not. `raise_rust_condition_via_stop` now
// takes an optional `ConditionData` and splices it into the condition list.

/// ALTREP integer that raises `error!(data = ...)` on element access.
#[derive(AltrepInteger)]
#[altrep(class = "DataErrorAltrep", manual)]
pub struct DataErrorAltrepData {
    pub len: usize,
    pub field_a: i32,
    pub message: String,
}

impl AltrepLen for DataErrorAltrepData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltIntegerData for DataErrorAltrepData {
    fn elt(&self, _i: usize) -> i32 {
        miniextendr_api::error!(
            class = "altrep_data_error",
            data = ("field_a", self.field_a),
            "{}",
            self.message
        );
    }
}

/// Create an ALTREP integer that raises `error!(data = ...)` on element access.
///
/// Regression fixture for issue #996 path 2: structured `data =` fields must
/// survive the ALTREP `RUnwind` degradation path
/// (`raise_rust_condition_via_stop`), not just the message and class.
///
/// @param n Length of the vector.
/// @param field_a Integer value carried in the `field_a` data field.
/// @param message Error message.
/// @return An ALTREP integer vector.
/// @export
#[miniextendr]
pub fn altrep_data_error_on_elt(n: i32, field_a: i32, message: &str) -> SEXP {
    let data = DataErrorAltrepData {
        len: n.max(0) as usize,
        field_a,
        message: message.to_string(),
    };
    data.into_sexp()
}

// endregion
