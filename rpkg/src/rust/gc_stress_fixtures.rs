//! Fixtures for GC stress tests.
//!
//! Provides `SharedData` (R6 class) and `into_sexp_altrep` for the GC stress
//! and ALTREP serialization test suites.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use miniextendr_api::SEXPTYPE;
use miniextendr_api::into_r::IntoR;
use miniextendr_api::prelude::{OwnedProtect, SEXP, SexpExt};
use miniextendr_api::{IntoDataFrameSplit, IntoRAltrep, miniextendr};
#[cfg(feature = "jiff")]
use miniextendr_api::{JiffZonedVec, Timestamp};

use crate::dataframe_enum_payload_matrix::{
    BTreeMapEvent, Direction, HashMapEvent, NestedFactorEvent, NestedFlattenEvent, NestedListEvent,
    Point, Status, StructFlattenEvent, StructListEvent,
};

/// Simple R6 class for GC stress tests.
#[derive(miniextendr_api::ExternalPtr)]
pub struct SharedData {
    x: f64,
    y: f64,
    label: String,
}

/// @param x Numeric x-coordinate.
/// @param y Numeric y-coordinate.
/// @param label Character label.
#[miniextendr(r6)]
impl SharedData {
    pub fn new(x: f64, y: f64, label: String) -> Self {
        SharedData { x, y, label }
    }

    pub fn get_x(&self) -> f64 {
        self.x
    }

    pub fn get_y(&self) -> f64 {
        self.y
    }

    pub fn get_label(&self) -> String {
        self.label.clone()
    }
}

/// Build a `Vec<ExternalPtr<T>>` the *naive* way and read every handle back
/// through `R_ExternalPtrAddr`, under GC pressure.
///
/// This is the regression fixture for #836: `ExternalPtr::new` roots its
/// `EXTPTRSXP` via `R_PreserveObject` for the handle's whole Rust lifetime, so
/// the per-element allocation in `.map(ExternalPtr::new).collect()` cannot
/// collect the earlier handles already sitting in the `Vec`. Pre-#836 this
/// reliably corrupted under `gctorture(TRUE)`; the `#827` fixtures had to root
/// each handle manually with a `ProtectScope`.
///
/// The readback re-wraps via `wrap_sexp` (the *honest* check — it inspects
/// `R_ExternalPtrAddr`), not the cached `*mut T` `Deref`, which would read
/// freed memory and silently "pass". The throwaway-batch loop also drives the
/// new `Drop` → `R_ReleaseObject` path while the kept handles are still live.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_externalptr_vec() {
    use miniextendr_api::externalptr::ExternalPtr;

    let n = 24;

    // NAIVE construction — no manual rooting. Each `ExternalPtr::new` allocates
    // (prot VECSXP + EXTPTRSXP); under GC pressure those allocations would
    // collect the earlier, unrooted handles pre-#836.
    let handles: Vec<ExternalPtr<SharedData>> = (0..n)
        .map(|i| {
            ExternalPtr::new(SharedData {
                x: i as f64,
                y: (i * 2) as f64,
                label: format!("bag-{i}"),
            })
        })
        .collect();

    // Churn the GC further while still holding every kept handle: build and
    // immediately drop a second batch. The drop releases each throwaway's root
    // (the new `Drop` path) and must not disturb the handles we keep.
    for i in 0..n {
        let _throwaway = ExternalPtr::new(SharedData {
            x: -1.0,
            y: i as f64,
            label: String::from("throwaway"),
        });
    }

    // Honest readback: a collected handle has a null `R_ExternalPtrAddr`, so
    // `wrap_sexp` returns `None`.
    for (i, h) in handles.iter().enumerate() {
        let reread = unsafe { ExternalPtr::<SharedData>::wrap_sexp(h.as_sexp()) }
            .expect("ExternalPtr handle was collected (R_ExternalPtrAddr is null)");
        assert_eq!(reread.get_x(), i as f64);
        assert_eq!(reread.get_label(), format!("bag-{i}"));
    }
}

/// Build an R `list()` of external pointers via `ExternalPtr::collect_into_r_list`
/// under GC pressure, then read every element back through `R_ExternalPtrAddr`.
///
/// Regression fixture for the destination-rooting bulk builder — the safe, no-pool
/// alternative to a hot `Vec<ExternalPtr>` build. `collect_into_r_list` creates
/// each `EXTPTRSXP` *directly into the protected result list*, so every earlier
/// element stays rooted by the list while later elements allocate. Under
/// `gctorture(TRUE)` each per-element allocation forces a full GC; if the list
/// did not root the elements (a missing `Rf_protect` or `SET_VECTOR_ELT`), the
/// earlier ones would be collected and the readback would see a null
/// `R_ExternalPtrAddr`.
///
/// The readback is allocation-free (`VECTOR_ELT` + `R_ExternalPtrAddr` are
/// pointer reads), so no GC can fire between the build and the reads — the
/// returned (caller-rooted-by-contract) list stays live on the Rust stack
/// without us touching R's protect stack off the main thread.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_externalptr_collect_list() {
    use miniextendr_api::externalptr::ExternalPtr;

    let n: usize = 24;

    // Bulk-build the list with no manual rooting and no pool: each element is
    // allocated straight into the protected list inside `collect_into_r_list`.
    let list = ExternalPtr::<SharedData>::collect_into_r_list((0..n).map(|i| SharedData {
        x: i as f64,
        y: (i * 2) as f64,
        label: format!("item-{i}"),
    }));

    // Honest, allocation-free readback: a collected element has a null
    // `R_ExternalPtrAddr`, so `wrap_sexp` returns `None`.
    assert_eq!(list.len(), n, "collect_into_r_list produced wrong length");
    for i in 0..n {
        let elt = list.vector_elt(i as isize);
        let reread = unsafe { ExternalPtr::<SharedData>::wrap_sexp(elt) }
            .expect("collect_into_r_list element was collected (R_ExternalPtrAddr is null)");
        assert_eq!(reread.get_x(), i as f64);
        assert_eq!(reread.get_label(), format!("item-{i}"));
    }
}

/// Convert a `Vec<SharedData>` to an R `list()` of external pointers through
/// the production `-> Vec<Class>` return path under GC pressure (#1284).
///
/// Drives the `IntoR for Vec<T>` blanket via the `IntoRVecElement` impl that
/// `#[derive(ExternalPtr)]` emits — the exact conversion a
/// `#[miniextendr] fn ... -> Vec<SharedData>` return performs. Internally this
/// funnels into `ExternalPtr::collect_into_r_list`, which allocates each
/// `EXTPTRSXP` directly into the already-protected destination list; under
/// `gctorture(TRUE)` every per-element allocation forces a full GC, so a
/// missing root on the list (or an element stored before rooting) would leave
/// earlier elements collected and the readback would see a null
/// `R_ExternalPtrAddr`.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_vec_class_return() {
    use miniextendr_api::externalptr::ExternalPtr;

    let n: usize = 24;

    let values: Vec<SharedData> = (0..n)
        .map(|i| SharedData {
            x: i as f64,
            y: (i * 2) as f64,
            label: format!("vec-{i}"),
        })
        .collect();

    // The production `-> Vec<Class>` conversion: IntoR for Vec<SharedData>.
    let list = values.into_sexp();

    // Honest, allocation-free readback: a collected element has a null
    // `R_ExternalPtrAddr`, so `wrap_sexp` returns `None`.
    assert_eq!(list.len(), n, "Vec<Class> IntoR produced wrong length");
    for i in 0..n {
        let elt = list.vector_elt(i as isize);
        let reread = unsafe { ExternalPtr::<SharedData>::wrap_sexp(elt) }
            .expect("Vec<Class> list element was collected (R_ExternalPtrAddr is null)");
        assert_eq!(reread.get_x(), i as f64);
        assert_eq!(reread.get_label(), format!("vec-{i}"));
    }
}

/// Exercise `Vec<Option<collection>>` conversions under GC pressure.
///
/// Allocates `Vec<Option<Vec<i32>>>`, `Vec<Option<HashSet<String>>>`, and
/// `Vec<Option<BTreeSet<i32>>>` and converts each to SEXP, verifying that the
/// `OwnedProtect` in `vec_option_of_into_r_to_list` keeps the outer list live
/// across inner `into_sexp()` calls.
#[miniextendr(noexport)]
pub fn gc_stress_vec_option_collection() {
    // Vec<Option<Vec<i32>>>: mix of Some and None
    let vec_opt: Vec<Option<Vec<i32>>> = vec![
        Some(vec![1, 2, 3]),
        None,
        Some(vec![4, 5]),
        None,
        Some(vec![]),
    ];
    let _ = vec_opt.into_sexp();

    // Vec<Option<HashSet<String>>>: some with multiple strings, some None
    let hs_opt: Vec<Option<HashSet<String>>> = vec![
        Some(["a", "b", "c"].iter().map(|s| s.to_string()).collect()),
        None,
        Some(["d"].iter().map(|s| s.to_string()).collect()),
    ];
    let _ = hs_opt.into_sexp();

    // Vec<Option<BTreeSet<i32>>>: sorted elements, some None
    let bt_opt: Vec<Option<BTreeSet<i32>>> = vec![
        Some([3, 1, 2].iter().copied().collect()),
        None,
        Some([5, 4].iter().copied().collect()),
    ];
    let _ = bt_opt.into_sexp();
}

/// Exercise `Vec<Option<&str>>` and `Vec<Option<&[T]>>` conversions under GC pressure.
///
/// Allocates STRSXP + list-column SEXPs with interleaved None/Some values to verify
/// PROTECT discipline across string and slice allocations.
#[miniextendr(noexport)]
pub fn gc_stress_vec_option_borrowed() {
    // Vec<Option<&str>>: STRSXP with NA_character_
    let str_opt: Vec<Option<&str>> = vec![Some("hello"), None, Some("world"), None];
    let _ = str_opt.into_sexp();

    // Vec<Option<&[f64]>>: list-column, NULL for None
    let a: &[f64] = &[1.0, 2.0, 3.0];
    let b: &[f64] = &[4.0];
    let slice_opt: Vec<Option<&[f64]>> = vec![Some(a), None, Some(b), None];
    let _ = slice_opt.into_sexp();

    // Vec<Option<&[String]>>: list-column (character vector per row)
    let sa: Vec<String> = vec!["x".to_string(), "y".to_string()];
    let sb: Vec<String> = vec!["z".to_string()];
    let str_slice_opt: Vec<Option<&[String]>> =
        vec![Some(sa.as_slice()), None, Some(sb.as_slice())];
    let _ = str_slice_opt.into_sexp();
}

/// Exercise map-field `Vec<Vec<K>>` / `Vec<Vec<V>>` column codegen under GC pressure.
///
/// Synthesizes realistic `HashMapEvent` and `BTreeMapEvent` rows and drives both
/// `to_dataframe` (align) and `into_dataframe_split` (per-variant partition) paths.
/// Verifies that the `ProtectScope::protect_raw` calls in the generated map-column
/// code keep each inner `Vec<K>` / `Vec<V>` SEXP live across the subsequent
/// `into_sexp()` call for the parallel column.
#[miniextendr(noexport)]
pub fn gc_stress_dataframe_map() {
    // HashMap align path — multiple variants, multiple rows, includes empty map.
    let hm_rows = vec![
        HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::from([
                ("x".to_string(), 1i32),
                ("y".to_string(), 2i32),
                ("z".to_string(), 3i32),
            ]),
        },
        HashMapEvent::Empty { label: "b".into() },
        HashMapEvent::Tally {
            label: "c".into(),
            tally: HashMap::new(), // empty map → Some(vec![]) in both columns
        },
        HashMapEvent::Tally {
            label: "d".into(),
            tally: HashMap::from([("p".to_string(), 10i32)]),
        },
        HashMapEvent::Empty { label: "e".into() },
    ];
    let _ = HashMapEvent::to_dataframe(hm_rows.clone());
    let _ = hm_rows.into_dataframe_split();

    // BTreeMap align path — same shape, sorted key order exercised.
    let bt_rows = vec![
        BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::from([
                ("z".to_string(), 3i32),
                ("a".to_string(), 1i32),
                ("m".to_string(), 2i32),
            ]),
        },
        BTreeMapEvent::Empty { label: "b".into() },
        BTreeMapEvent::Tally {
            label: "c".into(),
            tally: BTreeMap::new(), // empty map
        },
        BTreeMapEvent::Tally {
            label: "d".into(),
            tally: BTreeMap::from([("q".to_string(), 99i32)]),
        },
        BTreeMapEvent::Empty { label: "e".into() },
    ];
    let _ = BTreeMapEvent::to_dataframe(bt_rows.clone());
    let _ = bt_rows.into_dataframe_split();
}

/// Exercise struct-field DataFrameRow flatten + as_list paths under GC pressure.
///
/// Allocates `StructFlattenEvent` and `StructListEvent` rows, calls both
/// `to_dataframe` (align) and `into_dataframe_split`, and converts to SEXP,
/// verifying that `ProtectScope` keeps inner column SEXPs live across scatter
/// allocations.  No arguments required — suitable for the fast gctorture sweep.
#[miniextendr(noexport)]
pub fn gc_stress_dataframe_struct() {
    use miniextendr_api::into_r::IntoR as _;

    // Flatten path: struct-typed field expands to prefixed columns.
    let flatten_rows = vec![
        StructFlattenEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructFlattenEvent::Other { id: 2 },
        StructFlattenEvent::Located {
            id: 3,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructFlattenEvent::Other { id: 4 },
    ];
    let _ = StructFlattenEvent::to_dataframe(flatten_rows.clone()).into_sexp();
    let _ = flatten_rows.into_dataframe_split().into_sexp();

    // as_list path: struct-typed field kept as opaque list-column.
    let list_rows = vec![
        StructListEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructListEvent::Other { id: 2 },
        StructListEvent::Located {
            id: 3,
            origin: Point { x: 5.0, y: 6.0 },
        },
        StructListEvent::Other { id: 4 },
    ];
    let _ = StructListEvent::to_dataframe(list_rows.clone()).into_sexp();
    let _ = list_rows.into_dataframe_split().into_sexp();
}

/// Exercise nested-enum field DataFrameRow flatten + as_factor + as_list paths
/// under GC pressure.
///
/// Drives both `to_dataframe` (align) and `into_dataframe_split` paths for all
/// three nested-enum field modes. No arguments — suitable for the fast gctorture
/// sweep.
#[miniextendr(noexport)]
pub fn gc_stress_dataframe_nested_enum() {
    use miniextendr_api::into_r::IntoR as _;

    // Flatten path: nested payload-bearing DataFrameRow enum expands to prefixed columns.
    // Uses Status (Ok / Err { code }) so status_variant + status_code columns are exercised.
    let flatten_rows = vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Other { id: 2 },
        NestedFlattenEvent::Tracked {
            id: 3,
            status: Status::Err { code: 404 },
        },
        NestedFlattenEvent::Other { id: 4 },
    ];
    let _ = NestedFlattenEvent::to_dataframe(flatten_rows.clone()).into_sexp();
    let _ = flatten_rows.into_dataframe_split().into_sexp();

    // as_factor path: unit-only enum field stored as factor column.
    let factor_rows = vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedFactorEvent::Stop { id: 2 },
        NestedFactorEvent::Move {
            id: 3,
            dir: Direction::East,
        },
        NestedFactorEvent::Stop { id: 4 },
    ];
    let _ = NestedFactorEvent::to_dataframe(factor_rows.clone()).into_sexp();
    let _ = factor_rows.into_dataframe_split().into_sexp();

    // as_list path: enum field kept as opaque list-column.
    let list_rows = vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::South,
        },
        NestedListEvent::Stop { id: 2 },
        NestedListEvent::Move {
            id: 3,
            dir: Direction::West,
        },
        NestedListEvent::Stop { id: 4 },
    ];
    let _ = NestedListEvent::to_dataframe(list_rows.clone()).into_sexp();
    let _ = list_rows.into_dataframe_split().into_sexp();

    // Status is already exercised via NestedFlattenEvent above.
}

/// Exercise the native-SEXP ALTREP (`NativeSexpIntAltrep`) under GC pressure.
///
/// Constructs an ALTREP-backed integer vector where `data1` is a plain
/// `INTSXP` (no ExternalPtr).  Exercises both element access (via `Elt`) and
/// the dataptr path (`as.integer`) so that any PROTECT-discipline bug around
/// the `data1` allocation would be caught by `gctorture(TRUE)`.
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[miniextendr(noexport)]
pub fn gc_stress_native_sexp_altrep() {
    use crate::native_sexp_altrep_fixture::native_sexp_altrep_new;
    use miniextendr_api::prelude::SexpExt as _;

    // Construct a small ALTREP-backed integer vector.
    let values = vec![10i32, 20, 30, 40, 50];
    let sexp = native_sexp_altrep_new(values);

    // Verify that it is indeed ALTREP.
    assert!(
        sexp.is_altrep(),
        "native_sexp_altrep_new should return an ALTREP SEXP"
    );

    // Force element access (exercises the Elt path) via the SexpExt trait.
    let n = sexp.len();
    assert_eq!(n, 5);
    for i in 0..n {
        let v = sexp.integer_elt(i as isize);
        assert_eq!(v, (i as i32 + 1) * 10);
    }

    // Force the Dataptr path (exercises `AltVec::dataptr`).
    let _ptr = unsafe { miniextendr_api::sys::DATAPTR_RO(sexp) };
}

/// Exercise `JiffZonedVec` ALTREP construction and element access under GC pressure.
///
/// Constructs a single-timezone `JiffZonedVec` with several `America/New_York`
/// timestamps, converts it to an SEXP via `into_sexp`, and forces element access
/// to verify that both the `into_sexp_altrep` allocation and the
/// `set_posixct_tz` class/timezone attributes survive intact.
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[cfg(feature = "jiff")]
#[miniextendr(noexport)]
pub fn gc_stress_jiff_zoned_vec() {
    use miniextendr_api::jiff::tz::TimeZone;

    let tz = TimeZone::get("America/New_York").expect("America/New_York must exist");
    let timestamps = [
        Timestamp::new(1_735_689_600, 0).expect("valid timestamp"), // 2025-01-01 UTC
        Timestamp::new(1_750_000_000, 0).expect("valid timestamp"),
        Timestamp::new(1_760_000_000, 0).expect("valid timestamp"),
    ];
    let data: Vec<miniextendr_api::Zoned> = timestamps
        .iter()
        .map(|ts| ts.to_zoned(tz.clone()))
        .collect::<Vec<_>>();
    let vec = JiffZonedVec::new(data).expect("single-tz JiffZonedVec construction");

    // Convert to SEXP (exercises ALTREP allocation + set_posixct_tz PROTECT path).
    let sexp = vec.into_posixct_sexp();

    // Assert the metadata created by set_posixct_tz. Merely reading ALTREP
    // elements would not detect a collected or reused tzone CHARSXP.
    use miniextendr_api::prelude::SexpExt as _;
    let tzone_symbol = unsafe { miniextendr_api::sys::Rf_install(c"tzone".as_ptr()) };
    let tzone = sexp.get_attr(tzone_symbol);
    assert_eq!(tzone.xlength(), 1);
    assert_eq!(tzone.string_elt(0).r_char_str(), Some("America/New_York"));

    // Force element access via the ALTREP Elt path.
    let n = sexp.len();
    assert_eq!(n, 3);
    for i in 0..n {
        let _v = sexp.real_elt(i as isize);
    }
}

/// Exercise `array_to_sexp` PROTECT discipline for integer, float, and boolean
/// TOML arrays under GC pressure.
///
/// Constructs `TomlValue::Array` values covering the four homogeneous branches
/// of `array_to_sexp` and converts each to SEXP via `IntoR`, verifying that
/// the `OwnedProtect` guards keep each allocation live across the fill loop.
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[cfg(feature = "toml")]
#[miniextendr(noexport)]
pub fn gc_stress_toml_array() {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::toml_impl::TomlValue;

    // Integer branch (all fit in i32) → INTSXP
    let int_arr = TomlValue::Array(vec![
        TomlValue::Integer(1),
        TomlValue::Integer(2),
        TomlValue::Integer(i64::from(i32::MAX)),
    ]);
    let _ = int_arr.into_sexp();

    // Integer branch (one value exceeds i32 range) → REALSXP fallback
    let big_int_arr = TomlValue::Array(vec![
        TomlValue::Integer(1),
        TomlValue::Integer(i64::from(i32::MAX) + 1),
        TomlValue::Integer(3),
    ]);
    let _ = big_int_arr.into_sexp();

    // Float branch → REALSXP
    let float_arr = TomlValue::Array(vec![
        TomlValue::Float(1.1),
        TomlValue::Float(2.2),
        TomlValue::Float(3.3),
    ]);
    let _ = float_arr.into_sexp();

    // Boolean branch → LGLSXP
    let bool_arr = TomlValue::Array(vec![
        TomlValue::Boolean(true),
        TomlValue::Boolean(false),
        TomlValue::Boolean(true),
    ]);
    let _ = bool_arr.into_sexp();
}

/// Exercise `NamedDataFrameListBuilder` SEXP protection under GC pressure.
///
/// Pushes two `DataFrame`s (synthesised internally via `vec_to_dataframe`) into
/// a `NamedDataFrameListBuilder` and calls `build()`.  Each `push` stores the
/// input SEXP in an internal `Vec<SEXP>` (protected by the builder's
/// `ProtectScope`), making this the canonical gctorture coverage for the
/// SEXP-storage-across-allocations path in the builder.
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_named_df_list_builder() -> SEXP {
    use miniextendr_api::NamedDataFrameListBuilder;
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::vec_to_dataframe;

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct OkRow {
        id: i32,
        value: f64,
    }

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct ErrRow {
        id: i32,
        msg: String,
    }

    let oks: Vec<OkRow> = (0..50)
        .map(|i| OkRow {
            id: i,
            value: f64::from(i) * 1.5,
        })
        .collect();
    let errs: Vec<ErrRow> = (0..20)
        .map(|i| ErrRow {
            id: i,
            msg: format!("err {i}"),
        })
        .collect();

    // `vec_to_dataframe` now returns an owned `BuiltDataFrame`; deref to the view
    // for `push` (each temporary handle lives to the end of the `.push(...)`
    // statement, so its frame stays rooted until `push` re-roots it internally).
    NamedDataFrameListBuilder::new()
        .push("results", *vec_to_dataframe(&oks).unwrap())
        .push("error", *vec_to_dataframe(&errs).unwrap())
        .build()
        .into_sexp()
}

/// Exercise `DataFrame::group_by` + `GroupedDataFrame::frames` SEXP handling
/// under GC pressure.
///
/// `GroupedDataFrame` holds the source frame's SEXP across the per-group
/// `select_rows` allocations, and each yielded sub-frame is a rooted
/// `BuiltDataFrame` (#1247) whose root must hold until the
/// `NamedDataFrameListBuilder` push re-roots the view — the two
/// SEXP-across-allocations paths this fixture drives. Synthesizes a 60-row frame with a
/// character key (incl. NAs) internally.
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[miniextendr(noexport)]
pub fn gc_stress_group_by() -> SEXP {
    use miniextendr_api::dataframe::{DataFrame, NamedDataFrameListBuilder};

    let df = DataFrame::builder(60)
        .column::<i32>("v", |chunk, offset| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = i32::try_from(offset + i).expect("row index exceeds i32");
            }
        })
        .column_str("g", |i| match i % 4 {
            0 => Some("a".to_string()),
            1 => Some("b".to_string()),
            2 => Some("c".to_string()),
            _ => None,
        })
        .build();

    let grouped = df
        .group_by("g")
        .expect("gc_stress_group_by: group_by failed");
    let mut out = NamedDataFrameListBuilder::with_capacity(grouped.len());
    for (key, sub) in grouped.frames() {
        // `sub` is a rooted `BuiltDataFrame` (#1247); deref to the view for
        // push, which protects it in the builder's scope before `sub` drops.
        out = out.push(key.label(), *sub);
    }
    out.build().into_sexp()
}

/// Exercise `DataFrame::group_by_multi` (composite keys) + `GroupedDataFrame::frames`
/// SEXP handling under GC pressure.
///
/// Same SEXP-across-allocations paths as [`gc_stress_group_by`] — the source
/// frame held across per-group `select_rows`, and each rooted `BuiltDataFrame`
/// held until the builder push re-roots it — but keyed by a `GroupKey::Tuple`
/// over two columns (character with NAs × integer), so it drives the
/// composite-key bucketing/ordering path too. Synthesizes a 60-row frame internally.
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[miniextendr(noexport)]
pub fn gc_stress_group_by_multi() -> SEXP {
    use miniextendr_api::dataframe::{DataFrame, NamedDataFrameListBuilder};

    let df = DataFrame::builder(60)
        .column::<i32>("v", |chunk, offset| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = i32::try_from(offset + i).expect("row index exceeds i32");
            }
        })
        .column_str("g", |i| match i % 4 {
            0 => Some("a".to_string()),
            1 => Some("b".to_string()),
            2 => Some("c".to_string()),
            _ => None,
        })
        .column::<i32>("k", |chunk, offset| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = i32::try_from((offset + i) % 3).expect("key fits i32");
            }
        })
        .build();

    let grouped = df
        .group_by_multi(&["g", "k"])
        .expect("gc_stress_group_by_multi: group_by_multi failed");
    let mut out = NamedDataFrameListBuilder::with_capacity(grouped.len());
    for (key, sub) in grouped.frames() {
        // `sub` is a rooted `BuiltDataFrame` (#1247); deref to the view for
        // push, which protects it in the builder's scope before `sub` drops.
        out = out.push(key.label(), *sub);
    }
    out.build().into_sexp()
}

/// Convert an R vector to an ALTREP-backed vector by materializing then re-wrapping.
/// Dispatches on `type_of()`: INTSXP, REALSXP, STRSXP.
/// @param x An integer, numeric, or character vector to convert.
#[miniextendr]
pub fn into_sexp_altrep(x: SEXP) -> SEXP {
    let sxp_type = x.type_of();
    match sxp_type {
        SEXPTYPE::INTSXP => {
            let v: Vec<i32> = miniextendr_api::from_r::TryFromSexp::try_from_sexp(x).unwrap();
            v.into_sexp_altrep()
        }
        SEXPTYPE::REALSXP => {
            let v: Vec<f64> = miniextendr_api::from_r::TryFromSexp::try_from_sexp(x).unwrap();
            v.into_sexp_altrep()
        }
        SEXPTYPE::STRSXP => {
            // Use Vec<Option<String>> to preserve NA_character_ values
            let v: Vec<Option<String>> =
                miniextendr_api::from_r::TryFromSexp::try_from_sexp(x).unwrap();
            v.into_sexp_altrep()
        }
        _ => panic!("into_sexp_altrep: unsupported SEXP type {:?}", sxp_type),
    }
}

// region: dataframe_to_vec / with_dataframe_rows GC stress

/// Exercise `dataframe_to_vec` PROTECT discipline under GC pressure.
///
/// Synthesizes 10 rows via `vec_to_dataframe`, then runs `dataframe_to_vec`
/// to reconstruct the original rows. Returns the row count to verify execution.
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_dataframe_to_vec() -> i32 {
    use crate::serde::{Deserialize, Serialize};
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{dataframe_to_vec, vec_to_dataframe};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "crate::serde")]
    struct GcRow {
        id: i32,
        score: f64,
        label: Option<String>,
    }

    let original: Vec<GcRow> = (0i32..10)
        .map(|i| GcRow {
            id: i,
            score: f64::from(i) * 1.1,
            label: if i % 2 == 0 {
                Some(format!("item_{}", i))
            } else {
                None
            },
        })
        .collect();

    let sexp = vec_to_dataframe(&original)
        .expect("gc_stress_dataframe_to_vec: vec_to_dataframe failed")
        .into_sexp();

    let back: Vec<GcRow> =
        dataframe_to_vec(sexp).expect("gc_stress_dataframe_to_vec: dataframe_to_vec failed");

    assert_eq!(back.len(), original.len());
    for (a, b) in original.iter().zip(back.iter()) {
        assert_eq!(a, b);
    }

    back.len() as i32
}

/// Exercise `with_dataframe_rows` PROTECT discipline under GC pressure.
///
/// Synthesizes 10 rows, converts to a data.frame, then uses `with_dataframe_rows`
/// to compute a sum via the scoped callback. Returns the sum to verify execution.
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_with_dataframe_rows() -> f64 {
    use crate::serde::{Deserialize, Serialize};
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{vec_to_dataframe, with_dataframe_rows};

    #[derive(Serialize, Deserialize)]
    #[serde(crate = "crate::serde")]
    struct GcRow {
        value: f64,
        tag: String,
    }

    let source: Vec<GcRow> = (0i32..10)
        .map(|i| GcRow {
            value: f64::from(i) * 2.0,
            tag: format!("t{}", i),
        })
        .collect();

    let sexp = vec_to_dataframe(&source)
        .expect("gc_stress_with_dataframe_rows: vec_to_dataframe failed")
        .into_sexp();

    with_dataframe_rows(sexp, |rows: &[GcRow]| {
        // Compute a sum and verify tags are valid.
        let sum: f64 = rows.iter().map(|r| r.value).sum();
        for r in rows {
            assert!(
                r.tag.starts_with('t'),
                "tag should start with 't': {}",
                r.tag
            );
        }
        sum
    })
    .expect("gc_stress_with_dataframe_rows: with_dataframe_rows failed")
}

/// Exercise `dataframe_to_vec` nested-struct un-flattening under GC pressure.
///
/// Synthesizes 10 rows of a nested `Person { name, address: Address { city, zip } }`
/// struct via `vec_to_dataframe`, then reconstructs them via `dataframe_to_vec`
/// using the single-underscore prefix-matching path. Returns the row count.
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_dataframe_to_vec_nested() -> i32 {
    use crate::serde::{Deserialize, Serialize};
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{dataframe_to_vec, vec_to_dataframe};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "crate::serde")]
    struct Address {
        city: String,
        zip: String,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "crate::serde")]
    struct Person {
        name: String,
        address: Address,
    }

    let original: Vec<Person> = (0i32..10)
        .map(|i| Person {
            name: format!("person_{}", i),
            address: Address {
                city: format!("city_{}", i),
                zip: format!("{:05}", i * 1000),
            },
        })
        .collect();

    let sexp = vec_to_dataframe(&original)
        .expect("gc_stress_dataframe_to_vec_nested: vec_to_dataframe failed")
        .into_sexp();

    let back: Vec<Person> =
        dataframe_to_vec(sexp).expect("gc_stress_dataframe_to_vec_nested: dataframe_to_vec failed");

    assert_eq!(back.len(), original.len());
    for (a, b) in original.iter().zip(back.iter()) {
        assert_eq!(a, b);
    }

    back.len() as i32
}

// endregion

// region: Streaming serialize GC stress

/// Exercise `iter_to_dataframe` PROTECT discipline under GC pressure.
///
/// Synthesizes 50 rows via an iterator and runs them through
/// `iter_to_dataframe`, verifying that the per-builder `ProtectScope` keeps
/// every protected SEXP live across the fill loop and `assemble_dataframe`.
///
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_iter_to_dataframe() -> SEXP {
    use crate::serde::Serialize;
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::iter_to_dataframe;

    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct StreamRow {
        id: i32,
        value: f64,
        label: String,
        flag: Option<i32>,
    }

    let df = iter_to_dataframe(
        (0i32..50).map(|i| StreamRow {
            id: i,
            value: f64::from(i) * 1.5,
            label: format!("row_{i}"),
            flag: if i % 3 == 0 { None } else { Some(i * 2) },
        }),
        None,
    )
    .expect("gc_stress_iter_to_dataframe: iter_to_dataframe failed");
    df.into_sexp()
}

/// Exercise `dispatch_to_dataframes` PROTECT discipline under GC pressure.
///
/// Synthesises a mixed Result-iterator (every third row is an Err with a
/// distinct payload shape) and runs it through the streaming two-builder
/// path. Each builder's `ProtectScope` plus the assembling
/// `NamedDataFrameListBuilder` need to keep every protected SEXP live.
///
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_dispatch_to_dataframes() -> SEXP {
    use crate::serde::Serialize;
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{DispatchNames, dispatch_to_dataframes};

    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct OkRow {
        id: i32,
        val: f64,
    }
    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct ErrRow {
        id: i32,
        reason: String,
    }

    let list = dispatch_to_dataframes(
        (0i32..30).map(|i| {
            if i % 3 == 0 {
                Err(ErrRow {
                    id: i,
                    reason: format!("skip_{i}"),
                })
            } else {
                Ok(OkRow {
                    id: i,
                    val: f64::from(i) * 0.5,
                })
            }
        }),
        Some(30),
        DispatchNames::default(),
    )
    .expect("gc_stress_dispatch_to_dataframes: dispatch_to_dataframes failed");
    list.into_sexp()
}

// endregion

// region: SerdeRowBuilder with_schema / grow_schema GC stress

/// Exercise [`SerdeRowBuilder::with_schema`] PROTECT discipline under GC
/// pressure. Builds a pre-declared schema, pushes 50 rows including a
/// `None`-bearing optional column, then assembles via `finish()`. Verifies
/// the per-builder `ProtectScope` keeps every protected SEXP live across
/// the fill loop and final assembly.
///
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_builder_with_schema() -> SEXP {
    use crate::serde::Serialize;
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{SerdeRowBuilder, TypeSpec};

    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct Row {
        id: i32,
        ratio: f64,
        tag: Option<String>,
    }

    let mut b = SerdeRowBuilder::<Row>::with_schema(
        [
            ("id", TypeSpec::Integer),
            ("ratio", TypeSpec::Real),
            // Optional(Character) keeps the column character-typed even if
            // the first row's `tag` is None.
            ("tag", TypeSpec::Optional(Box::new(TypeSpec::Character))),
        ],
        Some(50),
    );
    for i in 0..50i32 {
        b.push(Row {
            id: i,
            ratio: f64::from(i) * 0.25,
            tag: if i % 3 == 0 {
                None
            } else {
                Some(format!("tag_{i}"))
            },
        })
        .expect("gc_stress_builder_with_schema: push failed");
    }
    let df = b
        .finish()
        .expect("gc_stress_builder_with_schema: finish failed");
    df.into_sexp()
}

/// Exercise [`SerdeRowBuilder::grow_schema`] PROTECT discipline under GC
/// pressure. Pushes 30 heterogeneous `BTreeMap` rows that progressively
/// introduce new keys, forcing the back-fill path to allocate columns and
/// NA-fill prior rows. Verifies the grown columns stay rooted and
/// length-aligned through `finish()`.
///
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_builder_grow_schema() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::SerdeRowBuilder;
    use std::collections::BTreeMap;

    let mut b = SerdeRowBuilder::<BTreeMap<String, i32>>::new(None).grow_schema();
    for i in 0..30i32 {
        let mut row: BTreeMap<String, i32> = BTreeMap::new();
        row.insert("base".into(), i);
        // Introduce a fresh key every 5 rows to drive the back-fill path.
        if i % 5 == 0 {
            row.insert(format!("k_{i}"), i * 10);
        }
        // A common second column that appears from row 1 onward.
        if i >= 1 {
            row.insert("common".into(), i + 100);
        }
        b.push(row)
            .expect("gc_stress_builder_grow_schema: push failed");
    }
    let df = b
        .finish()
        .expect("gc_stress_builder_grow_schema: finish failed");
    df.into_sexp()
}

/// Exercise [`BuiltDataFrame`](miniextendr_api::dataframe::BuiltDataFrame)'s GC
/// root (issue #1128): build a frame via [`SerdeRowBuilder::finish`], force a
/// burst of R allocations **while holding the handle**, then read every column
/// back and assert the values survived.
///
/// This is the regression harness the issue names. On pre-#1128 code (`finish`
/// returned a bare, unrooted `DataFrame`) the held frame is reclaimed by the GC
/// during the allocation burst under `gctorture(TRUE)`, and the column reads
/// then panic ("DataFrame always carries a names attribute") or return corrupt
/// values. The owned `BuiltDataFrame` handle roots the frame with
/// `R_PreserveObject`, so holding it across allocations is safe and this stays
/// green. (Verified by temporarily dereferencing-then-dropping the handle
/// (`let df = *b.finish()…`), which reproduces the old bare-view crash.)
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_built_dataframe() {
    use crate::serde::Serialize;
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::SerdeRowBuilder;

    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct Row {
        id: i32,
        ratio: f64,
        label: String,
    }

    const N: i32 = 40;
    let mut b = SerdeRowBuilder::<Row>::new(Some(N as usize));
    for i in 0..N {
        b.push(Row {
            id: i,
            ratio: f64::from(i) * 0.5,
            label: format!("row_{i}"),
        })
        .expect("gc_stress_built_dataframe: push failed");
    }
    // The returned handle owns a GC root (`R_PreserveObject`) for the frame.
    let built = b
        .finish()
        .expect("gc_stress_built_dataframe: finish failed");

    // Force a burst of R allocations while `built` is held. Under gctorture(TRUE)
    // every allocation is a full GC; a bare (unrooted) frame would be reclaimed
    // here — and, once its nodes are reused by these throwaway vectors, the reads
    // below would fault or read garbage. Each churn SEXP is dropped immediately,
    // so only `built`'s root keeps its frame alive.
    for i in 0..64i32 {
        let churn: Vec<f64> = (0..128).map(|j| f64::from(i + j)).collect();
        let _ = churn.into_sexp();
        let more: Vec<Option<String>> = (0..16).map(|j| Some(format!("g{i}_{j}"))).collect();
        let _ = more.into_sexp();
    }

    // Read every column back through the (still-rooted) handle and assert the
    // frame is intact. `column`/`names` route through `named_list()`, whose
    // `.expect("DataFrame always carries a names attribute")` fires if the names
    // attribute was reclaimed.
    assert_eq!(built.nrow(), N as usize, "row count changed under GC");
    assert_eq!(built.ncol(), 3, "column count changed under GC");
    assert_eq!(
        built.names(),
        vec!["id", "ratio", "label"],
        "names reclaimed under GC"
    );
    // Read the raw column SEXPs back through the (still-rooted) handle and check
    // they survived intact — length, type, and first/last values. `column_raw`
    // routes through `named_list()`, whose `.expect` fires if the names
    // attribute was reclaimed.
    let id_col = built
        .column_raw("id")
        .expect("id column reclaimed under GC");
    assert_eq!(
        id_col.type_of(),
        SEXPTYPE::INTSXP,
        "id column type corrupted"
    );
    let ids: &[i32] = unsafe { id_col.as_slice() };
    assert_eq!(ids.len(), N as usize, "id column length changed under GC");
    assert_eq!(ids[0], 0);
    assert_eq!(ids[(N - 1) as usize], N - 1, "id column values corrupted");
    let label_col = built
        .column_raw("label")
        .expect("label column reclaimed under GC");
    assert_eq!(
        label_col.type_of(),
        SEXPTYPE::STRSXP,
        "label column type corrupted"
    );
    assert_eq!(label_col.xlength() as usize, N as usize);
    assert_eq!(
        label_col.string_elt_str(0),
        Some("row_0"),
        "label[0] corrupted under GC"
    );
}

/// Exercise the #1247 constructor → edit chains: every new-frame editing
/// method (`drop` / `select` / `select_rows` / `prepend_column` /
/// `with_column`) returns an owned, GC-rooted
/// [`BuiltDataFrame`](miniextendr_api::dataframe::BuiltDataFrame), and the
/// `BuiltDataFrame` inherent forwards keep chains rooted at every link.
///
/// This is the exact UAF scenario the issue names: pre-#1247,
/// `create(...).drop("y")` dropped the constructor's temporary handle at the
/// end of the statement (releasing the ORIGINAL frame's root) while the NEW
/// frame from `.drop()` was never rooted — the allocation burst below would
/// reap it under `gctorture(TRUE)`, and the reads would panic ("DataFrame
/// always carries a names attribute") or return corrupt values.
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[miniextendr(noexport)]
pub fn gc_stress_dataframe_edit_chain() {
    use miniextendr_api::dataframe::{BuiltDataFrame, DataFrame};

    const N: usize = 24;

    // Constructor: a rooted BuiltDataFrame with columns x (double),
    // y (double), label (character).
    let build = || -> BuiltDataFrame {
        DataFrame::builder(N)
            .column::<f64>("x", |chunk, off| {
                for (i, v) in chunk.iter_mut().enumerate() {
                    *v = (off + i) as f64;
                }
            })
            .column::<f64>("y", |chunk, off| {
                for (i, v) in chunk.iter_mut().enumerate() {
                    *v = (off + i) as f64 * 10.0;
                }
            })
            .column_str("label", |i| Some(format!("row{i}")))
            .build()
    };

    // Force a burst of R allocations. Under gctorture(TRUE) every allocation
    // is a full GC — any unrooted frame produced above is reclaimed here.
    let churn = || {
        for i in 0..32i32 {
            let v: Vec<f64> = (0..64).map(|j| f64::from(i + j)).collect();
            let _ = v.into_sexp();
            let s: Vec<Option<String>> = (0..8).map(|j| Some(format!("c{i}_{j}"))).collect();
            let _ = s.into_sexp();
        }
    };

    // drop: constructor → edit, allocate, then read.
    let dropped = build().drop("y");
    churn();
    assert_eq!(
        dropped.names(),
        vec!["x", "label"],
        "drop: names corrupted under GC"
    );
    assert_eq!(dropped.nrow(), N, "drop: nrow corrupted under GC");

    // select: reorder + subset.
    let selected = build().select(&["label", "x"]);
    churn();
    assert_eq!(
        selected.names(),
        vec!["label", "x"],
        "select: names corrupted under GC"
    );

    // select_rows (&self forward): keep the even rows.
    let idx: Vec<usize> = (0..N).step_by(2).collect();
    let dense = build().select_rows(&idx);
    churn();
    assert_eq!(
        dense.nrow(),
        idx.len(),
        "select_rows: nrow corrupted under GC"
    );
    let xs: Vec<f64> = dense.column("x").expect("select_rows: x column reclaimed");
    assert_eq!(xs[1], 2.0, "select_rows: values corrupted under GC");

    // with_column (append) → rename (in-place) → drop, one chain: every
    // intermediate link is a rooted handle.
    // SAFETY: R main thread (this fn builds SEXPs throughout). `with_column`'s
    // caller contract: the column SEXP must stay reachable across the call.
    let newcol = unsafe {
        OwnedProtect::new(
            (0..N)
                .map(|i| i32::try_from(i * 2).expect("index exceeds i32"))
                .collect::<Vec<i32>>()
                .into_sexp(),
        )
    };
    let chained = build()
        .with_column("doubled", *newcol)
        .rename("label", "tag")
        .drop("y");
    drop(newcol);
    churn();
    assert_eq!(
        chained.names(),
        vec!["x", "tag", "doubled"],
        "with_column/rename/drop chain: names corrupted under GC"
    );
    let doubled: Vec<i32> = chained
        .column("doubled")
        .expect("chain: doubled column reclaimed");
    assert_eq!(doubled[3], 6, "chain: appended values corrupted under GC");

    // prepend_column, then strip_prefix (in-place forward).
    // SAFETY: as above — root the column across the call.
    let idcol = unsafe {
        OwnedProtect::new(
            (0..N)
                .map(|i| i32::try_from(i).expect("index exceeds i32"))
                .collect::<Vec<i32>>()
                .into_sexp(),
        )
    };
    let prepended = chained
        .prepend_column("pfx_id", *idcol)
        .strip_prefix("pfx_");
    drop(idcol);
    churn();
    assert_eq!(
        prepended.names(),
        vec!["id", "x", "tag", "doubled"],
        "prepend_column/strip_prefix: names corrupted under GC"
    );

    // with_column (in-place replace path): re-roots the same frame.
    // SAFETY: as above — root the column across the call.
    let replcol = unsafe {
        OwnedProtect::new(
            (0..N)
                .map(|i| -(f64::from(u32::try_from(i).expect("index exceeds u32"))))
                .collect::<Vec<f64>>()
                .into_sexp(),
        )
    };
    let replaced = prepended.with_column("x", *replcol);
    drop(replcol);
    churn();
    let xs: Vec<f64> = replaced.column("x").expect("replace: x column reclaimed");
    assert_eq!(xs[2], -2.0, "replace: values corrupted under GC");
    assert_eq!(replaced.nrow(), N, "replace: nrow corrupted under GC");
}

// endregion

// region: BorrowedRows GC stress

/// Exercise `dataframe_to_vec_borrowed` PROTECT discipline under GC pressure.
///
/// Synthesises 10 rows via `vec_to_dataframe`, then deserialises through
/// `dataframe_to_vec_borrowed` and reads back via the `BorrowedRows<'_, T>`
/// handle. Verifies the internal `OwnedProtect` keeps the source SEXP rooted
/// while we read each row.
///
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_borrowed_rows() -> i32 {
    use crate::serde::{Deserialize, Serialize};
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{BorrowedRows, dataframe_to_vec_borrowed, vec_to_dataframe};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "crate::serde")]
    struct Row {
        id: i32,
        score: f64,
        label: Option<String>,
    }

    let original: Vec<Row> = (0i32..10)
        .map(|i| Row {
            id: i,
            score: f64::from(i) * 1.1,
            label: if i % 2 == 0 {
                Some(format!("item_{i}"))
            } else {
                None
            },
        })
        .collect();

    let sexp = vec_to_dataframe(&original)
        .expect("gc_stress_borrowed_rows: vec_to_dataframe failed")
        .into_sexp();

    let bundle: BorrowedRows<'_, Row> =
        dataframe_to_vec_borrowed(sexp).expect("gc_stress_borrowed_rows: deserialise failed");

    assert_eq!(bundle.len(), original.len());
    for (a, b) in original.iter().zip(bundle.iter()) {
        assert_eq!(a, b);
    }

    bundle.len() as i32
}

// endregion

// region: STRSXP-building PROTECT discipline GC stress

/// Exercise the PROTECT discipline of the migrated STRSXP-building / factor
/// paths under GC pressure.
///
/// Drives:
/// - `match_arg::choices_sexp` (via a derived `MatchArg` enum's `IntoR for Vec<T>`),
/// - `match_arg::match_arg_vec_into_sexp` (same path),
/// - `factor::build_factor_with_levels` (via `FactorVec` / `FactorOptionVec`
///   blanket impls — bare `RFactor` enum, no caching),
/// - `named_vector::set_names_on_sexp` (via `NamedVector<HashMap<String, i32>>`).
///
/// Each path allocates a fresh STRSXP and reads/writes elements across
/// allocations that can fire GC — exactly the shape that broke in PR #344's
/// `make_rust_condition_value`. No arguments — suitable for the fast
/// gctorture no-arg fixture sweep.
#[miniextendr(noexport)]
pub fn gc_stress_protect_discipline() {
    use miniextendr_api::NamedVector;
    use miniextendr_api::factor::{FactorOptionVec, FactorVec};
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::match_arg::MatchArg;

    // FactorVec<Color> → build_factor_with_levels exercised on a no-cache path
    // (RFactor blanket through FactorVec).
    let fv: FactorVec<crate::factor_tests::Color> = FactorVec(vec![
        crate::factor_tests::Color::Red,
        crate::factor_tests::Color::Green,
        crate::factor_tests::Color::Blue,
        crate::factor_tests::Color::Red,
    ]);
    let _ = fv.into_sexp();

    // FactorOptionVec<Color> with NA → same path, different indices builder.
    let fov: FactorOptionVec<crate::factor_tests::Color> = FactorOptionVec(vec![
        Some(crate::factor_tests::Color::Red),
        None,
        Some(crate::factor_tests::Color::Green),
        None,
    ]);
    let _ = fov.into_sexp();

    // Vec<Mode>: IntoR (Vec<T: MatchArg>) → match_arg_vec_into_sexp,
    // exercising the STRSXP build loop with the R_BlankString short-circuit
    // path skipped (no empty strings in CHOICES here).
    let _ = miniextendr_api::match_arg::match_arg_vec_into_sexp(vec![
        crate::match_arg_tests::Mode::Fast,
        crate::match_arg_tests::Mode::Safe,
        crate::match_arg_tests::Mode::Debug,
    ]);

    // choices_sexp<Mode> → STRSXP build with the same short-circuit branch.
    let _ = miniextendr_api::match_arg::choices_sexp::<crate::match_arg_tests::Mode>();

    // NamedVector<HashMap<String, i32>> → set_names_on_sexp STRSXP build.
    let mut m = HashMap::new();
    m.insert("alpha".to_string(), 1i32);
    m.insert("beta".to_string(), 2);
    m.insert("gamma".to_string(), 3);
    let _ = NamedVector(m).into_sexp();

    // Empty-key edge case: hits the `R_BlankString` short-circuit in
    // `set_names_on_sexp`.
    let mut m2 = HashMap::new();
    m2.insert(String::new(), 42i32);
    let _ = NamedVector(m2).into_sexp();

    // CHOICES read-back to silence unused-warning on the MatchArg trait import
    // (also a sanity check that the enum type is reachable at compile time).
    let _ = <crate::match_arg_tests::Mode as MatchArg>::CHOICES.len();
}

// endregion

// region: map_to_dataframe / result_to_dataframe / vec_to_dataframe_split helpers (#700, #697, #699)

/// Exercise `map_to_dataframe` under GC pressure.
///
/// Allocates a `BTreeMap<i32, KvValue>`, serialises through the helper,
/// converts to SEXP. Touches the same SEXP-storage code paths as
/// `vec_to_dataframe` (its underlying call) — `gctorture(TRUE)` validates
/// the schema-discovery / column-buffer / character-column protections.
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_map_to_dataframe() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::map_to_dataframe;
    use std::collections::BTreeMap;

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct KvValue {
        score: f64,
        label: String,
    }

    let mut map: BTreeMap<i32, KvValue> = BTreeMap::new();
    for i in 0..40i32 {
        map.insert(
            i,
            KvValue {
                score: f64::from(i) * 1.25,
                label: format!("entry_{i}"),
            },
        );
    }

    map_to_dataframe(&map, "id")
        .expect("gc_stress_map_to_dataframe: map_to_dataframe failed")
        .into_sexp()
}

/// Exercise `result_to_dataframe(Auto)` under GC pressure with mixed Ok/Err
/// rows. Exercises the split path (intermediate `DataFrame` pair +
/// `NamedDataFrameListBuilder` assembly in `IntoR for DataFrameShape`).
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_result_to_dataframe_auto() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{ResultShape, result_to_dataframe};

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct OkRow {
        id: i32,
        value: f64,
    }
    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct ErrRow {
        id: i32,
        reason: String,
    }

    let rows: Vec<Result<OkRow, ErrRow>> = (0..40i32)
        .map(|i| {
            if i % 3 == 0 {
                Err(ErrRow {
                    id: i,
                    reason: format!("err {i}"),
                })
            } else {
                Ok(OkRow {
                    id: i,
                    value: f64::from(i) * 1.5,
                })
            }
        })
        .collect();

    result_to_dataframe(
        &rows,
        ResultShape::Auto {
            empty_ok_sentinel: (),
        },
    )
    .expect("gc_stress_result_to_dataframe_auto: helper failed")
    .into_sexp()
}

/// Exercise `result_to_dataframe(Collated)` under GC pressure. Union-schema
/// emission goes through the `TaggedVariantRow`/`MapForwarder` path.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_result_to_dataframe_collated() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{ResultShape, result_to_dataframe};

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct OkRow {
        id: i32,
        value: f64,
    }
    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct ErrRow {
        id: i32,
        reason: String,
    }

    let rows: Vec<Result<OkRow, ErrRow>> = (0..30i32)
        .map(|i| {
            if i % 4 == 0 {
                Err(ErrRow {
                    id: i,
                    reason: format!("err {i}"),
                })
            } else {
                Ok(OkRow {
                    id: i,
                    value: f64::from(i) * 2.0,
                })
            }
        })
        .collect();

    result_to_dataframe::<_, _, ()>(&rows, ResultShape::Collated)
        .expect("gc_stress_result_to_dataframe_collated: helper failed")
        .into_sexp()
}

/// Exercise `result_to_dataframe(Split)` with all-Err input → sentinel
/// path. Validates that the user-supplied sentinel SEXP is rooted while
/// the outer named list is assembled.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_result_to_dataframe_split_sentinel() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{ResultShape, result_to_dataframe};

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct OkRow {
        id: i32,
    }
    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct ErrRow {
        id: i32,
        reason: String,
    }

    let rows: Vec<Result<OkRow, ErrRow>> = (0..25i32)
        .map(|i| {
            Err(ErrRow {
                id: i,
                reason: format!("err {i}"),
            })
        })
        .collect();

    // Sentinel is a String — ends up as an STRSXP and exercises the
    // protect-around-CHARSXP-allocation path inside DataFrameShape::IntoR.
    result_to_dataframe(
        &rows,
        ResultShape::Split {
            empty_ok_sentinel: String::from("no ok rows"),
        },
    )
    .expect("gc_stress_result_to_dataframe_split_sentinel: helper failed")
    .into_sexp()
}

/// Exercise `vec_to_dataframe_split(PerVariantListWithTag)` under GC
/// pressure. The tag-column prepend allocates a per-partition STRSXP that
/// must be protected through the `prepend_column` VECSXP reshuffle.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_split_with_tag() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{SplitShape, vec_to_dataframe_split};

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    enum Event {
        Click { x: f64, y: f64 },
        Scroll { delta: f64 },
        KeyPress { code: String },
    }

    let rows: Vec<Event> = (0..30i32)
        .map(|i| match i % 3 {
            0 => Event::Click {
                x: f64::from(i),
                y: f64::from(i) * 2.0,
            },
            1 => Event::Scroll {
                delta: f64::from(i) * -0.5,
            },
            _ => Event::KeyPress {
                code: format!("k{i}"),
            },
        })
        .collect();

    vec_to_dataframe_split(
        &rows,
        SplitShape::PerVariantListWithTag {
            column: "variant".into(),
        },
    )
    .expect("gc_stress_split_with_tag: helper failed")
    .into_sexp()
}

/// Exercise `vec_to_dataframe_split(Collated)` under GC pressure. Drives
/// the `TaggedVariantRow` / `MapForwarder` collation path through the
/// schema-union machinery in `vec_to_dataframe`.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_split_collated() -> SEXP {
    use miniextendr_api::into_r::IntoR as _;
    use miniextendr_api::serde::{SplitShape, vec_to_dataframe_split};

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    enum Event {
        Click { x: f64, y: f64 },
        Scroll { delta: f64 },
        KeyPress { code: String },
    }

    let rows: Vec<Event> = (0..30i32)
        .map(|i| match i % 3 {
            0 => Event::Click {
                x: f64::from(i),
                y: f64::from(i) * 2.0,
            },
            1 => Event::Scroll {
                delta: f64::from(i) * -0.5,
            },
            _ => Event::KeyPress {
                code: format!("k{i}"),
            },
        })
        .collect();

    vec_to_dataframe_split(
        &rows,
        SplitShape::Collated {
            column: "kind".into(),
        },
    )
    .expect("gc_stress_split_collated: helper failed")
    .into_sexp()
}

/// Hold a `DataFrameShape` across an R-allocation burst before converting —
/// the exact future-caller UAF #1265 closes.
///
/// Pre-#1265 the shape carried bare `DataFrame` views whose producing roots
/// (the split arm's `ProtectScope`, or the `BuiltDataFrame` handles deref'd
/// at the `Bare` boundary) released when the helper returned, so the shape
/// was sound only under an implicit "convert immediately, do not allocate in
/// between" contract. Now every leg rides its own root (`BuiltDataFrame` /
/// `RootedSentinel`), so this fixture allocates heavily *between* the helper
/// call and the `IntoR` conversion, then validates the converted values.
/// Under `gctorture(TRUE)` every allocation is a full GC — an unrooted
/// partition would be reaped in the burst.
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_dataframe_shape_held() {
    use miniextendr_api::dataframe::DataFrame;
    use miniextendr_api::serde::{
        DataFrameShape, ResultShape, SplitResults, SplitShape, result_to_dataframe,
        vec_to_dataframe_split,
    };

    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    enum Event {
        Click { x: f64, y: f64 },
        Scroll { delta: f64 },
    }
    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct OkRow {
        id: i32,
        value: f64,
    }
    #[derive(crate::serde::Serialize)]
    #[serde(crate = "crate::serde")]
    struct ErrRow {
        id: i32,
        reason: String,
    }

    let events: Vec<Event> = (0..24i32)
        .map(|i| {
            if i % 2 == 0 {
                Event::Click {
                    x: f64::from(i),
                    y: f64::from(i) * 2.0,
                }
            } else {
                Event::Scroll {
                    delta: f64::from(i) * -0.5,
                }
            }
        })
        .collect();

    // Force a burst of R allocations. Under gctorture(TRUE) every allocation
    // is a full GC — any unrooted frame inside a held shape is reclaimed here.
    let churn = || {
        for i in 0..32i32 {
            let v: Vec<f64> = (0..64).map(|j| f64::from(i + j)).collect();
            let _ = v.into_sexp();
            let s: Vec<Option<String>> = (0..8).map(|j| Some(format!("c{i}_{j}"))).collect();
            let _ = s.into_sexp();
        }
    };

    // PerVariantList (with tag): two partition handles held across the burst.
    {
        let shape = vec_to_dataframe_split(
            &events,
            SplitShape::PerVariantListWithTag {
                column: "variant".into(),
            },
        )
        .expect("shape_held: split failed");
        churn();
        // Read through the still-held handles first…
        let DataFrameShape::PerVariantList(parts) = &shape else {
            panic!("shape_held: expected PerVariantList");
        };
        assert_eq!(parts.len(), 2, "shape_held: partition count");
        for (name, df) in parts {
            assert_eq!(
                df.names().first().map(String::as_str),
                Some("variant"),
                "shape_held: tag column lost in {name}"
            );
        }
        // …then convert and validate the assembled list.
        // SAFETY: R main thread. Root the converted list across the reads
        // below — `nrow()`/`from_sexp` go through `Rf_getAttrib(row.names)`,
        // which can allocate (compact row.names expand to an ALTREP range).
        let sexp = unsafe { OwnedProtect::new(shape.into_sexp()) };
        assert_eq!(sexp.xlength(), 2, "shape_held: list length");
        let names = sexp.get_names();
        assert_eq!(names.string_elt_str(0), Some("Click"));
        assert_eq!(names.string_elt_str(1), Some("Scroll"));
        let click = DataFrame::from_sexp(sexp.vector_elt(0)).expect("shape_held: Click partition");
        assert_eq!(click.nrow(), 12, "shape_held: Click nrow");
    }

    // Bare (collated): a single frame handle held across the burst.
    {
        let shape = vec_to_dataframe_split(
            &events,
            SplitShape::Collated {
                column: "kind".into(),
            },
        )
        .expect("shape_held: collated failed");
        churn();
        let DataFrameShape::Bare(df) = &shape else {
            panic!("shape_held: expected Bare");
        };
        assert_eq!(df.nrow(), 24, "shape_held: collated nrow");
        assert_eq!(
            df.names().first().map(String::as_str),
            Some("kind"),
            "shape_held: collated tag column"
        );
    }

    // Split with an Ok partition: both frame handles held across the burst.
    {
        let results: Vec<Result<OkRow, ErrRow>> = (0..20i32)
            .map(|i| {
                if i % 4 == 0 {
                    Err(ErrRow {
                        id: i,
                        reason: format!("err {i}"),
                    })
                } else {
                    Ok(OkRow {
                        id: i,
                        value: f64::from(i) * 1.5,
                    })
                }
            })
            .collect();
        let shape = result_to_dataframe(
            &results,
            ResultShape::Auto {
                empty_ok_sentinel: (),
            },
        )
        .expect("shape_held: result Auto failed");
        churn();
        // SAFETY: as above — root the converted list across the reads.
        let sexp = unsafe { OwnedProtect::new(shape.into_sexp()) };
        assert_eq!(sexp.xlength(), 2, "shape_held: results+error length");
        let ok_df =
            DataFrame::from_sexp(sexp.vector_elt(0)).expect("shape_held: results partition");
        assert_eq!(ok_df.nrow(), 15, "shape_held: Ok nrow");
        let err_df = DataFrame::from_sexp(sexp.vector_elt(1)).expect("shape_held: error partition");
        assert_eq!(err_df.nrow(), 5, "shape_held: Err nrow");
    }

    // Split with the all-Err sentinel: the `RootedSentinel`'s own root must
    // keep the sentinel STRSXP alive across the burst.
    {
        let all_err: Vec<Result<OkRow, ErrRow>> = (0..10i32)
            .map(|i| {
                Err(ErrRow {
                    id: i,
                    reason: format!("err {i}"),
                })
            })
            .collect();
        let shape = result_to_dataframe(
            &all_err,
            ResultShape::Split {
                empty_ok_sentinel: String::from("no ok rows"),
            },
        )
        .expect("shape_held: result Split failed");
        churn();
        let DataFrameShape::Split {
            results: SplitResults::None(sentinel),
            ..
        } = &shape
        else {
            panic!("shape_held: expected Split with sentinel");
        };
        assert_eq!(
            sentinel.as_sexp().string_elt_str(0),
            Some("no ok rows"),
            "shape_held: sentinel reclaimed"
        );
        // SAFETY: as above — root the converted list across the reads.
        let sexp = unsafe { OwnedProtect::new(shape.into_sexp()) };
        let results_slot = sexp.vector_elt(0);
        assert_eq!(
            results_slot.string_elt_str(0),
            Some("no ok rows"),
            "shape_held: sentinel slot corrupted"
        );
        let err_df =
            DataFrame::from_sexp(sexp.vector_elt(1)).expect("shape_held: all-err partition");
        assert_eq!(err_df.nrow(), 10, "shape_held: all-err nrow");
    }
}

// endregion

// region: factor-label dataframe_to_vec GC stress (issue #689)

/// Exercise the factor-label path through `dataframe_to_vec` under GC
/// pressure.
///
/// The factor branch in `CellDeserializer::deserialize_str/string/char/any`
/// reads `levels` (via `Rf_getAttrib`) and dereferences `STRING_ELT` on the
/// levels SEXP — a path that crosses a GC barrier. This fixture synthesises
/// a one-column factor data.frame internally, then runs `dataframe_to_vec`
/// to reconstruct `Vec<Row { status: String }>`. The intermediate factor +
/// data.frame SEXPs are protected via `OwnedProtect`; the deserialiser must
/// keep them rooted across the per-row label lookups.
///
/// No arguments — suitable for the fast gctorture no-arg sweep.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_factor_labels() -> i32 {
    use crate::serde::Deserialize;
    use miniextendr_api::SEXPTYPE;
    use miniextendr_api::factor::{build_factor, build_levels_sexp};
    use miniextendr_api::gc_protect::OwnedProtect;
    use miniextendr_api::prelude::{SEXP, SexpExt};
    use miniextendr_api::serde::dataframe_to_vec;
    use miniextendr_api::sys::Rf_allocVector;

    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(crate = "crate::serde")]
    struct Row {
        status: Option<String>,
    }

    let levels = ["active", "pending", "archived"];
    // Build 30 rows cycling through codes 1..=3 to exercise multiple label
    // hits per call, plus a few NA cells (NA_INTEGER → `status: None`).
    const NA: i32 = i32::MIN; // NA_INTEGER
    let codes: Vec<i32> = (0..30)
        .map(|i| if i % 7 == 0 { NA } else { i % 3 + 1 })
        .collect();

    // SAFETY: all R API calls happen on the worker thread (this fixture is
    // invoked via the #[miniextendr] wrapper), and every transient
    // allocation is protected before the next allocation can run GC.
    let df = unsafe {
        let levels_sexp = build_levels_sexp(&levels);
        let _levels_guard = OwnedProtect::new(levels_sexp);
        let col = build_factor(&codes, levels_sexp);
        let _col_guard = OwnedProtect::new(col);

        let list = Rf_allocVector(SEXPTYPE::VECSXP, 1);
        let _list_guard = OwnedProtect::new(list);
        list.set_vector_elt(0, col);

        let names_sexp = Rf_allocVector(SEXPTYPE::STRSXP, 1);
        let _names_guard = OwnedProtect::new(names_sexp);
        names_sexp.set_string_elt(0, SEXP::charsxp("status"));
        list.set_names(names_sexp);

        let class_sexp = Rf_allocVector(SEXPTYPE::STRSXP, 1);
        let _class_guard = OwnedProtect::new(class_sexp);
        class_sexp.set_string_elt(0, SEXP::charsxp("data.frame"));
        list.set_class(class_sexp);

        let row_names = Rf_allocVector(SEXPTYPE::INTSXP, 2);
        let _rn_guard = OwnedProtect::new(row_names);
        let rn = row_names.as_mut_slice::<i32>();
        rn[0] = i32::MIN;
        rn[1] = -(codes.len() as i32);
        list.set_row_names(row_names);

        // Hand `list` to dataframe_to_vec while every transient is still
        // protected — the inner guards drop at the end of this block, but
        // by then the only live SEXP we care about (`list`) is unprotected
        // and immediately consumed.
        dataframe_to_vec::<Row>(list).expect("gc_stress_factor_labels: dataframe_to_vec failed")
    };

    assert_eq!(df.len(), codes.len());
    let mut na_count = 0;
    for (i, row) in df.iter().enumerate() {
        match (&row.status, codes[i]) {
            (None, NA) => na_count += 1,
            (Some(label), code) if code != NA => assert!(
                levels.contains(&label.as_str()),
                "unexpected label at row {i}: {label}"
            ),
            other => panic!("row {i} mismatch: {:?} vs code {}", other, codes[i]),
        }
    }
    assert!(na_count > 0, "fixture should exercise NA rows");
    df.len() as i32
}

// endregion

// region: typed_dataframe! (#698)

/// Exercise the `typed_dataframe!`-generated `TryFromSexp` impl under GC
/// pressure.
///
/// Synthesises an R `data.frame` from scratch, drives it through
/// `TheophDf::try_from_sexp`, and then forces every per-column borrowed
/// accessor to read its slice. The accessors call
/// `RNativeType::dataptr_mut` which is the path most likely to surface
/// PROTECT-discipline bugs (each column is allocated then promoted into a
/// generic VECSXP via `from_raw_pairs`, then attribute-tagged into a
/// data.frame).
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[miniextendr(noexport)]
pub fn gc_stress_typed_dataframe() {
    use crate::typed_dataframe_tests::TheophDf;
    use miniextendr_api::IntoR as _;
    use miniextendr_api::from_r::TryFromSexp as _;
    use miniextendr_api::gc_protect::ProtectScope;
    use miniextendr_api::list::List;

    // Construct a Theoph-shaped data.frame internally.
    //
    // Each column SEXP must be PROTECTed before `List::from_raw_pairs`
    // takes ownership — the docs say "The input SEXPs should already
    // be protected." Without this, gctorture(TRUE) (and even normal
    // R-devel runs) can reap unprotected column SEXPs while the names
    // STRSXP is being built.
    let nrow = 12usize;
    let subject: Vec<i32> = (0..nrow as i32).collect();
    let weight: Vec<f64> = (0..nrow).map(|i| 60.0 + i as f64).collect();
    let dose: Vec<f64> = vec![320.0; nrow];
    let time: Vec<f64> = (0..nrow).map(|i| i as f64 * 0.5).collect();
    let conc: Vec<f64> = (0..nrow).map(|i| 1.0 + i as f64 * 0.1).collect();

    let scope = unsafe { ProtectScope::new() };
    let subject_sexp = unsafe { scope.protect_raw(subject.into_sexp()) };
    let weight_sexp = unsafe { scope.protect_raw(weight.into_sexp()) };
    let dose_sexp = unsafe { scope.protect_raw(dose.into_sexp()) };
    let time_sexp = unsafe { scope.protect_raw(time.into_sexp()) };
    let conc_sexp = unsafe { scope.protect_raw(conc.into_sexp()) };

    let list = List::from_raw_pairs(vec![
        ("subject", subject_sexp),
        ("weight", weight_sexp),
        ("dose", dose_sexp),
        ("time", time_sexp),
        ("conc", conc_sexp),
    ]);
    // Root the container: from_raw_pairs' internal guards drop on return, so
    // the fresh VECSXP is unprotected while as_data_frame() allocates its
    // class / row.names attributes and TryFromSexp validates. Without this,
    // gctorture reaps the container and the names attribute reads back nil
    // once the cell is reused ("DataFrame always carries a names attribute").
    unsafe { scope.protect_raw(list.as_sexp()) };
    let df = list
        .as_data_frame()
        .expect("synthetic data.frame promotion should succeed");
    let sexp = df.as_sexp();

    // Drive the validating TryFromSexp path. The result stores per-column
    // SEXPs as plain fields; the surrounding data.frame SEXP is owned by
    // `df` (held alive by `sexp`).
    let theoph = TheophDf::try_from_sexp(sexp).expect("TheophDf validation should succeed");

    // Force every accessor to read its slice — this calls
    // `dataptr_mut` on every column SEXP, exercising any PROTECT
    // problems in storage.
    let subj_sum: i32 = theoph.subject().iter().copied().sum();
    let weight_sum: f64 = theoph.weight().iter().copied().sum();
    let dose_sum: f64 = theoph.dose().iter().copied().sum();
    let time_sum: f64 = theoph.time().iter().copied().sum();
    let conc_sum: f64 = theoph.conc().iter().copied().sum();
    assert_eq!(theoph.nrow(), nrow);
    assert!(subj_sum >= 0);
    assert!(weight_sum > 0.0);
    assert!(dose_sum > 0.0);
    assert!(time_sum >= 0.0);
    assert!(conc_sum > 0.0);

    // Optional column was absent in this fixture.
    assert!(theoph.flag().is_none());

    drop(scope);

    // Now repeat the dance with the optional column populated, so the
    // `Option<SEXP>` storage path is also stressed.
    let subject2: Vec<i32> = (0..nrow as i32).collect();
    let weight2: Vec<f64> = (0..nrow).map(|i| 50.0 + i as f64).collect();
    let dose2: Vec<f64> = vec![160.0; nrow];
    let time2: Vec<f64> = (0..nrow).map(|i| i as f64).collect();
    let conc2: Vec<f64> = (0..nrow).map(|i| 2.0 + i as f64 * 0.25).collect();
    let flag2: Vec<i32> = (0..nrow as i32).map(|i| i % 2).collect();

    let scope2 = unsafe { ProtectScope::new() };
    let subject2_sexp = unsafe { scope2.protect_raw(subject2.into_sexp()) };
    let weight2_sexp = unsafe { scope2.protect_raw(weight2.into_sexp()) };
    let dose2_sexp = unsafe { scope2.protect_raw(dose2.into_sexp()) };
    let time2_sexp = unsafe { scope2.protect_raw(time2.into_sexp()) };
    let conc2_sexp = unsafe { scope2.protect_raw(conc2.into_sexp()) };
    let flag2_sexp = unsafe { scope2.protect_raw(flag2.into_sexp()) };

    let list2 = List::from_raw_pairs(vec![
        ("subject", subject2_sexp),
        ("weight", weight2_sexp),
        ("dose", dose2_sexp),
        ("time", time2_sexp),
        ("conc", conc2_sexp),
        ("flag", flag2_sexp),
    ]);
    // Same rooting as above: the container outlives from_raw_pairs' guards.
    unsafe { scope2.protect_raw(list2.as_sexp()) };
    let df2 = list2
        .as_data_frame()
        .expect("synthetic data.frame promotion should succeed");
    let sexp2 = df2.as_sexp();

    let theoph2 =
        TheophDf::try_from_sexp(sexp2).expect("TheophDf validation should succeed (with flag)");
    let flag_sum: i32 = theoph2
        .flag()
        .expect("flag column should be present")
        .iter()
        .copied()
        .sum();
    assert!(flag_sum >= 0);
}

// endregion

// region: rayon RDataFrameBuilder

/// Exercise `RDataFrameBuilder` parallel column-fill under GC pressure.
///
/// Builds a heterogeneous data.frame (numeric `x`, integer `y`, character
/// `label` with interspersed `NA`) via the parallel builder, across two column
/// shapes:
///
/// 1. A balanced 3-column frame (`x`/`y`/`label`).
/// 2. A **few-long-columns** frame (one numeric + one character, both spanning
///    enough rows to split into many row-range chunks) — the shape the flattened
///    `(column, row-range)` work-list targets, and the one most likely to expose
///    a PROTECT-window bug because the parallel pass touches many disjoint slices
///    of the same column buffers.
///
/// The builder allocates each column SEXP serially, holds them protected across
/// subsequent column / names / row.names / class allocations, and assembles the
/// parent `VECSXP`. This is exactly the SEXP-across-allocation path that needs a
/// gctorture pass. No arguments — suitable for the fast gctorture sweep.
#[cfg(feature = "rayon")]
#[miniextendr(noexport)]
pub fn gc_stress_dataframe_rayon() {
    // Shape 1: balanced 3-column frame.
    build_and_check_rayon_df(200);

    // Shape 2: few-long-columns frame (the flattening target).
    build_and_check_rayon_df_tall(5000);
}

/// Build a 3-column (`f64`/`i32`/`character`) rayon data.frame and assert it.
#[cfg(feature = "rayon")]
fn build_and_check_rayon_df(nrow: usize) {
    use miniextendr_api::gc_protect::ProtectScope;
    use miniextendr_api::rayon_bridge::RDataFrameBuilder;

    let df = RDataFrameBuilder::new(nrow)
        .column::<f64>("x", |chunk: &mut [f64], offset: usize| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = ((offset + i) as f64).sqrt();
            }
        })
        .column::<i32>("y", |chunk: &mut [i32], offset: usize| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = (offset + i) as i32 * 3;
            }
        })
        .column_str("label", |i: usize| {
            if i % 7 == 6 {
                None
            } else {
                Some(format!("row_{i}"))
            }
        })
        .build();

    // Force materialization of every column under the same protect scope so any
    // stale/freed column surfaces. The builder leaves `df` unprotected, so
    // protect it before reading.
    let scope = unsafe { ProtectScope::new() };
    let df = unsafe { scope.protect_raw(df.as_sexp()) };
    assert!(df.is_data_frame());
    assert_eq!(df.xlength(), 3);

    let x = df.vector_elt(0);
    let y = df.vector_elt(1);
    let label = df.vector_elt(2);
    assert_eq!(x.xlength() as usize, nrow);
    assert_eq!(y.xlength() as usize, nrow);
    assert_eq!(label.xlength() as usize, nrow);

    let x_slice: &[f64] = unsafe { x.as_slice() };
    let y_slice: &[i32] = unsafe { y.as_slice() };
    let x_sum: f64 = x_slice.iter().copied().sum();
    let y_sum: i32 = y_slice.iter().copied().sum();
    assert!(x_sum > 0.0);
    assert!(y_sum > 0);

    // Touch each CHARSXP, including the NA slots.
    let mut na_count = 0usize;
    for i in 0..nrow as isize {
        if label.string_elt_str(i).is_none() {
            na_count += 1;
        }
    }
    assert!(na_count > 0);

    drop(scope);
}

/// Build a few-long-columns rayon data.frame (one `f64` + one character column,
/// `nrow` rows) and assert it. This shape splits each column into many row-range
/// chunks, exercising the flattened work-list's disjoint-slice writes.
#[cfg(feature = "rayon")]
fn build_and_check_rayon_df_tall(nrow: usize) {
    use miniextendr_api::gc_protect::ProtectScope;
    use miniextendr_api::rayon_bridge::RDataFrameBuilder;

    let df = RDataFrameBuilder::new(nrow)
        .column::<f64>("v", |chunk: &mut [f64], offset: usize| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = (offset + i) as f64 * 0.5;
            }
        })
        .column_str("s", |i: usize| {
            if i % 11 == 10 {
                None
            } else {
                Some(format!("v_{i}"))
            }
        })
        .build();

    let scope = unsafe { ProtectScope::new() };
    let df = unsafe { scope.protect_raw(df.as_sexp()) };
    assert!(df.is_data_frame());
    assert_eq!(df.xlength(), 2);

    let v = df.vector_elt(0);
    let s = df.vector_elt(1);
    assert_eq!(v.xlength() as usize, nrow);
    assert_eq!(s.xlength() as usize, nrow);

    // Every native slot must equal its index formula — catches any chunk that
    // was skipped or double-written by the flattened scheduler.
    let v_slice: &[f64] = unsafe { v.as_slice() };
    for (i, &val) in v_slice.iter().enumerate() {
        assert_eq!(val, i as f64 * 0.5);
    }

    // Touch every CHARSXP including the NA slots.
    let mut na_count = 0usize;
    for i in 0..nrow as isize {
        if s.string_elt_str(i).is_none() {
            na_count += 1;
        }
    }
    assert!(na_count > 0);

    drop(scope);
}

// endregion

// region: DataFrame::builder serial (non-rayon) GC-stress fixture (#1055)

/// Exercise `DataFrame::builder` on the **non-rayon** (serial) fill path under GC
/// pressure.
///
/// rpkg compiles with `rayon` **off** by default, so this fixture drives the
/// serial fill branch of `RDataFrameBuilder::build` (#1055) — proving the builder
/// surface is reachable and correct without the `rayon` feature. It uses the
/// public `DataFrame::builder(nrow)` entry point (no `rayon_bridge` import), then
/// materializes every column under a single protect scope so any stale/freed
/// column surfaces. The builder allocates each column SEXP serially and holds it
/// protected across the subsequent column / names / row.names / class allocations
/// — the SEXP-across-allocation path that needs a gctorture pass. No arguments —
/// suitable for the fast gctorture sweep.
#[miniextendr(noexport)]
pub fn gc_stress_dataframe_builder_serial() {
    use miniextendr_api::dataframe::DataFrame;
    use miniextendr_api::gc_protect::ProtectScope;

    let nrow = 256usize;
    let df = DataFrame::builder(nrow)
        .column::<f64>("x", |chunk: &mut [f64], offset: usize| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = (offset + i) as f64;
            }
        })
        .column::<i32>("y", |chunk: &mut [i32], offset: usize| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = (offset + i) as i32 * 2;
            }
        })
        .column_str("label", |i: usize| {
            if i % 5 == 4 {
                None
            } else {
                Some(format!("row_{i}"))
            }
        })
        .build();

    // The builder leaves `df` unprotected; protect before reading.
    let scope = unsafe { ProtectScope::new() };
    let df = unsafe { scope.protect_raw(df.as_sexp()) };
    assert!(df.is_data_frame());
    assert_eq!(df.xlength(), 3);

    let x = df.vector_elt(0);
    let y = df.vector_elt(1);
    let label = df.vector_elt(2);
    assert_eq!(x.xlength() as usize, nrow);
    assert_eq!(y.xlength() as usize, nrow);
    assert_eq!(label.xlength() as usize, nrow);

    // Serial fill writes the full range once: every slot must match its formula.
    let x_slice: &[f64] = unsafe { x.as_slice() };
    let y_slice: &[i32] = unsafe { y.as_slice() };
    for (i, &val) in x_slice.iter().enumerate() {
        assert_eq!(val, i as f64);
    }
    for (i, &val) in y_slice.iter().enumerate() {
        assert_eq!(val, i as i32 * 2);
    }

    // Touch every CHARSXP, including the NA slots.
    let mut na_count = 0usize;
    for i in 0..nrow as isize {
        if label.string_elt_str(i).is_none() {
            na_count += 1;
        }
    }
    assert!(na_count > 0);

    drop(scope);
}

// endregion

// region: enum reader GC-stress fixtures (#807)
//
// The enum reader allocates sub-frames (via `select` + `strip_prefix` + `select_rows`)
// while holding column SEXPs — a pattern that requires PROTECT discipline across
// the per-column loop. These no-arg fixtures exercise that path under gctorture(TRUE).

/// Exercise the nested-payload enum flatten reader under GC pressure.
///
/// Builds a `RETracked` frame with the writer, holds the df SEXP live, then reads
/// it back with `Vec::<RETracked>::from_dataframe`. The reader calls `select` +
/// `strip_prefix` + `select_rows` + inner reader, each of which may fire GC.
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[miniextendr(noexport)]
pub fn gc_stress_reader_enum_flatten() {
    use crate::dataframe_reader_enum_roundtrip_test::{REStatus, RETracked};
    use miniextendr_api::dataframe::{DataFrame, FromDataFrame, IntoDataFrame};
    use miniextendr_api::into_r::IntoR as _;

    let rows = vec![
        RETracked::Tracked {
            id: 1,
            status: REStatus::Ok,
        },
        RETracked::Tracked {
            id: 2,
            status: REStatus::Err { code: 500 },
        },
        RETracked::Other { id: 3 },
        RETracked::Tracked {
            id: 4,
            status: REStatus::Ok,
        },
    ];
    let df_sexp = rows.into_dataframe().unwrap().into_sexp();
    // Root the writer-produced frame across the read-back. A Rust binding does NOT
    // protect an R SEXP, so under gctorture(TRUE) the sub-frame select/densify
    // allocations would reclaim the parent frame mid-read. In real usage the input
    // frame is an R-rooted call argument; the fixture stands in for that root.
    let _df_guard = unsafe { miniextendr_api::OwnedProtect::new(df_sexp) };
    let frame = DataFrame::from_sexp(df_sexp).unwrap();
    let back: Vec<RETracked> = <Vec<RETracked>>::from_dataframe(&frame).unwrap();
    assert_eq!(back.len(), 4);
}

/// Exercise the as_factor unit-enum reader under GC pressure.
///
/// Builds a `REMove` frame with the writer, holds the df SEXP live, then reads
/// it back with `Vec::<REMove>::from_dataframe`. The reader calls
/// `unit_factor_option_vec_from_sexp` which validates levels and reads INTSXP
/// elements — exercises the factor PROTECT path.
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[miniextendr(noexport)]
pub fn gc_stress_reader_enum_factor() {
    use crate::dataframe_reader_enum_roundtrip_test::{REDir, REMove};
    use miniextendr_api::dataframe::{DataFrame, FromDataFrame, IntoDataFrame};
    use miniextendr_api::into_r::IntoR as _;

    let rows = vec![
        REMove::Move {
            id: 1,
            dir: REDir::East,
        },
        REMove::Stop { id: 2 },
        REMove::Move {
            id: 3,
            dir: REDir::South,
        },
        REMove::Stop { id: 4 },
        REMove::Move {
            id: 5,
            dir: REDir::North,
        },
    ];
    let df_sexp = rows.into_dataframe().unwrap().into_sexp();
    // Root the writer-produced frame across the read-back. A Rust binding does NOT
    // protect an R SEXP, so under gctorture(TRUE) the factor-level allocations in the
    // reader would reclaim the parent frame mid-read. In real usage the input frame is
    // an R-rooted call argument; the fixture stands in for that root.
    let _df_guard = unsafe { miniextendr_api::OwnedProtect::new(df_sexp) };
    let frame = DataFrame::from_sexp(df_sexp).unwrap();
    let back: Vec<REMove> = <Vec<REMove>>::from_dataframe(&frame).unwrap();
    assert_eq!(back.len(), 5);
}

/// Exercise the map-column enum reader under GC pressure.
///
/// Builds a `REMapB` frame with the writer, holds the df SEXP live, then reads it
/// back with `Vec::<REMapB>::from_dataframe`. The reader walks the `tally_keys` /
/// `tally_values` VECSXP list-columns, converting each row's element via
/// `Vec<elem>: TryFromSexp` — exercises the list-column regroup path (NULL rows,
/// empty-map rows, and populated rows all present in the fixture frame).
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[miniextendr(noexport)]
pub fn gc_stress_reader_enum_map() {
    use crate::dataframe_reader_enum_roundtrip_test::REMapB;
    use miniextendr_api::dataframe::{DataFrame, FromDataFrame, IntoDataFrame};
    use miniextendr_api::into_r::IntoR as _;
    use std::collections::BTreeMap;

    let rows = vec![
        REMapB::Tally {
            label: "a".to_string(),
            tally: BTreeMap::from([("x".to_string(), 1i32), ("y".to_string(), 2i32)]),
        },
        REMapB::Empty {
            label: "b".to_string(),
        },
        REMapB::Tally {
            label: "c".to_string(),
            tally: BTreeMap::new(),
        },
        REMapB::Tally {
            label: "d".to_string(),
            tally: BTreeMap::from([("z".to_string(), 9i32)]),
        },
    ];
    let df_sexp = rows.into_dataframe().unwrap().into_sexp();
    // Root the writer-produced frame across the read-back. A Rust binding does NOT
    // protect an R SEXP, so under gctorture(TRUE) a later allocation would reclaim it
    // mid-read. In real usage the input frame is an R-rooted call argument; the fixture
    // stands in for that root.
    let _df_guard = unsafe { miniextendr_api::OwnedProtect::new(df_sexp) };
    let frame = DataFrame::from_sexp(df_sexp).unwrap();
    let back: Vec<REMapB> = <Vec<REMapB>>::from_dataframe(&frame).unwrap();
    assert_eq!(back.len(), 4);
}

// endregion

// region: Arrow RecordBatch materialization (#867)

/// Materialize a `RecordBatch` whose columns are *sliced views* of R-backed
/// Arrow buffers, exercising `RecordBatch::into_sexp` → `arrow_array_to_sexp`
/// → the per-array zero-copy SEXP-recovery path under GC pressure.
///
/// Regression fixture for #867. The subquery fixture (now folded into
/// `test_df_sql_query`) segfaulted on the strict
/// glibc Linux runner because DataFusion's contiguous-run filter optimization
/// returns a *slice* of the R-backed input column: `values().as_ptr()` then
/// points into the middle of the R vector, and the speculative
/// `try_recover_r_sexp` probe (which subtracts the SEXPREC header offset)
/// read off into unrelated memory and false-positived as a "recovered SEXP".
/// The crash was heap-layout-dependent — deterministic on the strict runner,
/// silent elsewhere — so a no-arg fixture is the only portable guard.
///
/// This fixture builds R-backed `Int32Array`/`Float64Array` columns, slices
/// them (offset > 0, the exact filter-optimization shape), assembles a
/// `RecordBatch`, and drives the production materialization. It then verifies
/// the resulting data.frame holds the correct values — a false-positive
/// recovery would yield wrong values or corrupt memory rather than copying.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[cfg(feature = "arrow")]
#[miniextendr(noexport)]
pub fn gc_stress_arrow_sliced_recordbatch() {
    use miniextendr_api::arrow_impl::{
        ArrayRef, DataType, Field, Float64Array, Int32Array, RecordBatch, Schema,
    };
    use miniextendr_api::from_r::TryFromSexp;
    use miniextendr_api::into_r::IntoR;
    use miniextendr_api::prelude::SexpExt;
    use std::sync::Arc;

    // Build R-backed Arrow arrays from fresh R vectors, then slice them so the
    // value pointer is advanced into the middle of the R buffer (offset = 2),
    // matching DataFusion's contiguous-run filter output.
    let xi_sexp = vec![1i32, 2, 3, 4, 5].into_sexp();
    let _xi_guard = unsafe { miniextendr_api::OwnedProtect::new(xi_sexp) };
    let xi = Int32Array::try_from_sexp(xi_sexp).expect("int32 from R");

    let yf_sexp = vec![10.0f64, 20.0, 30.0, 40.0, 50.0].into_sexp();
    let _yf_guard = unsafe { miniextendr_api::OwnedProtect::new(yf_sexp) };
    let yf = Float64Array::try_from_sexp(yf_sexp).expect("float64 from R");

    let x_slice = xi.slice(2, 3); // c(3L, 4L, 5L)
    let y_slice = yf.slice(2, 3); // c(30, 40, 50)

    let schema = Arc::new(Schema::new(vec![
        Field::new("x", DataType::Int32, false),
        Field::new("y", DataType::Float64, false),
    ]));
    let cols: Vec<ArrayRef> = vec![Arc::new(x_slice), Arc::new(y_slice)];
    let batch = RecordBatch::try_new(schema, cols).expect("record batch");

    // Drive the production materialization (the path a subquery via
    // test_df_sql_query hits).
    let out = batch.into_sexp();
    let _out_guard = unsafe { miniextendr_api::OwnedProtect::new(out) };

    // Honest check: the data.frame must hold the correct sliced values, proving
    // the materialization copied rather than returning a bogus recovered SEXP.
    let names = out.get_names();
    assert_eq!(names.len(), 2);
    let x_col = out.vector_elt(0);
    let y_col = out.vector_elt(1);
    let xs: &[i32] = unsafe { x_col.as_slice() };
    let ys: &[f64] = unsafe { y_col.as_slice() };
    assert_eq!(xs, [3, 4, 5]);
    assert_eq!(ys, [30.0, 40.0, 50.0]);
}

// endregion

// region: Cow<[T]> borrowed sub-slice round-trip (#880)

/// Round-trip a `Cow::Borrowed` *sub-slice* of an R-backed vector through
/// `IntoR`, exercising the `Cow<[T]>::into_sexp` copy path under GC pressure.
///
/// Regression guard for #880. A borrowed sub-slice (`&full[2..]`) points into
/// the *middle* of an R vector: `slice.as_ptr()` is offset from the SEXP data
/// start, so the old speculative `try_recover_r_sexp` probe (`data_ptr − header`)
/// would read provenance-free memory off the front of the vector and could
/// false-positive as a "recovered SEXP" — the same class of hazard #867 fixed
/// for Arrow, but for a bare `&[T]` that carries no metadata to gate on. The
/// fix removes the probe and always copies; this fixture proves the copy
/// round-trips the correct sliced values without crashing or returning a bogus
/// SEXP.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_cow_subslice_roundtrip() {
    use miniextendr_api::from_r::TryFromSexp;
    use std::borrow::Cow;

    // R-backed source vectors. Keep them rooted across the allocations that
    // `.into_sexp()` performs (mirrors a rooted call argument in real usage).
    let f_sexp = vec![1.0f64, 2.0, 3.0, 4.0, 5.0].into_sexp();
    let _f_guard = unsafe { miniextendr_api::OwnedProtect::new(f_sexp) };
    let i_sexp = vec![10i32, 20, 30, 40, 50].into_sexp();
    let _i_guard = unsafe { miniextendr_api::OwnedProtect::new(i_sexp) };

    // Borrow as 'static slices (offset 0), then sub-slice into the middle so
    // `as_ptr()` no longer points at the R vector's data start — the exact
    // shape that defeated the old speculative recovery.
    let f_cow: Cow<'static, [f64]> = TryFromSexp::try_from_sexp(f_sexp).unwrap();
    let Cow::Borrowed(f_full) = f_cow else {
        unreachable!("Cow<[f64]> from R is always Borrowed");
    };
    let i_cow: Cow<'static, [i32]> = TryFromSexp::try_from_sexp(i_sexp).unwrap();
    let Cow::Borrowed(i_full) = i_cow else {
        unreachable!("Cow<[i32]> from R is always Borrowed");
    };

    let f_sub: Cow<'static, [f64]> = Cow::Borrowed(&f_full[2..5]); // c(3, 4, 5)
    let i_sub: Cow<'static, [i32]> = Cow::Borrowed(&i_full[1..4]); // c(20L, 30L, 40L)

    let f_out = f_sub.into_sexp();
    let _f_out_guard = unsafe { miniextendr_api::OwnedProtect::new(f_out) };
    let i_out = i_sub.into_sexp();
    let _i_out_guard = unsafe { miniextendr_api::OwnedProtect::new(i_out) };

    // The copy must hold the correct sliced values — a bogus recovered SEXP
    // would surface as wrong values (or corruption) rather than a clean copy.
    let fs: &[f64] = unsafe { f_out.as_slice() };
    let is: &[i32] = unsafe { i_out.as_slice() };
    assert_eq!(fs, [3.0, 4.0, 5.0]);
    assert_eq!(is, [20, 30, 40]);
}

// endregion

// region: RCow<T> round-trip (safe zero-copy — #880)

/// Drive `RCow<T>`'s borrowed (zero-copy) and owned (copy-on-write) `IntoR`
/// paths under GC pressure.
///
/// Companion to `gc_stress_cow_subslice_roundtrip` (#880). `RCow` is the safe
/// zero-copy round-trip type: its borrowed arm carries the source SEXP, so
/// `IntoR` hands that SEXP straight back with no `data_ptr − header` probe. This
/// fixture confirms (a) the borrowed round-trip returns the *same* SEXP, and
/// (b) `to_mut()` forces a fresh, value-correct copy — both without corruption
/// under `gctorture(TRUE)`.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_rcow_roundtrip() {
    use miniextendr_api::RCow;
    use miniextendr_api::from_r::TryFromSexp;

    let src = vec![1.0f64, 2.0, 3.0, 4.0, 5.0].into_sexp();
    let _src_guard = unsafe { miniextendr_api::OwnedProtect::new(src) };

    // Borrowed round-trip must return the *same* SEXP (true zero-copy).
    let borrowed: RCow<'static, f64> = TryFromSexp::try_from_sexp(src).unwrap();
    let back = borrowed.into_sexp();
    assert_eq!(back, src, "borrowed RCow round-trip must be identity");

    // Copy-on-write: mutating forces a fresh, value-correct R vector.
    let mut owned: RCow<'static, f64> = TryFromSexp::try_from_sexp(src).unwrap();
    for v in owned.to_mut() {
        *v += 100.0;
    }
    let out = owned.into_sexp();
    let _out_guard = unsafe { miniextendr_api::OwnedProtect::new(out) };
    assert_ne!(out, src, "mutated RCow must be a fresh SEXP");
    let vals: &[f64] = unsafe { out.as_slice() };
    assert_eq!(vals, [101.0, 102.0, 103.0, 104.0, 105.0]);
}

// endregion

// region: R longjmp inside with_r_thread from a worker job (#733)

/// Drive an **R error raised inside a `with_r_thread` closure from a
/// `run_on_worker` job** end-to-end, so the `cleanup_handler` /
/// `R_ContinueUnwind` path in `worker.rs` (the worker-job R-longjmp branch)
/// is exercised under R's normal top-level error handler.
///
/// This is the #733 follow-up to #731's `worker_channel_stress.rs` cargo
/// suite. That cargo path **cannot** safely cover this case: the cargo-test R
/// embedding (`r_test_utils.rs`) runs raw FFI jobs in a `for` loop with no R
/// top-level handler, so `R_ContinueUnwind(token)` would resume an unwind with
/// nowhere to land — a likely segfault rather than a clean test. Inside an
/// rpkg testthat run R's normal error handler is in place, so the resumed
/// unwind lands at the top level and the framework re-raises it as a tagged
/// condition.
///
/// Shape (entirely synthesised internally so it's callable with no args, per
/// the no-arg gctorture-fixture convention, #430):
///
/// 1. The fn returns `SEXP`, so the macro picks the **MainThread** strategy:
///    the body runs on R's main thread and is the thread that drives
///    `dispatch_to_worker`'s event loop (no re-entrant `run_on_worker`).
/// 2. `run_on_worker` moves the closure onto the worker thread.
/// 3. From the worker, `with_r_thread` routes back to the main thread, which
///    calls `Rf_error_unchecked` — an R error. R `R_UnwindProtect`'s
///    `cleanup_handler` fires, sends `Err(msg)` to the worker so it can't
///    deadlock on `response_rx.recv()`, then `R_ContinueUnwind`s the longjmp.
///    (`_unchecked` is valid here: we're inside a `with_r_thread` callback —
///    MXL301-permitted context.)
/// 4. `R_ContinueUnwind` resumes the *original* R `Rf_error` longjmp on the
///    main thread, which lands directly at R's top level. So the surfaced
///    condition is a **bare `simpleError`** carrying the original message — the
///    macro's tagged-condition transport (`rust_error`) is bypassed entirely,
///    and the `run_on_worker` call below **never returns** on this path. This
///    answers the #733 open question on how the condition is layered. The
///    `match` below is therefore a defensive guard, not the live path.
///
/// The companion testthat test (`test-worker-longjmp.R`) asserts that the call
/// errors and that the condition is a `simpleError` carrying the original
/// message (and explicitly NOT a `rust_error`). Existing `test-worker.R`
/// coverage of the same raw-`Rf_error` shape only matched the message text.
#[miniextendr(noexport)]
pub fn gc_stress_with_r_thread_stop() -> SEXP {
    use miniextendr_api::worker::{run_on_worker, with_r_thread};

    let outcome = run_on_worker(|| {
        // On the worker: route back to the main thread and raise an R error.
        // The error longjmps through R_UnwindProtect's cleanup_handler, which
        // sends an Err to this worker job before R_ContinueUnwind resumes the
        // unwind. The closure never returns normally.
        with_r_thread::<_, ()>(|| unsafe {
            // Deliberately raise a raw R error to exercise the worker-job
            // R-longjmp path under test (#733). `_unchecked` is valid inside a
            // `with_r_thread` callback (MXL301-permitted context); the raw
            // `Rf_error` is the whole point of the fixture (MXL300).
            // mxl::allow(MXL300, MXL301)
            miniextendr_api::sys::Rf_error_unchecked(
                c"%s".as_ptr(),
                c"R error inside with_r_thread from a worker job".as_ptr(),
            );
        });
        // Unreachable: with_r_thread above diverges via the R longjmp.
        SEXP::nil()
    });

    // Defensive guard only: on the path under test, `R_ContinueUnwind` resumes
    // the original R longjmp straight to top level, so `run_on_worker` above
    // never returns and neither arm is reached. If the unwind behaviour ever
    // changes so the worker's Err *does* propagate, surface it loudly rather
    // than silently returning.
    match outcome {
        Err(msg) => panic!("{msg}"),
        Ok(_) => panic!(
            "gc_stress_with_r_thread_stop: worker job returned normally; \
             the R longjmp path was not exercised"
        ),
    }
}

// endregion

// region: worker re-usability after R longjmp + leak bound (#931)

/// A normal `run_on_worker` round-trip that also routes through `with_r_thread`,
/// returning a value the R side can assert on.
///
/// This is the *second* job in the #931 worker-re-usability test: the test first
/// triggers an R longjmp through a worker job via [`gc_stress_with_r_thread_stop`]
/// (which never returns — it resumes the original `Rf_error` straight to R's top
/// level, see #733 / PR #930), catches that error at top level, then calls this
/// fixture. If the worker thread were left poisoned by the unwind (its `recv()`
/// loop not re-armed, thread-locals not cleared, a wedged channel), this second
/// dispatch would hang, panic, or return the wrong value.
///
/// Shape: the fn returns `SEXP`, so the macro picks the **MainThread** strategy —
/// the body is the thread that drives `dispatch_to_worker`'s event loop. It
/// dispatches a closure to the worker, which routes a trivial computation back
/// to the main thread via `with_r_thread`, then returns the sum. We assert the
/// arithmetic is correct so a partially-corrupted worker (stale thread-local
/// channels from the prior aborted job) surfaces as a wrong answer rather than a
/// silent pass.
///
/// No arguments — also picked up by the fast `gctorture(TRUE)` no-arg sweep
/// (#430): it holds no SEXPs across allocations itself, but exercising the worker
/// dispatch under GC pressure is cheap insurance.
#[miniextendr(noexport)]
pub fn gc_stress_worker_roundtrip() -> i32 {
    use miniextendr_api::worker::{run_on_worker, with_r_thread};

    let outcome = run_on_worker(|| {
        // Route a trivial computation back to the main thread, mirroring the
        // structure of the longjmp fixture but on the success path. Summing the
        // per-step results proves the worker→main→worker hand-off is intact.
        let a = with_r_thread(|| 1_000i32);
        let b = with_r_thread(move || a + 234);
        a + b
    });

    match outcome {
        Ok(sum) => sum,
        Err(msg) => panic!("gc_stress_worker_roundtrip: worker dispatch failed: {msg}"),
    }
}

// endregion

// region: zero-copy &str argument borrow (#664)

/// Production helper for the `&str` zero-copy argument-borrow path (#664).
///
/// On the main-thread default, `#[miniextendr]` lowers a `&str` argument to a
/// *direct* zero-copy borrow over R's CHARSXP pool (no owning-`String` copy).
/// This fn just measures the borrow and echoes it back so the round-trip can be
/// asserted from the GC driver below.
#[miniextendr]
pub fn str_borrow_len(s: &str) -> i32 {
    // Touch the borrow non-trivially so the zero-copy view is actually read.
    s.chars().count() as i32
}

/// Drive the zero-copy `&str` argument borrow under GC pressure (#664).
///
/// `#[miniextendr] fn(s: &str)` now borrows R's CHARSXP data directly instead of
/// copying into a `String` first. The borrow is sound because the argument SEXP
/// is a live wrapper parameter for the whole `with_r_unwind_protect` call, and
/// its lifetime is tied to that scope (storing it past the call is a borrow-check
/// error). This fixture synthesises a STRSXP internally, takes a zero-copy
/// `&str` view over each element exactly as the generated wrapper does, and
/// verifies the bytes survive `gctorture(TRUE)` — a corrupted/freed borrow would
/// surface as wrong contents rather than a clean read.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_str_borrow() {
    let inputs = ["", "ascii", "héllo-wörld", "a longer string with spaces"];

    // Build a STRSXP and root it for the whole stress loop.
    let src = inputs.to_vec().into_sexp();
    let _src_guard = unsafe { miniextendr_api::OwnedProtect::new(src) };

    for _ in 0..64 {
        for (i, expected) in inputs.iter().enumerate() {
            // `string_elt_str` is the same zero-copy primitive the generated
            // `&str` argument path uses: it reads R's CHARSXP pool directly via
            // `R_CHAR` with no allocation. A freed/corrupted borrow under GC would
            // surface as wrong bytes rather than a clean read.
            let borrowed: &str = src.string_elt_str(i as isize).expect("non-NA element");
            assert_eq!(
                borrowed, *expected,
                "zero-copy &str borrow corrupted under GC at index {i}"
            );
        }
    }
}

// endregion

// region: serde RSerializer state machines (#943)

/// Drive every `ser.rs` serializer state machine under GC pressure.
///
/// The serializers hold intermediate element SEXPs in `Vec<SEXP>` fields
/// across the allocations performed by subsequent `serialize_*` calls — the
/// canonical "SEXP storage across allocations" shape. This fixture exercises
/// `SeqSerializer` (homogeneous + heterogeneous), `MapSerializer`,
/// `StructSerializer`, `TupleVariantSerializer`, `StructVariantSerializer`,
/// and the `make_tagged_list` newtype-variant path in one nested value.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn gc_stress_serde_ser() {
    use crate::serde::Serialize;
    use miniextendr_api::serde::to_r;

    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    enum StressEnum {
        Unit,
        Newtype(Vec<i32>),
        Tuple(i32, String, f64),
        Struct { a: Vec<f64>, b: Option<String> },
    }

    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct StressInner {
        ints: Vec<i32>,
        mixed: (i32, String, bool),
        map: BTreeMap<String, Vec<f64>>,
    }

    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct StressOuter {
        inner: Vec<StressInner>,
        variants: Vec<StressEnum>,
        label: Option<String>,
    }

    for round in 0..8 {
        let value = StressOuter {
            inner: (0..4)
                .map(|i| StressInner {
                    ints: (0..16).map(|j| i * 100 + j).collect(),
                    mixed: (i, format!("row-{i}-{round}"), i % 2 == 0),
                    map: (0..3)
                        .map(|k| (format!("k{k}"), vec![f64::from(k) * 0.5; 5]))
                        .collect(),
                })
                .collect(),
            variants: vec![
                StressEnum::Unit,
                StressEnum::Newtype((0..8).collect()),
                StressEnum::Tuple(round, format!("tag-{round}"), 2.5),
                StressEnum::Struct {
                    a: vec![1.0, 2.0, 3.0],
                    b: if round % 2 == 0 {
                        Some("present".to_string())
                    } else {
                        None
                    },
                },
            ],
            label: Some(format!("outer-{round}")),
        };

        let sexp = to_r(&value).expect("gc_stress_serde_ser: serialization failed");
        assert_eq!(
            sexp.type_of(),
            SEXPTYPE::VECSXP,
            "gc_stress_serde_ser: expected outer list"
        );
        assert_eq!(
            sexp.xlength(),
            3,
            "gc_stress_serde_ser: outer list must have 3 fields"
        );
        let inner = sexp.vector_elt(0);
        assert_eq!(
            inner.xlength(),
            4,
            "gc_stress_serde_ser: inner list must have 4 rows"
        );
    }
}

// endregion

// region: condition data payloads (#346)

/// Exercise `make_rust_condition_value_with_data` under GC pressure.
///
/// The condition-data path builds a fresh VECSXP + names STRSXP and
/// materialises each `RValue` field one at a time,
/// rooting every intermediate into the protected data list before the next
/// allocation — the canonical "SEXP storage across allocations" shape (#430).
/// This fixture drives the production code path directly with thirteen fields:
/// the eight base types (scalar + vector × i32/f64/bool/String) plus the #995
/// richer forms (NA-aware `Option`/`Vec<Option>`, the wide-integer ladder, a
/// `Debug`-stringified value, and a nested named list). It reads every field
/// back to verify integrity.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_condition_data() {
    use miniextendr_api::RValue;
    use miniextendr_api::condition::ConditionData;
    use miniextendr_api::error_value::make_rust_condition_value_with_data;

    for round in 0..4 {
        let data: ConditionData = vec![
            ("int".to_string(), RValue::from(round)),
            ("real".to_string(), RValue::from(1.5_f64)),
            ("flag".to_string(), RValue::from(true)),
            ("label".to_string(), RValue::from(format!("round {round}"))),
            ("ints".to_string(), RValue::from(vec![1_i32, 2, 3])),
            ("reals".to_string(), RValue::from(vec![0.5_f64, 2.5])),
            ("flags".to_string(), RValue::from(vec![true, false])),
            (
                "labels".to_string(),
                RValue::from(vec!["a".to_string(), "b".to_string()]),
            ),
            // #995 richer value types, expressed as RValue (#1050): NA-aware
            // scalars/vectors, wide integer (→ double, out of i32 range),
            // Debug-stringify, and a nested named list (which itself allocates a
            // fresh VECSXP + names under GC pressure).
            ("opt_int".to_string(), RValue::from(None::<i32>)),
            (
                "opt_int_vec".to_string(),
                RValue::from(vec![Some(1_i32), None, Some(3)]),
            ),
            ("long".to_string(), RValue::from(5_000_000_000_i64)),
            ("dbg".to_string(), RValue::debug(round..=round + 2)),
            (
                "nested".to_string(),
                RValue::List(vec![
                    (Some("x".to_string()), RValue::from(round)),
                    (Some("y".to_string()), RValue::from(vec![Some(7_i32), None])),
                ]),
            ),
        ];

        // SAFETY: gc_stress fixtures run on the R main thread under gctorture.
        let tagged = unsafe {
            make_rust_condition_value_with_data(
                "gc stress condition",
                miniextendr_api::error_value::kind::ERROR,
                Some("gc_stress_class"),
                None,
                Some(data),
            )
        };
        // The returned SEXP is unprotected — root it before the readback
        // (string_elt_str etc. do not allocate, but the next loop round does).
        let _guard = unsafe { miniextendr_api::gc_protect::OwnedProtect::new(tagged) };

        assert_eq!(tagged.len(), 5, "tagged condition value must have 5 slots");
        let data_list = tagged.vector_elt(4);
        assert_eq!(data_list.len(), 13, "data list must carry all 13 fields");

        // Scalars
        assert_eq!(data_list.vector_elt(0).integer_elt(0), round);
        assert_eq!(data_list.vector_elt(1).real_elt(0), 1.5);
        assert_eq!(data_list.vector_elt(2).logical_elt(0), 1);
        assert_eq!(
            data_list.vector_elt(3).string_elt_str(0),
            Some(format!("round {round}").as_str())
        );
        // Vectors
        let ints = data_list.vector_elt(4);
        assert_eq!(ints.len(), 3);
        assert_eq!(ints.integer_elt(2), 3);
        let reals = data_list.vector_elt(5);
        assert_eq!(reals.real_elt(1), 2.5);
        let flags = data_list.vector_elt(6);
        assert_eq!(flags.logical_elt(0), 1);
        assert_eq!(flags.logical_elt(1), 0);
        let labels = data_list.vector_elt(7);
        assert_eq!(labels.string_elt_str(1), Some("b"));

        // #995 richer value types
        // opt_int (None) → scalar NA_integer_ (== i32::MIN)
        assert_eq!(data_list.vector_elt(8).integer_elt(0), i32::MIN);
        // opt_int_vec → integer(3) with NA in the middle
        let opt_vec = data_list.vector_elt(9);
        assert_eq!(opt_vec.len(), 3);
        assert_eq!(opt_vec.integer_elt(0), 1);
        assert_eq!(opt_vec.integer_elt(1), i32::MIN);
        assert_eq!(opt_vec.integer_elt(2), 3);
        // long (5e9) → REALSXP (out of i32 range)
        assert_eq!(data_list.vector_elt(10).real_elt(0), 5_000_000_000.0);
        // dbg → character(1) of the Debug rendering
        assert_eq!(
            data_list.vector_elt(11).string_elt_str(0),
            Some(format!("{}..={}", round, round + 2).as_str())
        );
        // nested → VECSXP with named children x, y
        let nested = data_list.vector_elt(12);
        assert_eq!(nested.len(), 2);
        assert_eq!(nested.vector_elt(0).integer_elt(0), round);
        let nested_y = nested.vector_elt(1);
        assert_eq!(nested_y.len(), 2);
        assert_eq!(nested_y.integer_elt(0), 7);
        assert_eq!(nested_y.integer_elt(1), i32::MIN);
        let nested_names = nested.get_names();
        assert_eq!(nested_names.string_elt_str(0), Some("x"));
        assert_eq!(nested_names.string_elt_str(1), Some("y"));

        // Names round-trip
        let names = data_list.get_names();
        assert_eq!(names.string_elt_str(0), Some("int"));
        assert_eq!(names.string_elt_str(7), Some("labels"));
        assert_eq!(names.string_elt_str(12), Some("nested"));
    }
}

/// Drive the lowered `r!()` call-tree path under GC pressure.
///
/// The lowered path for a call like `r!(c(1L, 2L, 3L))` builds several
/// `SEXP` scalars and a nested `LANGSXP`, protecting each via
/// `ProtectScope::protect_raw` before passing them to `RCall`. This fixture
/// exercises that SEXP-storage sequence under `gctorture(TRUE)` to catch
/// any missing protect.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_r_macro_lowering() {
    // Each r!() call here goes through the lowered RCall path (call-shaped
    // top-level expression). We verify the result to confirm correct
    // evaluation, not just survival.
    let result = miniextendr_api::r!(c(1L, 2L, 3L)).expect("r!(c(1L, 2L, 3L)) should evaluate");
    let len = result.xlength();
    assert_eq!(len, 3, "c(1L, 2L, 3L) should have length 3");

    // Nested call: c(1L, c(2L, 3L)) — exercises nested LANGSXP protection.
    let result2 =
        miniextendr_api::r!(c(1L, c(2L, 3L))).expect("r!(c(1L, c(2L, 3L))) should evaluate");
    assert_eq!(result2.xlength(), 3, "nested c() should have length 3");
    assert_eq!(result2.integer_elt(0), 1);
    assert_eq!(result2.integer_elt(1), 2);
    assert_eq!(result2.integer_elt(2), 3);

    // String args — tests scalar_string_from_str protect.
    let pasted =
        miniextendr_api::r!(paste("gc", "stress")).expect("r!(paste(...)) should evaluate");
    // paste() returns a character(1); check it's non-nil.
    assert!(!pasted.is_nil(), "paste() result must not be nil");

    // Named arg — tests named_arg path.
    let seqed =
        miniextendr_api::r!(seq(1L, 6L, by = 2L)).expect("r!(seq(1L, 6L, by = 2L)) should work");
    assert_eq!(seqed.xlength(), 3, "seq(1,6,by=2) should have length 3");
}

// endregion

// region: ProtectScope FFI wrapper alloc helpers (#509/#510)

/// Exercise the new `ProtectScope` FFI wrapper methods under GC pressure.
///
/// Chains at least 6 of the new wrappers with intermediate allocations between
/// each call to ensure GC pressure between steps:
///   - `alloc_lang` (LANGSXP node)
///   - `cons` / `lcons` (pairlist / language cons cells)
///   - `lengthgets` (resize a vector)
///   - `alloc_array` (n-dimensional array with protected dims INTSXP)
///   - `mkchar_len_ce` + `mkchar_ce` (CHARSXP with specified encoding)
///
/// Each step forces a fresh allocation, so `gctorture(TRUE)` exercises the
/// protection of each intermediate result. Returns a trivial integer count
/// (the number of operations performed) so the testthat assertion is meaningful.
///
/// No arguments — suitable for the fast gctorture no-arg fixture sweep.
#[miniextendr(noexport)]
pub fn gc_stress_alloc_wrappers() -> i32 {
    use miniextendr_api::SEXPTYPE;
    use miniextendr_api::cetype_t::CE_UTF8;
    use miniextendr_api::gc_protect::ProtectScope;

    unsafe {
        let scope = ProtectScope::new();
        let mut ops = 0i32;

        // 1. alloc_lang — allocate a LANGSXP of length 3
        let _lang = scope.alloc_lang(3);
        ops += 1;

        // 2. cons — allocate a bare LISTSXP cons-cell node
        let _cell = scope.cons(SEXP::nil(), SEXP::nil());
        ops += 1;

        // 3. cons — build a 2-element pairlist from two protected nodes
        let head = scope.cons(SEXP::nil(), SEXP::nil());
        // Intermediate allocation between cons args — exercises GC pressure
        let _gap = scope.alloc_integer(4);
        let tail = scope.cons(SEXP::nil(), SEXP::nil());
        let _pair = scope.cons(head.get(), tail.get());
        ops += 1;

        // 4. lengthgets — resize a real vector (forces a fresh allocation)
        let base_vec = scope.alloc_real(5);
        // Intermediate allocation before lengthgets
        let _gap2 = scope.alloc_integer(2);
        let _resized = scope.lengthgets(base_vec.get(), 10);
        ops += 1;

        // 5. alloc_array — 2×3 integer array; dims INTSXP is built and
        //    protected inside the scope before Rf_allocArray is called
        let _arr = scope.alloc_array(SEXPTYPE::REALSXP, &[2, 3]);
        ops += 1;

        // 6. mkchar_len_ce — CHARSXP from a byte slice with explicit encoding
        let _char1 = scope.mkchar_len_ce(b"hello", CE_UTF8);
        ops += 1;

        // 7. mkchar_ce — CHARSXP from a &str with explicit encoding
        let _char2 = scope.mkchar_ce("world", CE_UTF8);
        ops += 1;

        // 8. alloc_3d_array — 2×3×4 raw array
        let _arr3d = scope.alloc_3d_array(SEXPTYPE::RAWSXP, 2, 3, 4);
        ops += 1;

        // 9. lcons — language cons cell
        let head2 = scope.cons(SEXP::nil(), SEXP::nil());
        let tail2 = scope.cons(SEXP::nil(), SEXP::nil());
        let _lc = scope.lcons(head2.get(), tail2.get());
        ops += 1;

        // 10. xlengthgets — long-vector resize path
        let long_base = scope.alloc_integer(3);
        let _long_resized = scope.xlengthgets(long_base.get(), 6);
        ops += 1;

        ops
    }
}

// endregion

// endregion

// region: AsNamedList deferred-value GC stress (issue #1030)

/// Exercise the `AsNamedList<Vec<(String, V)>>` deferred-conversion path under
/// GC pressure.
///
/// `AsNamedList::into_sexp` builds each value's `SEXP` and hands the lot to
/// `List::from_raw_pairs`. Before #1030 the values were collected unprotected
/// (`.map(|(k, v)| (k, v.into_sexp())).collect()`), so an earlier value sat
/// unrooted while a later value's `into_sexp()` allocated — a use-after-free
/// under `gctorture(TRUE)`. The fix wraps each `v.into_sexp()` in
/// `ProtectScope::protect_raw` so every sibling stays rooted across the next
/// allocation.
///
/// This fixture drives a *typed* heterogeneous list (the GC-safe replacement
/// for the old raw-`SEXP`-value fixture): a mix of character, integer and
/// double scalar values (each scalar allocation can fire GC), repeated enough
/// times that `from_raw_pairs`'s own `charsxp`/`SET_VECTOR_ELT` allocations
/// interleave with the value builds. Honest readback: every element is read
/// back through `VECTOR_ELT` and compared, so a collected value would surface
/// as a wrong/garbage element rather than silently passing.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_as_named_list_deferred() {
    use miniextendr_api::prelude::SexpExt as _;
    use miniextendr_api::{AsNamedList, RValue};

    let n = 24usize;

    // Build a typed heterogeneous list: each value is plain Rust data until
    // `AsNamedList::into_sexp` converts + protects it. Three scalar shapes so
    // STRSXP, INTSXP and REALSXP allocations all interleave.
    let pairs: Vec<(String, RValue)> = (0..n)
        .map(|i| {
            let value = match i % 3 {
                0 => RValue::from(format!("v-{i}")),
                1 => RValue::from(i as i32),
                _ => RValue::from(f64::from(i as i32) * 1.5),
            };
            (format!("k-{i}"), value)
        })
        .collect();

    // Drive the production conversion path (defers to `from_raw_pairs`).
    let sexp = AsNamedList(pairs).into_sexp();

    // Allocation-free honest readback: a collected value SEXP would be garbage.
    assert_eq!(sexp.len(), n, "AsNamedList produced wrong length");
    for i in 0..n {
        let elt = sexp.vector_elt(i as isize);
        match i % 3 {
            0 => {
                let s = elt.string_elt_str(0).expect("string value collected (NA)");
                assert_eq!(s, format!("v-{i}"), "string value collected/garbled");
            }
            1 => {
                assert_eq!(elt.integer_elt(0), i as i32, "int value collected/garbled");
            }
            _ => {
                assert_eq!(
                    elt.real_elt(0),
                    f64::from(i as i32) * 1.5,
                    "real value collected/garbled"
                );
            }
        }
    }
}

// endregion

// region: expression RCall builder (SEXP args held across allocations)

/// Drive the `RCall` builder + eval path under GC pressure. The builder holds
/// its argument SEXPs in a `Vec<(Option<CString>, SEXP)>` while later
/// arguments and the `build()` cons-chain allocate — exactly the
/// SEXP-storage-across-allocations shape #430 requires a no-arg fixture for.
/// Arguments are protected here per the builder's contract (caller keeps args
/// reachable); the fixture verifies `build()`/`eval()`'s internal PROTECT
/// discipline. Returns `paste("alpha", "beta", sep = "-")`, i.e. "alpha-beta".
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_expression_call() -> Result<SEXP, String> {
    use miniextendr_api::expression::RCall;

    unsafe {
        // Locals drop in reverse declaration order — LIFO, matching the
        // PROTECT/UNPROTECT stack discipline OwnedProtect relies on.
        let a = OwnedProtect::new(SEXP::scalar_string_from_str("alpha"));
        let b = OwnedProtect::new(SEXP::scalar_string_from_str("beta"));
        let sep = OwnedProtect::new(SEXP::scalar_string_from_str("-"));
        RCall::new("paste")
            .arg(a.get())
            .arg(b.get())
            .named_arg("sep", sep.get())
            .eval_base()
    }
}

// endregion

// region: RValue (#1050)

/// Stress `RValue`'s recursive `List` build + full SEXP→`RValue` round-trip
/// under GC pressure.
///
/// `RValue::List::into_sexp` builds a `VECSXP` and roots each recursively-built
/// child before the next allocates — the canonical "SEXP storage across
/// allocations" shape (#430). This fixture builds a nested, mixed, NA-bearing
/// tree (named + unnamed slots, every atomic variant, empty vectors, a nested
/// list), converts it to a `SEXP`, then decodes it back with
/// `RValue::try_from_sexp` and asserts the structure survived. The interleaved
/// `charsxp` / `SET_VECTOR_ELT` allocations of the list build exercise the
/// protect discipline; a collected child would surface as a wrong/garbage slot.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_rvalue_roundtrip() {
    use miniextendr_api::from_r::TryFromSexp as _;
    use miniextendr_api::{IntoR, RValue};

    for _round in 0..4 {
        let tree = RValue::List(vec![
            (Some("lgl".into()), RValue::Logical(vec![Some(true), None])),
            (Some("int".into()), RValue::Integer(vec![Some(7), None])),
            (Some("dbl".into()), RValue::Double(vec![1.5, 2.5])),
            (
                Some("chr".into()),
                RValue::Character(vec![Some("hi".into()), None]),
            ),
            (Some("raw".into()), RValue::Raw(vec![1, 2, 3])),
            (Some("nul".into()), RValue::Null),
            (Some("empty".into()), RValue::Integer(vec![])),
            (
                Some("nested".into()),
                RValue::List(vec![
                    (None, RValue::Integer(vec![Some(1)])),
                    (
                        Some("inner".into()),
                        RValue::Character(vec![Some("x".into())]),
                    ),
                ]),
            ),
        ]);

        let sexp = tree.into_sexp();
        let _guard = unsafe { miniextendr_api::gc_protect::OwnedProtect::new(sexp) };

        let decoded = RValue::try_from_sexp(sexp).expect("RValue must round-trip");
        let RValue::List(pairs) = decoded else {
            panic!("expected List, got {decoded:?}");
        };
        assert_eq!(pairs.len(), 8, "top-level list lost slots");
        assert!(matches!(&pairs[0].1, RValue::Logical(v) if v == &[Some(true), None]));
        assert!(matches!(&pairs[1].1, RValue::Integer(v) if v == &[Some(7), None]));
        assert!(matches!(&pairs[3].1, RValue::Character(v) if v == &[Some("hi".into()), None]));
        assert!(matches!(&pairs[4].1, RValue::Raw(v) if v == &[1, 2, 3]));
        assert!(matches!(&pairs[5].1, RValue::Null));
        assert!(matches!(&pairs[6].1, RValue::Integer(v) if v.is_empty()));
        // Nested list: unnamed first slot (None), named second slot.
        let RValue::List(inner) = &pairs[7].1 else {
            panic!("expected nested List, got {:?}", pairs[7].1);
        };
        assert_eq!(inner[0].0, None, "unnamed nested slot lost its None name");
        assert_eq!(inner[1].0.as_deref(), Some("inner"));
    }
}

// endregion
