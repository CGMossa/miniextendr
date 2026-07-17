//! Env-class R wrapper generator.
//!
//! Generates an R environment (`new.env(parent = emptyenv())`) that serves as
//! the class namespace, with `obj$method()` dispatched through an `$.ClassName`
//! S3 method. This is the **fastest** of the six class systems and has **no R
//! package dependencies**, but provides no formal class machinery: no
//! inheritance, no multi-dispatch, no slot validation. Pick env for simple
//! ExternalPtr-backed APIs; reach for R6/S3/S4/S7 when you need dispatch or
//! formal class semantics.

use super::ParsedImpl;

/// Generates the complete R wrapper string for an environment-based class.
///
/// Produces an R environment object (`new.env(parent = emptyenv())`) that serves as a
/// class namespace, with methods attached as `ClassName$method_name`. This pattern
/// supports both inherent methods and trait namespace dispatch via `$`/`[[`.
///
/// The generated code includes:
/// - Class environment: `ClassName <- new.env(parent = emptyenv())`
/// - Constructor: `ClassName$new(...)` that calls the Rust `new` function, sets
///   `class(self) <- "ClassName"`, and returns the ExternalPtr as `self`
/// - Instance methods: `ClassName$method(x = self, ...)` using default-arg binding
///   so that `$` dispatch re-parents the environment to make `self` visible
/// - Static methods: `ClassName$method(...)` that call Rust directly
/// - `$.ClassName` S3 method: dispatches `obj$method(...)` by looking up the method
///   in the class environment, binding `self` for instance methods, and supporting
///   trait namespace environments (nested envs with `.__mx_instance__` attributes)
/// - `[[.ClassName` alias: delegates to `$.ClassName`
///
/// Roxygen2 documentation is generated for the class, each method, and the
/// dispatch methods, with appropriate `@export`/`@keywords internal`/`@noRd` tags.
pub fn generate_env_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::{
        ClassDocBuilder, MethodDocBuilder, ParsedImplExt, should_export_from_tags,
    };

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    // Check if class has @noRd - if so, skip method documentation. A plain
    // `noexport` (without `internal`) is folded in too — it must suppress Rd
    // contribution entirely, matching `ClassDocBuilder::build`'s `suppress_rd` gate.
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(&parsed_impl.doc_tags, "noRd")
        || (parsed_impl.noexport && !parsed_impl.internal);

    let mut lines = Vec::new();

    // Class environment documentation and definition
    lines.extend(
        ClassDocBuilder::new(&class_name, type_ident, &parsed_impl.doc_tags, "")
            .with_export_control(parsed_impl.internal, parsed_impl.noexport)
            .build(),
    );
    // Inject lifecycle imports from methods into class-level roxygen block
    if let Some(lc_import) = crate::lifecycle::collect_lifecycle_imports(
        parsed_impl
            .methods
            .iter()
            .filter_map(|m| m.method_attrs.lifecycle.as_ref()),
    ) {
        let insert_pos = lines.len().saturating_sub(1);
        lines.insert(insert_pos, format!("#' {}", lc_import));
    }
    lines.push(format!("{} <- new.env(parent = emptyenv())", class_name));
    lines.push(String::new());

    // Constructor
    if let Some(ctx) = parsed_impl.constructor_context() {
        lines.push(ctx.source_comment(type_ident));
        // Skip method documentation if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, "new", type_ident, &ctx.method.doc_tags)
                    .with_name_prefix("$")
                    .with_params_as_details();
            lines.extend(method_doc.build());
        }
        lines.push(format!("{}$new <- function({}) {{", class_name, ctx.params));
        for check in ctx.precondition_checks() {
            lines.push(format!("  {}", check));
        }
        // Inject match.arg validation for match_arg/choices params
        for line in ctx.match_arg_prelude() {
            lines.push(format!("  {}", line));
        }
        lines.push(format!("  .val <- {}", ctx.static_call()));
        lines.extend(crate::method_return_builder::condition_check_lines("  "));
        lines.push("  self <- .val".to_string());
        lines.push(format!("  class(self) <- \"{}\"", class_name));
        lines.push("  self".to_string());
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Instance methods
    for ctx in parsed_impl.instance_method_contexts() {
        let method_name = ctx.method.r_method_name();
        lines.push(ctx.source_comment(type_ident));
        // Skip method documentation if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                    .with_name_prefix("$")
                    .with_params_as_details();
            lines.extend(method_doc.build());
        }

        lines.push(format!(
            "{}${} <- function({}) {{",
            class_name, method_name, ctx.params
        ));

        let what = format!("{}${}", class_name, method_name);
        ctx.emit_method_prelude(&mut lines, "  ", &what);

        let call = ctx.instance_call("self");
        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_return_class_from_method(ctx.method, &type_ident.to_string());
        lines.extend(return_builder.build());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Static methods
    for ctx in parsed_impl.static_method_contexts() {
        let method_name = ctx.method.r_method_name();
        lines.push(ctx.source_comment(type_ident));
        // Skip method documentation if class has @noRd
        if !class_has_no_rd {
            let method_doc =
                MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                    .with_name_prefix("$")
                    .with_params_as_details();
            lines.extend(method_doc.build());
        }

        lines.push(format!(
            "{}${} <- function({}) {{",
            class_name, method_name, ctx.params
        ));

        let what = format!("{}${}", class_name, method_name);
        ctx.emit_method_prelude(&mut lines, "  ", &what);

        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_return_class_from_method(ctx.method, &type_ident.to_string());
        lines.extend(return_builder.build());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // $ dispatch - export as S3 methods
    // Handles both functions (inherent methods) and environments (trait namespaces)
    let should_export = should_export_from_tags(
        &parsed_impl.doc_tags,
        parsed_impl.noexport || parsed_impl.internal,
    );

    // Generate roxygen tags for dispatch methods.
    // roxygen2 8.0.0+ enforces that any `generic.class`-named function carry
    // @export or @exportS3Method (@noRd alone doesn't satisfy the check). We
    // always emit @export so roxygen2 emits a properly-quoted
    // `S3method("$", Class)` / `S3method("[[", Class)` (bare @exportS3Method
    // skips the operator-name quoting and produces invalid NAMESPACE entries).
    // For internal/noexport classes the @rdname target is dropped so the
    // helpers don't bleed into the user-visible Rd page.
    if class_has_no_rd {
        lines.push("#' @noRd".to_string());
        lines.push("#' @export".to_string());
    } else if !should_export {
        lines.push("#' @export".to_string());
    } else {
        lines.push(format!("#' @rdname {}", class_name));
        lines.push("#' @param self The object instance.".to_string());
        lines.push("#' @param name Method name for dispatch.".to_string());
        lines.push("#' @export".to_string());
    }
    lines.push(format!("`$.{}` <- function(self, name) {{", class_name));
    lines.push(format!("  obj <- {}[[name]]", class_name));
    lines.push("  if (is.environment(obj)) {".to_string());
    lines.push("    # Trait namespace - wrap instance methods to prepend self".to_string());
    lines.push("    bound <- new.env(parent = emptyenv())".to_string());
    lines.push("    for (method_name in names(obj)) {".to_string());
    lines.push("      method <- obj[[method_name]]".to_string());
    lines.push("      if (is.function(method)) {".to_string());
    lines.push("        if (isTRUE(attr(method, \".__mx_instance__\"))) {".to_string());
    lines.push("          local({".to_string());
    lines.push("            m <- method".to_string());
    lines.push("            bound[[method_name]] <<- function(...) m(self, ...)".to_string());
    lines.push("          })".to_string());
    lines.push("        } else {".to_string());
    lines.push("          bound[[method_name]] <- method".to_string());
    lines.push("        }".to_string());
    lines.push("      }".to_string());
    lines.push("    }".to_string());
    lines.push("    bound".to_string());
    lines.push("  } else if (is.null(obj)) {".to_string());
    lines.push("    # Not found at top level -- search trait namespace environments".to_string());
    lines.push(format!("    for (ns_name in names({})) {{", class_name));
    lines.push(format!("      ns <- {}[[ns_name]]", class_name));
    lines.push(
        "      if (is.environment(ns) && exists(name, envir = ns, inherits = FALSE)) {".to_string(),
    );
    lines.push("        method <- ns[[name]]".to_string());
    lines.push(
        "        if (is.function(method) && isTRUE(attr(method, \".__mx_instance__\"))) {"
            .to_string(),
    );
    lines.push("          # Instance method -- bind self as first arg".to_string());
    lines.push("          m <- method".to_string());
    lines.push("          s <- self".to_string());
    lines.push("          return(function(...) m(s, ...))".to_string());
    lines.push("        } else if (is.function(method)) {".to_string());
    lines.push("          return(method)".to_string());
    lines.push("        }".to_string());
    lines.push("      }".to_string());
    lines.push("    }".to_string());
    lines.push("    NULL".to_string());
    lines.push("  } else {".to_string());
    lines.push("    environment(obj) <- environment()".to_string());
    lines.push("    obj".to_string());
    lines.push("  }".to_string());
    lines.push("}".to_string());
    if class_has_no_rd {
        lines.push("#' @noRd".to_string());
        lines.push("#' @export".to_string());
    } else if !should_export {
        lines.push("#' @export".to_string());
    } else {
        lines.push(format!("#' @rdname {}", class_name));
        lines.push("#' @export".to_string());
    }
    lines.push(format!("`[[.{}` <- `$.{}`", class_name, class_name));

    lines.join("\n")
}
