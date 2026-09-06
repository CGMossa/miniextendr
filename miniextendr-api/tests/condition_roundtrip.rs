// Round-trip test for `from_tagged_sexp` slot [4] (structured condition data).
//
// These tests exercise the full Rust→SEXP→Rust path for structured condition
// data through `make_rust_condition_value_with_data` + `from_tagged_sexp`.
// They require an R runtime and are guarded by `r_test_utils::with_r_thread`.

mod r_test_utils;

#[test]
fn condition_data_roundtrip_scalars_and_vecs() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::RValue;
        use miniextendr_api::condition::{ConditionData, RCondition};
        use miniextendr_api::error_value::make_rust_condition_value_with_data;

        // Build a ConditionData with each supported scalar type.
        let fields: ConditionData = vec![
            ("field_a".to_string(), RValue::Integer(vec![Some(42)])),
            (
                "field_b".to_string(),
                RValue::Character(vec![Some("hello".to_string())]),
            ),
            ("field_c".to_string(), RValue::Double(vec![1.23])),
            ("field_d".to_string(), RValue::Logical(vec![Some(true)])),
            (
                "field_e".to_string(),
                RValue::Integer(vec![Some(1), Some(2), Some(3)]),
            ),
            ("field_f".to_string(), RValue::Double(vec![0.5, 1.5])),
            (
                "field_g".to_string(),
                RValue::Character(vec![Some("x".to_string()), Some("y".to_string())]),
            ),
        ];

        let sexp = make_rust_condition_value_with_data(
            "test message",
            miniextendr_api::error_value::kind::ERROR,
            &["my_test_class".to_string()],
            None,
            Some(fields),
        );

        let cond = RCondition::from_tagged_sexp(sexp)
            .expect("from_tagged_sexp should return Some for a valid tagged SEXP");

        match cond {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "test message");
                assert_eq!(class, vec!["my_test_class"]);
                let data = data.expect("data must be Some after round-trip");
                assert_eq!(data.len(), 7);
                assert_eq!(data[0].0, "field_a");
                assert!(matches!(&data[0].1, RValue::Integer(v) if v == &[Some(42)]));
                assert_eq!(data[1].0, "field_b");
                assert!(
                    matches!(&data[1].1, RValue::Character(v) if v == &[Some("hello".to_string())])
                );
                assert_eq!(data[2].0, "field_c");
                // f64 comparison via debug repr (RValue has no PartialEq for f64)
                assert!(
                    matches!(&data[2].1, RValue::Double(v) if v.len() == 1 && (v[0] - 1.23_f64).abs() < 1e-10),
                    "expected Double([1.23]), got {:?}",
                    data[2].1
                );
                assert_eq!(data[3].0, "field_d");
                assert!(matches!(&data[3].1, RValue::Logical(v) if v == &[Some(true)]));
                assert_eq!(data[4].0, "field_e");
                assert!(
                    matches!(&data[4].1, RValue::Integer(v) if v == &[Some(1), Some(2), Some(3)])
                );
                assert_eq!(data[5].0, "field_f");
                assert!(
                    matches!(&data[5].1, RValue::Double(v) if v.len() == 2),
                    "expected Double of len 2, got {:?}",
                    data[5].1
                );
                assert_eq!(data[6].0, "field_g");
                assert!(
                    matches!(&data[6].1, RValue::Character(v) if v[0].as_deref() == Some("x") && v[1].as_deref() == Some("y"))
                );
            }
            other => panic!("wrong RCondition variant: {other:?}"),
        }
    });
}

#[test]
fn condition_data_roundtrip_no_data() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::condition::RCondition;
        use miniextendr_api::error_value::make_rust_condition_value_with_data;

        let sexp = make_rust_condition_value_with_data(
            "plain error",
            miniextendr_api::error_value::kind::ERROR,
            &[],
            None,
            None, // no data
        );

        let cond = RCondition::from_tagged_sexp(sexp).expect("must parse");
        match cond {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "plain error");
                assert!(class.is_empty());
                assert!(data.is_none(), "data should be None when not provided");
            }
            other => panic!("wrong variant: {other:?}"),
        }
    });
}

#[test]
fn condition_data_roundtrip_na_field_survives() {
    // RValue is NA-aware: a field whose element is NA round-trips faithfully as
    // `Integer([None])` rather than being dropped (the pre-#1050 behavior). This
    // pins the NA fidelity that folding in `RValue` delivers.
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::RValue;
        use miniextendr_api::condition::{ConditionData, RCondition};
        use miniextendr_api::error_value::make_rust_condition_value_with_data;

        // Build a ConditionData with one NA-bearing field alongside a valid one.
        let fields: ConditionData = vec![
            ("good_field".to_string(), RValue::Integer(vec![Some(7)])),
            // NA_integer_ — materialised as integer(1) containing NA, decoded as None.
            ("na_field".to_string(), RValue::Integer(vec![None])),
        ];

        let sexp = make_rust_condition_value_with_data(
            "error with na field",
            miniextendr_api::error_value::kind::ERROR,
            &[],
            None,
            Some(fields),
        );

        let cond = RCondition::from_tagged_sexp(sexp).expect("must parse");
        match cond {
            RCondition::Error { data, .. } => {
                let data = data.expect("data must be Some");
                assert_eq!(
                    data.len(),
                    2,
                    "both fields survive (NA-aware); got: {data:?}"
                );
                assert_eq!(data[0].0, "good_field");
                assert!(matches!(&data[0].1, RValue::Integer(v) if v == &[Some(7)]));
                assert_eq!(data[1].0, "na_field");
                assert!(matches!(&data[1].1, RValue::Integer(v) if v == &[None]));
            }
            other => panic!("wrong variant: {other:?}"),
        }
    });
}

#[test]
fn condition_data_roundtrip_warning_carries_data() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::RValue;
        use miniextendr_api::condition::{ConditionData, RCondition};
        use miniextendr_api::error_value::make_rust_condition_value_with_data;

        let fields: ConditionData = vec![
            ("dropped".to_string(), RValue::Integer(vec![Some(3)])),
            (
                "reason".to_string(),
                RValue::Character(vec![Some("truncated".to_string())]),
            ),
        ];

        let sexp = make_rust_condition_value_with_data(
            "truncated 3 rows",
            miniextendr_api::error_value::kind::WARNING,
            &["truncation_warning".to_string()],
            None,
            Some(fields),
        );

        let cond = RCondition::from_tagged_sexp(sexp).expect("must parse");
        match cond {
            RCondition::Warning {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "truncated 3 rows");
                assert_eq!(class, vec!["truncation_warning"]);
                let data = data.expect("data must survive for Warning too");
                assert_eq!(data.len(), 2);
                assert!(matches!(&data[0].1, RValue::Integer(v) if v == &[Some(3)]));
                assert!(
                    matches!(&data[1].1, RValue::Character(v) if v == &[Some("truncated".to_string())])
                );
            }
            other => panic!("wrong variant: {other:?}"),
        }
    });
}

/// Class vectors (#1435): every element of the class slot survives the
/// SEXP round trip, in order (member first, family second).
#[test]
fn condition_class_vector_roundtrip() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::condition::RCondition;
        use miniextendr_api::error_value::make_rust_condition_value_with_data;

        let classes = vec![
            "pkg_error_missing_field".to_string(),
            "pkg_error".to_string(),
        ];
        let sexp = make_rust_condition_value_with_data(
            "layered",
            miniextendr_api::error_value::kind::ERROR,
            &classes,
            None,
            None,
        );
        match RCondition::from_tagged_sexp(sexp).expect("tagged") {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "layered");
                assert_eq!(class, classes);
                assert!(data.is_none());
            }
            other => panic!("wrong variant: {other:?}"),
        }
    });
}

/// `result_err_condition_value` (#1434): the probed parts of a classed error
/// land in the tagged value with `kind = "result_err"`, class and data intact.
#[test]
fn result_err_parts_roundtrip() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::RValue;
        use miniextendr_api::condition::{RCondition, RError};
        use miniextendr_api::error_value::result_err_condition_value;

        let err = RError::new("too large")
            .class(["pkg_too_large", "pkg_error"])
            .data("rule", 3)
            .data("max", 100.0);
        let parts = miniextendr_api::__mx_result_err_parts!(err, "pkg_error");
        let sexp = result_err_condition_value(parts, None);
        match RCondition::from_tagged_sexp(sexp).expect("tagged") {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "too large");
                assert_eq!(class, vec!["pkg_too_large", "pkg_error"]);
                let data = data.expect("data");
                assert_eq!(data[0].0, "rule");
                assert!(matches!(&data[0].1, RValue::Integer(v) if v == &[Some(3)]));
                assert_eq!(data[1].0, "max");
            }
            other => panic!("wrong variant: {other:?}"),
        }

        // A plain String error: under the `serde` feature it takes the serde
        // arm (family class, text unquoted); otherwise the Debug fallback keeps
        // the historical rendering.
        let plain = String::from("boom");
        let parts = miniextendr_api::__mx_result_err_parts!(plain, "pkg_error");
        if cfg!(feature = "serde") {
            assert_eq!(parts.message, "boom");
            assert_eq!(parts.class, ["pkg_error"]);
        } else {
            assert_eq!(parts.message, "\"boom\"");
            assert!(parts.class.is_empty());
        }
        assert!(parts.data.is_none());
    });
}
