//! Tests for `vec_to_dataframe` all-`None` column downgrade, schema-upgrade
//! ordering, and recursive `Compound` union.
//!
//! When every row has `None` for an `Option<T>` field, the column lands as a
//! logical NA vector (LGLSXP) rather than `list(NULL, NULL, …)`.  R coerces
//! logical NA to the surrounding type on first use. The final region covers
//! #1370: a nested struct sub-field probed `None` on the first row (or a
//! `Compound` shape missing a key present in another row's shape) must still
//! resolve via the union across all rows, not just the first one seen.

use crate::serde::Serialize;
use miniextendr_api::dataframe::BuiltDataFrame;
use miniextendr_api::miniextendr;
use miniextendr_api::serde::vec_to_dataframe;
use std::collections::HashMap;

// region: Test types

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptU64 {
    name: String,
    stored: Option<u64>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptString {
    id: i32,
    label: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptBool {
    id: i32,
    flag: Option<bool>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct InnerStruct {
    x: f64,
    y: f64,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptUserStruct {
    id: i32,
    point: Option<InnerStruct>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptHashMap {
    id: i32,
    attrs: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptBytes {
    id: i32,
    data: Option<Vec<u8>>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithBytesAndOpt {
    raw: Vec<u8>,
    stored: Option<u64>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct InnerWithOpt {
    // No skip_serializing_if — size is always serialized so it appears in the
    // schema and can demonstrate the all-None downgrade via flatten.
    size: Option<u64>,
    name: String,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithFlattenedOptField {
    id: i32,
    #[serde(flatten)]
    inner: InnerWithOpt,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
#[serde(tag = "kind")]
enum EventWithOptX {
    A { x: Option<u64> },
    B { x: Option<u64> },
}

/// For compound-vs-compound different-shapes test.
#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptPoint {
    id: i32,
    point: Option<InnerStruct>,
}

/// Enum with two variants that have different nested struct shapes.
/// Used to verify existing-wins semantics for compound-vs-compound.
#[derive(Serialize)]
#[serde(crate = "crate::serde")]
#[serde(tag = "kind")]
enum EventDifferentNested {
    A { value: f64 },
    B { value: f64, extra: f64 },
}

// endregion

// region: All-None fixtures — downgrade fires

/// All-None `Option<u64>` column — single row (the dvs2 trigger case).
///
#[miniextendr(noexport)]
pub fn test_columnar_opt_u64_all_none_single() -> BuiltDataFrame {
    let rows = vec![WithOptU64 {
        name: "a".into(),
        stored: None,
    }];
    vec_to_dataframe(&rows).expect("from_rows")
}

/// All-None `Option<u64>` column — multiple rows.
///
#[miniextendr(noexport)]
pub fn test_columnar_opt_u64_all_none_multi() -> BuiltDataFrame {
    let rows = vec![
        WithOptU64 {
            name: "a".into(),
            stored: None,
        },
        WithOptU64 {
            name: "b".into(),
            stored: None,
        },
        WithOptU64 {
            name: "c".into(),
            stored: None,
        },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

/// All-None `Option<String>` column.
///
#[miniextendr(noexport)]
pub fn test_columnar_opt_string_all_none() -> BuiltDataFrame {
    let rows = vec![
        WithOptString { id: 1, label: None },
        WithOptString { id: 2, label: None },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

/// All-None `Option<bool>` column.
///
#[miniextendr(noexport)]
pub fn test_columnar_opt_bool_all_none() -> BuiltDataFrame {
    let rows = vec![
        WithOptBool { id: 1, flag: None },
        WithOptBool { id: 2, flag: None },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

/// All-None `Option<UserStruct>` — nested struct with all entries `None`.
///
/// When every row is `None`, the probe never sees any struct fields, so struct
/// expansion never fires.  The entire `point` field becomes a single logical NA
/// column under the field name `"point"` — not per-subfield `"point_x"`/`"point_y"`.
///
#[miniextendr(noexport)]
pub fn test_columnar_opt_user_struct_all_none() -> BuiltDataFrame {
    let rows = vec![
        WithOptUserStruct { id: 1, point: None },
        WithOptUserStruct { id: 2, point: None },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

/// All-None `Option<HashMap<…>>` — foreign generic, downgrade still fires.
///
#[miniextendr(noexport)]
pub fn test_columnar_opt_hashmap_all_none() -> BuiltDataFrame {
    let rows = vec![
        WithOptHashMap { id: 1, attrs: None },
        WithOptHashMap { id: 2, attrs: None },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

/// All-None `Option<Vec<u8>>` — downgrade fires (no values, no list semantics to preserve).
///
#[miniextendr(noexport)]
pub fn test_columnar_opt_bytes_all_none() -> BuiltDataFrame {
    let rows = vec![
        WithOptBytes { id: 1, data: None },
        WithOptBytes { id: 2, data: None },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

// endregion

// region: Mixed Some/None fixtures — downgrade must NOT fire

/// Mixed `Option<u64>`: some rows have values, no downgrade.
///
#[miniextendr(noexport)]
pub fn test_columnar_opt_u64_mixed() -> BuiltDataFrame {
    let rows = vec![
        WithOptU64 {
            name: "a".into(),
            stored: Some(42),
        },
        WithOptU64 {
            name: "b".into(),
            stored: None,
        },
        WithOptU64 {
            name: "c".into(),
            stored: Some(99),
        },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

/// Mixed `Option<String>`: some rows have values.
///
#[miniextendr(noexport)]
pub fn test_columnar_opt_string_mixed() -> BuiltDataFrame {
    let rows = vec![
        WithOptString {
            id: 1,
            label: Some("hello".into()),
        },
        WithOptString { id: 2, label: None },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

// endregion

// region: Vec<u8> with values — no downgrade (stays a list column)

/// `Vec<u8>` field with values — stays a list column regardless.
///
#[miniextendr(noexport)]
pub fn test_columnar_bytes_with_values() -> BuiltDataFrame {
    let rows = vec![
        WithOptBytes {
            id: 1,
            data: Some(vec![1u8, 2, 3]),
        },
        WithOptBytes {
            id: 2,
            data: Some(vec![4u8, 5]),
        },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

// endregion

// region: Mixed columns — bytes with values + optional all-None

/// `Vec<u8>` column with values alongside an all-None `Option<u64>` column.
/// The bytes column stays a list; the optional column downgrades to logical NA.
///
#[miniextendr(noexport)]
pub fn test_columnar_bytes_and_opt_none() -> BuiltDataFrame {
    let rows = vec![
        WithBytesAndOpt {
            raw: vec![1u8, 2],
            stored: None,
        },
        WithBytesAndOpt {
            raw: vec![3u8],
            stored: None,
        },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

// endregion

// region: Flatten with all-None inner field

/// `#[serde(flatten)]` with all-None inner field: the flattened optional field
/// becomes a logical NA column.
///
#[miniextendr(noexport)]
pub fn test_columnar_flatten_all_none() -> BuiltDataFrame {
    let rows = vec![
        WithFlattenedOptField {
            id: 1,
            inner: InnerWithOpt {
                size: None,
                name: "a".into(),
            },
        },
        WithFlattenedOptField {
            id: 2,
            inner: InnerWithOpt {
                size: None,
                name: "b".into(),
            },
        },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

// endregion

// region: Enum union: all-variant-A with x = None, then one variant-B with x = Some

/// Enum: all variant-A rows have `x = None` → logical NA column.
///
#[miniextendr(noexport)]
pub fn test_columnar_enum_all_none() -> BuiltDataFrame {
    let rows = vec![EventWithOptX::A { x: None }, EventWithOptX::A { x: None }];
    vec_to_dataframe(&rows).expect("from_rows")
}

/// Enum: variant-A rows with `x = None`, then one variant-B row with `x = Some(42)`.
///
/// With the two-phase discovery, the probe scans all rows before resolving the
/// schema.  Variant-B's `x = Some(42u64)` contributes a `Scalar(Real)` candidate
/// which beats the `Scalar(Generic)` from variant-A's `x = None`.  The column
/// ends up as a numeric vector with `NA` in row 1.
///
#[miniextendr(noexport)]
pub fn test_columnar_enum_some_flips_type() -> BuiltDataFrame {
    let rows = vec![
        EventWithOptX::A { x: None },
        EventWithOptX::B { x: Some(42) },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

// endregion

// region: Schema upgrade — first-row-None then Some

/// First-row `x = None`, second row `x = Some(42u64)`.
///
/// Two-phase discovery: the `Scalar(Real)` candidate from row 2 beats
/// `Scalar(Generic)` from row 1.  Result: numeric column with NA at index 1.
///
#[miniextendr(noexport)]
pub fn test_columnar_schema_upgrade_scalar() -> BuiltDataFrame {
    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct Row {
        x: Option<u64>,
    }
    let rows = vec![Row { x: None }, Row { x: Some(42) }];
    vec_to_dataframe(&rows).expect("from_rows")
}

/// First-row `point = None`, second row `point = Some(Point{x:1.0, y:2.0})`.
///
/// Two-phase discovery: `Compound` candidate from row 2 beats `Scalar(Generic)`
/// from row 1.  Result: columns `point_x` and `point_y`, with NA in row 1.
///
#[miniextendr(noexport)]
pub fn test_columnar_schema_upgrade_nested() -> BuiltDataFrame {
    let rows = vec![
        WithOptPoint { id: 1, point: None },
        WithOptPoint {
            id: 2,
            point: Some(InnerStruct { x: 1.0, y: 2.0 }),
        },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

/// Multiple leading None rows before a Some value.
///
/// Rows: None, None, Some(42u64), None.  Two-phase discovery resolves the
/// column to `Scalar(Real)`.  Result: numeric column with NA at positions 1, 2, 4.
///
#[miniextendr(noexport)]
pub fn test_columnar_schema_upgrade_multi_none_first() -> BuiltDataFrame {
    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct Row {
        x: Option<u64>,
    }
    let rows = vec![
        Row { x: None },
        Row { x: None },
        Row { x: Some(42) },
        Row { x: None },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

/// Compound-vs-compound with different struct shapes: existing-wins semantics.
///
/// Rows alternate between variant A (has `value` only) and variant B (has `value`
/// and `extra`).  The `extra` field only appears in B rows; `value` appears in both.
/// Because both rows contribute scalar candidates for `value`, the first non-Generic
/// wins.  The schema discovers both `value` and `extra` fields (from their respective
/// rows) because they are distinct keys — they are *not* the same compound key.
///
/// This fixture tests that the per-key candidate accumulation works correctly for
/// an enum with fields that differ between variants.
///
/// (Formerly a "TODO (union recursion)" placeholder here: when a single *key* maps
/// to two different `Compound` shapes for the same row set, the first `Compound`
/// silently won. Recursive `Compound` union is now implemented — see
/// [#1370](https://github.com/A2-ai/miniextendr/issues/1370) — and the
/// same-key case is covered directly by
/// [`test_columnar_nested_subfield_none_first`] and
/// [`test_columnar_nested_compound_union_of_keys`] below.)
///
#[miniextendr(noexport)]
pub fn test_columnar_compound_different_shapes() -> BuiltDataFrame {
    let rows = vec![
        EventDifferentNested::A { value: 1.0 },
        EventDifferentNested::B {
            value: 2.0,
            extra: 3.0,
        },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

// endregion

// region: Recursive Compound union — same key, different shapes (#1370)

/// Nested struct whose `variant` sub-field is `Option<String>` — used to
/// reproduce #1370's exact repro shape (a sub-field literally named `variant`,
/// as in the issue, probed `None` on the first row).
#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct MetaWithOptVariant {
    variant: Option<String>,
    size: f64,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithNestedOptVariant {
    id: i32,
    meta: MetaWithOptVariant,
}

/// Nested struct with a `#[serde(skip_serializing_if)]` sub-field: some rows'
/// probe of `meta` omits `extra` entirely (a genuinely different `Compound`
/// shape — one fewer key — not just a differently-typed same key).
#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct MetaMaybeExtra {
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<f64>,
    size: f64,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithNestedMaybeExtra {
    id: i32,
    meta: MetaMaybeExtra,
}

/// #1370 repro: nested sub-field `meta.variant` is `None` on the *first* row,
/// typed (`Some`) on the second. Pre-fix this locked `meta_variant` to a
/// Generic (list) column; post-fix it resolves to `character` with `NA` at
/// row 1, matching what a `Some`-first ordering already produced.
#[miniextendr(noexport)]
pub fn test_columnar_nested_subfield_none_first() -> BuiltDataFrame {
    let rows = vec![
        WithNestedOptVariant {
            id: 1,
            meta: MetaWithOptVariant {
                variant: None,
                size: 4.0,
            },
        },
        WithNestedOptVariant {
            id: 2,
            meta: MetaWithOptVariant {
                variant: Some("x".into()),
                size: 5.0,
            },
        },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

/// Same rows as [`test_columnar_nested_subfield_none_first`], reversed
/// (`Some`-first). The emitted schema must be identical regardless of row
/// order — this is the order-independence half of the #1370 regression.
#[miniextendr(noexport)]
pub fn test_columnar_nested_subfield_some_first() -> BuiltDataFrame {
    let rows = vec![
        WithNestedOptVariant {
            id: 2,
            meta: MetaWithOptVariant {
                variant: Some("x".into()),
                size: 5.0,
            },
        },
        WithNestedOptVariant {
            id: 1,
            meta: MetaWithOptVariant {
                variant: None,
                size: 4.0,
            },
        },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

/// #1370 "union of keys" case: the *same* key (`meta`) maps to two genuinely
/// different `Compound` shapes across rows — row 1's probe omits `extra`
/// entirely (`skip_serializing_if` fires), row 2's includes it. Pre-fix, row
/// 2's `extra` sub-field was silently dropped (first-Compound-wins); post-fix
/// both `meta_extra` and `meta_size` are discovered.
#[miniextendr(noexport)]
pub fn test_columnar_nested_compound_union_of_keys() -> BuiltDataFrame {
    let rows = vec![
        WithNestedMaybeExtra {
            id: 1,
            meta: MetaMaybeExtra {
                extra: None,
                size: 1.0,
            },
        },
        WithNestedMaybeExtra {
            id: 2,
            meta: MetaMaybeExtra {
                extra: Some(3.0),
                size: 2.0,
            },
        },
    ];
    vec_to_dataframe(&rows).expect("from_rows")
}

// endregion
