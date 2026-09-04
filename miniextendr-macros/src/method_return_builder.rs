//! Shared utilities for handling method return values in R wrapper generation.
//!
//! This module provides helpers for generating consistent return value handling
//! across all class systems (Env, R6, S7, S3, S4).

use crate::miniextendr_impl::ParsedMethod;
use crate::miniextendr_impl::ReceiverKind;

// region: Shared R error-check helpers
//
// All wrappers route through the internal helper `.miniextendr_raise_condition`
// emitted once at the top of the generated wrappers file (see
// `miniextendr-api/src/registry.rs` `write_r_wrappers_to_file`). The helper
// performs the `.val$kind` dispatch and `rust_*` class layering; each wrapper
// only needs a one-line guard at the call site.

/// Generate the R guard that re-raises a tagged Rust error/condition value.
///
/// Expects `.val` to already be assigned (e.g., `.val <- .Call(...)`). Emits a
/// single line indented by `indent`: when `.val` is a tagged `rust_condition_value`,
/// hand off to the shared helper and return from the enclosing function.
///
/// The helper dispatches on `.val$kind` (see
/// `miniextendr_api::error_value::kind` for canonical kind strings):
///
/// - `error` / `panic` / `result_err` / `none_err` / `conversion` (and any
///   unknown kind) — `stop()` longjmps with the appropriate `rust_*` class
///   layering.
/// - `warning` — `warning()` signals; the wrapper's surrounding `return(...)`
///   propagates `invisible(NULL)` as the wrapper's result.
/// - `message` — `message()` signals; same propagation.
/// - `condition` — `signalCondition()` signals; same propagation.
pub fn condition_check_lines(indent: &str) -> Vec<String> {
    vec![format!(
        "{indent}if (inherits(.val, \"rust_condition_value\") && isTRUE(attr(.val, \"__rust_condition__\"))) return(.miniextendr_raise_condition(.val, sys.call()))"
    )]
}

/// Generate an inline R error-check block for single-expression contexts (S7, S4).
///
/// Returns a multi-line block string: `{ .val <- <call_expr>; if (...) return(...); <inner> }`.
/// Used where the class system requires a single expression rather than separate lines
/// (e.g., S7 property definitions, S4 method bodies).
///
/// - `call_expr`: The `.Call()` expression to evaluate
/// - `inner`: The final expression to return after the error check passes
/// - `indent`: Leading whitespace for the inner lines (e.g., `"    "` for 4-space)
pub fn condition_check_inline_block(call_expr: &str, inner: &str, indent: &str) -> String {
    format!(
        "{{\n{indent}.val <- {call_expr}\n\
         {indent}if (inherits(.val, \"rust_condition_value\") && isTRUE(attr(.val, \"__rust_condition__\"))) return(.miniextendr_raise_condition(.val, sys.call()))\n\
         {indent}{inner}\n  \
         }}"
    )
}

/// Generate a standalone-function R wrapper body.
///
/// Returns the full body string: `.val <- <call_expr>; if (...) return(...); <final_return>`.
/// Used for top-level `#[miniextendr]` functions (not class methods).
///
/// - `call_expr`: The `.Call()` expression to evaluate
/// - `final_return`: The expression to return (typically `".val"` or `"invisible(.val)"`)
/// - `indent`: Leading whitespace for the body lines (e.g., `"  "` for 2-space)
pub fn standalone_body(call_expr: &str, final_return: &str, indent: &str) -> String {
    format!(
        ".val <- {call_expr}\n\
         {indent}if (inherits(.val, \"rust_condition_value\") && isTRUE(attr(.val, \"__rust_condition__\"))) return(.miniextendr_raise_condition(.val, sys.call()))\n\
         {indent}{final_return}"
    )
}
// endregion

// region: Return strategy

/// Return handling strategy for class methods.
///
/// Determines how the R wrapper function processes and returns the `.Call()` result.
/// Each class system generator uses this to produce idiomatic R return code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnStrategy {
    /// The method returns `Self`. The wrapper wraps the raw pointer result with
    /// the appropriate class attribute or creates a new class object (e.g.,
    /// `R6Class$new(.ptr = result)` or `structure(result, class = "...")`).
    ReturnSelf,
    /// The method returns a bare type name that may be another registered
    /// ExternalPtr-backed class. The final decision is deferred to
    /// `write_r_wrappers_to_file`, where the complete class registry is known.
    ReturnOtherClass,
    /// The method returns a *list* of values (`Vec<Class>`, plus the
    /// `Option<Vec<Class>>` / `Result<Vec<Class>, E>` wrappers the C wrapper
    /// already unwraps) whose element type may be another registered
    /// ExternalPtr-backed class (#1284). Deferred to `write_r_wrappers_to_file`
    /// like [`ReturnOtherClass`](Self::ReturnOtherClass), but through a
    /// distinct `.__MX_WRAP_LIST_RETURN_*` marker that resolves to a per-element
    /// `lapply(...)` wrap. The marker prefix must never overlap the scalar
    /// `.__MX_WRAP_RETURN_` family: the scalar resolver rewrites unrecognized
    /// markers of its own prefix to the bare expression, so a shared prefix
    /// would consume list markers before the list resolver ran.
    ReturnOtherClassList,
    /// The method is a `&mut self` method returning `()`. The wrapper calls the
    /// `.Call()` for its side effect and returns the receiver (`self`/`x`) for
    /// method chaining (e.g., `invisible(self)` for R6).
    ChainableMutation,
    /// Default strategy: return the `.Call()` result directly without wrapping.
    Direct,
}

impl ReturnStrategy {
    /// Determine the return strategy for a parsed method.
    ///
    /// - Methods that return `Self`, `Result<Self, E>`, or `Option<Self>` use
    ///   `ReturnSelf`. For the latter two, the C wrapper already raised on
    ///   `Err` / `None` (see
    ///   [`crate::c_wrapper_builder::ReturnHandling::ResultExternalPtr`] and
    ///   [`crate::c_wrapper_builder::ReturnHandling::OptionExternalPtr`]), so a
    ///   successful `.val` is a bare ExternalPtr — identical in shape to the
    ///   bare-`Self` case — and gets the same class-wrapping tail.
    /// - In-place builders (`&mut self -> &mut Self` / `&self -> Self`) and
    ///   `&mut self -> ()` methods use `ChainableMutation`. Both return the
    ///   receiver object (`x` / `invisible(self)`) so the call composes under
    ///   the native pipe (`obj |> set_a(1) |> set_b(2)`); the C wrapper hands
    ///   back the same ExternalPtr handle (see
    ///   [`crate::c_wrapper_builder::ReturnHandling::SelfHandle`]).
    /// - Bare capitalized return types that are not known primitives/containers
    ///   use `ReturnOtherClass`; write-time registry lookup wraps registered
    ///   classes and leaves false positives unchanged.
    /// - `Vec<Class>` (and the `Option`/`Result` wrappers of it the C wrapper
    ///   unwraps) use `ReturnOtherClassList`; write-time registry lookup wraps
    ///   registered element classes via `lapply` and leaves false positives
    ///   unchanged (#1284).
    /// - All other methods use `Direct`.
    pub fn for_method(method: &ParsedMethod) -> Self {
        // In-place builders (`&mut self -> &mut Self` / `&self -> Self`), their
        // fallible forms (`-> Result<&mut Self, E>` / `Option<&mut Self>`, #1433),
        // `&mut self -> ()` mutators, and consuming steps (`self -> Self` /
        // `Result<Self, E>` / `Option<Self>`, #1432: the C wrapper writes the
        // result back into the same handle) all return the receiver object.
        let is_self_ref_builder = method.env.is_instance()
            && (method.returns_self_ref()
                || method.returns_result_self_ref()
                || method.returns_option_self_ref());
        let is_unit_mutator = method.env.is_mut() && method.returns_unit();
        let returns_self_shaped =
            method.returns_self() || method.returns_result_self() || method.returns_option_self();
        let is_consuming_builder = method.env == ReceiverKind::Value && returns_self_shaped;
        if is_consuming_builder || is_self_ref_builder || is_unit_mutator {
            ReturnStrategy::ChainableMutation
        } else if returns_self_shaped {
            ReturnStrategy::ReturnSelf
        } else if method.returns_other_class().is_some() {
            ReturnStrategy::ReturnOtherClass
        } else if method.returns_other_class_list().is_some() {
            ReturnStrategy::ReturnOtherClassList
        } else {
            ReturnStrategy::Direct
        }
    }
}

/// Class-specific tail closures, one per [`ReturnStrategy`] variant.
///
/// Each closure receives `(indent, class_name)` and produces the tail lines that
/// run after `.val <- <call_expr>` and the condition check (both emitted by
/// [`MethodReturnBuilder::build_with_tails`]). The tail therefore always
/// references `.val` directly.
///
/// `class_name` is `""` for [`ReturnStrategy::ChainableMutation`],
/// [`ReturnStrategy::ReturnOtherClass`],
/// [`ReturnStrategy::ReturnOtherClassList`], and [`ReturnStrategy::Direct`] —
/// those tails should not use it.
#[allow(clippy::type_complexity)]
struct ReturnTails<'a> {
    /// Tail for [`ReturnStrategy::ReturnSelf`].
    ///
    /// Parameters: `(indent, class_name) -> lines`
    self_tail: Box<dyn Fn(&str, &str) -> Vec<String> + 'a>,
    /// Tail for [`ReturnStrategy::ChainableMutation`].
    ///
    /// Parameters: `(indent) -> lines`
    chain_tail: Box<dyn Fn(&str) -> Vec<String> + 'a>,
    /// Tail for [`ReturnStrategy::Direct`].
    ///
    /// Parameters: `(indent) -> lines`
    direct_tail: Box<dyn Fn(&str) -> Vec<String> + 'a>,
}

/// Builder for generating R method body lines with appropriate return handling.
///
/// Produces lines of R code for a method body, combining the `.Call()` expression
/// with the return strategy and the tagged-condition error guard. Each class
/// system has specialized builder methods (`build_r6_body`, `build_s3_body`,
/// etc.) that produce idiomatic R code for that system.
pub struct MethodReturnBuilder {
    /// The `.Call()` expression string (e.g., `".Call(C_Counter__inc, .call = match.call(), self)"`).
    call_expr: String,
    /// How to handle the return value (direct, chaining, or Self wrapping).
    strategy: ReturnStrategy,
    /// R class name, required when `strategy` is `ReturnSelf` to construct
    /// the class wrapper (e.g., `"Counter"` for `Counter$new(.ptr = result)`).
    class_name: Option<String>,
    /// Rust return type name, required when `strategy` is `ReturnOtherClass`
    /// or `ReturnOtherClassList` (the element type in the list case). The
    /// write-time resolver maps this Rust name to the registered target class
    /// and constructor syntax, or falls back to the bare value on miss.
    return_class: Option<String>,
    /// Variable name to return for `ChainableMutation` strategy (e.g., `"self"` for R6,
    /// `"x"` for S3). Defaults to `"self"` if not set.
    chain_var: Option<String>,
    /// Number of leading spaces for each generated line.
    indent: usize,
}

impl MethodReturnBuilder {
    /// Create a new builder with the given .Call expression.
    pub fn new(call_expr: String) -> Self {
        Self {
            call_expr,
            strategy: ReturnStrategy::Direct,
            class_name: None,
            return_class: None,
            chain_var: None,
            indent: 2,
        }
    }

    /// Set the return strategy.
    pub fn with_strategy(mut self, strategy: ReturnStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set the class name (for Self returns).
    pub fn with_class_name(mut self, class_name: String) -> Self {
        self.class_name = Some(class_name);
        self
    }

    /// Set the return class name (for cross-class ExternalPtr returns).
    pub fn with_return_class(mut self, return_class: String) -> Self {
        self.return_class = Some(return_class);
        self
    }

    /// Attach the method's cross-class return name when this builder uses
    /// [`ReturnStrategy::ReturnOtherClass`] or
    /// [`ReturnStrategy::ReturnOtherClassList`].
    pub fn with_return_class_from_method(mut self, method: &ParsedMethod) -> Self {
        match self.strategy {
            ReturnStrategy::ReturnOtherClass => {
                let return_class = method
                    .returns_other_class()
                    .expect("return_class required for ReturnOtherClass strategy");
                self = self.with_return_class(return_class.to_string());
            }
            ReturnStrategy::ReturnOtherClassList => {
                let return_class = method
                    .returns_other_class_list()
                    .expect("return_class required for ReturnOtherClassList strategy");
                self = self.with_return_class(return_class.to_string());
            }
            _ => {}
        }
        self
    }

    /// Set the variable name to return for chaining (default: "self").
    pub fn with_chain_var(mut self, var: String) -> Self {
        self.chain_var = Some(var);
        self
    }

    /// Set indentation level (number of spaces).
    pub fn with_indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    fn return_other_class_expr(&self) -> String {
        let return_class = self
            .return_class
            .as_ref()
            .expect("return_class required for ReturnOtherClass strategy");
        format!(".__MX_WRAP_RETURN_{return_class}__(.val)")
    }

    /// List-shaped sibling of
    /// [`return_other_class_expr`](Self::return_other_class_expr) (#1284).
    ///
    /// The `LIST_` infix keeps the marker family disjoint from the scalar
    /// `.__MX_WRAP_RETURN_` prefix: the scalar resolver rewrites unrecognized
    /// markers of its own prefix to the bare expression, so a shared prefix
    /// would destroy list markers before the list resolver saw them.
    fn return_other_class_list_expr(&self) -> String {
        let return_class = self
            .return_class
            .as_ref()
            .expect("return_class required for ReturnOtherClassList strategy");
        format!(".__MX_WRAP_LIST_RETURN_{return_class}__(.val)")
    }

    // region: Core shared build path

    /// Emit:
    /// ```text
    /// <indent>.val <- <call_expr>
    /// <indent>if (inherits(.val, ...) ...) return(...)
    /// <tail lines — .val is live>
    /// ```
    ///
    /// All paths capture the `.Call()` result, dispatch on the tagged
    /// condition value if present, and then run the class-specific tail.
    fn build_with_tails(&self, tails: ReturnTails<'_>) -> Vec<String> {
        let indent = " ".repeat(self.indent);
        let class_name = self.class_name.as_deref().unwrap_or("");
        let call_expr = &self.call_expr;

        let mut lines = vec![format!("{}.val <- {}", indent, call_expr)];
        lines.extend(condition_check_lines(&indent));
        match self.strategy {
            ReturnStrategy::ReturnSelf => {
                lines.extend((tails.self_tail)(&indent, class_name));
            }
            ReturnStrategy::ChainableMutation => {
                lines.extend((tails.chain_tail)(&indent));
            }
            ReturnStrategy::ReturnOtherClass => {
                lines.push(format!("{}{}", indent, self.return_other_class_expr()));
            }
            ReturnStrategy::ReturnOtherClassList => {
                lines.push(format!("{}{}", indent, self.return_other_class_list_expr()));
            }
            ReturnStrategy::Direct => {
                lines.extend((tails.direct_tail)(&indent));
            }
        }
        lines
    }

    // endregion

    /// Build R code lines for the method body.
    ///
    /// Returns a vector of strings, one per line (without trailing newlines).
    pub fn build(&self) -> Vec<String> {
        let chain_var = self.chain_var.as_deref().unwrap_or("self").to_owned();
        self.build_with_tails(ReturnTails {
            self_tail: Box::new(|indent, class_name| {
                assert!(
                    !class_name.is_empty(),
                    "class_name required for ReturnSelf strategy"
                );
                vec![
                    format!("{}class(.val) <- \"{}\"", indent, class_name),
                    format!("{}.val", indent),
                ]
            }),
            chain_tail: Box::new(move |indent| vec![format!("{}{}", indent, chain_var)]),
            direct_tail: Box::new(|indent| vec![format!("{}.val", indent)]),
        })
    }
}

/// Specialized builders for different class systems.
impl MethodReturnBuilder {
    /// Build R6-style return (uses invisible(self) for chaining).
    pub fn build_r6_body(&self) -> Vec<String> {
        self.build_with_tails(ReturnTails {
            self_tail: Box::new(|indent, class_name| {
                assert!(
                    !class_name.is_empty(),
                    "class_name required for ReturnSelf strategy"
                );
                vec![format!("{}{}$new(.ptr = .val)", indent, class_name)]
            }),
            chain_tail: Box::new(|indent| vec![format!("{}invisible(self)", indent)]),
            direct_tail: Box::new(|indent| vec![format!("{}.val", indent)]),
        })
    }

    /// Build S3-style return (uses structure() for Self returns).
    pub fn build_s3_body(&self) -> Vec<String> {
        let chain_var = self.chain_var.as_deref().unwrap_or("x").to_owned();
        self.build_with_tails(ReturnTails {
            self_tail: Box::new(|indent, class_name| {
                assert!(
                    !class_name.is_empty(),
                    "class_name required for ReturnSelf strategy"
                );
                vec![format!(
                    "{}structure(.val, class = \"{}\")",
                    indent, class_name
                )]
            }),
            chain_tail: Box::new(move |indent| vec![format!("{}{}", indent, chain_var)]),
            direct_tail: Box::new(|indent| vec![format!("{}.val", indent)]),
        })
    }

    /// Build S7-style method body lines (creates new S7 object with .ptr).
    ///
    /// Returns lines suitable for embedding inside an outer `function(...) { ... }`
    /// block — unlike [`build_s7_inline`](Self::build_s7_inline) which wraps the
    /// body in its own `{ }` and is intended for callers that emit
    /// `function(...) <expr>` directly (e.g., S7 `convert` definitions).
    pub fn build_s7_body(&self) -> Vec<String> {
        // The chained-mutation tail returns the receiver. The S7 generic method
        // names its receiver `x`; the per-class fast-path shortcut (#949) names
        // it `self`. Honour `chain_var` (default `x`) so both reuse this body.
        let chain_var = self.chain_var.as_deref().unwrap_or("x").to_owned();
        self.build_with_tails(ReturnTails {
            self_tail: Box::new(|indent, class_name| {
                assert!(
                    !class_name.is_empty(),
                    "class_name required for ReturnSelf strategy"
                );
                vec![format!("{}{}(.ptr = .val)", indent, class_name)]
            }),
            chain_tail: Box::new(move |indent| vec![format!("{}{}", indent, chain_var)]),
            direct_tail: Box::new(|indent| vec![format!("{}.val", indent)]),
        })
    }

    /// Build S4-style method body lines (uses methods::new() to wrap Self returns).
    ///
    /// Returns lines suitable for embedding inside an outer
    /// `function(...) { ... }` block, mirroring [`build_s7_body`](Self::build_s7_body).
    pub fn build_s4_body(&self) -> Vec<String> {
        self.build_with_tails(ReturnTails {
            self_tail: Box::new(|indent, class_name| {
                assert!(
                    !class_name.is_empty(),
                    "class_name required for ReturnSelf strategy"
                );
                vec![format!(
                    "{}methods::new(\"{}\", ptr = .val)",
                    indent, class_name
                )]
            }),
            chain_tail: Box::new(|indent| vec![format!("{}x", indent)]),
            direct_tail: Box::new(|indent| vec![format!("{}.val", indent)]),
        })
    }

    /// Build S7-style return (creates new S7 object with .ptr).
    ///
    /// Returns a multi-line block expression that performs the condition check
    /// inline (suitable for S7 property definitions / convert methods that
    /// require a single expression).
    pub fn build_s7_inline(&self) -> String {
        let inner = match self.strategy {
            ReturnStrategy::ReturnSelf => {
                let class_name = self
                    .class_name
                    .as_ref()
                    .expect("class_name required for ReturnSelf strategy");
                format!("{}(.ptr = .val)", class_name)
            }
            ReturnStrategy::ChainableMutation => "x".to_string(),
            ReturnStrategy::ReturnOtherClass => self.return_other_class_expr(),
            ReturnStrategy::ReturnOtherClassList => self.return_other_class_list_expr(),
            ReturnStrategy::Direct => ".val".to_string(),
        };
        condition_check_inline_block(&self.call_expr, &inner, "    ")
    }

    /// Build S4-style return (uses methods::new()).
    ///
    /// Returns a multi-line block expression that performs the condition check
    /// inline.
    pub fn build_s4_inline(&self) -> String {
        let inner = match self.strategy {
            ReturnStrategy::ReturnSelf => {
                let class_name = self
                    .class_name
                    .as_ref()
                    .expect("class_name required for ReturnSelf strategy");
                format!("methods::new(\"{}\", ptr = .val)", class_name)
            }
            ReturnStrategy::ChainableMutation => "x".to_string(),
            ReturnStrategy::ReturnOtherClass => self.return_other_class_expr(),
            ReturnStrategy::ReturnOtherClassList => self.return_other_class_list_expr(),
            ReturnStrategy::Direct => ".val".to_string(),
        };
        condition_check_inline_block(&self.call_expr, &inner, "    ")
    }
}

#[cfg(test)]
mod tests;
// endregion
