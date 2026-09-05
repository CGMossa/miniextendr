//! Demo fixtures for the condition macro system.
//!
//! These functions exercise `error!()`, `warning!()`, `message!()`, and
//! `condition!()` macros including the optional `class = "..."` form.
//! Tests live in `rpkg/tests/testthat/test-conditions.R`.

use miniextendr_api::miniextendr;

// Type alias avoids ambiguous-associated-type errors when using enum variants
// (RCondition impls TryFrom/IntoR which have `Error`/`Condition` assoc types).
type RCondition = miniextendr_api::condition::RCondition;

// region: error! fixtures

/// Raise a rust_error with the standard class layering.
///
/// @export
#[miniextendr]
pub fn demo_error(msg: &str) {
    miniextendr_api::error!("{msg}");
}

/// Raise a rust_error with a custom class prepended.
///
/// @export
#[miniextendr]
pub fn demo_error_custom_class(class: &str, msg: &str) {
    // Can't use a runtime string as the `class =` argument in the macro because
    // the macro takes a literal. Use the enum directly for the variable-class case.
    std::panic::panic_any(RCondition::Error {
        message: msg.to_string(),
        class: vec![class.to_string()],
        data: None,
    });
}

// endregion

// region: warning! fixtures

/// Raise a rust_warning.
///
/// @export
#[miniextendr]
pub fn demo_warning(msg: &str) {
    miniextendr_api::warning!("{msg}");
}

/// Raise a rust_warning with a custom class prepended.
///
/// @export
#[miniextendr]
pub fn demo_warning_custom_class(class: &str, msg: &str) {
    std::panic::panic_any(RCondition::Warning {
        message: msg.to_string(),
        class: vec![class.to_string()],
        data: None,
    });
}

// endregion

// region: message! fixtures

/// Emit a rust_message.
///
/// @export
#[miniextendr]
pub fn demo_message(msg: &str) {
    miniextendr_api::message!("{msg}");
}

// endregion

// region: condition! fixtures

/// Signal a generic rust_condition (no-op if unhandled).
///
/// @export
#[miniextendr]
pub fn demo_condition(msg: &str) {
    miniextendr_api::condition!("{msg}");
}

/// Signal a rust_condition with a custom class.
///
/// @export
#[miniextendr]
pub fn demo_condition_custom_class(class: &str, msg: &str) {
    std::panic::panic_any(RCondition::Condition {
        message: msg.to_string(),
        class: vec![class.to_string()],
        data: None,
    });
}

// endregion

// region: data = ... payload fixtures (issue #346)

/// Raise a classed error carrying a single structured field (`value`).
///
/// Handlers can read `e$value` instead of parsing the message string.
///
/// @export
#[miniextendr]
pub fn demo_error_data_scalar(value: i32) {
    miniextendr_api::error!(
        class = "range_error",
        data = ("value", value),
        "value {value} out of range"
    );
}

/// Raise a classed error carrying several heterogeneous scalar fields.
///
/// Demonstrates the bracketed `data = [(..), (..)]` form with mixed value
/// types: integer, double, logical, and character.
///
/// @export
#[miniextendr]
pub fn demo_error_data_multi(value: f64, code: i32, label: &str) {
    miniextendr_api::error!(
        class = "validation_error",
        data = [
            ("value", value),
            ("code", code),
            ("label", label),
            ("fatal", true)
        ],
        "validation failed for {label}"
    );
}

/// Raise a classed error whose data field is a vector.
///
/// @export
#[miniextendr]
pub fn demo_error_data_vector(values: Vec<i32>) {
    miniextendr_api::error!(
        class = "batch_error",
        data = ("offending", values),
        "batch contained out-of-range values"
    );
}

/// Raise a classed warning carrying structured data.
///
/// @export
#[miniextendr]
pub fn demo_warning_data(count: i32) {
    miniextendr_api::warning!(
        class = "truncation_warning",
        data = ("dropped", count),
        "dropped {count} rows"
    );
}

/// Emit a message carrying structured data.
///
/// @export
#[miniextendr]
pub fn demo_message_data(step: i32) {
    miniextendr_api::message!(data = ("step", step), "progress at step {step}");
}

/// Signal a classed condition carrying structured data.
///
/// @export
#[miniextendr]
pub fn demo_condition_data(n: i32) {
    miniextendr_api::condition!(
        class = "progress",
        data = ("processed", n),
        "processed {n} items"
    );
}

// endregion

// region: data = ... richer value types + keyed builder (issue #995)

/// Raise an error carrying NA-aware `Option` fields.
///
/// `present` rides through as a value; `missing = NULL` lands as R `NA`. The
/// field is *present* on the condition object but its value is `NA` — distinct
/// from a wholly absent field.
///
/// @export
#[miniextendr]
pub fn demo_error_data_option(present: i32, has_value: bool) {
    // Bare `Option<i32>` values ride through via `RValue: From<Option<i32>>`:
    // `None` materialises as `NA_integer_`.
    let opt: Option<i32> = if has_value { Some(present) } else { None };
    miniextendr_api::error!(
        class = "option_error",
        data = [("present", Some(present)), ("missing", opt)],
        "option payload"
    );
}

/// Raise an error whose vector field carries embedded `NA` elements.
///
/// @export
#[miniextendr]
pub fn demo_error_data_na_vector() {
    miniextendr_api::error!(
        class = "na_vector_error",
        data = ("codes", vec![Some(1_i32), None, Some(3)]),
        "vector with NA"
    );
}

/// Raise an error carrying a wide integer (`i64`) field.
///
/// Values within `i32` range materialise as `integer(1)`; larger values become
/// `double(1)` via the smart wide-integer ladder.
///
/// @export
#[miniextendr]
pub fn demo_error_data_long(value: f64) {
    // f64 argument lets R pass values beyond i32 range; the `RValue: From<i64>`
    // wide-integer ladder narrows to integer(1) when it fits, double(1) otherwise.
    let as_long = value as i64;
    miniextendr_api::error!(
        class = "long_error",
        data = ("big", as_long),
        "wide integer payload"
    );
}

/// Raise an error carrying a nested named list under `details`.
///
/// Handlers read `e$details$min` / `e$details$max`.
///
/// @export
#[miniextendr]
pub fn demo_error_data_nested(min: i32, max: i32) {
    use miniextendr_api::RValue;
    let nested: Vec<(Option<String>, RValue)> = vec![
        (Some("min".to_string()), RValue::from(min)),
        (Some("max".to_string()), RValue::from(max)),
    ];
    miniextendr_api::error!(
        class = "nested_error",
        data = ("details", RValue::List(nested)),
        "nested payload"
    );
}

/// Raise an error using the `Debug`-stringify fallback variant.
///
/// The Rust `RangeInclusive` has no R-native mapping, so it rides along as a
/// `character(1)` of its `{:?}` rendering.
///
/// @export
#[miniextendr]
pub fn demo_error_data_debug(lo: i32, hi: i32) {
    use miniextendr_api::RValue;
    miniextendr_api::error!(
        class = "debug_error",
        data = ("range", RValue::debug(lo..=hi)),
        "debug payload"
    );
}

/// Raise an error using the keyed builder sugar (`data = { k = v, ... }`).
///
/// @export
#[miniextendr]
pub fn demo_error_data_keyed(value: i32, code: i32) {
    miniextendr_api::error!(
        class = "keyed_error",
        data = { value = value, code = code },
        "keyed payload"
    );
}

// endregion

// region: NA-scalar regression pin (issue #1103)

/// Raise a classed error whose scalar `count` field is `NA_integer_`.
///
/// Regression pin for issue #1103: the field is *present* on the condition
/// object with value `NA`, not silently dropped. #1103 described this as a
/// limitation of the old `ConditionDataValue` type (no `Option`-bearing
/// variants); that type no longer exists — `ConditionData` is
/// `Vec<(String, RValue)>` and `RValue`'s scalar variants are `Option`-aware
/// (`Vec<Option<i32>>` etc.), so `None` already round-trips as `NA` rather
/// than being dropped. This fixture pins that behaviour directly against the
/// issue's own example shape (`data = ("count", <NA integer>)`).
///
/// @export
#[miniextendr]
pub fn demo_error_data_na_scalar() {
    let count: Option<i32> = None;
    miniextendr_api::error!(
        class = "na_scalar_error",
        data = ("count", count),
        "count is NA"
    );
}

// endregion
