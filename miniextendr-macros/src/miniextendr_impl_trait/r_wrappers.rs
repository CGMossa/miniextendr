//! R wrapper generation for trait methods across all class systems.
//!
//! Each class system (Env, S3, S4, S7, R6, Vctrs) has its own generator that
//! produces R code strings for instance methods, static methods, and associated
//! constants. The top-level [`generate_trait_r_wrapper`] dispatches to the
//! appropriate generator and applies post-processing for export/documentation control.

use super::method_context::{TraitMethodContext, trait_namespace_env_var, trait_namespace_target};
use super::{TraitConst, TraitMethod};
use crate::miniextendr_impl::ClassSystem;
use crate::r_class_formatter::emit_s3_generic_guard;

/// Options controlling export visibility and documentation for trait R wrapper generation.
pub(super) struct TraitWrapperOpts {
    /// Which R class system to generate wrappers for (env, r6, s3, s4, s7, vctrs).
    pub(super) class_system: ClassSystem,
    /// Whether the impl block has `@noRd`, suppressing roxygen documentation output.
    /// For S3/vctrs, method registration tags are preserved even when this is true.
    pub(super) class_has_no_rd: bool,
    /// Whether `#[miniextendr(internal)]` is set, adding `@keywords internal` and
    /// suppressing `@export`/`@exportMethod`.
    pub(super) internal: bool,
    /// Whether `#[miniextendr(noexport)]` is set, suppressing `@export`/`@exportMethod`
    /// without adding `@keywords internal`.
    pub(super) noexport: bool,
}

/// Generate R wrapper code for trait methods and consts, dispatching by class system.
///
/// Calls the appropriate class-system-specific generator (env, s3, s4, s7, r6),
/// then applies post-processing for `@noRd`, `internal`, and `noexport` options:
///
/// - `class_has_no_rd`: Strips roxygen blocks (for S3/vctrs, keeps `@method`/`@export` tags)
/// - `internal`: Replaces `@export`/`@exportMethod` with `@keywords internal`
/// - `noexport`: Removes `@export`/`@exportMethod` entirely
///
/// Returns the complete R wrapper code as a string ready for embedding in a `const`.
pub(super) fn generate_trait_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
    opts: TraitWrapperOpts,
) -> syn::Result<String> {
    let TraitWrapperOpts {
        class_system,
        class_has_no_rd,
        internal,
        noexport,
    } = opts;
    let result = match class_system {
        ClassSystem::Env => generate_trait_env_r_wrapper(type_ident, trait_name, methods, consts)?,
        ClassSystem::S3 => generate_trait_s3_r_wrapper(type_ident, trait_name, methods, consts),
        ClassSystem::S4 => generate_trait_s4_r_wrapper(type_ident, trait_name, methods, consts),
        ClassSystem::S7 => generate_trait_s7_r_wrapper(type_ident, trait_name, methods, consts),
        ClassSystem::R6 => generate_trait_r6_r_wrapper(type_ident, trait_name, methods, consts),
        // vctrs uses S3 under the hood, so use the S3 trait wrapper
        ClassSystem::Vctrs => generate_trait_s3_r_wrapper(type_ident, trait_name, methods, consts),
    };

    // When the impl block has @noRd, suppress documentation generation. A plain
    // `noexport` (without `internal`) is folded into the same gate — it must
    // produce no Rd contribution at all (no alias, no usage entry, nothing on a
    // shared page), same as `@noRd`. `internal` wins if both flags are set on
    // the same impl block (mirrors the standalone-fn precedent, where `internal`
    // + `noexport` together is a compile error). See #431 for the inherent-impl
    // S3 generator's analogous `should_register_s3method = !noexport` rule.
    let suppress_all_rd = class_has_no_rd || (noexport && !internal);
    if suppress_all_rd {
        if matches!(class_system, ClassSystem::S3 | ClassSystem::Vctrs) {
            // A user-written `@noRd` still preserves S3 dispatch registration
            // (`@method`/`@export`) so `S3method()` lands in NAMESPACE — the
            // class stays undocumented but dispatchable. A `noexport`-driven
            // suppression (no explicit `@noRd`) additionally drops `@export`:
            // `noexport` means zero observable trace, not "documented nowhere
            // but still dispatchable".
            let keep_export = class_has_no_rd;
            let mut filtered = Vec::new();
            let mut roxygen_block: Vec<&str> = Vec::new();

            let flush_block = |block: &mut Vec<&str>, out: &mut Vec<String>| {
                if block.iter().any(|line| line.contains("@method ")) {
                    out.push("#' @noRd".to_string());
                    for &line in block.iter() {
                        if line.contains("@method ")
                            || line.contains("@param ")
                            || (keep_export && line.contains("@export"))
                        {
                            out.push(line.to_string());
                        }
                    }
                }
                block.clear();
            };

            for line in result.lines() {
                if line.starts_with("#'") {
                    roxygen_block.push(line);
                    continue;
                }

                if !roxygen_block.is_empty() {
                    flush_block(&mut roxygen_block, &mut filtered);
                }
                filtered.push(line.to_string());
            }

            if !roxygen_block.is_empty() {
                flush_block(&mut roxygen_block, &mut filtered);
            }

            Ok(filtered.join("\n"))
        } else {
            Ok(result
                .lines()
                .filter(|line| !line.starts_with("#'"))
                .collect::<Vec<_>>()
                .join("\n"))
        }
    } else if internal {
        // internal → documented, but @export/@exportMethod becomes @keywords internal
        let has_export = result.lines().any(|line| line.contains("@export"));
        let mut processed: Vec<String> = result
            .lines()
            .flat_map(|line| {
                if line.contains("@export") {
                    vec!["#' @keywords internal".to_string()]
                } else {
                    vec![line.to_string()]
                }
            })
            .collect();
        // For class systems without @export (e.g., Env), insert @keywords internal
        // before the first roxygen tag if no @export line was found to replace.
        if !has_export && let Some(pos) = processed.iter().position(|l| l.starts_with("#'")) {
            processed.insert(pos, "#' @keywords internal".to_string());
        }
        Ok(processed.join("\n"))
    } else {
        Ok(result)
    }
}

/// `@rdname` for a trait method's own wrapper block: the method-level
/// override when the doc comment carries one, else the type's shared page.
/// Generics and consts always use the type page (#1438).
fn method_rdname<'a>(method: &'a TraitMethod, type_str: &'a str) -> &'a str {
    method.rdname.as_deref().unwrap_or(type_str)
}

/// Attach a structural `@title` to a prose-less roxygen block that has been
/// split onto its own page by a method-level `@rdname`; without one roxygen2
/// skips the page ("no name and/or title"). No-op on the shared type page,
/// which gets its title from the class block.
fn title_if_split(
    builder: crate::r_wrapper_builder::RoxygenBuilder,
    method: &TraitMethod,
    title: &str,
) -> crate::r_wrapper_builder::RoxygenBuilder {
    if method.rdname.is_some() {
        builder.title(title)
    } else {
        builder
    }
}

/// Generate Env-style R wrapper code for trait methods.
///
/// Env-class trait methods use a namespace hierarchy: `Type$Trait$method(x, ...)`.
/// Instance methods take `x` as the first parameter (the self object) and are
/// stamped with `.__mx_instance__` attribute for `$` dispatch detection.
/// Void instance methods return `invisible(x)` for pipe-friendly chaining.
///
/// Static methods and constants also live under `Type$Trait$name`.
///
/// Returns an error if an instance method has a parameter named `x` (collides
/// with the self parameter in env-class dispatch).
fn generate_trait_env_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> syn::Result<String> {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();

    // Header comment
    lines.push(format!(
        "# Trait methods and consts for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push(String::new());

    // Create trait namespace environment
    lines.push(format!(
        "{}${} <- new.env(parent = emptyenv())",
        type_ident, trait_name
    ));
    lines.push(String::new());

    for method in methods {
        let r_name = method.r_method_name();
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);

        // Trait-namespace assignment target (`Type$Trait$method`), owned by
        // `trait_namespace_target` — see #1141.
        let target = ctx.namespace_target(ClassSystem::Env);

        // Build roxygen tags
        let roxygen = title_if_split(
            RoxygenBuilder::new()
                .name(target.clone())
                .rdname(method_rdname(method, &type_str)),
            method,
            &target,
        )
        .build();
        lines.extend(roxygen);

        // Check for 'x' parameter collision in instance methods
        if method.has_self {
            for input in &method.sig.inputs {
                if let syn::FnArg::Typed(pt) = input
                    && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
                    && pat_ident.ident == "x"
                {
                    return Err(syn::Error::new_spanned(
                        &pat_ident.ident,
                        "trait instance method parameter cannot be named `x` \
                         (collides with self parameter in env-class dispatch)",
                    ));
                }
            }
        }

        // Build .Call() invocation — C name uses Rust ident, R name uses r_name
        let (full_params, call) = if method.has_self {
            let fp = if ctx.params.is_empty() {
                "x".to_string()
            } else {
                format!("x, {}", ctx.params)
            };
            (fp, ctx.instance_call("x"))
        } else {
            (ctx.params.clone(), ctx.static_call())
        };

        // Generate method wrapper (R-facing name)
        lines.push(format!("{target} <- function({full_params}) {{"));
        ctx.emit_method_prelude(&mut lines, "  ", &r_name);
        lines.extend(ctx.method_body_lines(&call, ClassSystem::Env));
        if method.has_self && method.returns_unit() {
            lines.push("  invisible(x)".to_string());
        }
        lines.push("}".to_string());

        // Stamp instance methods with attribute for $ dispatch detection
        if method.has_self {
            lines.push(format!("attr({target}, \".__mx_instance__\") <- TRUE"));
        }

        lines.push(String::new());
    }

    // Generate const wrappers
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let const_str = const_name.to_string();
        let target = trait_namespace_target(ClassSystem::Env, type_ident, trait_name, &const_str);

        // Build roxygen tags
        let roxygen = RoxygenBuilder::new()
            .name(target.clone())
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        // Build .Call() invocation
        let c_ident = trait_const.c_wrapper_ident_string(type_ident, trait_name);
        let call = DotCallBuilder::new(&c_ident).build();

        // Generate const wrapper
        lines.push(format!("{target} <- function() {{"));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    Ok(lines.join("\n"))
}

/// Generate S3-style R wrapper code (generic + method.Type).
///
/// For `impl Counter for SimpleCounter`, generates:
/// - S3 generic `value(x, ...)` (if not already defined)
/// - S3 method `value.SimpleCounter <- function(x, ...) { .Call(...) }`
/// - S7 method registration if the generic is an S7 generic
///
/// Static methods and constants use `Type$Trait$name` namespace (env-style).
/// Void instance methods return `invisible(x)` for pipe-friendly chaining.
///
/// Also used for `ClassSystem::Vctrs` since vctrs uses S3 under the hood.
fn generate_trait_s3_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> String {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();

    // Header comment
    lines.push(format!(
        "# S3 trait methods for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr(s3)] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push(String::new());

    // Separate instance methods (S3 dispatch) from static methods (namespace access)
    let instance_methods: Vec<_> = methods.iter().filter(|m| m.has_self).collect();
    let static_methods: Vec<_> = methods.iter().filter(|m| !m.has_self).collect();

    // Generate S3 generics + methods for instance methods
    for method in &instance_methods {
        let generic_name = method.r_method_name();
        let s3_method_name = format!("{}.{}", generic_name, type_str);
        let generic_symbol = crate::naming::r_symbol(&generic_name);
        let method_symbol = crate::naming::r_symbol(&s3_method_name);
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);

        // S3 generic roxygen (only create if doesn't exist). The type-qualified
        // @name avoids duplicate aliases across types, but it is also the S3
        // method's own name, so the generic block must follow a method-level
        // `@rdname` onto the split page or R CMD check reports the alias as
        // duplicated across two pages. The `@title` is inert on the type page
        // (the type block's title comes first) and would beat the method's
        // structural title on a split page, so it is only emitted when the
        // block stays on the type page.
        let generic_roxygen = RoxygenBuilder::new();
        let generic_roxygen = if method.rdname.is_none() {
            generic_roxygen.title(format!("S3 generic for `{}`", generic_name))
        } else {
            generic_roxygen
        };
        let generic_roxygen = generic_roxygen
            .custom(format!("S3 generic for `{}`", generic_name))
            .name(format!("{}.{}", generic_name, type_str))
            .rdname(method_rdname(method, &type_str))
            .custom("@param x An object")
            .custom("@param ... Additional arguments passed to methods")
            .source(format!(
                "Generated by miniextendr from `impl {} for {}`",
                trait_name, type_ident
            ))
            .export()
            .build();
        lines.extend(generic_roxygen);

        // S3 generic definition
        lines.push(emit_s3_generic_guard(generic_name.as_str()));
        lines.push(String::new());

        // S3 method roxygen (include @param tags from method doc comments)
        let mut method_roxygen = title_if_split(
            RoxygenBuilder::new().rdname(method_rdname(method, &type_str)),
            method,
            &s3_method_name,
        )
        .export()
        .method(&generic_name, &type_str);
        for tag in &method.param_tags {
            method_roxygen = method_roxygen.custom(tag.clone());
        }
        lines.extend(method_roxygen.build());

        // S3 method: generic.class
        let full_params = if ctx.params.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", ctx.params)
        };

        // Build .Call() invocation
        let call = ctx.instance_call("x");

        // Always define the S3 method (roxygen expects it for NAMESPACE export)
        lines.push(format!("{} <- function({}) {{", method_symbol, full_params));
        ctx.emit_method_prelude(&mut lines, "  ", &generic_name);
        lines.extend(ctx.method_body_lines(&call, ClassSystem::S3));
        // Void instance methods return invisible(x) for pipe-friendly chaining
        if method.returns_unit() {
            lines.push("  invisible(x)".to_string());
        }
        lines.push("}".to_string());

        // Additionally register as S7 method if the generic is S7
        // This ensures S7 dispatch works when the generic was defined by an S7 class
        lines.push(format!(
            "if (inherits(get0(\"{generic_name}\", mode = \"function\"), \"S7_generic\")) {{"
        ));
        lines.push(format!(
            "  S7::method({generic_symbol}, S7::new_S3_class(\"{type_str}\")) <- {method_symbol}"
        ));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Create trait namespace for static methods and consts BEFORE assigning to it
    if !static_methods.is_empty() || !consts.is_empty() {
        lines.push(format!(
            "{}${} <- new.env(parent = emptyenv())",
            type_ident, trait_name
        ));
        lines.push(String::new());
    }

    // Generate static methods in Type$Trait$ namespace
    for method in &static_methods {
        let r_name = method.r_method_name();
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);
        let target = ctx.namespace_target(ClassSystem::S3);

        // Static method roxygen
        lines.push(format!(
            "#' Static trait method {}::{}()",
            trait_name, r_name
        ));
        let roxygen = RoxygenBuilder::new()
            .name(target.clone())
            .rdname(method_rdname(method, &type_str))
            .build();
        lines.extend(roxygen);

        let call = ctx.static_call();

        lines.push(format!("{target} <- function({}) {{", ctx.params));
        ctx.emit_method_prelude(&mut lines, "  ", &r_name);
        lines.extend(ctx.method_body_lines(&call, ClassSystem::S3));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Generate const wrappers in Type$Trait$ namespace
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let const_str = const_name.to_string();
        let target = trait_namespace_target(ClassSystem::S3, type_ident, trait_name, &const_str);

        let roxygen = RoxygenBuilder::new()
            .name(target.clone())
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        let c_ident = trait_const.c_wrapper_ident_string(type_ident, trait_name);
        let call = DotCallBuilder::new(&c_ident).build();

        lines.push(format!("{target} <- function() {{"));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate S4-style R wrapper code.
///
/// For `impl Counter for SimpleCounter`, generates:
/// - `setOldClass("SimpleCounter")` to register the S3 class for S4 dispatch
/// - S4 generic `s4_trait_Counter_value(x, ...)` via `setGeneric()`
/// - S4 method via `setMethod("s4_trait_Counter_value", "SimpleCounter", ...)`
///
/// Generic names are prefixed with `s4_trait_{Trait}_` to avoid collisions
/// with user-defined S4 generics. Static methods and constants are generated
/// as standalone exported functions: `{Type}_{Trait}_{method}()`.
fn generate_trait_s4_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> String {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();

    // Header comment
    lines.push(format!(
        "# S4 trait methods for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr(s4)] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push(String::new());

    // NOTE: We do NOT call setOldClass here. The inherent impl's class registration
    // (setClass for S4, or setOldClass for S3/env) takes care of that. Calling
    // setOldClass here would clobber a proper S4 setClass with slots.
    lines.push("#' @importFrom methods setGeneric setMethod".to_string());
    lines.push(String::new());

    // Separate instance methods from static methods
    let instance_methods: Vec<_> = methods.iter().filter(|m| m.has_self).collect();
    let static_methods: Vec<_> = methods.iter().filter(|m| !m.has_self).collect();

    // Generate S4 generics + methods for instance methods
    for method in &instance_methods {
        let method_name = &method.ident;
        let generic_name = format!("s4_trait_{}_{}", trait_name, method.r_method_name());
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);

        // Build full parameter list (x first, then others, then ...)
        let full_params = if ctx.params.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", ctx.params)
        };

        // S4 generic roxygen (include @param tags from method doc comments)
        // S4 generic names are already type-qualified (s4_trait_TypeName_method)
        // so @name won't create duplicate aliases across types.
        let mut generic_roxygen = RoxygenBuilder::new()
            .custom(format!(
                "S4 generic for trait method `{}::{}`",
                trait_name, method_name
            ))
            .name(&generic_name)
            .rdname(&type_str)
            .source(format!(
                "Generated by miniextendr from `impl {} for {}`",
                trait_name, type_ident
            ))
            .custom(format!("@param x A `{}` object", type_str))
            .custom("@param ... Additional arguments passed to methods");
        for tag in &method.param_tags {
            generic_roxygen = generic_roxygen.custom(tag.clone());
        }
        lines.extend(generic_roxygen.export().build());

        // Define generic only if it doesn't already exist in THIS namespace
        // (avoid clearing methods). Scoped to topenv(environment()) so an
        // attached installed copy of the package can't satisfy the check and
        // starve the setMethod below during load_all() (#1158).
        lines.push(format!(
            "if (!exists(\"{generic_name}\", where = topenv(environment()), inherits = FALSE)) methods::setGeneric(\"{generic_name}\", function(x, ...) standardGeneric(\"{generic_name}\"))"
        ));
        lines.push(String::new());

        // S4 method roxygen + definition (include @param tags from method doc comments)
        lines.push(format!("#' @rdname {}", method_rdname(method, &type_str)));
        if method.rdname.is_some() {
            // Split page (#1438): the S4 method object supplies the name; a
            // structural title is still needed or roxygen2 skips the page.
            lines.push(format!("#' @title {}", generic_name));
        }
        for tag in &method.param_tags {
            lines.push(format!("#' {}", tag));
        }
        lines.push(format!("#' @exportMethod {}", generic_name));

        lines.push(format!(
            "methods::setMethod(\"{}\", \"{}\", function({}) {{",
            generic_name, type_str, full_params
        ));
        // S4 objects store the ExternalPtr in x@ptr — extract it for .Call()
        lines.push("  .ptr <- x@ptr".to_string());
        let s4_call = ctx.instance_call(".ptr");
        ctx.emit_method_prelude(&mut lines, "  ", &method.r_method_name());
        lines.extend(ctx.method_body_lines(&s4_call, ClassSystem::S4));
        // Void instance methods return invisible(x) for pipe-friendly chaining
        if method.returns_unit() {
            lines.push("  invisible(x)".to_string());
        }
        lines.push("})".to_string());
        lines.push(String::new());
    }

    // Generate static methods as standalone functions. S4 objects intercept
    // `$<-`, so these use the flat, class-qualified `Type_Trait_method` name
    // (owned by `trait_namespace_target`) rather than `Type$Trait$method`.
    for method in &static_methods {
        let r_name = method.r_method_name();
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);
        let fn_name = ctx.namespace_target(ClassSystem::S4);

        // Static method roxygen
        lines.push(format!(
            "#' Static trait method {}::{}() for {}",
            trait_name, r_name, type_str
        ));
        let roxygen = RoxygenBuilder::new()
            .name(&fn_name)
            .rdname(method_rdname(method, &type_str))
            .export()
            .build();
        lines.extend(roxygen);

        let call = ctx.static_call();

        lines.push(format!("{} <- function({}) {{", fn_name, ctx.params));
        ctx.emit_method_prelude(&mut lines, "  ", &r_name);
        lines.extend(ctx.method_body_lines(&call, ClassSystem::S4));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Generate const wrappers as standalone functions (flat name, as above)
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let const_str = const_name.to_string();
        let fn_name = trait_namespace_target(ClassSystem::S4, type_ident, trait_name, &const_str);

        let roxygen = RoxygenBuilder::new()
            .name(&fn_name)
            .rdname(&type_str)
            .export()
            .build();
        lines.extend(roxygen);

        let c_ident = trait_const.c_wrapper_ident_string(type_ident, trait_name);
        let call = DotCallBuilder::new(&c_ident).build();

        lines.push(format!("{} <- function() {{", fn_name));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate S7-style R wrapper code.
///
/// For `impl Counter for SimpleCounter`, generates:
/// - S7 S3-class wrapper: `.s7_class_SimpleCounter <- S7::new_S3_class("SimpleCounter")`
/// - S7 generic: `s7_trait_Counter_value <- S7::new_generic(...)` (if not exists)
/// - S7 method registration: `S7::method(s7_trait_Counter_value, .s7_class_SimpleCounter) <- ...`
///
/// Generic names are prefixed with `s7_trait_{Trait}_` to avoid collisions.
/// Static methods and constants use `Type$Trait$name` namespace (env-style).
fn generate_trait_s7_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> String {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();
    let trait_str = trait_name.to_string();
    let s7_class_var = format!(".s7_class_{}", type_str);

    // Header comment
    lines.push(format!(
        "# S7 trait methods for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr(s7)] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push(String::new());

    // Use the S7 class object directly for method dispatch.
    // new_S3_class("Foo") creates a descriptor for "Foo" but S7 new_class
    // creates instances with the namespaced class "pkg::Foo", so new_S3_class
    // wouldn't match. Using the class object directly works correctly.
    lines.push("#' @importFrom S7 new_generic method S7_dispatch".to_string());
    lines.push(format!("{} <- {}", s7_class_var, type_str));
    lines.push(String::new());

    // Separate instance methods from static methods
    let instance_methods: Vec<_> = methods.iter().filter(|m| m.has_self).collect();
    let static_methods: Vec<_> = methods.iter().filter(|m| !m.has_self).collect();

    // Generate S7 generics + methods for instance methods
    for method in &instance_methods {
        let method_name = &method.ident;
        let generic_name = format!("s7_trait_{}_{}", trait_name, method.r_method_name());
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);

        // Build full parameter list (x first, then others, then ...)
        let full_params = if ctx.params.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", ctx.params)
        };

        // S7 generic roxygen
        // Note: Don't include method-specific @param tags here since S7 methods
        // are assignments and won't appear in \usage, which would cause warnings
        // S7 generic names are already type-qualified so @name won't duplicate.
        let generic_roxygen = RoxygenBuilder::new()
            .custom(format!(
                "S7 generic for trait method `{}::{}`",
                trait_name, method_name
            ))
            .name(&generic_name)
            .rdname(&type_str)
            .source(format!(
                "Generated by miniextendr from `impl {} for {}`",
                trait_name, type_ident
            ))
            .export()
            .build();
        lines.extend(generic_roxygen);

        // S7 generic definition
        lines.push(format!(
            "if (!exists(\"{generic_name}\", mode = \"function\")) {{"
        ));
        lines.push(format!(
            "  {generic_name} <- S7::new_generic(\"{generic_name}\", \"x\", function(x, ...) S7::S7_dispatch())"
        ));
        lines.push("}".to_string());
        lines.push(String::new());

        // S7 method definition
        lines.push(format!(
            "S7::method({}, {}) <- function({}) {{",
            generic_name, s7_class_var, full_params
        ));
        // S7 objects store the ExternalPtr in x@.ptr — extract it for .Call()
        lines.push("  .ptr <- x@.ptr".to_string());
        let s7_call = ctx.instance_call(".ptr");
        ctx.emit_method_prelude(&mut lines, "  ", &method.r_method_name());
        lines.extend(ctx.method_body_lines(&s7_call, ClassSystem::S7));
        // Void instance methods return invisible(x) for pipe-friendly chaining
        if method.returns_unit() {
            lines.push("  invisible(x)".to_string());
        }
        lines.push("}".to_string());
        lines.push(String::new());

        // Per-class fast-path dispatch shortcut (#987).
        //
        // Mirror the inherent-impl S7 shortcut (#982, see
        // `miniextendr_impl::s7_class`): alongside the trait generic, emit a
        // plain `<ClassName>_<method>(self, ...)` function that calls `.Call`
        // directly, bypassing `S7::S7_dispatch()`. The receiver is named `self`
        // here (the generic names it `x`) and wired through `self@.ptr`.
        // `s7(no_shortcut)` opts a method out.
        if !method.no_shortcut {
            let shortcut_name = format!("{}_{}", type_str, method.r_method_name());
            let shortcut_formals = if ctx.params.is_empty() {
                "self, ...".to_string()
            } else {
                format!("self, {}, ...", ctx.params)
            };
            let shortcut_call = ctx.instance_call("self@.ptr");

            // Roxygen: shared advisory prose + scaffolding. The shared @rdname
            // page (type_str) already carries the method's prose via the generic
            // block above, so only document `self` + each formal here to keep the
            // shortcut's \usage fully covered (no "undocumented argument" warning).
            lines.extend(crate::miniextendr_impl::s7_class::shortcut_advisory_lines(
                &method.r_method_name(),
                &type_str,
            ));
            lines.push(format!("#' @param self A `{}` object.", type_str));
            for tag in &method.param_tags {
                lines.push(format!("#' {}", tag));
            }
            // Auto-document any formal lacking an explicit @param tag. `...` is
            // included so roxygen2 covers it (otherwise R CMD check warns about
            // an undocumented argument). Split on top-level commas only — a
            // naive `split(", ")` breaks a `mode = c("fast", "slow")` default
            // into a bogus `"slow")` formal (undocumented-argument warning).
            for formal in crate::roxygen::split_r_formals(&shortcut_formals) {
                let pname = crate::roxygen::formal_name(formal);
                if pname == "self" {
                    continue;
                }
                let documented = crate::roxygen::param_documented(&method.param_tags, pname);
                if documented {
                    continue;
                }
                if pname == "..." {
                    lines.push(
                        "#' @param ... Additional arguments; ignored by the fast-path shortcut."
                            .to_string(),
                    );
                } else {
                    lines.push(format!("#' @param {} (undocumented)", pname));
                }
            }
            lines.push(format!("#' @name {}", shortcut_name));
            lines.push(format!("#' @rdname {}", method_rdname(method, &type_str)));
            lines.push(format!(
                "#' @source Generated by miniextendr from `impl {} for {}` (`{}` shortcut)",
                trait_name, type_ident, method_name
            ));
            lines.push("#' @export".to_string());

            lines.push(format!(
                "{} <- function({}) {{",
                shortcut_name, shortcut_formals
            ));
            ctx.emit_method_prelude(&mut lines, "  ", &method.r_method_name());
            lines.extend(ctx.method_body_lines(&shortcut_call, ClassSystem::S7));
            // Void instance methods return invisible(self) for pipe-friendly chaining
            if method.returns_unit() {
                lines.push("  invisible(self)".to_string());
            }
            lines.push("}".to_string());
            lines.push(String::new());
        }
    }

    // Create trait namespace for static methods and consts.
    // For S7 classes, use a local variable + attr() to avoid S7's $<- interception.
    let trait_env_var = trait_namespace_env_var(type_ident, trait_name);
    if !static_methods.is_empty() || !consts.is_empty() {
        lines.push(format!("{} <- new.env(parent = emptyenv())", trait_env_var));
        lines.push(String::new());
    }

    // Generate static methods in trait namespace. The wrapper is *assigned*
    // into the local env (`trait_namespace_target(S7, ..)` = `.Type__Trait$m`),
    // but its documented `@name` is the call-site form `Type$Trait$m` — S7's
    // `$` on the class object falls through to the attached attribute, so users
    // still spell it `Type$Trait$m`.
    for method in &static_methods {
        let r_name = method.r_method_name();
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);

        lines.push(format!(
            "#' Static trait method {}::{}()",
            trait_name, r_name
        ));
        let roxygen = RoxygenBuilder::new()
            .name(format!("{}${}${}", type_str, trait_str, r_name))
            .rdname(method_rdname(method, &type_str))
            .build();
        lines.extend(roxygen);

        let call = ctx.static_call();

        lines.push(format!(
            "{} <- function({}) {{",
            ctx.namespace_target(ClassSystem::S7),
            ctx.params
        ));
        ctx.emit_method_prelude(&mut lines, "  ", &r_name);
        lines.extend(ctx.method_body_lines(&call, ClassSystem::S7));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Generate const wrappers in trait namespace (assigned into `.Type__Trait`,
    // documented as `Type$Trait$const` — see the static-method note above).
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let const_str = const_name.to_string();

        let roxygen = RoxygenBuilder::new()
            .name(format!("{}${}${}", type_str, trait_str, const_str))
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        let c_ident = trait_const.c_wrapper_ident_string(type_ident, trait_name);
        let call = DotCallBuilder::new(&c_ident).build();

        lines.push(format!(
            "{} <- function() {{",
            trait_namespace_target(ClassSystem::S7, type_ident, trait_name, &const_str)
        ));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Attach the trait env to the S7 class via attr() to bypass S7's $<- interception.
    // R's $ accessor on S7 objects falls through to attributes, so Type$Trait$method still works.
    if !static_methods.is_empty() || !consts.is_empty() {
        lines.push(format!(
            "attr({}, \"{}\") <- {}",
            type_ident, trait_name, trait_env_var
        ));
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate R6-style R wrapper code.
///
/// R6 classes are defined monolithically (all methods in `R6Class()`), so trait
/// methods cannot be injected into the class definition. Instead, both instance
/// and static trait methods live in the class-scoped `Type$Trait$name`
/// namespace (env-style) — the R6 generator object is an environment, so
/// `Type$Trait <- new.env()` attaches cleanly and `Type$Trait$method(x)`
/// resolves at the call site.
///
/// For `impl Counter for SimpleCounter`, generates:
/// - `SimpleCounter$Counter$value(x)`      -- instance method (takes the object)
/// - `SimpleCounter$Counter$increment(x)`  -- instance method
///
/// This class-qualified shape is collision-free by construction: two R6 impls
/// of one trait on different types no longer share an unqualified
/// `r6_trait_<Trait>_<method>` name (#1115). It also unifies R6 with the Env/S3
/// namespace shape (#1141) — R6 instance and static methods previously
/// disagreed on shape for no functional reason.
fn generate_trait_r6_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> String {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();

    // Header comment
    lines.push(format!(
        "# R6 trait methods for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr(r6)] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push("# Note: R6 trait methods live in the Type$Trait$method namespace".to_string());
    lines.push(String::new());

    // Separate instance methods from static methods
    let instance_methods: Vec<_> = methods.iter().filter(|m| m.has_self).collect();
    let static_methods: Vec<_> = methods.iter().filter(|m| !m.has_self).collect();

    // Create the trait namespace env up front — instance methods now live in it
    // too (not just static methods / consts).
    if !methods.is_empty() || !consts.is_empty() {
        lines.push(format!(
            "{}${} <- new.env(parent = emptyenv())",
            type_ident, trait_name
        ));
        lines.push(String::new());
    }

    // Generate instance methods in the Type$Trait$ namespace
    for method in &instance_methods {
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);
        let target = ctx.namespace_target(ClassSystem::R6);

        // Build parameter list (x first, then others)
        let full_params = if ctx.params.is_empty() {
            "x".to_string()
        } else {
            format!("x, {}", ctx.params)
        };

        // Namespace-member roxygen — a `$<-` assignment target, so roxygen emits
        // no `\usage` and needs no per-formal `@param` docs (matches Env; #1141).
        let roxygen = title_if_split(
            RoxygenBuilder::new()
                .name(target.clone())
                .rdname(method_rdname(method, &type_str)),
            method,
            &target,
        )
        .build();
        lines.extend(roxygen);

        let call = ctx.instance_call(".ptr");

        lines.push(format!("{target} <- function({full_params}) {{"));
        // R6 objects store the ExternalPtr in private$.ptr — extract it for .Call()
        lines.push("  .ptr <- x$.__enclos_env__$private$.ptr".to_string());
        ctx.emit_method_prelude(&mut lines, "  ", &method.r_method_name());
        lines.extend(ctx.method_body_lines(&call, ClassSystem::R6));
        // Void instance methods return invisible(x) for pipe-friendly chaining
        if method.returns_unit() {
            lines.push("  invisible(x)".to_string());
        }
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Generate static methods in Type$Trait$ namespace
    for method in &static_methods {
        let r_name = method.r_method_name();
        let ctx = TraitMethodContext::new(method, type_ident, trait_name);
        let target = ctx.namespace_target(ClassSystem::R6);

        lines.push(format!(
            "#' Static trait method {}::{}()",
            trait_name, r_name
        ));
        let roxygen = RoxygenBuilder::new()
            .name(target.clone())
            .rdname(method_rdname(method, &type_str))
            .build();
        lines.extend(roxygen);

        let call = ctx.static_call();

        lines.push(format!("{target} <- function({}) {{", ctx.params));
        ctx.emit_method_prelude(&mut lines, "  ", &r_name);
        lines.extend(ctx.method_body_lines(&call, ClassSystem::R6));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Generate const wrappers in Type$Trait$ namespace
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let const_str = const_name.to_string();
        let target = trait_namespace_target(ClassSystem::R6, type_ident, trait_name, &const_str);

        let roxygen = RoxygenBuilder::new()
            .name(target.clone())
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        let c_ident = trait_const.c_wrapper_ident_string(type_ident, trait_name);
        let call = DotCallBuilder::new(&c_ident).build();

        lines.push(format!("{target} <- function() {{"));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}
