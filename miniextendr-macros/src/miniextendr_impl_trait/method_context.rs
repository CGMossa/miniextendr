//! Shared per-method context for trait-impl R wrapper generation.
//!
//! Parallels `r_class_formatter::MethodContext` (used by the 6 inherent-impl
//! class generators) so the 5 trait generators (env/s3/s4/s7/r6) build
//! `.Call()` invocations and method preludes the same way instead of each
//! hand-rolling its own version. See
//! `audit/2026-07-03-dogfooding-macros-codegen.md` finding #1: the trait path
//! previously shared nothing above `DotCallBuilder`, which caused a
//! substring-corruption bug in the receiver-ptr extraction (S4/S7/R6) and a
//! silent 3-of-6-step prelude omission (missing `precondition_checks` /
//! `match_arg_prelude`) relative to inherent methods.

use super::{TraitMethod, trait_method_body_lines};
use crate::miniextendr_impl::ClassSystem;

/// R-visible assignment target for a trait member (method or const) placed in
/// the per-class trait namespace — the single owner of the namespace-shape
/// policy the 5 generators previously hand-rolled independently (audit
/// `2026-07-03-dogfooding-macros-codegen.md` finding #1; #1141). `member` is
/// the R-facing method/const name.
///
/// Policy:
/// - **Env / S3 / R6 / Vctrs**: `Type$Trait$member` — class-scoped, so it is
///   collision-free by construction. This is what fixes #1115: two R6 impls of
///   one trait on *different* types now emit `TypeA$Trait$m` / `TypeB$Trait$m`
///   rather than a shared, unqualified `r6_trait_Trait_m` that aborted
///   wrapper-gen via the duplicate-definition guard.
/// - **S4**: flat `Type_Trait_member` standalone name. S4 objects intercept
///   `$<-`, so `Type$Trait$member` cannot be assigned onto them; the class
///   component in the flat name keeps it collision-free.
/// - **S7**: `env_var$member`, where `env_var` = [`trait_namespace_env_var`]
///   (`.Type__Trait`) is a local env attached to `Type` via `attr()` at the end
///   of the generator. S7 objects also intercept `$<-`; routing through an
///   attribute-attached env lets `Type$Trait$member` still resolve at the call
///   site (R's `$` on an S7 object falls through to attributes).
pub(super) fn trait_namespace_target(
    class_system: ClassSystem,
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    member: &str,
) -> String {
    match class_system {
        ClassSystem::Env | ClassSystem::S3 | ClassSystem::R6 | ClassSystem::Vctrs => {
            format!("{type_ident}${trait_name}${member}")
        }
        ClassSystem::S4 => format!("{type_ident}_{trait_name}_{member}"),
        ClassSystem::S7 => {
            format!(
                "{}${member}",
                trait_namespace_env_var(type_ident, trait_name)
            )
        }
    }
}

/// The local environment variable S7 trait wrappers assign members into before
/// attaching it to the class object via `attr()`. One owner so
/// [`trait_namespace_target`]'s S7 arm and the generator's `new.env` / `attr()`
/// lines can't drift apart.
pub(super) fn trait_namespace_env_var(type_ident: &syn::Ident, trait_name: &syn::Ident) -> String {
    format!(".{type_ident}__{trait_name}")
}

/// Pre-computed context for a trait method, mirroring `MethodContext`
/// (`r_class_formatter.rs`) for inherent-impl methods. Holds the C wrapper
/// name, R formals (with defaults), and `.Call()` argument string so all 5
/// trait generators build calls identically.
pub(super) struct TraitMethodContext<'a> {
    /// Reference to the parsed trait method metadata.
    pub(super) method: &'a TraitMethod,
    /// The implementing type ident (e.g. `Foo`) — used to build the per-class
    /// trait-namespace assignment target and the Self-return wrapper class.
    pub(super) type_ident: &'a syn::Ident,
    /// The trait ident (e.g. `Bar`) — used to build the per-class trait
    /// namespace assignment target.
    pub(super) trait_name: &'a syn::Ident,
    /// The C wrapper identifier string (e.g., `"C_Foo__Bar__value"`), used in `.Call()`.
    pub(super) c_ident: String,
    /// R formals string with defaults, used in `function(...)` signatures.
    pub(super) params: String,
    /// R call arguments string (without defaults), used inside `.Call()`
    /// expressions. `Missing<T>` parameters are forwarded as
    /// `if (missing(p)) quote(expr=) else p` — see
    /// `r_wrapper_builder::RArgumentBuilder::build_call_args_vec`. Before this
    /// context existed, trait methods built call args via
    /// `collect_param_idents` instead, which skipped that forwarding entirely.
    pub(super) args: String,
}

impl<'a> TraitMethodContext<'a> {
    /// Build a context for `method`, which implements `trait_name` for `type_ident`.
    pub(super) fn new(
        method: &'a TraitMethod,
        type_ident: &'a syn::Ident,
        trait_name: &'a syn::Ident,
    ) -> Self {
        let c_ident = method.c_wrapper_ident_string(type_ident, trait_name);
        // match_arg/choices formal defaults are load-bearing for match.arg()
        // (see `effective_r_defaults` docs) — not just cosmetic.
        let effective_defaults = crate::r_class_formatter::effective_r_defaults(
            &method.param_defaults,
            &method.per_param,
            &c_ident,
        );
        let params =
            crate::r_wrapper_builder::build_r_formals_from_sig(&method.sig, &effective_defaults);
        let args = crate::r_wrapper_builder::build_r_call_args_from_sig(&method.sig);
        Self {
            method,
            type_ident,
            trait_name,
            c_ident,
            params,
            args,
        }
    }

    /// The R-visible assignment target for this method within `class_system`'s
    /// per-class trait namespace — a thin wrapper over [`trait_namespace_target`]
    /// using this method's R-facing name. See that function for the policy.
    pub(super) fn namespace_target(&self, class_system: ClassSystem) -> String {
        trait_namespace_target(
            class_system,
            self.type_ident,
            self.trait_name,
            &self.method.r_method_name(),
        )
    }

    /// Whether this method re-wraps its return into a classed object — i.e. it
    /// returns `Self` / `Result<Self, E>` / `Option<Self>`. Mirrors the
    /// `ReturnStrategy::ReturnSelf` arm of `ReturnStrategy::for_method` on the
    /// inherent-impl path.
    pub(super) fn returns_self(&self) -> bool {
        self.method.returns_self()
            || self.method.returns_result_self()
            || self.method.returns_option_self()
    }

    /// Emit the R body lines for a trait method (2-space indent): capture
    /// `.Call()` into `.val`, run the tagged-condition guard, then return
    /// `.val` — or, for `-> Self` factory methods, re-wrap `.val` into a
    /// classed object via the shared [`crate::MethodReturnBuilder`], exactly
    /// as the inherent-impl generators do (audit finding #4 / #1141). The
    /// wrapper class is the implementing type; the wrapping idiom is selected
    /// per class system (`class(.val) <-` / `structure()` / `methods::new()` /
    /// `Class(.ptr=)` / `Class$new(.ptr=)`).
    pub(super) fn method_body_lines(&self, call: &str, class_system: ClassSystem) -> Vec<String> {
        if !self.returns_self() {
            return trait_method_body_lines(call, "  ");
        }
        let builder = crate::MethodReturnBuilder::new(call.to_string())
            .with_strategy(crate::ReturnStrategy::ReturnSelf)
            .with_class_name(self.type_ident.to_string())
            .with_indent(2);
        match class_system {
            ClassSystem::Env => builder.build(),
            ClassSystem::S3 | ClassSystem::Vctrs => builder.build_s3_body(),
            ClassSystem::S4 => builder.build_s4_body(),
            ClassSystem::S7 => builder.build_s7_body(),
            ClassSystem::R6 => builder.build_r6_body(),
        }
    }

    /// Build the `.Call()` expression for a static (non-receiver) trait method.
    pub(super) fn static_call(&self) -> String {
        crate::r_wrapper_builder::DotCallBuilder::new(&self.c_ident)
            .with_args_str(&self.args)
            .build()
    }

    /// Build the `.Call()` expression for an instance trait method, with
    /// `self_expr` passed directly as the receiver argument (e.g. `".ptr"`,
    /// `"x"`, `"self@.ptr"`).
    ///
    /// This is the structured equivalent of `MethodContext::instance_call` —
    /// no string surgery. It fixes the substring-corruption bug where S4/S7/R6
    /// built the call with `self = "x"` and then ran
    /// `call.replace(", x", ", .ptr")`: `str::replace` rewrites *every* match
    /// of the substring `", x"`, so a parameter whose R name started with `x`
    /// (e.g. `x_factor`) was corrupted into `.ptr_factor`, producing a runtime
    /// "object '.ptr_factor' not found" error. Passing the receiver expression
    /// directly to `with_self` never touches the other arguments.
    pub(super) fn instance_call(&self, self_expr: &str) -> String {
        crate::r_wrapper_builder::DotCallBuilder::new(&self.c_ident)
            .with_self(self_expr)
            .with_args_str(&self.args)
            .build()
    }

    /// Build R prelude lines validating `match_arg`/`choices` params. See
    /// `r_class_formatter::build_match_arg_prelude`.
    pub(super) fn match_arg_prelude(&self) -> Vec<String> {
        crate::r_class_formatter::build_match_arg_prelude(&self.method.per_param, &self.c_ident)
    }

    /// R-side `stopifnot()` precondition checks for this method's parameters.
    /// See `MethodContext::precondition_checks` for the inherent-impl twin.
    pub(super) fn precondition_checks(&self) -> Vec<String> {
        crate::r_class_formatter::build_method_precondition_checks(
            &self.method.sig.inputs,
            &self.method.per_param,
            self.method.coerce,
        )
    }

    /// Emit the shared method prelude into `lines`, each line prefixed with
    /// `indent` — the trait-impl twin of `MethodContext::emit_method_prelude`.
    /// In order: `r_entry`, `r_on_exit`, `lifecycle_prelude`,
    /// `precondition_checks`, `match_arg_prelude`, `r_post_checks`.
    ///
    /// `Missing<T>` forwarding is not a prelude step here either — it's inline
    /// in `self.args` (built in `new`), matching the inherent path.
    ///
    /// `what` is the human-readable label passed to the lifecycle prelude.
    /// This mirrors the pre-refactor `trait_method_preamble_lines`, which
    /// always used the R-facing method name as `what` regardless of class
    /// system (inherent methods instead use a class-qualified label like
    /// `"Type.method"` — trait methods keep the simpler unqualified label to
    /// avoid changing existing lifecycle-warning wording).
    pub(super) fn emit_method_prelude(&self, lines: &mut Vec<String>, indent: &str, what: &str) {
        let m = self.method;
        if let Some(ref entry) = m.r_entry {
            for line in entry.lines() {
                lines.push(format!("{indent}{line}"));
            }
        }
        if let Some(ref on_exit) = m.r_on_exit {
            lines.push(format!("{indent}{}", on_exit.to_r_code()));
        }
        if let Some(ref spec) = m.lifecycle
            && let Some(prelude) = spec.r_prelude(what)
        {
            for line in prelude.lines() {
                lines.push(format!("{indent}{line}"));
            }
        }
        for check in self.precondition_checks() {
            lines.push(format!("{indent}{check}"));
        }
        for line in self.match_arg_prelude() {
            lines.push(format!("{indent}{line}"));
        }
        if let Some(ref post) = m.r_post_checks {
            for line in post.lines() {
                lines.push(format!("{indent}{line}"));
            }
        }
    }
}
