//! Cardinality × payload-shape matrix for enum-derived data frames.
//!
//! For each currently-supported payload shape, exercise four cardinality cells
//! against both `to_dataframe` (align/NA-fill) and `into_dataframe_split`
//! (per-variant partition, from the `IntoDataFrameSplit` trait):
//!   - 1v1r: one variant, one row  → split returns a 1-row data.frame in that
//!     variant's slot, and a 0-row data.frame in every other slot
//!   - 1vNr: one variant, many rows
//!   - Nv1r: many variants, one row each
//!   - NvNr: many variants, many rows each
//!
//! A single-variant enum is also exposed for 1v1r / 1vNr to verify the
//! bare-data.frame return path of `into_dataframe_split`.
//!
//! `HashMap`/`BTreeMap` enum payloads are exercised in the map-fields section (issue #457).
//! Nested-enum payloads and struct-in-variant payloads are tracked by GH issues #458 / #459.
//! `&str` and `&[T]` enum payloads are exercised in the borrowed-string section below.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use miniextendr_api::{BuiltDataFrame, IntoDataFrame, IntoDataFrameSplit};
use miniextendr_api::{DataFrameRow, IntoList, List, miniextendr};

// region: 0a. Vec<i32> opaque (no expand/width → list-column)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum VecOpaqueEvent {
    Items { label: String, items: Vec<i32> },
    NoItems { label: String },
}

fn vec_opaque_payload(label: &str, items: Vec<i32>) -> VecOpaqueEvent {
    VecOpaqueEvent::Items {
        label: label.into(),
        items,
    }
}

#[miniextendr]
pub fn vec_opaque_split_1v1r() -> List {
    vec![vec_opaque_payload("a", vec![1, 2, 3])].into_dataframe_split()
}

#[miniextendr]
pub fn vec_opaque_split_1vnr() -> List {
    vec![
        vec_opaque_payload("a", vec![1, 2, 3]),
        vec_opaque_payload("b", vec![4, 5]),
        vec_opaque_payload("c", vec![]),
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_opaque_split_nv1r() -> List {
    vec![
        vec_opaque_payload("a", vec![1, 2, 3]),
        VecOpaqueEvent::NoItems { label: "b".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_opaque_align_nvnr() -> BuiltDataFrame {
    vec![
        vec_opaque_payload("a", vec![1, 2, 3]),
        VecOpaqueEvent::NoItems { label: "b".into() },
        vec_opaque_payload("c", vec![4, 5]),
        VecOpaqueEvent::NoItems { label: "d".into() },
    ]
    .into_dataframe()
    .unwrap()
}

#[miniextendr]
pub fn vec_opaque_split_nvnr() -> List {
    vec![
        vec_opaque_payload("a", vec![1, 2, 3]),
        VecOpaqueEvent::NoItems { label: "b".into() },
        vec_opaque_payload("c", vec![4, 5]),
        VecOpaqueEvent::NoItems { label: "d".into() },
    ]
    .into_dataframe_split()
}

// endregion

// region: 0b. HashSet<String> opaque (list-column, unordered elements)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum HashSetEvent {
    Tagged { id: i32, tags: HashSet<String> },
    Untagged { id: i32 },
}

fn hashset_payload(id: i32, tags: &[&str]) -> HashSetEvent {
    HashSetEvent::Tagged {
        id,
        tags: tags.iter().map(|s| s.to_string()).collect(),
    }
}

#[miniextendr]
pub fn hashset_split_1v1r() -> List {
    vec![hashset_payload(1, &["a", "b"])].into_dataframe_split()
}

#[miniextendr]
pub fn hashset_split_1vnr() -> List {
    vec![
        hashset_payload(1, &["a", "b"]),
        hashset_payload(2, &["c"]),
        hashset_payload(3, &[]),
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn hashset_split_nv1r() -> List {
    vec![
        hashset_payload(1, &["a", "b"]),
        HashSetEvent::Untagged { id: 2 },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn hashset_align_nvnr() -> BuiltDataFrame {
    vec![
        hashset_payload(1, &["a", "b"]),
        HashSetEvent::Untagged { id: 2 },
        hashset_payload(3, &["c"]),
        HashSetEvent::Untagged { id: 4 },
    ]
    .into_dataframe()
    .unwrap()
}

#[miniextendr]
pub fn hashset_split_nvnr() -> List {
    vec![
        hashset_payload(1, &["a", "b"]),
        HashSetEvent::Untagged { id: 2 },
        hashset_payload(3, &["c"]),
        HashSetEvent::Untagged { id: 4 },
    ]
    .into_dataframe_split()
}

// endregion

// region: 0c. BTreeSet<i32> opaque (list-column, sorted elements)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BTreeSetEvent {
    Cats { label: String, cats: BTreeSet<i32> },
    NoCats { label: String },
}

fn btreeset_payload(label: &str, cats: &[i32]) -> BTreeSetEvent {
    BTreeSetEvent::Cats {
        label: label.into(),
        cats: cats.iter().copied().collect(),
    }
}

#[miniextendr]
pub fn btreeset_split_1v1r() -> List {
    vec![btreeset_payload("a", &[3, 1, 2])].into_dataframe_split()
}

#[miniextendr]
pub fn btreeset_split_1vnr() -> List {
    vec![
        btreeset_payload("a", &[3, 1, 2]),
        btreeset_payload("b", &[5, 4]),
        btreeset_payload("c", &[]),
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn btreeset_split_nv1r() -> List {
    vec![
        btreeset_payload("a", &[3, 1, 2]),
        BTreeSetEvent::NoCats { label: "b".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn btreeset_align_nvnr() -> BuiltDataFrame {
    vec![
        btreeset_payload("a", &[3, 1, 2]),
        BTreeSetEvent::NoCats { label: "b".into() },
        btreeset_payload("c", &[5, 4]),
        BTreeSetEvent::NoCats { label: "d".into() },
    ]
    .into_dataframe()
    .unwrap()
}

#[miniextendr]
pub fn btreeset_split_nvnr() -> List {
    vec![
        btreeset_payload("a", &[3, 1, 2]),
        BTreeSetEvent::NoCats { label: "b".into() },
        btreeset_payload("c", &[5, 4]),
        BTreeSetEvent::NoCats { label: "d".into() },
    ]
    .into_dataframe_split()
}

// endregion

// region: 1. Vec<T> width = N (pinned expansion)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum VecWidthEvent {
    Score {
        label: String,
        #[dataframe(width = 3)]
        scores: Vec<f64>,
    },
    NoScore {
        label: String,
    },
}

fn vec_width_payload(label: &str, scores: Vec<f64>) -> VecWidthEvent {
    VecWidthEvent::Score {
        label: label.into(),
        scores,
    }
}

#[miniextendr]
pub fn vec_width_split_1v1r() -> List {
    vec![vec_width_payload("a", vec![1.0, 2.0, 3.0])].into_dataframe_split()
}

#[miniextendr]
pub fn vec_width_split_1vnr() -> List {
    vec![
        vec_width_payload("a", vec![1.0, 2.0, 3.0]),
        vec_width_payload("b", vec![4.0]),
        vec_width_payload("c", vec![]),
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_width_split_nv1r() -> List {
    vec![
        vec_width_payload("a", vec![1.0, 2.0, 3.0]),
        VecWidthEvent::NoScore { label: "b".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_width_split_nvnr() -> List {
    vec![
        vec_width_payload("a", vec![1.0, 2.0, 3.0]),
        VecWidthEvent::NoScore { label: "b".into() },
        vec_width_payload("c", vec![4.0]),
        VecWidthEvent::NoScore { label: "d".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_width_align_nvnr() -> BuiltDataFrame {
    vec![
        vec_width_payload("a", vec![1.0, 2.0, 3.0]),
        VecWidthEvent::NoScore { label: "b".into() },
        vec_width_payload("c", vec![4.0]),
    ]
    .into_dataframe()
    .unwrap()
}

// endregion

// region: 3. Vec<T> expand (auto-expand, runtime column count)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum VecExpandEvent {
    Vals {
        label: String,
        #[dataframe(expand)]
        vals: Vec<f64>,
    },
    NoVals {
        label: String,
    },
}

fn vec_expand_payload(label: &str, vals: Vec<f64>) -> VecExpandEvent {
    VecExpandEvent::Vals {
        label: label.into(),
        vals,
    }
}

#[miniextendr]
pub fn vec_expand_split_1v1r() -> List {
    vec![vec_expand_payload("a", vec![1.0, 2.0])].into_dataframe_split()
}

#[miniextendr]
pub fn vec_expand_split_1vnr() -> List {
    vec![
        vec_expand_payload("a", vec![1.0, 2.0]),
        vec_expand_payload("b", vec![3.0]),
        vec_expand_payload("c", vec![]),
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_expand_split_nv1r() -> List {
    vec![
        vec_expand_payload("a", vec![1.0, 2.0]),
        VecExpandEvent::NoVals { label: "b".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_expand_split_nvnr() -> List {
    vec![
        vec_expand_payload("a", vec![1.0, 2.0]),
        VecExpandEvent::NoVals { label: "b".into() },
        vec_expand_payload("c", vec![3.0]),
        VecExpandEvent::NoVals { label: "d".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_expand_align_nvnr() -> BuiltDataFrame {
    vec![
        vec_expand_payload("a", vec![1.0, 2.0]),
        VecExpandEvent::NoVals { label: "b".into() },
        vec_expand_payload("c", vec![3.0]),
    ]
    .into_dataframe()
    .unwrap()
}

// endregion

// region: 4. [T; N] (auto-expand fixed array)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum ArrayEvent {
    Coords { id: i32, coords: [f64; 2] },
    NoCoords { id: i32 },
}

fn array_payload(id: i32, coords: [f64; 2]) -> ArrayEvent {
    ArrayEvent::Coords { id, coords }
}

#[miniextendr]
pub fn array_split_1v1r() -> List {
    vec![array_payload(1, [10.0, 20.0])].into_dataframe_split()
}

#[miniextendr]
pub fn array_split_1vnr() -> List {
    vec![
        array_payload(1, [10.0, 20.0]),
        array_payload(2, [30.0, 40.0]),
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn array_split_nv1r() -> List {
    vec![
        array_payload(1, [10.0, 20.0]),
        ArrayEvent::NoCoords { id: 2 },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn array_split_nvnr() -> List {
    vec![
        array_payload(1, [10.0, 20.0]),
        ArrayEvent::NoCoords { id: 2 },
        array_payload(3, [30.0, 40.0]),
        ArrayEvent::NoCoords { id: 4 },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn array_align_nvnr() -> BuiltDataFrame {
    vec![
        array_payload(1, [10.0, 20.0]),
        ArrayEvent::NoCoords { id: 2 },
        array_payload(3, [30.0, 40.0]),
    ]
    .into_dataframe()
    .unwrap()
}

// endregion

// region: 5. Box<[T]> with expand (auto-expand)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BoxedSliceEvent {
    Buffer {
        name: String,
        #[dataframe(expand)]
        data: Box<[f64]>,
    },
    NoBuffer {
        name: String,
    },
}

fn boxed_slice_payload(name: &str, data: &[f64]) -> BoxedSliceEvent {
    BoxedSliceEvent::Buffer {
        name: name.into(),
        data: data.to_vec().into_boxed_slice(),
    }
}

#[miniextendr]
pub fn boxed_slice_split_1v1r() -> List {
    vec![boxed_slice_payload("a", &[1.0, 2.0, 3.0])].into_dataframe_split()
}

#[miniextendr]
pub fn boxed_slice_split_1vnr() -> List {
    vec![
        boxed_slice_payload("a", &[1.0, 2.0, 3.0]),
        boxed_slice_payload("b", &[4.0]),
        boxed_slice_payload("c", &[]),
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn boxed_slice_split_nv1r() -> List {
    vec![
        boxed_slice_payload("a", &[1.0, 2.0, 3.0]),
        BoxedSliceEvent::NoBuffer { name: "b".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn boxed_slice_split_nvnr() -> List {
    vec![
        boxed_slice_payload("a", &[1.0, 2.0, 3.0]),
        BoxedSliceEvent::NoBuffer { name: "b".into() },
        boxed_slice_payload("c", &[4.0]),
        BoxedSliceEvent::NoBuffer { name: "d".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn boxed_slice_align_nvnr() -> BuiltDataFrame {
    vec![
        boxed_slice_payload("a", &[1.0, 2.0, 3.0]),
        BoxedSliceEvent::NoBuffer { name: "b".into() },
        boxed_slice_payload("c", &[4.0]),
    ]
    .into_dataframe()
    .unwrap()
}

// endregion

// region: 5. Single-variant enum: bare-data.frame return from split
#[derive(Clone, Debug, DataFrameRow)]
pub enum SingletonRow {
    Only { id: i32, label: String },
}

#[miniextendr]
pub fn singleton_split_1v1r() -> List {
    vec![SingletonRow::Only {
        id: 1,
        label: "alpha".into(),
    }]
    .into_dataframe_split()
}

#[miniextendr]
pub fn singleton_split_1vnr() -> List {
    vec![
        SingletonRow::Only {
            id: 1,
            label: "alpha".into(),
        },
        SingletonRow::Only {
            id: 2,
            label: "beta".into(),
        },
        SingletonRow::Only {
            id: 3,
            label: "gamma".into(),
        },
    ]
    .into_dataframe_split()
}

// endregion

// region: 6. &str field (borrowed text → STRSXP with NA_character_)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BorrowedStrEvent<'a> {
    Named { id: i32, name: &'a str },
    Bare { id: i32 },
}

#[miniextendr]
pub fn borrowed_str_split_1v1r() -> List {
    let data: Vec<BorrowedStrEvent<'static>> = vec![BorrowedStrEvent::Named {
        id: 1,
        name: "alice",
    }];
    data.into_dataframe_split()
}

#[miniextendr]
pub fn borrowed_str_split_1vnr() -> List {
    let data: Vec<BorrowedStrEvent<'static>> = vec![
        BorrowedStrEvent::Named {
            id: 1,
            name: "alice",
        },
        BorrowedStrEvent::Named { id: 2, name: "bob" },
        BorrowedStrEvent::Named {
            id: 3,
            name: "carol",
        },
    ];
    data.into_dataframe_split()
}

#[miniextendr]
pub fn borrowed_str_split_nv1r() -> List {
    let data: Vec<BorrowedStrEvent<'static>> = vec![
        BorrowedStrEvent::Named {
            id: 1,
            name: "alice",
        },
        BorrowedStrEvent::Bare { id: 2 },
    ];
    data.into_dataframe_split()
}

#[miniextendr]
pub fn borrowed_str_align_nvnr() -> BuiltDataFrame {
    vec![
        BorrowedStrEvent::Named {
            id: 1,
            name: "alice",
        },
        BorrowedStrEvent::Bare { id: 2 },
        BorrowedStrEvent::Named {
            id: 3,
            name: "carol",
        },
        BorrowedStrEvent::Bare { id: 4 },
    ]
    .into_dataframe()
    .unwrap()
}

#[miniextendr]
pub fn borrowed_str_split_nvnr() -> List {
    let data: Vec<BorrowedStrEvent<'static>> = vec![
        BorrowedStrEvent::Named {
            id: 1,
            name: "alice",
        },
        BorrowedStrEvent::Bare { id: 2 },
        BorrowedStrEvent::Named {
            id: 3,
            name: "carol",
        },
        BorrowedStrEvent::Bare { id: 4 },
    ];
    data.into_dataframe_split()
}

// endregion

// region: 7. &[T] field opaque (borrowed slice → list-column with NULL)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BorrowedSliceEvent<'a> {
    Buffer { label: String, data: &'a [f64] },
    NoBuffer { label: String },
}

#[miniextendr]
pub fn borrowed_slice_split_1v1r() -> List {
    let data: Vec<BorrowedSliceEvent<'static>> = vec![BorrowedSliceEvent::Buffer {
        label: "a".into(),
        data: &[1.0, 2.0, 3.0],
    }];
    data.into_dataframe_split()
}

#[miniextendr]
pub fn borrowed_slice_split_1vnr() -> List {
    let data: Vec<BorrowedSliceEvent<'static>> = vec![
        BorrowedSliceEvent::Buffer {
            label: "a".into(),
            data: &[1.0, 2.0, 3.0],
        },
        BorrowedSliceEvent::Buffer {
            label: "b".into(),
            data: &[4.0],
        },
        BorrowedSliceEvent::Buffer {
            label: "c".into(),
            data: &[],
        },
    ];
    data.into_dataframe_split()
}

#[miniextendr]
pub fn borrowed_slice_split_nv1r() -> List {
    let data: Vec<BorrowedSliceEvent<'static>> = vec![
        BorrowedSliceEvent::Buffer {
            label: "a".into(),
            data: &[1.0, 2.0, 3.0],
        },
        BorrowedSliceEvent::NoBuffer { label: "b".into() },
    ];
    data.into_dataframe_split()
}

#[miniextendr]
pub fn borrowed_slice_align_nvnr() -> BuiltDataFrame {
    vec![
        BorrowedSliceEvent::Buffer {
            label: "a".into(),
            data: &[1.0, 2.0, 3.0],
        },
        BorrowedSliceEvent::NoBuffer { label: "b".into() },
        BorrowedSliceEvent::Buffer {
            label: "c".into(),
            data: &[4.0],
        },
        BorrowedSliceEvent::NoBuffer { label: "d".into() },
    ]
    .into_dataframe()
    .unwrap()
}

#[miniextendr]
pub fn borrowed_slice_split_nvnr() -> List {
    let data: Vec<BorrowedSliceEvent<'static>> = vec![
        BorrowedSliceEvent::Buffer {
            label: "a".into(),
            data: &[1.0, 2.0, 3.0],
        },
        BorrowedSliceEvent::NoBuffer { label: "b".into() },
        BorrowedSliceEvent::Buffer {
            label: "c".into(),
            data: &[4.0],
        },
        BorrowedSliceEvent::NoBuffer { label: "d".into() },
    ];
    data.into_dataframe_split()
}

// endregion

// region: 8. Map fields (HashMap<K,V> / BTreeMap<K,V>)
// HashMap and BTreeMap fields expand to two parallel list-columns:
//   `<field>_keys` and `<field>_values`.
// Absent-variant rows produce NULL in both. An empty map produces integer(0)/character(0).
// Key order: BTreeMap = sorted; HashMap = non-deterministic.
// Use setequal/sort checks in R tests for HashMap, exact checks for BTreeMap.

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum HashMapEvent {
    Tally {
        label: String,
        tally: HashMap<String, i32>,
    },
    Empty {
        label: String,
    },
}

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BTreeMapEvent {
    Tally {
        label: String,
        tally: BTreeMap<String, i32>,
    },
    Empty {
        label: String,
    },
}

// region: HashMap fixtures – all 4 cardinality cells

/// 1v1r: one variant (Tally), one row.
#[miniextendr]
pub fn hashmap_split_1v1r() -> List {
    vec![HashMapEvent::Tally {
        label: "a".into(),
        tally: HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
    }]
    .into_dataframe_split()
}

/// 1vNr: one variant (Tally), multiple rows.
#[miniextendr]
pub fn hashmap_split_1vnr() -> List {
    vec![
        HashMapEvent::Tally {
            label: "x".into(),
            tally: HashMap::from([("x".to_string(), 5i32)]),
        },
        HashMapEvent::Tally {
            label: "y".into(),
            tally: HashMap::new(), // empty map
        },
        HashMapEvent::Tally {
            label: "z".into(),
            tally: HashMap::from([("p".to_string(), 10i32), ("q".to_string(), 20i32)]),
        },
    ]
    .into_dataframe_split()
}

/// Nv1r: both variants, one row each.
#[miniextendr]
pub fn hashmap_split_nv1r() -> List {
    vec![
        HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
        },
        HashMapEvent::Empty { label: "b".into() },
    ]
    .into_dataframe_split()
}

/// 1v1r align: one variant (Tally), one row.
#[miniextendr]
pub fn hashmap_align_1v1r() -> BuiltDataFrame {
    vec![HashMapEvent::Tally {
        label: "a".into(),
        tally: HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
    }]
    .into_dataframe()
    .unwrap()
}

/// 1vNr align: one variant (Tally), multiple rows.
#[miniextendr]
pub fn hashmap_align_1vnr() -> BuiltDataFrame {
    vec![
        HashMapEvent::Tally {
            label: "x".into(),
            tally: HashMap::from([("x".to_string(), 5i32)]),
        },
        HashMapEvent::Tally {
            label: "y".into(),
            tally: HashMap::new(), // empty map
        },
        HashMapEvent::Tally {
            label: "z".into(),
            tally: HashMap::from([("p".to_string(), 10i32), ("q".to_string(), 20i32)]),
        },
    ]
    .into_dataframe()
    .unwrap()
}

/// Nv1r align: both variants, one row each.
#[miniextendr]
pub fn hashmap_align_nv1r() -> BuiltDataFrame {
    vec![
        HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
        },
        HashMapEvent::Empty { label: "b".into() },
    ]
    .into_dataframe()
    .unwrap()
}

/// NvNr align: both variants, multiple rows each.
#[miniextendr]
pub fn hashmap_align_nvnr() -> BuiltDataFrame {
    vec![
        HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
        },
        HashMapEvent::Empty { label: "b".into() },
        HashMapEvent::Tally {
            label: "c".into(),
            tally: HashMap::from([("x".to_string(), 5i32)]),
        },
        HashMapEvent::Empty { label: "d".into() },
    ]
    .into_dataframe()
    .unwrap()
}

/// NvNr split: both variants, multiple rows each.
#[miniextendr]
pub fn hashmap_split_nvnr() -> List {
    vec![
        HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
        },
        HashMapEvent::Empty { label: "b".into() },
        HashMapEvent::Tally {
            label: "c".into(),
            tally: HashMap::from([("x".to_string(), 5i32)]),
        },
        HashMapEvent::Empty { label: "d".into() },
    ]
    .into_dataframe_split()
}

// endregion: HashMap fixtures

// region: BTreeMap fixtures – all 4 cardinality cells

/// 1v1r: one variant (Tally), one row.
#[miniextendr]
pub fn btreemap_split_1v1r() -> List {
    vec![BTreeMapEvent::Tally {
        label: "a".into(),
        tally: BTreeMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
    }]
    .into_dataframe_split()
}

/// 1vNr: one variant (Tally), multiple rows.
#[miniextendr]
pub fn btreemap_split_1vnr() -> List {
    vec![
        BTreeMapEvent::Tally {
            label: "x".into(),
            tally: BTreeMap::from([("z".to_string(), 3i32), ("a".to_string(), 1i32)]),
        },
        BTreeMapEvent::Tally {
            label: "y".into(),
            tally: BTreeMap::new(), // empty map
        },
        BTreeMapEvent::Tally {
            label: "w".into(),
            tally: BTreeMap::from([("m".to_string(), 7i32)]),
        },
    ]
    .into_dataframe_split()
}

/// Nv1r: both variants, one row each.
#[miniextendr]
pub fn btreemap_split_nv1r() -> List {
    vec![
        BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
        },
        BTreeMapEvent::Empty { label: "b".into() },
    ]
    .into_dataframe_split()
}

/// 1v1r align: one variant (Tally), one row.
#[miniextendr]
pub fn btreemap_align_1v1r() -> BuiltDataFrame {
    vec![BTreeMapEvent::Tally {
        label: "a".into(),
        tally: BTreeMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
    }]
    .into_dataframe()
    .unwrap()
}

/// 1vNr align: one variant (Tally), multiple rows.
#[miniextendr]
pub fn btreemap_align_1vnr() -> BuiltDataFrame {
    vec![
        BTreeMapEvent::Tally {
            label: "x".into(),
            tally: BTreeMap::from([("z".to_string(), 3i32), ("a".to_string(), 1i32)]),
        },
        BTreeMapEvent::Tally {
            label: "y".into(),
            tally: BTreeMap::new(), // empty map
        },
        BTreeMapEvent::Tally {
            label: "w".into(),
            tally: BTreeMap::from([("m".to_string(), 7i32)]),
        },
    ]
    .into_dataframe()
    .unwrap()
}

/// Nv1r align: both variants, one row each.
#[miniextendr]
pub fn btreemap_align_nv1r() -> BuiltDataFrame {
    vec![
        BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
        },
        BTreeMapEvent::Empty { label: "b".into() },
    ]
    .into_dataframe()
    .unwrap()
}

/// NvNr align: both variants, multiple rows each.
#[miniextendr]
pub fn btreemap_align_nvnr() -> BuiltDataFrame {
    vec![
        BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::from([("z".to_string(), 3i32), ("a".to_string(), 1i32)]),
        },
        BTreeMapEvent::Empty { label: "b".into() },
        BTreeMapEvent::Tally {
            label: "c".into(),
            tally: BTreeMap::from([("m".to_string(), 7i32)]),
        },
        BTreeMapEvent::Empty { label: "d".into() },
    ]
    .into_dataframe()
    .unwrap()
}

/// NvNr split: both variants, multiple rows each.
#[miniextendr]
pub fn btreemap_split_nvnr() -> List {
    vec![
        BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::from([("z".to_string(), 3i32), ("a".to_string(), 1i32)]),
        },
        BTreeMapEvent::Empty { label: "b".into() },
        BTreeMapEvent::Tally {
            label: "c".into(),
            tally: BTreeMap::from([("m".to_string(), 7i32)]),
        },
        BTreeMapEvent::Empty { label: "d".into() },
    ]
    .into_dataframe_split()
}

// endregion: BTreeMap fixtures

// endregion: Map fields

#[cfg(test)]
mod tests {
    use super::*;
    use miniextendr_api::dataframe::ColumnarFrame;

    #[test]
    fn vec_width_align_pinned_columns() {
        let df = VecWidthEvent::to_dataframe(vec![
            VecWidthEvent::Score {
                label: "a".into(),
                scores: vec![1.0, 2.0, 3.0],
            },
            VecWidthEvent::NoScore { label: "b".into() },
            VecWidthEvent::Score {
                label: "c".into(),
                scores: vec![4.0],
            },
        ]);
        assert_eq!(df.scores_1, vec![Some(1.0), None, Some(4.0)]);
        assert_eq!(df.scores_2, vec![Some(2.0), None, None]);
        assert_eq!(df.scores_3, vec![Some(3.0), None, None]);
    }

    #[test]
    fn vec_expand_align_runtime_columns() {
        let df = VecExpandEvent::to_dataframe(vec![
            VecExpandEvent::Vals {
                label: "a".into(),
                vals: vec![1.0, 2.0],
            },
            VecExpandEvent::NoVals { label: "b".into() },
            VecExpandEvent::Vals {
                label: "c".into(),
                vals: vec![3.0],
            },
        ]);
        // expand stores Vec<Option<Vec<T>>> in companion struct
        assert_eq!(df.vals[0], Some(vec![1.0, 2.0]));
        assert_eq!(df.vals[1], None);
        assert_eq!(df.vals[2], Some(vec![3.0]));
    }

    #[test]
    fn array_align_expanded_columns() {
        let df = ArrayEvent::to_dataframe(vec![
            ArrayEvent::Coords {
                id: 1,
                coords: [10.0, 20.0],
            },
            ArrayEvent::NoCoords { id: 2 },
            ArrayEvent::Coords {
                id: 3,
                coords: [30.0, 40.0],
            },
        ]);
        assert_eq!(df.coords_1, vec![Some(10.0), None, Some(30.0)]);
        assert_eq!(df.coords_2, vec![Some(20.0), None, Some(40.0)]);
    }

    #[test]
    fn boxed_slice_expand_companion_shape() {
        let df = BoxedSliceEvent::to_dataframe(vec![
            BoxedSliceEvent::Buffer {
                name: "a".into(),
                data: vec![1.0, 2.0, 3.0].into_boxed_slice(),
            },
            BoxedSliceEvent::NoBuffer { name: "b".into() },
        ]);
        // expand on Box<[T]> in enum stores Vec<Option<Box<[T]>>> in companion
        assert_eq!(df.data.len(), 2);
        assert_eq!(df.data[0].as_deref(), Some(&[1.0, 2.0, 3.0][..]));
        assert!(df.data[1].is_none());
    }

    #[test]
    fn singleton_split_returns_bare_dataframe_shape() {
        // Single-variant split returns a bare List (data.frame on R side),
        // not a list-of-lists. Companion check: from_rows lays out as expected.
        let df = SingletonRowDataFrame::from_rows(vec![
            SingletonRow::Only {
                id: 1,
                label: "alpha".into(),
            },
            SingletonRow::Only {
                id: 2,
                label: "beta".into(),
            },
        ]);
        assert_eq!(df.id.len(), 2);
        assert_eq!(df.label.len(), 2);
    }

    // region: Map fields Rust unit tests

    #[test]
    fn test_hashmap_event_align_companion_shape() {
        // Verify companion struct has tally_keys and tally_values with correct option shape.
        let df = HashMapEvent::to_dataframe(vec![
            HashMapEvent::Tally {
                label: "a".into(),
                tally: HashMap::from([("x".to_string(), 1i32)]),
            },
            HashMapEvent::Empty { label: "b".into() },
        ]);
        assert_eq!(df.tally_keys.len(), 2);
        assert_eq!(df.tally_values.len(), 2);
        assert!(df.tally_keys[0].is_some());
        assert!(df.tally_values[0].is_some());
        assert!(df.tally_keys[1].is_none());
        assert!(df.tally_values[1].is_none());
        // Pairwise alignment: same length within a row.
        let k = df.tally_keys[0].as_ref().unwrap();
        let v = df.tally_values[0].as_ref().unwrap();
        assert_eq!(k.len(), v.len());
    }

    #[test]
    fn test_hashmap_empty_map_row() {
        // An empty HashMap produces Some(vec![]) (not None) in both columns.
        let df = HashMapEvent::to_dataframe(vec![HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::new(),
        }]);
        assert_eq!(df.tally_keys[0], Some(vec![]));
        assert_eq!(df.tally_values[0], Some(vec![]));
    }

    #[test]
    fn test_btreemap_keys_sorted() {
        // BTreeMap preserves sorted order: keys should be ["a", "z"], values [1, 3].
        let df = BTreeMapEvent::to_dataframe(vec![BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::from([("z".to_string(), 3i32), ("a".to_string(), 1i32)]),
        }]);
        assert_eq!(
            df.tally_keys[0].as_deref(),
            Some(vec!["a".to_string(), "z".to_string()].as_slice())
        );
        assert_eq!(
            df.tally_values[0].as_deref(),
            Some(vec![1i32, 3i32].as_slice())
        );
    }

    #[test]
    fn test_btreemap_empty_map_row() {
        let df = BTreeMapEvent::to_dataframe(vec![BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::new(),
        }]);
        assert_eq!(df.tally_keys[0], Some(vec![]));
        assert_eq!(df.tally_values[0], Some(vec![]));
    }

    #[test]
    fn test_btreemap_absent_variant_is_none() {
        let df = BTreeMapEvent::to_dataframe(vec![BTreeMapEvent::Empty { label: "b".into() }]);
        assert!(df.tally_keys[0].is_none());
        assert!(df.tally_values[0].is_none());
    }

    // endregion: Map fields Rust unit tests
}

// region: 9. Struct fields (DataFrameRow flatten / as_list opt-out)
// Struct-typed variant fields flatten into prefixed columns by default.
// The inner struct must #[derive(DataFrameRow)].
// Per-field #[dataframe(as_list)] keeps the struct as an opaque list-column
// (inner must then implement IntoR, e.g. via #[derive(IntoList)]).

/// A simple 2-column inner struct for testing struct-field flattening.
#[derive(Clone, Debug, DataFrameRow, IntoList)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Flatten path: `origin: Point` expands to `origin_x` + `origin_y` columns.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum StructFlattenEvent {
    Located { id: i32, origin: Point },
    Other { id: i32 },
}

/// as_list opt-out: `origin` becomes an opaque list-column.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum StructListEvent {
    Located {
        id: i32,
        #[dataframe(as_list)]
        origin: Point,
    },
    Other {
        id: i32,
    },
}

// region: Flatten fixtures — all 4 cardinality cells × 2 modes

/// 1v1r split: single Located row.
#[miniextendr]
pub fn struct_flatten_split_1v1r() -> List {
    vec![StructFlattenEvent::Located {
        id: 1,
        origin: Point { x: 1.0, y: 2.0 },
    }]
    .into_dataframe_split()
}

/// 1vNr split: multiple Located rows, all same variant.
#[miniextendr]
pub fn struct_flatten_split_1vnr() -> List {
    vec![
        StructFlattenEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructFlattenEvent::Located {
            id: 2,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructFlattenEvent::Located {
            id: 3,
            origin: Point { x: 5.0, y: 6.0 },
        },
    ]
    .into_dataframe_split()
}

/// Nv1r split: one Located and one Other row.
#[miniextendr]
pub fn struct_flatten_split_nv1r() -> List {
    vec![
        StructFlattenEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructFlattenEvent::Other { id: 2 },
    ]
    .into_dataframe_split()
}

/// NvNr split: multiple rows across both variants.
#[miniextendr]
pub fn struct_flatten_split_nvnr() -> List {
    vec![
        StructFlattenEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructFlattenEvent::Located {
            id: 2,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructFlattenEvent::Other { id: 3 },
        StructFlattenEvent::Other { id: 4 },
    ]
    .into_dataframe_split()
}

/// NvNr align: aligned data frame with NA-fill for absent variant.
#[miniextendr]
pub fn struct_flatten_align_nvnr() -> BuiltDataFrame {
    vec![
        StructFlattenEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructFlattenEvent::Located {
            id: 2,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructFlattenEvent::Other { id: 3 },
        StructFlattenEvent::Other { id: 4 },
    ]
    .into_dataframe()
    .unwrap()
}

// endregion: Flatten fixtures

// region: as_list fixtures — all 4 cardinality cells × 2 modes

/// 1v1r split (as_list): single Located row, origin as list-column.
#[miniextendr]
pub fn struct_list_split_1v1r() -> List {
    vec![StructListEvent::Located {
        id: 1,
        origin: Point { x: 1.0, y: 2.0 },
    }]
    .into_dataframe_split()
}

/// 1vNr split (as_list): multiple Located rows.
#[miniextendr]
pub fn struct_list_split_1vnr() -> List {
    vec![
        StructListEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructListEvent::Located {
            id: 2,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructListEvent::Located {
            id: 3,
            origin: Point { x: 5.0, y: 6.0 },
        },
    ]
    .into_dataframe_split()
}

/// Nv1r split (as_list): one Located and one Other row.
#[miniextendr]
pub fn struct_list_split_nv1r() -> List {
    vec![
        StructListEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructListEvent::Other { id: 2 },
    ]
    .into_dataframe_split()
}

/// NvNr split (as_list): multiple rows across both variants.
#[miniextendr]
pub fn struct_list_split_nvnr() -> List {
    vec![
        StructListEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructListEvent::Located {
            id: 2,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructListEvent::Other { id: 3 },
        StructListEvent::Other { id: 4 },
    ]
    .into_dataframe_split()
}

/// NvNr align (as_list): aligned data frame, origin as list-column.
#[miniextendr]
pub fn struct_list_align_nvnr() -> BuiltDataFrame {
    vec![
        StructListEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructListEvent::Located {
            id: 2,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructListEvent::Other { id: 3 },
        StructListEvent::Other { id: 4 },
    ]
    .into_dataframe()
    .unwrap()
}

// endregion: as_list fixtures

// region: Struct fields Rust unit tests

#[cfg(test)]
mod struct_field_tests {
    use super::*;

    #[test]
    fn test_struct_flatten_align_located_rows_have_values() {
        let df = StructFlattenEvent::to_dataframe(vec![
            StructFlattenEvent::Located {
                id: 1,
                origin: Point { x: 1.0, y: 2.0 },
            },
            StructFlattenEvent::Other { id: 2 },
        ]);
        // origin is present at row 0 (Located), absent at row 1 (Other)
        // The companion struct field is always _tag regardless of the tag= column name.
        assert_eq!(df._tag[0], "Located".to_string());
        assert_eq!(df._tag[1], "Other".to_string());
        assert_eq!(df.id[0], Some(1i32));
        assert_eq!(df.id[1], Some(2i32));
        // origin column holds the raw Point values
        assert!(df.origin[0].is_some());
        assert!(df.origin[1].is_none());
    }

    // This test calls into_dataframe_split which calls into_data_frame → R API.
    // It's exercised by the R-level tests in test-dataframe-enum-payload-matrix.R.
    #[test]
    #[ignore = "requires R runtime (calls into_data_frame)"]
    fn test_struct_flatten_split_located_has_correct_count() {
        let split = vec![
            StructFlattenEvent::Located {
                id: 1,
                origin: Point { x: 1.0, y: 2.0 },
            },
            StructFlattenEvent::Located {
                id: 2,
                origin: Point { x: 3.0, y: 4.0 },
            },
            StructFlattenEvent::Other { id: 3 },
        ]
        .into_dataframe_split();
        // split returns a list of per-variant data frames
        // We can't easily inspect column names from Rust, but we can check row counts
        let _ = split; // compiled and not panicked ⇒ success
    }

    #[test]
    fn test_struct_list_align_origin_is_some() {
        let df = StructListEvent::to_dataframe(vec![
            StructListEvent::Located {
                id: 1,
                origin: Point { x: 1.0, y: 2.0 },
            },
            StructListEvent::Other { id: 2 },
        ]);
        // origin is the list-column holding Option<Point>
        assert!(df.origin[0].is_some());
        assert!(df.origin[1].is_none());
    }
}

// endregion: Struct fields Rust unit tests

// endregion: 9. Struct fields

// region: 10. Nested enum fields (as_factor / flatten / as_list)
// Inner enum `Direction` is a unit-only enum that derives `DataFrameRow`.
// The derive auto-emits `IntoR`, `IntoR for Vec<Option<Self>>`, and `IntoList`
// so it can be used with `as_factor` (factor column) or `as_list` (list column)
// in an outer enum's variant fields.
//
// Inner enum `Status` has payload variants (not unit-only) and thus implements
// `DataFrameRow` for flattening only — its variants get prefixed columns.
//
// Three outer enums exercise the three field-treatment modes:
//   - `NestedFlattenEvent`: `dir: Direction` flattens via DataFrameRow (produces
//     `dir_variant` column from the `tag = "variant"` attribute on Direction)
//   - `NestedFactorEvent`: `#[dataframe(as_factor)] dir: Direction` → factor column
//   - `NestedListEvent`: `#[dataframe(as_list)] dir: Direction` → list column
//
// Note: the inner-payload-field-named-`variant` pattern (i.e., a `Status` payload
// field named `"variant"`) is now a **compile error** enforced via
// `assert_no_payload_field_collision` at the outer `#[derive(DataFrameRow)]` site.
// See issue #486 and PR #542.

/// Unit-only inner enum — derives `DataFrameRow`, which auto-emits `IntoR` and
/// `IntoR for Vec<Option<Self>>` as factor SEXPs, and `IntoList`.
#[derive(Clone, Copy, Debug, DataFrameRow)]
#[dataframe(tag = "variant")]
pub enum Direction {
    North,
    South,
    East,
    West,
}

/// Payload inner enum — derives `DataFrameRow` for flattening only.
/// `Status` has a payload variant so it is NOT unit-only; no factor impls are emitted.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "variant")]
pub enum Status {
    Ok,
    Err { code: i32 },
}

/// Outer enum: `status: Status` field flattens via DataFrameRow.
/// `Status` is a payload-bearing inner enum (has `Ok` and `Err { code }`).
/// After prefixing: `status_variant` (discriminant), `status_code` (NA for Ok rows).
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum NestedFlattenEvent {
    Tracked { id: i32, status: Status },
    Other { id: i32 },
}

/// Outer enum: `#[dataframe(as_factor)] dir: Direction` → single factor column.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum NestedFactorEvent {
    Move {
        id: i32,
        #[dataframe(as_factor)]
        dir: Direction,
    },
    Stop {
        id: i32,
    },
}

/// Outer enum: `#[dataframe(as_list)] dir: Direction` → opaque list column.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum NestedListEvent {
    Move {
        id: i32,
        #[dataframe(as_list)]
        dir: Direction,
    },
    Stop {
        id: i32,
    },
}

// region: Flatten fixtures — all 4 cardinality cells × 2 modes (split + align)
//
// Uses the payload-bearing `Status` inner enum so the flatten path exercises:
//   - `status_variant` discriminant column
//   - `status_code` payload column (NA for `Status::Ok` rows, Some(i32) for `Status::Err`)

/// 1v1r split (flatten): single Tracked/Ok row.
#[miniextendr]
pub fn nested_flatten_split_1v1r() -> List {
    vec![NestedFlattenEvent::Tracked {
        id: 1,
        status: Status::Ok,
    }]
    .into_dataframe_split()
}

/// 1vNr split (flatten): multiple Tracked rows (mix of Ok and Err).
#[miniextendr]
pub fn nested_flatten_split_1vnr() -> List {
    vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Tracked {
            id: 2,
            status: Status::Err { code: 404 },
        },
        NestedFlattenEvent::Tracked {
            id: 3,
            status: Status::Err { code: 500 },
        },
    ]
    .into_dataframe_split()
}

/// Nv1r split (flatten): one Tracked and one Other row.
#[miniextendr]
pub fn nested_flatten_split_nv1r() -> List {
    vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Other { id: 2 },
    ]
    .into_dataframe_split()
}

/// NvNr split (flatten): multiple rows across both variants.
#[miniextendr]
pub fn nested_flatten_split_nvnr() -> List {
    vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Tracked {
            id: 2,
            status: Status::Err { code: 404 },
        },
        NestedFlattenEvent::Other { id: 3 },
        NestedFlattenEvent::Other { id: 4 },
    ]
    .into_dataframe_split()
}

/// 1v1r align (flatten): single Tracked row, aligned data frame.
#[miniextendr]
pub fn nested_flatten_align_1v1r() -> BuiltDataFrame {
    vec![NestedFlattenEvent::Tracked {
        id: 1,
        status: Status::Ok,
    }]
    .into_dataframe()
    .unwrap()
}

/// 1vNr align (flatten): multiple Tracked rows, aligned data frame.
#[miniextendr]
pub fn nested_flatten_align_1vnr() -> BuiltDataFrame {
    vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Tracked {
            id: 2,
            status: Status::Err { code: 404 },
        },
        NestedFlattenEvent::Tracked {
            id: 3,
            status: Status::Err { code: 500 },
        },
    ]
    .into_dataframe()
    .unwrap()
}

/// Nv1r align (flatten): one Tracked, one Other — NA-fill for Other rows.
#[miniextendr]
pub fn nested_flatten_align_nv1r() -> BuiltDataFrame {
    vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Other { id: 2 },
    ]
    .into_dataframe()
    .unwrap()
}

/// NvNr align (flatten): multiple rows across both variants — NA-fill for Other rows.
#[miniextendr]
pub fn nested_flatten_align_nvnr() -> BuiltDataFrame {
    vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Tracked {
            id: 2,
            status: Status::Err { code: 404 },
        },
        NestedFlattenEvent::Other { id: 3 },
        NestedFlattenEvent::Other { id: 4 },
    ]
    .into_dataframe()
    .unwrap()
}

// endregion: Flatten fixtures

// region: as_factor fixtures — all 4 cardinality cells × 2 modes

/// 1v1r split (as_factor): single Move row, dir as factor column.
#[miniextendr]
pub fn nested_factor_split_1v1r() -> List {
    vec![NestedFactorEvent::Move {
        id: 1,
        dir: Direction::North,
    }]
    .into_dataframe_split()
}

/// 1vNr split (as_factor): multiple Move rows.
#[miniextendr]
pub fn nested_factor_split_1vnr() -> List {
    vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedFactorEvent::Move {
            id: 2,
            dir: Direction::South,
        },
        NestedFactorEvent::Move {
            id: 3,
            dir: Direction::East,
        },
    ]
    .into_dataframe_split()
}

/// Nv1r split (as_factor): one Move and one Stop row.
#[miniextendr]
pub fn nested_factor_split_nv1r() -> List {
    vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::West,
        },
        NestedFactorEvent::Stop { id: 2 },
    ]
    .into_dataframe_split()
}

/// NvNr split (as_factor): multiple rows across both variants.
#[miniextendr]
pub fn nested_factor_split_nvnr() -> List {
    vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedFactorEvent::Move {
            id: 2,
            dir: Direction::East,
        },
        NestedFactorEvent::Stop { id: 3 },
        NestedFactorEvent::Stop { id: 4 },
    ]
    .into_dataframe_split()
}

/// 1v1r align (as_factor): single Move row.
#[miniextendr]
pub fn nested_factor_align_1v1r() -> BuiltDataFrame {
    vec![NestedFactorEvent::Move {
        id: 1,
        dir: Direction::North,
    }]
    .into_dataframe()
    .unwrap()
}

/// 1vNr align (as_factor): multiple Move rows.
#[miniextendr]
pub fn nested_factor_align_1vnr() -> BuiltDataFrame {
    vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedFactorEvent::Move {
            id: 2,
            dir: Direction::South,
        },
        NestedFactorEvent::Move {
            id: 3,
            dir: Direction::East,
        },
    ]
    .into_dataframe()
    .unwrap()
}

/// Nv1r align (as_factor): one Move and one Stop — NA for Stop's dir.
#[miniextendr]
pub fn nested_factor_align_nv1r() -> BuiltDataFrame {
    vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::West,
        },
        NestedFactorEvent::Stop { id: 2 },
    ]
    .into_dataframe()
    .unwrap()
}

/// NvNr align (as_factor): aligned data frame with NA-fill for dir in Stop rows.
#[miniextendr]
pub fn nested_factor_align_nvnr() -> BuiltDataFrame {
    vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedFactorEvent::Move {
            id: 2,
            dir: Direction::East,
        },
        NestedFactorEvent::Stop { id: 3 },
        NestedFactorEvent::Stop { id: 4 },
    ]
    .into_dataframe()
    .unwrap()
}

// endregion: as_factor fixtures

// region: as_list fixtures — all 4 cardinality cells × 2 modes

/// 1v1r split (as_list): single Move row, dir as list column.
#[miniextendr]
pub fn nested_list_split_1v1r() -> List {
    vec![NestedListEvent::Move {
        id: 1,
        dir: Direction::North,
    }]
    .into_dataframe_split()
}

/// 1vNr split (as_list): multiple Move rows.
#[miniextendr]
pub fn nested_list_split_1vnr() -> List {
    vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedListEvent::Move {
            id: 2,
            dir: Direction::South,
        },
        NestedListEvent::Move {
            id: 3,
            dir: Direction::East,
        },
    ]
    .into_dataframe_split()
}

/// Nv1r split (as_list): one Move and one Stop row.
#[miniextendr]
pub fn nested_list_split_nv1r() -> List {
    vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::West,
        },
        NestedListEvent::Stop { id: 2 },
    ]
    .into_dataframe_split()
}

/// NvNr split (as_list): multiple rows across both variants.
#[miniextendr]
pub fn nested_list_split_nvnr() -> List {
    vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedListEvent::Move {
            id: 2,
            dir: Direction::East,
        },
        NestedListEvent::Stop { id: 3 },
        NestedListEvent::Stop { id: 4 },
    ]
    .into_dataframe_split()
}

/// 1v1r align (as_list): single Move row.
#[miniextendr]
pub fn nested_list_align_1v1r() -> BuiltDataFrame {
    vec![NestedListEvent::Move {
        id: 1,
        dir: Direction::North,
    }]
    .into_dataframe()
    .unwrap()
}

/// 1vNr align (as_list): multiple Move rows.
#[miniextendr]
pub fn nested_list_align_1vnr() -> BuiltDataFrame {
    vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedListEvent::Move {
            id: 2,
            dir: Direction::South,
        },
        NestedListEvent::Move {
            id: 3,
            dir: Direction::East,
        },
    ]
    .into_dataframe()
    .unwrap()
}

/// Nv1r align (as_list): one Move and one Stop — NULL for Stop's dir.
#[miniextendr]
pub fn nested_list_align_nv1r() -> BuiltDataFrame {
    vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::West,
        },
        NestedListEvent::Stop { id: 2 },
    ]
    .into_dataframe()
    .unwrap()
}

/// NvNr align (as_list): aligned data frame with NULL-fill for dir in Stop rows.
#[miniextendr]
pub fn nested_list_align_nvnr() -> BuiltDataFrame {
    vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedListEvent::Move {
            id: 2,
            dir: Direction::East,
        },
        NestedListEvent::Stop { id: 3 },
        NestedListEvent::Stop { id: 4 },
    ]
    .into_dataframe()
    .unwrap()
}

// endregion: as_list fixtures

// region: Nested enum Rust unit tests
#[cfg(test)]
mod nested_enum_field_tests {
    use super::*;

    #[test]
    fn test_direction_is_unit_only() {
        // DataFrameRow derive emits IntoR for Direction; Direction must compile.
        // Verify all variants can be constructed (no payload).
        let _ = Direction::North;
        let _ = Direction::South;
        let _ = Direction::East;
        let _ = Direction::West;
    }

    /// Companion struct for `NestedFlattenEvent` has `status: Vec<Option<Status>>`.
    /// `Other` rows produce `None`; `Tracked` rows produce `Some(Status::*)`.
    #[test]
    fn test_nested_flatten_companion_struct_shape() {
        let df = NestedFlattenEvent::to_dataframe(vec![
            NestedFlattenEvent::Tracked {
                id: 1,
                status: Status::Ok,
            },
            NestedFlattenEvent::Other { id: 2 },
            NestedFlattenEvent::Tracked {
                id: 3,
                status: Status::Err { code: 404 },
            },
        ]);
        assert_eq!(df.id[0], Some(1i32));
        assert_eq!(df.id[1], Some(2i32));
        assert_eq!(df.id[2], Some(3i32));
        // status field holds Option<Status>
        assert!(df.status[0].is_some()); // Tracked → Some(Status::Ok)
        assert!(df.status[1].is_none()); // Other → None (absent variant)
        assert!(df.status[2].is_some()); // Tracked → Some(Status::Err)
        // Verify Status discriminator via DataFrameRow
        let inner_df = Status::to_dataframe(vec![Status::Ok, Status::Err { code: 500 }]);
        assert!(inner_df.code[0].is_none()); // Ok has no code
        assert_eq!(inner_df.code[1], Some(500i32)); // Err has code
    }

    /// Verify payload-flatten partition: inner Status columns are correctly populated.
    /// `Status::Ok` has no `code` field; `Status::Err` does.
    #[test]
    fn test_nested_flatten_status_payload_columns() {
        // Verify inner Status companion struct has the expected column layout.
        let inner_df =
            Status::to_dataframe(vec![Status::Ok, Status::Err { code: 404 }, Status::Ok]);
        // `code` column: None for Ok rows, Some(i32) for Err rows.
        assert!(inner_df.code[0].is_none()); // Ok has no code
        assert_eq!(inner_df.code[1], Some(404i32));
        assert!(inner_df.code[2].is_none());
    }

    #[test]
    fn test_nested_factor_align_col_types() {
        let df = NestedFactorEvent::to_dataframe(vec![
            NestedFactorEvent::Move {
                id: 1,
                dir: Direction::North,
            },
            NestedFactorEvent::Stop { id: 2 },
        ]);
        // The companion struct has id: Vec<Option<i32>> and dir: Vec<Option<Direction>>
        assert_eq!(df.id[0], Some(1i32));
        assert_eq!(df.id[1], Some(2i32));
        assert!(df.dir[0].is_some());
        assert!(df.dir[1].is_none()); // Stop variant has no dir field
    }

    /// Factor levels must match Direction's variant order: North, South, East, West.
    #[test]
    fn test_direction_factor_levels_match_variant_order() {
        use miniextendr_api::UnitEnumFactor;
        // Direction is unit-only → DataFrameRow derive auto-emits UnitEnumFactor.
        // FACTOR_LEVELS must be in declaration order.
        let levels = Direction::FACTOR_LEVELS;
        assert_eq!(levels, &["North", "South", "East", "West"]);
    }

    #[test]
    fn test_nested_list_align_col_types() {
        let df = NestedListEvent::to_dataframe(vec![
            NestedListEvent::Move {
                id: 1,
                dir: Direction::North,
            },
            NestedListEvent::Stop { id: 2 },
        ]);
        assert_eq!(df.id[0], Some(1i32));
        assert!(df.dir[0].is_some());
        assert!(df.dir[1].is_none());
    }

    #[test]
    fn test_nested_flatten_align_col_types() {
        let df = NestedFlattenEvent::to_dataframe(vec![
            NestedFlattenEvent::Tracked {
                id: 1,
                status: Status::Ok,
            },
            NestedFlattenEvent::Other { id: 2 },
        ]);
        assert_eq!(df.id[0], Some(1i32));
        // status flattened → status column holds Option<Status>
        assert!(df.status[0].is_some());
        assert!(df.status[1].is_none()); // Other row: status is None
    }
}
// endregion: Nested enum Rust unit tests

// endregion: 10. Nested enum fields
