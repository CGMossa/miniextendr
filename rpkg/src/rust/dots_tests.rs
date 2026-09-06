//! Tests for R dots (`...`) handling.

use miniextendr_api::prelude::SexpExt;
use miniextendr_api::{miniextendr, typed_list};

#[miniextendr]
/// @title Dots Handling Tests
/// @name rpkg_dots
/// @description Dots (`...`) handling tests
/// @examples
/// \dontrun{
/// greetings_with_named_dots(a = 1, b = 2)
/// greetings_with_nameless_dots(1, 2, 3)
/// greetings_last_as_named_dots(1L, x = 1, y = 2)
/// }
/// @param ... Additional arguments (captured as dots).
pub fn greetings_with_named_dots(dots: ...) {
    let _ = dots;
}

/// Test dots handling with named but unused dots parameter.
/// @param ... Additional arguments (captured but unused).
#[miniextendr]
pub fn greetings_with_named_and_unused_dots(_dots: ...) {}

/// Test dots handling with nameless (underscore-prefixed) dots.
/// @param ... Additional arguments (captured but unused).
#[miniextendr]
pub fn greetings_with_nameless_dots(_dots: ...) {}

// LIMITATION: Good!
// #[miniextendr]
// fn greetings_with_dots_then_arg(dots: ..., exclamations: i32) {}

/// Test dots as the last parameter with a named but unused dots argument.
/// @param exclamations Integer count of exclamation marks.
/// @param ... Additional arguments (captured but unused).
#[miniextendr]
pub fn greetings_last_as_named_and_unused_dots(_exclamations: i32, _dots: ...) {}

/// Test dots as the last parameter with a named dots argument.
/// @param exclamations Integer count of exclamation marks.
/// @param ... Additional arguments (captured as dots).
#[miniextendr]
pub fn greetings_last_as_named_dots(_exclamations: i32, dots: ...) {
    let _ = dots;
}

/// Test dots as the last parameter with nameless dots.
/// @param exclamations Integer count of exclamation marks.
/// @param ... Additional arguments (captured but unused).
#[miniextendr]
pub fn greetings_last_as_nameless_dots(_exclamations: i32, _dots: ...) {}

// region: typed_list! macro examples

/// Test typed_list! validation with numeric, list, and optional character fields.
/// @param ... Named arguments: `alpha` (numeric vector of length 4), `beta` (list), `gamma` (optional character).
#[miniextendr]
pub fn validate_numeric_args(dots: ...) -> Result<i32, String> {
    let args = dots
        .typed(typed_list!(
            alpha => numeric(4),
            beta => list(),
            gamma? => character()
        ))
        .map_err(|e| e.to_string())?;

    // Get the raw SEXP and return its length
    let alpha = args.get_raw("alpha").map_err(|e| e.to_string())?;
    Ok(alpha.xlength() as i32)
}

/// Test typed_list! with exact mode (no extra fields allowed).
/// @param ... Named arguments: `x` (numeric), `y` (numeric). No extra fields allowed.
#[miniextendr]
pub fn validate_strict_args(dots: ...) -> Result<String, String> {
    let args = dots
        .typed(typed_list!(@exact; x => numeric(), y => numeric()))
        .map_err(|e| e.to_string())?;

    let x: f64 = args.get("x").map_err(|e| e.to_string())?;
    let y: f64 = args.get("y").map_err(|e| e.to_string())?;

    Ok(format!("x={}, y={}", x, y))
}

/// Test typed_list! validation with a class-typed field (data.frame).
/// @param ... Named arguments: `data` (data.frame).
#[miniextendr]
pub fn validate_class_args(dots: ...) -> Result<i32, String> {
    let args = dots
        .typed(typed_list!(data => "data.frame"))
        .map_err(|e| e.to_string())?;

    // Data frame is a list of columns, so Rf_xlength returns ncol
    let data = args.get_raw("data").map_err(|e| e.to_string())?;
    let ncol = data.xlength();

    Ok(ncol as i32)
}
// endregion

// region: Attribute sugar for typed_list validation

/// Test dots attribute sugar for typed_list validation (x and y numeric).
/// @param ... Named arguments: `x` (numeric), `y` (numeric).
#[miniextendr(dots = typed_list!(x => numeric(), y => numeric()))]
pub fn validate_with_attribute(_dots: ...) -> String {
    // dots_typed is automatically created by the attribute
    let x: f64 = dots_typed.get("x").expect("x");
    let y: f64 = dots_typed.get("y").expect("y");
    format!("x={}, y={}", x, y)
}

/// Test dots attribute sugar with an optional field (greeting).
/// @param ... Named arguments: `name` (character), `greeting` (optional character).
#[miniextendr(dots = typed_list!(name => character(), greeting? => character()))]
pub fn validate_attr_optional(_dots: ...) -> String {
    let name: String = dots_typed.get("name").expect("name");
    let greeting: Option<String> = dots_typed.get_opt("greeting").expect("greeting");
    let greeting = greeting.unwrap_or_else(|| "Hello".to_string());
    format!("{}, {}!", greeting, name)
}
// endregion

// region: TypedList::get / get_opt with VECTOR target types (golife hiccup #5)

/// Retrieve `numeric()` / `integer()` / `character()` typed_list fields as
/// *vectors* via `.get::<Vec<_>>()` and `.get_opt::<Vec<_>>()`.
///
/// Regression fixture for golife hiccup #5: before the fix, `get`/`get_opt`
/// required `T: TryFromSexp<Error = SexpError>`, which only scalar target types
/// satisfy — every vector `TryFromSexp` impl uses the narrower `SexpTypeError`,
/// so `.get::<Vec<i32>>()` failed to *compile*. The bound is now
/// `T: TryFromSexp` where `T::Error: Display`, so vector targets work too.
///
/// No arguments: the list is synthesised internally so the fixture is trivially
/// callable from R (and picked up by the no-arg export sweep). Returns a summary
/// string the testthat side asserts against.
#[miniextendr]
pub fn typed_list_get_vectors() -> Result<String, String> {
    use miniextendr_api::gc_protect::ProtectScope;
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::list::List;
    use miniextendr_api::typed_list::validate_list;

    let weights: Vec<f64> = vec![1.5, 2.5, 3.5];
    let survive: Vec<i32> = vec![10, 20, 30, 40];
    let labels: Vec<String> = vec!["a".to_string(), "b".to_string()];

    // Build the named list internally. Each field SEXP must be rooted before
    // `from_raw_pairs` assembles the container, and the container rooted before
    // validation allocates attributes — the gc_stress_typed_dataframe idiom.
    let scope = unsafe { ProtectScope::new() };
    let weights_sexp = unsafe { scope.protect_raw(weights.into_sexp()) };
    let survive_sexp = unsafe { scope.protect_raw(survive.into_sexp()) };
    let labels_sexp = unsafe { scope.protect_raw(labels.into_sexp()) };
    let list = List::from_raw_pairs(vec![
        ("weights", weights_sexp),
        ("survive", survive_sexp),
        ("labels", labels_sexp),
    ]);
    unsafe { scope.protect_raw(list.as_sexp()) };

    let spec = typed_list!(
        weights => numeric(),
        survive => integer(),
        labels? => character(),
        absent? => character()
    );
    let args = validate_list(list, &spec).map_err(|e| e.to_string())?;

    // The fix under test: vector target types now satisfy the get/get_opt bound.
    let weights_out: Vec<f64> = args.get("weights").map_err(|e| e.to_string())?;
    let survive_out: Vec<i32> = args.get("survive").map_err(|e| e.to_string())?;
    let labels_out: Option<Vec<String>> = args.get_opt("labels").map_err(|e| e.to_string())?;
    let absent_out: Option<Vec<String>> = args.get_opt("absent").map_err(|e| e.to_string())?;

    Ok(format!(
        "weights_sum={}, survive_sum={}, labels={:?}, absent_is_none={}",
        weights_out.iter().sum::<f64>(),
        survive_out.iter().sum::<i32>(),
        labels_out,
        absent_out.is_none(),
    ))
}
// endregion
