//! Classed `Result` errors, class vectors and the condition-data prefix
//! (#1434, #1435, #1440).
//!
//! - `RConditionError` on a package error enum: every `Err` raised from a
//!   `Result<T, PkgError>` return carries the member + family classes and the
//!   variant's fields as `e$<name>`, while the Rust side keeps `?` composition.
//! - `RError`: the ready-made classed value, built from a `std::error::Error`
//!   with `?` / `From` and decorated with `.class(..)` / `.data(..)` /
//!   `.data_prefix(..)`.
//! - `class = [..]` vectors on `rust_error!` / `warning!` / `rust_condition!`.
//! - `data_prefix = ".."` on the macros, letting fields named `kind` /
//!   `message` / `call` through as `e$<prefix>kind` etc.; without a prefix a
//!   computed reserved name is rejected at runtime (literal names fail to
//!   compile, see the `compile_fail` doctest on
//!   `miniextendr_api::condition::is_reserved_condition_field`).

use miniextendr_api::condition::{ConditionData, RConditionError, RError};
use miniextendr_api::{miniextendr, rust_condition, rust_error, warning};

// region: RConditionError on a package error enum

/// Stand-in for a downstream package's thiserror-style error enum.
#[derive(Debug)]
pub enum PkgError {
    MissingField { field: String },
    OutOfRange { value: f64, max: f64 },
}

impl std::fmt::Display for PkgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PkgError::MissingField { field } => write!(f, "field `{field}` is missing"),
            PkgError::OutOfRange { value, max } => write!(f, "{value} exceeds the maximum {max}"),
        }
    }
}

impl std::error::Error for PkgError {}

impl RConditionError for PkgError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn class(&self) -> Vec<String> {
        let member = match self {
            PkgError::MissingField { .. } => "pkg_error_missing_field",
            PkgError::OutOfRange { .. } => "pkg_error_out_of_range",
        };
        vec![member.to_string(), "pkg_error".to_string()]
    }

    fn data(&self) -> Option<ConditionData> {
        Some(match self {
            PkgError::MissingField { field } => vec![("field".to_string(), field.as_str().into())],
            PkgError::OutOfRange { value, max } => vec![
                ("value".to_string(), (*value).into()),
                ("max".to_string(), (*max).into()),
            ],
        })
    }
}

fn require_field(field: &str) -> Result<i32, PkgError> {
    if field.is_empty() {
        Err(PkgError::MissingField {
            field: "id".to_string(),
        })
    } else {
        Ok(field.len() as i32)
    }
}

/// `Result<i32, PkgError>`: empty `field` raises `pkg_error_missing_field`
/// with `e$field == "id"`; otherwise returns the length.
#[miniextendr]
pub fn classed_result_missing(field: &str) -> Result<i32, PkgError> {
    let n = require_field(field)?;
    Ok(n)
}

/// `Result<f64, PkgError>`: values above 100 raise `pkg_error_out_of_range`
/// with `e$value` and `e$max`.
#[miniextendr]
pub fn classed_result_range(value: f64) -> Result<f64, PkgError> {
    if value > 100.0 {
        return Err(PkgError::OutOfRange { value, max: 100.0 });
    }
    Ok(value)
}

/// Unit-returning `Result<(), PkgError>` (a different codegen arm).
#[miniextendr]
pub fn classed_result_unit(value: f64) -> Result<(), PkgError> {
    classed_result_range(value).map(|_| ())
}

/// The same error type raised from an S3 method (impl-block codegen arm).
#[derive(miniextendr_api::ExternalPtr)]
pub struct ClassedChecker {
    max: f64,
}

#[miniextendr(s3)]
impl ClassedChecker {
    /// A checker with an upper bound.
    /// @param max Upper bound.
    pub fn new(max: f64) -> Self {
        ClassedChecker { max }
    }

    /// Check `value` against the bound; raises `pkg_error_out_of_range`.
    /// @param value Value to check.
    pub fn check_bound(&self, value: f64) -> Result<f64, PkgError> {
        if value > self.max {
            return Err(PkgError::OutOfRange {
                value,
                max: self.max,
            });
        }
        Ok(value)
    }
}

// endregion

// region: RError

/// `Result<i32, RError>` built from a `ParseIntError` via `From`, then classed.
#[miniextendr]
pub fn rerror_parse(s: &str) -> Result<i32, RError> {
    let n: i32 = s.parse().map_err(|e| {
        RError::from(e)
            .class(["pkg_bad_number", "pkg_error"])
            .data("input", s)
    })?;
    Ok(n)
}

/// `RError` with `data_prefix("p_")` and fields named like the reserved slots.
#[miniextendr]
pub fn rerror_prefixed() -> Result<i32, RError> {
    Err(RError::new("prefixed fields")
        .class("pkg_prefixed")
        .data_prefix("p_")
        .data("kind", 7)
        .data("message", "inner")
        .data("call", "wrapped::step"))
}

/// `RError` whose field name is computed at runtime and reserved: raises a
/// plain `rust_error` about the reserved name instead of overwriting `e$kind`.
#[miniextendr]
pub fn rerror_reserved_runtime(name: &str) -> Result<i32, RError> {
    Err(RError::new("reserved field").data(name, 1))
}

/// Classless `RError`: same rendering as a plain `Result<_, String>` error.
#[miniextendr]
pub fn rerror_plain() -> Result<i32, RError> {
    Err(RError::new("plain rerror"))
}

// endregion

// region: class vectors and data_prefix on the macros

/// `rust_error!(class = [member, family], ...)`.
#[miniextendr]
pub fn classed_error_vec(member: &str, family: &str) {
    rust_error!(class = [member, family], "layered error");
}

/// `warning!` with a two-element class vector and data.
#[miniextendr]
pub fn classed_warning_vec(n: i32) -> i32 {
    warning!(
        class = ["pkg_warning_dropped", "pkg_warning"],
        data = ("dropped", n),
        "dropped {n} rows"
    );
    n
}

/// `rust_condition!` with a class vector supplied as a `Vec<String>`.
#[miniextendr]
pub fn classed_condition_vec(classes: Vec<String>) -> i32 {
    rust_condition!(class = classes, "signalled");
    1
}

/// `data_prefix` on the macro: `kind` / `message` arrive as `e$p_kind` /
/// `e$p_message`; the base `e$kind` / `e$message` are untouched.
#[miniextendr]
pub fn prefixed_data_macro(kind: i32) {
    rust_error!(
        class = "pkg_prefixed_macro",
        data_prefix = "p_",
        data = { kind = kind, message = "inner", call = "wrapped::step" },
        "prefixed macro fields"
    );
}

/// Computed field name without a prefix: the runtime reserved-name check fires.
#[miniextendr]
pub fn reserved_data_macro_runtime(name: &str) {
    rust_error!(data = (name, 1), "should not reach R with this field");
}

// endregion
