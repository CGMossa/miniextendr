//! Standalone S3 methods on non-syntactic generics (`[`, `$`, and `[[` from an
//! impl block) plus `@describeIn` documentation written in Rust doc comments
//! (#1475, #1476).
//!
//! `mx_bag` is a plain classed double vector created in R
//! (`structure(c(1, 2, 3), class = "mx_bag")`); only its methods live here. The
//! generated wrapper names are not syntactic R (`[.mx_bag`), so the wrappers
//! file has to backtick-quote them or it does not parse.

use miniextendr_api::miniextendr;

// region: standalone S3 methods on operator generics (#1475)

/// Subset a bag by 1-based integer positions.
///
/// @param x A `mx_bag`.
/// @param i Integer positions to keep.
/// @param ... Ignored.
/// @export
#[miniextendr(s3(generic = "[", class = "mx_bag"))]
pub fn mx_bag_subset(x: Vec<f64>, i: Vec<i32>, _dots: ...) -> Result<Vec<f64>, String> {
    i.iter()
        .map(|&k| {
            usize::try_from(k)
                .ok()
                .and_then(|k| k.checked_sub(1))
                .and_then(|k| x.get(k).copied())
                .ok_or_else(|| format!("index {k} out of range for a bag of {}", x.len()))
        })
        .collect()
}

/// Named summaries via `$`: `bag$n` and `bag$sum`.
///
/// @param x A `mx_bag`.
/// @param name Either `"n"` or `"sum"`.
/// @export
#[miniextendr(s3(generic = "$", class = "mx_bag"))]
pub fn mx_bag_dollar(x: Vec<f64>, name: String) -> Result<f64, String> {
    match name.as_str() {
        "n" => Ok(f64::from(u32::try_from(x.len()).unwrap_or(u32::MAX))),
        "sum" => Ok(x.iter().sum()),
        other => Err(format!("unknown field `{other}`; use `n` or `sum`")),
    }
}

// endregion

// region: impl-block S3 class with a `[[` generic override (#1475)

/// Handle-backed bag whose element access is the `[[` generic.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MxBagHandle {
    values: Vec<f64>,
}

#[miniextendr(s3)]
impl MxBagHandle {
    /// @param values Numeric values held by the handle.
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }

    /// Element at 1-based position `i`.
    #[miniextendr(s3(generic = "[["))]
    pub fn at(&self, i: i32) -> Result<f64, String> {
        usize::try_from(i)
            .ok()
            .and_then(|k| k.checked_sub(1))
            .and_then(|k| self.values.get(k).copied())
            .ok_or_else(|| format!("index {i} out of range for {} values", self.values.len()))
    }

    /// Number of values.
    pub fn size(&self) -> i32 {
        i32::try_from(self.values.len()).unwrap_or(i32::MAX)
    }
}

// endregion

// region: @describeIn from a Rust doc comment (#1476)

/// Sum of a bag.
///
/// Numeric summaries of `mx_bag` values. `mx_bag_len()` is documented on this
/// page through `@describeIn`, written in its Rust doc comment. roxygen2 sends
/// a `@describeIn` block to the destination *object's own* topic, so the
/// destination has to live on that page: `@rdname mx_bag_sum` keeps it off the
/// file-stem page this module would otherwise share.
///
/// @rdname mx_bag_sum
/// @param x A bag (numeric vector).
/// @return A double scalar.
/// @export
#[miniextendr]
pub fn mx_bag_sum(x: Vec<f64>) -> f64 {
    x.iter().sum()
}

/// @describeIn mx_bag_sum Number of values in the bag,
///   as an integer scalar. This sentence sits on a wrapped line and only
///   reaches the Rd page when the tag keeps its continuation lines.
/// @param x A bag (numeric vector).
/// @export
#[miniextendr]
pub fn mx_bag_len(x: Vec<f64>) -> i32 {
    i32::try_from(x.len()).unwrap_or(i32::MAX)
}

// endregion
