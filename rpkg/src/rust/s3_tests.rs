//! Tests for S3-style impl blocks (e.g., `#[miniextendr(s3)] impl Foo`).

use miniextendr_api::miniextendr;

/// A simple counter that demonstrates S3-style impl block support.
/// This gets exported as S3 methods: `new_s3counter()`, `value.S3Counter()`, etc.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S3Counter {
    value: i32,
}

/// S3 counter class with constructor, accessor, and mutation methods.
#[miniextendr(s3)]
impl S3Counter {
    /// Creates a new counter with the given initial value.
    /// @param initial Initial value for the counter.
    pub fn new(initial: i32) -> Self {
        S3Counter { value: initial }
    }

    /// Returns the current value (S3-specific method name to avoid conflicts).
    pub fn s3_value(&self) -> i32 {
        self.value
    }

    /// Increments the counter by 1 and returns the new value.
    pub fn s3_inc(&mut self) -> i32 {
        self.value += 1;
        self.value
    }

    /// Adds the given amount to the counter and returns the new value.
    /// @param amount The amount to add to the counter.
    pub fn s3_add(&mut self, amount: i32) -> i32 {
        self.value += amount;
        self.value
    }

    /// Extracts the counter's value through the S3 dollar operator.
    /// @param name The field name, `"value"`.
    #[miniextendr(s3(generic = "$"))]
    pub fn field(&self, name: &str) -> i32 {
        assert_eq!(name, "value", "unknown counter field");
        self.value
    }

    /// A static method that returns a default counter (value = 0).
    pub fn default_counter() -> Self {
        S3Counter { value: 0 }
    }
}

// region: S3NonGenericCollision — #1248 regression fixture

/// S3 class whose method name deliberately collides with the plain
/// (non-generic) `stats::var` closure. Before #1248 an `#[miniextendr(s3)]`
/// impl with this name installed cleanly but silently never dispatched: the
/// generic guard's bare `exists()` check saw `var` already bound to
/// `stats::var` and never created the `UseMethod` dispatcher, so `var(obj)`
/// kept resolving to `stats::var` instead of the registered
/// `var.S3NonGenericCollision` method. The usability classifier now shadows
/// the base closure with a package-local `UseMethod` generic and registers a
/// default method delegating to the masked `stats::var` via
/// `base::registerS3method()` (methods-table-only — no `var.default`
/// namespace binding), so ordinary (non-dispatching) calls like `var(1:10)`
/// keep working.
///
/// Mirrors the S7 `S7NonGenericCollision` fixture (#1114,
/// `rpkg/src/rust/s7_tests.rs`).
#[derive(miniextendr_api::ExternalPtr)]
pub struct S3NonGenericCollision {
    values: Vec<f64>,
}

/// S3 class exercising the #1248 base-name-collision fix.
#[miniextendr(s3)]
impl S3NonGenericCollision {
    /// @param values Numeric vector held by the gauge.
    pub fn new(values: Vec<f64>) -> Self {
        S3NonGenericCollision { values }
    }

    /// Collides with `stats::var`. Returns the *sum* (not the variance) so a
    /// test can tell our dispatched method apart from the masked
    /// `stats::var` fallback.
    pub fn var(&self) -> f64 {
        self.values.iter().sum()
    }
}

/// A second S3 class also defining `var`, to pin the reuse path: once the
/// first class's method shadows `stats::var` with a package-local
/// `UseMethod` generic, this class's guard must classify that generic as
/// already-usable (`isS3stdGeneric`) and reuse it as-is — not re-shadow it
/// (which would silently drop the first class's registered default
/// delegation).
#[derive(miniextendr_api::ExternalPtr)]
pub struct S3NonGenericCollisionSecond {
    values: Vec<f64>,
}

/// Second S3 class sharing the shadowed `var` generic with
/// [`S3NonGenericCollision`].
#[miniextendr(s3)]
impl S3NonGenericCollisionSecond {
    /// @param values Numeric vector held by the gauge.
    pub fn new(values: Vec<f64>) -> Self {
        S3NonGenericCollisionSecond { values }
    }

    /// Collides with `stats::var`, same as [`S3NonGenericCollision::var`].
    /// Returns the *product* (not the variance) so a test can distinguish
    /// dispatch to this class from the sibling class and from the masked
    /// `stats::var` fallback.
    pub fn var(&self) -> f64 {
        self.values.iter().product()
    }
}
// endregion

// region: S3 operator free functions — #1475 regression fixtures

/// Subsets an integer vector carrying the `mx_s3_vector` class.
/// @param x An integer vector with class `mx_s3_vector`.
/// @param i One-based integer indices.
/// @param ... Additional arguments (unused).
#[miniextendr(s3(generic = "[", class = "mx_s3_vector"))]
pub fn s3_operator_subset(x: Vec<i32>, i: Vec<i32>, _dots: ...) -> Vec<i32> {
    i.into_iter()
        .map(|index| x[usize::try_from(index - 1).expect("index must be positive")])
        .collect()
}

/// Extracts one integer from an `mx_s3_vector` object without a wrapper prelude.
/// @param x An integer vector with class `mx_s3_vector`.
/// @param i A one-based integer index.
/// @param ... Additional arguments (unused).
#[miniextendr(no_preconditions, s3(generic = "[[", class = "mx_s3_vector"))]
pub fn s3_operator_extract(x: Vec<i32>, i: i32, _dots: ...) -> i32 {
    x[usize::try_from(i - 1).expect("index must be positive")]
}
// endregion
