use super::*;

// region: Helper function for parsing impl blocks

fn default_impl_attrs(class_system: ClassSystem) -> ImplAttrs {
    ImplAttrs {
        class_system,
        class_name: None,
        label: None,
        vctrs_attrs: VctrsAttrs::default(),
        r6_inherit: None,
        r6_portable: None,
        r6_cloneable: None,
        r6_lock_objects: None,
        r6_lock_class: None,
        s7_parent: None,
        s7_abstract: false,
        r_data_accessors: false,
        strict: false,
        no_preconditions: false,
        no_call_attribution: false,
        internal: false,
        noexport: false,
        blanket: false,
    }
}

fn parse_impl(class_system: ClassSystem, code: syn::ItemImpl) -> ParsedImpl {
    let attrs = default_impl_attrs(class_system);
    ParsedImpl::parse(attrs, code).expect("failed to parse impl")
}

fn parse_impl_with_class_name(
    class_system: ClassSystem,
    class_name: &str,
    code: syn::ItemImpl,
) -> ParsedImpl {
    let mut attrs = default_impl_attrs(class_system);
    attrs.class_name = Some(class_name.to_string());
    ParsedImpl::parse(attrs, code).expect("failed to parse impl")
}

fn parse_impl_with_label(
    class_system: ClassSystem,
    label: &str,
    code: syn::ItemImpl,
) -> ParsedImpl {
    let mut attrs = default_impl_attrs(class_system);
    attrs.label = Some(label.to_string());
    ParsedImpl::parse(attrs, code).expect("failed to parse impl")
}
// endregion

// region: Impl-method dots support

#[test]
fn impl_method_rewrites_raw_variadic_in_reemitted_impl() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl DotsThing {
            pub fn collect(&self, dots: ...) -> i32 {
                unimplemented!()
            }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let method = &parsed.methods[0];

    assert!(method.has_dots);
    assert_eq!(
        method.method_attrs.named_dots.as_ref().unwrap().to_string(),
        "dots"
    );
    assert!(method.sig.variadic.is_none());

    let syn::ImplItem::Fn(reemitted) = &parsed.original_impl.items[0] else {
        panic!("expected reemitted method");
    };
    assert!(reemitted.sig.variadic.is_none());

    let Some(syn::FnArg::Typed(last)) = reemitted.sig.inputs.last() else {
        panic!("expected trailing dots arg");
    };
    let syn::Pat::Ident(pat_ident) = last.pat.as_ref() else {
        panic!("expected dots ident");
    };
    assert_eq!(pat_ident.ident, "dots");
    assert!(crate::miniextendr_fn::is_dots_type(last.ty.as_ref()));
}

#[test]
fn r6_dots_constructor_and_method_emit_variadic_r_wrappers() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl R6DotsThing {
            pub fn new(dots: ...) -> Self {
                unimplemented!()
            }

            pub fn collect(&self, dots: ...) -> i32 {
                unimplemented!()
            }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    assert!(wrapper.contains("initialize = function(..., .ptr = NULL)"));
    assert!(wrapper.contains("R6DotsThing$set(\"public\", \"collect\", function(...)"));
    assert!(
        wrapper.contains(
            ".Call(C_miniextendr_macros_R6DotsThing__new, .call = match.call(), list(...))"
        )
    );
    assert!(
        wrapper.contains(
            ".Call(C_miniextendr_macros_R6DotsThing__collect, .call = match.call(), private$.ptr, list(...))"
        )
    );
}

#[test]
fn s3_user_dots_suppress_duplicate_dispatch_dots() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl S3DotsThing {
            pub fn new(dots: ...) -> Self {
                unimplemented!()
            }

            pub fn collect(&self, dots: ...) -> i32 {
                unimplemented!()
            }
        }
    };

    let parsed = parse_impl(ClassSystem::S3, item_impl);
    let wrapper = generate_s3_r_wrapper(&parsed);

    assert!(wrapper.contains("new_s3dotsthing <- function(...)"));
    assert!(wrapper.contains("collect.S3DotsThing <- function(x, ...)"));
    assert!(
        wrapper.contains(
            ".Call(C_miniextendr_macros_S3DotsThing__new, .call = match.call(), list(...))"
        )
    );
    assert!(wrapper.contains(
        ".Call(C_miniextendr_macros_S3DotsThing__collect, .call = match.call(), x, list(...))"
    ));
    assert!(!wrapper.contains("function(x, ..., ...)"));
}

#[test]
fn s4_user_dots_emit_variadic_without_duplicate_dispatch_dots() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl S4DotsThing {
            pub fn new(dots: ...) -> Self {
                unimplemented!()
            }

            pub fn collect(&self, dots: ...) -> i32 {
                unimplemented!()
            }
        }
    };

    let parsed = parse_impl(ClassSystem::S4, item_impl);
    let wrapper = generate_s4_r_wrapper(&parsed);

    assert!(wrapper.contains("S4DotsThing <- function(...)"));
    assert!(
        wrapper.contains("methods::setMethod(\"s4_collect\", \"S4DotsThing\", function(x, ...)")
    );
    assert!(
        wrapper.contains(
            ".Call(C_miniextendr_macros_S4DotsThing__new, .call = match.call(), list(...))"
        )
    );
    assert!(wrapper.contains(".Call(C_miniextendr_macros_S4DotsThing__collect"));
    assert!(wrapper.contains("list(...)"));
    assert!(!wrapper.contains("function(x, ..., ...)"));
}

#[test]
fn s7_user_dots_emit_variadic_without_duplicate_dispatch_dots() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl S7DotsThing {
            pub fn new(dots: ...) -> Self {
                unimplemented!()
            }

            pub fn collect(&self, dots: ...) -> i32 {
                unimplemented!()
            }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let wrapper = generate_s7_r_wrapper(&parsed);

    assert!(wrapper.contains("constructor = function(..., .ptr = NULL)"));
    assert!(wrapper.contains("list(...)"));
    // The S7 generic and per-class fast-path shortcut both carry a single `...`.
    assert!(!wrapper.contains("function(x, ..., ...)"));
    assert!(!wrapper.contains("function(self, ..., ...)"));
}

#[test]
fn env_user_dots_emit_variadic_in_constructor_and_method() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl EnvDotsThing {
            pub fn new(dots: ...) -> Self {
                unimplemented!()
            }

            pub fn collect(&self, dots: ...) -> i32 {
                unimplemented!()
            }
        }
    };

    let parsed = parse_impl(ClassSystem::Env, item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    assert!(wrapper.contains("EnvDotsThing$new <- function(...)"));
    assert!(wrapper.contains("EnvDotsThing$collect <- function(...)"));
    assert!(wrapper.contains(
        ".Call(C_miniextendr_macros_EnvDotsThing__collect, .call = match.call(), self, list(...))"
    ));
    assert!(!wrapper.contains("function(..., ...)"));
}

#[test]
fn vctrs_user_dots_emit_variadic_in_constructor_and_static_helper() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl VctrsDotsThing {
            pub fn new(x: f64, dots: ...) -> Vec<f64> {
                unimplemented!()
            }

            pub fn combine(amounts: Vec<f64>, dots: ...) -> Vec<f64> {
                unimplemented!()
            }
        }
    };

    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Vctr,
        base: Some("double".to_string()),
        inherit_base_type: Some(false),
        ptype: None,
        abbr: Some("vdt".to_string()),
    };

    let parsed = parse_impl_vctrs(vctrs_attrs, item_impl);
    let wrapper = generate_vctrs_r_wrapper(&parsed);

    assert!(wrapper.contains("new_vctrsdotsthing <- function(x, ...)"));
    assert!(wrapper.contains("vctrsdotsthing_combine <- function(amounts, ...)"));
    assert!(wrapper.contains(
        ".Call(C_miniextendr_macros_VctrsDotsThing__new, .call = match.call(), x, list(...))"
    ));
    assert!(wrapper.contains("list(...)"));
    assert!(!wrapper.contains("function(amounts, ..., ...)"));
}

#[test]
fn impl_method_dots_sugar_injects_dots_typed_binding() {
    // `#[miniextendr(dots = typed_list!(...))]` on an impl method should inject
    // the `dots_typed` binding at the top of the re-emitted Rust body, exactly
    // like the standalone-fn path. Because injection happens on the single
    // `original_impl` node, this holds for all six class systems by
    // construction, so one class system exercises the shared path.
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl SugarDotsThing {
            #[miniextendr(dots = typed_list!(scale => numeric()))]
            pub fn scaled(&self, dots: ...) -> f64 {
                unimplemented!()
            }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let syn::ImplItem::Fn(reemitted) = &parsed.original_impl.items[0] else {
        panic!("expected reemitted method");
    };
    let first = reemitted
        .block
        .stmts
        .first()
        .expect("injected dots_typed statement");
    let rendered = quote::quote!(#first).to_string();
    assert!(
        rendered.contains("dots_typed"),
        "expected injected dots_typed binding, got: {rendered}"
    );
    assert!(
        rendered.contains("typed_list"),
        "expected typed_list spec in binding, got: {rendered}"
    );
}

#[test]
fn impl_method_dots_sugar_injects_binding_for_explicit_dots_param() {
    // The sugar resolves its dots ident from `named_dots`, which covers both the
    // rewritten `...` path and an explicit trailing `&Dots` parameter. Injection
    // must land on the explicit-param method the same way.
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl ExplicitSugarDots {
            #[miniextendr(dots = typed_list!(scale => numeric()))]
            pub fn scaled(&self, my_dots: &Dots) -> f64 {
                unimplemented!()
            }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let syn::ImplItem::Fn(reemitted) = &parsed.original_impl.items[0] else {
        panic!("expected reemitted method");
    };
    let first = reemitted
        .block
        .stmts
        .first()
        .expect("injected dots_typed statement");
    let rendered = quote::quote!(#first).to_string();
    assert!(
        rendered.contains("dots_typed") && rendered.contains("my_dots"),
        "expected dots_typed binding over `my_dots`, got: {rendered}"
    );
}

#[test]
fn impl_method_dots_sugar_without_dots_param_errors() {
    // The sugar requires the method to actually take a dots parameter (a
    // trailing `...` or `&Dots`); using it on a method with no dots parameter
    // is a clear compile error rather than a silently-dropped attribute.
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl NoDotsSugar {
            #[miniextendr(dots = typed_list!(scale => numeric()))]
            pub fn oops(&self, x: i32) -> i32 {
                x
            }
        }
    };

    let attrs = default_impl_attrs(ClassSystem::R6);
    let err = ParsedImpl::parse(attrs, item_impl)
        .expect_err("dots sugar without a dots parameter must error");
    assert!(
        err.to_string()
            .contains("requires the method to take a dots parameter"),
        "unexpected error: {err}"
    );
}

// endregion

// region: Env class system tests

#[test]
fn env_wrappers_preserve_static_params() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl ReceiverCounter {
            pub fn new(initial: i32) -> Self {
                unimplemented!()
            }

            pub fn add(&self, amount: i32) -> i32 {
                amount
            }

            pub fn default_counter(step: i32) -> Self {
                unimplemented!()
            }
        }
    };

    let parsed = parse_impl(ClassSystem::Env, item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    assert!(wrapper.contains("ReceiverCounter$new <- function(initial)"));
    assert!(wrapper.contains("ReceiverCounter$add <- function(amount)"));
    assert!(wrapper.contains("ReceiverCounter$default_counter <- function(step)"));
}

#[test]
fn env_wrapper_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
            pub fn add(&mut self, n: i32) -> i32 { unimplemented!() }
            pub fn from_string(s: String) -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::Env, item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    // Verify class environment creation
    assert!(wrapper.contains("Counter <- new.env(parent = emptyenv())"));

    // Verify constructor
    assert!(wrapper.contains("Counter$new <- function(value)"));
    assert!(wrapper.contains(".Call(C_miniextendr_macros_Counter__new"));
    assert!(wrapper.contains("class(self) <- \"Counter\""));

    // Verify instance methods
    assert!(wrapper.contains("Counter$get <- function()"));
    assert!(wrapper.contains("Counter$increment <- function()"));
    assert!(wrapper.contains("Counter$add <- function(n)"));
    assert!(
        wrapper.contains(".Call(C_miniextendr_macros_Counter__get, .call = match.call(), self)")
    );
    assert!(
        wrapper
            .contains(".Call(C_miniextendr_macros_Counter__increment, .call = match.call(), self)")
    );
    assert!(
        wrapper.contains(".Call(C_miniextendr_macros_Counter__add, .call = match.call(), self, n)")
    );

    // Verify static methods
    assert!(wrapper.contains("Counter$from_string <- function(s)"));
    assert!(
        wrapper
            .contains(".Call(C_miniextendr_macros_Counter__from_string, .call = match.call(), s)")
    );

    // Verify $ dispatch
    assert!(wrapper.contains("`$.Counter` <- function(self, name)"));
    assert!(wrapper.contains("`[[.Counter` <- `$.Counter`"));
}

#[test]
fn env_wrapper_with_custom_class_name() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl MyRustType {
            pub fn new() -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl_with_class_name(ClassSystem::Env, "RCounter", item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    assert!(wrapper.contains("RCounter <- new.env(parent = emptyenv())"));
    assert!(wrapper.contains("RCounter$new <- function()"));
    assert!(wrapper.contains("class(self) <- \"RCounter\""));
}
// endregion

// region: R6 class system tests

#[test]
fn r6_wrapper_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
            pub fn from_value(v: i32) -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    // Verify R6Class definition
    assert!(wrapper.contains("Counter <- R6::R6Class(\"Counter\","));

    // Verify public list
    assert!(wrapper.contains("public = list("));

    // Verify initialize with .ptr parameter (because from_value returns Self)
    assert!(wrapper.contains("initialize = function(value, .ptr = NULL)"));
    assert!(wrapper.contains("if (!is.null(.ptr))"));
    assert!(wrapper.contains("private$.ptr <- .ptr"));
    assert!(wrapper.contains(".val <- .Call(C_miniextendr_macros_Counter__new"));
    assert!(wrapper.contains("private$.ptr <- .val"));

    // Verify public instance methods migrate to top-level `$set("public", ...)` blocks (#369)
    assert!(wrapper.contains("Counter$set(\"public\", \"get\", function()"));
    assert!(wrapper.contains("Counter$set(\"public\", \"increment\", function()"));
    assert!(
        wrapper.contains(
            ".Call(C_miniextendr_macros_Counter__get, .call = match.call(), private$.ptr)"
        )
    );
    assert!(wrapper.contains(
        ".Call(C_miniextendr_macros_Counter__increment, .call = match.call(), private$.ptr)"
    ));

    // Verify private list
    assert!(wrapper.contains("private = list("));
    assert!(wrapper.contains(".ptr = NULL"));

    // Verify class options
    assert!(wrapper.contains("lock_objects = TRUE"));
    assert!(wrapper.contains("lock_class = FALSE"));
    assert!(wrapper.contains("cloneable = FALSE"));

    // Verify static methods as separate functions
    assert!(wrapper.contains("Counter$from_value <- function(v)"));
    assert!(
        wrapper
            .contains(".Call(C_miniextendr_macros_Counter__from_value, .call = match.call(), v)")
    );
}

#[test]
fn r6_wrapper_private_methods() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            fn internal_compute(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    // Public method migrates to a top-level `$set("public", ...)` block (#369)
    assert!(wrapper.contains("Counter$set(\"public\", \"get\", function()"));

    // Private method stays inline in the private list
    assert!(wrapper.contains("internal_compute = function()"));
}

#[test]
fn r6_wrapper_roxygen_imports() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    assert!(wrapper.contains("@importFrom R6 R6Class"));
}

#[test]
fn r6_wrapper_inherit() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Child {
            pub fn new() -> Self { unimplemented!() }
            pub fn child_method(&self) -> i32 { unimplemented!() }
        }
    };

    let mut attrs = default_impl_attrs(ClassSystem::R6);
    attrs.r6_inherit = Some("ParentClass".to_string());
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    let wrapper = generate_r6_r_wrapper(&parsed);

    // inherit = uses a placeholder; resolver replaces at cdylib write time
    assert!(
        wrapper
            .contains("Child <- R6::R6Class(\"Child\", inherit = .__MX_CLASS_REF_ParentClass__,")
    );
}

#[test]
fn r6_wrapper_cloneable_and_locks() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl MyClass {
            pub fn new() -> Self { unimplemented!() }
        }
    };

    let mut attrs = default_impl_attrs(ClassSystem::R6);
    attrs.r6_cloneable = Some(true);
    attrs.r6_lock_objects = Some(false);
    attrs.r6_lock_class = Some(true);
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    let wrapper = generate_r6_r_wrapper(&parsed);

    assert!(wrapper.contains("cloneable = TRUE"));
    assert!(wrapper.contains("lock_objects = FALSE,"));
    // The class is created unlocked so `$set()` can add methods; `lock_class = TRUE`
    // is re-applied via a trailing `MyClass$lock()` (#369).
    assert!(wrapper.contains("lock_class = FALSE,"));
    assert!(wrapper.contains("MyClass$lock()"));
}

#[test]
fn r6_wrapper_non_portable() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl MyClass {
            pub fn new() -> Self { unimplemented!() }
        }
    };

    let mut attrs = default_impl_attrs(ClassSystem::R6);
    attrs.r6_portable = Some(false);
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    let wrapper = generate_r6_r_wrapper(&parsed);

    assert!(wrapper.contains("portable = FALSE,"));
}

#[test]
fn r6_wrapper_defaults_unchanged() {
    // Verify that default R6 options match the old hardcoded values
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl MyClass {
            pub fn new() -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    // Defaults: lock_objects=TRUE, lock_class=FALSE, cloneable=FALSE
    assert!(wrapper.contains("lock_objects = TRUE,"));
    assert!(wrapper.contains("lock_class = FALSE,"));
    assert!(wrapper.contains("cloneable = FALSE"));
    // No inherit or portable=FALSE by default
    assert!(!wrapper.contains("inherit ="));
    assert!(!wrapper.contains("portable = FALSE"));
}

#[test]
fn r6_active_binding_noexport_emits_field_internal() {
    // `#[miniextendr(noexport)]` on an R6 active binding emits a minimal
    // `#' @field name (internal)` description. The roxygen2 8.0.0 NEWS claims
    // `@field name NULL` is the opt-out, but in practice `r6_resolve_fields`
    // still warns "Undocumented R6 active binding" for that form because
    // `expected` is introspected from the class definition and is not pruned
    // in sync with the NULL-description discard. A minimal real description
    // satisfies the warning and keeps the binding clearly marked as internal.
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Sensor {
            pub fn new(v: f64, r: i32) -> Self { unimplemented!() }
            #[miniextendr(r6(active))]
            pub fn value(&self) -> f64 { unimplemented!() }
            #[miniextendr(r6(active), noexport)]
            pub fn raw_bytes(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    // Exported active binding: normal `@field name <description>` form.
    // (Default text when no doc comment provided is "Active binding.")
    assert!(
        wrapper.contains("#' @field value Active binding."),
        "exported active binding must have '@field value Active binding.'\n{}",
        wrapper
    );

    // Noexported active binding: minimal `(internal)` description.
    assert!(
        wrapper.contains("#' @field raw_bytes (internal)"),
        "noexported active binding must emit '@field raw_bytes (internal)'\n{}",
        wrapper
    );

    assert!(
        wrapper.contains(
            "#' @field value Active binding.\n\
             #' @field raw_bytes (internal)\n\
             #' @export\n\
             Sensor <- R6::R6Class"
        ),
        "active-binding fields must live on the class documentation block\n{}",
        wrapper
    );
    assert!(
        ["value", "raw_bytes"].iter().all(|binding| {
            let marker = format!("Sensor$set(\"active\", \"{binding}\"");
            let before_call = wrapper
                .split(&marker)
                .next()
                .expect("active binding call must exist");
            before_call
                .lines()
                .rev()
                .find(|line| !line.is_empty())
                .is_some_and(|line| !line.starts_with("#'"))
        }),
        "dynamic active-binding calls must not have adjacent roxygen blocks\n{}",
        wrapper
    );

    // Must NOT emit "Active binding." for the noexported binding, and must NOT
    // emit the old `NULL`-opt-out form.
    assert!(
        !wrapper.contains("#' @field raw_bytes Active binding."),
        "noexported active binding must not have regular description\n{}",
        wrapper
    );
    assert!(
        !wrapper.contains("#' @field raw_bytes NULL"),
        "noexported active binding must not use the (broken) NULL opt-out\n{}",
        wrapper
    );
}

#[test]
fn r6_active_binding_reuses_explicit_class_field_doc() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        /// @field reading User-authored class-level documentation.
        impl Sensor {
            pub fn new() -> Self { unimplemented!() }
            /// Getter prose must not create a duplicate field tag.
            #[miniextendr(r6(active, prop = "reading"))]
            pub fn value(&self) -> f64 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    assert_eq!(
        wrapper
            .matches("#' @field reading User-authored class-level documentation.")
            .count(),
        1,
        "an explicit class-level field must not be duplicated\n{}",
        wrapper
    );
    assert!(
        !wrapper.contains("#' @field value"),
        "the Rust getter name must not leak when prop renames the binding\n{}",
        wrapper
    );
}

#[test]
fn r6_active_binding_uses_renamed_property_in_generated_field_doc() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Sensor {
            pub fn new() -> Self { unimplemented!() }
            /// Current sensor reading.
            #[miniextendr(r6(active, prop = "reading"))]
            pub fn value(&self) -> f64 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    assert!(
        wrapper.contains("#' @field reading Current sensor reading."),
        "generated field documentation must use the R-visible property name\n{}",
        wrapper
    );
    assert!(
        !wrapper.contains("#' @field value"),
        "the Rust getter name must not leak into generated field documentation\n{}",
        wrapper
    );
}

#[test]
fn r6_active_binding_internal_emits_field_internal() {
    // `#[miniextendr(internal)]` on an R6 active binding emits the same
    // `#' @field name (internal)` form as `noexport`.
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Device {
            pub fn new() -> Self { unimplemented!() }
            #[miniextendr(r6(active))]
            pub fn status(&self) -> i32 { unimplemented!() }
            #[miniextendr(r6(active), internal)]
            pub fn debug_ptr(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    assert!(
        wrapper.contains("#' @field debug_ptr (internal)"),
        "internal active binding must emit '@field debug_ptr (internal)'\n{}",
        wrapper
    );
    assert!(
        !wrapper.contains("#' @field debug_ptr Active binding."),
        "internal active binding must not have regular description\n{}",
        wrapper
    );
    assert!(
        !wrapper.contains("#' @field debug_ptr NULL"),
        "internal active binding must not use the (broken) NULL opt-out\n{}",
        wrapper
    );
}

#[test]
fn r6_active_binding_setter_emits_preconditions_and_condition_guard() {
    // Audit 2026-07-06 finding 4: the setter branch of a combined
    // getter/setter active binding used to be a bare `.Call()` — no
    // `stopifnot` precondition (unlike the standalone `set_*` method) and no
    // `rust_condition_value` re-raise guard, so `obj$prop <- <bad value>`
    // silently discarded the transported conversion error.
    //
    // The setter's Rust parameter is deliberately NOT named `value` here: the
    // active binding's formal is always `value`, so the emitted checks must be
    // renamed to match (a check referencing `temp` would error at runtime).
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Temperature {
            pub fn new(celsius: f64) -> Self { unimplemented!() }
            #[miniextendr(r6(active))]
            pub fn celsius(&self) -> f64 { unimplemented!() }
            #[miniextendr(r6(setter, prop = "celsius"))]
            pub fn set_celsius(&mut self, temp: f64) { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    // Setter branch: precondition block referencing the binding's `value`
    // formal (not the Rust parameter name `temp`).
    assert!(
        wrapper.contains(
            "  } else {\n\
             \x20   stopifnot(\n\
             \x20     \"'value' must be numeric, logical, or raw\" = is.numeric(value) || is.logical(value) || is.raw(value),\n\
             \x20     \"'value' must have length 1\" = length(value) == 1L\n\
             \x20   )"
        ),
        "active-binding setter branch must emit the standalone setter's stopifnot block, renamed to 'value'\n{}",
        wrapper
    );
    // The standalone `set_celsius` method keeps its own `temp` formal; only
    // the active-binding `$set("active", ...)` block must not reference the Rust
    // parameter name.
    let active_section = wrapper
        .split("$set(\"active\"")
        .nth(1)
        .expect("wrapper must contain an active binding $set call");
    assert!(
        !active_section.contains("temp"),
        "active-binding preconditions must not reference the Rust parameter name 'temp'\n{}",
        active_section
    );

    // Setter branch: `.Call()` result must be captured and guarded so a
    // transported Rust condition re-raises instead of being discarded. In the
    // `$set("active", ...)` block the setter branch body sits at 4-space indent.
    assert!(
        wrapper.contains("    .val <- .Call(C_miniextendr_macros_Temperature__set_celsius"),
        "setter branch must capture the .Call result in .val\n{}",
        wrapper
    );

    // Getter branch of the combined binding gets the same guard as the
    // getter-only active-binding path (4-space indent inside the `$set` block).
    assert!(
        wrapper.contains("    .val <- .Call(C_miniextendr_macros_Temperature__celsius"),
        "getter branch must capture the .Call result in .val\n{}",
        wrapper
    );
}
// endregion

// region: S3 class system tests

#[test]
fn s3_wrapper_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
            pub fn zero() -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S3, item_impl);
    let wrapper = generate_s3_r_wrapper(&parsed);

    // Verify constructor (lowercase convention)
    assert!(wrapper.contains("new_counter <- function(value)"));
    assert!(wrapper.contains(".val <- .Call(C_miniextendr_macros_Counter__new"));
    assert!(wrapper.contains("structure(.val, class = \"Counter\")"));

    // Verify S3 generics are created
    assert!(wrapper.contains("get <- function(x, ...) UseMethod(\"get\")"));
    assert!(wrapper.contains("increment <- function(x, ...) UseMethod(\"increment\")"));

    // Verify S3 methods
    assert!(wrapper.contains("#' @method get Counter"));
    assert!(wrapper.contains("get.Counter <- function(x, ...)"));
    assert!(wrapper.contains(".Call(C_miniextendr_macros_Counter__get, .call = match.call(), x)"));

    assert!(wrapper.contains("#' @method increment Counter"));
    assert!(wrapper.contains("increment.Counter <- function(x, ...)"));
    assert!(
        wrapper.contains(".Call(C_miniextendr_macros_Counter__increment, .call = match.call(), x)")
    );

    // Verify static methods with prefix
    assert!(wrapper.contains("counter_zero <- function()"));

    // Verify class environment for trait namespace compatibility
    assert!(wrapper.contains("Counter <- new.env(parent = emptyenv())"));
}

#[test]
fn s3_wrapper_generic_override() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }

            #[miniextendr(s3(generic = "print"))]
            pub fn show(&self) -> String { unimplemented!() }

            #[miniextendr(s3(generic = "length"))]
            pub fn len(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S3, item_impl);
    let wrapper = generate_s3_r_wrapper(&parsed);

    // Should NOT create new generics for print and length (they exist in base R)
    assert!(!wrapper.contains("print <- function(x, ...) UseMethod(\"print\")"));
    assert!(!wrapper.contains("length <- function(x, ...) UseMethod(\"length\")"));

    // Should create S3 methods using the generic name
    assert!(wrapper.contains("#' @method print Counter"));
    assert!(wrapper.contains("print.Counter <- function(x, ...)"));
    assert!(wrapper.contains("#' @method length Counter"));
    assert!(wrapper.contains("length.Counter <- function(x, ...)"));
}

#[test]
fn s3_internal_keeps_s3method_export() {
    // Regression: #431. `internal` on an S3 impl must suppress NAMESPACE
    // export of the bare generic (so `R CMD check --as-cran` doesn't flag
    // an exported-but-undocumented generic) while keeping `S3method()`
    // registration on each method — otherwise dispatch on instances of the
    // class breaks for the package's own tests and any downstream caller.
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
        }
    };
    let mut attrs = default_impl_attrs(ClassSystem::S3);
    attrs.internal = true;
    let parsed = ParsedImpl::parse(attrs, item_impl).expect("parse");
    let wrapper = generate_s3_r_wrapper(&parsed);

    // Generic is defined but NOT exported.
    assert!(wrapper.contains("get <- function(x, ...) UseMethod(\"get\")"));
    assert!(
        !wrapper.contains("#' @export get"),
        "internal S3 must not export the bare generic"
    );

    // The method block keeps `#' @method get Counter` + `#' @export` so
    // roxygen2 emits `S3method(get, Counter)` in NAMESPACE.
    assert!(wrapper.contains("#' @method get Counter"));
    let method_export = wrapper
        .lines()
        .skip_while(|l| !l.contains("#' @method get Counter"))
        .take(6)
        .any(|l| l.trim() == "#' @export");
    assert!(
        method_export,
        "internal S3 must keep S3method() registration\n--- wrapper ---\n{wrapper}"
    );
}
// endregion

// region: S4 class system tests

#[test]
fn s4_wrapper_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S4, item_impl);
    let wrapper = generate_s4_r_wrapper(&parsed);

    // Verify setClass
    assert!(wrapper.contains("methods::setClass(\"Counter\", slots = c(ptr = \"externalptr\"))"));

    // Verify @importFrom methods
    assert!(wrapper.contains("@importFrom methods setClass setGeneric setMethod new"));

    // Verify @slot documentation
    assert!(wrapper.contains("@slot ptr External pointer to Rust `Counter` struct"));

    // Verify constructor
    assert!(wrapper.contains("Counter <- function(value)"));
    assert!(wrapper.contains(".val <- .Call(C_miniextendr_macros_Counter__new"));
    assert!(wrapper.contains("methods::new(\"Counter\", ptr = .val)"));

    // Verify S4 generics: guarded by a namespace-local exists() check. A bare
    // isGeneric() sees an attached installed copy and starves setMethod under
    // load_all(); isGeneric(where=) routes through package resolution and
    // breaks mid-install (findpack). exists() is a plain env lookup (#1158).
    assert!(wrapper.contains(
        "if (!exists(\"s4_get\", where = topenv(environment()), inherits = FALSE)) methods::setGeneric(\"s4_get\", function(x, ...) standardGeneric(\"s4_get\"))"
    ));
    assert!(wrapper.contains(
        "if (!exists(\"s4_increment\", where = topenv(environment()), inherits = FALSE)) methods::setGeneric(\"s4_increment\", function(x, ...) standardGeneric(\"s4_increment\"))"
    ));

    // Verify setMethod calls
    assert!(wrapper.contains("methods::setMethod(\"s4_get\", \"Counter\""));
    assert!(wrapper.contains("methods::setMethod(\"s4_increment\", \"Counter\""));

    // Verify @exportMethod tags
    assert!(wrapper.contains("@exportMethod s4_get"));
    assert!(wrapper.contains("@exportMethod s4_increment"));
}

#[test]
fn s4_wrapper_generic_override() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }

            #[miniextendr(s4(generic = "show"))]
            pub fn display(&self) { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S4, item_impl);
    let wrapper = generate_s4_r_wrapper(&parsed);

    // Should use "show" generic instead of "s4_display"
    assert!(wrapper.contains("methods::setMethod(\"show\", \"Counter\""));
    assert!(wrapper.contains("@exportMethod show"));
}
// endregion

// region: S7 class system tests

#[test]
fn s7_wrapper_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
            pub fn from_parts(a: i32, b: i32) -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Verify S7 class definition
    assert!(wrapper.contains("Counter <- S7::new_class(\"Counter\","));

    // Verify @importFrom S7
    assert!(
        wrapper
            .contains("@importFrom S7 new_class class_any new_object S7_object new_generic method")
    );

    // Verify properties
    assert!(wrapper.contains("properties = list("));
    assert!(wrapper.contains(".ptr = S7::class_any"));

    // Verify constructor with .ptr param (because from_parts returns Self)
    assert!(wrapper.contains("constructor = function(value, .ptr = NULL)"));
    assert!(wrapper.contains("if (!is.null(.ptr))"));
    assert!(wrapper.contains("S7::new_object(S7::S7_object(), .ptr = .ptr)"));
    assert!(wrapper.contains(".val <- .Call(C_miniextendr_macros_Counter__new"));
    assert!(wrapper.contains("S7::new_object(S7::S7_object(), .ptr = .val)"));

    // Verify S7 generics use the usability classifier (#1114): a plain base
    // closure like `get` must not be treated as a usable generic. The leading
    // statement is an `if (!base::exists(...))` (never a top-level assignment,
    // which roxygen2 would document), with the classifier in the `else if`,
    // wrapped in `local()` so `.mx_gen` doesn't leak into the namespace
    // (#1261 item 1).
    assert!(wrapper.contains("if (!base::exists(\"get\", mode = \"function\")) {"));
    assert!(wrapper.contains(
        "} else if (local({ .mx_gen <- base::get(\"get\", mode = \"function\"); !(inherits(.mx_gen, \"S7_generic\") || is.primitive(.mx_gen) || isTRUE(utils::isS3stdGeneric(.mx_gen)) || methods::isGeneric(\"get\")) })) {"
    ));
    assert!(
        wrapper.contains(
            "  get <- S7::new_generic(\"get\", \"x\", function(x, ...) S7::S7_dispatch())"
        )
    );
    // When the local generic masks an existing function, a class_any fallback
    // delegates ordinary calls to it (keeps `get(...)`/`var(1:10)` working).
    // `local()` + eager `.mx_masked <-` avoids the unforced-promise capture bug.
    assert!(
        wrapper.contains(
            "    S7::method(.mx_g, S7::class_any) <- function(x, ...) .mx_masked(x, ...)"
        )
    );
    assert!(wrapper.contains("if (!base::exists(\"increment\", mode = \"function\")) {"));
    assert!(wrapper.contains(
        "} else if (local({ .mx_gen <- base::get(\"increment\", mode = \"function\"); !(inherits(.mx_gen, \"S7_generic\") || is.primitive(.mx_gen) || isTRUE(utils::isS3stdGeneric(.mx_gen)) || methods::isGeneric(\"increment\")) })) {"
    ));
    assert!(wrapper.contains(
        "  increment <- S7::new_generic(\"increment\", \"x\", function(x, ...) S7::S7_dispatch())"
    ));

    // Verify S7 method definitions
    assert!(wrapper.contains("S7::method(get, Counter) <- function(x, ...)"));
    assert!(wrapper.contains("S7::method(increment, Counter) <- function(x, ...)"));
    assert!(
        wrapper.contains(".Call(C_miniextendr_macros_Counter__get, .call = match.call(), x@.ptr)")
    );
    assert!(
        wrapper.contains(
            ".Call(C_miniextendr_macros_Counter__increment, .call = match.call(), x@.ptr)"
        )
    );

    // Verify static methods
    assert!(wrapper.contains("Counter_from_parts <- function(a, b)"));
}

#[test]
fn s7_wrapper_generic_override() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }

            #[miniextendr(s7(generic = "base::print"))]
            pub fn show(&self) { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should use external generic for base::print
    assert!(wrapper.contains("print <- S7::new_external_generic(\"base\", \"print\")"));
    assert!(wrapper.contains("S7::method(print, Counter) <- function(x, ...)"));
}

/// `s7(no_shortcut)` suppresses the `<ClassName>_<method>` fast-dispatch
/// shortcut while keeping the S7 generic + method registration (#986).
#[test]
fn s7_no_shortcut_suppresses_shortcut() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            #[miniextendr(s7(no_shortcut))]
            pub fn increment(&mut self) { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // `get` still gets its shortcut.
    assert!(
        wrapper.contains("Counter_get <- function(self, ...)"),
        "un-annotated method should keep its shortcut, got:\n{}",
        wrapper
    );
    // `increment` does not.
    assert!(
        !wrapper.contains("Counter_increment <- function"),
        "no_shortcut must suppress the Counter_increment shortcut, got:\n{}",
        wrapper
    );
    // The generic + method registration for `increment` must still exist.
    assert!(
        wrapper.contains("S7::method(increment, Counter) <- function(x, ...)"),
        "no_shortcut must not remove the dispatched method, got:\n{}",
        wrapper
    );
}

/// A static method `r_name`-aliased onto an instance method's shortcut name is
/// a compile error (#986): both would emit `Counter_value` and the last one
/// would silently win at R load time.
#[test]
fn s7_shortcut_collision_with_static_is_error() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn value(&self) -> i32 { unimplemented!() }
            #[miniextendr(s7(r_name = "value"))]
            pub fn make() -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let err = check_s7_shortcut_collisions(&parsed).expect_err("collision must be detected");
    let msg = err.to_string();
    assert!(
        msg.contains("Counter_value") && msg.contains("no_shortcut"),
        "error should name the colliding function and suggest no_shortcut, got: {msg}"
    );
}

/// `s7(no_shortcut)` resolves a would-be shortcut/static collision.
#[test]
fn s7_no_shortcut_resolves_collision() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            #[miniextendr(s7(no_shortcut))]
            pub fn value(&self) -> i32 { unimplemented!() }
            #[miniextendr(s7(r_name = "value"))]
            pub fn make() -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    check_s7_shortcut_collisions(&parsed).expect("no_shortcut should resolve the collision");
}

/// Non-S7 class systems never collide on shortcut names (no shortcuts emitted).
#[test]
fn s7_shortcut_collision_check_ignores_other_class_systems() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn value(&self) -> i32 { unimplemented!() }
            #[miniextendr(env(r_name = "value"))]
            pub fn make() -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::Env, item_impl);
    check_s7_shortcut_collisions(&parsed).expect("non-S7 systems are exempt");
}
// endregion

// region: Label support tests

#[test]
fn label_affects_c_wrapper_names() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl_with_label(ClassSystem::Env, "basic", item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    // C wrapper names should include label
    assert!(wrapper.contains("C_miniextendr_macros_Counter_basic_new"));
    assert!(wrapper.contains("C_miniextendr_macros_Counter_basic_get"));
}
// endregion

// region: Parameter defaults tests

#[test]
fn parameter_defaults_in_r_wrapper() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }

            #[miniextendr(defaults(step = "1L", verbose = "FALSE"))]
            pub fn increment(&mut self, step: i32, verbose: bool) { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::Env, item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    // R wrapper should include defaults
    assert!(wrapper.contains("Counter$increment <- function(step = 1L, verbose = FALSE)"));
}

#[test]
fn parameter_defaults_r6() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }

            #[miniextendr(defaults(n = "10L"))]
            pub fn add(&mut self, n: i32) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    // R6 method should include default (now emitted as a `$set("public", ...)` block, #369)
    assert!(wrapper.contains("Counter$set(\"public\", \"add\", function(n = 10L)"));
}
// endregion

// region: Roxygen propagation tests

#[test]
fn roxygen_tags_propagate_to_wrapper() {
    // The roxygen system propagates explicit @tags (like @param, @return)
    // Plain doc comments are NOT automatically converted to @description
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            /// @param value Initial value
            /// @return The new Counter instance
            pub fn new(value: i32) -> Self { unimplemented!() }

            /// @return The counter value
            pub fn get(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::Env, item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    // For env-class, @param tags are converted to \describe blocks (avoids R CMD check warning)
    assert!(
        wrapper.contains("\\item{\\code{value}}{Initial value}"),
        "wrapper should contain param as \\describe item"
    );
    assert!(
        wrapper.contains("#' @return The counter value"),
        "wrapper should contain @return tag"
    );

    // Generated tags should be present
    assert!(wrapper.contains("#' @name Counter$new"));
    assert!(wrapper.contains("#' @name Counter$get"));
    assert!(wrapper.contains("#' @rdname Counter"));
    assert!(wrapper.contains("#' @source Generated by miniextendr"));
    assert!(wrapper.contains("#' @export"));
}
// endregion

// region: Return strategy tests (method chaining, Self returns)

#[test]
fn returns_self_method_chains_in_env() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }
            pub fn increment(&mut self) -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::Env, item_impl);
    let wrapper = generate_env_r_wrapper(&parsed);

    // increment returns Self, so it should return self (the R object, not the .Call result)
    // The return strategy should handle this
    assert!(wrapper.contains("Counter$increment <- function()"));
}

#[test]
fn self_ref_builder_returns_object_in_s3() {
    // `&mut self -> &mut Self` builder methods must compose under R's native
    // pipe: the generated S3 free function takes the object first and returns
    // the (same) object, so chaining works.
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Builder {
            pub fn new() -> Self { unimplemented!() }
            pub fn set_width(&mut self, w: i32) -> &mut Self { unimplemented!() }
            pub fn finish(&self) -> String { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S3, item_impl);
    let wrapper = generate_s3_r_wrapper(&parsed);

    // The builder step is a free function dispatching on `x` and returning `x`
    // (the same handle), not the raw `.Call()` result.
    assert!(wrapper.contains("set_width.Builder <- function(x, w, ...)"));
    let set_width_body = wrapper
        .split("set_width.Builder <- function(x, w, ...) {")
        .nth(1)
        .expect("set_width method body");
    let set_width_body = set_width_body.split('}').next().unwrap();
    assert!(
        set_width_body.lines().any(|l| l.trim() == "x"),
        "self-ref builder should return `x`, got:\n{set_width_body}"
    );
    // The terminal accessor returns the converted value directly.
    assert!(wrapper.contains("finish.Builder <- function(x, ...)"));
}

#[test]
fn self_ref_builder_uses_self_handle_return() {
    use crate::c_wrapper_builder::ReturnHandling;

    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Builder {
            pub fn set_width(&mut self, w: i32) -> &mut Self { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S3, item_impl);
    let method = parsed
        .methods
        .iter()
        .find(|m| m.ident == "set_width")
        .unwrap();
    assert!(method.returns_self_ref());
    assert!(!method.returns_self());

    let r_wrappers_const = syn::parse_quote! { R_WRAPPERS_TEST };
    let tokens =
        crate::miniextendr_impl::generate_method_c_wrapper(&parsed, method, &r_wrappers_const)
            .to_string();
    // The C wrapper hands the same SEXP handle back (no clone / rewrap).
    assert!(tokens.contains("self_sexp"));

    // And the strategy resolves to ChainableMutation (returns the object).
    assert_eq!(
        crate::ReturnStrategy::for_method(method),
        crate::ReturnStrategy::ChainableMutation
    );
    // Sanity: SelfHandle exists and is distinct.
    let _ = ReturnHandling::SelfHandle;
}

/// Audit A4: a static returning `Result<Self, E>` (e.g. `from_r`) must wrap its
/// successful return exactly like a bare-`Self`-returning constructor (e.g.
/// `new`) — a usable class object, not a bare `ExternalPtr`. The C wrapper still
/// raises on `Err` via the normal `Result` error path.
#[test]
fn result_self_static_wraps_like_constructor() {
    use crate::c_wrapper_builder::ReturnHandling;

    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl SerdeRPoint {
            pub fn new(x: f64, y: f64) -> Self { unimplemented!() }
            pub fn from_r(sexp: SEXP) -> Result<Self, String> { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::Env, item_impl);
    let from_r = parsed.methods.iter().find(|m| m.ident == "from_r").unwrap();

    // Detected as a `Result<Self, E>` return, not a bare `Self` return.
    assert!(from_r.returns_result_self());
    assert!(!from_r.returns_self());

    // C-wrapper return handling wraps `Ok(Self)` in an ExternalPtr, distinct
    // from the plain `Result<T, E> -> IntoR` path.
    assert!(matches!(
        crate::c_wrapper_builder::detect_return_handling(&from_r.sig.output),
        ReturnHandling::ResultExternalPtr
    ));

    // R-side strategy matches the bare-Self constructor path.
    let new_method = parsed.methods.iter().find(|m| m.ident == "new").unwrap();
    assert_eq!(
        crate::ReturnStrategy::for_method(from_r),
        crate::ReturnStrategy::for_method(new_method)
    );
    assert_eq!(
        crate::ReturnStrategy::for_method(from_r),
        crate::ReturnStrategy::ReturnSelf
    );

    // The generated Env wrapper wraps the successful result in the class,
    // exactly like `$new()`.
    let wrapper = generate_env_r_wrapper(&parsed);
    let from_r_body = wrapper
        .split("SerdeRPoint$from_r <- function(sexp) {")
        .nth(1)
        .expect("from_r method body")
        .split("\n}")
        .next()
        .expect("from_r method body");
    assert!(
        from_r_body.contains("class(.val) <- \"SerdeRPoint\""),
        "from_r should wrap its successful return like a class constructor, got:\n{from_r_body}"
    );
}

/// #1164: a static returning `Option<Self>` (e.g. `try_find`) must wrap its
/// successful return exactly like a bare-`Self`-returning constructor (e.g.
/// `new`) — a usable class object, not a bare `ExternalPtr`. The C wrapper
/// still raises on `None` via the normal `Option` error path. Symmetric with
/// the `Result<Self, E>` case above (audit A4).
#[test]
fn option_self_static_wraps_like_constructor() {
    use crate::c_wrapper_builder::ReturnHandling;

    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl OptionLookup {
            pub fn new(id: i32) -> Self { unimplemented!() }
            pub fn try_find(id: i32) -> Option<Self> { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::Env, item_impl);
    let try_find = parsed
        .methods
        .iter()
        .find(|m| m.ident == "try_find")
        .unwrap();

    // Detected as an `Option<Self>` return, not a bare `Self` return.
    assert!(try_find.returns_option_self());
    assert!(!try_find.returns_self());
    assert!(!try_find.returns_result_self());

    // C-wrapper return handling wraps `Some(Self)` in an ExternalPtr, distinct
    // from the plain `Option<T> -> IntoR` unwrap path.
    assert!(matches!(
        crate::c_wrapper_builder::detect_return_handling(&try_find.sig.output),
        ReturnHandling::OptionExternalPtr
    ));

    // R-side strategy matches the bare-Self constructor path.
    let new_method = parsed.methods.iter().find(|m| m.ident == "new").unwrap();
    assert_eq!(
        crate::ReturnStrategy::for_method(try_find),
        crate::ReturnStrategy::for_method(new_method)
    );
    assert_eq!(
        crate::ReturnStrategy::for_method(try_find),
        crate::ReturnStrategy::ReturnSelf
    );

    // The generated Env wrapper wraps the successful result in the class,
    // exactly like `$new()`.
    let wrapper = generate_env_r_wrapper(&parsed);
    let try_find_body = wrapper
        .split("OptionLookup$try_find <- function(id) {")
        .nth(1)
        .expect("try_find method body")
        .split("\n}")
        .next()
        .expect("try_find method body");
    assert!(
        try_find_body.contains("class(.val) <- \"OptionLookup\""),
        "try_find should wrap its successful return like a class constructor, got:\n{try_find_body}"
    );
}

#[test]
fn other_class_return_strategy_detects_bare_capitalized_type_only() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Builder {
            pub fn build(&self) -> Board { unimplemented!() }
            pub fn label(&self) -> String { unimplemented!() }
            pub fn many(&self) -> Vec<Board> { unimplemented!() }
            pub fn count(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let build = parsed.methods.iter().find(|m| m.ident == "build").unwrap();
    assert_eq!(build.returns_other_class().unwrap().to_string(), "Board");
    assert_eq!(
        crate::ReturnStrategy::for_method(build),
        crate::ReturnStrategy::ReturnOtherClass
    );

    for name in ["label", "many", "count"] {
        let method = parsed.methods.iter().find(|m| m.ident == name).unwrap();
        assert!(
            method.returns_other_class().is_none(),
            "{name} should not be treated as a scalar cross-class return"
        );
    }
    for name in ["label", "count"] {
        let method = parsed.methods.iter().find(|m| m.ident == name).unwrap();
        assert_eq!(
            crate::ReturnStrategy::for_method(method),
            crate::ReturnStrategy::Direct
        );
    }
    // `Vec<Board>` takes the list-shaped strategy (#1284), not the scalar one.
    let many = parsed.methods.iter().find(|m| m.ident == "many").unwrap();
    assert_eq!(
        crate::ReturnStrategy::for_method(many),
        crate::ReturnStrategy::ReturnOtherClassList
    );
}

#[test]
fn other_class_return_strategy_detects_option_and_result_containers() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Builder {
            pub fn maybe(&self) -> Option<Board> { unimplemented!() }
            pub fn checked(&self) -> Result<Board, String> { unimplemented!() }
            pub fn null_on_err(&self) -> Result<Board, ()> { unimplemented!() }
            pub fn maybe_self(&self) -> Option<Self> { unimplemented!() }
            pub fn maybe_scalar(&self) -> Option<i32> { unimplemented!() }
            pub fn many(&self) -> Vec<Board> { unimplemented!() }
            pub fn maybe_many(&self) -> Option<Vec<Board>> { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);

    for name in ["maybe", "checked"] {
        let method = parsed.methods.iter().find(|m| m.ident == name).unwrap();
        assert_eq!(
            method.returns_other_class().unwrap().to_string(),
            "Board",
            "{name} should resolve the Ok/Some type argument as the cross-class target"
        );
        assert_eq!(
            crate::ReturnStrategy::for_method(method),
            crate::ReturnStrategy::ReturnOtherClass
        );
    }

    for name in [
        "null_on_err",
        "maybe_self",
        "maybe_scalar",
        "many",
        "maybe_many",
    ] {
        let method = parsed.methods.iter().find(|m| m.ident == name).unwrap();
        assert!(
            method.returns_other_class().is_none(),
            "{name} should not be treated as a scalar cross-class return"
        );
    }

    // `Option<Self>` still takes the `ReturnSelf` strategy (checked before
    // `returns_other_class` in `ReturnStrategy::for_method`).
    let maybe_self = parsed
        .methods
        .iter()
        .find(|m| m.ident == "maybe_self")
        .unwrap();
    assert_eq!(
        crate::ReturnStrategy::for_method(maybe_self),
        crate::ReturnStrategy::ReturnSelf
    );

    for name in ["null_on_err", "maybe_scalar"] {
        let method = parsed.methods.iter().find(|m| m.ident == name).unwrap();
        assert_eq!(
            crate::ReturnStrategy::for_method(method),
            crate::ReturnStrategy::Direct
        );
    }

    // `Vec<Board>` / `Option<Vec<Board>>` take the list-shaped strategy (#1284).
    for name in ["many", "maybe_many"] {
        let method = parsed.methods.iter().find(|m| m.ident == name).unwrap();
        assert_eq!(
            crate::ReturnStrategy::for_method(method),
            crate::ReturnStrategy::ReturnOtherClassList
        );
    }
}

#[test]
fn list_return_strategy_detects_vec_of_class_shapes() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Builder {
            pub fn many(&self) -> Vec<Board> { unimplemented!() }
            pub fn maybe_many(&self) -> Option<Vec<Board>> { unimplemented!() }
            pub fn checked_many(&self) -> Result<Vec<Board>, String> { unimplemented!() }
            pub fn null_on_err_many(&self) -> Result<Vec<Board>, ()> { unimplemented!() }
            pub fn many_selves(&self) -> Vec<Self> { unimplemented!() }
            pub fn many_scalars(&self) -> Vec<i32> { unimplemented!() }
            pub fn many_strings(&self) -> Vec<String> { unimplemented!() }
            pub fn maybe_many_scalars(&self) -> Option<Vec<i32>> { unimplemented!() }
            pub fn many_options(&self) -> Vec<Option<Board>> { unimplemented!() }
            pub fn many_handles(&self) -> Vec<ExternalPtr<Board>> { unimplemented!() }
            pub fn nested_many(&self) -> Vec<Vec<Board>> { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);

    for name in ["many", "maybe_many", "checked_many"] {
        let method = parsed.methods.iter().find(|m| m.ident == name).unwrap();
        assert_eq!(
            method.returns_other_class_list().unwrap().to_string(),
            "Board",
            "{name} should resolve the Vec element type as the cross-class target"
        );
        assert_eq!(
            crate::ReturnStrategy::for_method(method),
            crate::ReturnStrategy::ReturnOtherClassList
        );
        // The two families never overlap on one method.
        assert!(method.returns_other_class().is_none());
    }

    for name in [
        "null_on_err_many",
        "many_selves",
        "many_scalars",
        "many_strings",
        "maybe_many_scalars",
        "many_options",
        "many_handles",
        "nested_many",
    ] {
        let method = parsed.methods.iter().find(|m| m.ident == name).unwrap();
        assert!(
            method.returns_other_class_list().is_none(),
            "{name} should not be treated as a list cross-class return"
        );
        assert_eq!(
            crate::ReturnStrategy::for_method(method),
            crate::ReturnStrategy::Direct
        );
    }
}

#[test]
fn r6_list_return_emits_write_time_list_marker() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Builder {
            pub fn new() -> Self { unimplemented!() }
            pub fn build_many(&self, n: i32) -> Vec<Board> { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);
    let body = wrapper
        .split("$set(\"public\", \"build_many\", function(")
        .nth(1)
        .expect("build_many R6 method")
        .split("\n})")
        .next()
        .expect("build_many method body");

    assert!(
        body.contains(".__MX_WRAP_LIST_RETURN_Board__(.val)"),
        "R6 Vec<Class> return should emit the write-time list marker, got:\n{body}"
    );
    assert!(
        !body.contains(".__MX_WRAP_RETURN_Board__"),
        "list returns must not emit the scalar marker (the scalar resolver \
         would consume it), got:\n{body}"
    );
}

#[test]
fn r6_other_class_return_emits_write_time_marker() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Builder {
            pub fn new() -> Self { unimplemented!() }
            pub fn build(&self) -> Board { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);
    let body = wrapper
        .split("$set(\"public\", \"build\", function(")
        .nth(1)
        .expect("build R6 method")
        .split("\n})")
        .next()
        .expect("build method body");

    assert!(
        body.contains(".__MX_WRAP_RETURN_Board__(.val)"),
        "R6 cross-class return should emit write-time wrap marker, got:\n{body}"
    );
    assert!(
        !body.contains("Builder$new(.ptr = .val)"),
        "cross-class returns must not wrap as the receiver class, got:\n{body}"
    );
}

#[test]
fn r6_class_without_constructor_still_has_ptr_initialize() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Landing {
            pub fn value(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);
    assert!(
        wrapper.contains("initialize = function(.ptr = NULL)"),
        "R6 classes without `new` need a .ptr initialize hatch, got:\n{wrapper}"
    );
}

#[test]
fn s7_class_without_constructor_still_has_ptr_constructor() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Landing {
            pub fn value(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let wrapper = generate_s7_r_wrapper(&parsed);
    assert!(
        wrapper.contains("constructor = function(.ptr = NULL)"),
        "S7 classes without `new` need a .ptr constructor hatch, got:\n{wrapper}"
    );
}

#[test]
fn returns_unit_method_in_r6() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new() -> Self { unimplemented!() }
            pub fn reset(&mut self) { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);
    let wrapper = generate_r6_r_wrapper(&parsed);

    // reset returns unit, should have invisible(self) for chaining
    assert!(wrapper.contains("Counter$set(\"public\", \"reset\", function()"));
}

/// Self-ref builders (`&mut self -> &mut Self`) on R6 must chain via
/// `invisible(self)`, NOT `Class$new(.ptr = .val)`.
///
/// R6 is already reference-semantic: the C wrapper returns the same
/// `private$.ptr` handle, so re-wrapping with `Class$new(.ptr = .val)` would
/// mint a *new* R6 environment around the same pointer — breaking object
/// identity (`obj |> set_x(1) |> get_x()` would read through a different
/// wrapper than `obj`). `ReturnStrategy::for_method` routes self-ref builders
/// to `ChainableMutation`, whose R6 tail is `invisible(self)`. This test pins
/// that wiring so a future regression that minted duplicate wrappers is caught.
#[test]
fn method_return_builder_r6_self_ref_chains_invisible_self() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Builder {
            pub fn new() -> Self { unimplemented!() }
            pub fn set_width(&mut self, w: i32) -> &mut Self { unimplemented!() }
            pub fn finish(&self) -> i32 { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::R6, item_impl);

    // Sanity: the strategy itself resolves to ChainableMutation, not ReturnSelf.
    let set_width = parsed
        .methods
        .iter()
        .find(|m| m.ident == "set_width")
        .unwrap();
    assert!(set_width.returns_self_ref());
    assert!(!set_width.returns_self());
    assert_eq!(
        crate::ReturnStrategy::for_method(set_width),
        crate::ReturnStrategy::ChainableMutation
    );

    let wrapper = generate_r6_r_wrapper(&parsed);

    // Isolate the body of the `set_width` R6 method (now a `$set("public", ...)` block, #369).
    let body = wrapper
        .split("$set(\"public\", \"set_width\", function(")
        .nth(1)
        .expect("set_width R6 method")
        .split("\n})")
        .next()
        .expect("set_width method body");

    // The self-ref builder chains the receiver — `invisible(self)`.
    assert!(
        body.lines().any(|l| l.trim() == "invisible(self)"),
        "R6 self-ref builder should chain via `invisible(self)`, got:\n{body}"
    );
    // It must NOT mint a new R6 wrapper around the same pointer.
    assert!(
        !body.contains("$new(.ptr"),
        "R6 self-ref builder must not re-wrap via `Builder$new(.ptr = .val)` \
         (would break object identity), got:\n{body}"
    );

    // The owned-`Self` constructor still uses the ReturnSelf path (`$new(.ptr`)
    // somewhere in the wrapper — confirms the two paths are genuinely distinct.
    let new_method = parsed.methods.iter().find(|m| m.ident == "new").unwrap();
    assert!(new_method.returns_self());
    assert_eq!(
        crate::ReturnStrategy::for_method(new_method),
        crate::ReturnStrategy::ReturnSelf
    );
}
// endregion

// region: vctrs class system tests

fn parse_impl_vctrs(vctrs_attrs: VctrsAttrs, code: syn::ItemImpl) -> ParsedImpl {
    let mut attrs = default_impl_attrs(ClassSystem::Vctrs);
    attrs.vctrs_attrs = vctrs_attrs;
    ParsedImpl::parse(attrs, code).expect("failed to parse impl")
}

#[test]
fn vctrs_wrapper_vctr_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Percent {
            pub fn new(x: f64) -> Vec<f64> { unimplemented!() }
            // Static helpers — &self is not allowed on vctrs impls (MXL120)
            pub fn scale(amounts: Vec<f64>, factor: f64) -> Vec<f64> { unimplemented!() }
        }
    };

    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Vctr,
        base: Some("double".to_string()),
        inherit_base_type: Some(false),
        ptype: None,
        abbr: Some("pct".to_string()),
    };

    let parsed = parse_impl_vctrs(vctrs_attrs, item_impl);
    let wrapper = generate_vctrs_r_wrapper(&parsed);

    // Verify constructor (vctrs convention: new_<class>)
    assert!(wrapper.contains("new_percent <- function(x)"));
    assert!(wrapper.contains(".val <- .Call(C_miniextendr_macros_Percent__new"));
    assert!(wrapper.contains("data <- .val"));
    assert!(
        wrapper.contains("vctrs::new_vctr(data, class = \"Percent\", inherit_base_type = FALSE)")
    );

    // Verify vec_ptype_abbr
    assert!(wrapper.contains("vec_ptype_abbr.Percent <- function(x, ...) \"pct\""));

    // Verify vec_ptype2 self-coercion
    assert!(wrapper.contains("#' @method vec_ptype2 Percent.Percent"));
    assert!(wrapper.contains("vec_ptype2.Percent.Percent <- function(x, y, ...) vctrs::new_vctr(double(), class = \"Percent\", inherit_base_type = FALSE)"));

    // Verify vec_cast self-coercion
    assert!(wrapper.contains("#' @method vec_cast Percent.Percent"));
    assert!(wrapper.contains("vec_cast.Percent.Percent <- function(x, to, ...) x"));

    // Verify static helper emitted as regular function
    assert!(wrapper.contains("percent_scale <- function(amounts, factor)"));

    // Verify imports
    assert!(wrapper.contains("@importFrom vctrs"));
}

#[test]
fn vctrs_wrapper_rcrd_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Rational {
            pub fn new(n: i32, d: i32) -> Vec<i32> { unimplemented!() }
            // Static helpers — &self is not allowed on vctrs impls (MXL120)
            pub fn numerator(n: Vec<i32>, _d: Vec<i32>) -> Vec<i32> { unimplemented!() }
        }
    };

    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Rcrd,
        base: None,
        inherit_base_type: None,
        ptype: None,
        abbr: Some("rat".to_string()),
    };

    let parsed = parse_impl_vctrs(vctrs_attrs, item_impl);
    let wrapper = generate_vctrs_r_wrapper(&parsed);

    // Verify constructor uses new_rcrd
    assert!(wrapper.contains("new_rational <- function(n, d)"));
    assert!(wrapper.contains("vctrs::new_rcrd(data, class = \"Rational\")"));

    // Verify vec_ptype_abbr
    assert!(wrapper.contains("vec_ptype_abbr.Rational <- function(x, ...) \"rat\""));

    // Verify vec_ptype2 for record uses x[0] pattern
    assert!(wrapper.contains("vec_ptype2.Rational.Rational <- function(x, y, ...) x[0]"));

    // Verify vec_cast self-coercion
    assert!(wrapper.contains("vec_cast.Rational.Rational <- function(x, to, ...) x"));
}

#[test]
fn vctrs_wrapper_list_of_full_snapshot() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl IntList {
            pub fn new(data: Vec<Vec<i32>>) -> Vec<Vec<i32>> { unimplemented!() }
            // Static helper — &self is not allowed on vctrs impls (MXL120)
            pub fn len(data: Vec<Vec<i32>>) -> i32 { unimplemented!() }
        }
    };

    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::ListOf,
        base: None,
        inherit_base_type: None,
        ptype: Some("integer()".to_string()),
        abbr: Some("int[]".to_string()),
    };

    let parsed = parse_impl_vctrs(vctrs_attrs, item_impl);
    let wrapper = generate_vctrs_r_wrapper(&parsed);

    // Verify constructor uses new_list_of with ptype
    assert!(wrapper.contains("new_intlist <- function(data)"));
    assert!(wrapper.contains("vctrs::new_list_of(data, class = \"IntList\", ptype = integer())"));

    // Verify vec_ptype_abbr
    assert!(wrapper.contains("vec_ptype_abbr.IntList <- function(x, ...) \"int[]\""));

    // Verify vec_ptype2 for list_of
    assert!(wrapper.contains("vec_ptype2.IntList.IntList <- function(x, y, ...) vctrs::new_list_of(list(), class = \"IntList\", ptype = integer())"));

    // Verify vec_cast self-coercion
    assert!(wrapper.contains("vec_cast.IntList.IntList <- function(x, to, ...) x"));
}

#[test]
fn vctrs_wrapper_no_abbr() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Simple {
            pub fn new(x: f64) -> Vec<f64> { unimplemented!() }
        }
    };

    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Vctr,
        base: None,
        inherit_base_type: None,
        ptype: None,
        abbr: None, // No abbreviation
    };

    let parsed = parse_impl_vctrs(vctrs_attrs, item_impl);
    let wrapper = generate_vctrs_r_wrapper(&parsed);

    // Should NOT have vec_ptype_abbr
    assert!(!wrapper.contains("vec_ptype_abbr.Simple"));

    // But should still have ptype2 and cast
    assert!(wrapper.contains("vec_ptype2.Simple.Simple"));
    assert!(wrapper.contains("vec_cast.Simple.Simple"));
}

#[test]
fn vctrs_protocol_method_override() {
    // vctrs protocol overrides must use static methods (MXL120 rejects &self receivers).
    // The vctrs(format) attribute maps a static method to the format.<Class> S3 method.
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Currency {
            pub fn new(amounts: Vec<f64>) -> Vec<f64> { unimplemented!() }

            // Static helper: regular function named currency_symbol
            pub fn symbol(amounts: Vec<f64>) -> Vec<String> { unimplemented!() }

            // vctrs protocol override: maps to format.Currency
            #[miniextendr(vctrs(format))]
            pub fn format_currency(amounts: Vec<f64>) -> Vec<String> { unimplemented!() }
        }
    };

    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Vctr,
        base: Some("double".to_string()),
        inherit_base_type: None,
        ptype: None,
        abbr: Some("$".to_string()),
    };

    let parsed = parse_impl_vctrs(vctrs_attrs, item_impl);
    let wrapper = generate_vctrs_r_wrapper(&parsed);

    // format_currency method should be generated as format.Currency, not format_currency.Currency
    assert!(wrapper.contains("#' @method format Currency"));
    // Protocol methods get a trailing `...` so `format(x, nsmall = 2)` doesn't error
    // with "unused argument (nsmall = 2)" when R dispatches to format.Currency.
    assert!(wrapper.contains("format.Currency <- function(amounts, ...)"));

    // Should NOT create a new S3 generic for "format" (it's a base R function)
    assert!(!wrapper.contains("format <- function(x, ...) UseMethod(\"format\")"));

    // symbol static helper (non-protocol) should keep fixed formals — no trailing `...`
    assert!(wrapper.contains("currency_symbol <- function(amounts)"));
    assert!(!wrapper.contains("currency_symbol <- function(amounts, ...)"));
}

/// #1180: a `#[miniextendr(vctrs(...), noexport)]` impl must not contribute to
/// any Rd page — the self-coercion blocks (`vec_ptype_abbr` / `vec_ptype2` /
/// `vec_cast`) and static-method docs all collapse to `@noRd`, matching the
/// other five class generators' `class_has_no_rd || (noexport && !internal)`
/// fold. `@method` + `@export` S3method() registration pairs survive
/// (NAMESPACE dispatch plumbing, not Rd/export() surface) and the R functions
/// themselves are still emitted.
#[test]
fn vctrs_noexport_suppresses_all_rd() {
    // No constructor on purpose: the class-doc header goes through the shared
    // ClassDocBuilder (audit A10's path); this test pins the blocks the vctrs
    // generator owns itself.
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Hidden {
            // Static helper — &self is not allowed on vctrs impls (MXL120)
            pub fn payload_sum(values: Vec<f64>) -> f64 { unimplemented!() }
        }
    };
    let mut attrs = default_impl_attrs(ClassSystem::Vctrs);
    attrs.vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Vctr,
        base: Some("double".to_string()),
        inherit_base_type: None,
        ptype: None,
        abbr: Some("hid".to_string()),
    };
    attrs.noexport = true;
    let parsed = ParsedImpl::parse(attrs, item_impl).expect("failed to parse impl");
    let wrapper = generate_vctrs_r_wrapper(&parsed);

    assert!(
        wrapper.contains("#' @noRd"),
        "noexport must emit @noRd, got:\n{wrapper}"
    );
    for tag in ["@rdname", "@title", "@description", "@param"] {
        assert!(
            !wrapper.contains(tag),
            "noexport must not emit `{tag}` (no Rd contribution), got:\n{wrapper}"
        );
    }
    // The R functions themselves are still emitted (callable via :::)…
    assert!(wrapper.contains("vec_ptype_abbr.Hidden <- function(x, ...) \"hid\""));
    assert!(wrapper.contains("vec_ptype2.Hidden.Hidden <- function(x, y, ...)"));
    assert!(wrapper.contains("vec_cast.Hidden.Hidden <- function(x, to, ...) x"));
    // …the @method + @export S3method() registration pairs survive the gate
    // (dispatch from the vctrs namespace requires them; roxygen2 warns on
    // recognized-but-unregistered S3 methods)…
    assert!(wrapper.contains("#' @method vec_ptype_abbr Hidden\n#' @export"));
    assert!(wrapper.contains("#' @method vec_ptype2 Hidden.Hidden\n#' @export"));
    assert!(wrapper.contains("#' @method vec_cast Hidden.Hidden\n#' @export"));
    // …while the plain-function static helper gets @noRd and no @export at all.
    assert!(
        wrapper.contains("#' @noRd\nhidden_payload_sum <- function(values)"),
        "static helper must be @noRd with no @export, got:\n{wrapper}"
    );
}

/// #1180: a user-written `@noRd` on the impl block engages the same gate as
/// `noexport` — every block (constructor included, via ClassDocBuilder)
/// collapses to `@noRd`.
#[test]
fn vctrs_class_no_rd_tag_suppresses_all_rd() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        /// @noRd
        impl Quiet {
            pub fn new(values: Vec<f64>) -> Vec<f64> { unimplemented!() }
        }
    };
    let mut attrs = default_impl_attrs(ClassSystem::Vctrs);
    attrs.vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Vctr,
        base: Some("double".to_string()),
        inherit_base_type: None,
        ptype: None,
        abbr: Some("qt".to_string()),
    };
    let parsed = ParsedImpl::parse(attrs, item_impl).expect("failed to parse impl");
    let wrapper = generate_vctrs_r_wrapper(&parsed);

    assert!(
        wrapper.contains("#' @noRd"),
        "@noRd class must emit @noRd, got:\n{wrapper}"
    );
    for tag in ["@rdname", "@title"] {
        assert!(
            !wrapper.contains(tag),
            "@noRd class must not emit `{tag}`, got:\n{wrapper}"
        );
    }
    // Constructor is emitted but carries no @export (ClassDocBuilder gates it).
    assert!(wrapper.contains("new_quiet <- function(values)"));
    assert!(
        !wrapper.contains("#' @export\nnew_quiet <- function"),
        "@noRd class constructor must not be exported, got:\n{wrapper}"
    );
    // Self-coercion keeps its S3method() registration pair.
    assert!(wrapper.contains("#' @method vec_ptype2 Quiet.Quiet\n#' @export"));
    assert!(wrapper.contains("vec_ptype2.Quiet.Quiet <- function(x, y, ...)"));
}

/// Companion: `#[miniextendr(vctrs(...), internal)]` stays documented — the
/// self-coercion blocks keep `@rdname`/`@param`, no `@noRd` anywhere; the
/// class-level export() surface is suppressed while S3method() registration
/// stays (#431 semantics: internal keeps dispatch working).
#[test]
fn vctrs_internal_stays_documented() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Gauge {
            pub fn new(values: Vec<f64>) -> Vec<f64> { unimplemented!() }
        }
    };
    let mut attrs = default_impl_attrs(ClassSystem::Vctrs);
    attrs.vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Vctr,
        base: Some("double".to_string()),
        inherit_base_type: None,
        ptype: None,
        abbr: Some("gau".to_string()),
    };
    attrs.internal = true;
    let parsed = ParsedImpl::parse(attrs, item_impl).expect("failed to parse impl");
    let wrapper = generate_vctrs_r_wrapper(&parsed);

    assert!(
        wrapper.contains("#' @rdname Gauge"),
        "internal must keep @rdname (stays documented), got:\n{wrapper}"
    );
    assert!(
        wrapper.contains("#' @keywords internal"),
        "internal must add @keywords internal on the class block, got:\n{wrapper}"
    );
    assert!(
        !wrapper.contains("#' @noRd"),
        "internal must NOT emit @noRd, got:\n{wrapper}"
    );
    assert!(
        !wrapper.contains("#' @export\nnew_gauge <- function"),
        "internal class constructor must not be exported, got:\n{wrapper}"
    );
    assert!(
        wrapper.contains("#' @method vec_ptype2 Gauge.Gauge"),
        "internal must keep the S3method() registration pair, got:\n{wrapper}"
    );
}
// endregion

// region: S7 property class type tests

#[test]
fn s7_property_class_types() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Range {
            pub fn new(start: f64, end: f64) -> Self { unimplemented!() }

            #[miniextendr(s7(getter))]
            pub fn length(&self) -> f64 { unimplemented!() }

            #[miniextendr(s7(getter, prop = "midpoint"))]
            pub fn get_midpoint(&self) -> f64 { unimplemented!() }

            #[miniextendr(s7(setter, prop = "midpoint"))]
            pub fn set_midpoint(&mut self, value: f64) { unimplemented!() }

            #[miniextendr(s7(getter))]
            pub fn is_valid(&self) -> bool { unimplemented!() }

            #[miniextendr(s7(getter))]
            pub fn name(&self) -> String { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Debug: print the generated wrapper
    eprintln!("Generated S7 wrapper:\n{}", wrapper);

    // Verify class types are included in property definitions
    assert!(
        wrapper.contains("length = S7::new_property(class = S7::class_double, getter ="),
        "length property missing class type"
    );
    assert!(
        wrapper.contains("midpoint = S7::new_property(class = S7::class_double, getter ="),
        "midpoint property missing class type"
    );
    assert!(
        wrapper.contains("is_valid = S7::new_property(class = S7::class_logical, getter ="),
        "is_valid property missing class type"
    );
    assert!(
        wrapper.contains("name = S7::new_property(class = S7::class_character, getter ="),
        "name property missing class type"
    );

    // Verify imports include the class types
    assert!(
        wrapper.contains("class_double"),
        "missing class_double import"
    );
    assert!(
        wrapper.contains("class_logical"),
        "missing class_logical import"
    );
    assert!(
        wrapper.contains("class_character"),
        "missing class_character import"
    );
}

#[test]
fn s7_property_option_class_type() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Container {
            pub fn new() -> Self { unimplemented!() }

            #[miniextendr(s7(getter))]
            pub fn maybe_value(&self) -> Option<i32> { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Option<i32> should map to NULL | S7::class_integer
    assert!(
        wrapper
            .contains("maybe_value = S7::new_property(class = NULL | S7::class_integer, getter =")
    );
}

#[test]
fn s7_property_mirrors_s7_tests_rs() {
    // This test mirrors the exact structure of s7_tests.rs::S7Range
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl S7Range {
            pub fn new(start: f64, end: f64) -> Self {
                S7Range { start, end }
            }

            #[miniextendr(s7(getter))]
            pub fn length(&self) -> f64 {
                self.end - self.start
            }

            #[miniextendr(s7(getter, prop = "midpoint"))]
            pub fn get_midpoint(&self) -> f64 {
                (self.start + self.end) / 2.0
            }

            #[miniextendr(s7(setter, prop = "midpoint"))]
            pub fn set_midpoint(&mut self, value: f64) {
                let half_length = (self.end - self.start) / 2.0;
                self.start = value - half_length;
                self.end = value + half_length;
            }

            pub fn s7_start(&self) -> f64 {
                self.start
            }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);

    // Debug: check method attributes
    for method in &parsed.methods {
        if method.ident == "length" {
            eprintln!(
                "length method attrs: s7_getter={}, s7_setter={}",
                method.method_attrs.s7.getter, method.method_attrs.s7.setter
            );
            eprintln!("length return type: {:?}", method.sig.output);
        }
    }

    let wrapper = generate_s7_r_wrapper(&parsed);
    eprintln!("Generated wrapper for S7Range:\n{}", wrapper);

    // Should have class type for length property
    assert!(
        wrapper.contains("length = S7::new_property(class = S7::class_double"),
        "length property should have class = S7::class_double"
    );
}
// endregion

// region: S7 type mapping tests

#[test]
fn s7_type_mapping_scalars() {
    use super::rust_type_to_s7_class;

    // Integer types
    let ty: syn::Type = syn::parse_quote!(i32);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_integer".to_string())
    );

    let ty: syn::Type = syn::parse_quote!(i16);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_integer".to_string())
    );

    // Float types
    let ty: syn::Type = syn::parse_quote!(f64);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_double".to_string())
    );

    let ty: syn::Type = syn::parse_quote!(f32);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_double".to_string())
    );

    // Logical
    let ty: syn::Type = syn::parse_quote!(bool);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_logical".to_string())
    );

    // Raw
    let ty: syn::Type = syn::parse_quote!(u8);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_raw".to_string())
    );

    // Character
    let ty: syn::Type = syn::parse_quote!(String);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_character".to_string())
    );
}

#[test]
fn s7_type_mapping_references() {
    use super::rust_type_to_s7_class;

    // &str maps to character
    let ty: syn::Type = syn::parse_quote!(&str);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_character".to_string())
    );
}

#[test]
fn s7_type_mapping_vec() {
    use super::rust_type_to_s7_class;

    // Vec<i32> -> class_integer
    let ty: syn::Type = syn::parse_quote!(Vec<i32>);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_integer".to_string())
    );

    // Vec<f64> -> class_double
    let ty: syn::Type = syn::parse_quote!(Vec<f64>);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_double".to_string())
    );

    // Vec<String> -> class_character
    let ty: syn::Type = syn::parse_quote!(Vec<String>);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_character".to_string())
    );
}

#[test]
fn s7_type_mapping_option() {
    use super::rust_type_to_s7_class;

    // Option<i32> -> NULL | class_integer
    let ty: syn::Type = syn::parse_quote!(Option<i32>);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("NULL | S7::class_integer".to_string())
    );

    // Option<String> -> NULL | class_character
    let ty: syn::Type = syn::parse_quote!(Option<String>);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("NULL | S7::class_character".to_string())
    );
}

#[test]
fn s7_type_mapping_result() {
    use super::rust_type_to_s7_class;

    // Result<i32, E> -> class_integer (from Ok type)
    let ty: syn::Type = syn::parse_quote!(Result<i32, String>);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some("S7::class_integer".to_string())
    );
}

#[test]
fn s7_type_mapping_unknown() {
    use super::rust_type_to_s7_class;

    // Bare PascalCase types emit the quiet-fallback CLASS_REF placeholder
    // (#203). The cdylib resolver substitutes it with the registered R class
    // name for S7 types, or `S7::class_any` for unregistered / non-S7 ones.
    let ty: syn::Type = syn::parse_quote!(MyCustomType);
    assert_eq!(
        rust_type_to_s7_class(&ty),
        Some(".__MX_CLASS_REF_OR_ANY_MyCustomType__".to_string())
    );

    // Generic types (path with args) still return None so the caller omits
    // the `class =` entirely — S7 defaults to class_any in that case too.
    let ty: syn::Type = syn::parse_quote!(ExternalPtr<Foo>);
    assert_eq!(rust_type_to_s7_class(&ty), None);
}
// endregion

// region: S7 Phase 2: validation/defaults/required/frozen/deprecated tests

#[test]
fn s7_property_default_value() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Range {
            #[miniextendr(s7(getter, default = "0.0"))]
            pub fn score(&self) -> f64 { self.score }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should include default = 0.0 in property definition
    assert!(
        wrapper.contains("default = 0.0"),
        "Expected default value in property, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_property_required() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl User {
            #[miniextendr(s7(getter, required))]
            pub fn id(&self) -> String { self.id.clone() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should include error message for required property
    assert!(
        wrapper.contains("@id is required"),
        "Expected required error in property, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("stop("),
        "Expected stop() call for required property, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_property_frozen() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Config {
            #[miniextendr(s7(getter, frozen))]
            pub fn created_at(&self) -> f64 { self.created_at }

            #[miniextendr(s7(setter, prop = "created_at"))]
            pub fn set_created_at(&mut self, value: f64) { self.created_at = value; }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should include frozen check in setter
    assert!(
        wrapper.contains("is frozen"),
        "Expected frozen error message in setter, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("cannot be modified"),
        "Expected frozen check in setter, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_property_deprecated() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Legacy {
            #[miniextendr(s7(getter, deprecated = "Use 'value' instead"))]
            pub fn old_value(&self) -> i32 { self.value }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should include deprecation warning in getter
    assert!(
        wrapper.contains("is deprecated"),
        "Expected deprecation warning in getter, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("Use 'value' instead"),
        "Expected deprecation message in getter, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("warning("),
        "Expected warning() call for deprecated property, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_property_validator() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Score {
            #[miniextendr(s7(getter))]
            pub fn score(&self) -> f64 { self.score }

            #[miniextendr(s7(validate, prop = "score"))]
            pub fn validate_score(value: f64) -> Result<(), String> {
                if value < 0.0 || value > 100.0 {
                    Err("score must be between 0 and 100".into())
                } else {
                    Ok(())
                }
            }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should include validator function in property
    assert!(
        wrapper.contains("validator = function(value)"),
        "Expected validator in property, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("C_miniextendr_macros_Score__validate_score"),
        "Expected validator C function call, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_property_combined_patterns() {
    // Test combining default + deprecated
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Config {
            #[miniextendr(s7(getter, default = "\"default\"", deprecated = "Will be removed"))]
            pub fn legacy_name(&self) -> String { self.name.clone() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should have both default and deprecation
    assert!(
        wrapper.contains("default = \"default\""),
        "Expected default value, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("Will be removed"),
        "Expected deprecation message, got:\n{}",
        wrapper
    );
}
// endregion

// region: S7 Phase 3: Generic dispatch control tests

#[test]
fn s7_generic_no_dots() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            #[miniextendr(s7(no_dots))]
            pub fn length(&self) -> i32 { self.len }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should have generic without ... in signature
    assert!(
        wrapper.contains("function(x) S7::S7_dispatch()"),
        "Expected no_dots generic, got:\n{}",
        wrapper
    );
    // Should NOT have ... in the generic definition
    assert!(
        !wrapper.contains("function(x, ...) S7::S7_dispatch()"),
        "Expected no_dots to remove ..., got:\n{}",
        wrapper
    );
}

#[test]
fn s7_generic_multi_dispatch() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Dog {
            #[miniextendr(s7(dispatch = "x,y"))]
            pub fn compare(&self, other: i32) -> i32 { 0 }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should have c("x", "y") dispatch args
    assert!(
        wrapper.contains(r#"c("x", "y")"#),
        "Expected multi-dispatch args, got:\n{}",
        wrapper
    );
    // Should have function(x, y, ...) signature
    assert!(
        wrapper.contains("function(x, y, ...) S7::S7_dispatch()"),
        "Expected multi-dispatch signature, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_generic_multi_dispatch_no_dots() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Matrix {
            #[miniextendr(s7(dispatch = "x,y", no_dots))]
            pub fn multiply(&self, other: i32) -> i32 { 0 }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should have c("x", "y") dispatch args
    assert!(
        wrapper.contains(r#"c("x", "y")"#),
        "Expected multi-dispatch args, got:\n{}",
        wrapper
    );
    // Should have function(x, y) signature without ...
    assert!(
        wrapper.contains("function(x, y) S7::S7_dispatch()"),
        "Expected strict multi-dispatch signature, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_generic_fallback() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Printer {
            #[miniextendr(s7(fallback))]
            pub fn describe(&self) -> String { "unknown".to_string() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should register method for class_any instead of Printer
    assert!(
        wrapper.contains("S7::method(describe, S7::class_any)"),
        "Expected fallback to class_any, got:\n{}",
        wrapper
    );
    // Fallback should use safe self extraction (inherits check), not raw x@.ptr
    assert!(
        wrapper.contains("inherits(x, \"S7_object\")"),
        "Expected safe self extraction with inherits check, got:\n{}",
        wrapper
    );
    assert!(
        !wrapper.contains(".Call(wrap__Printer__describe, x@.ptr,"),
        "Fallback should NOT use raw x@.ptr in .Call, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_generic_override_fallback() {
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Printer {
            #[miniextendr(s7(generic = "base::print", fallback))]
            pub fn print_it(&self) -> String { "printed".to_string() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Generic-override + fallback should use class_any, not Printer
    assert!(
        wrapper.contains("S7::method(print, S7::class_any)"),
        "Expected generic-override fallback to class_any, got:\n{}",
        wrapper
    );
    // Should also use safe self extraction
    assert!(
        wrapper.contains("inherits(x, \"S7_object\")"),
        "Expected safe self extraction in generic-override fallback, got:\n{}",
        wrapper
    );
}
// endregion

// region: S7 Phase 4: convert() methods from Rust From/TryFrom patterns

#[test]
fn s7_convert_from() {
    // Test convert_from: converts FROM another type TO this type
    // Pattern: static method takes OtherType and returns Self
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Point3D {
            pub fn new(x: f64, y: f64, z: f64) -> Self { Self { x, y, z } }

            #[miniextendr(s7(convert_from = "Point2D"))]
            pub fn from_2d(p: Point2D) -> Self { Self { x: 0.0, y: 0.0, z: 0.0 } }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should generate S7::method(convert, list(.__MX_CLASS_REF_Point2D__, Point3D))
    // from_type is a cross-reference → placeholder; resolver replaces at cdylib write time.
    assert!(
        wrapper.contains("S7::method(convert, list(.__MX_CLASS_REF_Point2D__, Point3D))"),
        "Expected placeholder for cross-ref in convert method, got:\n{}",
        wrapper
    );
    // The method should call the C wrapper with from@.ptr
    assert!(
        wrapper.contains("from@.ptr"),
        "Expected from@.ptr in convert call, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_convert_to() {
    // Test convert_to: converts FROM this type TO another type
    // Pattern: instance method takes &self and returns OtherType
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Point3D {
            pub fn new(x: f64, y: f64, z: f64) -> Self { Self { x, y, z } }

            #[miniextendr(s7(convert_to = "Point2D"))]
            pub fn to_2d(&self) -> Point2D { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should generate S7::method(convert, list(Point3D, .__MX_CLASS_REF_Point2D__))
    // to_type is a cross-reference → placeholder; resolver replaces at cdylib write time.
    assert!(
        wrapper.contains("S7::method(convert, list(Point3D, .__MX_CLASS_REF_Point2D__))"),
        "Expected placeholder for cross-ref in convert method, got:\n{}",
        wrapper
    );
    // The method should call the C wrapper with from@.ptr
    assert!(
        wrapper.contains("from@.ptr"),
        "Expected from@.ptr in convert call, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_convert_bidirectional() {
    // Test both convert_from and convert_to on the same class
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Celsius {
            pub fn new(value: f64) -> Self { Self { value } }

            #[miniextendr(s7(convert_from = "Fahrenheit"))]
            pub fn from_fahrenheit(f: Fahrenheit) -> Self { unimplemented!() }

            #[miniextendr(s7(convert_to = "Fahrenheit"))]
            pub fn to_fahrenheit(&self) -> Fahrenheit { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, impl_code);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Both cross-references use placeholders; resolver replaces at cdylib write time.
    assert!(
        wrapper.contains("S7::method(convert, list(.__MX_CLASS_REF_Fahrenheit__, Celsius))"),
        "Expected placeholder for Fahrenheit in convert_from, got:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains("S7::method(convert, list(Celsius, .__MX_CLASS_REF_Fahrenheit__))"),
        "Expected placeholder for Fahrenheit in convert_to, got:\n{}",
        wrapper
    );
}

#[test]
fn s7_convert_from_and_to_mutually_exclusive() {
    // Test that specifying both convert_from and convert_to on the same method is an error
    let impl_code: syn::ItemImpl = syn::parse_quote! {
        impl Converter {
            pub fn new() -> Self { Self {} }

            // This should be invalid - can't have both on same method
            #[miniextendr(s7(convert_from = "TypeA", convert_to = "TypeB"))]
            pub fn invalid_convert(&self) -> TypeB { unimplemented!() }
        }
    };

    // This should fail during parsing/validation
    let result = std::panic::catch_unwind(|| parse_impl(ClassSystem::S7, impl_code));

    // The parse_impl function should panic or return an error for this invalid config
    // If it doesn't panic, we need to check the behavior differently
    if result.is_ok() {
        // If parsing succeeded, the validation should have caught this
        // The current implementation validates during parse_impl
        panic!("Expected error when both convert_from and convert_to are specified on same method");
    }
}

#[test]
fn s7_wrapper_parent() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Circle {
            pub fn new(radius: f64) -> Self { unimplemented!() }
            pub fn area(&self) -> f64 { unimplemented!() }
        }
    };

    let mut attrs = default_impl_attrs(ClassSystem::S7);
    attrs.s7_parent = Some("Shape".to_string());
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    let wrapper = generate_s7_r_wrapper(&parsed);

    // parent = uses a placeholder; resolver replaces at cdylib write time
    assert!(
        wrapper.contains("Circle <- S7::new_class(\"Circle\", parent = .__MX_CLASS_REF_Shape__,")
    );
}

#[test]
fn s7_wrapper_abstract() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Shape {
            pub fn new() -> Self { unimplemented!() }
        }
    };

    let mut attrs = default_impl_attrs(ClassSystem::S7);
    attrs.s7_abstract = true;
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    let wrapper = generate_s7_r_wrapper(&parsed);

    assert!(wrapper.contains("abstract = TRUE,"));
}

#[test]
fn s7_wrapper_defaults_no_parent_no_abstract() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl MyClass {
            pub fn new() -> Self { unimplemented!() }
        }
    };

    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let wrapper = generate_s7_r_wrapper(&parsed);

    // No parent or abstract by default
    assert!(!wrapper.contains("parent ="));
    assert!(!wrapper.contains("abstract = TRUE"));
}
// endregion

// region: ImplAttrs parsing tests

#[test]
fn parse_r6_with_options() {
    let attrs: ImplAttrs =
        syn::parse_str("r6(inherit = \"ParentClass\", cloneable, lock_class = true)").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::R6);
    assert_eq!(attrs.r6_inherit, Some("ParentClass".to_string()));
    assert_eq!(attrs.r6_cloneable, Some(true));
    assert_eq!(attrs.r6_lock_class, Some(true));
    assert_eq!(attrs.r6_portable, None);
    assert_eq!(attrs.r6_lock_objects, None);
}

#[test]
fn parse_r6_plain() {
    let attrs: ImplAttrs = syn::parse_str("r6").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::R6);
    assert_eq!(attrs.r6_inherit, None);
    assert_eq!(attrs.r6_cloneable, None);
}

#[test]
fn parse_s7_with_parent() {
    let attrs: ImplAttrs = syn::parse_str("s7(parent = \"Shape\")").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::S7);
    assert_eq!(attrs.s7_parent, Some("Shape".to_string()));
    assert!(!attrs.s7_abstract);
}

#[test]
fn parse_s7_abstract() {
    let attrs: ImplAttrs = syn::parse_str("s7(abstract)").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::S7);
    assert!(attrs.s7_abstract);
}

#[test]
fn parse_s7_parent_and_abstract() {
    let attrs: ImplAttrs = syn::parse_str("s7(parent = \"Base\", abstract)").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::S7);
    assert_eq!(attrs.s7_parent, Some("Base".to_string()));
    assert!(attrs.s7_abstract);
}
// endregion

// region: r_data_accessors parsing tests

#[test]
fn parse_r6_with_r_data_accessors() {
    let attrs: ImplAttrs = syn::parse_str("r6(r_data_accessors)").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::R6);
    assert!(attrs.r_data_accessors);
}

#[test]
fn parse_r6_with_r_data_accessors_and_options() {
    let attrs: ImplAttrs = syn::parse_str("r6(cloneable, lock_class, r_data_accessors)").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::R6);
    assert!(attrs.r_data_accessors);
    assert_eq!(attrs.r6_cloneable, Some(true));
    assert_eq!(attrs.r6_lock_class, Some(true));
}

#[test]
fn parse_s7_with_r_data_accessors() {
    let attrs: ImplAttrs = syn::parse_str("s7(r_data_accessors)").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::S7);
    assert!(attrs.r_data_accessors);
}

#[test]
fn parse_r6_without_r_data_accessors() {
    let attrs: ImplAttrs = syn::parse_str("r6(cloneable)").unwrap();
    assert_eq!(attrs.class_system, ClassSystem::R6);
    assert!(!attrs.r_data_accessors);
}
// endregion

// region: R6 r_data_accessors wrapper generation test

#[test]
fn r6_wrapper_r_data_accessors() {
    let code: syn::ItemImpl = syn::parse_quote! {
        impl MyType {
            pub fn new() -> Self { Self }
        }
    };

    let mut attrs = default_impl_attrs(ClassSystem::R6);
    attrs.r_data_accessors = true;
    let parsed = ParsedImpl::parse(attrs, code).unwrap();
    let wrapper = generate_r6_r_wrapper(&parsed);

    // Should contain the call to .rdata_active_bindings_MyType
    assert!(
        wrapper.contains(".rdata_active_bindings_MyType(MyType)"),
        "Expected .rdata_active_bindings_MyType(MyType) in:\n{}",
        wrapper
    );
}

#[test]
fn r6_wrapper_no_r_data_accessors() {
    let code: syn::ItemImpl = syn::parse_quote! {
        impl MyType {
            pub fn new() -> Self { Self }
        }
    };

    let attrs = default_impl_attrs(ClassSystem::R6);
    let parsed = ParsedImpl::parse(attrs, code).unwrap();
    let wrapper = generate_r6_r_wrapper(&parsed);

    // Should NOT contain the call to .rdata_active_bindings
    assert!(
        !wrapper.contains(".rdata_active_bindings"),
        "Should not have .rdata_active_bindings in:\n{}",
        wrapper
    );
}
// endregion

// region: S7 r_data_accessors wrapper generation test

#[test]
fn s7_wrapper_r_data_accessors() {
    let code: syn::ItemImpl = syn::parse_quote! {
        impl MyType {
            pub fn new() -> Self { Self }
        }
    };

    let mut attrs = default_impl_attrs(ClassSystem::S7);
    attrs.r_data_accessors = true;
    let parsed = ParsedImpl::parse(attrs, code).unwrap();
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should use c(list(...), .rdata_properties_MyType) pattern
    assert!(
        wrapper.contains("properties = c(list("),
        "Expected 'properties = c(list(' in:\n{}",
        wrapper
    );
    assert!(
        wrapper.contains(".rdata_properties_MyType)"),
        "Expected '.rdata_properties_MyType)' in:\n{}",
        wrapper
    );
}

#[test]
fn s7_wrapper_no_r_data_accessors() {
    let code: syn::ItemImpl = syn::parse_quote! {
        impl MyType {
            pub fn new() -> Self { Self }
        }
    };

    let attrs = default_impl_attrs(ClassSystem::S7);
    let parsed = ParsedImpl::parse(attrs, code).unwrap();
    let wrapper = generate_s7_r_wrapper(&parsed);

    // Should use regular properties = list(...) pattern
    assert!(
        wrapper.contains("properties = list("),
        "Expected 'properties = list(' in:\n{}",
        wrapper
    );
    assert!(
        !wrapper.contains(".rdata_properties"),
        "Should not have .rdata_properties in:\n{}",
        wrapper
    );
}
// endregion

// region: Insta snapshot tests for R wrapper output stability
//
// These tests capture the full generated R wrapper output as snapshots.
// Run `cargo insta review` to review and accept changes after modifying codegen.

#[test]
fn snapshot_env_basic() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            /// Create a new counter
            /// @param value Initial value
            pub fn new(value: i32) -> Self { unimplemented!() }
            /// Get the current value
            pub fn get(&self) -> i32 { unimplemented!() }
            /// Increment by one
            pub fn increment(&mut self) { unimplemented!() }
            /// Add a value and return the result
            pub fn add(&mut self, n: i32) -> i32 { unimplemented!() }
            /// Create from string
            pub fn from_string(s: String) -> Self { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::Env, item_impl);
    insta::assert_snapshot!(generate_env_r_wrapper(&parsed));
}

#[test]
fn snapshot_env_defaults() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Widget {
            pub fn new() -> Self { unimplemented!() }
            #[miniextendr(defaults(step = "1L", verbose = "FALSE"))]
            pub fn update(&mut self, step: i32, verbose: bool) { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::Env, item_impl);
    insta::assert_snapshot!(generate_env_r_wrapper(&parsed));
}

#[test]
fn snapshot_r6_basic() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            /// Create a new counter
            /// @param value Initial value
            pub fn new(value: i32) -> Self { unimplemented!() }
            /// Get the current value
            pub fn get(&self) -> i32 { unimplemented!() }
            /// Increment by one
            pub fn increment(&mut self) { unimplemented!() }
            /// Create from value (static factory)
            pub fn from_value(v: i32) -> Self { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::R6, item_impl);
    insta::assert_snapshot!(generate_r6_r_wrapper(&parsed));
}

#[test]
fn snapshot_r6_with_options() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Child {
            pub fn new() -> Self { unimplemented!() }
            pub fn greet(&self) -> String { unimplemented!() }
        }
    };
    let mut attrs = default_impl_attrs(ClassSystem::R6);
    attrs.r6_inherit = Some("ParentClass".to_string());
    attrs.r6_cloneable = Some(true);
    attrs.r6_lock_class = Some(true);
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    insta::assert_snapshot!(generate_r6_r_wrapper(&parsed));
}

#[test]
fn snapshot_r6_active_bindings() {
    // Pins the combined getter/setter active-binding emission: the setter
    // branch must carry the standalone setter's stopifnot precondition block
    // (renamed to the binding's `value` formal) and both branches must guard
    // the `.Call()` result against transported Rust conditions (audit
    // 2026-07-06 finding 4).
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Temperature {
            /// Create a new temperature.
            /// @param celsius Temperature in Celsius.
            pub fn new(celsius: f64) -> Self { unimplemented!() }
            /// Temperature in Celsius.
            #[miniextendr(r6(active))]
            pub fn celsius(&self) -> f64 { unimplemented!() }
            /// Set the temperature in Celsius.
            #[miniextendr(r6(setter, prop = "celsius"))]
            pub fn set_celsius(&mut self, value: f64) { unimplemented!() }
            /// Temperature in Fahrenheit (read-only).
            #[miniextendr(r6(active))]
            pub fn fahrenheit(&self) -> f64 { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::R6, item_impl);
    insta::assert_snapshot!(generate_r6_r_wrapper(&parsed));
}

#[test]
fn snapshot_r6_private_methods() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Secure {
            pub fn new() -> Self { unimplemented!() }
            pub fn public_api(&self) -> i32 { unimplemented!() }
            fn internal_compute(&self) -> i32 { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::R6, item_impl);
    insta::assert_snapshot!(generate_r6_r_wrapper(&parsed));
}

#[test]
fn snapshot_s3_basic() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
            pub fn zero() -> Self { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S3, item_impl);
    insta::assert_snapshot!(generate_s3_r_wrapper(&parsed));
}

#[test]
fn snapshot_s4_basic() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S4, item_impl);
    insta::assert_snapshot!(generate_s4_r_wrapper(&parsed));
}

#[test]
fn snapshot_s7_basic() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
            pub fn increment(&mut self) { unimplemented!() }
            pub fn from_parts(a: i32, b: i32) -> Self { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, item_impl);
    insta::assert_snapshot!(generate_s7_r_wrapper(&parsed));
}

#[test]
fn snapshot_s7_properties() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Range {
            pub fn new(start: f64, end: f64) -> Self { unimplemented!() }

            #[miniextendr(s7(getter))]
            pub fn length(&self) -> f64 { unimplemented!() }

            #[miniextendr(s7(getter, prop = "midpoint"))]
            pub fn get_midpoint(&self) -> f64 { unimplemented!() }

            #[miniextendr(s7(setter, prop = "midpoint"))]
            pub fn set_midpoint(&mut self, value: f64) { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, item_impl);
    insta::assert_snapshot!(generate_s7_r_wrapper(&parsed));
}

/// Snapshot: S7 class with `r_data_accessors` and NO impl-block properties.
/// Verifies that the sidecar prop docs placeholder is emitted.
#[test]
fn snapshot_s7_sidecar_only_props() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl SidecarOnly {
            pub fn new(value: i32) -> Self { unimplemented!() }
        }
    };
    let mut attrs = default_impl_attrs(ClassSystem::S7);
    attrs.r_data_accessors = true;
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    insta::assert_snapshot!(generate_s7_r_wrapper(&parsed));
}

/// Snapshot: S7 class with `r_data_accessors` AND impl-block properties.
/// Verifies impl-block @prop lines come first, then sidecar placeholder.
#[test]
fn snapshot_s7_sidecar_and_impl_props() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Mixed {
            pub fn new(x: f64) -> Self { unimplemented!() }

            /// The computed length.
            #[miniextendr(s7(getter))]
            pub fn length(&self) -> f64 { unimplemented!() }
        }
    };
    let mut attrs = default_impl_attrs(ClassSystem::S7);
    attrs.r_data_accessors = true;
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    insta::assert_snapshot!(generate_s7_r_wrapper(&parsed));
}

/// Snapshot: S7 class with constructor params that have defaults and varargs.
/// In-scope NIT from #379: constructor-param @param doc coverage for these cases.
#[test]
fn snapshot_s7_prop_tags_with_defaults_and_varargs() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl WithDefaults {
            /// Constructor with defaults.
            /// @param name Character name.
            /// @param ... Additional parameters.
            pub fn new(name: String, scale: f64, mode: Option<i32>) -> Self { unimplemented!() }

            #[miniextendr(s7(getter))]
            pub fn name(&self) -> String { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, item_impl);
    insta::assert_snapshot!(generate_s7_r_wrapper(&parsed));
}

/// Snapshot: S7 class with documented impl-block properties.
/// Verifies that getter doc comments are propagated to @prop lines.
#[test]
fn snapshot_s7_documented_props() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Documented {
            pub fn new() -> Self { unimplemented!() }

            /// The integer count value.
            #[miniextendr(s7(getter))]
            pub fn count(&self) -> i32 { unimplemented!() }

            #[miniextendr(s7(setter))]
            pub fn count(&mut self, value: i32) { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, item_impl);
    insta::assert_snapshot!(generate_s7_r_wrapper(&parsed));
}

/// Snapshot: S7 class where a getter has a multi-paragraph doc comment.
/// Verifies that paragraph 2+ is preserved in the @prop line (fix for #579).
#[test]
fn snapshot_s7_prop_multi_paragraph_doc() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl MultiParaProp {
            pub fn new(start: f64, end: f64) -> Self { unimplemented!() }

            /// Computed length of the range.
            ///
            /// This is a read-only computed property.
            /// It returns end minus start.
            #[miniextendr(s7(getter))]
            pub fn length(&self) -> f64 { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S7, item_impl);
    let output = generate_s7_r_wrapper(&parsed);
    // Multi-paragraph doc: para 1 on @prop line, para 2 as continuation
    assert!(
        output.contains("@prop length Computed length of the range"),
        "first paragraph missing from @prop: {}",
        output
    );
    assert!(
        output.contains("read-only computed property"),
        "second paragraph missing from @prop continuation: {}",
        output
    );
}

#[test]
fn snapshot_vctrs_vctr() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Percent {
            pub fn new(x: f64) -> Vec<f64> { unimplemented!() }
            // Static methods only — &self not allowed on vctrs impls (MXL120)
            pub fn value(amounts: Vec<f64>) -> Vec<f64> { unimplemented!() }
            pub fn scale(amounts: Vec<f64>, factor: f64) -> Vec<f64> { unimplemented!() }
        }
    };
    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Vctr,
        base: Some("double".to_string()),
        inherit_base_type: Some(false),
        ptype: None,
        abbr: Some("pct".to_string()),
    };
    let mut attrs = default_impl_attrs(ClassSystem::Vctrs);
    attrs.vctrs_attrs = vctrs_attrs;
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    insta::assert_snapshot!(generate_vctrs_r_wrapper(&parsed));
}

#[test]
fn snapshot_vctrs_rcrd() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Rational {
            pub fn new(n: i32, d: i32) -> Vec<i32> { unimplemented!() }
            // Static helper — &self not allowed on vctrs impls (MXL120)
            pub fn numerator(n: Vec<i32>) -> Vec<i32> { unimplemented!() }
        }
    };
    let vctrs_attrs = VctrsAttrs {
        kind: VctrsKind::Rcrd,
        base: None,
        inherit_base_type: None,
        ptype: None,
        abbr: Some("rat".to_string()),
    };
    let mut attrs = default_impl_attrs(ClassSystem::Vctrs);
    attrs.vctrs_attrs = vctrs_attrs;
    let parsed = ParsedImpl::parse(attrs, item_impl).unwrap();
    insta::assert_snapshot!(generate_vctrs_r_wrapper(&parsed));
}
// endregion

// region: internal / noexport export-control (audit A10)

fn simple_counter_impl() -> syn::ItemImpl {
    syn::parse_quote! {
        impl Counter {
            pub fn new(value: i32) -> Self { unimplemented!() }
            pub fn get(&self) -> i32 { unimplemented!() }
        }
    }
}

/// Audit A10: a `#[miniextendr(<class>, noexport)]` impl block must produce no
/// Rd contribution at all — its class-doc block carries `@noRd` and no
/// `@title`/`@name`/`@rdname`, and no `@export` anywhere.
#[test]
fn class_generators_noexport_suppresses_all_rd() {
    type Gen = fn(&ParsedImpl) -> String;
    let cases: &[(ClassSystem, Gen)] = &[
        (ClassSystem::Env, generate_env_r_wrapper as Gen),
        (ClassSystem::R6, generate_r6_r_wrapper as Gen),
        (ClassSystem::S3, generate_s3_r_wrapper as Gen),
        (ClassSystem::S4, generate_s4_r_wrapper as Gen),
        (ClassSystem::S7, generate_s7_r_wrapper as Gen),
    ];
    for (class_system, generator) in cases {
        let mut attrs = default_impl_attrs(*class_system);
        attrs.noexport = true;
        let parsed = ParsedImpl::parse(attrs, simple_counter_impl()).unwrap();
        let wrapper = generator(&parsed);

        assert!(
            wrapper.contains("@noRd"),
            "{class_system:?}: noexport must emit @noRd, got:\n{wrapper}"
        );
        assert!(
            !wrapper.contains("@rdname") && !wrapper.contains("@title"),
            "{class_system:?}: noexport must not emit @rdname/@title (no Rd contribution), got:\n{wrapper}"
        );
        assert!(
            !wrapper.contains("@keywords internal"),
            "{class_system:?}: noexport must not add @keywords internal, got:\n{wrapper}"
        );
        // Env keeps `@export` ONLY for the `$.<Class>` / `[[.<Class>` dispatch
        // S3 methods (S3method() entries, required for the class to function);
        // S3-class noexport drops even dispatch registration (see #431).
        if !matches!(class_system, ClassSystem::Env) {
            assert!(
                !wrapper.contains("#' @export"),
                "{class_system:?}: noexport must not emit @export, got:\n{wrapper}"
            );
        }
    }
}

/// Companion: `#[miniextendr(<class>, internal)]` stays documented — the
/// class-doc block keeps `@rdname` and gains `@keywords internal`; no plain
/// class-level `@export`.
#[test]
fn class_generators_internal_stays_documented() {
    type Gen = fn(&ParsedImpl) -> String;
    let cases: &[(ClassSystem, Gen)] = &[
        (ClassSystem::Env, generate_env_r_wrapper as Gen),
        (ClassSystem::R6, generate_r6_r_wrapper as Gen),
        (ClassSystem::S3, generate_s3_r_wrapper as Gen),
        (ClassSystem::S4, generate_s4_r_wrapper as Gen),
        (ClassSystem::S7, generate_s7_r_wrapper as Gen),
    ];
    for (class_system, generator) in cases {
        let mut attrs = default_impl_attrs(*class_system);
        attrs.internal = true;
        let parsed = ParsedImpl::parse(attrs, simple_counter_impl()).unwrap();
        let wrapper = generator(&parsed);

        assert!(
            wrapper.contains("@rdname"),
            "{class_system:?}: internal must keep @rdname (stays documented), got:\n{wrapper}"
        );
        assert!(
            wrapper.contains("@keywords internal"),
            "{class_system:?}: internal must add @keywords internal, got:\n{wrapper}"
        );
        assert!(
            !wrapper.contains("@noRd"),
            "{class_system:?}: internal must NOT emit @noRd, got:\n{wrapper}"
        );
    }
}

/// The S4 generator strips the auto-added class-level `@export` by popping the
/// last ClassDocBuilder line. With `internal` set the builder emits no
/// `@export`, so a blind pop used to delete `@keywords internal` instead —
/// regression guard for the conditional pop.
#[test]
fn s4_internal_keeps_keywords_internal_on_class_block() {
    let mut attrs = default_impl_attrs(ClassSystem::S4);
    attrs.internal = true;
    let parsed = ParsedImpl::parse(attrs, simple_counter_impl()).unwrap();
    let wrapper = generate_s4_r_wrapper(&parsed);

    let set_class_pos = wrapper
        .find("methods::setClass")
        .expect("setClass must be emitted");
    let class_block = &wrapper[..set_class_pos];
    assert!(
        class_block.contains("@keywords internal"),
        "S4 internal class block must keep @keywords internal, got:\n{class_block}"
    );
}

// endregion

// region: consuming `self` receivers (#1432) and fallible in-place steps (#1433)

fn consuming_builder_impl() -> syn::ItemImpl {
    syn::parse_quote! {
        impl Builder {
            pub fn new() -> Self { unimplemented!() }
            /// `self -> Self`: written back into the same handle.
            pub fn with_step(mut self, v: i32) -> Self { unimplemented!() }
            /// `self -> Result<Self, E>`: runs on a clone, overwritten on Ok.
            pub fn try_step(mut self, v: i32) -> Result<Self, String> { unimplemented!() }
            /// `self -> Option<Self>`.
            pub fn maybe_step(mut self, v: i32) -> Option<Self> { unimplemented!() }
            /// Terminal consume: the handle is left consumed.
            pub fn finish(self) -> i32 { unimplemented!() }
            /// Fallible in-place (#1433).
            pub fn checked_bump(&mut self, v: i32) -> Result<&mut Self, String> { unimplemented!() }
            /// Option in-place (#1433).
            pub fn maybe_bump(&mut self, v: i32) -> Option<&mut Self> { unimplemented!() }
        }
    }
}

/// `#[miniextendr(postfix = "_impl")]` on a method appends to the Rust name
/// for the R-facing method (#1451); `r_name` still wins when both paths are
/// compared, and unannotated methods keep the Rust name.
#[test]
fn method_postfix_renames_the_r_method() {
    let parsed = parse_impl(
        ClassSystem::S3,
        syn::parse_quote! {
            impl Widget {
                pub fn new(n: i32) -> Self { unimplemented!() }
                #[miniextendr(postfix = "_impl")]
                pub fn bump(&self, by: i32) -> i32 { unimplemented!() }
                #[miniextendr(r_name = "peek_at")]
                pub fn peek(&self) -> i32 { unimplemented!() }
                pub fn plain(&self) -> i32 { unimplemented!() }
            }
        },
    );
    let name = |n: &str| {
        parsed
            .methods
            .iter()
            .find(|m| m.ident == n)
            .unwrap()
            .r_method_name()
    };
    assert_eq!(name("bump"), "bump_impl");
    assert_eq!(name("peek"), "peek_at");
    assert_eq!(name("plain"), "plain");

    let wrapper = generate_s3_r_wrapper(&parsed);
    assert!(
        wrapper.contains("bump_impl.Widget <- function(x, by, ...)"),
        "{wrapper}"
    );
    assert!(!wrapper.contains("bump.Widget <- function"), "{wrapper}");
    // `r_name` on an S3 instance method also names the generic (previously
    // the Rust ident leaked through `generic_name()`).
    assert!(
        wrapper.contains("peek_at.Widget <- function(x, ...)"),
        "{wrapper}"
    );
    assert!(!wrapper.contains("peek.Widget <- function"), "{wrapper}");
    // The C symbol keeps the Rust name.
    let tokens = c_wrapper_tokens(&parsed, "bump");
    assert!(tokens.contains("Widget__bump"), "{tokens}");
    assert!(!tokens.contains("bump_impl"), "{tokens}");
}

/// `postfix` cannot combine with another naming source on a method.
#[test]
fn method_postfix_validation() {
    let err = ParsedImpl::parse(
        default_impl_attrs(ClassSystem::S3),
        syn::parse_quote! {
            impl Widget {
                #[miniextendr(postfix = "_impl", r_name = "bump2")]
                pub fn bump(&self, by: i32) -> i32 { unimplemented!() }
            }
        },
    )
    .expect_err("postfix + r_name must fail");
    assert!(
        err.to_string().contains("both set the R method name"),
        "{err}"
    );

    let err = ParsedImpl::parse(
        default_impl_attrs(ClassSystem::S3),
        syn::parse_quote! {
            impl Widget {
                #[miniextendr(s3(generic = "format"), postfix = "_impl")]
                pub fn bump(&self, by: i32) -> i32 { unimplemented!() }
            }
        },
    )
    .expect_err("postfix + generic must fail");
    assert!(err.to_string().contains("`postfix` and `generic`"), "{err}");

    let err = ParsedImpl::parse(
        default_impl_attrs(ClassSystem::S3),
        syn::parse_quote! {
            impl Widget {
                #[miniextendr(postfix = "")]
                pub fn bump(&self, by: i32) -> i32 { unimplemented!() }
            }
        },
    )
    .expect_err("empty postfix must fail");
    assert!(err.to_string().contains("must not be empty"), "{err}");
}

fn c_wrapper_tokens(parsed: &ParsedImpl, name: &str) -> String {
    let method = parsed.methods.iter().find(|m| m.ident == name).unwrap();
    let r_wrappers_const = syn::parse_quote! { R_WRAPPERS_TEST };
    crate::miniextendr_impl::generate_method_c_wrapper(parsed, method, &r_wrappers_const)
        .to_string()
}

/// Every consuming receiver is an instance method (has `self_sexp`), none is
/// mistaken for a finalizer or a constructor, and the parse succeeds without
/// any marker attribute.
#[test]
fn consuming_self_receivers_parse_as_instance_methods() {
    let parsed = parse_impl(ClassSystem::S3, consuming_builder_impl());
    for name in ["with_step", "try_step", "maybe_step", "finish"] {
        let m = parsed.methods.iter().find(|m| m.ident == name).unwrap();
        assert_eq!(m.env, ReceiverKind::Value, "{name}");
        assert!(
            m.env.is_instance(),
            "{name}: consuming receiver is an instance method"
        );
        assert!(
            !m.is_finalizer(),
            "{name}: finalizer is never inferred from the receiver"
        );
        assert!(!m.is_constructor(), "{name}");
    }
    assert!(parsed.finalizer().is_none());
    let names: Vec<_> = parsed
        .instance_methods()
        .map(|m| m.ident.to_string())
        .collect();
    for name in [
        "with_step",
        "try_step",
        "maybe_step",
        "finish",
        "checked_bump",
        "maybe_bump",
    ] {
        assert!(
            names.contains(&name.to_string()),
            "{name} missing from instance methods: {names:?}"
        );
    }
}

/// `self -> Self` moves the value out, writes the result back, and returns the
/// same handle; the R strategy is the chainable one (no re-wrap).
#[test]
fn consuming_self_returning_self_writes_back_same_handle() {
    let parsed = parse_impl(ClassSystem::S3, consuming_builder_impl());
    let tokens = c_wrapper_tokens(&parsed, "with_step");
    assert!(tokens.contains("take_for_consuming"), "{tokens}");
    assert!(tokens.contains("restore_after_consuming"), "{tokens}");
    assert!(
        !tokens.contains("ExternalPtr :: new"),
        "must not mint a new handle: {tokens}"
    );
    let m = parsed
        .methods
        .iter()
        .find(|m| m.ident == "with_step")
        .unwrap();
    assert_eq!(
        crate::ReturnStrategy::for_method(m),
        crate::ReturnStrategy::ChainableMutation
    );
    let wrapper = generate_s3_r_wrapper(&parsed);
    assert!(
        wrapper.contains("with_step.Builder <- function(x, v, ...)"),
        "{wrapper}"
    );
}

/// `self -> Result<Self, E>` / `Option<Self>` clone the stored value, overwrite
/// on success, and raise on failure through the Result / Option error paths.
#[test]
fn consuming_self_fallible_clones_and_overwrites_on_success() {
    let parsed = parse_impl(ClassSystem::R6, consuming_builder_impl());
    let tokens = c_wrapper_tokens(&parsed, "try_step");
    assert!(tokens.contains("clone_for_consuming"), "{tokens}");
    assert!(tokens.contains("RESULT_ERR"), "Err must raise: {tokens}");
    assert!(
        !tokens.contains("take_for_consuming"),
        "fallible step must not empty the slot: {tokens}"
    );
    let tokens = c_wrapper_tokens(&parsed, "maybe_step");
    assert!(tokens.contains("clone_for_consuming"), "{tokens}");
    assert!(tokens.contains("NONE_ERR"), "None must raise: {tokens}");
    for name in ["try_step", "maybe_step"] {
        let m = parsed.methods.iter().find(|m| m.ident == name).unwrap();
        assert_eq!(
            crate::ReturnStrategy::for_method(m),
            crate::ReturnStrategy::ChainableMutation,
            "{name}: consuming builder returns the receiver"
        );
    }
    let wrapper = generate_r6_r_wrapper(&parsed);
    assert!(wrapper.contains("invisible(self)"), "{wrapper}");
}

/// A terminal `self -> T` moves the value out and converts the result; the
/// slot is left consumed (no write-back).
#[test]
fn consuming_self_terminal_takes_value_without_write_back() {
    let parsed = parse_impl(ClassSystem::Env, consuming_builder_impl());
    let tokens = c_wrapper_tokens(&parsed, "finish");
    assert!(tokens.contains("take_for_consuming"), "{tokens}");
    assert!(!tokens.contains("restore_after_consuming"), "{tokens}");
    let m = parsed.methods.iter().find(|m| m.ident == "finish").unwrap();
    assert_eq!(
        crate::ReturnStrategy::for_method(m),
        crate::ReturnStrategy::Direct
    );
}

/// `&mut self -> Result<&mut Self, E>` / `Option<&mut Self>` (#1433) use the
/// self-handle strategy instead of falling through to `IntoR` (E0277).
#[test]
fn fallible_self_ref_builders_use_self_handle() {
    let parsed = parse_impl(ClassSystem::S3, consuming_builder_impl());
    let checked = parsed
        .methods
        .iter()
        .find(|m| m.ident == "checked_bump")
        .unwrap();
    assert!(checked.returns_result_self_ref());
    assert!(!checked.returns_self_ref());
    let maybe = parsed
        .methods
        .iter()
        .find(|m| m.ident == "maybe_bump")
        .unwrap();
    assert!(maybe.returns_option_self_ref());
    for name in ["checked_bump", "maybe_bump"] {
        let tokens = c_wrapper_tokens(&parsed, name);
        assert!(tokens.contains("self_sexp"), "{name}: {tokens}");
        assert!(
            !tokens.contains("IntoR :: into_sexp (__result)"),
            "{name} must not route the borrow through IntoR: {tokens}"
        );
        let m = parsed.methods.iter().find(|m| m.ident == name).unwrap();
        assert_eq!(
            crate::ReturnStrategy::for_method(m),
            crate::ReturnStrategy::ChainableMutation,
            "{name}"
        );
    }
    let tokens = c_wrapper_tokens(&parsed, "checked_bump");
    assert!(tokens.contains("RESULT_ERR"), "{tokens}");
    let tokens = c_wrapper_tokens(&parsed, "maybe_bump");
    assert!(tokens.contains("NONE_ERR"), "{tokens}");
}

/// The former escape hatch (`constructor` on a `self` method) is a clear error
/// now that consuming steps need no marker.
#[test]
fn constructor_marker_on_consuming_self_is_rejected() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Builder {
            #[miniextendr(s3(constructor))]
            pub fn with_step(mut self, v: i32) -> Result<Self, String> { unimplemented!() }
        }
    };
    let err = ParsedImpl::parse(default_impl_attrs(ClassSystem::S3), item_impl)
        .expect_err("constructor on self receiver must be rejected")
        .to_string();
    assert!(
        err.contains("takes `self` and is marked `constructor`"),
        "got: {err}"
    );
    assert!(err.contains("writes its result back"), "got: {err}");
}

/// Smart-pointer receivers cannot be handed over by the wrapper.
#[test]
fn smart_pointer_self_receiver_is_rejected() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Builder {
            pub fn consume(self: Box<Self>) -> i32 { unimplemented!() }
        }
    };
    let err = ParsedImpl::parse(default_impl_attrs(ClassSystem::S3), item_impl)
        .expect_err("Box<Self> receiver must be rejected")
        .to_string();
    assert!(err.contains("unsupported receiver type"), "got: {err}");
}

/// `self: Self` is the same as bare `self`.
#[test]
fn typed_self_value_receiver_is_consuming() {
    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl Builder {
            pub fn with_step(self: Self, v: i32) -> Self { unimplemented!() }
        }
    };
    let parsed = parse_impl(ClassSystem::S3, item_impl);
    let m = parsed
        .methods
        .iter()
        .find(|m| m.ident == "with_step")
        .unwrap();
    assert_eq!(m.env, ReceiverKind::Value);
    assert!(c_wrapper_tokens(&parsed, "with_step").contains("restore_after_consuming"));
}

// endregion
