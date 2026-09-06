//! Shared utilities for building R wrapper code.
//!
//! This module provides builders for constructing R function signatures and call arguments
//! consistently across both standalone functions and impl methods.
//!
//! ## Key Components
//!
//! - [`RArgumentBuilder`]: Builds R formals and `.Call()` arguments from Rust signatures
//! - [`DotCallBuilder`]: Formats `.Call()` invocations with proper argument handling
//! - [`RoxygenBuilder`]: Generates roxygen2 documentation tags
//!
//! ## Usage
//!
//! ```ignore
//! // Build R function signature
//! let formals = build_r_formals_from_sig(&method.sig, &defaults);
//! let call_args = build_r_call_args_from_sig(&method.sig);
//!
//! // Build .Call() invocation
//! let call = DotCallBuilder::new("C_MyType__method")
//!     .with_self("self")
//!     .with_args(&["x", "y"])
//!     .build();
//!
//! // Build roxygen tags
//! let tags = RoxygenBuilder::new("MyType")
//!     .name("method")
//!     .rdname("MyType")
//!     .export()
//!     .build();
//! ```

/// Normalizes Rust argument identifiers for R.
///
/// - Leading `_` → stripped (Rust convention for unused params)
/// - Leading `__` → stripped
/// - Otherwise → unchanged
///
/// # Examples
/// - `_x` → `x`
/// - `_to` → `to`
/// - `__field` → `field`
/// - `value` → `value`
///
/// Note: We strip underscores rather than prefixing "unused" because R callers
/// (like vctrs) may use named arguments that must match the original name.
pub fn normalize_r_arg_ident(rust_ident: &syn::Ident) -> syn::Ident {
    syn::Ident::new(
        &normalize_r_arg_string(&crate::naming::ident_name(rust_ident)),
        rust_ident.span(),
    )
}

/// String form of [`normalize_r_arg_ident`] that skips the `syn::Ident` round-trip.
///
/// Most callers feed the result into `format!`/`HashMap` keys and immediately
/// `.to_string()` the returned ident — this avoids that allocation pair.
pub fn normalize_r_arg_string(name: &str) -> String {
    let normalized = name.trim_start_matches('_');
    if normalized.is_empty() {
        "arg".to_string()
    } else {
        normalized.to_string()
    }
}

/// Split a comma-separated choices list (as given to `choices(param = "a, b, c")`)
/// into individual trimmed entries. Surrounding double-quotes are tolerated so
/// users can spell the list either way: `"a, b"` or `"\"a\", \"b\""`.
///
/// Shared by the inherent-impl (`miniextendr_impl.rs`) and trait-impl
/// (`miniextendr_impl_trait/vtable.rs`) `choices(...)` attribute parsers so the
/// two independently-maintained parsers can't drift on quoting/whitespace rules.
pub(crate) fn split_choice_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Builder for R function formal parameters and call arguments.
///
/// Handles:
/// - Underscore normalization (`_x` → `unused_x`)
/// - Unit type defaults (`()` → `= NULL`)
/// - Dots (`...`) with optional naming
/// - Consistent formatting across function and method wrappers
pub struct RArgumentBuilder<'a> {
    /// The function's input parameters from the parsed Rust signature.
    inputs: &'a syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    /// If true, last parameter is treated as dots (`...`).
    has_dots: bool,
    /// Optional named binding for dots (e.g., `args: ...` in Rust becomes a named dots param).
    /// The name is normalized (leading underscores stripped) but only used on the Rust side;
    /// R formals always emit plain `...`.
    named_dots: Option<String>,
    /// If true, skip the first parameter (used for `self`/`&self` in method wrappers,
    /// since the self argument is handled separately by [`DotCallBuilder::with_self`]).
    skip_first: bool,
    /// Parameter default values from `#[miniextendr(default = "...")]` attributes.
    /// Keys are normalized R parameter names, values are R expressions emitted verbatim
    /// (e.g., `"1L"`, `"c(1, 2, 3)"`, `"NULL"`).
    defaults: std::collections::HashMap<String, String>,
}

impl<'a> RArgumentBuilder<'a> {
    /// Create a new builder for the given function inputs.
    pub fn new(inputs: &'a syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> Self {
        let named_dots = crate::miniextendr_fn::trailing_dots_ident(inputs)
            .map(|ident| normalize_r_arg_ident(&ident).to_string());
        Self {
            inputs,
            has_dots: named_dots.is_some(),
            named_dots,
            skip_first: false,
            defaults: std::collections::HashMap::new(),
        }
    }

    /// Add parameter defaults from `#[miniextendr(default = "...")]` attributes.
    ///
    /// Keys are normalized R parameter names (after underscore stripping),
    /// values are R expression strings emitted verbatim into formals.
    pub fn with_defaults(mut self, defaults: std::collections::HashMap<String, String>) -> Self {
        self.defaults = defaults;
        self
    }

    /// Mark the last parameter as dots (`...`).
    ///
    /// If `named_dots` is `Some("name")`, the dots have a Rust-side binding
    /// (from `name: ...` syntax). The name is normalized but only affects the
    /// Rust side -- R formals always emit plain `...`.
    pub fn with_dots(mut self, named_dots: Option<String>) -> Self {
        self.has_dots = true;
        self.named_dots = named_dots.map(|s| normalize_r_arg_string(&s));
        self
    }

    /// Skip the first parameter (for instance methods with `self`).
    pub fn skip_first(mut self) -> Self {
        self.skip_first = true;
        self
    }

    /// Build R formal parameters string (for function signature).
    ///
    /// # Returns
    /// Comma-separated parameter list, e.g., `"x, y = NULL, ..."`
    ///
    /// This method handles R-style defaults (like `1L`, `c(1,2,3)`) that aren't
    /// valid Rust syntax by outputting them directly as strings.
    pub fn build_formals(&self) -> String {
        let mut formals = Vec::new();
        let last_idx = self.inputs.len().saturating_sub(1);

        for (idx, input) in self.inputs.iter().enumerate() {
            // Skip first if requested (for self in methods)
            if self.skip_first && idx == 0 {
                continue;
            }

            let pat_type = match input {
                syn::FnArg::Typed(pt) => pt,
                syn::FnArg::Receiver(_) => continue, // Skip self receivers
            };

            // Handle dots (must be last)
            // Note: In R, `...` cannot have a name/default in formals - it must be just `...`
            // The named_dots is only used on the Rust side. R formals always use plain `...`
            if self.has_dots && idx == last_idx {
                formals.push("...".to_string());
                continue;
            }

            // Extract and normalize argument name
            let arg_ident = match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => normalize_r_arg_ident(&pat_ident.ident),
                _ => continue,
            };

            // Check for user-specified default value
            if let Some(default_val) = self.defaults.get(&arg_ident.to_string()) {
                // User provided default via #[miniextendr(default = "...")]
                // Output directly as string - supports R-style defaults like "1L", "c(1,2,3)"
                formals.push(format!("{} = {}", arg_ident, default_val));
                continue;
            }

            // Add default for unit types
            match pat_type.ty.as_ref() {
                syn::Type::Tuple(t) if t.elems.is_empty() => {
                    formals.push(format!("{} = NULL", arg_ident));
                }
                _ => {
                    formals.push(arg_ident.to_string());
                }
            }
        }

        formals.join(", ")
    }

    /// Build R call arguments string (for `.Call()` invocation).
    ///
    /// # Returns
    /// Comma-separated argument list, e.g., `"x, y, list(...)"`
    pub fn build_call_args(&self) -> String {
        self.build_call_args_vec().join(", ")
    }

    /// Build R call arguments as a `Vec<String>`.
    ///
    /// Each element is a single argument expression. Dots parameters become
    /// `"list(...)"` to capture variadic args as an R list for the `.Call()` interface.
    pub fn build_call_args_vec(&self) -> Vec<String> {
        let mut call_args = Vec::new();
        let last_idx = self.inputs.len().saturating_sub(1);

        for (idx, input) in self.inputs.iter().enumerate() {
            // Skip first if requested (for self in methods)
            if self.skip_first && idx == 0 {
                continue;
            }

            let syn::FnArg::Typed(pat_type) = input else {
                continue;
            };

            // Handle dots special case
            // Always use list(...) since R formals always have plain `...`
            if self.has_dots && idx == last_idx {
                call_args.push("list(...)".to_string());
                continue;
            }

            // Extract and normalize argument name
            let arg_ident = match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => normalize_r_arg_ident(&pat_ident.ident),
                _ => continue,
            };

            // `Missing<T>`: forward true missingness as the `R_MissingArg`
            // sentinel, produced *at the argument position*. A binding holding
            // the sentinel errors on symbol lookup ("argument is missing, with
            // no default"), so the former `if (missing(x)) x <- quote(expr=)`
            // prelude broke every truly-missing call. (`Missing<T>` + user
            // default is rejected at macro parse time, so no default-shadowing
            // concern here.)
            if is_missing_type(pat_type.ty.as_ref()) {
                call_args.push(format!(
                    "if (missing({p})) quote(expr=) else {p}",
                    p = arg_ident
                ));
                continue;
            }

            call_args.push(arg_ident.to_string());
        }

        call_args
    }
}

/// Build R formal parameters from a Rust function signature, with optional defaults.
///
/// Automatically skips `self`/`&self` receivers. `Missing<T>` parameters without
/// user-provided defaults appear as bare formals (no default value); the
/// `R_MissingArg` sentinel forwarding is emitted inline in the `.Call()` args
/// (see [`RArgumentBuilder::build_call_args_vec`]).
///
/// Returns a comma-separated string of R formals, e.g., `"x, y = NULL, ..."`.
pub(crate) fn build_r_formals_from_sig(
    sig: &syn::Signature,
    defaults: &std::collections::HashMap<String, String>,
) -> String {
    let mut builder = RArgumentBuilder::new(&sig.inputs);
    if matches!(sig.inputs.first(), Some(syn::FnArg::Receiver(_))) {
        builder = builder.skip_first();
    }
    builder = builder.with_defaults(defaults.clone());
    builder.build_formals()
}

/// Build R `.Call()` arguments from a Rust function signature.
///
/// Automatically skips `self`/`&self` receivers (those are passed separately
/// via [`DotCallBuilder::with_self`]). Dots become `list(...)`.
///
/// Returns a comma-separated string of R call arguments, e.g., `"x, y, list(...)"`.
pub(crate) fn build_r_call_args_from_sig(sig: &syn::Signature) -> String {
    let mut builder = RArgumentBuilder::new(&sig.inputs);
    if matches!(sig.inputs.first(), Some(syn::FnArg::Receiver(_))) {
        builder = builder.skip_first();
    }
    builder.build_call_args()
}

// region: Missing<T> detection for automatic defaults

/// Check if a type is `Missing<T>` by examining the last path segment.
///
/// `Missing<T>` is the miniextendr wrapper for R's "missing argument" concept,
/// allowing Rust functions to accept optional arguments that R callers can omit.
pub(crate) fn is_missing_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|s| s.ident == "Missing")
            .unwrap_or(false),
        _ => false,
    }
}

// endregion

// region: DotCallBuilder - .Call() invocation formatting

/// Which frame a generated wrapper hands to `.Call(.., .call = ..)` and uses as
/// the raise fallback (`.miniextendr_raise_condition(.val, <default>)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CallAttribution {
    /// `.call = match.call()`: the wrapper's own call, formals matched (default).
    #[default]
    Wrapper,
    /// `.call = .mx_call`, where the wrapper body first binds
    /// `.mx_parent <- sys.parent()`, the parent frame's function (`.mx_def`)
    /// and call (`.mx_pc`), then
    /// `.mx_call <- match.call(.mx_def, .mx_pc, envir = parent.frame(2L))`:
    /// the caller's call with the caller's formals matched. `envir` is the
    /// frame the caller's call was evaluated in (the caller's caller, two
    /// frames up from the wrapper), which is where a literal `...` in that
    /// call is bound: a `function(...)` helper forwarding into the caller, or
    /// `lapply()`'s `FUN(X[[i]], ...)`. `match.call`'s own default,
    /// `parent.frame(2L)` evaluated inside `match.call`, is the caller's frame,
    /// which has no `...`, so every call through such a helper failed with
    /// `... used in a situation where it does not exist` (#1462). It falls back to
    /// the wrapper's own `match.call()` when there is no parent frame (top
    /// level) or the parent frame is not a closure: `eval()`'d code such as a
    /// testthat block or `source()` has the `eval` primitive as its frame
    /// function, and `match.call()` rejects a non-closure definition. For a
    /// `noexport` entry point behind a hand-written R function
    /// (`#[miniextendr(noexport, call = caller)]`). Every `sys.*` lookup is a
    /// plain statement in the wrapper's own frame, never a promise forced by
    /// `match.call` (where `sys.call(0)` would resolve to `match.call`'s frame).
    Caller,
    /// `.call = NULL`: `no_call_attribution` / `fast`; the raise helper falls
    /// back to the wrapper's `sys.call()`.
    None,
}

impl CallAttribution {
    /// The `.call = ...` argument for the `.Call()` line.
    pub fn dot_call_arg(self) -> &'static str {
        match self {
            CallAttribution::Wrapper => ".call = match.call()",
            CallAttribution::Caller => ".call = .mx_call",
            CallAttribution::None => ".call = NULL",
        }
    }

    /// The fallback call handed to `.miniextendr_raise_condition`.
    pub fn raise_default(self) -> &'static str {
        match self {
            CallAttribution::Caller => ".mx_call",
            CallAttribution::Wrapper | CallAttribution::None => "sys.call()",
        }
    }

    /// Statements the wrapper body needs before the `.Call()` line: empty except
    /// for [`CallAttribution::Caller`], which binds `.mx_call`. Each line ends
    /// with a newline plus `indent`, so the result can be prepended to a body
    /// whose first line is already positioned.
    pub fn prelude(self, indent: &str) -> String {
        match self {
            CallAttribution::Caller => format!(
                ".mx_parent <- sys.parent()\n{indent}\
                 .mx_def <- if (.mx_parent > 0L) sys.function(.mx_parent)\n{indent}\
                 .mx_pc <- if (.mx_parent > 0L) sys.call(.mx_parent)\n{indent}\
                 .mx_call <- if (typeof(.mx_def) == \"closure\") match.call(.mx_def, .mx_pc, envir = parent.frame(2L)) else match.call()\n{indent}"
            ),
            CallAttribution::Wrapper | CallAttribution::None => String::new(),
        }
    }
}

/// Builder for formatting `.Call()` invocations in R wrapper code.
///
/// Handles the common pattern of `.Call(C_ident, .call = match.call(), args...)`.
///
/// # Example
///
/// ```ignore
/// let call = DotCallBuilder::new("C_Counter__increment")
///     .with_self("self")
///     .build();
/// // => ".Call(C_Counter__increment, .call = match.call(), self)"
///
/// let call = DotCallBuilder::new("C_Counter__add")
///     .with_self("x")
///     .with_args(&["n"])
///     .build();
/// // => ".Call(C_Counter__add, .call = match.call(), x, n)"
/// ```
pub struct DotCallBuilder {
    /// The C entry point symbol name (e.g., `"C_Counter__increment"`).
    /// This is the first argument to `.Call()`.
    c_ident: String,
    /// Optional self/receiver variable name (e.g., `"self"`, `"x"`).
    /// When present, prepended before other arguments in the `.Call()` invocation.
    self_var: Option<String>,
    /// Additional argument names passed after self (if any) in the `.Call()` invocation.
    args: Vec<String>,
    /// Expression for the `.call` named argument. `None` means `match.call()` (the default).
    /// Set via [`DotCallBuilder::null_call_attribution`] to emit `.call = NULL` instead.
    call_expr: Option<String>,
}

impl DotCallBuilder {
    /// Create a new builder with the C function identifier.
    pub fn new(c_ident: impl Into<String>) -> Self {
        Self {
            c_ident: c_ident.into(),
            self_var: None,
            args: Vec::new(),
            call_expr: None,
        }
    }

    /// Add a self/x parameter (prepended to args).
    pub fn with_self(mut self, var: impl Into<String>) -> Self {
        self.self_var = Some(var.into());
        self
    }

    /// Add arguments after self (if any).
    pub fn with_args(mut self, args: &[impl AsRef<str>]) -> Self {
        self.args = args.iter().map(|s| s.as_ref().to_string()).collect();
        self
    }

    /// Add a pre-joined argument string (e.g., `"x, y"`) as a single emit unit.
    ///
    /// Empty strings are ignored, so callers can pass the result of
    /// `build_r_call_args_from_sig` directly without a length check.
    pub fn with_args_str(mut self, args: &str) -> Self {
        if !args.is_empty() {
            self.args.push(args.to_string());
        }
        self
    }

    /// Pass `.call = NULL` instead of `.call = match.call()`.
    ///
    /// Use for lambda dispatch sites (R6 finalizer/`deep_clone`, S7 property
    /// getter/setter/validator) where `match.call()` captures an internal
    /// dispatch frame instead of the user's call. With `NULL`, the
    /// `if (is.null(.val$call)) .call_default else .val$call` fallback in `condition_check_lines` surfaces the
    /// nearest meaningful frame instead.
    pub fn null_call_attribution(mut self) -> Self {
        self.call_expr = Some("NULL".to_string());
        self
    }

    /// Build the `.Call()` string.
    pub fn build(&self) -> String {
        let call_arg = self.call_expr.as_deref().unwrap_or("match.call()");

        let mut all_args = Vec::new();

        if let Some(ref self_var) = self.self_var {
            all_args.push(self_var.clone());
        }
        all_args.extend(self.args.clone());

        if all_args.is_empty() {
            format!(".Call({}, .call = {})", self.c_ident, call_arg)
        } else {
            format!(
                ".Call({}, .call = {}, {})",
                self.c_ident,
                call_arg,
                all_args.join(", ")
            )
        }
    }
}
// endregion

// region: RoxygenBuilder - roxygen2 documentation tag generation

/// Builder for generating roxygen2 documentation tags.
///
/// Provides a fluent API for building common roxygen tag patterns used
/// across all class systems.
///
/// # Example
///
/// ```ignore
/// let tags = RoxygenBuilder::new()
///     .name("Counter$increment")
///     .rdname("Counter")
///     .export()
///     .build();
/// // => vec!["#' @name Counter$increment", "#' @rdname Counter", "#' @export"]
/// ```
pub struct RoxygenBuilder {
    /// Value for `@name` tag. Identifies the documented topic (e.g., `"Counter$increment"`).
    name: Option<String>,
    /// Value for `@rdname` tag. Groups multiple entries onto a single help page
    /// (e.g., all methods of `"Counter"` share one Rd file).
    rdname: Option<String>,
    /// Value for `@title` tag. The one-line title shown in help page headers.
    title: Option<String>,
    /// Value for `@description` tag. Longer description text below the title.
    description: Option<String>,
    /// Value for `@source` tag. Typically `"Generated by miniextendr"` provenance info.
    source: Option<String>,
    /// Whether to emit `@export`. When true, the item is exported from the package NAMESPACE.
    export: bool,
    /// Value for `@exportMethod` tag. Used for S4 method exports (e.g., `"show"`).
    export_method: Option<String>,
    /// Values for `@method` tag as `(generic, class)`. Used for S3 method dispatch
    /// (e.g., `("print", "Counter")` emits `@method print Counter`).
    method: Option<(String, String)>,
    /// Additional custom tag lines emitted verbatim (without the `#' ` prefix,
    /// which is added during [`build`](Self::build)). Used for tags like
    /// `@keywords internal` or `@param` entries.
    custom_tags: Vec<String>,
}

impl RoxygenBuilder {
    /// Create a new empty builder.
    pub fn new() -> Self {
        Self {
            name: None,
            rdname: None,
            title: None,
            description: None,
            source: None,
            export: false,
            export_method: None,
            method: None,
            custom_tags: Vec::new(),
        }
    }

    /// Set the `@name` tag.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the `@rdname` tag (groups docs into one page).
    pub fn rdname(mut self, rdname: impl Into<String>) -> Self {
        self.rdname = Some(rdname.into());
        self
    }

    /// Set the `@title` tag.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the `@description` tag.
    #[allow(dead_code)] // Exercised by tests
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the `@source` tag (typically "Generated by miniextendr...").
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Add `@export` tag.
    pub fn export(mut self) -> Self {
        self.export = true;
        self
    }

    /// Add `@exportMethod` tag (for S4).
    #[allow(dead_code)] // Exercised by tests
    pub fn export_method(mut self, method: impl Into<String>) -> Self {
        self.export_method = Some(method.into());
        self
    }

    /// Add `@method` tag (for S3).
    pub fn method(mut self, generic: impl Into<String>, class: impl Into<String>) -> Self {
        self.method = Some((generic.into(), class.into()));
        self
    }

    /// Add a custom tag line (without the `#' ` prefix).
    pub fn custom(mut self, tag: impl Into<String>) -> Self {
        self.custom_tags.push(tag.into());
        self
    }

    /// Build the roxygen tag lines (each prefixed with `#' `).
    pub fn build(&self) -> Vec<String> {
        let mut lines = Vec::new();

        if let Some(ref title) = self.title {
            lines.push(format!("#' @title {}", title));
        }
        if let Some(ref desc) = self.description {
            lines.push(format!("#' @description {}", desc));
        }
        if let Some(ref name) = self.name {
            lines.push(format!("#' @name {}", name));
        }
        if let Some(ref rdname) = self.rdname {
            lines.push(format!("#' @rdname {}", rdname));
        }
        if let Some(ref source) = self.source {
            lines.push(format!("#' @source {}", source));
        }
        if let Some((ref generic, ref class)) = self.method {
            lines.push(format!("#' @method {} {}", generic, class));
        }
        for tag in &self.custom_tags {
            lines.push(format!("#' {}", tag));
        }
        if self.export {
            lines.push("#' @export".to_string());
        }
        if let Some(ref method) = self.export_method {
            lines.push(format!("#' @exportMethod {}", method));
        }

        lines
    }
}

/// Creates an empty builder with no tags set.
impl Default for RoxygenBuilder {
    fn default() -> Self {
        Self::new()
    }
}
// endregion

// region: Tests

#[cfg(test)]
mod tests;
// endregion
