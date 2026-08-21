//! Fixture for documenting noexported R6 active bindings without warnings.
//!
//! roxygen2 8.0.0 still reports a binding documented as `@field name NULL` as
//! undocumented. A method tagged with `#[miniextendr(noexport)]` or
//! `#[miniextendr(internal)]` therefore gets a minimal class-level
//! `@field name (internal)` description.

use miniextendr_api::miniextendr;

/// An R6 class demonstrating internal active-binding documentation.
///
/// `R6SensorReading` has two active bindings:
///
/// - `value`: exported, documents in the generated `.Rd`.
/// - `raw_bytes`: tagged `noexport`; gets a class-level
///   `@field raw_bytes (internal)` entry while remaining available at runtime.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6SensorReading {
    value: f64,
    raw: i32,
}

/// R6 sensor reading with one documented and one undocumented active binding.
/// @param value Numeric sensor value.
/// @param raw Integer raw ADC reading.
#[miniextendr(r6)]
impl R6SensorReading {
    /// Creates a new sensor reading.
    pub fn new(value: f64, raw: i32) -> Self {
        R6SensorReading { value, raw }
    }

    /// The calibrated sensor value (exported active binding).
    #[miniextendr(r6(active))]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Internal raw ADC reading — not part of the public API.
    #[miniextendr(r6(active), noexport)]
    pub fn raw_bytes(&self) -> i32 {
        self.raw
    }
}
