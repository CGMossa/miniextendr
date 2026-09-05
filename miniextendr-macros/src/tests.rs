use crate::miniextendr_fn::{
    MiniextendrFnAttrs, MiniextendrFunctionParsed, is_miniextendr_coerce_attr,
};
#[test]
fn parsed_fn_rewrites_unnamed_dots_to_dots_arg() {
    let parsed: MiniextendrFunctionParsed =
        syn::parse2(quote::quote! { fn f(a: i32, ...) -> i32 { a } }).unwrap();

    assert!(parsed.has_dots());
    assert!(parsed.named_dots().is_none());
    assert!(parsed.item().sig.variadic.is_none());

    let last = parsed.inputs().last().unwrap();
    let syn::FnArg::Typed(pat_type) = last else {
        panic!("expected a typed arg");
    };
    let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
        panic!("expected ident pattern");
    };
    assert_eq!(pat_ident.ident, "__miniextendr_dots");

    let syn::Type::Reference(r) = pat_type.ty.as_ref() else {
        panic!("expected reference type");
    };
    let syn::Type::Path(tp) = r.elem.as_ref() else {
        panic!("expected path type");
    };
    assert_eq!(
        tp.path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>(),
        vec!["miniextendr_api", "dots", "Dots"]
    );
}

#[test]
fn parsed_fn_rewrites_named_dots_to_named_dots_arg() {
    let parsed: MiniextendrFunctionParsed =
        syn::parse2(quote::quote! { fn f(a: i32, dots: ...) -> i32 { a } }).unwrap();

    assert!(parsed.has_dots());
    assert_eq!(parsed.named_dots().unwrap(), "dots");

    let last = parsed.inputs().last().unwrap();
    let syn::FnArg::Typed(pat_type) = last else {
        panic!("expected a typed arg");
    };
    let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
        panic!("expected ident pattern");
    };
    assert_eq!(pat_ident.ident, "dots");
}

#[test]
fn parsed_fn_rewrites_wildcards_and_tracks_per_param_coerce() {
    let parsed: MiniextendrFunctionParsed = syn::parse2(quote::quote! {
        fn f(#[miniextendr(coerce)] _: u16, _: i32) {}
    })
    .unwrap();

    assert!(parsed.has_coerce_attr("__unused0"));
    assert!(!parsed.has_coerce_attr("__unused1"));

    let args: Vec<&syn::FnArg> = parsed.inputs().iter().collect();
    let syn::FnArg::Typed(first) = args[0] else {
        panic!("expected typed arg");
    };
    let syn::Pat::Ident(first_ident) = first.pat.as_ref() else {
        panic!("expected ident pattern");
    };
    assert_eq!(first_ident.ident, "__unused0");
    assert!(!first.attrs.iter().any(is_miniextendr_coerce_attr));
}

#[test]
fn parsed_fn_errors_on_unnamed_dots_conflicting_with_dots_arg_name() {
    let err = syn::parse2::<MiniextendrFunctionParsed>(quote::quote! {
        fn f(__miniextendr_dots: i32, ...) {}
    })
    .err()
    .unwrap();

    assert!(
        err.to_string()
            .contains("conflicts with implicit dots parameter")
    );
}

#[test]
fn parsed_fn_errors_on_non_ident_dots_pattern() {
    let err = syn::parse2::<MiniextendrFunctionParsed>(quote::quote! {
        fn f((a, b): ...) {}
    })
    .err()
    .unwrap();

    assert!(
        err.to_string()
            .contains("variadic pattern must be a simple identifier")
    );
}

#[test]
fn miniextendr_attr_rejects_unknown_options() {
    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(typo))
        .err()
        .unwrap();
    assert!(err.to_string().contains("unknown `#[miniextendr]` option"));
}

#[test]
fn miniextendr_attr_rejects_option_arguments() {
    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(invisible("string_value")))
        .err()
        .unwrap();
    assert!(
        err.to_string()
            .contains("does not accept parenthesized arguments")
    );
}

#[test]
fn miniextendr_attr_accepts_multiple_flags() {
    let attrs = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(coerce, invisible))
        .expect("should parse multiple flags");
    assert!(attrs.coerce_all);
    assert_eq!(attrs.force_invisible, Some(true));
}

#[test]
fn miniextendr_attr_accepts_unwrap_in_r() {
    // Tagged-condition transport is now the only mode; `unwrap_in_r` is
    // orthogonal (Result-as-value vs Result-as-error-boundary) and stands alone.
    let attrs = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(unwrap_in_r))
        .expect("should parse unwrap_in_r");
    assert!(attrs.unwrap_in_r);
}

#[test]
fn miniextendr_attr_postfix_parses_and_rejects_conflicts() {
    let attrs = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(noexport, postfix = "_impl"))
        .expect("should parse postfix");
    assert_eq!(attrs.postfix.as_deref(), Some("_impl"));
    assert!(attrs.noexport);
    assert!(attrs.r_name.is_none());

    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(postfix = "_impl", r_name = "f2"))
        .err()
        .expect("postfix + r_name must fail");
    assert!(
        err.to_string().contains("both set the R wrapper name"),
        "{err}"
    );

    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(
        postfix = "_impl",
        s3(generic = "print", class = "widget")
    ))
    .err()
    .expect("postfix + s3 must fail");
    assert!(
        err.to_string().contains("cannot be used with `s3("),
        "{err}"
    );

    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(postfix = ""))
        .err()
        .expect("empty postfix must fail");
    assert!(err.to_string().contains("must not be empty"), "{err}");

    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(postfix = "-impl"))
        .err()
        .expect("non-identifier postfix must fail");
    assert!(
        err.to_string().contains("valid R identifier fragment"),
        "{err}"
    );
}

#[test]
fn miniextendr_attr_serde_error_forms() {
    use crate::miniextendr_fn::SerdeErrorSpec;

    // Bare flag: defaults (`kind` tag, `<crate>_error` prefix).
    let attrs = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(serde_error))
        .expect("should parse serde_error");
    let spec = attrs.serde_error.expect("serde_error set");
    assert_eq!(spec, SerdeErrorSpec::default());
    assert_eq!(spec.tag(), "kind");
    assert!(
        spec.prefix().ends_with("_error"),
        "default prefix is `<crate>_error`: {}",
        spec.prefix()
    );

    // Nested options.
    let attrs = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(serde_error(
        tag = "type",
        prefix = "engine"
    )))
    .expect("should parse nested serde_error");
    let spec = attrs.serde_error.expect("serde_error set");
    assert_eq!(spec.tag(), "type");
    assert_eq!(spec.prefix(), "engine");

    // Explicit boolean.
    let attrs = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(serde_error = false))
        .expect("should parse serde_error = false");
    assert!(attrs.serde_error.is_none());
    let attrs = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(serde_error = true))
        .expect("should parse serde_error = true");
    assert!(attrs.serde_error.is_some());
}

#[test]
fn miniextendr_attr_serde_error_rejects_bad_input() {
    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(serde_error(tags = "x")))
        .err()
        .expect("unknown nested key must fail");
    assert!(
        err.to_string().contains("expected `tag` or `prefix`"),
        "{err}"
    );

    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(serde_error(prefix = "")))
        .err()
        .expect("empty prefix must fail");
    assert!(err.to_string().contains("must not be empty"), "{err}");

    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(serde_error, unwrap_in_r))
        .err()
        .expect("serde_error + unwrap_in_r must fail");
    assert!(
        err.to_string()
            .contains("cannot be used with `unwrap_in_r`"),
        "{err}"
    );
}

#[test]
fn miniextendr_attr_call_caller_parses_and_validates() {
    let attrs = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(noexport, call = caller))
        .expect("should parse call = caller");
    assert!(attrs.call_caller);
    assert!(
        !attrs.no_call_attribution,
        "explicit caller attribution keeps the call slot"
    );

    let attrs = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(internal, call = "caller"))
        .expect("string form parses too");
    assert!(attrs.call_caller);

    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(call = caller))
        .err()
        .expect("call = caller without noexport/internal must fail");
    assert!(
        err.to_string().contains("add `noexport` or `internal`"),
        "{err}"
    );

    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(noexport, call = caller, fast))
        .err()
        .expect("call = caller + fast must fail");
    assert!(err.to_string().contains("no_call_attribution"), "{err}");

    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(noexport, call = wrapper))
        .err()
        .expect("only `caller` is accepted");
    assert!(err.to_string().contains("accepts only `caller`"), "{err}");
}

#[test]
fn err_parts_mode_expr_selects_the_serde_path() {
    use crate::c_wrapper_builder::ErrPartsMode;
    use crate::miniextendr_fn::SerdeErrorSpec;

    assert_eq!(ErrPartsMode::from_spec(None), ErrPartsMode::Probe);
    let probe = ErrPartsMode::Probe.expr().to_string();
    assert!(probe.contains("__mx_result_err_parts"), "{probe}");

    let spec = SerdeErrorSpec {
        tag: Some("type".into()),
        prefix: Some("engine".into()),
    };
    let mode = ErrPartsMode::from_spec(Some(&spec));
    assert_eq!(
        mode,
        ErrPartsMode::Serde {
            tag: "type".into(),
            prefix: "engine".into()
        }
    );
    let serde = mode.expr().to_string();
    assert!(serde.contains("serde_err_parts"), "{serde}");
    assert!(serde.contains("\"type\""), "{serde}");
    assert!(serde.contains("\"engine\""), "{serde}");
    assert!(!serde.contains("__mx_result_err_parts"), "{serde}");
}

#[test]
fn dots_validation_stmt_formats_error_with_display_not_debug() {
    // Audit A8: `#[miniextendr(dots = typed_list!(...))]` used to inject
    // `.expect("dots validation failed")`, which Debug-formats the
    // `TypedListError` and leaks PascalCase enum-variant names (e.g.
    // `Missing { name: "x" }`) into the R-visible message — a different
    // style than the direct `typed_list!` path, which already goes through
    // `TypedListError`'s human-phrased `Display` impl. Pin that the
    // generated statement uses `unwrap_or_else` + `{e}` (Display) instead
    // of `expect` (Debug).
    let dots_param = syn::Ident::new("__miniextendr_dots", proc_macro2::Span::call_site());
    // The real `spec_tokens` is the whole `typed_list!(...)` invocation captured
    // as an expression (see `dots_spec` in miniextendr_fn.rs), not the bare spec
    // interior — pass the same shape so `parse_quote!` sees valid Rust.
    let spec_tokens = quote::quote! { typed_list!(x => numeric()) };

    let stmt = crate::build_dots_validation_stmt(&dots_param, &spec_tokens);
    let s = normalize_tokens(quote::quote! { #stmt });

    assert!(s.contains("unwrap_or_else"));
    assert!(s.contains("panic!(\"dotsvalidationfailed:{e}\")"));
    assert!(!s.contains(".expect("));
}

#[test]
fn parsed_fn_adds_inline_never_for_rust_abi() {
    let mut parsed: MiniextendrFunctionParsed = syn::parse2(quote::quote! { fn f() {} }).unwrap();
    parsed.add_inline_never_if_needed();

    let has_inline_never = parsed.item().attrs.iter().any(|attr| {
        attr.path().is_ident("inline")
            && matches!(&attr.meta, syn::Meta::List(list)
                if list.tokens.to_string() == "never")
    });
    assert!(
        has_inline_never,
        "should add #[inline(never)] to Rust ABI functions"
    );
}

#[test]
fn parsed_fn_preserves_explicit_inline() {
    let mut parsed: MiniextendrFunctionParsed =
        syn::parse2(quote::quote! { #[inline(always)] fn f() {} }).unwrap();
    parsed.add_inline_never_if_needed();

    // Should not add inline(never) since inline(always) is already present
    let inline_count = parsed
        .item()
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("inline"))
        .count();
    assert_eq!(
        inline_count, 1,
        "should preserve existing #[inline] attribute"
    );
}

#[test]
fn parsed_fn_no_inline_for_extern_c() {
    let mut parsed: MiniextendrFunctionParsed =
        syn::parse2(quote::quote! { extern "C-unwind" fn f() {} }).unwrap();
    parsed.add_inline_never_if_needed();

    let has_inline = parsed
        .item()
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("inline"));
    assert!(
        !has_inline,
        "should not add #[inline] to extern C functions"
    );
}

fn normalize_tokens(ts: proc_macro2::TokenStream) -> String {
    ts.to_string()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

#[test]
fn derive_into_list_skips_ignored_named_fields() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        struct Foo {
            a: i32,
            #[into_list(ignore)]
            b: i32,
        }
    })
    .unwrap();

    let expanded = crate::list_derive::derive_into_list(input).unwrap();
    let s = normalize_tokens(expanded);

    assert!(s.contains("\"a\""));
    assert!(!s.contains("\"b\""));
}

#[test]
fn derive_try_from_list_defaults_ignored_named_fields() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        struct Foo {
            a: i32,
            #[into_list(ignore)]
            b: i32,
        }
    })
    .unwrap();

    let expanded = crate::list_derive::derive_try_from_list(input).unwrap();
    let s = normalize_tokens(expanded);

    assert!(s.contains("get_named_sexp(\"a\")"));
    assert!(!s.contains("get_named_sexp(\"b\")"));
    assert!(s.contains("b:::core::default::Default::default()"));
}

#[test]
fn derive_try_from_list_does_not_pin_error_type() {
    // Regression for #861: the per-field bound must not pin
    // `TryFromSexp::Error = SexpError`, otherwise fields like `Vec<f64>`
    // (whose error is `SexpTypeError`) fail to compile with E0271. Instead we
    // bound `TryFromSexp` and require the field error convert into `SexpError`.
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        struct Foo {
            estimate: Vec<f64>,
            sigma: f64,
        }
    })
    .unwrap();

    let expanded = crate::list_derive::derive_try_from_list(input).unwrap();
    let s = normalize_tokens(expanded);

    // No equality bound on the associated Error type.
    assert!(!s.contains("TryFromSexp<Error="));
    // Convertibility bound is present for the numeric-vector field.
    assert!(s.contains("SexpError:::core::convert::From<<Vec<f64>as"));
    // Conversion happens at extraction time.
    assert!(s.contains("map_err(::miniextendr_api::from_r::SexpError::from)"));
}

#[test]
fn derive_into_list_skips_ignored_tuple_fields() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        struct Foo(i32, #[into_list(ignore)] i32, i32);
    })
    .unwrap();

    let expanded = crate::list_derive::derive_into_list(input).unwrap();
    let s = normalize_tokens(expanded);

    assert!(s.contains("_field0"));
    assert!(s.contains("_field2"));
    assert!(!s.contains("_field1"));
}

#[test]
fn derive_try_from_list_defaults_ignored_tuple_fields() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        struct Foo(i32, #[into_list(ignore)] i32, i32);
    })
    .unwrap();

    let expanded = crate::list_derive::derive_try_from_list(input).unwrap();
    let s = normalize_tokens(expanded);

    assert!(s.contains("expected:2"));
    assert!(s.contains("get(0"));
    assert!(s.contains("get(1"));
    assert!(!s.contains("get(2"));
    assert!(s.contains("Self(_field0,::core::default::Default::default(),_field2)"));
}

#[test]
fn list_attrs_error_on_unknown_options() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        struct Foo {
            #[into_list(typo)]
            a: i32,
        }
    })
    .unwrap();

    let err = crate::list_derive::derive_into_list(input).unwrap_err();
    assert!(err.to_string().contains("unknown #[into_list(...)] option"));
}

// region: ALTREP derive macro tests

#[test]
fn test_derive_altrep_integer_basic() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        pub struct TestData {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_integer(input).unwrap();
    let output_str = output.to_string();

    // Should generate AltrepLen impl
    assert!(output_str.contains("AltrepLen"));
    assert!(output_str.contains("fn len"));

    // Should generate AltIntegerData impl
    assert!(output_str.contains("AltIntegerData"));
    assert!(output_str.contains("fn elt"));

    // Path (a) / #711: the integer family emits the underlying trait-impl
    // macros directly — it no longer delegates to impl_altinteger_from_data!.
    assert!(
        !output_str.contains("impl_altinteger_from_data"),
        "migrated integer family must not delegate to the declarative macro"
    );
    assert!(output_str.contains("__impl_altrep_base"));
    assert!(output_str.contains("__impl_altvec_integer_dataptr"));
    assert!(output_str.contains("__impl_altinteger_methods"));
    assert!(output_str.contains("impl_inferbase_integer"));

    // Registration: TypedExternal
    assert!(
        output_str.contains("TypedExternal"),
        "AltrepInteger derive must generate TypedExternal impl"
    );

    // Registration: AltrepClass
    assert!(
        output_str.contains("AltrepClass"),
        "AltrepInteger derive must generate AltrepClass impl"
    );

    // Registration: RegisterAltrep (OnceLock class registration)
    assert!(
        output_str.contains("RegisterAltrep"),
        "AltrepInteger derive must generate RegisterAltrep impl"
    );

    // Registration: IntoR / into_sexp conversion
    assert!(
        output_str.contains("IntoR"),
        "AltrepInteger derive must generate IntoR impl"
    );
    assert!(
        output_str.contains("into_sexp"),
        "AltrepInteger derive must generate into_sexp method"
    );

    // Registration: linkme distributed_slice entry
    assert!(
        output_str.contains("distributed_slice") || output_str.contains("MX_ALTREP_REGISTRATIONS"),
        "AltrepInteger derive must generate linkme distributed_slice entry"
    );

    // Registration: Ref and Mut accessor types
    assert!(
        output_str.contains("TestDataRef"),
        "AltrepInteger derive must generate Ref accessor type"
    );
    assert!(
        output_str.contains("TestDataMut"),
        "AltrepInteger derive must generate Mut accessor type"
    );
}

#[test]
fn test_derive_altrep_integer_with_elt_field() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(elt = "value")]
        pub struct ConstantData {
            value: i32,
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_integer(input).unwrap();
    let output_str = output.to_string();

    // Should use field for elt()
    assert!(output_str.contains("self . value"));
}

#[test]
fn test_derive_altrep_integer_with_options() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(dataptr, serialize)]
        pub struct VecData {
            data: Vec<i32>,
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_integer(input).unwrap();
    let output_str = output.to_string();

    // Should pass options to macro
    assert!(output_str.contains("dataptr"));
    assert!(output_str.contains("serialize"));
}

#[test]
fn test_derive_altrep_logical_basic() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        pub struct TestLogical {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_logical(input).unwrap();
    let output_str = output.to_string();

    // Should generate AltrepLen impl
    assert!(output_str.contains("AltrepLen"));
    assert!(output_str.contains("fn len"));

    // Should generate AltLogicalData impl with default NA
    assert!(output_str.contains("AltLogicalData"));
    assert!(output_str.contains("Logical :: Na"));

    // #933: the logical family emits the underlying trait-impl macros
    // directly — it no longer delegates to impl_altlogical_from_data!.
    assert!(
        !output_str.contains("impl_altlogical_from_data"),
        "migrated logical family must not delegate to the declarative macro"
    );
    assert!(output_str.contains("__impl_altrep_base"));
    assert!(output_str.contains("__impl_altvec_logical_dataptr"));
    assert!(output_str.contains("__impl_altlogical_methods"));
    assert!(output_str.contains("impl_inferbase_logical"));

    // Registration checks: every family derive must generate registration code
    assert!(
        output_str.contains("TypedExternal"),
        "AltrepLogical derive must generate TypedExternal impl"
    );
    assert!(
        output_str.contains("AltrepClass"),
        "AltrepLogical derive must generate AltrepClass impl"
    );
    assert!(
        output_str.contains("RegisterAltrep"),
        "AltrepLogical derive must generate RegisterAltrep impl"
    );
    assert!(
        output_str.contains("IntoR"),
        "AltrepLogical derive must generate IntoR impl"
    );
    assert!(
        output_str.contains("into_sexp"),
        "AltrepLogical derive must generate into_sexp method"
    );
    assert!(
        output_str.contains("distributed_slice") || output_str.contains("MX_ALTREP_REGISTRATIONS"),
        "AltrepLogical derive must generate linkme entry"
    );
    assert!(
        output_str.contains("TestLogicalRef"),
        "AltrepLogical derive must generate Ref accessor type"
    );
    assert!(
        output_str.contains("TestLogicalMut"),
        "AltrepLogical derive must generate Mut accessor type"
    );
}

#[test]
fn test_derive_altrep_logical_with_elt_field() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(elt = "value")]
        pub struct LogicalValue {
            value: miniextendr_api::altrep_data::Logical,
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_logical(input).unwrap();
    let output_str = normalize_tokens(output);

    // Should use field conversion via Into<Logical>
    assert!(output_str.contains("self.value.into()"));
}

#[test]
fn test_derive_altrep_logical_with_options() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(dataptr)]
        pub struct LogicalVecData {
            value: bool,
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_logical(input).unwrap();
    let output_str = output.to_string();

    // Should pass options to macro
    assert!(output_str.contains("dataptr"));
}
#[test]
fn test_derive_altrep_real_generates_registration() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        pub struct TestReal {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_real(input).unwrap();
    let output_str = output.to_string();

    // Low-level traits — #933: direct emit, no declarative-macro delegation
    assert!(output_str.contains("AltrepLen"));
    assert!(output_str.contains("AltRealData"));
    assert!(
        !output_str.contains("impl_altreal_from_data"),
        "migrated real family must not delegate to the declarative macro"
    );
    assert!(output_str.contains("__impl_altrep_base"));
    assert!(output_str.contains("__impl_altvec_real_dataptr"));
    assert!(output_str.contains("__impl_altreal_methods"));
    assert!(output_str.contains("impl_inferbase_real"));

    // Registration: TypedExternal, AltrepClass, RegisterAltrep, IntoR, linkme, Ref/Mut
    assert!(
        output_str.contains("TypedExternal"),
        "AltrepReal derive must generate TypedExternal impl"
    );
    assert!(
        output_str.contains("AltrepClass"),
        "AltrepReal derive must generate AltrepClass impl"
    );
    assert!(
        output_str.contains("RegisterAltrep"),
        "AltrepReal derive must generate RegisterAltrep impl"
    );
    assert!(
        output_str.contains("IntoR"),
        "AltrepReal derive must generate IntoR impl"
    );
    assert!(
        output_str.contains("into_sexp"),
        "AltrepReal derive must generate into_sexp method"
    );
    assert!(
        output_str.contains("distributed_slice") || output_str.contains("MX_ALTREP_REGISTRATIONS"),
        "AltrepReal derive must generate linkme entry"
    );
    assert!(
        output_str.contains("TestRealRef"),
        "AltrepReal derive must generate Ref accessor type"
    );
    assert!(
        output_str.contains("TestRealMut"),
        "AltrepReal derive must generate Mut accessor type"
    );
}

#[test]
fn test_derive_altrep_raw_generates_registration() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        pub struct TestRaw {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_raw(input).unwrap();
    let output_str = output.to_string();

    // Low-level traits — #933: direct emit, no declarative-macro delegation
    assert!(output_str.contains("AltrepLen"));
    assert!(output_str.contains("AltRawData"));
    assert!(
        !output_str.contains("impl_altraw_from_data"),
        "migrated raw family must not delegate to the declarative macro"
    );
    assert!(output_str.contains("__impl_altrep_base"));
    assert!(output_str.contains("__impl_altvec_raw_dataptr"));
    assert!(output_str.contains("__impl_altraw_methods"));
    assert!(output_str.contains("impl_inferbase_raw"));

    // Registration
    assert!(
        output_str.contains("TypedExternal"),
        "AltrepRaw derive must generate TypedExternal impl"
    );
    assert!(
        output_str.contains("AltrepClass"),
        "AltrepRaw derive must generate AltrepClass impl"
    );
    assert!(
        output_str.contains("RegisterAltrep"),
        "AltrepRaw derive must generate RegisterAltrep impl"
    );
    assert!(
        output_str.contains("IntoR"),
        "AltrepRaw derive must generate IntoR impl"
    );
    assert!(
        output_str.contains("into_sexp"),
        "AltrepRaw derive must generate into_sexp method"
    );
    assert!(
        output_str.contains("distributed_slice") || output_str.contains("MX_ALTREP_REGISTRATIONS"),
        "AltrepRaw derive must generate linkme entry"
    );
    assert!(
        output_str.contains("TestRawRef"),
        "AltrepRaw derive must generate Ref accessor type"
    );
    assert!(
        output_str.contains("TestRawMut"),
        "AltrepRaw derive must generate Mut accessor type"
    );
}

#[test]
fn test_derive_altrep_string_generates_registration() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        pub struct TestString {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_string(input).unwrap();
    let output_str = output.to_string();

    // Low-level traits — #933: direct emit, no declarative-macro delegation.
    // String's default arm routes through the whole-vector STRSXP
    // materialization macro (no typed contiguous dataptr).
    assert!(output_str.contains("AltrepLen"));
    assert!(output_str.contains("AltStringData"));
    assert!(
        !output_str.contains("impl_altstring_from_data"),
        "migrated string family must not delegate to the declarative macro"
    );
    assert!(output_str.contains("__impl_altrep_base"));
    assert!(output_str.contains("__impl_altvec_string_dataptr"));
    assert!(output_str.contains("__impl_altstring_methods"));
    assert!(output_str.contains("impl_inferbase_string"));

    // Registration
    assert!(
        output_str.contains("TypedExternal"),
        "AltrepString derive must generate TypedExternal impl"
    );
    assert!(
        output_str.contains("AltrepClass"),
        "AltrepString derive must generate AltrepClass impl"
    );
    assert!(
        output_str.contains("RegisterAltrep"),
        "AltrepString derive must generate RegisterAltrep impl"
    );
    assert!(
        output_str.contains("IntoR"),
        "AltrepString derive must generate IntoR impl"
    );
    assert!(
        output_str.contains("into_sexp"),
        "AltrepString derive must generate into_sexp method"
    );
    assert!(
        output_str.contains("distributed_slice") || output_str.contains("MX_ALTREP_REGISTRATIONS"),
        "AltrepString derive must generate linkme entry"
    );
    assert!(
        output_str.contains("TestStringRef"),
        "AltrepString derive must generate Ref accessor type"
    );
    assert!(
        output_str.contains("TestStringMut"),
        "AltrepString derive must generate Mut accessor type"
    );
}

#[test]
fn test_derive_altrep_complex_generates_registration() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        pub struct TestComplex {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_complex(input).unwrap();
    let output_str = output.to_string();

    // Low-level traits — #933: direct emit, no declarative-macro delegation
    assert!(output_str.contains("AltrepLen"));
    assert!(output_str.contains("AltComplexData"));
    assert!(
        !output_str.contains("impl_altcomplex_from_data"),
        "migrated complex family must not delegate to the declarative macro"
    );
    assert!(output_str.contains("__impl_altrep_base"));
    assert!(output_str.contains("__impl_altvec_complex_dataptr"));
    assert!(output_str.contains("__impl_altcomplex_methods"));
    assert!(output_str.contains("impl_inferbase_complex"));

    // Registration
    assert!(
        output_str.contains("TypedExternal"),
        "AltrepComplex derive must generate TypedExternal impl"
    );
    assert!(
        output_str.contains("AltrepClass"),
        "AltrepComplex derive must generate AltrepClass impl"
    );
    assert!(
        output_str.contains("RegisterAltrep"),
        "AltrepComplex derive must generate RegisterAltrep impl"
    );
    assert!(
        output_str.contains("IntoR"),
        "AltrepComplex derive must generate IntoR impl"
    );
    assert!(
        output_str.contains("into_sexp"),
        "AltrepComplex derive must generate into_sexp method"
    );
    assert!(
        output_str.contains("distributed_slice") || output_str.contains("MX_ALTREP_REGISTRATIONS"),
        "AltrepComplex derive must generate linkme entry"
    );
    assert!(
        output_str.contains("TestComplexRef"),
        "AltrepComplex derive must generate Ref accessor type"
    );
    assert!(
        output_str.contains("TestComplexMut"),
        "AltrepComplex derive must generate Mut accessor type"
    );
}
// endregion

// region: ALTREP list registration test

#[test]
fn test_derive_altrep_list_generates_registration() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        pub struct TestListData {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_list(input).unwrap();
    let output_str = output.to_string();

    // Must generate registration (not just low-level traits)
    assert!(
        output_str.contains("TypedExternal"),
        "AltrepList derive must generate TypedExternal"
    );
    assert!(
        output_str.contains("AltrepClass"),
        "AltrepList derive must generate AltrepClass"
    );
    assert!(
        output_str.contains("RegisterAltrep"),
        "AltrepList derive must generate RegisterAltrep"
    );
    assert!(
        output_str.contains("IntoR"),
        "AltrepList derive must generate IntoR"
    );
    assert!(
        output_str.contains("into_sexp"),
        "AltrepList derive must generate into_sexp"
    );
    assert!(
        output_str.contains("distributed_slice") || output_str.contains("MX_ALTREP_REGISTRATIONS"),
        "AltrepList derive must generate linkme entry"
    );
    assert!(
        output_str.contains("TestListDataRef"),
        "AltrepList derive must generate Ref accessor type"
    );
    assert!(
        output_str.contains("TestListDataMut"),
        "AltrepList derive must generate Mut accessor type"
    );

    // Low-level list traits — #933: direct emit, no declarative-macro delegation
    assert!(output_str.contains("AltrepLen"));
    assert!(output_str.contains("AltListData"));
    assert!(
        !output_str.contains("impl_altlist_from_data"),
        "migrated list family must not delegate to the declarative macro"
    );
    assert!(output_str.contains("__impl_altrep_base"));
    assert!(output_str.contains("AltList"));
    assert!(output_str.contains("altrep_extract_ref"));
    assert!(output_str.contains("impl_inferbase_list"));
}
// endregion

// region: ALTREP class attribute tests

#[test]
fn test_derive_altrep_class_attr() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(class = "CustomName")]
        pub struct TestData {
            data: Vec<i32>,
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep::derive_altrep(input).unwrap();
    let output_str = output.to_string();

    assert!(
        output_str.contains("CustomName"),
        "class name must appear in output"
    );
    assert!(
        output_str.contains("TypedExternal"),
        "must generate TypedExternal"
    );
    assert!(
        output_str.contains("RegisterAltrep"),
        "must generate RegisterAltrep"
    );
}

#[test]
fn test_derive_altrep_family_class_attr() {
    // Test that family-specific derives also respect #[altrep(class = "...")]
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(class = "MyCustomInteger")]
        pub struct CustomIntData {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_integer(input).unwrap();
    let output_str = output.to_string();

    assert!(
        output_str.contains("MyCustomInteger"),
        "family derive must use custom class name from #[altrep(class = ...)]"
    );
    assert!(output_str.contains("TypedExternal"));
    assert!(output_str.contains("RegisterAltrep"));
    assert!(output_str.contains("IntoR"));
}
// endregion

// region: ALTREP guard mode tests

#[test]
fn test_derive_altrep_integer_unsafe_guard() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(r#unsafe)]
        pub struct FastData {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_integer(input).unwrap();
    let output_str = output.to_string();

    // Non-default guard: should expand individual macros, not impl_altinteger_from_data!
    assert!(!output_str.contains("impl_altinteger_from_data"));
    assert!(output_str.contains("__impl_altrep_base"));
    assert!(output_str.contains("Unsafe"));
}

#[test]
fn test_derive_altrep_integer_r_unwind_guard() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(r_unwind)]
        pub struct SafeData {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_integer(input).unwrap();
    let output_str = output.to_string();

    // RUnwind is the default guard. Pre-#711 this used the high-level
    // impl_altinteger_from_data! macro; the integer family now emits the
    // underlying __impl_* macros directly (with the default RUnwind guard).
    assert!(!output_str.contains("impl_altinteger_from_data"));
    assert!(output_str.contains("__impl_altrep_base"));
    assert!(output_str.contains("__impl_altvec_integer_dataptr"));
    assert!(output_str.contains("RUnwind"));
}

#[test]
fn test_derive_altrep_integer_rust_unwind_guard_uses_expanded_path() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(rust_unwind)]
        pub struct DefaultData {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_integer(input).unwrap();
    let output_str = output.to_string();

    // rust_unwind is non-default (RUnwind is default) — should use expanded path
    assert!(!output_str.contains("impl_altinteger_from_data"));
    assert!(output_str.contains("__impl_altrep_base"));
    assert!(output_str.contains("RustUnwind"));
}

#[test]
fn test_derive_altrep_unsafe_guard_with_serialize() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(r#unsafe, serialize)]
        pub struct SerData {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_integer(input).unwrap();
    let output_str = output.to_string();

    // Expanded path emits __impl_altrep_base!(Ty, Unsafe, with_serialize)
    assert!(output_str.contains("__impl_altrep_base"));
    assert!(output_str.contains("with_serialize"));
    assert!(output_str.contains("Unsafe"));
}

#[test]
fn test_derive_altrep_list_with_guard() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(r_unwind)]
        pub struct ListData {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_list(input).unwrap();
    let output_str = output.to_string();

    // #933: direct emit with the resolved (default RUnwind) guard
    assert!(!output_str.contains("impl_altlist_from_data"));
    assert!(output_str.contains("__impl_altrep_base"));
    assert!(output_str.contains("RUnwind"));
}
// endregion

// region: ALTREP invalid option combo tests

#[test]
fn test_derive_altrep_real_accepts_subset() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(subset)]
        pub struct GoodReal {
            len: usize,
        }
    })
    .unwrap();

    // subset is now supported for all atomic types including real
    crate::altrep_derive::derive_altrep_real(input).unwrap();
}

#[test]
fn test_derive_altrep_logical_accepts_subset() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(subset)]
        pub struct GoodLogical {
            len: usize,
        }
    })
    .unwrap();

    crate::altrep_derive::derive_altrep_logical(input).unwrap();
}

#[test]
fn test_derive_altrep_raw_accepts_subset() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(subset)]
        pub struct GoodRaw {
            len: usize,
        }
    })
    .unwrap();

    crate::altrep_derive::derive_altrep_raw(input).unwrap();
}

#[test]
fn test_derive_altrep_string_accepts_subset() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(subset)]
        pub struct GoodString {
            len: usize,
        }
    })
    .unwrap();

    crate::altrep_derive::derive_altrep_string(input).unwrap();
}

#[test]
fn test_derive_altrep_integer_rejects_dataptr_plus_subset() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(dataptr, subset)]
        pub struct BadCombo {
            len: usize,
        }
    })
    .unwrap();

    let err = crate::altrep_derive::derive_altrep_integer(input).unwrap_err();
    assert!(err.to_string().contains("mutually exclusive"));
}

#[test]
fn test_derive_altrep_complex_rejects_dataptr_plus_subset() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(dataptr, subset)]
        pub struct BadCombo {
            len: usize,
        }
    })
    .unwrap();

    let err = crate::altrep_derive::derive_altrep_complex(input).unwrap_err();
    assert!(err.to_string().contains("mutually exclusive"));
}

#[test]
fn test_derive_altrep_list_rejects_dataptr() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(dataptr)]
        pub struct BadList {
            len: usize,
        }
    })
    .unwrap();

    let err = crate::altrep_derive::derive_altrep_list(input).unwrap_err();
    assert!(err.to_string().contains("not supported"));
}

#[test]
fn test_derive_altrep_list_accepts_serialize() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(serialize)]
        pub struct GoodList {
            len: usize,
        }
    })
    .unwrap();

    // serialize is now supported for list
    crate::altrep_derive::derive_altrep_list(input).unwrap();
}

#[test]
fn test_derive_altrep_real_subset_accepted_with_guard_too() {
    // subset is now supported for real, even with non-default guard
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(r#unsafe, subset)]
        pub struct GoodReal {
            len: usize,
        }
    })
    .unwrap();

    crate::altrep_derive::derive_altrep_real(input).unwrap();
}

#[test]
fn test_altrep_try_from_sexp_expected_tag_uses_family_base() {
    // #890: the not-an-ALTREP diagnostic must report the family's base
    // SEXPTYPE (via AltrepClass::BASE), not a hardcoded INTSXP.
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        pub struct DiagReal {
            len: usize,
        }
    })
    .unwrap();

    let expanded = crate::altrep_derive::derive_altrep_real(input).unwrap();
    let s = normalize_tokens(expanded);

    assert!(s.contains("BASE.sexptype()"));
    assert!(!s.contains("expected:SEXPTYPE::INTSXP"));
    // The data1-downcast branch legitimately expects an EXTPTRSXP.
    assert!(s.contains("expected:SEXPTYPE::EXTPTRSXP"));
}
// endregion

// region: fast-default feature resolution tests

/// When `fast-default` feature is enabled, `no_preconditions` and
/// `no_call_attribution` both resolve to `true` by default (no annotation
/// needed). With explicit `no_fast`, they resolve to `false` even under the
/// feature.
///
/// Run with: `cargo test -p miniextendr-macros --features fast-default`
#[cfg(feature = "fast-default")]
#[test]
fn fast_default_fn_attrs_resolve_both_true() {
    // Empty attrs → both fields resolved via cfg!(feature = "fast-default")
    let attrs: MiniextendrFnAttrs = syn::parse2(quote::quote! {}).unwrap();
    assert!(
        attrs.no_preconditions,
        "fast-default should set no_preconditions to true by default"
    );
    assert!(
        attrs.no_call_attribution,
        "fast-default should set no_call_attribution to true by default"
    );
}

#[cfg(feature = "fast-default")]
#[test]
fn fast_default_no_fast_opt_out_restores_false() {
    // `no_fast` explicitly opts out of both, even under fast-default
    let attrs: MiniextendrFnAttrs = syn::parse2(quote::quote! { no_fast }).unwrap();
    assert!(
        !attrs.no_preconditions,
        "no_fast should set no_preconditions to false"
    );
    assert!(
        !attrs.no_call_attribution,
        "no_fast should set no_call_attribution to false"
    );
}

#[cfg(not(feature = "fast-default"))]
#[test]
fn no_fast_default_fn_attrs_resolve_false() {
    // Without the feature, empty attrs → both false
    let attrs: MiniextendrFnAttrs = syn::parse2(quote::quote! {}).unwrap();
    assert!(
        !attrs.no_preconditions,
        "without fast-default, no_preconditions should be false"
    );
    assert!(
        !attrs.no_call_attribution,
        "without fast-default, no_call_attribution should be false"
    );
}

#[test]
fn fast_bundle_alias_sets_both() {
    let attrs: MiniextendrFnAttrs = syn::parse2(quote::quote! { fast }).unwrap();
    assert!(attrs.no_preconditions, "fast should set no_preconditions");
    assert!(
        attrs.no_call_attribution,
        "fast should set no_call_attribution"
    );
}

#[test]
fn no_fast_clears_both() {
    let attrs: MiniextendrFnAttrs = syn::parse2(quote::quote! { no_fast }).unwrap();
    // In the non-fast-default case no_fast → Some(false), resolves to false.
    // In the fast-default case same thing.
    assert!(
        !attrs.no_preconditions,
        "no_fast should clear no_preconditions"
    );
    assert!(
        !attrs.no_call_attribution,
        "no_fast should clear no_call_attribution"
    );
}

#[test]
fn fast_eq_false_name_value_clears_both() {
    let attrs: MiniextendrFnAttrs = syn::parse2(quote::quote! { fast = false }).unwrap();
    assert!(!attrs.no_preconditions);
    assert!(!attrs.no_call_attribution);
}

#[test]
fn fast_eq_true_name_value_sets_both() {
    let attrs: MiniextendrFnAttrs = syn::parse2(quote::quote! { fast = true }).unwrap();
    assert!(attrs.no_preconditions);
    assert!(attrs.no_call_attribution);
}

#[test]
fn no_preconditions_independent_parse() {
    let attrs: MiniextendrFnAttrs = syn::parse2(quote::quote! { no_preconditions }).unwrap();
    assert!(attrs.no_preconditions);
}

#[test]
fn no_call_attribution_independent_parse() {
    let attrs: MiniextendrFnAttrs = syn::parse2(quote::quote! { no_call_attribution }).unwrap();
    assert!(attrs.no_call_attribution);
}
// endregion
