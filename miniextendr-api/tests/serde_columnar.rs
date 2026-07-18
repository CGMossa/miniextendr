//! Integration tests for `NamedDataFrameListBuilder` and `vec_to_dataframe_split`.
//!
//! These tests require R to be initialized and run on the R main thread.

#![cfg(feature = "serde")]

mod r_test_utils;

use miniextendr_api::NamedDataFrameListBuilder;
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::into_r::IntoR as _;
use miniextendr_api::prelude::SexpExt as _;
use miniextendr_api::serde::{
    DataFrameShape, RSerdeError, ResultShape, SplitResults, SplitShape, dataframe_to_vec,
    dataframe_to_vec_collated, dataframe_to_vec_with_enum_tags,
    dataframe_to_vec_with_struct_fields, hashmap_to_dataframe, map_to_dataframe,
    result_to_dataframe, vec_to_dataframe, vec_to_dataframe_flatten_enums,
    vec_to_dataframe_flatten_enums_with_tags, vec_to_dataframe_split,
};
use serde::{Deserialize, Serialize};

// region: NamedDataFrameListBuilder

/// build() on a new() builder returns a 0-element named list.
#[test]
fn builder_empty_build_yields_empty_list() {
    r_test_utils::with_r_thread(|| {
        let list = NamedDataFrameListBuilder::new().build();
        let sexp = list.into_sexp();
        assert_eq!(sexp.xlength(), 0, "expected 0-element list");
    });
}

/// Pushing two data.frames and building produces a 2-element named list whose
/// element SEXPs are valid VECSXPs.
#[test]
fn builder_push_protects_input() {
    #[derive(Serialize)]
    struct Row {
        id: i32,
        val: f64,
    }

    r_test_utils::with_r_thread(|| {
        let oks: Vec<Row> = (0..10)
            .map(|i| Row {
                id: i,
                val: i as f64,
            })
            .collect();
        let errs: Vec<Row> = (0..5).map(|i| Row { id: i, val: -1.0 }).collect();

        let list = NamedDataFrameListBuilder::new()
            .push("results", *vec_to_dataframe(&oks).unwrap())
            .push("error", *vec_to_dataframe(&errs).unwrap())
            .build();

        let sexp = list.into_sexp();
        assert_eq!(sexp.xlength(), 2, "expected 2 entries");

        let elem0 = sexp.vector_elt(0);
        let elem1 = sexp.vector_elt(1);
        // Each element should be a VECSXP (data.frame); nrow = 10 and 5
        assert_eq!(elem0.xlength(), 2, "results df should have 2 columns");
        assert_eq!(elem1.xlength(), 2, "error df should have 2 columns");
    });
}

/// Dropping the builder without calling build() does not leave dangling
/// protections — the scope count returns to baseline.
#[test]
fn builder_drop_without_build() {
    #[derive(Serialize)]
    struct Row {
        x: i32,
    }

    r_test_utils::with_r_thread(|| unsafe {
        // Baseline: create a sibling scope and record its depth before the builder
        let outer = ProtectScope::new();
        let before = outer.count();

        {
            let rows: Vec<Row> = (0..3).map(|i| Row { x: i }).collect();
            let _builder =
                NamedDataFrameListBuilder::new().push("a", *vec_to_dataframe(&rows).unwrap());
            // _builder drops here, unprotecting via its internal scope
        }

        // Outer scope unaffected
        assert_eq!(
            outer.count(),
            before,
            "outer scope count should be unchanged after builder drop"
        );
        drop(outer);
    });
}

/// Entries pushed in order a → b → c appear in that order in the names
/// attribute of the result list.
#[test]
fn builder_with_capacity_preserves_order() {
    #[derive(Serialize)]
    struct Row {
        v: i32,
    }

    r_test_utils::with_r_thread(|| {
        let rows: Vec<Row> = vec![Row { v: 1 }];
        let list = NamedDataFrameListBuilder::with_capacity(3)
            .push("a", *vec_to_dataframe(&rows).unwrap())
            .push("b", *vec_to_dataframe(&rows).unwrap())
            .push("c", *vec_to_dataframe(&rows).unwrap())
            .build();

        let sexp = list.into_sexp();
        assert_eq!(sexp.xlength(), 3, "expected 3 entries");

        let names = sexp.get_names();
        assert_eq!(names.string_elt_str(0), Some("a"));
        assert_eq!(names.string_elt_str(1), Some("b"));
        assert_eq!(names.string_elt_str(2), Some("c"));
    });
}

// endregion

// region: vec_to_dataframe_split regression

/// vec_to_dataframe_split on a single-variant input returns a bare data.frame,
/// not a named list (single-variant short-circuit preserved after migration).
#[test]
fn vec_to_dataframe_split_single_variant_regression() {
    #[derive(Serialize)]
    enum E {
        Ok { id: i32 },
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![E::Ok { id: 1 }, E::Ok { id: 2 }];
        let shape = vec_to_dataframe_split(&rows, SplitShape::PerVariantList).unwrap();
        // Single-variant short-circuit: DataFrameShape::Bare; single-column df.
        assert!(matches!(shape, DataFrameShape::Bare(_)));
        let sexp = shape.into_sexp();
        assert_eq!(sexp.xlength(), 1, "single-column data.frame expected");
    });
}

/// vec_to_dataframe_split on a multi-variant input returns a PerVariantList shape.
#[test]
fn vec_to_dataframe_split_multi_variant_regression() {
    #[derive(Serialize)]
    enum E {
        Ok { id: i32 },
        Err { msg: String },
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            E::Ok { id: 1 },
            E::Err { msg: "oops".into() },
            E::Ok { id: 2 },
        ];
        let shape = vec_to_dataframe_split(&rows, SplitShape::PerVariantList).unwrap();
        assert!(matches!(shape, DataFrameShape::PerVariantList(_)));
        let sexp = shape.into_sexp();
        assert_eq!(sexp.xlength(), 2, "expected 2 named partitions");
    });
}

// endregion

// region: map_to_dataframe regression

#[test]
fn map_to_dataframe_btreemap_basic() {
    use std::collections::BTreeMap;

    #[derive(Serialize)]
    struct Summary {
        cmax: f64,
        tmax: f64,
    }

    r_test_utils::with_r_thread(|| {
        let mut map: BTreeMap<i32, Summary> = BTreeMap::new();
        map.insert(
            1,
            Summary {
                cmax: 10.5,
                tmax: 2.0,
            },
        );
        map.insert(
            2,
            Summary {
                cmax: 8.1,
                tmax: 3.0,
            },
        );

        let df = map_to_dataframe(&map, "subject").unwrap();
        let sexp = df.into_sexp();
        assert_eq!(
            sexp.xlength(),
            3,
            "expected 3 columns: subject + cmax + tmax"
        );

        let names = sexp.get_names();
        assert_eq!(names.string_elt_str(0), Some("subject"));
    });
}

#[test]
fn hashmap_to_dataframe_sorted_keys() {
    use std::collections::HashMap;

    #[derive(Serialize)]
    struct Row {
        v: i32,
    }

    r_test_utils::with_r_thread(|| {
        let mut map: HashMap<i32, Row> = HashMap::new();
        map.insert(3, Row { v: 30 });
        map.insert(1, Row { v: 10 });
        map.insert(2, Row { v: 20 });

        let df = hashmap_to_dataframe(&map, "id").unwrap();
        let sexp = df.into_sexp();
        assert_eq!(sexp.xlength(), 2, "expected 2 columns: id + v");
    });
}

// endregion

// region: result_to_dataframe regression

#[derive(Serialize)]
struct Obs {
    id: i32,
    value: f64,
}

#[derive(Serialize)]
struct ErrRow {
    id: i32,
    reason: String,
}

#[test]
fn result_to_dataframe_auto_all_ok_bare() {
    r_test_utils::with_r_thread(|| {
        let rows: Vec<Result<Obs, ErrRow>> =
            vec![Ok(Obs { id: 1, value: 1.0 }), Ok(Obs { id: 2, value: 2.0 })];
        let shape = result_to_dataframe(
            &rows,
            ResultShape::Auto {
                empty_ok_sentinel: (),
            },
        )
        .unwrap();
        assert!(matches!(shape, DataFrameShape::Bare(_)));
    });
}

#[test]
fn result_to_dataframe_auto_mixed_split() {
    r_test_utils::with_r_thread(|| {
        let rows: Vec<Result<Obs, ErrRow>> = vec![
            Ok(Obs { id: 1, value: 1.0 }),
            Err(ErrRow {
                id: 2,
                reason: "bad".into(),
            }),
        ];
        let shape = result_to_dataframe(
            &rows,
            ResultShape::Auto {
                empty_ok_sentinel: (),
            },
        )
        .unwrap();
        assert!(matches!(
            shape,
            DataFrameShape::Split {
                results: SplitResults::Some(_),
                ..
            }
        ));
        let sexp = shape.into_sexp();
        assert_eq!(sexp.xlength(), 2, "results + error elements");
    });
}

#[test]
fn result_to_dataframe_split_all_err_uses_sentinel() {
    r_test_utils::with_r_thread(|| {
        let rows: Vec<Result<Obs, ErrRow>> = vec![Err(ErrRow {
            id: 1,
            reason: "x".into(),
        })];
        let shape = result_to_dataframe(
            &rows,
            ResultShape::Split {
                empty_ok_sentinel: "no ok rows".to_string(),
            },
        )
        .unwrap();
        let DataFrameShape::Split {
            results: SplitResults::None(sentinel),
            ..
        } = &shape
        else {
            panic!("expected Split with sentinel");
        };
        // The sentinel carries its own GC root (#1265) and exposes the
        // rooted SEXP for inspection.
        let s = sentinel.as_sexp();
        assert_eq!(s.xlength(), 1, "sentinel STRSXP length");
        assert_eq!(s.string_elt_str(0), Some("no ok rows"));
    });
}

/// A `DataFrameShape` is rooted by construction (#1265): holding it across R
/// allocations and converting later must preserve the children — the
/// pre-#1265 convert-immediately contract is gone.
#[test]
fn dataframe_shape_survives_allocations_before_conversion() {
    #[derive(Serialize)]
    enum E {
        Click { x: f64 },
        Scroll { delta: f64 },
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![E::Click { x: 1.0 }, E::Scroll { delta: -1.0 }];
        let shape = vec_to_dataframe_split(&rows, SplitShape::PerVariantList).unwrap();

        // Allocate after the helper returned but before the conversion.
        for i in 0..64i32 {
            let v: Vec<f64> = (0..32).map(|j| f64::from(i + j)).collect();
            let _ = v.into_sexp();
        }

        let DataFrameShape::PerVariantList(parts) = shape else {
            panic!("expected PerVariantList");
        };
        assert_eq!(parts.len(), 2);
        for (name, df) in &parts {
            assert_eq!(df.nrow(), 1, "partition {name} corrupted");
        }
    });
}

#[test]
fn result_to_dataframe_collated_yields_bare() {
    r_test_utils::with_r_thread(|| {
        let rows: Vec<Result<Obs, ErrRow>> = vec![
            Ok(Obs { id: 1, value: 1.0 }),
            Err(ErrRow {
                id: 2,
                reason: "bad".into(),
            }),
        ];
        let shape = result_to_dataframe(&rows, ResultShape::<()>::Collated).unwrap();
        let DataFrameShape::Bare(df) = shape else {
            panic!("expected Bare");
        };
        let sexp = df.into_sexp();
        // is_error + id (union of Obs.id and ErrRow.id) + value + reason
        assert_eq!(sexp.xlength(), 4, "is_error + id + value + reason");
        let names = sexp.get_names();
        assert_eq!(
            names.string_elt_str(0),
            Some("is_error"),
            "is_error first column"
        );
    });
}

// endregion

// region: SplitShape variants

#[test]
fn split_pervariantlist_with_tag_prepends_column() {
    #[derive(Serialize)]
    enum E {
        Click { x: f64 },
        Scroll { delta: f64 },
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![E::Click { x: 1.0 }, E::Scroll { delta: -1.0 }];
        let shape = vec_to_dataframe_split(
            &rows,
            SplitShape::PerVariantListWithTag {
                column: "variant".into(),
            },
        )
        .unwrap();
        let DataFrameShape::PerVariantList(parts) = shape else {
            panic!("expected PerVariantList");
        };
        assert_eq!(parts.len(), 2);
        // Each partition df should have a leading "variant" column.
        for (name, df) in parts {
            let sexp = df.into_sexp();
            let names = sexp.get_names();
            assert_eq!(
                names.string_elt_str(0),
                Some("variant"),
                "variant column should be first in {name}"
            );
        }
    });
}

#[test]
fn split_collated_emits_single_df_with_tag() {
    #[derive(Serialize)]
    enum E {
        Click { x: f64 },
        Scroll { delta: f64 },
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![E::Click { x: 1.0 }, E::Scroll { delta: -1.0 }];
        let shape = vec_to_dataframe_split(
            &rows,
            SplitShape::Collated {
                column: "kind".into(),
            },
        )
        .unwrap();
        let DataFrameShape::Bare(df) = shape else {
            panic!("expected Bare");
        };
        let sexp = df.into_sexp();
        // kind + x + delta = 3 columns
        assert_eq!(sexp.xlength(), 3, "kind + x + delta");
        let names = sexp.get_names();
        assert_eq!(names.string_elt_str(0), Some("kind"));
    });
}

#[test]
fn split_collated_empty_input_errors() {
    #[derive(Serialize)]
    #[allow(dead_code)]
    enum E {
        Click { x: f64 },
    }

    r_test_utils::with_r_thread(|| {
        let rows: Vec<E> = Vec::new();
        let res = vec_to_dataframe_split(&rows, SplitShape::Collated { column: "k".into() });
        assert!(res.is_err(), "collated empty input must error");
    });
}

// endregion

// region: vec_to_dataframe_flatten_enums (#1056)

/// The ordered column names of a data.frame SEXP.
fn col_names(sexp: &miniextendr_api::SEXP) -> Vec<String> {
    let names = sexp.get_names();
    (0..names.xlength())
        .map(|i| names.string_elt_str(i).unwrap_or("<NA>").to_string())
        .collect()
}

/// The column SEXP for `name`, or panic.
fn col(sexp: &miniextendr_api::SEXP, name: &str) -> miniextendr_api::SEXP {
    let names = sexp.get_names();
    for i in 0..names.xlength() {
        if names.string_elt_str(i) == Some(name) {
            return sexp.vector_elt(i);
        }
    }
    panic!("column {name:?} not found in {:?}", col_names(sexp));
}

/// Externally-tagged enum field, two struct variants across rows: the enum
/// field flattens into a `action_variant` tag plus the union of the variants'
/// payload columns, NA-filled where a variant lacks a field.
#[test]
fn flatten_enum_field_externally_tagged_two_variants() {
    #[derive(Serialize)]
    enum Action {
        Add { file: f64, weight: f64 },
        Init { path: String },
    }
    #[derive(Serialize)]
    struct AuditRow {
        id: i32,
        action: Action,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            AuditRow {
                id: 1,
                action: Action::Add {
                    file: 10.0,
                    weight: 2.5,
                },
            },
            AuditRow {
                id: 2,
                action: Action::Init {
                    path: "/tmp".into(),
                },
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["action"]).unwrap();
        let sexp = df.into_sexp();
        let names = col_names(&sexp);
        // id, action_variant, then the union of Add (file, weight) + Init (path).
        assert_eq!(names[0], "id", "scalar field stays first");
        assert!(
            names.contains(&"action_variant".to_string()),
            "tag column present, got {names:?}"
        );
        assert!(names.contains(&"action_file".to_string()), "{names:?}");
        assert!(names.contains(&"action_weight".to_string()), "{names:?}");
        assert!(names.contains(&"action_path".to_string()), "{names:?}");

        // Tag values.
        let variant = col(&sexp, "action_variant");
        assert_eq!(variant.string_elt_str(0), Some("Add"));
        assert_eq!(variant.string_elt_str(1), Some("Init"));

        // Row 0 (Add) has file=10, weight=2.5, path=NA.
        let file = col(&sexp, "action_file");
        assert_eq!(file.real_elt(0), 10.0);
        assert!(file.real_elt(1).is_nan(), "Init row file should be NA");
        let path = col(&sexp, "action_path");
        assert_eq!(path.string_elt_str(0), None, "Add row path should be NA");
        assert_eq!(path.string_elt_str(1), Some("/tmp"));
    });
}

/// A unit variant mixed with a data variant: unit rows get the tag plus NA for
/// the data variant's payload columns.
#[test]
fn flatten_enum_field_unit_variant_mixed() {
    #[derive(Serialize)]
    enum Action {
        Reset,
        Set { value: f64 },
    }
    #[derive(Serialize)]
    struct Row {
        id: i32,
        action: Action,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                action: Action::Set { value: 7.0 },
            },
            Row {
                id: 2,
                action: Action::Reset,
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["action"]).unwrap();
        let sexp = df.into_sexp();
        let variant = col(&sexp, "action_variant");
        assert_eq!(variant.string_elt_str(0), Some("Set"));
        assert_eq!(variant.string_elt_str(1), Some("Reset"));
        let value = col(&sexp, "action_value");
        assert_eq!(value.real_elt(0), 7.0);
        assert!(
            value.real_elt(1).is_nan(),
            "unit-variant row payload should be NA"
        );
    });
}

/// `Option<Enum>` with a `None` row: the tag and payload columns are NA for
/// that row.
#[test]
fn flatten_enum_field_option_none() {
    #[derive(Serialize)]
    enum Action {
        Go { steps: f64 },
    }
    #[derive(Serialize)]
    struct Row {
        id: i32,
        action: Option<Action>,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                action: Some(Action::Go { steps: 3.0 }),
            },
            Row {
                id: 2,
                action: None,
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["action"]).unwrap();
        let sexp = df.into_sexp();
        let variant = col(&sexp, "action_variant");
        assert_eq!(variant.string_elt_str(0), Some("Go"));
        assert_eq!(variant.string_elt_str(1), None, "None row tag should be NA");
        let steps = col(&sexp, "action_steps");
        assert_eq!(steps.real_elt(0), 3.0);
        assert!(steps.real_elt(1).is_nan(), "None row payload should be NA");
    });
}

/// Newtype variant wrapping a struct: its fields flatten under the prefix.
#[test]
fn flatten_enum_field_newtype_wrapping_struct() {
    #[derive(Serialize)]
    struct Coords {
        x: f64,
        y: f64,
    }
    #[derive(Serialize)]
    enum Shape {
        Point(Coords),
        Empty,
    }
    #[derive(Serialize)]
    struct Row {
        id: i32,
        shape: Shape,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                shape: Shape::Point(Coords { x: 1.0, y: 2.0 }),
            },
            Row {
                id: 2,
                shape: Shape::Empty,
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["shape"]).unwrap();
        let sexp = df.into_sexp();
        let names = col_names(&sexp);
        assert!(names.contains(&"shape_variant".to_string()), "{names:?}");
        assert!(names.contains(&"shape_x".to_string()), "{names:?}");
        assert!(names.contains(&"shape_y".to_string()), "{names:?}");
        let x = col(&sexp, "shape_x");
        assert_eq!(x.real_elt(0), 1.0);
        assert!(x.real_elt(1).is_nan(), "Empty row x should be NA");
    });
}

/// A nested enum field NOT named in `flatten_enum_fields` retains the default
/// behaviour — it becomes a single list-column, not flattened.
#[test]
fn flatten_enum_field_untargeted_enum_stays_list_column() {
    #[derive(Serialize)]
    enum Action {
        Add { file: f64 },
    }
    #[derive(Serialize)]
    struct Row {
        id: i32,
        action: Action,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![Row {
            id: 1,
            action: Action::Add { file: 1.0 },
        }];
        // Empty target set: nothing flattened.
        let df = vec_to_dataframe_flatten_enums(&rows, &[]).unwrap();
        let sexp = df.into_sexp();
        let names = col_names(&sexp);
        assert_eq!(names, vec!["id".to_string(), "action".to_string()]);
        // The untargeted enum is a Generic (list) column.
        let action = col(&sexp, "action");
        assert_eq!(
            action.type_of(),
            miniextendr_api::SEXPTYPE::VECSXP,
            "untargeted enum field must remain a list-column"
        );
    });
}

/// A non-enum nested struct field is unaffected by the new fn: it still
/// flattens to `parent_child` columns exactly as plain `vec_to_dataframe`.
#[test]
fn flatten_enum_field_nested_struct_unaffected() {
    #[derive(Serialize)]
    struct Meta {
        size: f64,
        ok: bool,
    }
    #[derive(Serialize)]
    struct Row {
        id: i32,
        meta: Meta,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                meta: Meta {
                    size: 4.0,
                    ok: true,
                },
            },
            Row {
                id: 2,
                meta: Meta {
                    size: 9.0,
                    ok: false,
                },
            },
        ];
        // "meta" is targeted but it's NOT an enum — it must error (only enum /
        // Option<enum> fields are flattenable).
        assert!(
            vec_to_dataframe_flatten_enums(&rows, &["meta"]).is_err(),
            "targeting a non-enum struct field must error"
        );
        // Untargeted: behaves exactly like vec_to_dataframe — meta_size/meta_ok.
        let df = vec_to_dataframe_flatten_enums(&rows, &[]).unwrap();
        let sexp = df.into_sexp();
        let names = col_names(&sexp);
        assert_eq!(
            names,
            vec![
                "id".to_string(),
                "meta_size".to_string(),
                "meta_ok".to_string()
            ]
        );
    });
}

/// An internally-tagged enum field (`#[serde(tag = ...)]`) is flattened in
/// place: the embedded tag rides along as a normal `<field>_<tag>` column and
/// NO duplicate synthetic `<field>_variant` is emitted.
#[test]
fn flatten_enum_field_internally_tagged_no_synthetic_variant() {
    #[derive(Serialize)]
    #[serde(tag = "kind")]
    enum Action {
        Add { file: f64 },
        Reset,
    }
    #[derive(Serialize)]
    struct Row {
        id: i32,
        action: Action,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                action: Action::Add { file: 2.0 },
            },
            Row {
                id: 2,
                action: Action::Reset,
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["action"]).unwrap();
        let sexp = df.into_sexp();
        let names = col_names(&sexp);
        // The embedded tag becomes action_kind; no synthetic action_variant.
        assert!(names.contains(&"action_kind".to_string()), "{names:?}");
        assert!(
            !names.contains(&"action_variant".to_string()),
            "internally-tagged field must not synthesize a duplicate variant tag, got {names:?}"
        );
        let kind = col(&sexp, "action_kind");
        assert_eq!(kind.string_elt_str(0), Some("Add"));
        assert_eq!(kind.string_elt_str(1), Some("Reset"));
        let file = col(&sexp, "action_file");
        assert_eq!(file.real_elt(0), 2.0);
        assert!(file.real_elt(1).is_nan(), "Reset row file should be NA");
    });
}

// endregion

// region: split/flattened enum reader round-trip (#1060 + #1061)

/// Nested enum field round-trip: unit + struct variants across rows, asserting
/// full `Vec<T>` equality after write → read.
#[test]
fn flatten_enum_reader_struct_and_unit_variants_roundtrip() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum Action {
        Reset,
        Add { file: f64, weight: f64 },
        Init { path: String },
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct AuditRow {
        id: i32,
        action: Action,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            AuditRow {
                id: 1,
                action: Action::Add {
                    file: 10.0,
                    weight: 2.5,
                },
            },
            AuditRow {
                id: 2,
                action: Action::Init {
                    path: "/tmp".into(),
                },
            },
            AuditRow {
                id: 3,
                action: Action::Reset,
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["action"]).unwrap();
        let sexp = df.as_sexp();
        // The default reader (no config) reads flattened enum fields via the
        // `<field>_variant` convention.
        let round: Vec<AuditRow> = dataframe_to_vec(sexp).unwrap();
        assert_eq!(round, rows);
    });
}

/// Newtype variant wrapping a struct round-trips (its inner fields flatten
/// under the prefix).
#[test]
fn flatten_enum_reader_newtype_variant_roundtrip() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Coords {
        x: f64,
        y: f64,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum Shape {
        Empty,
        Point(Coords),
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
        shape: Shape,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                shape: Shape::Point(Coords { x: 1.0, y: 2.0 }),
            },
            Row {
                id: 2,
                shape: Shape::Empty,
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["shape"]).unwrap();
        let round: Vec<Row> = dataframe_to_vec(df.as_sexp()).unwrap();
        assert_eq!(round, rows);
    });
}

/// `Option`-bearing payload field with a `None` row round-trips (the payload
/// cell is NA, read back as `None`).
#[test]
fn flatten_enum_reader_option_payload_field_roundtrip() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum Action {
        Go { steps: Option<f64>, label: String },
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
        action: Action,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                action: Action::Go {
                    steps: Some(3.0),
                    label: "a".into(),
                },
            },
            Row {
                id: 2,
                action: Action::Go {
                    steps: None,
                    label: "b".into(),
                },
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["action"]).unwrap();
        let round: Vec<Row> = dataframe_to_vec(df.as_sexp()).unwrap();
        assert_eq!(round, rows);
    });
}

/// `Option<Enum>` field with a `None` row: the NA tag column reads back as
/// `None`; a present row reads back as `Some(variant)`.
#[test]
fn flatten_enum_reader_option_enum_none_roundtrip() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum Action {
        Go { steps: f64 },
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
        action: Option<Action>,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                action: Some(Action::Go { steps: 3.0 }),
            },
            Row {
                id: 2,
                action: None,
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["action"]).unwrap();
        let round: Vec<Row> = dataframe_to_vec(df.as_sexp()).unwrap();
        assert_eq!(round, rows);
    });
}

/// #1320 repro, default behaviour pinned: an `Option<NestedStruct>` whose
/// sub-field is literally named `variant` collides with the enum tag-column
/// name (`meta_variant`) — an NA in that cell reads the *whole struct* back as
/// `None`, dropping the other sub-fields. This is the documented lossy
/// default; [`option_nested_struct_with_struct_fields_roundtrips`] is the fix.
#[test]
fn option_nested_struct_tag_collision_default_is_lossy() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Meta {
        variant: Option<String>,
        size: f64,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
        meta: Option<Meta>,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                meta: Some(Meta {
                    variant: Some("a".into()),
                    size: 1.0,
                }),
            },
            Row {
                id: 2,
                meta: Some(Meta {
                    variant: None,
                    size: 4.0,
                }),
            },
        ];
        // Columns: id, meta_variant, meta_size — `meta_variant` is NA at row 2.
        let df = vec_to_dataframe(&rows).unwrap();
        let round: Vec<Row> = dataframe_to_vec(df.as_sexp()).unwrap();
        // Row 1 survives (tag cell not NA → nested-struct read).
        assert_eq!(round[0], rows[0]);
        // Row 2's struct is silently dropped by the heuristic. If this
        // assertion starts failing, the lossy default documented in #1320
        // changed — update the docs alongside.
        assert_eq!(round[1], Row { id: 2, meta: None });
    });
}

/// #1320 fix: listing the field in `dataframe_to_vec_with_struct_fields`
/// disables the tag-column heuristic for it — the
/// `Meta { variant: None, size: 4.0 }` row round-trips losslessly.
#[test]
fn option_nested_struct_with_struct_fields_roundtrips() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Meta {
        variant: Option<String>,
        size: f64,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
        meta: Option<Meta>,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                meta: Some(Meta {
                    variant: Some("a".into()),
                    size: 1.0,
                }),
            },
            Row {
                id: 2,
                meta: Some(Meta {
                    variant: None,
                    size: 4.0,
                }),
            },
        ];
        let df = vec_to_dataframe(&rows).unwrap();
        let round: Vec<Row> = dataframe_to_vec_with_struct_fields(df.as_sexp(), &["meta"]).unwrap();
        assert_eq!(round, rows);
    });
}

/// The struct-field opt-out is per-field: `Option<Enum>` `None` rows keep
/// round-tripping through `dataframe_to_vec_with_struct_fields` for fields
/// *not* listed — the tag-column heuristic still fires for them.
#[test]
fn struct_fields_optout_keeps_option_enum_none_for_other_fields() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum Action {
        Go { steps: f64 },
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Meta {
        variant: Option<String>,
        size: f64,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
        meta: Option<Meta>,
        action: Option<Action>,
    }

    r_test_utils::with_r_thread(|| {
        // Row 1 keeps `meta.variant = Some(...)`; row 2 has `meta.variant =
        // None`. Prior to #1370's recursive Compound union this row order
        // avoided a *separate* writer limitation (a nested sub-field probed
        // as `None` on the first row locked its column to Generic/list) that
        // would otherwise be conflated with the #1320 heuristic under test
        // here. #1370 is now fixed (see `option_nested_struct_tag_collision_...`-
        // adjacent tests in this file for direct coverage), so this ordering
        // is no longer load-bearing for the writer — kept as-is since it's
        // still a valid, order-independent regression for the #1320 opt-out.
        let rows = vec![
            Row {
                id: 1,
                meta: Some(Meta {
                    variant: Some("x".into()),
                    size: 5.0,
                }),
                action: Some(Action::Go { steps: 3.0 }),
            },
            Row {
                id: 2,
                meta: Some(Meta {
                    variant: None,
                    size: 4.0,
                }),
                action: None,
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["action"]).unwrap();
        // `meta` is opted out (always a struct); `action` keeps the default
        // `action_variant` tag heuristic, so its `None` row still reads back.
        let round: Vec<Row> = dataframe_to_vec_with_struct_fields(df.as_sexp(), &["meta"]).unwrap();
        assert_eq!(round, rows);
    });
}

/// Top-level Collated frame round-trips: the whole row is an enum, tag column
/// named by the caller (symmetric with `SplitShape::Collated { column }`).
#[test]
fn collated_reader_roundtrip() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum Event {
        Login { user: String },
        Logout,
        Click { x: f64, y: f64 },
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Event::Login { user: "a".into() },
            Event::Logout,
            Event::Click { x: 1.0, y: 2.0 },
        ];
        let shape = vec_to_dataframe_split(
            &rows,
            SplitShape::Collated {
                column: "kind".into(),
            },
        )
        .unwrap();
        let DataFrameShape::Bare(ref df) = shape else {
            panic!("Collated shape must be Bare, got a different shape");
        };
        let round: Vec<Event> = dataframe_to_vec_collated(df.as_sexp(), "kind").unwrap();
        assert_eq!(round, rows);
    });
}

/// Custom tag-column names round-trip on both the writer and reader
/// (#1061): the tag column is `state_tag`, payload columns stay `state_<sub>`.
#[test]
fn flatten_enum_reader_custom_tag_name_roundtrip() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum State {
        On,
        Off { reason: String },
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
        state: State,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                state: State::On,
            },
            Row {
                id: 2,
                state: State::Off {
                    reason: "boom".into(),
                },
            },
        ];
        let tags = [("state", "state_tag")];
        let df = vec_to_dataframe_flatten_enums_with_tags(&rows, &["state"], &tags).unwrap();
        let sexp = df.as_sexp();
        // Writer emitted the custom tag column, not the default.
        let names = col_names(&sexp);
        assert!(
            names.contains(&"state_tag".to_string()),
            "custom tag column present, got {names:?}"
        );
        assert!(
            !names.contains(&"state_variant".to_string()),
            "default tag column must NOT be emitted, got {names:?}"
        );
        // Reader honours the same mapping.
        let round: Vec<Row> = dataframe_to_vec_with_enum_tags(sexp, &tags).unwrap();
        assert_eq!(round, rows);
    });
}

/// Reading with a tag mapping that names a non-existent column errors, naming
/// the expected column.
#[test]
fn flatten_enum_reader_missing_tag_column_errors() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum Action {
        Add { file: f64 },
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
        action: Action,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![Row {
            id: 1,
            action: Action::Add { file: 2.0 },
        }];
        // Writer used the default `action_variant`; reader asks for `action_tag`.
        let df = vec_to_dataframe_flatten_enums(&rows, &["action"]).unwrap();
        let err = dataframe_to_vec_with_enum_tags::<Row>(df.as_sexp(), &[("action", "action_tag")])
            .unwrap_err();
        let RSerdeError::Message(msg) = err else {
            panic!("expected a Message error, got {err:?}");
        };
        assert!(
            msg.contains("action_tag"),
            "error must name the expected tag column, got: {msg}"
        );
    });
}

/// An unknown variant string in the tag cell surfaces serde's standard
/// `unknown variant` error listing the known variants.
#[test]
fn flatten_enum_reader_unknown_variant_errors() {
    #[derive(Serialize)]
    enum Wide {
        Add { file: f64 },
        Bogus { file: f64 },
    }
    #[derive(Serialize)]
    struct WideRow {
        id: i32,
        action: Wide,
    }
    // `Narrow` knows only `Add`. It also derives Serialize + PartialEq so the
    // test can prove a positive round-trip first (which reads the fields) and
    // then hit the unknown-variant error on the wider frame.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum Narrow {
        Add { file: f64 },
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct NarrowRow {
        id: i32,
        action: Narrow,
    }

    r_test_utils::with_r_thread(|| {
        // Positive control: an all-`Add` frame round-trips as `NarrowRow`.
        let narrow = vec![NarrowRow {
            id: 1,
            action: Narrow::Add { file: 1.0 },
        }];
        let ndf = vec_to_dataframe_flatten_enums(&narrow, &["action"]).unwrap();
        let round: Vec<NarrowRow> = dataframe_to_vec(ndf.as_sexp()).unwrap();
        assert_eq!(round, narrow);

        // A frame carrying the unknown `Bogus` variant fails on that row.
        let wide = vec![
            WideRow {
                id: 1,
                action: Wide::Add { file: 1.0 },
            },
            WideRow {
                id: 2,
                action: Wide::Bogus { file: 2.0 },
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&wide, &["action"]).unwrap();
        let err = dataframe_to_vec::<NarrowRow>(df.as_sexp()).unwrap_err();
        let RSerdeError::Message(msg) = err else {
            panic!("expected a Message error, got {err:?}");
        };
        assert!(
            msg.contains("Bogus") || msg.to_lowercase().contains("unknown variant"),
            "error must flag the unknown variant, got: {msg}"
        );
    });
}

/// Tuple variant round-trips: payload lands as `<field>__0` / `<field>__1`
/// (the writer's `_N` convention under the field prefix) and the
/// `EnumTupleSeqAccess` reader reassembles it; the unit-variant row leaves the
/// tuple columns NA and reads back untouched.
#[test]
fn flatten_enum_reader_tuple_variant_roundtrip() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum Measure {
        Empty,
        Pair(f64, f64),
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
        m: Measure,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                m: Measure::Pair(1.5, -2.0),
            },
            Row {
                id: 2,
                m: Measure::Empty,
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["m"]).unwrap();
        let sexp = df.as_sexp();
        // Pin the writer's tuple-payload column names before reading back.
        let names = col_names(&sexp);
        assert!(names.contains(&"m_variant".to_string()), "{names:?}");
        assert!(names.contains(&"m__0".to_string()), "{names:?}");
        assert!(names.contains(&"m__1".to_string()), "{names:?}");
        let round: Vec<Row> = dataframe_to_vec(sexp).unwrap();
        assert_eq!(round, rows);
    });
}

// endregion

// region: recursive Compound union in schema discovery (#1370)

/// #1370 repro, plain (non-`Option`) nested struct: a sub-field probed as
/// `None` on the *first* row must still type as an atomic column (not a
/// Generic/list column) once a later row types it.
#[test]
fn compound_none_first_nested_subfield_types_atomic() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Meta {
        variant: Option<String>,
        size: f64,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
        meta: Meta,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                meta: Meta {
                    variant: None,
                    size: 4.0,
                },
            },
            Row {
                id: 2,
                meta: Meta {
                    variant: Some("x".into()),
                    size: 5.0,
                },
            },
        ];
        let df = vec_to_dataframe(&rows).unwrap();
        let sexp = df.as_sexp();
        let variant_col = col(&sexp, "meta_variant");
        assert_eq!(
            variant_col.type_of(),
            miniextendr_api::SEXPTYPE::STRSXP,
            "None-first nested sub-field must type as character, not a list column"
        );
        assert_eq!(variant_col.string_elt_str(0), None, "row 1 is NA");
        assert_eq!(variant_col.string_elt_str(1), Some("x"));
        let round: Vec<Row> = dataframe_to_vec(sexp).unwrap();
        assert_eq!(round, rows);
    });
}

/// Same repro through an `Option<Meta>` outer field (the issue notes "plain
/// or `Option<Meta>` — same result"): the outer `Option` resolves to
/// `Compound` (both rows are `Some`), so the same nested-sub-field-`None`-
/// first bug applies to `meta_variant` underneath it.
///
/// Reads back via the `dataframe_to_vec_with_struct_fields` opt-out (not the
/// default `dataframe_to_vec`) so this test isolates the *writer* fix (#1370)
/// from the *reader*'s separate `variant`-name tag-column collision (#1320,
/// covered by `option_nested_struct_tag_collision_default_is_lossy` and
/// `compound_fix_resolves_1320_interaction_none_first_no_longer_errors`).
#[test]
fn compound_none_first_nested_subfield_types_atomic_through_option_outer() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Meta {
        variant: Option<String>,
        size: f64,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
        meta: Option<Meta>,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                meta: Some(Meta {
                    variant: None,
                    size: 4.0,
                }),
            },
            Row {
                id: 2,
                meta: Some(Meta {
                    variant: Some("x".into()),
                    size: 5.0,
                }),
            },
        ];
        let df = vec_to_dataframe(&rows).unwrap();
        let sexp = df.as_sexp();
        let variant_col = col(&sexp, "meta_variant");
        assert_eq!(
            variant_col.type_of(),
            miniextendr_api::SEXPTYPE::STRSXP,
            "None-first nested sub-field must type as character, not a list column"
        );
        let round: Vec<Row> = dataframe_to_vec_with_struct_fields(sexp, &["meta"]).unwrap();
        assert_eq!(round, rows);
    });
}

/// Row order must not change the emitted schema: the same two rows, reversed
/// (`Some`-first instead of `None`-first), produce the identical column set
/// and types for the nested sub-field.
#[test]
fn compound_schema_is_row_order_independent() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Meta {
        variant: Option<String>,
        size: f64,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
        meta: Meta,
    }

    r_test_utils::with_r_thread(|| {
        let none_first = vec![
            Row {
                id: 1,
                meta: Meta {
                    variant: None,
                    size: 4.0,
                },
            },
            Row {
                id: 2,
                meta: Meta {
                    variant: Some("x".into()),
                    size: 5.0,
                },
            },
        ];
        let some_first = vec![none_first[1].clone(), none_first[0].clone()];

        let df_none_first = vec_to_dataframe(&none_first).unwrap();
        let df_some_first = vec_to_dataframe(&some_first).unwrap();

        let names_a = col_names(&df_none_first.as_sexp());
        let names_b = col_names(&df_some_first.as_sexp());
        assert_eq!(
            names_a, names_b,
            "column set/order must match regardless of row order"
        );

        let type_a = col(&df_none_first.as_sexp(), "meta_variant").type_of();
        let type_b = col(&df_some_first.as_sexp(), "meta_variant").type_of();
        assert_eq!(
            type_a, type_b,
            "meta_variant's column type must not depend on row order"
        );
        assert_eq!(
            type_a,
            miniextendr_api::SEXPTYPE::STRSXP,
            "both orderings must resolve to character"
        );
    });
}

/// Two `Compound` candidates of genuinely *different* shapes (a
/// `#[serde(skip_serializing_if)]` sub-field entirely absent from one row's
/// probe, present in another's) must union their keys rather than drop the
/// side not seen first.
#[test]
fn compound_union_of_keys_when_shapes_differ() {
    #[derive(Debug, Serialize)]
    struct Meta {
        #[serde(skip_serializing_if = "Option::is_none")]
        extra: Option<String>,
        size: f64,
    }
    #[derive(Debug, Serialize)]
    struct Row {
        id: i32,
        meta: Meta,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                meta: Meta {
                    extra: None, // `extra` is omitted entirely from row 1's probe
                    size: 1.0,
                },
            },
            Row {
                id: 2,
                meta: Meta {
                    extra: Some("x".into()),
                    size: 2.0,
                },
            },
        ];
        let df = vec_to_dataframe(&rows).unwrap();
        let sexp = df.as_sexp();
        let names = col_names(&sexp);
        assert!(
            names.contains(&"meta_extra".to_string()),
            "row 2's extra field must not be dropped just because row 1's Compound omitted it: {names:?}"
        );
        assert!(names.contains(&"meta_size".to_string()), "{names:?}");
        let extra = col(&sexp, "meta_extra");
        assert_eq!(extra.string_elt_str(0), None, "row 1 omitted extra -> NA");
        assert_eq!(extra.string_elt_str(1), Some("x"));
        let size = col(&sexp, "meta_size");
        assert_eq!(size.real_elt(0), 1.0);
        assert_eq!(size.real_elt(1), 2.0);
    });
}

/// #1320 interaction (issue's point 3): with the writer bug present, a
/// colliding `variant` sub-field would flip failure mode with row order —
/// `Some`-first silently drops the struct (the #1320 default), `None`-first
/// errored loudly with a type mismatch. Post-#1370, `None`-first no longer
/// errors: it hits the *same* documented #1320 lossy default as `Some`-first
/// (the opt-out in `option_nested_struct_with_struct_fields_roundtrips`
/// still fixes it losslessly, order notwithstanding).
#[test]
fn compound_fix_resolves_1320_interaction_none_first_no_longer_errors() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Meta {
        variant: Option<String>,
        size: f64,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
        meta: Option<Meta>,
    }

    r_test_utils::with_r_thread(|| {
        // None-first (reverse of `option_nested_struct_tag_collision_default_is_lossy`).
        let rows = vec![
            Row {
                id: 2,
                meta: Some(Meta {
                    variant: None,
                    size: 4.0,
                }),
            },
            Row {
                id: 1,
                meta: Some(Meta {
                    variant: Some("a".into()),
                    size: 1.0,
                }),
            },
        ];
        let df = vec_to_dataframe(&rows).unwrap();
        // Pre-#1370 this failed writing: `meta_variant` would only be
        // discovered as Generic (list), and the type-mismatch reader error
        // from the issue (`column "meta_variant": type mismatch: expected
        // character, got list`) never even applied because the *writer*
        // itself never produced a working frame here in the first place —
        // confirm the default (non-opt-out) reader no longer errors at all.
        let round: Vec<Row> = dataframe_to_vec(df.as_sexp()).unwrap();
        // Row 0 (`variant: None`) collides with the tag-column heuristic and
        // is read back as `meta: None` — the same #1320 lossy default as the
        // `Some`-first ordering, just applied to whichever row has the NA.
        assert_eq!(round[0], Row { id: 2, meta: None });
        // Row 1 (`variant: Some`) is unaffected.
        assert_eq!(round[1], rows[1]);

        // The per-field opt-out still makes both rows round-trip losslessly,
        // regardless of order.
        let round_optout: Vec<Row> =
            dataframe_to_vec_with_struct_fields(df.as_sexp(), &["meta"]).unwrap();
        assert_eq!(round_optout, rows);
    });
}

/// Out of scope (stays as documented): a nested `Option<Struct>` that is
/// `None` in *every* row never sees a `Compound` probe at all — the key
/// resolves to `Scalar(Generic)` and the assembly-time all-None downgrade
/// still converts it to a single logical-NA column, not a per-sub-field
/// atomic column. Recursive union cannot help here: there's nothing to union.
#[test]
fn compound_all_none_option_struct_downgrade_still_applies() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Meta {
        variant: Option<String>,
        size: f64,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Row {
        id: i32,
        meta: Option<Meta>,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![Row { id: 1, meta: None }, Row { id: 2, meta: None }];
        let df = vec_to_dataframe(&rows).unwrap();
        let sexp = df.as_sexp();
        let names = col_names(&sexp);
        // No Compound ever discovered: `meta` stays a single (unflattened)
        // column, not `meta_variant`/`meta_size`.
        assert_eq!(names, vec!["id".to_string(), "meta".to_string()]);
        let meta_col = col(&sexp, "meta");
        assert_eq!(
            meta_col.type_of(),
            miniextendr_api::SEXPTYPE::LGLSXP,
            "all-None Option<Struct> must downgrade to a logical-NA column, per docs"
        );
    });
}

// endregion
