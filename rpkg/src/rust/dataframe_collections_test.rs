//! Test DataFrameRow with various collection types

#![allow(dead_code)]

use miniextendr_api::{DataFrameRow, IntoList};
use std::collections::{BTreeSet, HashMap, HashSet};

// Test with Vec fields (no expansion — stays opaque)
#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct WithVec {
    pub ids: Vec<i32>,
    pub names: Vec<String>,
}

// Test with boxed slices (scalar — no expansion)
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithBoxedSlice {
    pub data: Box<[f64]>,
}

impl ::miniextendr_api::list::IntoList for WithBoxedSlice {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        // Protect each value as it is built so it survives `from_raw_pairs`'s internal
        // allocations (VECSXP + names STRSXP via `charsxp`) — mirrors `#[derive(IntoList)]`.
        // SAFETY: IntoList runs on the R main thread.
        unsafe {
            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
            ::miniextendr_api::List::from_raw_pairs(vec![(
                "data",
                __scope.protect_raw(self.data.into_vec().into_sexp()),
            )])
        }
    }
}

// Test with arrays — now auto-expands to suffixed columns
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithArray {
    pub coords: [f64; 3],
}

// Test with arrays + as_list to suppress expansion
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithArrayAsList {
    #[dataframe(as_list)]
    pub coords: [f64; 3],
}

impl ::miniextendr_api::list::IntoList for WithArrayAsList {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        // SAFETY: IntoList runs on the R main thread. Protect-as-built (see WithBoxedSlice).
        unsafe {
            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
            ::miniextendr_api::List::from_raw_pairs(vec![(
                "coords",
                __scope.protect_raw(self.coords.to_vec().into_sexp()),
            )])
        }
    }
}

// Test with HashSet
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithHashSet {
    pub tags: HashSet<String>,
}

impl ::miniextendr_api::list::IntoList for WithHashSet {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        let tags_vec: Vec<String> = self.tags.into_iter().collect();
        // SAFETY: IntoList runs on the R main thread. Protect-as-built (see WithBoxedSlice).
        unsafe {
            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
            ::miniextendr_api::List::from_raw_pairs(vec![(
                "tags",
                __scope.protect_raw(tags_vec.into_sexp()),
            )])
        }
    }
}

// Test with BTreeSet
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithBTreeSet {
    pub categories: BTreeSet<i32>,
}

impl ::miniextendr_api::list::IntoList for WithBTreeSet {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        let cats_vec: Vec<i32> = self.categories.into_iter().collect();
        // SAFETY: IntoList runs on the R main thread. Protect-as-built (see WithBoxedSlice).
        unsafe {
            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
            ::miniextendr_api::List::from_raw_pairs(vec![(
                "categories",
                __scope.protect_raw(cats_vec.into_sexp()),
            )])
        }
    }
}

// Test with mixed collection types — array auto-expands, vec stays opaque
#[derive(Clone, Debug, DataFrameRow)]
pub struct MixedCollections {
    pub vec_field: Vec<i32>,
    pub array_field: [f64; 2],
    pub boxed_field: Box<[String]>,
}

impl ::miniextendr_api::list::IntoList for MixedCollections {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        // SAFETY: IntoList runs on the R main thread. Protect-as-built (see WithBoxedSlice).
        unsafe {
            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
            ::miniextendr_api::List::from_raw_pairs(vec![
                ("vec_field", __scope.protect_raw(self.vec_field.into_sexp())),
                (
                    "array_field",
                    __scope.protect_raw(self.array_field.to_vec().into_sexp()),
                ),
                (
                    "boxed_field",
                    __scope.protect_raw(self.boxed_field.into_vec().into_sexp()),
                ),
            ])
        }
    }
}

// Test skip attribute
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithSkip {
    pub name: String,
    #[dataframe(skip)]
    pub internal_id: u64,
    pub value: f64,
}

impl ::miniextendr_api::list::IntoList for WithSkip {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        // SAFETY: IntoList runs on the R main thread. Protect-as-built (see WithBoxedSlice).
        unsafe {
            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
            ::miniextendr_api::List::from_raw_pairs(vec![
                ("name", __scope.protect_raw(self.name.into_sexp())),
                ("value", __scope.protect_raw(self.value.into_sexp())),
            ])
        }
    }
}

// Test rename attribute
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithRename {
    #[dataframe(rename = "label")]
    pub name: String,
    pub value: f64,
}

impl ::miniextendr_api::list::IntoList for WithRename {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        // SAFETY: IntoList runs on the R main thread. Protect-as-built (see WithBoxedSlice).
        unsafe {
            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
            ::miniextendr_api::List::from_raw_pairs(vec![
                ("label", __scope.protect_raw(self.name.into_sexp())),
                ("value", __scope.protect_raw(self.value.into_sexp())),
            ])
        }
    }
}

// Test Vec with width (pinned expansion)
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithPinnedVec {
    pub name: String,
    #[dataframe(width = 3)]
    pub scores: Vec<f64>,
}

// Test rename on expanded array
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithRenamedArray {
    #[dataframe(rename = "pos")]
    pub coords: [f64; 2],
}

// Test enum with expanded array field
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(tag = "_type")]
pub enum EventWithCoords {
    Click { id: i32, coords: [f64; 2] },
    Hover { id: i32 },
}

// Test enum with pinned Vec expansion
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(tag = "_type")]
pub enum EventWithScores {
    Result {
        label: String,
        #[dataframe(width = 2)]
        scores: Vec<f64>,
    },
    Empty {
        label: String,
    },
}

// Test Vec with auto-expand (runtime column count)
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithAutoExpand {
    pub name: String,
    #[dataframe(expand)]
    pub values: Vec<f64>,
}

// Test Vec with unnest alias
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithUnnest {
    pub label: String,
    #[dataframe(unnest)]
    pub items: Vec<i32>,
}

// Test enum with auto-expand
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(tag = "_type")]
pub enum EventWithAutoExpand {
    Scores {
        label: String,
        #[dataframe(expand)]
        vals: Vec<f64>,
    },
    Empty {
        label: String,
    },
}

// Test Box<[T]> with auto-expand (runtime column count)
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithBoxedSliceExpand {
    pub name: String,
    #[dataframe(expand)]
    pub scores: Box<[f64]>,
}

// Test Box<[T]> with pinned width
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithBoxedSlicePinned {
    pub name: String,
    #[dataframe(width = 3)]
    pub coords: Box<[f64]>,
}

// Test &[T] with auto-expand (runtime column count)
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithSliceExpand<'a> {
    pub name: &'a str,
    #[dataframe(expand)]
    pub values: &'a [f64],
}

// Test &[T] with pinned width
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithSlicePinned<'a> {
    pub label: &'a str,
    #[dataframe(width = 2)]
    pub coords: &'a [f64],
}

// Test that DataFrameRow-annotated enums with borrowed fields work when using
// a struct with a lifetime (issue #460). For enums, use String/owned types
// instead of &'a str (Vec<Option<&str>> lacks IntoR in enum context).
// The struct path supports borrowed fields directly (see WithSliceExpand below).
// This enum uses only owned types to verify the enum expand path compiles.
#[derive(Clone, Debug, DataFrameRow)]
pub enum WithLifetimeEnumOwned {
    Row { label: String, value: f64 },
}

// Test enum with skip
#[derive(Clone, Debug, DataFrameRow)]
pub enum EventWithSkip {
    A {
        value: f64,
        #[dataframe(skip)]
        internal: i32,
    },
    B {
        value: f64,
    },
}

// Test parallel fill with expansion (struct)
#[derive(Clone, Debug, DataFrameRow)]
pub struct ParallelExpanded {
    pub id: i32,
    pub coords: [f64; 3],
    #[dataframe(width = 2)]
    pub tags: Vec<String>,
    #[dataframe(expand)]
    pub values: Vec<f64>,
}

// Test parallel fill with expansion (enum)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum ParallelExpandedEvent {
    Measurement { sensor: String, readings: [f64; 2] },
    Status { sensor: String, code: i32 },
}

// Test: `Option<HashMap<K, V>>` with `#[dataframe(as_list)]` — escape hatch for #484.
// The field becomes an opaque list-column (`Vec<Option<HashMap<String, i32>>>` in the
// companion struct).  Without `as_list` the macro now emits a compile error.
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithOptMapAsList {
    pub id: i32,
    #[dataframe(as_list)]
    pub counts: Option<HashMap<String, i32>>,
}

impl IntoList for WithOptMapAsList {
    fn into_list(self) -> miniextendr_api::List {
        use miniextendr_api::{IntoR, SEXP};
        // SAFETY: IntoList runs on the R main thread. Protect both values as built — the
        // pre-built `counts_sexp` must survive `self.id.into_sexp()` and `from_raw_pairs`'s
        // internal allocations. Mirrors `#[derive(IntoList)]`.
        unsafe {
            let __scope = miniextendr_api::gc_protect::ProtectScope::new();
            let counts_sexp = __scope.protect_raw(match self.counts {
                Some(m) => m.into_list().into_sexp(),
                None => SEXP::nil(),
            });
            miniextendr_api::List::from_raw_pairs(vec![
                ("id", __scope.protect_raw(self.id.into_sexp())),
                ("counts", counts_sexp),
            ])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use miniextendr_api::dataframe::ColumnarFrame;

    #[test]
    fn test_vec_dataframe() {
        let rows = vec![
            WithVec {
                ids: vec![1, 2, 3],
                names: vec!["a".into(), "b".into()],
            },
            WithVec {
                ids: vec![4, 5],
                names: vec!["c".into()],
            },
        ];

        let df = WithVec::to_dataframe(rows.clone());
        assert_eq!(df.ids.len(), 2);
        assert_eq!(df.names.len(), 2);

        // Test round-trip (companion → rows via `ColumnarFrame::into_rows`)
        let back: Vec<WithVec> = df.into_rows();
        assert_eq!(back.len(), 2);
    }

    #[test]
    fn test_boxed_slice_dataframe() {
        let rows = vec![
            WithBoxedSlice {
                data: vec![1.0, 2.0, 3.0].into_boxed_slice(),
            },
            WithBoxedSlice {
                data: vec![4.0, 5.0].into_boxed_slice(),
            },
        ];

        let df = WithBoxedSlice::to_dataframe(rows);
        assert_eq!(df.data.len(), 2);
    }

    #[test]
    fn test_array_expansion() {
        // [f64; 3] now auto-expands to coords_1, coords_2, coords_3
        let rows = vec![
            WithArray {
                coords: [1.0, 2.0, 3.0],
            },
            WithArray {
                coords: [4.0, 5.0, 6.0],
            },
        ];

        let df = WithArray::to_dataframe(rows);
        assert_eq!(df.coords_1.len(), 2);
        assert_eq!(df.coords_2.len(), 2);
        assert_eq!(df.coords_3.len(), 2);
        assert_eq!(df.coords_1[0], 1.0);
        assert_eq!(df.coords_2[0], 2.0);
        assert_eq!(df.coords_3[0], 3.0);
        assert_eq!(df.coords_1[1], 4.0);
        assert_eq!(df.coords_2[1], 5.0);
        assert_eq!(df.coords_3[1], 6.0);
    }

    #[test]
    fn test_array_as_list() {
        // as_list suppresses expansion
        let rows = vec![
            WithArrayAsList {
                coords: [1.0, 2.0, 3.0],
            },
            WithArrayAsList {
                coords: [4.0, 5.0, 6.0],
            },
        ];

        let df = WithArrayAsList::to_dataframe(rows);
        // Single column, not expanded
        assert_eq!(df.coords.len(), 2);
        assert_eq!(df.coords[0], [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_hashset_dataframe() {
        let mut tags1 = HashSet::new();
        tags1.insert("rust".to_string());
        tags1.insert("r".to_string());

        let mut tags2 = HashSet::new();
        tags2.insert("python".to_string());

        let rows = vec![WithHashSet { tags: tags1 }, WithHashSet { tags: tags2 }];

        let df = WithHashSet::to_dataframe(rows);
        assert_eq!(df.tags.len(), 2);
    }

    #[test]
    fn test_btreeset_dataframe() {
        let mut cats1 = BTreeSet::new();
        cats1.insert(1);
        cats1.insert(2);

        let mut cats2 = BTreeSet::new();
        cats2.insert(3);

        let rows = vec![
            WithBTreeSet { categories: cats1 },
            WithBTreeSet { categories: cats2 },
        ];

        let df = WithBTreeSet::to_dataframe(rows);
        assert_eq!(df.categories.len(), 2);
    }

    #[test]
    fn test_mixed_collections() {
        // array_field: [f64; 2] now expands to array_field_1, array_field_2
        let rows = vec![
            MixedCollections {
                vec_field: vec![1, 2],
                array_field: [1.0, 2.0],
                boxed_field: vec!["a".into(), "b".into()].into_boxed_slice(),
            },
            MixedCollections {
                vec_field: vec![3, 4, 5],
                array_field: [3.0, 4.0],
                boxed_field: vec!["c".into()].into_boxed_slice(),
            },
        ];

        let df = MixedCollections::to_dataframe(rows);
        assert_eq!(df.vec_field.len(), 2);
        assert_eq!(df.array_field_1.len(), 2);
        assert_eq!(df.array_field_2.len(), 2);
        assert_eq!(df.boxed_field.len(), 2);
        assert_eq!(df.array_field_1[0], 1.0);
        assert_eq!(df.array_field_2[0], 2.0);
        // No from_dataframe for structs with expansion
    }

    #[test]
    fn test_skip_field() {
        let rows = vec![
            WithSkip {
                name: "a".into(),
                internal_id: 42,
                value: 1.0,
            },
            WithSkip {
                name: "b".into(),
                internal_id: 99,
                value: 2.0,
            },
        ];

        let df = WithSkip::to_dataframe(rows);
        // Only name and value columns (internal_id skipped)
        assert_eq!(df.name.len(), 2);
        assert_eq!(df.value.len(), 2);
        assert_eq!(df.name[0], "a");
        assert_eq!(df.value[1], 2.0);
    }

    #[test]
    fn test_rename_field() {
        let rows = vec![WithRename {
            name: "hello".into(),
            value: 1.0,
        }];

        let df = WithRename::to_dataframe(rows);
        // Column is named "label" in the companion struct
        assert_eq!(df.label.len(), 1);
        assert_eq!(df.label[0], "hello");
    }

    #[test]
    fn test_pinned_vec_expansion() {
        let rows = vec![
            WithPinnedVec {
                name: "a".into(),
                scores: vec![1.0, 2.0, 3.0],
            },
            WithPinnedVec {
                name: "b".into(),
                scores: vec![4.0], // shorter: trailing None
            },
            WithPinnedVec {
                name: "c".into(),
                scores: vec![], // empty: all None
            },
        ];

        let df = WithPinnedVec::to_dataframe(rows);
        assert_eq!(df.name.len(), 3);
        assert_eq!(df.scores_1.len(), 3);
        assert_eq!(df.scores_2.len(), 3);
        assert_eq!(df.scores_3.len(), 3);

        // First row: all present
        assert_eq!(df.scores_1[0], Some(1.0));
        assert_eq!(df.scores_2[0], Some(2.0));
        assert_eq!(df.scores_3[0], Some(3.0));

        // Second row: only first present
        assert_eq!(df.scores_1[1], Some(4.0));
        assert_eq!(df.scores_2[1], None);
        assert_eq!(df.scores_3[1], None);

        // Third row: all None
        assert_eq!(df.scores_1[2], None);
        assert_eq!(df.scores_2[2], None);
        assert_eq!(df.scores_3[2], None);
    }

    #[test]
    fn test_renamed_array_expansion() {
        let rows = vec![
            WithRenamedArray { coords: [1.0, 2.0] },
            WithRenamedArray { coords: [3.0, 4.0] },
        ];

        let df = WithRenamedArray::to_dataframe(rows);
        // rename = "pos" → columns pos_1, pos_2
        assert_eq!(df.pos_1.len(), 2);
        assert_eq!(df.pos_2.len(), 2);
        assert_eq!(df.pos_1[0], 1.0);
        assert_eq!(df.pos_2[1], 4.0);
    }

    #[test]
    fn test_enum_array_expansion() {
        let rows = vec![
            EventWithCoords::Click {
                id: 1,
                coords: [10.0, 20.0],
            },
            EventWithCoords::Hover { id: 2 },
            EventWithCoords::Click {
                id: 3,
                coords: [30.0, 40.0],
            },
        ];

        let df = EventWithCoords::to_dataframe(rows);
        assert_eq!(df._tag.len(), 3);
        assert_eq!(df._tag[0], "Click");
        assert_eq!(df._tag[1], "Hover");

        // id is shared — always present
        assert_eq!(df.id, vec![Some(1), Some(2), Some(3)]);

        // coords expand to coords_1, coords_2 with Option
        assert_eq!(df.coords_1, vec![Some(10.0), None, Some(30.0)]);
        assert_eq!(df.coords_2, vec![Some(20.0), None, Some(40.0)]);
    }

    #[test]
    fn test_enum_pinned_vec_expansion() {
        let rows = vec![
            EventWithScores::Result {
                label: "a".into(),
                scores: vec![1.0, 2.0],
            },
            EventWithScores::Empty { label: "b".into() },
            EventWithScores::Result {
                label: "c".into(),
                scores: vec![3.0],
            },
        ];

        let df = EventWithScores::to_dataframe(rows);
        assert_eq!(
            df.label,
            vec![
                Some("a".to_string()),
                Some("b".to_string()),
                Some("c".to_string())
            ]
        );

        // scores_1, scores_2 — wrapped in Option<Option<f64>> → Option<f64> since enum already wraps
        // Actually: enum columns are Vec<Option<T>>. For ExpandedVec with elem_ty=f64,
        // the value pushed is `binding.get(i).cloned()` which is Option<f64>.
        // So the column is Vec<Option<Option<f64>>>? No — let me check.
        // In register_column, the col_ty is elem_ty (f64), and columns use Vec<Option<col_ty>>.
        // But the push is `binding.get(i).cloned()` which returns Option<f64>.
        // That's being pushed as `col.push(binding.get(i).cloned())` — that would be
        // pushing Option<f64> into Vec<Option<f64>>. That works!
        // For absent variant (Empty), it pushes None (which is Option<f64>::None).

        assert_eq!(df.scores_1, vec![Some(1.0), None, Some(3.0)]);
        assert_eq!(df.scores_2, vec![Some(2.0), None, None]);
    }

    #[test]
    fn test_enum_skip() {
        let rows = vec![
            EventWithSkip::A {
                value: 1.0,
                internal: 42,
            },
            EventWithSkip::B { value: 2.0 },
        ];

        let df = EventWithSkip::to_dataframe(rows);
        assert_eq!(df.value, vec![Some(1.0), Some(2.0)]);
        // No `internal` column
    }

    #[test]
    fn test_auto_expand_varying_lengths() {
        // Auto-expand stores Vec<Vec<T>> in companion struct.
        // The individual values_1, values_2, ... columns are generated
        // at runtime by IntoDataFrame (tested from R side).
        let rows = vec![
            WithAutoExpand {
                name: "a".into(),
                values: vec![1.0, 2.0, 3.0],
            },
            WithAutoExpand {
                name: "b".into(),
                values: vec![4.0],
            },
            WithAutoExpand {
                name: "c".into(),
                values: vec![5.0, 6.0],
            },
        ];

        let df = WithAutoExpand::to_dataframe(rows);
        assert_eq!(df.name.len(), 3);
        assert_eq!(df.values.len(), 3);
        assert_eq!(df.values[0], vec![1.0, 2.0, 3.0]);
        assert_eq!(df.values[1], vec![4.0]);
        assert_eq!(df.values[2], vec![5.0, 6.0]);
    }

    #[test]
    fn test_auto_expand_all_empty() {
        let rows = vec![
            WithAutoExpand {
                name: "a".into(),
                values: vec![],
            },
            WithAutoExpand {
                name: "b".into(),
                values: vec![],
            },
        ];

        let df = WithAutoExpand::to_dataframe(rows);
        assert_eq!(df.name.len(), 2);
        assert_eq!(df.values.len(), 2);
        assert!(df.values[0].is_empty());
        assert!(df.values[1].is_empty());
    }

    #[test]
    fn test_unnest_alias() {
        // `unnest` is an alias for `expand` — same auto-expand behavior.
        let rows = vec![
            WithUnnest {
                label: "x".into(),
                items: vec![10, 20],
            },
            WithUnnest {
                label: "y".into(),
                items: vec![30],
            },
        ];

        let df = WithUnnest::to_dataframe(rows);
        assert_eq!(df.label.len(), 2);
        assert_eq!(df.items.len(), 2);
        assert_eq!(df.items[0], vec![10, 20]);
        assert_eq!(df.items[1], vec![30]);
    }

    #[test]
    fn test_enum_auto_expand() {
        let rows = vec![
            EventWithAutoExpand::Scores {
                label: "a".into(),
                vals: vec![1.0, 2.0],
            },
            EventWithAutoExpand::Empty { label: "b".into() },
            EventWithAutoExpand::Scores {
                label: "c".into(),
                vals: vec![3.0],
            },
        ];

        let df = EventWithAutoExpand::to_dataframe(rows);
        assert_eq!(df._tag, vec!["Scores", "Empty", "Scores"]);
        assert_eq!(
            df.label,
            vec![
                Some("a".to_string()),
                Some("b".to_string()),
                Some("c".to_string()),
            ]
        );
        // vals is Vec<Option<Vec<f64>>> in companion struct
        assert_eq!(df.vals.len(), 3);
        assert_eq!(df.vals[0], Some(vec![1.0, 2.0]));
        assert_eq!(df.vals[1], None);
        assert_eq!(df.vals[2], Some(vec![3.0]));
    }

    #[test]
    fn test_boxed_slice_auto_expand() {
        let rows = vec![
            WithBoxedSliceExpand {
                name: "alice".into(),
                scores: vec![1.0, 2.0, 3.0].into_boxed_slice(),
            },
            WithBoxedSliceExpand {
                name: "bob".into(),
                scores: vec![4.0].into_boxed_slice(),
            },
            WithBoxedSliceExpand {
                name: "carol".into(),
                scores: vec![].into_boxed_slice(),
            },
        ];
        let df = WithBoxedSliceExpand::to_dataframe(rows);
        assert_eq!(df.name, vec!["alice", "bob", "carol"]);
        // Auto-expand stores Box<[f64]> in companion struct
        assert_eq!(df.scores.len(), 3);
        assert_eq!(&*df.scores[0], &[1.0, 2.0, 3.0]);
        assert_eq!(&*df.scores[1], &[4.0]);
        assert_eq!(&*df.scores[2], &[] as &[f64]);
    }

    #[test]
    fn test_boxed_slice_pinned_width() {
        let rows = vec![
            WithBoxedSlicePinned {
                name: "origin".into(),
                coords: vec![0.0, 0.0, 0.0].into_boxed_slice(),
            },
            WithBoxedSlicePinned {
                name: "unit".into(),
                coords: vec![1.0, 1.0].into_boxed_slice(),
            },
        ];
        let df = WithBoxedSlicePinned::to_dataframe(rows);
        assert_eq!(df.name, vec!["origin", "unit"]);
        // Pinned width = 3: coords_1, coords_2, coords_3
        assert_eq!(df.coords_1, vec![Some(0.0), Some(1.0)]);
        assert_eq!(df.coords_2, vec![Some(0.0), Some(1.0)]);
        assert_eq!(df.coords_3, vec![Some(0.0), None]);
    }

    #[test]
    fn test_slice_auto_expand() {
        let data_a = [1.0, 2.0, 3.0];
        let data_b = [4.0];
        let data_c = [5.0, 6.0];
        let rows = vec![
            WithSliceExpand {
                name: "a",
                values: &data_a,
            },
            WithSliceExpand {
                name: "b",
                values: &data_b,
            },
            WithSliceExpand {
                name: "c",
                values: &data_c,
            },
        ];

        let df = WithSliceExpand::to_dataframe(rows);
        assert_eq!(df.name.len(), 3);
        assert_eq!(df.values.len(), 3);
        // Companion stores Vec<&[f64]> — verify slice contents
        assert_eq!(df.values[0], &[1.0, 2.0, 3.0]);
        assert_eq!(df.values[1], &[4.0]);
        assert_eq!(df.values[2], &[5.0, 6.0]);
    }

    #[test]
    fn test_slice_pinned_width() {
        let coords_a = [10.0, 20.0];
        let coords_b = [30.0, 40.0, 50.0];
        let rows = vec![
            WithSlicePinned {
                label: "origin",
                coords: &coords_a,
            },
            WithSlicePinned {
                label: "far",
                coords: &coords_b,
            },
        ];

        let df = WithSlicePinned::to_dataframe(rows);
        assert_eq!(df.label, vec!["origin", "far"]);
        // Pinned width = 2: coords_1, coords_2
        assert_eq!(df.coords_1, vec![Some(10.0), Some(30.0)]);
        assert_eq!(df.coords_2, vec![Some(20.0), Some(40.0)]);
        // Third element of coords_b (50.0) is truncated
    }

    #[test]
    fn test_from_rows_struct() {
        // Test explicit from_rows (sequential)
        let rows: Vec<ParallelExpanded> = (0..10)
            .map(|i| ParallelExpanded {
                id: i,
                coords: [i as f64, 0.0, 0.0],
                tags: vec![format!("a{}", i)],
                values: vec![i as f64],
            })
            .collect();

        let df = ParallelExpandedDataFrame::from_rows(rows);
        assert_eq!(df.id.len(), 10);
        assert_eq!(df.coords_1[0], 0.0);
        assert_eq!(df.tags_1[5], Some("a5".to_string()));
        assert_eq!(df.tags_2[5], None);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn test_from_rows_par_struct() {
        // Test explicit from_rows_par (parallel)
        let rows: Vec<ParallelExpanded> = (0..5000)
            .map(|i| ParallelExpanded {
                id: i,
                coords: [i as f64, (i * 2) as f64, (i * 3) as f64],
                tags: vec![format!("t{}", i), format!("u{}", i)],
                values: vec![(i as f64) * 0.1, (i as f64) * 0.2],
            })
            .collect();

        let df = ParallelExpandedDataFrame::from_rows_par(rows);
        assert_eq!(df.id.len(), 5000);
        assert_eq!(df.coords_1.len(), 5000);
        assert_eq!(df.coords_2.len(), 5000);
        assert_eq!(df.coords_3.len(), 5000);
        assert_eq!(df.tags_1.len(), 5000);
        assert_eq!(df.tags_2.len(), 5000);
        assert_eq!(df.values.len(), 5000);

        // Verify values
        assert_eq!(df.id[0], 0);
        assert_eq!(df.id[4999], 4999);
        assert_eq!(df.coords_1[100], 100.0);
        assert_eq!(df.coords_2[100], 200.0);
        assert_eq!(df.coords_3[100], 300.0);
        assert_eq!(df.tags_1[42], Some("t42".to_string()));
        assert_eq!(df.tags_2[42], Some("u42".to_string()));
        assert_eq!(df.values[10], vec![1.0, 2.0]);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn test_from_rows_par_struct_matches_sequential() {
        // Verify both paths produce identical results
        let make_rows = || -> Vec<ParallelExpanded> {
            (0..100)
                .map(|i| ParallelExpanded {
                    id: i,
                    coords: [i as f64, (i * 2) as f64, (i * 3) as f64],
                    tags: vec![format!("t{}", i)],
                    values: vec![(i as f64) * 0.1],
                })
                .collect()
        };

        let df_seq = ParallelExpandedDataFrame::from_rows(make_rows());
        let df_par = ParallelExpandedDataFrame::from_rows_par(make_rows());

        assert_eq!(df_seq.id, df_par.id);
        assert_eq!(df_seq.coords_1, df_par.coords_1);
        assert_eq!(df_seq.coords_2, df_par.coords_2);
        assert_eq!(df_seq.coords_3, df_par.coords_3);
        assert_eq!(df_seq.tags_1, df_par.tags_1);
        assert_eq!(df_seq.tags_2, df_par.tags_2);
        assert_eq!(df_seq.values, df_par.values);
    }

    #[test]
    fn test_from_rows_enum() {
        // Test explicit from_rows (sequential) for enum
        let rows = vec![
            ParallelExpandedEvent::Measurement {
                sensor: "temp".into(),
                readings: [22.5, 23.0],
            },
            ParallelExpandedEvent::Status {
                sensor: "temp".into(),
                code: 200,
            },
        ];

        let df = ParallelExpandedEventDataFrame::from_rows(rows);
        assert_eq!(df._tag, vec!["Measurement", "Status"]);
        assert_eq!(
            df.sensor,
            vec![Some("temp".to_string()), Some("temp".to_string())]
        );
        assert_eq!(df.readings_1, vec![Some(22.5), None]);
        assert_eq!(df.readings_2, vec![Some(23.0), None]);
        assert_eq!(df.code, vec![None, Some(200)]);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn test_from_rows_par_enum() {
        // Test explicit from_rows_par (parallel) for enum
        let rows: Vec<ParallelExpandedEvent> = (0..5000)
            .map(|i| {
                if i % 2 == 0 {
                    ParallelExpandedEvent::Measurement {
                        sensor: format!("s{}", i),
                        readings: [i as f64, (i + 1) as f64],
                    }
                } else {
                    ParallelExpandedEvent::Status {
                        sensor: format!("s{}", i),
                        code: i,
                    }
                }
            })
            .collect();

        let df = ParallelExpandedEventDataFrame::from_rows_par(rows);
        assert_eq!(df._tag.len(), 5000);
        assert_eq!(df.sensor.len(), 5000);
        assert_eq!(df.readings_1.len(), 5000);
        assert_eq!(df.readings_2.len(), 5000);
        assert_eq!(df.code.len(), 5000);

        // Verify values
        assert_eq!(df._tag[0], "Measurement");
        assert_eq!(df._tag[1], "Status");
        assert_eq!(df.sensor[0], Some("s0".to_string()));
        assert_eq!(df.readings_1[0], Some(0.0));
        assert_eq!(df.readings_2[0], Some(1.0));
        assert_eq!(df.code[0], None);
        assert_eq!(df.code[1], Some(1));
        assert_eq!(df.readings_1[1], None);
    }

    #[test]
    fn test_lifetime_enum_owned() {
        // Verify enum DataFrameRow with owned types works end-to-end (issue #460).
        // Borrowed fields (&'a str) produce Vec<Option<&str>> which lacks IntoR in
        // enum context — use String instead. The struct path (WithSliceExpand, etc.)
        // supports borrowed &'a str / &'a [T] fields directly.
        let rows = vec![
            WithLifetimeEnumOwned::Row {
                label: "a".to_string(),
                value: 1.0,
            },
            WithLifetimeEnumOwned::Row {
                label: "b".to_string(),
                value: 2.0,
            },
        ];
        let df = WithLifetimeEnumOwned::to_dataframe(rows);
        assert_eq!(df.label.len(), 2);
        assert_eq!(df.label[0], Some("a".to_string()));
        assert_eq!(df.label[1], Some("b".to_string()));
        assert_eq!(df.value.len(), 2);
        assert_eq!(df.value[0], Some(1.0));
        assert_eq!(df.value[1], Some(2.0));
    }

    #[test]
    fn test_option_hashmap_as_list() {
        // Verify that `Option<HashMap<K, V>>` with `#[dataframe(as_list)]` compiles
        // and produces a single opaque list-column (#484 positive case).  Without the
        // annotation the macro now emits a compile error, so this test exists to confirm
        // the escape hatch works end-to-end.
        let rows = vec![
            WithOptMapAsList {
                id: 1,
                counts: None,
            },
            WithOptMapAsList {
                id: 2,
                counts: Some({
                    let mut m = HashMap::new();
                    m.insert("a".to_string(), 10);
                    m
                }),
            },
        ];
        let df = WithOptMapAsList::to_dataframe(rows);
        // Two rows → two entries in each companion column.
        assert_eq!(df.id.len(), 2);
        assert_eq!(df.id[0], 1);
        assert_eq!(df.id[1], 2);
        assert_eq!(df.counts.len(), 2);
    }
}
