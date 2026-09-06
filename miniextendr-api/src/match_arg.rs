//! `match.arg`-style enum conversion for R string arguments.
//!
//! This module provides the [`MatchArg`] trait for converting between Rust
//! fieldless enums and R character strings with `match.arg` semantics
//! (exact match or unique partial matching).
//!
//! # Usage
//!
//! ```ignore
//! use miniextendr_api::MatchArg;
//!
//! #[derive(Copy, Clone, MatchArg)]
//! #[match_arg(rename_all = "snake_case")]
//! enum Mode {
//!     Fast,
//!     Safe,
//!     Debug,
//! }
//!
//! #[miniextendr]
//! fn run(#[miniextendr(match_arg)] mode: Mode) -> String {
//!     format!("{mode:?}")
//! }
//! ```
//!
//! The generated R wrapper uses `base::match.arg()` for validation before
//! the main `.Call()`, giving users familiar R error messages and partial
//! matching.
//!
//! An `Option<T>` parameter (`#[miniextendr(match_arg)] mode: Option<Mode>`)
//! is the optional form: the R formal defaults to `NULL`, `NULL` converts to
//! `None`, and any other value is matched as usual. The generated C wrapper
//! decodes it with [`match_arg_option_from_sexp`] (there is no
//! `TryFromSexp for Option<T>` blanket: that slot belongs to the newtype
//! forwarding impls in [`crate::newtype`], and a downstream crate cannot add
//! one for its own enum under the orphan rule), so a hand-written `MatchArg`
//! impl needs nothing extra.
//!
//! `several_ok` parameters are validated strictly on the R side: every element
//! has to match a choice, and `NULL` selects every choice (the same fallback
//! [`match_arg_vec_from_sexp`] applies).

use crate::from_r::{SexpError, TryFromSexp, charsxp_to_str};
use crate::gc_protect::ProtectScope;
use crate::{R_xlen_t, SEXP, SEXPTYPE, SexpExt};

/// Trait for enum types that support `match.arg`-style string conversion.
///
/// Implementors provide a fixed set of choice strings and bidirectional
/// conversion between enum variants and their string representations.
///
/// Use `#[derive(MatchArg)]` to auto-generate this implementation.
pub trait MatchArg: Sized + Copy + 'static {
    /// The canonical choice strings, in variant declaration order.
    ///
    /// The first choice is the default when the R argument is `NULL`.
    const CHOICES: &'static [&'static str];

    /// Convert a choice string to the corresponding enum variant.
    ///
    /// Returns `None` if the string doesn't match any choice exactly.
    fn from_choice(choice: &str) -> Option<Self>;

    /// Convert the enum variant to its canonical choice string.
    fn to_choice(self) -> &'static str;
}

/// Error type for `MatchArg` conversion failures.
#[derive(Debug, Clone)]
pub enum MatchArgError {
    /// The SEXP was not a character or factor type.
    InvalidType(SEXPTYPE),
    /// The input had length != 1.
    InvalidLength(usize),
    /// The input was NA.
    IsNa,
    /// No choice matched the input.
    NoMatch {
        /// The input string that didn't match.
        input: String,
        /// The valid choices.
        choices: &'static [&'static str],
    },
}

impl std::fmt::Display for MatchArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchArgError::InvalidType(ty) => {
                write!(f, "match.arg: expected character or factor, got {:?}", ty)
            }
            MatchArgError::InvalidLength(len) => {
                write!(f, "match.arg: expected length 1, got {}", len)
            }
            MatchArgError::IsNa => write!(f, "match.arg: input is NA"),
            MatchArgError::NoMatch { input, choices } => {
                write!(
                    f,
                    "'arg' should be one of {}, got {:?}",
                    choices
                        .iter()
                        .map(|c| format!("{:?}", c))
                        .collect::<Vec<_>>()
                        .join(", "),
                    input,
                )
            }
        }
    }
}

impl std::error::Error for MatchArgError {}

impl From<MatchArgError> for crate::from_r::SexpError {
    fn from(e: MatchArgError) -> Self {
        crate::from_r::SexpError::InvalidValue(e.to_string())
    }
}

/// Escape a Rust `&str` for embedding inside an R double-quoted string literal.
///
/// Handles `\`, `"`, newline, carriage return, and tab — the characters R
/// recognises as escape sequences inside `"..."`. Used when formatting
/// `MatchArg::CHOICES` into the default of a generated R wrapper formal, so
/// that a choice like `say "hi"` or `c:\path` cannot produce syntactically
/// invalid R code.
pub fn escape_r_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str(r"\\"),
            '"' => out.push_str(r#"\""#),
            '\n' => out.push_str(r"\n"),
            '\r' => out.push_str(r"\r"),
            '\t' => out.push_str(r"\t"),
            c => out.push(c),
        }
    }
    out
}

/// Build an R character vector (STRSXP) from the choices of a `MatchArg` type.
///
/// This is called by generated choices-helper C wrappers to provide the
/// choice list to `base::match.arg()` in the R wrapper.
pub fn choices_sexp<T: MatchArg>() -> SEXP {
    let choices = <T as MatchArg>::CHOICES;
    unsafe {
        let scope = ProtectScope::new();
        // Preserve the R_BlankString short-circuit: skips hash-lookup on the
        // interned empty CHARSXP for any empty choice strings.
        let vec = scope.alloc_strsxp(choices.len()).into_raw();
        for (i, s) in choices.iter().enumerate() {
            let charsxp = if s.is_empty() {
                SEXP::blank_string()
            } else {
                SEXP::charsxp(s)
            };
            vec.set_string_elt(i as R_xlen_t, charsxp);
        }
        vec
    }
}

/// Map a `SexpError` from a string `TryFromSexp` conversion into a `MatchArgError`.
///
/// Only `Type` and `Length` are produced by the `&str`/`Option<&str>`/
/// `Vec<Option<&str>>` conversions we delegate to — other variants are
/// unreachable in this context.
fn sexp_err_to_match_arg_err(e: SexpError) -> MatchArgError {
    match e {
        SexpError::Type(t) => MatchArgError::InvalidType(t.actual),
        SexpError::Length(l) => MatchArgError::InvalidLength(l.actual),
        other => unreachable!("unexpected SexpError from string conversion: {other}"),
    }
}

/// Extract a single choice from a factor SEXP (INTSXP with `levels` attribute).
fn factor_elt_to_choice<T: MatchArg>(sexp: SEXP) -> Result<T, MatchArgError> {
    let len = sexp.len();
    if len != 1 {
        return Err(MatchArgError::InvalidLength(len));
    }
    let idx = sexp.integer_elt(0);
    if idx == i32::MIN {
        // NA_integer_
        return Err(MatchArgError::IsNa);
    }
    let levels = sexp.get_levels();
    // R factor indices are 1-based.
    let level_idx = (idx - 1) as R_xlen_t;
    if level_idx < 0 || level_idx >= levels.len() as R_xlen_t {
        return Err(MatchArgError::NoMatch {
            input: format!("<factor index {}>", idx),
            choices: <T as MatchArg>::CHOICES,
        });
    }
    let charsxp = levels.string_elt(level_idx);
    // UTF-8 locale asserted at package init — charsxp_to_str is safe.
    match_choice::<T>(unsafe { charsxp_to_str(charsxp) })
}

/// Extract a single string from an R SEXP and match it against a `MatchArg` type.
///
/// Used by the generated `TryFromSexp for T` implementation (single-value `match.arg`).
pub fn match_arg_from_sexp<T: MatchArg>(sexp: SEXP) -> Result<T, MatchArgError> {
    // NIL → default (first choice), matching `base::match.arg()`.
    if sexp.type_of() == SEXPTYPE::NILSXP {
        return T::from_choice(<T as MatchArg>::CHOICES[0]).ok_or_else(|| MatchArgError::NoMatch {
            input: String::new(),
            choices: <T as MatchArg>::CHOICES,
        });
    }

    // Factors (INTSXP with STRSXP levels attribute): extract the level label.
    if sexp.is_factor() {
        return factor_elt_to_choice::<T>(sexp);
    }

    // STRSXP length-1 path — delegate type/length checks to Option<&str>.
    // NIL was handled above, so `None` here means NA_character_.
    let input = <Option<&'static str> as TryFromSexp>::try_from_sexp(sexp)
        .map_err(sexp_err_to_match_arg_err)?
        .ok_or(MatchArgError::IsNa)?;

    match_choice::<T>(input)
}

/// Optional form of [`match_arg_from_sexp`] for `Option<T>` parameters (#1473).
///
/// `NULL` is `None` (the R formal of an `Option<T>` choice parameter defaults
/// to `NULL`, meaning no choice was made); anything else goes through the same
/// exact-or-partial matching as the plain type and becomes `Some`.
///
/// Used by the generated C wrapper for `#[miniextendr(match_arg)] x: Option<T>`
/// parameters instead of `TryFromSexp`, because `Option<T>` cannot carry a
/// `TryFromSexp` impl for a downstream enum (orphan rule) and a `T: MatchArg`
/// blanket would collide with the newtype blanket in [`crate::newtype`].
pub fn match_arg_option_from_sexp<T: MatchArg>(sexp: SEXP) -> Result<Option<T>, MatchArgError> {
    if sexp.type_of() == SEXPTYPE::NILSXP {
        return Ok(None);
    }
    match_arg_from_sexp(sexp).map(Some)
}

/// Match a string against the choices of a `MatchArg` type (exact or partial).
fn match_choice<T: MatchArg>(input: &str) -> Result<T, MatchArgError> {
    // Exact match
    if let Some(val) = T::from_choice(input) {
        return Ok(val);
    }

    // Unique partial match (like R's match.arg)
    let mut matches: Vec<(usize, &'static str)> = Vec::new();
    for (i, choice) in <T as MatchArg>::CHOICES.iter().enumerate() {
        if choice.starts_with(input) {
            matches.push((i, choice));
        }
    }

    match matches.len() {
        1 => T::from_choice(matches[0].1).ok_or(MatchArgError::NoMatch {
            input: input.to_string(),
            choices: <T as MatchArg>::CHOICES,
        }),
        _ => Err(MatchArgError::NoMatch {
            input: input.to_string(),
            choices: <T as MatchArg>::CHOICES,
        }),
    }
}

/// Convert a `Vec<T: MatchArg>` to an R character vector (STRSXP).
///
/// Each element is written as its canonical choice string via [`MatchArg::to_choice`].
/// Empty choice strings are stored as `R_BlankString` (parity with [`choices_sexp`]).
///
/// Called by the [`MatchArg`]→[`IntoRVecElement`](crate::newtype::IntoRVecElement)
/// bridge below, which backs `IntoR for Vec<MyEnum>`.
pub fn match_arg_vec_into_sexp<T: MatchArg>(values: Vec<T>) -> SEXP {
    unsafe {
        let scope = ProtectScope::new();
        // Preserve the R_BlankString short-circuit: skips hash-lookup on the
        // interned empty CHARSXP for any empty choice strings.
        let vec = scope.alloc_strsxp(values.len()).into_raw();
        for (i, v) in values.into_iter().enumerate() {
            let s = v.to_choice();
            let charsxp = if s.is_empty() {
                SEXP::blank_string()
            } else {
                SEXP::charsxp(s)
            };
            vec.set_string_elt(i as R_xlen_t, charsxp);
        }
        vec
    }
}

/// Bridge: every [`MatchArg`] type is an [`IntoRVecElement`](crate::newtype::IntoRVecElement),
/// so `Vec<MyEnum>` converts to an R character vector (STRSXP) via
/// [`match_arg_vec_into_sexp`].
///
/// `IntoR for Vec<T>` has a single blanket slot (see [`crate::newtype`]). Routing
/// `MatchArg` through `IntoRVecElement` lets `#[derive(IntoR)]` newtypes share
/// that slot — a newtype implements `IntoRVecElement` concretely in its own crate,
/// which coexists with this bridge because a local newtype is provably not
/// `MatchArg` (deriving both `MatchArg` and `IntoR` on one type is an E0119
/// coherence error, by design). Stable Rust has no negative trait bounds, so a
/// *second* `impl<T: …> IntoR for Vec<T>` blanket would conflict directly; this
/// indirection is what avoids that.
impl<T: MatchArg> crate::newtype::IntoRVecElement for T {
    fn elements_into_sexp(values: Vec<Self>) -> SEXP {
        match_arg_vec_into_sexp(values)
    }
}

/// Extract multiple strings from an R SEXP (STRSXP) and match each against
/// the choices of a `MatchArg` type.
///
/// Used by the generated C wrapper for `match_arg + several_ok` parameters
/// (`match.arg` with `several.ok = TRUE`).
///
/// NULL input returns all variants. The R prelude's strict `several_ok`
/// helper maps `NULL` the same way and rejects any element that matches no
/// choice before the value reaches this function, so the per-element check
/// here is the second line of defence, not the only one (#1472).
///
/// Note: factors (INTSXP) are not handled here — the R wrapper coerces factors
/// to character before the `.Call()` boundary.
pub fn match_arg_vec_from_sexp<T: MatchArg>(sexp: SEXP) -> Result<Vec<T>, MatchArgError> {
    // NIL → all choices (match.arg default with several.ok = TRUE).
    if sexp.type_of() == SEXPTYPE::NILSXP {
        return <T as MatchArg>::CHOICES
            .iter()
            .map(|c| match_choice::<T>(c))
            .collect();
    }

    // STRSXP path — delegate type check + per-element NA handling
    // to Vec<Option<&str>>. `None` means NA_character_ (IsNa error).
    <Vec<Option<&'static str>> as TryFromSexp>::try_from_sexp(sexp)
        .map_err(sexp_err_to_match_arg_err)?
        .into_iter()
        .map(|opt| opt.ok_or(MatchArgError::IsNa).and_then(match_choice::<T>))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::escape_r_string;

    #[test]
    fn escapes_backslash_and_quote() {
        assert_eq!(escape_r_string(r#"say "hi""#), r#"say \"hi\""#);
        assert_eq!(escape_r_string(r"c:\path"), r"c:\\path");
    }

    #[test]
    fn escapes_control_characters() {
        assert_eq!(escape_r_string("line1\nline2"), r"line1\nline2");
        assert_eq!(escape_r_string("tab\there"), r"tab\there");
        assert_eq!(escape_r_string("cr\rlf"), r"cr\rlf");
    }

    #[test]
    fn passes_through_plain_strings() {
        assert_eq!(escape_r_string("Fast"), "Fast");
        assert_eq!(escape_r_string("it's"), "it's");
        assert_eq!(escape_r_string(""), "");
    }
}
