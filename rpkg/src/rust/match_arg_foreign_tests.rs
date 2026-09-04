//! `match.arg` choices sourced from an enum this crate does not own (#1436).
//!
//! `#[derive(MatchArg)]` can only be attached to an enum declared in a crate
//! that depends on miniextendr. When the enum belongs to a wrapped library,
//! the bridge wraps it in a newtype and implements the three traits the derive
//! would have generated (`MatchArg`, `TryFromSexp`, `IntoR`) by hand. Every
//! `match_arg` feature then works unchanged: the `match.arg()` prelude, the
//! choices spliced into the R formal default, `several_ok`, factor input, the
//! `Vec<T>` return path, and the auto-injected `@param` choices doc.
//!
//! `wrapped` below stands in for that foreign crate: plain Rust, no
//! miniextendr types anywhere.

use miniextendr_api::match_arg::MatchArg;
use miniextendr_api::{IntoR, SEXP, SexpError, TryFromSexp, miniextendr};

/// Stand-in for a library crate that knows nothing about R.
pub mod wrapped {
    /// Interpolation method of the wrapped library.
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub enum Interp {
        Linear,
        Cubic,
        Nearest,
    }
}

/// Bridge newtype over [`wrapped::Interp`] carrying the `match.arg` contract.
///
/// The three impls below are exactly what `#[derive(MatchArg)]` emits for an
/// owned enum; only the variant table is hand-written. Adding a variant to the
/// wrapped enum without updating `CHOICES` / `from_choice` / `to_choice` is a
/// non-exhaustive-match compile error in `to_choice`, so the two cannot drift
/// silently.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InterpChoice(pub wrapped::Interp);

impl MatchArg for InterpChoice {
    const CHOICES: &'static [&'static str] = &["linear", "cubic", "nearest"];

    fn from_choice(choice: &str) -> Option<Self> {
        let inner = match choice {
            "linear" => wrapped::Interp::Linear,
            "cubic" => wrapped::Interp::Cubic,
            "nearest" => wrapped::Interp::Nearest,
            _ => return None,
        };
        Some(InterpChoice(inner))
    }

    fn to_choice(self) -> &'static str {
        match self.0 {
            wrapped::Interp::Linear => "linear",
            wrapped::Interp::Cubic => "cubic",
            wrapped::Interp::Nearest => "nearest",
        }
    }
}

impl TryFromSexp for InterpChoice {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        miniextendr_api::match_arg_from_sexp(sexp).map_err(Into::into)
    }
}

impl IntoR for InterpChoice {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> SEXP {
        self.to_choice().into_sexp()
    }
}

// region: fixtures

/// Name the wrapped library's interpolation variant selected via `match.arg()`.
///
/// @param method Interpolation method.
#[miniextendr]
pub fn foreign_enum_interp(#[miniextendr(match_arg)] method: InterpChoice) -> String {
    format!("{:?}", method.0)
}

/// Echo one or more interpolation methods; `methods` is left undocumented so
/// the auto-injected `@param` choices text is exercised on a hand-written
/// `MatchArg` impl. Returns through the blanket `Vec<T: MatchArg>` path.
#[miniextendr]
pub fn foreign_enum_interps(
    #[miniextendr(match_arg, several_ok)] methods: Vec<InterpChoice>,
) -> Vec<InterpChoice> {
    methods
}

/// The wrapped library's own default, returned as its choice string.
#[miniextendr]
pub fn foreign_enum_default() -> InterpChoice {
    InterpChoice(wrapped::Interp::Cubic)
}

// endregion
