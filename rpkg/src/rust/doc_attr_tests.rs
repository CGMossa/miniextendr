//! Tests for `#[miniextendr(doc = "...")]` attribute.

use miniextendr_api::miniextendr;

/// This doc comment should be replaced by the custom doc.
#[miniextendr(
    doc = "@title Custom title from doc attr\n@description This description was set via the doc attribute.\n@param x A numeric value.\n@return The value doubled."
)]
pub fn doc_attr_basic(x: f64) -> f64 {
    x * 2.0
}

#[miniextendr(doc = "@title No-param doc\n@description A function with custom doc and no params.")]
pub fn doc_attr_no_params() -> &'static str {
    "hello from doc_attr"
}

// region: Explicit shared pages and source order — #1476

/// @name
/// doc_shared_topic
/// @title Shared documentation
/// in source order.
/// @description Shared-page fixtures for standalone and S3 functions.
/// @keywords utilities
/// methods
/// @concept shared documentation
/// fixture
/// @param x An integer vector.
#[miniextendr]
pub fn doc_shared_topic(x: Vec<i32>) -> Vec<i32> {
    x
}

/// @describeIn doc_shared_topic Doubles each input value
/// while retaining the input order.
/// @param x An integer vector.
#[miniextendr]
pub fn doc_shared_double(x: Vec<i32>) -> Vec<i32> {
    x.into_iter().map(|value| value * 2).collect()
}

/// @describeIn doc_shared_topic Formats each input value
/// with the shared documentation fixture.
/// @param x An integer vector.
/// @param ... Additional arguments (unused).
#[miniextendr(s3(generic = "format", class = "doc_shared_vector"))]
pub fn doc_shared_format(x: Vec<i32>, _dots: ...) -> Vec<String> {
    x.into_iter().map(|value| value.to_string()).collect()
}
// endregion
