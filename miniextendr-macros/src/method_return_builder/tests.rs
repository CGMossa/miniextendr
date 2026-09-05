use super::*;

#[test]
fn test_direct_return() {
    let builder = MethodReturnBuilder::new(".Call(C_Counter__value, self)".to_string())
        .with_strategy(ReturnStrategy::Direct);

    let lines = builder.build();
    // .val <- <call>; if (inherits(.val, "rust_condition_value") && ...) return(...); .val
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains(".val <- .Call(C_Counter__value, self)"));
    assert!(lines[1].contains("inherits(.val, \"rust_condition_value\")"));
    assert_eq!(lines[2], "  .val");
}

#[test]
fn test_return_self() {
    let builder = MethodReturnBuilder::new(".Call(C_Counter__new)".to_string())
        .with_strategy(ReturnStrategy::ReturnSelf)
        .with_class_name("Counter".to_string());

    let lines = builder.build();
    // .val <- <call>; if (...) return(...); class(.val) <- "Counter"; .val
    assert_eq!(lines.len(), 4);
    assert!(lines[0].contains(".val <- .Call(C_Counter__new)"));
    assert!(lines[1].contains("inherits(.val, \"rust_condition_value\")"));
    assert!(lines[2].contains("class(.val) <- \"Counter\""));
    assert_eq!(lines[3], "  .val");
}

#[test]
fn test_chainable_mutation() {
    let builder = MethodReturnBuilder::new(".Call(C_Counter__inc, self)".to_string())
        .with_strategy(ReturnStrategy::ChainableMutation);

    let lines = builder.build();
    // .val <- <call>; if (...) return(...); self
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains(".val <- .Call(C_Counter__inc, self)"));
    assert!(lines[1].contains("inherits(.val, \"rust_condition_value\")"));
    assert_eq!(lines[2], "  self");
}

#[test]
fn test_s7_style() {
    let builder = MethodReturnBuilder::new(".Call(C_Counter__new)".to_string())
        .with_strategy(ReturnStrategy::ReturnSelf)
        .with_class_name("Counter".to_string());

    let inline = builder.build_s7_inline();
    // Inline block wraps in `{}` and routes the call through `.val` + the
    // condition-check guard before constructing the S7 object.
    assert!(inline.starts_with('{'));
    assert!(inline.contains(".val <- .Call(C_Counter__new)"));
    assert!(inline.contains("inherits(.val, \"rust_condition_value\")"));
    assert!(inline.contains("Counter(.ptr = .val)"));
}

/// The list-shaped cross-class strategy (#1284) emits the distinct
/// `.__MX_WRAP_LIST_RETURN_` marker — never the scalar `.__MX_WRAP_RETURN_`
/// prefix, which the scalar write-time resolver would consume and rewrite to
/// the bare expression before the list resolver ran.
#[test]
fn test_return_other_class_list_marker() {
    let builder = MethodReturnBuilder::new(".Call(C_Plan__build_many, self)".to_string())
        .with_strategy(ReturnStrategy::ReturnOtherClassList)
        .with_return_class("Board".to_string());

    let lines = builder.build();
    // .val <- <call>; if (...) return(...); .__MX_WRAP_LIST_RETURN_Board__(.val)
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains(".val <- .Call(C_Plan__build_many, self)"));
    assert!(lines[1].contains("inherits(.val, \"rust_condition_value\")"));
    assert_eq!(lines[2], "  .__MX_WRAP_LIST_RETURN_Board__(.val)");
    assert!(
        !lines[2].contains(".__MX_WRAP_RETURN_Board__"),
        "list strategy must not reuse the scalar marker prefix"
    );

    // Inline (S7/S4) variants carry the same marker.
    let s7 = builder.build_s7_inline();
    assert!(s7.contains(".__MX_WRAP_LIST_RETURN_Board__(.val)"));
    let s4 = builder.build_s4_inline();
    assert!(s4.contains(".__MX_WRAP_LIST_RETURN_Board__(.val)"));
}

#[test]
fn standalone_body_call_default_is_configurable() {
    let default = crate::method_return_builder::standalone_body(".Call(C_f, x)", ".val", "  ");
    assert!(
        default.contains(".miniextendr_raise_condition(.val, sys.call())"),
        "{default}"
    );
    let caller = crate::method_return_builder::standalone_body_with_call_default(
        ".Call(C_f, x)",
        ".val",
        "  ",
        ".mx_call",
    );
    assert!(
        caller.contains(".miniextendr_raise_condition(.val, .mx_call))"),
        "{caller}"
    );
    assert!(!caller.contains("sys.call()"), "{caller}");
}
