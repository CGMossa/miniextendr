//! R6-class R wrapper generator.
//!
//! Generates an `R6::R6Class(...)` definition with **mutable, reference
//! semantics**: `obj$method()` dispatches via R6, and `&mut self` methods
//! modify state in place without re-binding `obj`. Supports private methods,
//! active bindings (computed/settable properties), lifecycle hooks
//! (`finalize`, `deep_clone`), and inheritance via `r6(inherit = ...)`.
//! Slower than env (R6 dispatch chain) and requires the R6 package; no value
//! semantics — for value-semantics formal OOP, use S7.
//!
//! ## Emission shape (`$set()` form, #369)
//!
//! roxygen2 8.0.0 documents R6 methods added *after* class creation via
//! `Class$set("public"/"active", "name", function(...) {...})` from the roxygen
//! block directly preceding the `$set` call. We emit a minimal `R6Class` core
//! (only `initialize` inline in `public`, plus the private list) followed by one
//! `$set` block per public method. Active-binding `@field` tags stay on the
//! class block: roxygen2's dynamic `$set()` parser treats every target as a
//! method, including `$set("active", ...)`, so putting field tags immediately
//! above an active-binding call produces spurious "can't find matching R6
//! method" warnings.
//!
//! ```r
//! #' @field binding_name <binding doc>
//! ClassName <- R6::R6Class("ClassName",
//!   public = list(initialize = function(...) { ... }),
//!   private = list(.ptr = NULL),
//!   lock_objects = TRUE, lock_class = FALSE, cloneable = FALSE
//! )
//!
//! #' @description <method doc>
//! #' @param x <param doc>
//! ClassName$set("public", "method_name", function(x) { ... })
//!
//! # No adjacent roxygen block: the @field tag is on the class documentation.
//! ClassName$set("active", "binding_name", function(value) { ... })
//! ```
//!
//! `initialize` stays inline (R6Class needs a well-formed core; its `$new` doc
//! stays on the class page). Private methods, `finalize`, `deep_clone`, and the
//! `.ptr` field stay inline in the `private` list. The `$set` calls are part of
//! the same wrapper fragment string as the class definition, so their ordering
//! after the class is inherent.
//!
//! Because R6's `$set()` refuses to modify a locked class, the generator always
//! creates the class with `lock_class = FALSE` and, when the user asked for
//! `lock_class = TRUE`, re-locks it with `ClassName$lock()` after every `$set`
//! (including sidecar accessors) has run. The observable semantics are identical
//! to a class born locked — the unlocked window is confined to package load.

use super::{ParsedImpl, ParsedMethod};
use crate::r_class_formatter::class_ref_or_verbatim;

/// Build the `stopifnot()` precondition lines for the setter branch of a
/// combined getter/setter active binding.
///
/// This is the same precondition block the standalone `set_*` method gets via
/// [`crate::r_class_formatter::MethodContext::precondition_checks`] — the
/// active-binding branch used to skip it (audit 2026-07-06 finding 4), so
/// `obj$prop <- "bad"` bypassed the R-level type check the standalone setter
/// enforces.
///
/// R6 active bindings always receive the assigned value through a formal
/// named `value`, while the Rust setter's parameter may have any name, so the
/// first non-receiver parameter (the only one the binding forwards — see the
/// `.with_args(&["value"])` call site) is renamed to `value` before the
/// checks are built. Any additional parameters are ignored: the binding never
/// passes them, and checks referencing their names would error at runtime.
fn active_setter_precondition_checks(setter: &ParsedMethod) -> Vec<String> {
    let Some(mut value_arg) = setter.sig.inputs.iter().find_map(|arg| match arg {
        syn::FnArg::Typed(pat_type) => Some(pat_type.clone()),
        syn::FnArg::Receiver(_) => None,
    }) else {
        return Vec::new();
    };

    let mut per_param = setter.method_attrs.per_param.clone();
    if let syn::Pat::Ident(pat_ident) = value_arg.pat.as_mut() {
        let rust_name = pat_ident.ident.to_string();
        if rust_name != "value" {
            if let Some(attrs) = per_param.remove(&rust_name) {
                per_param.insert("value".to_string(), attrs);
            }
            pat_ident.ident = syn::Ident::new("value", pat_ident.ident.span());
        }
    }

    let mut inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]> =
        syn::punctuated::Punctuated::new();
    inputs.push(syn::FnArg::Typed(value_arg));

    crate::r_class_formatter::build_method_precondition_checks(
        &inputs,
        &per_param,
        setter.method_attrs.coerce,
    )
}

/// Build class-level `@field` documentation for an R6 active binding.
///
/// roxygen2 8.0's dynamic `$set()` parser does not distinguish `public` from
/// `active`: a roxygen block immediately before either call is classified as
/// method documentation. Active-binding docs therefore belong on the class
/// block, where roxygen2 resolves them against the runtime class's active
/// bindings without inventing a method owner.
fn field_tag_documents(tag: &str, field_name: &str) -> bool {
    let Some(rest) = tag.trim_start().strip_prefix("@field ") else {
        return false;
    };
    let Some(names) = rest.split_whitespace().next() else {
        return false;
    };
    names.split(',').any(|name| name.trim() == field_name)
}

fn active_binding_field_lines(method: &ParsedMethod, class_doc_tags: &[String]) -> Vec<String> {
    let prop_name = method
        .method_attrs
        .r6
        .prop
        .clone()
        .unwrap_or_else(|| method.r_method_name());

    if class_doc_tags
        .iter()
        .any(|tag| field_tag_documents(tag, &prop_name))
    {
        return Vec::new();
    }

    if method.method_attrs.noexport || method.method_attrs.internal {
        return vec![format!("#' @field {prop_name} (internal)")];
    }

    let description = method.doc_tags.iter().find_map(|tag| {
        tag.strip_prefix("@field ")
            .and_then(|rest| rest.split_once(char::is_whitespace).map(|(_, desc)| desc))
            .or_else(|| tag.strip_prefix("@description "))
            .or_else(|| tag.strip_prefix("@title "))
    });
    let Some(description) = description else {
        return vec![format!("#' @field {prop_name} Active binding.")];
    };

    let mut description_lines = description.lines();
    let first = description_lines.next().unwrap_or("Active binding.");
    let mut lines = vec![format!("#' @field {prop_name} {first}")];
    lines.extend(description_lines.map(|line| format!("#' {line}")));
    lines
}

/// Marker prefix emitted by the proc-macro when a subclass method param might be
/// inherited from a parent class documented in-package. Resolved at write-time in
/// `registry.rs` to either nothing (parent is documented → roxygen2 inherits) or
/// `(no documentation available)` (parent not found or not in-package).
pub(crate) const MX_INHERITED_PARAM_PREFIX: &str = ".__MX_INHERITED_PARAM__(";

/// Generates the complete R wrapper string for an R6-style class.
///
/// Produces an `R6::R6Class(...)` core definition followed by per-method
/// `$set()` blocks (see the module docs for the full `$set`-form shape, #369):
/// - `initialize` method (inline in `public`): calls the Rust `new` constructor,
///   or accepts a pre-made `.ptr` when static methods return `Self` (factory pattern)
/// - Public methods: one `ClassName$set("public", "name", function(...) {...})`
///   block per `&self`/`&mut self` instance method, each with its own roxygen
/// - Private methods (inline in `private`): methods marked with `#[miniextendr(private)]`
/// - Active bindings: one `ClassName$set("active", "name", function(...) {...})`
///   block per getter/setter property (`#[miniextendr(r6(prop = "..."))]`)
/// - Private `.ptr` field: holds the `ExternalPtr` to the Rust struct
/// - Finalizer (inline in `private`): optional destructor called when the R6 object is garbage-collected
/// - Deep clone (inline in `private`): optional custom clone logic via `#[miniextendr(r6(deep_clone))]`
/// - Static methods: emitted as `ClassName$method_name <- function(...)` outside the class
/// - Class options: `lock_objects`, `lock_class`, `cloneable`, `portable`, `inherit`
///
/// Also generates roxygen2 documentation blocks for the class, its methods,
/// and active bindings.
pub fn generate_r6_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    use crate::r_class_formatter::{ClassDocBuilder, MethodDocBuilder, ParsedImplExt};

    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;

    let mut lines = Vec::new();

    // Start R6Class definition with documentation
    lines.extend(
        ClassDocBuilder::new(&class_name, type_ident, class_doc_tags, "R6")
            .with_imports("@importFrom R6 R6Class")
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
        // Insert before @export (which is last)
        let insert_pos = lines.len().saturating_sub(1);
        lines.insert(insert_pos, format!("#' {}", lc_import));
    }

    // Every R6 ExternalPtr class accepts .ptr so write-time cross-class return
    // wrapping can land in the target constructor.
    if !crate::roxygen::has_roxygen_tag(class_doc_tags, "param .ptr") {
        // Insert before @export (which is last)
        let insert_pos = lines.len().saturating_sub(1);
        lines.insert(
            insert_pos,
            "#' @param .ptr Internal pointer (used by static methods, not for direct use)."
                .to_string(),
        );
    }

    // Active bindings are documented on the class block. roxygen2 8.0 parses
    // every dynamic `$set()` target as a method, even for `$set("active", ...)`;
    // an adjacent `@field` block consequently warns that it cannot find a
    // matching R6 method. Class-level fields are resolved against the runtime
    // R6 class and retain the same rendered Active bindings section.
    let active_method_contexts: Vec<_> = parsed_impl.active_instance_method_contexts().collect();
    let active_field_lines: Vec<_> = active_method_contexts
        .iter()
        .flat_map(|ctx| active_binding_field_lines(ctx.method, class_doc_tags))
        .collect();
    let active_field_insert = lines
        .iter()
        .position(|line| line == "#' @export")
        .unwrap_or(lines.len());
    lines.splice(active_field_insert..active_field_insert, active_field_lines);

    // R6Class definition — optionally include inherit.
    // Use a placeholder so the resolver can look up the actual R class name
    // at cdylib write time (handles `class = "Override"` on the parent).
    if let Some(ref parent) = parsed_impl.r6_inherit {
        let parent_ref = class_ref_or_verbatim(parent);
        lines.push(format!(
            "{} <- R6::R6Class(\"{}\", inherit = {},",
            class_name, class_name, parent_ref
        ));
    } else {
        lines.push(format!("{} <- R6::R6Class(\"{}\",", class_name, class_name));
    }

    // Portable flag (only emit if explicitly set to FALSE, since TRUE is default)
    if parsed_impl.r6_portable == Some(false) {
        lines.push("  portable = FALSE,".to_string());
    }

    // Public list
    lines.push("  public = list(".to_string());

    // Class-level @param names — params documented at class level (in class_doc_tags)
    // that roxygen2 8.0.0 inherits into all method docs automatically. Constructor and
    // method loops skip emitting `(no documentation available)` for covered names.
    let class_param_names = &parsed_impl.class_param_names;

    // Constructor (initialize) - accepts either normal params or a pre-made .ptr.
    // If there's no explicit `new()`, generate a minimal initialize(.ptr) so
    // other class methods can call $new(.ptr = val).
    if let Some(ctx) = parsed_impl.constructor_context() {
        lines.push(format!("    {}", ctx.source_comment(type_ident)));
        // Add inline roxygen documentation for initialize method
        // Note: @title is replaced with @description for R6 inline docs (roxygen requirement)
        let has_description = ctx
            .method
            .doc_tags
            .iter()
            .any(|t| t.starts_with("@description ") || t.starts_with("@title "));
        if !has_description {
            lines.push(format!(
                "    #' @description Create a new `{}`.",
                class_name
            ));
        }
        for tag in &ctx.method.doc_tags {
            for line in tag.lines() {
                let line = if line.starts_with("@title ") {
                    line.replacen("@title ", "@description ", 1)
                } else {
                    line.to_string()
                };
                lines.push(format!("    #' {}", line));
            }
        }
        // Document constructor params that aren't already documented.
        // Class-level @param tags (in class_doc_tags) are inherited by all methods
        // via roxygen2 8.0.0 — skip emitting a placeholder for params covered there.
        let ctor_mx_doc = ctx.match_arg_doc_placeholders();
        for param in ctx.params.split(", ").filter(|p| !p.is_empty()) {
            let param_name = param.split('=').next().unwrap_or(param).trim();
            if param_name == ".ptr" {
                continue;
            }
            let already_documented =
                crate::roxygen::param_documented(&ctx.method.doc_tags, param_name);
            if !already_documented {
                // Param covered by class-level @param → roxygen2 inherits; emit nothing.
                if class_param_names.contains(param_name) {
                    continue;
                }
                // match_arg'd constructor params get the write-time placeholder
                // so the cdylib pass renders `One of "A", "B".` (#210).
                let body = ctor_mx_doc
                    .get(param_name)
                    .map(String::as_str)
                    .unwrap_or("(no documentation available)");
                lines.push(format!("    #' @param {} {}", param_name, body));
            }
        }

        // Precondition checks for constructor parameters
        let ctor_preconditions = ctx.precondition_checks();

        // Missing param prelude for constructor

        let ctor_match_arg = ctx.match_arg_prelude();

        let full_params = if ctx.params.is_empty() {
            ".ptr = NULL".to_string()
        } else {
            format!("{}, .ptr = NULL", ctx.params)
        };
        lines.push(format!("    initialize = function({}) {{", full_params));
        // Preconditions + match.arg only when not using .ptr shortcut
        if !ctor_preconditions.is_empty() || !ctor_match_arg.is_empty() {
            lines.push("      if (is.null(.ptr)) {".to_string());
            for check in &ctor_preconditions {
                lines.push(format!("        {}", check));
            }
            for line in &ctor_match_arg {
                lines.push(format!("        {}", line));
            }
            lines.push("      }".to_string());
        }
        lines.push("      if (!is.null(.ptr)) {".to_string());
        lines.push("        private$.ptr <- .ptr".to_string());
        lines.push("      } else {".to_string());
        lines.push(format!("        .val <- {}", ctx.static_call()));
        // Use shared condition switch (supports error!/warning!/message!/condition!).
        for check_line in crate::method_return_builder::condition_check_lines("        ") {
            lines.push(check_line);
        }
        lines.push("        private$.ptr <- .val".to_string());
        lines.push("      }".to_string());
        // initialize is the sole `public = list(...)` member — every other public
        // method migrated to a `$set("public", ...)` block below — so no trailing comma.
        lines.push("    }".to_string());
    } else {
        // No explicit new() constructor, but cross-class returns need
        // $new(.ptr = val). Generate a minimal initialize that only accepts .ptr.
        lines.push(format!(
            "    #' @description Create a new `{}`.",
            class_name
        ));
        lines.push("    initialize = function(.ptr = NULL) {".to_string());
        lines.push("      if (!is.null(.ptr)) {".to_string());
        lines.push("        private$.ptr <- .ptr".to_string());
        lines.push("      }".to_string());
        lines.push("    }".to_string());
    }

    lines.push("  ),".to_string());

    // Private list - includes .ptr and any private methods
    lines.push("  private = list(".to_string());

    // Private instance methods
    for ctx in parsed_impl.private_instance_method_contexts() {
        lines.push(format!("    {}", ctx.source_comment(type_ident)));
        lines.push(format!(
            "    {} = function({}) {{",
            ctx.method.r_method_name(),
            ctx.params
        ));

        // Inject r_entry
        if let Some(ref entry) = ctx.method.method_attrs.r_entry {
            for line in entry.lines() {
                lines.push(format!("      {}", line));
            }
        }
        // Inject on.exit cleanup
        if let Some(ref on_exit) = ctx.method.method_attrs.r_on_exit {
            lines.push(format!("      {}", on_exit.to_r_code()));
        }
        // Inject missing param defaults
        // Inject match.arg validation for match_arg/choices params
        for line in ctx.match_arg_prelude() {
            lines.push(format!("      {}", line));
        }
        // Inject r_post_checks
        if let Some(ref post) = ctx.method.method_attrs.r_post_checks {
            for line in post.lines() {
                lines.push(format!("      {}", line));
            }
        }

        let call = ctx.instance_call("private$.ptr");
        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_return_class_from_method(ctx.method)
            .with_indent(6);
        lines.extend(return_builder.build_r6_body());

        lines.push("    },".to_string());
    }

    // Finalizer (if any)
    if let Some(finalizer) = parsed_impl.finalizer() {
        let c_ident = finalizer
            .c_wrapper_ident(type_ident, parsed_impl.label())
            .to_string();
        let finalize_call = crate::r_wrapper_builder::DotCallBuilder::new(&c_ident)
            .null_call_attribution()
            .with_self("private$.ptr")
            .build();
        lines.push(format!("    finalize = function() {finalize_call},"));
    }

    // deep_clone (if any method marked with #[miniextendr(r6(deep_clone))])
    if let Some(dc_method) = parsed_impl
        .methods
        .iter()
        .find(|m| m.method_attrs.r6.deep_clone && m.should_include())
    {
        let c_ident = dc_method
            .c_wrapper_ident(type_ident, parsed_impl.label())
            .to_string();
        let deep_clone_call = crate::r_wrapper_builder::DotCallBuilder::new(&c_ident)
            .null_call_attribution()
            .with_self("private$.ptr")
            .with_args(&["name", "value"])
            .build();
        lines.push(format!(
            "    deep_clone = function(name, value) {deep_clone_call},"
        ));
    }

    // .ptr field (always last, no trailing comma)
    lines.push("    .ptr = NULL".to_string());
    lines.push("  ),".to_string());

    // Class options
    let lock_objects = parsed_impl.r6_lock_objects.unwrap_or(true);
    let lock_class = parsed_impl.r6_lock_class.unwrap_or(false);
    let cloneable = parsed_impl.r6_cloneable.unwrap_or(false);
    lines.push(format!(
        "  lock_objects = {},",
        if lock_objects { "TRUE" } else { "FALSE" }
    ));
    // The class is always CREATED unlocked so the `$set()` blocks below (public
    // methods, active bindings, and any `#[derive(ExternalPtr)]` sidecar
    // accessors) can add to it — R6's `$set` refuses to touch a locked class
    // ("Can't modify a locked R6 class"). When the user requested
    // `lock_class = TRUE` we re-apply the lock via `ClassName$lock()` after every
    // `$set` has run (see the end of this function); the net effect is identical
    // to a class born locked, since the unlocked window is entirely within
    // package-load evaluation.
    lines.push("  lock_class = FALSE,".to_string());
    lines.push(format!(
        "  cloneable = {}",
        if cloneable { "TRUE" } else { "FALSE" }
    ));
    lines.push(")".to_string());

    // Public instance methods — one `$set("public", ...)` block each, so roxygen2
    // 8.0.0 documents every method from the block directly above its `$set` call
    // (#369). No `overwrite = TRUE`: these are the class's own methods, and the
    // default `overwrite = FALSE` catches accidental name collisions loudly.
    let public_method_contexts: Vec<_> = parsed_impl.public_instance_method_contexts().collect();
    for ctx in &public_method_contexts {
        lines.push(String::new());
        lines.push(ctx.source_comment(type_ident));
        // Add roxygen documentation for this method.
        // Note: @title is replaced with @description for R6 method docs (roxygen requirement)
        let r_name = ctx.method.r_method_name();
        let has_description = ctx
            .method
            .doc_tags
            .iter()
            .any(|t| t.starts_with("@description ") || t.starts_with("@title "));
        if !has_description {
            lines.push(format!("#' @description Method `{}`.", r_name));
        }
        // roxygen2 folds a `Class$set("public", ...)` block into the class
        // block, so a method-level `@rdname` / `@name` here would make it
        // "contain only one @rdname" and fail. R6 instance methods always
        // document on the class page (static methods can still split, #1438).
        for tag in &ctx.method.doc_tags {
            if crate::roxygen::roxygen_tag_name(tag).is_some_and(|n| n == "rdname" || n == "name") {
                continue;
            }
            for line in tag.lines() {
                let line = if line.starts_with("@title ") {
                    line.replacen("@title ", "@description ", 1)
                } else {
                    line.to_string()
                };
                lines.push(format!("#' {}", line));
            }
        }
        // Document method params that aren't already documented.
        // Class-level @param tags are inherited by roxygen2 8.0.0 — skip covered params.
        // For subclass methods (r6_inherit set) and params not covered at class level,
        // emit a write-time marker so the registry can detect in-package parent docs
        // and suppress the placeholder (letting roxygen2 pull from the parent method).
        let method_mx_doc = ctx.match_arg_doc_placeholders();
        let r_method_name = ctx.method.r_method_name();
        for param in ctx.params.split(", ").filter(|p| !p.is_empty()) {
            let param_name = param.split('=').next().unwrap_or(param).trim();
            let already_documented =
                crate::roxygen::param_documented(&ctx.method.doc_tags, param_name);
            if !already_documented {
                // Param covered by class-level @param → roxygen2 inherits; emit nothing.
                if class_param_names.contains(param_name) {
                    continue;
                }
                let body = method_mx_doc
                    .get(param_name)
                    .map(String::as_str)
                    .unwrap_or("(no documentation available)");
                // For subclass methods: if there is an in-package parent, emit a
                // write-time marker. The registry pass will resolve it to nothing
                // (parent documented → roxygen2 inherits) or to the fallback text.
                if let Some(ref parent) = parsed_impl.r6_inherit
                    && body == "(no documentation available)"
                {
                    lines.push(format!(
                        "#' {}class=\"{}\", parent=\"{}\", method=\"{}\", param=\"{}\")",
                        MX_INHERITED_PARAM_PREFIX, class_name, parent, r_method_name, param_name,
                    ));
                } else {
                    lines.push(format!("#' @param {} {}", param_name, body));
                }
            }
        }
        lines.push(format!(
            "{}$set(\"public\", \"{}\", function({}) {{",
            class_name, r_name, ctx.params
        ));

        let what = format!("{}${}", class_name, r_name);
        ctx.emit_method_prelude(&mut lines, "  ", &what);

        let call = ctx.instance_call("private$.ptr");
        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_return_class_from_method(ctx.method)
            .with_indent(2); // top-level `$set` closure body indents 2 spaces
        lines.extend(return_builder.build_r6_body());

        lines.push("})".to_string());
    }

    // Active bindings. Their @field documentation is intentionally attached to
    // the class block above; do not put a roxygen block before these dynamic
    // `$set("active", ...)` calls (roxygen2 misclassifies it as method docs).
    for ctx in &active_method_contexts {
        lines.push(String::new());

        // Determine the property name (from r6_prop or method name)
        let prop_name = ctx
            .method
            .method_attrs
            .r6
            .prop
            .clone()
            .unwrap_or_else(|| ctx.method.r_method_name());

        // Check if there's a matching setter for this property
        let setter = parsed_impl.find_setter_for_prop(&prop_name);

        if let Some(setter_method) = setter {
            // Combined getter/setter active binding
            // Format: name = function(value) { if (missing(value)) getter else setter }
            lines.push(format!(
                "{}$set(\"active\", \"{}\", function(value) {{",
                class_name, prop_name
            ));
            lines.push("  if (missing(value)) {".to_string());

            // Getter call — same condition re-raise guard as the
            // getter-only active binding path below.
            let getter_call = ctx.instance_call("private$.ptr");
            let getter_strategy = crate::ReturnStrategy::for_method(ctx.method);
            let getter_builder = crate::MethodReturnBuilder::new(getter_call)
                .with_strategy(getter_strategy)
                .with_class_name(class_name.clone())
                .with_return_class_from_method(ctx.method)
                .with_indent(4);
            lines.extend(getter_builder.build_r6_body());

            lines.push("  } else {".to_string());

            // Same `stopifnot` precondition block the standalone `set_*`
            // method gets (audit 2026-07-06 finding 4): without it,
            // `obj$prop <- <bad value>` skipped the R-level type check.
            for check in active_setter_precondition_checks(setter_method) {
                lines.push(format!("    {}", check));
            }

            // Setter call — construct directly, then re-raise any
            // transported Rust condition (the bare `.Call()` used to
            // discard the tagged condition value, silently dropping the
            // assignment error).
            let setter_c_ident = setter_method
                .c_wrapper_ident(type_ident, parsed_impl.label.as_deref())
                .to_string();
            let setter_call = crate::r_wrapper_builder::DotCallBuilder::new(&setter_c_ident)
                .with_self("private$.ptr")
                .with_args(&["value"])
                .build();
            lines.push(format!("    .val <- {}", setter_call));
            lines.extend(crate::method_return_builder::condition_check_lines("    "));
            lines.push("    invisible(self)".to_string());

            lines.push("  }".to_string());
            lines.push("})".to_string());
        } else {
            // Getter-only active binding (no parameters besides self)
            // Format: name = function() { ... }
            lines.push(format!(
                "{}$set(\"active\", \"{}\", function() {{",
                class_name, prop_name
            ));

            let call = ctx.instance_call("private$.ptr");
            let strategy = crate::ReturnStrategy::for_method(ctx.method);
            let return_builder = crate::MethodReturnBuilder::new(call)
                .with_strategy(strategy)
                .with_class_name(class_name.clone())
                .with_return_class_from_method(ctx.method)
                .with_indent(2); // top-level `$set` closure body indents 2 spaces
            lines.extend(return_builder.build_r6_body());

            lines.push("})".to_string());
        }
    }

    // If r_data_accessors is set, apply sidecar active bindings from #[derive(ExternalPtr)]
    if parsed_impl.r_data_accessors {
        let type_name = type_ident.to_string();
        lines.push(format!(
            ".rdata_active_bindings_{}({})",
            type_name, class_name
        ));
    }

    // Check if class has @noRd. A plain `noexport` (without `internal`) is folded
    // in here too — it must suppress Rd contribution entirely, matching
    // `ClassDocBuilder::build`'s `suppress_rd` gate (see r_class_formatter.rs).
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(class_doc_tags, "noRd")
        || (parsed_impl.noexport && !parsed_impl.internal);

    // Static methods as separate functions on the class object
    for ctx in parsed_impl.static_method_contexts() {
        let method_name = ctx.method.r_method_name();
        let static_method_name = format!("{}${}", class_name, method_name);
        lines.push(String::new());

        lines.push(ctx.source_comment(type_ident));
        let method_doc =
            MethodDocBuilder::new(&class_name, &method_name, type_ident, &ctx.method.doc_tags)
                .with_name_prefix("$")
                .with_class_no_rd(class_has_no_rd);
        lines.extend(method_doc.build());

        lines.push(format!(
            "{} <- function({}) {{",
            static_method_name, ctx.params
        ));

        let what = format!("{}${}", class_name, method_name);
        ctx.emit_method_prelude(&mut lines, "  ", &what);

        let strategy = crate::ReturnStrategy::for_method(ctx.method);
        let return_builder = crate::MethodReturnBuilder::new(ctx.static_call())
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_return_class_from_method(ctx.method);
        lines.extend(return_builder.build_r6_body());

        lines.push("}".to_string());
    }

    // Re-apply the requested `lock_class = TRUE` now that every `$set()` block —
    // public methods, active bindings, and (above) any sidecar accessors — has
    // run. The class was created with `lock_class = FALSE` (see the class-options
    // block) precisely so those `$set` calls would succeed; `$lock()` restores
    // the locked semantics. Emitted last so nothing further can `$set` it.
    if lock_class {
        lines.push(String::new());
        lines.push(format!("{}$lock()", class_name));
    }

    lines.join("\n")
}
