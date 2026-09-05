//! Tests for `#[miniextendr(internal)]` and `#[miniextendr(noexport)]` attributes.

use miniextendr_api::miniextendr;

/// A normal exported function for comparison.
///
/// @export
#[miniextendr]
pub fn export_control_normal() -> &'static str {
    "normal"
}

/// An internal function -- callable but not exported, with @keywords internal.
#[miniextendr(internal)]
pub fn export_control_internal() -> &'static str {
    "internal"
}

/// A noexport function -- callable but not exported, no @keywords internal.
#[miniextendr(noexport)]
pub fn export_control_noexport() -> &'static str {
    "noexport"
}

// region: vctrs class doc gating (#1180)
//
// The vctrs generator must gate every doc-emitting block on
// `@noRd || (noexport && !internal)` like the other five class generators —
// self-coercion methods (`vec_ptype2` / `vec_cast` / `vec_ptype_abbr`) and
// static helpers of a gated class must contribute no Rd aliases, while the
// R functions stay callable via `:::`. Asserted by
// tests/testthat/test-vctrs-export-control.R.

/// Marker struct — vctrs classes carry no Rust state; the vector payload
/// returned by `new` IS the R object (wrapped by `vctrs::new_vctr`).
pub struct VctrsNoexportGated;

/// A noexport vctrs class: no `export()` entries and no Rd aliases for its
/// self-coercion methods or static helpers.
#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "nxg"), noexport)]
impl VctrsNoexportGated {
    /// Vctrs ctors return the vector payload (not Self) — vctrs::new_vctr wraps it.
    /// @param values Numeric payload.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(values: Vec<f64>) -> Vec<f64> {
        values
    }

    /// Static helper — must not land on any Rd page for a noexport class.
    /// @param values Numeric payload.
    pub fn payload_sum(values: Vec<f64>) -> f64 {
        values.iter().sum()
    }
}

/// Marker struct for the `@noRd` variant of the gate.
pub struct VctrsNoRdGated;

/// @noRd
#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "nrd"))]
impl VctrsNoRdGated {
    /// Vctrs ctors return the vector payload (not Self) — vctrs::new_vctr wraps it.
    /// @param values Numeric payload.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(values: Vec<f64>) -> Vec<f64> {
        values
    }
}

// endregion

// region: postfix: internal entry points named from the Rust name (#1451)

/// Internal entry point for the hand-written `export_control_delegate()` in
/// `R/export_control.R`: the R wrapper is `export_control_delegate_impl`, the
/// C symbol keeps the Rust name. `noexport` keeps it out of NAMESPACE.
#[miniextendr(noexport, postfix = "_impl")]
pub fn export_control_delegate(x: i32) -> i32 {
    x * 2
}

/// R6 class whose method is renamed through `postfix`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct ExportControlWidget {
    n: i32,
}

/// Widget for the method-level `postfix` fixture.
/// @param n Starting value.
#[miniextendr(r6)]
impl ExportControlWidget {
    /// Creates a widget.
    pub fn new(n: i32) -> Self {
        ExportControlWidget { n }
    }

    /// Adds `by` to the stored value; the R method is `bump_impl`.
    /// @param by Increment.
    #[miniextendr(postfix = "_impl")]
    pub fn bump(&self, by: i32) -> i32 {
        self.n + by
    }
}

// endregion
