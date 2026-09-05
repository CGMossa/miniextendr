use super::*;

fn parse_inputs(s: &str) -> syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma> {
    let signature: syn::Signature = syn::parse_str(&format!("fn test({})", s)).unwrap();
    signature.inputs
}

#[test]
fn test_normalize_arg_ident() {
    // Leading underscores are stripped
    let ident = syn::Ident::new("_x", proc_macro2::Span::call_site());
    assert_eq!(normalize_r_arg_ident(&ident).to_string(), "x");

    let ident = syn::Ident::new("__private", proc_macro2::Span::call_site());
    assert_eq!(normalize_r_arg_ident(&ident).to_string(), "private");

    let ident = syn::Ident::new("value", proc_macro2::Span::call_site());
    assert_eq!(normalize_r_arg_ident(&ident).to_string(), "value");
}

#[test]
fn test_basic_formals() {
    let inputs = parse_inputs("x: i32, y: f64");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(builder.build_formals(), "x, y");
}

#[test]
fn test_unit_type_default() {
    // `_unused` becomes `unused` (underscore stripped), unit type gets NULL default
    let inputs = parse_inputs("x: i32, _unused: ()");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(builder.build_formals(), "x, unused = NULL");
}

#[test]
fn test_dots() {
    let inputs = parse_inputs("x: i32, _dots: &Dots");
    let builder = RArgumentBuilder::new(&inputs).with_dots(None);
    assert_eq!(builder.build_formals(), "x, ...");
    assert_eq!(builder.build_call_args(), "x, list(...)");
}

#[test]
fn test_trailing_dots_auto_detected() {
    let inputs = parse_inputs("x: i32, dots: &Dots");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(builder.build_formals(), "x, ...");
    assert_eq!(builder.build_call_args(), "x, list(...)");
}

#[test]
fn test_named_dots() {
    // Note: In R, `...` cannot have a name/default in formals.
    // The named_dots is only used on Rust side. R always uses plain `...`
    let inputs = parse_inputs("x: i32, _dots: &Dots");
    let builder = RArgumentBuilder::new(&inputs).with_dots(Some("args".to_string()));
    assert_eq!(builder.build_formals(), "x, ...");
    assert_eq!(builder.build_call_args(), "x, list(...)");
}

#[test]
fn test_skip_first() {
    let inputs = parse_inputs("&self, x: i32, y: f64");
    let builder = RArgumentBuilder::new(&inputs).skip_first();
    assert_eq!(builder.build_formals(), "x, y");
    assert_eq!(builder.build_call_args(), "x, y");
}

#[test]
fn test_underscore_normalization() {
    // Leading underscores are stripped in R (Rust convention for unused params)
    let inputs = parse_inputs("_x: i32, __private: String");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(builder.build_formals(), "x, private");
}

// DotCallBuilder tests
#[test]
fn test_dot_call_no_args() {
    let call = DotCallBuilder::new("C_Counter__new").build();
    assert_eq!(call, ".Call(C_Counter__new, .call = match.call())");
}

#[test]
fn test_dot_call_with_self() {
    let call = DotCallBuilder::new("C_Counter__value")
        .with_self("self")
        .build();
    assert_eq!(call, ".Call(C_Counter__value, .call = match.call(), self)");
}

#[test]
fn test_dot_call_with_self_and_args() {
    let call = DotCallBuilder::new("C_Counter__add")
        .with_self("x")
        .with_args(&["n"])
        .build();
    assert_eq!(call, ".Call(C_Counter__add, .call = match.call(), x, n)");
}

#[test]
fn test_dot_call_static_with_args() {
    let call = DotCallBuilder::new("C_Counter__from_parts")
        .with_args(&["a", "b", "c"])
        .build();
    assert_eq!(
        call,
        ".Call(C_Counter__from_parts, .call = match.call(), a, b, c)"
    );
}

#[test]
fn test_dot_call_with_args_str_empty_skips_args() {
    let call = DotCallBuilder::new("C_Counter__new")
        .with_args_str("")
        .build();
    assert_eq!(call, ".Call(C_Counter__new, .call = match.call())");
}

#[test]
fn test_dot_call_with_args_str_passes_through() {
    let call = DotCallBuilder::new("C_Counter__update")
        .with_self("self")
        .with_args_str("step, verbose")
        .build();
    assert_eq!(
        call,
        ".Call(C_Counter__update, .call = match.call(), self, step, verbose)"
    );
}

// null_call_attribution tests
#[test]
fn test_dot_call_null_call_no_args() {
    let call = DotCallBuilder::new("C_Type__finalize")
        .null_call_attribution()
        .with_self("private$.ptr")
        .build();
    assert_eq!(call, ".Call(C_Type__finalize, .call = NULL, private$.ptr)");
}

#[test]
fn test_dot_call_null_call_with_args() {
    let call = DotCallBuilder::new("C_Type__deep_clone")
        .null_call_attribution()
        .with_self("private$.ptr")
        .with_args(&["name", "value"])
        .build();
    assert_eq!(
        call,
        ".Call(C_Type__deep_clone, .call = NULL, private$.ptr, name, value)"
    );
}

#[test]
fn test_dot_call_null_call_validator() {
    let call = DotCallBuilder::new("C_Type__validate_prop")
        .null_call_attribution()
        .with_args(&["value"])
        .build();
    assert_eq!(call, ".Call(C_Type__validate_prop, .call = NULL, value)");
}

// RoxygenBuilder tests
#[test]
fn test_roxygen_basic() {
    let tags = RoxygenBuilder::new()
        .name("Counter$increment")
        .rdname("Counter")
        .export()
        .build();
    assert_eq!(
        tags,
        vec![
            "#' @name Counter$increment",
            "#' @rdname Counter",
            "#' @export"
        ]
    );
}

#[test]
fn test_roxygen_s3_method() {
    let tags = RoxygenBuilder::new()
        .name("value")
        .source("Generated by miniextendr from `impl Counter for MyType`")
        .method("value", "MyType")
        .export()
        .build();
    assert_eq!(
        tags,
        vec![
            "#' @name value",
            "#' @source Generated by miniextendr from `impl Counter for MyType`",
            "#' @method value MyType",
            "#' @export"
        ]
    );
}

#[test]
fn test_roxygen_s4_method() {
    let tags = RoxygenBuilder::new()
        .name("s4_trait_Counter_value")
        .source("Generated by miniextendr")
        .export_method("s4_trait_Counter_value")
        .build();
    assert_eq!(
        tags,
        vec![
            "#' @name s4_trait_Counter_value",
            "#' @source Generated by miniextendr",
            "#' @exportMethod s4_trait_Counter_value"
        ]
    );
}

// Missing<T> tests
#[test]
fn test_is_missing_type() {
    let inputs = parse_inputs("x: Missing<i32>");
    let arg = inputs.first().unwrap();
    if let syn::FnArg::Typed(pat_type) = arg {
        assert!(is_missing_type(&pat_type.ty));
    } else {
        panic!("Expected typed argument");
    }
}

#[test]
fn test_is_not_missing_type() {
    let inputs = parse_inputs("x: Option<i32>");
    let arg = inputs.first().unwrap();
    if let syn::FnArg::Typed(pat_type) = arg {
        assert!(!is_missing_type(&pat_type.ty));
    } else {
        panic!("Expected typed argument");
    }
}

#[test]
fn test_missing_type_call_args_inline_sentinel() {
    // The R_MissingArg sentinel must be produced at the argument position —
    // a prelude binding of the sentinel errors on symbol lookup.
    let inputs = parse_inputs("x: i32, y: Missing<f64>, z: String");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(
        builder.build_call_args(),
        "x, if (missing(y)) quote(expr=) else y, z"
    );
}

#[test]
fn test_multiple_missing_type_call_args() {
    let inputs = parse_inputs("a: Missing<i32>, b: f64, c: Missing<String>");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(
        builder.build_call_args(),
        "if (missing(a)) quote(expr=) else a, b, if (missing(c)) quote(expr=) else c"
    );
}

#[test]
fn test_missing_type_formals_clean_signature() {
    // Missing<T> params without user defaults appear as bare formals
    let inputs = parse_inputs("x: i32, y: Missing<f64>");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(builder.build_formals(), "x, y");
}

// region: Insta snapshot tests for builder output stability

#[test]
fn snapshot_dot_call_variations() {
    let mut output = String::new();

    output.push_str("# No args\n");
    output.push_str(&DotCallBuilder::new("C_my_fn").build());
    output.push_str("\n\n# Self only\n");
    output.push_str(
        &DotCallBuilder::new("C_Counter__get")
            .with_self("self")
            .build(),
    );
    output.push_str("\n\n# Self + args\n");
    output.push_str(
        &DotCallBuilder::new("C_Counter__add")
            .with_self("x")
            .with_args(&["n", "verbose"])
            .build(),
    );
    output.push_str("\n\n# Static with args\n");
    output.push_str(
        &DotCallBuilder::new("C_Counter__from_parts")
            .with_args(&["a", "b", "c"])
            .build(),
    );

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_roxygen_builder_variations() {
    let mut output = String::new();

    output.push_str("# Basic export\n");
    let tags = RoxygenBuilder::new()
        .name("Counter$increment")
        .rdname("Counter")
        .export()
        .build();
    output.push_str(&tags.join("\n"));

    output.push_str("\n\n# S3 method\n");
    let tags = RoxygenBuilder::new()
        .name("get.Counter")
        .source("Generated by miniextendr from `impl Counter`")
        .method("get", "Counter")
        .export()
        .build();
    output.push_str(&tags.join("\n"));

    output.push_str("\n\n# S4 method\n");
    let tags = RoxygenBuilder::new()
        .name("s4_trait_Counter_value")
        .source("Generated by miniextendr")
        .export_method("s4_trait_Counter_value")
        .build();
    output.push_str(&tags.join("\n"));

    output.push_str("\n\n# Title + description\n");
    let tags = RoxygenBuilder::new()
        .title("Widget constructor")
        .description("Creates a new Widget with default settings.")
        .name("Widget")
        .export()
        .build();
    output.push_str(&tags.join("\n"));

    output.push_str("\n\n# Custom tags\n");
    let tags = RoxygenBuilder::new()
        .name("my_fn")
        .custom("@param x A numeric value")
        .custom("@return The squared value")
        .export()
        .build();
    output.push_str(&tags.join("\n"));

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_formals_and_call_args() {
    let mut output = String::new();

    // Basic scalar args
    output.push_str("# Basic scalars\n");
    let inputs = parse_inputs("x: i32, y: f64, name: String");
    let builder = RArgumentBuilder::new(&inputs);
    output.push_str(&format!("formals: {}\n", builder.build_formals()));
    output.push_str(&format!("call_args: {}\n", builder.build_call_args()));

    // With unit type default
    output.push_str("\n# Unit type default\n");
    let inputs = parse_inputs("x: i32, _unused: ()");
    let builder = RArgumentBuilder::new(&inputs);
    output.push_str(&format!("formals: {}\n", builder.build_formals()));
    output.push_str(&format!("call_args: {}\n", builder.build_call_args()));

    // With dots
    output.push_str("\n# With dots\n");
    let inputs = parse_inputs("x: i32, _dots: &Dots");
    let builder = RArgumentBuilder::new(&inputs).with_dots(None);
    output.push_str(&format!("formals: {}\n", builder.build_formals()));
    output.push_str(&format!("call_args: {}\n", builder.build_call_args()));

    // With skip_first (method receiver)
    output.push_str("\n# Skip first (method)\n");
    let inputs = parse_inputs("&self, x: i32, y: f64");
    let builder = RArgumentBuilder::new(&inputs).skip_first();
    output.push_str(&format!("formals: {}\n", builder.build_formals()));
    output.push_str(&format!("call_args: {}\n", builder.build_call_args()));

    // With user defaults
    output.push_str("\n# User defaults\n");
    let inputs = parse_inputs("x: i32, step: i32, verbose: bool");
    let mut defaults = std::collections::HashMap::new();
    defaults.insert("step".to_string(), "1L".to_string());
    defaults.insert("verbose".to_string(), "FALSE".to_string());
    let builder = RArgumentBuilder::new(&inputs).with_defaults(defaults);
    output.push_str(&format!("formals: {}\n", builder.build_formals()));
    output.push_str(&format!("call_args: {}\n", builder.build_call_args()));

    // Missing<T> - clean formals (no quote(expr=) in signature); the sentinel
    // forwarding is inline in call_args
    output.push_str("\n# Missing<T> clean formals\n");
    let inputs = parse_inputs("x: i32, y: Missing<f64>, z: Missing<String>");
    let builder = RArgumentBuilder::new(&inputs);
    output.push_str(&format!("formals: {}\n", builder.build_formals()));
    output.push_str(&format!("call_args: {}\n", builder.build_call_args()));

    insta::assert_snapshot!(output);
}
// endregion

#[test]
fn call_attribution_strings() {
    assert_eq!(CallAttribution::default(), CallAttribution::Wrapper);
    assert_eq!(
        CallAttribution::Wrapper.dot_call_arg(),
        ".call = match.call()"
    );
    assert_eq!(CallAttribution::None.dot_call_arg(), ".call = NULL");
    assert_eq!(CallAttribution::Caller.dot_call_arg(), ".call = .mx_call");
    assert_eq!(CallAttribution::Wrapper.raise_default(), "sys.call()");
    assert_eq!(CallAttribution::None.raise_default(), "sys.call()");
    assert_eq!(CallAttribution::Caller.raise_default(), ".mx_call");
    assert_eq!(CallAttribution::Wrapper.prelude("  "), "");
    assert_eq!(CallAttribution::None.prelude("  "), "");
    let prelude = CallAttribution::Caller.prelude("  ");
    assert_eq!(
        prelude,
        ".mx_parent <- sys.parent()\n  \
         .mx_def <- if (.mx_parent > 0L) sys.function(.mx_parent)\n  \
         .mx_pc <- if (.mx_parent > 0L) sys.call(.mx_parent)\n  \
         .mx_call <- if (typeof(.mx_def) == \"closure\") match.call(.mx_def, .mx_pc) else match.call()\n  "
    );
}
