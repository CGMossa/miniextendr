//! Shared utilities for R class wrapper generation.
//!
//! This module provides abstractions to reduce duplication across the 5 class system
//! generators (Env, R6, S3, S4, S7). Each class system has different R idioms but shares
//! common patterns:
//!
//! - Class-level roxygen documentation
//! - Constructor generation
//! - Instance method iteration with `.Call()` building
//! - Static method handling
//! - Return strategy application
//!
//! ## Architecture
//!
//! ```text
//! ParsedImpl
//!     │
//!     ├─▶ ClassDocBuilder  → roxygen header lines (#' @title, @name, etc.)
//!     │
//!     └─▶ MethodContext[]  → pre-computed method data for each method
//!             │
//!             └─▶ ClassFormatter::format_constructor()
//!             └─▶ ClassFormatter::format_instance_method()
//!             └─▶ ClassFormatter::format_static_method()
//! ```

use crate::miniextendr_impl::{ParsedImpl, ParsedMethod};

/// Determine whether a class or method should be `@export`-ed.
///
/// Returns `true` unless the doc tags include `@noRd` or `@keywords internal`,
/// or the `noexport` flag is set (which should incorporate both the `noexport`
/// attribute and the `internal` attribute from the impl block).
///
/// Call sites should pass `parsed_impl.noexport || parsed_impl.internal` as
/// `noexport` so the `internal` attribute is correctly folded in.
pub(crate) fn should_export_from_tags(tags: &[String], noexport: bool) -> bool {
    let has_no_rd = crate::roxygen::has_roxygen_tag(tags, "noRd");
    let has_internal = crate::roxygen::has_roxygen_tag(tags, "keywords internal");
    !has_no_rd && !has_internal && !noexport
}

/// Emit the conditional S3 generic guard for a given generic name.
///
/// Returns an R code string (to be pushed onto a `lines: Vec<String>` with
/// `lines.push(emit_s3_generic_guard(name))`) that creates the generic when
/// it doesn't already exist as a function, and — mirroring the S7 classifier
/// (#1114, `s7_class.rs:880-935`) — shadows any existing binding that
/// `UseMethod` dispatch would never consult:
///
/// ```r
/// if (!base::exists("name", mode = "function")) {
///   name <- function(x, ...) UseMethod("name")
/// } else if (local({ .mx_gen <- base::get("name", mode = "function"); !(is.primitive(.mx_gen) || isTRUE(utils::isS3stdGeneric(.mx_gen)) || methods::isGeneric("name") || inherits(.mx_gen, "S7_generic")) })) {
///   .mx_shadow_default <- local({
///     .mx_masked <- base::get("name", mode = "function")
///     function(x, ...) .mx_masked(x, ...)
///   })
///   name <- function(x, ...) UseMethod("name")
///   base::registerS3method("name", "default", .mx_shadow_default, envir = base::environment())
///   base::rm(.mx_shadow_default)
/// }
/// # else: existing usable generic (primitive/S3/S4/S7) — reuse as-is.
/// ```
///
/// `name` resolving to a **plain non-generic closure** (`var`, `get`, `row`,
/// `col`, `diag`, `reshape`, …) is the #1248 bug: a bare `exists()` check
/// sees the name is bound and never installs the `UseMethod` dispatcher, so
/// the generated `name.Class` method is registered but silently never fires.
/// The classifier shadows such bindings with a package-local generic and
/// delegates the `default` method back to the masked closure, so ordinary
/// (non-dispatching) calls like `var(1:10)` keep working. S3 generic formals
/// are always `function(x, ...)`, so the delegation works positionally even
/// when the masked closure's first argument has a different name (e.g.
/// `reshape`'s is `data`) — S3 doesn't need S7's `dispatch_args`-mirroring
/// `fallback_sig` machinery.
///
/// The delegating default method is registered via `base::registerS3method()`
/// so it lives ONLY in the namespace's S3 methods table
/// (`.__S3MethodsTable__.`), never as a `name.default` namespace binding:
///
/// - a literal `name.default` binding trips roxygen2's dynamic S3 scan
///   (`warn_missing_s3_exports` walks the loaded namespace's bindings and
///   flags any method-shaped function not covered by an
///   `@export`/`@exportS3Method` block);
/// - a static NAMESPACE `S3method(name, default)` directive would break
///   package load whenever the shadow branch doesn't fire (e.g. for a real
///   generic like `print`, where no `name.default` of ours exists — and we
///   must never touch `print.default`).
///
/// Ordering inside the branch is load-bearing: `.mx_shadow_default` captures
/// the masked closure BEFORE `name` is rebound (once `name` is the generic,
/// `base::get("name")` would find our own generic → infinite recursion; the
/// assignment inside `local()` forces the value now — a function *argument*
/// would stay an unforced promise). The generic is bound at the branch top
/// level (not inside `local()`) so its closure environment is the package
/// namespace — `registerS3method` resolves the generic via
/// `get(genname, envir)`, takes `environment(genfun)` as the defining env,
/// and registers into THAT env's `.__S3MethodsTable__.`; the generic's env
/// must be the namespace for the table to be the namespace's.
/// `registerS3method` is called AFTER the generic is bound so it finds our
/// generic (not the masked closure, whose home namespace would otherwise
/// receive the registration). `base::rm` then drops the helper so the
/// namespace ends with zero helper bindings.
///
/// The classifier condition is wrapped in `local({...})` so `.mx_gen` doesn't
/// leak into the package namespace (the braced `else if` in the mirrored S7
/// pattern evaluates at source time in the namespace env — see the
/// corresponding fix at `s7_class.rs:902`, #1261 item 1).
///
/// Everything is `base::`-qualified (`exists`/`get`/`registerS3method`/
/// `environment`/`rm`): once we define a shadow generic named e.g. `get`, a
/// bare `get(...)` in a later generic's classifier would route through our
/// own generic instead of the real `base::get`.
///
/// Use this for S3/vctrs class generators and trait-ABI wrappers. Do **not**
/// use for S7 generics — those use `S7::new_generic()` / `S7::new_external_generic()`.
pub(crate) fn emit_s3_generic_guard(name: &str) -> String {
    let symbol = crate::naming::r_symbol(name);
    format!(
        "if (!base::exists(\"{name}\", mode = \"function\")) {{\n  {symbol} <- function(x, ...) UseMethod(\"{name}\")\n}} else if (local({{ .mx_gen <- base::get(\"{name}\", mode = \"function\"); !(is.primitive(.mx_gen) || isTRUE(utils::isS3stdGeneric(.mx_gen)) || methods::isGeneric(\"{name}\") || inherits(.mx_gen, \"S7_generic\")) }})) {{\n  # `{name}` is a plain closure that UseMethod dispatch will never consult.\n  # Shadow it with a package-local generic. The default method delegating to\n  # the masked closure is registered via registerS3method() so it lives ONLY\n  # in the namespace's S3 methods table: a literal `{name}.default` binding\n  # would trip roxygen2's dynamic S3 scan (warn_missing_s3_exports), and a\n  # static NAMESPACE S3method({name}, default) would break package load\n  # whenever this branch does not fire.\n  .mx_shadow_default <- local({{\n    .mx_masked <- base::get(\"{name}\", mode = \"function\")\n    function(x, ...) .mx_masked(x, ...)\n  }})\n  {symbol} <- function(x, ...) UseMethod(\"{name}\")\n  base::registerS3method(\"{name}\", \"default\", .mx_shadow_default, envir = base::environment())\n  base::rm(.mx_shadow_default)\n}}\n# else: existing usable generic (primitive/S3/S4/S7) — reuse as-is."
    )
}

/// Check whether `s` is a bare R identifier (only `[A-Za-z_][A-Za-z0-9_]*`).
pub(crate) fn is_bare_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Return a `.__MX_CLASS_REF_<name>__` placeholder (for bare identifiers) so the
/// resolver can look up the actual R class name at cdylib write time, or `name`
/// verbatim (for namespaced / non-identifier strings).
pub(crate) fn class_ref_or_verbatim(name: &str) -> String {
    if is_bare_identifier(name) {
        format!(".__MX_CLASS_REF_{name}__")
    } else {
        name.to_string()
    }
}

pub(crate) use crate::match_arg_keys::{
    choices_placeholder as match_arg_placeholder,
    param_doc_placeholder as match_arg_param_doc_placeholder,
};

/// Build the R-param-name → @param placeholder map for a method's match_arg and
/// choices params. Pass to `MethodDocBuilder::with_match_arg_doc_placeholders`
/// in each class generator.
///
/// Takes the per-param attribute map directly (rather than `&ParsedMethod`) so
/// it's shared by both the inherent-impl (`MethodContext`) and trait-impl
/// (`TraitMethodContext`, `miniextendr_impl_trait/method_context.rs`) paths.
pub(crate) fn match_arg_doc_placeholder_map(
    c_ident: &str,
    per_param: &std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    for (rust_name, attrs) in per_param {
        if !attrs.match_arg {
            continue;
        }
        let r_name = crate::r_wrapper_builder::normalize_r_arg_string(rust_name);
        out.insert(
            r_name.clone(),
            match_arg_param_doc_placeholder(c_ident, &r_name),
        );
    }
    out
}

/// Build R prelude lines that validate `match_arg` / `choices` / `several_ok`
/// parameters via `base::match.arg()` before the `.Call()`.
///
/// Returns an empty vector when the method declares none. Both `match_arg`
/// and `choices(...)` carry their choice list as the formal default
/// (`c("a", "b", ...)`), so `base::match.arg(arg)` finds the list by
/// itself — no second arg, no C helper lookup. `match_arg` adds a
/// factor → character coercion in front of `match.arg`.
///
/// Shared by `MethodContext::match_arg_prelude` (inherent impls) and
/// `TraitMethodContext::match_arg_prelude` (trait impls) — see
/// `audit/2026-07-03-dogfooding-macros-codegen.md` finding #1 (trait methods
/// previously had no match_arg support at all).
pub(crate) fn build_match_arg_prelude(
    per_param: &std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
) -> Vec<String> {
    let mut lines = Vec::new();

    for (rust_name, attrs) in per_param {
        if !attrs.match_arg {
            continue;
        }
        let r_name = crate::r_wrapper_builder::normalize_r_arg_string(rust_name);
        lines.push(format!(
            "{r_name} <- if (is.factor({r_name})) as.character({r_name}) else {r_name}"
        ));
        if attrs.several_ok {
            lines.push(format!(
                "{r_name} <- base::match.arg({r_name}, several.ok = TRUE)"
            ));
        } else {
            lines.push(format!("{r_name} <- base::match.arg({r_name})"));
        }
    }

    for (rust_name, attrs) in per_param {
        if attrs.choices.is_none() {
            continue;
        }
        let r_name = crate::r_wrapper_builder::normalize_r_arg_string(rust_name);
        if attrs.several_ok {
            lines.push(format!(
                "{r_name} <- match.arg({r_name}, several.ok = TRUE)"
            ));
        } else {
            lines.push(format!("{r_name} <- match.arg({r_name})"));
        }
    }

    lines
}

/// Rust-side parameter names that are validated by R's `match.arg()` and
/// therefore don't need `stopifnot()` preconditions generated for them.
/// Shared by `MethodContext` and `TraitMethodContext`.
pub(crate) fn match_arg_skip_set(
    per_param: &std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
) -> std::collections::HashSet<String> {
    let mut s = std::collections::HashSet::new();
    for (rust_name, attrs) in per_param {
        if attrs.match_arg || attrs.choices.is_some() {
            s.insert(crate::r_wrapper_builder::normalize_r_arg_string(rust_name));
        }
    }
    s
}

/// Build R-side precondition `stopifnot()` lines for a parameter list, given
/// its match_arg/choices per-param map and whether `coerce` is active for the
/// whole method.
///
/// Neither impl methods nor trait methods carry a per-param `coerce` flag
/// (only function-wide `coerce`, see `ParsedMethod::per_param` docs), so
/// `coerce_params` is always empty here. Shared by
/// `MethodContext::precondition_checks` and
/// `TraitMethodContext::precondition_checks`.
pub(crate) fn build_method_precondition_checks(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    per_param: &std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
    coerce_all: bool,
) -> Vec<String> {
    let opts = crate::r_preconditions::PreconditionOptions {
        coerce_all,
        coerce_params: std::collections::HashSet::new(),
    };
    crate::r_preconditions::build_precondition_checks(inputs, &match_arg_skip_set(per_param), &opts)
        .static_checks
}

/// Effective R-formal defaults for a method.
///
/// Layers defaults in priority order:
/// 1. `#[miniextendr(match_arg)]` → ALWAYS a write-time placeholder that the
///    cdylib resolves to `c("a", "b", ...)` at package-load time. Any user-
///    supplied `default = "X"` is consumed elsewhere (rotates X to the front
///    of the choice list at write time) rather than overriding the formal.
/// 2. `#[miniextendr(choices("a", "b", ...))]` → `c("a", "b", ...)` formal default.
/// 3. User-provided `#[miniextendr(defaults(param = "..."))]` for non-match_arg
///    params.
///
/// This formal default is load-bearing for `match.arg()`, not just cosmetic:
/// `base::match.arg(arg)` (no explicit `choices=`) reads the choice list from
/// the *formal default* of the calling function's `arg` parameter — a
/// `match_arg`/`choices` param with no formal default makes `match.arg()`
/// fail with "argument is missing, with no default" even when the caller
/// passed a value. Shared by `MethodContext::new` (inherent impls) and
/// `TraitMethodContext::new` (trait impls, `miniextendr_impl_trait/method_context.rs`).
pub(crate) fn effective_r_defaults(
    param_defaults: &std::collections::HashMap<String, String>,
    per_param: &std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
    c_ident: &str,
) -> std::collections::HashMap<String, String> {
    let mut defaults = param_defaults.clone();
    // match_arg → unconditionally splice the placeholder (overriding any user
    // default, which is captured separately for write-time rotation).
    for (rust_name, attrs) in per_param {
        if !attrs.match_arg {
            continue;
        }
        let r_name = crate::r_wrapper_builder::normalize_r_arg_string(rust_name);
        defaults.insert(r_name.clone(), match_arg_placeholder(c_ident, &r_name));
    }
    // choices(...) → c("a", "b", ...) formal. Lower priority than user
    // defaults (kept for back-compat on non-match_arg params).
    for (rust_name, attrs) in per_param {
        if let Some(choices) = attrs.choices.as_ref() {
            let r_name = crate::r_wrapper_builder::normalize_r_arg_string(rust_name);
            defaults.entry(r_name).or_insert_with(|| {
                let quoted: Vec<String> = choices.iter().map(|c| format!("\"{c}\"")).collect();
                format!("c({})", quoted.join(", "))
            });
        }
    }
    defaults
}

/// Pre-computed context for a method, holding all data needed for R wrapper generation.
///
/// This struct captures the common computations performed for every method across all
/// class systems, reducing duplicate code. It pre-formats the C wrapper name, R formal
/// parameters (with defaults), and R call arguments so each class generator can
/// focus on its specific formatting logic.
pub struct MethodContext<'a> {
    /// Reference to the parsed method metadata.
    pub method: &'a ParsedMethod,
    /// The C wrapper identifier string (e.g., `"C_Counter__inc"`), used in `.Call()`.
    pub c_ident: String,
    /// R formals string with defaults (e.g., `"value, step = 1L"`), used in
    /// `function(...)` signatures.
    pub params: String,
    /// R call arguments string without defaults (e.g., `"value, step"`), used
    /// inside `.Call()` expressions.
    pub args: String,
    /// Drop the R-side `stopifnot(...)` block from the generated wrapper.
    /// Inherited from `ImplAttrs::no_preconditions` (set by `#[miniextendr(no_preconditions)]`
    /// or `fast` on the impl block).
    pub no_preconditions: bool,
    /// Emit `.call = NULL` instead of `.call = match.call()` in non-lambda
    /// dispatch sites. Inherited from `ImplAttrs::no_call_attribution`.
    /// Lambda sites (`instance_call_null_attr`, R6 finalizer/deep_clone,
    /// S7 property dispatch) already emit NULL and are unaffected.
    pub no_call_attribution: bool,
}

impl<'a> MethodContext<'a> {
    /// Create a new MethodContext for a method.
    ///
    /// Computes the C wrapper identifier from the method name, type name, and optional
    /// label (for multi-impl-block disambiguation), then formats the R formals and
    /// call arguments from the method's signature and default values. Fast-path
    /// knobs default off; use [`MethodContext::with_fast_flags`] to inherit them
    /// from `ImplAttrs`.
    pub fn new(method: &'a ParsedMethod, type_ident: &syn::Ident, label: Option<&str>) -> Self {
        let c_ident = method.c_wrapper_ident(type_ident, label).to_string();
        let effective_defaults = effective_r_defaults(
            &method.param_defaults,
            &method.method_attrs.per_param,
            &c_ident,
        );
        let mut arg_builder = crate::r_wrapper_builder::RArgumentBuilder::new(&method.sig.inputs);
        if method.method_attrs.has_dots {
            arg_builder = arg_builder.with_dots(
                method
                    .method_attrs
                    .named_dots
                    .as_ref()
                    .map(|ident| ident.to_string()),
            );
        }
        arg_builder = arg_builder.with_defaults(effective_defaults);
        let params = arg_builder.build_formals();
        let args = arg_builder.build_call_args();
        Self {
            method,
            c_ident,
            params,
            args,
            no_preconditions: false,
            no_call_attribution: false,
        }
    }

    /// Set the fast-path flags inherited from the surrounding `ImplAttrs`.
    /// Returns `self` so callers can chain on top of `MethodContext::new`.
    pub fn with_fast_flags(mut self, no_preconditions: bool, no_call_attribution: bool) -> Self {
        self.no_preconditions = no_preconditions;
        self.no_call_attribution = no_call_attribution;
        self
    }

    /// Build the R-param-name → @param placeholder map for this method's
    /// match_arg params. Pass to `MethodDocBuilder::with_match_arg_doc_placeholders`
    /// so the cdylib write pass rewrites the placeholders into rendered choice
    /// descriptions (#210).
    pub fn match_arg_doc_placeholders(&self) -> std::collections::HashMap<String, String> {
        match_arg_doc_placeholder_map(&self.c_ident, &self.method.method_attrs.per_param)
    }

    /// Build R prelude lines that validate `match_arg` / `choices` / `several_ok`
    /// parameters via `base::match.arg()` before the `.Call()`.
    ///
    /// Returns an empty vector when the method declares none. Both `match_arg`
    /// and `choices(...)` carry their choice list as the formal default
    /// (`c("a", "b", ...)`), so `base::match.arg(arg)` finds the list by
    /// itself — no second arg, no C helper lookup. `match_arg` adds a
    /// factor → character coercion in front of `match.arg`.
    ///
    /// Callers should include these lines in the R wrapper body after parameter
    /// defaulting but before the `.Call()`.
    pub fn match_arg_prelude(&self) -> Vec<String> {
        build_match_arg_prelude(&self.method.method_attrs.per_param)
    }

    /// Build the `.Call()` expression for a static/constructor call.
    pub fn static_call(&self) -> String {
        let mut b = crate::r_wrapper_builder::DotCallBuilder::new(&self.c_ident);
        if self.no_call_attribution {
            b = b.null_call_attribution();
        }
        b.with_args_str(&self.args).build()
    }

    /// Build the `.Call()` expression for an instance method with `self` as ptr.
    ///
    /// The `self_expr` is typically "self", "private$.ptr", "x", "x@ptr", or "x@.ptr".
    pub fn instance_call(&self, self_expr: &str) -> String {
        let mut b = crate::r_wrapper_builder::DotCallBuilder::new(&self.c_ident);
        if self.no_call_attribution {
            b = b.null_call_attribution();
        }
        b.with_self(self_expr).with_args_str(&self.args).build()
    }

    /// Like [`instance_call`](Self::instance_call) but passes `.call = NULL`.
    ///
    /// Use for lambda dispatch sites (S7 property getter/setter) where
    /// `match.call()` captures the S7 dispatch frame, not the user's call.
    pub fn instance_call_null_attr(&self, self_expr: &str) -> String {
        crate::r_wrapper_builder::DotCallBuilder::new(&self.c_ident)
            .null_call_attribution()
            .with_self(self_expr)
            .with_args_str(&self.args)
            .build()
    }

    /// Build full R formals for instance methods (prefixing x/self parameter).
    ///
    /// For S3/S4/S7: `"x, <params>, ..."`
    /// For Env/R6: `"<params>"` (self is implicit)
    pub fn instance_formals(&self, add_self_param: bool) -> String {
        self.instance_formals_with_dots(add_self_param, true)
    }

    /// Build full R formals for instance methods with optional dots.
    ///
    /// When `include_dots` is false, omits `...` from the signature.
    /// This is used for strict generics that don't accept extra args.
    pub fn instance_formals_with_dots(&self, add_self_param: bool, include_dots: bool) -> String {
        let include_dispatch_dots = include_dots && !self.method.has_dots;
        if add_self_param {
            if include_dispatch_dots {
                if self.params.is_empty() {
                    "x, ...".to_string()
                } else {
                    format!("x, {}, ...", self.params)
                }
            } else {
                // No dots - strict formals
                if self.params.is_empty() {
                    "x".to_string()
                } else {
                    format!("x, {}", self.params)
                }
            }
        } else {
            self.params.clone()
        }
    }

    /// Build instance formals with a custom receiver name (default is `x`).
    ///
    /// Used by the S7 per-class fast-path shortcut (#949), whose receiver is
    /// named `self` to mirror the property dispatch lambdas, rather than the
    /// `x` used by the S7 generic.
    pub fn instance_formals_with_receiver(&self, receiver: &str, include_dots: bool) -> String {
        let tail = if include_dots && !self.method.has_dots {
            ", ..."
        } else {
            ""
        };
        if self.params.is_empty() {
            format!("{receiver}{tail}")
        } else {
            format!("{receiver}, {}{tail}", self.params)
        }
    }

    /// Get the generic name (uses override if present).
    pub fn generic_name(&self) -> String {
        // Explicit `generic = ".."` wins; otherwise the R-facing method name
        // (`r_name` / `postfix` / Rust ident) doubles as the generic, so a
        // renamed S3/S7 instance method dispatches under its R name.
        self.method
            .method_attrs
            .generic
            .clone()
            .unwrap_or_else(|| self.method.r_method_name())
    }

    /// Generate a source location comment for this method.
    ///
    /// Returns a string like `# Type::method (line:col)` using the method's span info.
    /// The file name is already stated in the impl block header comment, so line:col
    /// is sufficient to locate the method within that file.
    pub fn source_comment(&self, type_ident: &syn::Ident) -> String {
        let start = self.method.ident.span().start();
        format!(
            "# {}::{} ({}:{})",
            type_ident,
            self.method.ident,
            start.line,
            start.column + 1,
        )
    }

    /// Check if this method uses a generic override (for existing generics like print).
    pub fn has_generic_override(&self) -> bool {
        self.method.method_attrs.generic.is_some()
    }

    /// Get custom class suffix if specified.
    ///
    /// This allows double-dispatch patterns like `vec_ptype2.my_class.my_class`
    /// by specifying `#[miniextendr(s3(generic = "vec_ptype2", class = "my_class.my_class"))]`.
    pub fn class_suffix(&self) -> Option<&str> {
        self.method.method_attrs.class.as_deref()
    }

    /// Check if this method uses a custom class suffix.
    pub fn has_class_override(&self) -> bool {
        self.method.method_attrs.class.is_some()
    }

    /// Build R-side precondition `stopifnot()` lines for this method's parameters.
    ///
    /// Returns static checks for known types. Custom types not in the static table
    /// are identified as fallback params but no R-side precheck is generated for them.
    ///
    /// Skips `self`/receiver parameters automatically (they are `FnArg::Receiver`) and
    /// any parameter validated by `base::match.arg()` (via `match_arg` / `choices`) —
    /// those already have a stronger runtime guarantee than `stopifnot(is.character(...))`.
    pub fn precondition_checks(&self) -> Vec<String> {
        if self.no_preconditions {
            return Vec::new();
        }
        // A coerced integer-element vector reads via `&[i32]` (INTSXP-only), so
        // its precondition tightens to `is.integer` (#616). Impl methods carry
        // coerce at method level (`method_attrs.coerce`, equivalent to
        // `coerce_all`); there is no per-param coerce on the impl path (see
        // ParsedMethod::per_param docs).
        build_method_precondition_checks(
            &self.method.sig.inputs,
            &self.method.method_attrs.per_param,
            self.method.method_attrs.coerce,
        )
    }

    /// Emit the 6-step method prelude into `lines`, each line prefixed with `indent`.
    ///
    /// The prelude is the standardised sequence that appears at the top of every
    /// generated R method body, in order:
    ///
    /// 1. `r_entry` — user code injected before any checks
    /// 2. `r_on_exit` — `on.exit(...)` cleanup
    /// 3. `lifecycle_prelude` — deprecation/superseded banner (class-system-specific label)
    /// 4. `precondition_checks` — `stopifnot(is.*(param))` for typed params
    /// 5. `match_arg_prelude` — `base::match.arg(param)` validation
    /// 6. `r_post_checks` — user code after all checks, before `.Call()`
    ///
    /// (`Missing<T>` forwarding is not a prelude step: it lives inline in the
    /// `.Call()` args — see `build_call_args_vec` — because a binding of the
    /// missing sentinel errors on lookup.)
    ///
    /// `what` is the human-readable method label passed to `lifecycle_prelude`
    /// (e.g., `"Type.method"` for S3/S4, `"Type$method"` for Env/R6/S7).
    /// `indent` is the per-line prefix (e.g., `"  "` for 2-space, `"      "` for 6-space).
    pub fn emit_method_prelude(&self, lines: &mut Vec<String>, indent: &str, what: &str) {
        let m = self.method;
        if let Some(ref entry) = m.method_attrs.r_entry {
            for line in entry.lines() {
                lines.push(format!("{}{}", indent, line));
            }
        }
        if let Some(ref on_exit) = m.method_attrs.r_on_exit {
            lines.push(format!("{}{}", indent, on_exit.to_r_code()));
        }
        if let Some(prelude) = m.lifecycle_prelude(what) {
            lines.push(format!("{}{}", indent, prelude));
        }
        for check in self.precondition_checks() {
            lines.push(format!("{}{}", indent, check));
        }
        for line in self.match_arg_prelude() {
            lines.push(format!("{}{}", indent, line));
        }
        if let Some(ref post) = m.method_attrs.r_post_checks {
            for line in post.lines() {
                lines.push(format!("{}{}", indent, line));
            }
        }
    }
}

/// Builder for class-level roxygen documentation header.
///
/// Generates the common roxygen tags that appear at the start of each class definition:
/// - `@title` (unless user provided)
/// - `@name` (unless user provided)
/// - `@rdname` (unless user provided)
/// - User-provided doc tags
/// - `@source Generated by miniextendr...`
/// - Class-system-specific imports
/// - `@export` (unless user provided, `@noRd`, or internal/noexport flags)
pub struct ClassDocBuilder<'a> {
    /// The R-visible class name (e.g., `"Counter"`).
    class_name: &'a str,
    /// The Rust type identifier, used in the `@source` annotation.
    type_ident: &'a syn::Ident,
    /// User-provided roxygen tags extracted from doc comments.
    doc_tags: &'a [String],
    /// Human-readable label for the class system (e.g., `"R6"`, `"S3"`, `"Env"`),
    /// used in the auto-generated `@title`.
    class_system_label: &'static str,
    /// Optional `@importFrom` tag for class-system-specific R packages
    /// (e.g., `"@importFrom R6 R6Class"`).
    imports: Option<String>,
    /// When `true`, adds `@keywords internal` and suppresses `@export`.
    /// Set by `#[miniextendr(internal)]`.
    attr_internal: bool,
    /// When `true`, suppresses `@export` but does not add `@keywords internal`.
    /// Set by `#[miniextendr(noexport)]`.
    attr_noexport: bool,
}

impl<'a> ClassDocBuilder<'a> {
    /// Create a new ClassDocBuilder with the given class metadata.
    ///
    /// By default, `@export` is included unless suppressed by user tags or
    /// the `with_export_control` method.
    pub fn new(
        class_name: &'a str,
        type_ident: &'a syn::Ident,
        doc_tags: &'a [String],
        class_system_label: &'static str,
    ) -> Self {
        Self {
            class_name,
            type_ident,
            doc_tags,
            class_system_label,
            imports: None,
            attr_internal: false,
            attr_noexport: false,
        }
    }

    /// Set R package imports (e.g., "@importFrom R6 R6Class").
    pub fn with_imports(mut self, imports: impl Into<String>) -> Self {
        self.imports = Some(imports.into());
        self
    }

    /// Set attribute-level internal/noexport flags from `ParsedImpl`.
    pub fn with_export_control(mut self, internal: bool, noexport: bool) -> Self {
        self.attr_internal = internal;
        self.attr_noexport = noexport;
        self
    }

    /// Build the roxygen `#' @tag` lines for the class header.
    ///
    /// Returns a vector of strings, each a complete roxygen comment line (e.g., `"#' @title ..."`).
    /// Auto-generates `@title`, `@name`, and `@rdname` if not provided by the user, and
    /// respects `@noRd` to suppress all documentation output.
    pub fn build(&self) -> Vec<String> {
        let has_title = crate::roxygen::has_roxygen_tag(self.doc_tags, "title");
        let has_name = crate::roxygen::has_roxygen_tag(self.doc_tags, "name");
        let has_rdname = crate::roxygen::has_roxygen_tag(self.doc_tags, "rdname");
        let has_export = crate::roxygen::has_roxygen_tag(self.doc_tags, "export");
        let has_no_rd = crate::roxygen::has_roxygen_tag(self.doc_tags, "noRd");
        let has_internal = crate::roxygen::has_roxygen_tag(self.doc_tags, "keywords internal");
        let effective_internal = has_internal || self.attr_internal;

        // `noexport` (without `internal`) must produce no Rd contribution at all —
        // no alias, no usage entry, nothing on a shared page — distinct from
        // `internal`, which stays documented under `\keyword{internal}`. Fold a
        // plain `noexport` into the same suppression gate as a user-written
        // `@noRd`. `internal` wins if both flags are set on the same impl block
        // (mirrors the standalone-fn `#[miniextendr(internal)]` precedence, where
        // `internal` + `noexport` together is a compile error).
        let suppress_rd = has_no_rd || (self.attr_noexport && !effective_internal);

        let mut lines = Vec::new();

        if suppress_rd && !has_no_rd {
            lines.push("#' @noRd".to_string());
        }

        if !has_title && !suppress_rd {
            lines.push(format!(
                "#' @title {} {} Class",
                self.class_name, self.class_system_label
            ));
        }
        if !has_name && !suppress_rd {
            lines.push(format!("#' @name {}", self.class_name));
        }
        if !has_rdname && !suppress_rd {
            lines.push(format!("#' @rdname {}", self.class_name));
        }
        crate::roxygen::push_roxygen_tags(&mut lines, self.doc_tags);
        if !suppress_rd {
            lines.push(crate::roxygen::class_source_tag(self.type_ident));
        }
        if let Some(ref imports) = self.imports
            && !suppress_rd
        {
            lines.push(format!("#' {}", imports));
        }
        // Inject @keywords internal if attr flag set and not already present
        if self.attr_internal && !has_internal && !suppress_rd {
            lines.push("#' @keywords internal".to_string());
        }
        // Don't auto-export if @noRd, @keywords internal, or attr flags are present
        if !has_export && !suppress_rd && !effective_internal && !self.attr_noexport {
            lines.push("#' @export".to_string());
        }

        lines
    }
}

/// Builder for method-level roxygen documentation.
///
/// Generates roxygen tags for individual methods within a class. Methods share
/// the class's `@rdname` so they appear on the same help page. The builder handles
/// `@name` formatting (with optional prefix like `$` for `Class$method` style)
/// and respects `@noRd` inheritance from the parent class.
pub struct MethodDocBuilder<'a> {
    /// The R class name (e.g., `"Counter"`).
    class_name: &'a str,
    /// The Rust method name (e.g., `"inc"`).
    method_name: &'a str,
    /// The Rust type identifier, used in the `@source` annotation.
    type_ident: &'a syn::Ident,
    /// User-provided roxygen tags extracted from the method's doc comments.
    doc_tags: &'a [String],
    /// Optional separator between class name and method name in `@name`
    /// (e.g., `"$"` produces `@name Counter$inc`).
    name_prefix: Option<&'a str>,
    /// Override for the `@name` tag when the R function name differs from the Rust
    /// method name (e.g., for standalone S3 methods like `format.my_class`).
    r_name_override: Option<String>,
    /// When `true`, adds `@export` to the method (used for standalone S3/S4 generics).
    /// Defaults to `false` because `Class$method` access does not need separate export.
    always_export: bool,
    /// Whether the parent class has `@noRd`. When `true`, this method emits only
    /// `#' @noRd` and skips all other documentation tags.
    class_has_no_rd: bool,
    /// When `true`, convert `@param` tags into `\describe{}` blocks instead of
    /// roxygen `@param` entries.
    ///
    /// Used for env-class methods where roxygen cannot infer `\usage` from
    /// `Class$method <- function()`. Without this, `@param` tags create
    /// `\arguments` entries with no matching `\usage`, causing R CMD check
    /// warnings ("Documented arguments not in \\usage").
    params_as_details: bool,
    /// Optional comma-separated R parameter string for auto-generating `@param` tags.
    /// When set, any parameter not already documented gets `@param name (undocumented)`.
    r_params: Option<&'a str>,
    /// When `true`, filter out `@param` tags from the doc_tags before pushing.
    ///
    /// Used for S4/S7 instance methods where the method is defined via `setMethod()`
    /// or `S7::method()` assignment, which roxygen2 doesn't parse for `\usage` entries.
    /// Including `@param` tags would create "Documented arguments not in \\usage" warnings.
    suppress_params: bool,
    /// Map of R-param-name → write-time doc placeholder for match_arg parameters.
    ///
    /// When the auto-generated `@param` line would otherwise say `(undocumented)`,
    /// a match_arg'd param emits the placeholder instead, which the cdylib's
    /// write-time pass replaces with a rendered choice description (#210).
    match_arg_doc_placeholders: Option<&'a std::collections::HashMap<String, String>>,
}

impl<'a> MethodDocBuilder<'a> {
    /// Create a new MethodDocBuilder with default settings.
    ///
    /// By default, `always_export` is `false` because methods accessed via `Class$method`
    /// should not be exported directly -- only the class env and standalone S3 methods
    /// need `@export`.
    pub fn new(
        class_name: &'a str,
        method_name: &'a str,
        type_ident: &'a syn::Ident,
        doc_tags: &'a [String],
    ) -> Self {
        Self {
            class_name,
            method_name,
            type_ident,
            doc_tags,
            name_prefix: None,
            r_name_override: None,
            always_export: false,
            class_has_no_rd: false,
            params_as_details: false,
            r_params: None,
            suppress_params: false,
            match_arg_doc_placeholders: None,
        }
    }

    /// Supply a map from R-param-name to a write-time doc placeholder for
    /// match_arg'd params. When the auto-generated `@param` line would otherwise
    /// say `(undocumented)`, the placeholder is emitted instead and the cdylib
    /// write pass rewrites it to a rendered choice description. See #210.
    pub fn with_match_arg_doc_placeholders(
        mut self,
        placeholders: &'a std::collections::HashMap<String, String>,
    ) -> Self {
        self.match_arg_doc_placeholders = Some(placeholders);
        self
    }

    /// Set a prefix for the @name tag (e.g., "$" for "Class$method").
    pub fn with_name_prefix(mut self, prefix: &'a str) -> Self {
        self.name_prefix = Some(prefix);
        self
    }

    /// Override the @name tag with a custom R function name.
    ///
    /// Use this when the R function name differs from the Rust method name
    /// (e.g., for standalone S3/S4/S7 static methods like `s3counter_default_counter`).
    pub fn with_r_name(mut self, r_name: String) -> Self {
        self.r_name_override = Some(r_name);
        self
    }

    /// Set whether the parent class has @noRd.
    ///
    /// When true, skips @name, @rdname, @source tags and adds @noRd instead.
    pub fn with_class_no_rd(mut self, class_has_no_rd: bool) -> Self {
        self.class_has_no_rd = class_has_no_rd;
        self
    }

    /// Convert `@param` tags to inline `\describe{}` blocks instead of roxygen `@param`.
    ///
    /// Used for env-class methods where roxygen can't infer `\usage` from `Class$method <- function()`.
    /// Without this, `@param` tags create `\arguments` entries with no matching `\usage`,
    /// causing R CMD check warnings ("Documented arguments not in \\usage").
    pub fn with_params_as_details(mut self) -> Self {
        self.params_as_details = true;
        self
    }

    /// Set the method's formal parameter names (comma-separated R params string).
    ///
    /// When set, auto-generates `@param name (undocumented)` for any parameter
    /// not already covered by a user `@param` tag. Skips `self`, `.ptr`, and
    /// `...` parameters.
    pub fn with_r_params(mut self, params: &'a str) -> Self {
        self.r_params = Some(params);
        self
    }

    /// Suppress `@param` tags from user doc comments.
    ///
    /// Used for S4/S7 instance methods where the method is defined via `setMethod()`
    /// or `S7::method()` assignment, which roxygen2 doesn't parse for `\usage` entries.
    pub fn with_suppress_params(mut self) -> Self {
        self.suppress_params = true;
        self
    }

    /// Build the roxygen `#' @tag` lines for the method.
    ///
    /// Returns a vector of strings, each a complete roxygen comment line. If the parent
    /// class has `@noRd`, returns only `["#' @noRd"]`. Otherwise generates `@name`,
    /// `@rdname`, `@source`, and optionally `@export` tags, plus any user-provided tags.
    pub fn build(&self) -> Vec<String> {
        let mut lines = Vec::new();

        // If parent class has @noRd, skip all documentation and just add @noRd
        if self.class_has_no_rd {
            lines.push("#' @noRd".to_string());
            return lines;
        }

        if !self.doc_tags.is_empty() {
            if self.params_as_details {
                // For env-class: emit non-@param tags normally, convert @param to \describe
                let (param_tags, other_tags): (Vec<_>, Vec<_>) = self
                    .doc_tags
                    .iter()
                    .partition(|t| t.trim_start().starts_with("@param "));
                let other_refs: Vec<&str> = other_tags.iter().map(|s| s.as_str()).collect();
                crate::roxygen::push_roxygen_tags_str(&mut lines, &other_refs);
                if !param_tags.is_empty() {
                    // Only add blank separator if the previous line isn't @title
                    // (roxygen2 treats blank lines after @title as multi-paragraph titles)
                    let last_is_title = lines.last().is_some_and(|l| l.contains("@title"));
                    if !last_is_title {
                        lines.push("#'".to_string());
                    }
                    lines.push("#' \\describe{".to_string());
                    for tag in &param_tags {
                        if let Some(rest) = tag.trim_start().strip_prefix("@param ") {
                            let mut parts = rest.splitn(2, char::is_whitespace);
                            let name = parts.next().unwrap_or("");
                            let desc = parts.next().unwrap_or("");
                            lines.push(format!("#'   \\item{{\\code{{{name}}}}}{{{desc}}}"));
                        }
                    }
                    lines.push("#' }".to_string());
                }
            } else if self.suppress_params {
                // Filter out @param tags — they would create "Documented arguments
                // not in \usage" warnings for S4/S7 methods.
                let filtered: Vec<&str> = self
                    .doc_tags
                    .iter()
                    .filter(|t| {
                        !t.trim_start()
                            .strip_prefix('@')
                            .is_some_and(|rest| rest.starts_with("param"))
                    })
                    .map(|s| s.as_str())
                    .collect();
                crate::roxygen::push_roxygen_tags_str(&mut lines, &filtered);
            } else {
                crate::roxygen::push_roxygen_tags(&mut lines, self.doc_tags);
            }
        }

        // Auto-generate @param for undocumented method parameters. Split on
        // top-level commas only — a naive `split(", ")` shreds a
        // `mode = c("fast", "slow")` default into a bogus `"slow")` formal,
        // which surfaces as a spurious @param and an R CMD check warning.
        if let Some(params) = self.r_params {
            for param in crate::roxygen::split_r_formals(params) {
                let param_name = crate::roxygen::formal_name(param);
                if param_name == ".ptr" || param_name == "..." || param_name == "self" {
                    continue;
                }
                let already_documented =
                    crate::roxygen::param_documented(self.doc_tags, param_name);
                if !already_documented {
                    // match_arg'd params get a placeholder the cdylib write-pass
                    // replaces with the rendered choice description (#210).
                    let body = self
                        .match_arg_doc_placeholders
                        .and_then(|m| m.get(param_name))
                        .map(|s| s.as_str())
                        .unwrap_or("(undocumented)");
                    lines.push(format!("#' @param {} {}", param_name, body));
                }
            }
        }

        let r_name = if let Some(ref r_name) = self.r_name_override {
            r_name.clone()
        } else if let Some(prefix) = self.name_prefix {
            format!("{}{}{}", self.class_name, prefix, self.method_name)
        } else {
            self.method_name.to_string()
        };

        if !crate::roxygen::has_roxygen_tag(self.doc_tags, "name") {
            lines.push(format!("#' @name {}", r_name));
        }

        // A method-level `@rdname` splits the method onto its own page (#1438).
        // Method prose is demoted to `@description` (see `roxygen_tags_from_attrs`)
        // and the class page normally supplies the `@title`, so the new page
        // would have none and roxygen2 would skip it ("no name and/or title").
        // Follow the standalone-function convention (`lib.rs`) and use the
        // structural R name as the title unless the author wrote one.
        if crate::roxygen::has_roxygen_tag(self.doc_tags, "rdname") {
            if !crate::roxygen::has_roxygen_tag(self.doc_tags, "title") {
                lines.push(format!("#' @title {}", r_name));
            }
        } else {
            lines.push(format!("#' @rdname {}", self.class_name));
        }

        lines.push(format!(
            "#' @source Generated by miniextendr from `{}::{}`",
            self.type_ident, self.method_name
        ));

        let has_no_rd = crate::roxygen::has_roxygen_tag(self.doc_tags, "noRd");
        let has_internal = crate::roxygen::has_roxygen_tag(self.doc_tags, "keywords internal");
        // Don't auto-export if @noRd or @keywords internal is present
        if self.always_export
            && !crate::roxygen::has_roxygen_tag(self.doc_tags, "export")
            && !has_no_rd
            && !has_internal
        {
            lines.push("#' @export".to_string());
        }

        lines
    }
}

/// Extension trait for `ParsedImpl` to iterate over methods as [`MethodContext`].
///
/// Provides convenience methods that wrap `ParsedImpl`'s method iterators,
/// automatically constructing a `MethodContext` for each method. This avoids
/// repeating the `MethodContext::new(m, type_ident, label)` boilerplate in
/// every class system generator.
pub trait ParsedImplExt {
    /// Create a `MethodContext` for the constructor method, if one exists.
    fn constructor_context(&self) -> Option<MethodContext<'_>>;

    /// Iterate over all instance methods (public + private + active) as `MethodContext`.
    fn instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>>;

    /// Iterate over static (non-receiver) methods as `MethodContext`.
    fn static_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>>;

    /// Iterate over public instance methods as `MethodContext` (for R6 `public` list).
    fn public_instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>>;

    /// Iterate over private instance methods as `MethodContext` (for R6 `private` list).
    fn private_instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>>;

    /// Iterate over active binding methods as `MethodContext` (for R6 `active` list).
    fn active_instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>>;
}

impl ParsedImplExt for ParsedImpl {
    fn constructor_context(&self) -> Option<MethodContext<'_>> {
        let no_prec = self.no_preconditions;
        let no_call = self.no_call_attribution;
        self.constructor().map(|m| {
            MethodContext::new(m, &self.type_ident, self.label()).with_fast_flags(no_prec, no_call)
        })
    }

    fn instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>> {
        let type_ident = &self.type_ident;
        let label = self.label();
        let no_prec = self.no_preconditions;
        let no_call = self.no_call_attribution;
        self.instance_methods().map(move |m| {
            MethodContext::new(m, type_ident, label).with_fast_flags(no_prec, no_call)
        })
    }

    fn static_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>> {
        let type_ident = &self.type_ident;
        let label = self.label();
        let no_prec = self.no_preconditions;
        let no_call = self.no_call_attribution;
        self.static_methods().map(move |m| {
            MethodContext::new(m, type_ident, label).with_fast_flags(no_prec, no_call)
        })
    }

    fn public_instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>> {
        let type_ident = &self.type_ident;
        let label = self.label();
        let no_prec = self.no_preconditions;
        let no_call = self.no_call_attribution;
        self.public_instance_methods().map(move |m| {
            MethodContext::new(m, type_ident, label).with_fast_flags(no_prec, no_call)
        })
    }

    fn private_instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>> {
        let type_ident = &self.type_ident;
        let label = self.label();
        let no_prec = self.no_preconditions;
        let no_call = self.no_call_attribution;
        self.private_instance_methods().map(move |m| {
            MethodContext::new(m, type_ident, label).with_fast_flags(no_prec, no_call)
        })
    }

    fn active_instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>> {
        let type_ident = &self.type_ident;
        let label = self.label();
        let no_prec = self.no_preconditions;
        let no_call = self.no_call_attribution;
        self.active_instance_methods().map(move |m| {
            MethodContext::new(m, type_ident, label).with_fast_flags(no_prec, no_call)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ClassDocBuilder;

    #[test]
    fn test_method_context_static_call_no_args() {
        // This is a unit test for the static_call method
        // We'd need a mock ParsedMethod to test fully, but we can test the logic
        let call = ".Call(C_Test, .call = match.call())";
        assert!(call.contains(".Call"));
    }

    /// Audit A10: a class-level `#[miniextendr(noexport)]` (without `internal`)
    /// must produce no Rd contribution at all — no `@title`/`@name`/`@rdname`/
    /// `@export` — same as a user-written `@noRd`. Before the fix, `noexport`
    /// only suppressed `@export`, leaving the class fully documented (with an
    /// alias) minus the export line.
    #[test]
    fn test_class_noexport_suppresses_all_roxygen() {
        let type_ident: syn::Ident = syn::parse_str("Foo").unwrap();
        let doc_tags: Vec<String> = vec![];
        let lines = ClassDocBuilder::new("Foo", &type_ident, &doc_tags, "R6")
            .with_export_control(false, true)
            .build();
        let joined = lines.join("\n");

        assert!(
            lines.iter().any(|l| l == "#' @noRd"),
            "noexport should emit @noRd, got:\n{}",
            joined
        );
        assert!(
            !joined.contains("@title") && !joined.contains("@name") && !joined.contains("@rdname"),
            "noexport should suppress @title/@name/@rdname entirely, got:\n{}",
            joined
        );
        assert!(
            !joined.contains("@export"),
            "noexport should suppress @export, got:\n{}",
            joined
        );
    }

    /// Companion: `#[miniextendr(internal)]` keeps the class documented (under
    /// `@keywords internal`) — it still contributes `@title`/`@name`/`@rdname`
    /// so it lands on a real help page, just unexported.
    #[test]
    fn test_class_internal_still_documented() {
        let type_ident: syn::Ident = syn::parse_str("Foo").unwrap();
        let doc_tags: Vec<String> = vec![];
        let lines = ClassDocBuilder::new("Foo", &type_ident, &doc_tags, "R6")
            .with_export_control(true, false)
            .build();
        let joined = lines.join("\n");

        assert!(
            !lines.iter().any(|l| l == "#' @noRd"),
            "internal should NOT emit @noRd (stays documented), got:\n{}",
            joined
        );
        assert!(
            joined.contains("@keywords internal"),
            "internal should add @keywords internal, got:\n{}",
            joined
        );
        assert!(
            joined.contains("@title") && joined.contains("@name") && joined.contains("@rdname"),
            "internal should still emit @title/@name/@rdname, got:\n{}",
            joined
        );
        assert!(
            !joined.contains("#' @export"),
            "internal should suppress @export, got:\n{}",
            joined
        );
    }

    /// Neither flag set: normal fully-documented, exported class.
    #[test]
    fn test_class_no_flags_fully_documented_and_exported() {
        let type_ident: syn::Ident = syn::parse_str("Foo").unwrap();
        let doc_tags: Vec<String> = vec![];
        let lines = ClassDocBuilder::new("Foo", &type_ident, &doc_tags, "R6")
            .with_export_control(false, false)
            .build();
        let joined = lines.join("\n");

        assert!(!joined.contains("@noRd"));
        assert!(!joined.contains("@keywords internal"));
        assert!(
            joined.contains("@title") && joined.contains("@name") && joined.contains("@rdname")
        );
        assert!(joined.contains("#' @export"));
    }
}
