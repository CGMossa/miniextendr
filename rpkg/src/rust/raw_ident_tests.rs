//! Raw identifiers (`r#keyword`) in every user-facing name position.
//!
//! Rust keywords are ordinary R names (`type`, `where`, `match`, ...). The `r#`
//! prefix is Rust surface syntax only and must vanish from every generated name:
//! R wrapper names and formals, `@param` docs, class methods and active bindings,
//! sidecar accessors, list element names, and data-frame columns. Companion
//! testthat file: `test-raw-ident.R`. Compile-only coverage of the remaining
//! class systems and derives lives in
//! `miniextendr-macros/tests/ui/pass/raw_identifiers.rs`.

use miniextendr_api::externalptr::RSidecar;
use miniextendr_api::list::{IntoList, List};
use miniextendr_api::{BuiltDataFrame, DataFrameRow, IntoDataFrame, miniextendr};

// region: free functions

/// Free function whose Rust name is a keyword; R sees `where()`.
/// @param x Integer input, returned doubled.
#[miniextendr]
pub fn r#where(x: i32) -> i32 {
    x * 2
}

/// Free function whose parameters are all keywords.
/// @param where Character filter.
/// @param type Integer type code.
/// @param ref Integer reference (defaults to `1L`).
#[miniextendr]
pub fn raw_ident_args(
    r#where: &str,
    r#type: i32,
    #[miniextendr(default = "1L")] r#ref: i32,
) -> String {
    format!("{where}-{type}-{ref}")
}

/// `choices(...)` (`match.arg`) on a keyword parameter.
/// @param type One of `"fast"` or `"slow"`.
#[miniextendr]
pub fn raw_ident_choice(#[miniextendr(choices("fast", "slow"))] r#type: String) -> String {
    r#type
}

/// Choice list for the `match_arg` keyword-parameter case.
#[miniextendr(match_arg)]
#[derive(Copy, Clone)]
pub enum RawIdentMode {
    Fast,
    Slow,
}

/// `match_arg` on a keyword parameter: the choices helper, the write-time
/// placeholder, and the `match.arg()` prelude are all keyed by the plain name.
#[miniextendr]
pub fn raw_ident_match_arg(#[miniextendr(match_arg)] r#type: RawIdentMode) -> String {
    match r#type {
        RawIdentMode::Fast => "fast".to_string(),
        RawIdentMode::Slow => "slow".to_string(),
    }
}

/// Per-parameter `coerce` on a keyword parameter (attribute keyed by the plain name).
/// @param type Integer scalar coerced to `u16`; negative values error.
#[miniextendr]
pub fn raw_ident_coerce(#[miniextendr(coerce)] r#type: u16) -> i32 {
    i32::from(r#type)
}

/// Keyword-named dots: R still sees plain `...`.
/// @param type Integer added to the number of extra arguments.
/// @param ... Extra arguments; only counted.
#[miniextendr]
pub fn raw_ident_dots(r#type: i32, r#dyn: ...) -> i32 {
    r#type + r#dyn.len() as i32
}
// endregion

// region: R6 class

/// R6 class whose methods, active binding, and parameters are keywords.
#[derive(miniextendr_api::ExternalPtr)]
pub struct RawIdentR6 {
    value: i32,
}

/// R6 class whose methods, active binding, and parameters are keywords.
/// @param type Integer initial value.
/// @field type Integer current value (active binding).
#[miniextendr(r6)]
impl RawIdentR6 {
    /// Creates the object.
    pub fn new(r#type: i32) -> Self {
        Self { value: r#type }
    }

    /// Active binding `obj$type`.
    #[miniextendr(r6(active))]
    pub fn r#type(&self) -> i32 {
        self.value
    }

    /// Adds `where` to the value and returns the new value: `obj$move(where = 2L)`.
    pub fn r#move(&mut self, r#where: i32) -> i32 {
        self.value += r#where;
        self.value
    }

    /// Value modulo `mod`: `obj$use(mod = 4L)`.
    pub fn r#use(&self, r#mod: i32) -> i32 {
        self.value % r#mod
    }
}
// endregion

// region: env class + sidecar

/// Env class with a keyword `#[r_data]` sidecar field, exposed as
/// `RawIdentEnv_get_type()` / `RawIdentEnv_set_type()`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct RawIdentEnv {
    #[r_data]
    _r: RSidecar,
    /// Keyword-named sidecar field.
    #[r_data]
    pub r#type: i32,
    base: i32,
}

/// Env class registration for `RawIdentEnv`; its methods are keywords too.
#[miniextendr(env)]
impl RawIdentEnv {
    /// Creates the object.
    /// @param type Integer initial sidecar value.
    /// @param base Integer base value for `loop()`.
    pub fn new(r#type: i32, base: i32) -> Self {
        Self {
            _r: RSidecar,
            r#type,
            base,
        }
    }

    /// `base` plus `where`: `RawIdentEnv$loop(obj, where = 1L)`.
    pub fn r#loop(&self, r#where: i32) -> i32 {
        self.base + r#where
    }
}
// endregion

// region: derives

/// Row type whose columns are keywords.
#[derive(Clone, miniextendr_api::IntoList, DataFrameRow)]
pub struct RawIdentRow {
    pub r#type: String,
    pub r#where: i32,
}

/// Data frame with columns `type` and `where`.
#[miniextendr]
pub fn raw_ident_df() -> BuiltDataFrame {
    vec![
        RawIdentRow {
            r#type: "a".into(),
            r#where: 1,
        },
        RawIdentRow {
            r#type: "b".into(),
            r#where: 2,
        },
    ]
    .into_dataframe()
    .unwrap()
}

/// Named list with elements `type` and `where`.
#[miniextendr]
pub fn raw_ident_list() -> List {
    RawIdentRow {
        r#type: "a".into(),
        r#where: 1,
    }
    .into_list()
}
// endregion
