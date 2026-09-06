//! Compile-pass test: raw identifiers (`r#keyword`) in every user-facing name position.
//!
//! Rust keywords are ordinary R names (`type`, `where`, `match`, ...), so users spell
//! them as raw identifiers. `Display` for `syn::Ident` keeps the `r#` prefix, which
//! used to leak into C symbols (`C_pkg_r#match` — a proc-macro panic), R formals, and
//! synthesized bindings (`__storage_r#where`). Every name-producing site now routes
//! through `naming::ident_name` / `naming::unraw`, and derive-generated struct fields
//! named after keyword columns come back raw via `naming::name_ident`.
//!
//! Covered: free fn name + params (incl. `default`, `match_arg`/`choices`, `coerce`,
//! slices, named dots), inherent impl methods + params across r6/s3/env/s4/s7 (incl.
//! an R6 `set_*` setter and a `pub` sidecar field), trait methods + params with env
//! and s3 trait impls, and the `IntoList` / `TryFromList` / `DataFrameRow` (struct and
//! enum) derives.

#![allow(dead_code)]
use miniextendr_macros::{DataFrameRow, ExternalPtr, IntoList, TryFromList, miniextendr};

/// Free function whose Rust name and parameter names are all keywords.
/// @param where Where.
/// @param type Type.
/// @param ref Ref.
#[miniextendr]
pub fn r#match(r#where: &str, r#type: i32, #[miniextendr(default = "1L")] r#ref: i32) -> String {
    format!("{where}-{type}-{ref}")
}

/// `choices()` on a raw parameter.
/// @param type Type.
#[miniextendr]
pub fn raw_choices(#[miniextendr(choices("fast", "slow"))] r#type: String) -> String {
    r#type
}

#[miniextendr(match_arg)]
#[derive(Copy, Clone)]
pub enum RawMode {
    Fast,
    Slow,
}

/// `match_arg` on a raw parameter (choices helper + placeholder keyed by the plain name).
/// @param type Type.
#[miniextendr]
pub fn raw_match_arg(#[miniextendr(match_arg)] r#type: RawMode) -> i32 {
    r#type as i32
}

/// Slice + coerce on raw parameters (exercises `__storage_*` / `__vec_*` synth idents).
/// @param where Where.
/// @param type Type.
#[miniextendr]
pub fn raw_slices(r#where: &[f64], #[miniextendr(coerce)] r#type: u16) -> f64 {
    r#where.iter().sum::<f64>() + f64::from(r#type)
}

/// Raw named dots.
#[miniextendr]
pub fn raw_dots(r#type: i32, r#dyn: ...) -> i32 {
    r#type + r#dyn.len() as i32
}

#[derive(ExternalPtr)]
pub struct RawR6 {
    /// `#[r_data]` on a keyword field: sidecar accessors `RawR6_get_type` / `RawR6_set_type`.
    #[r_data]
    pub r#type: i32,
}

#[miniextendr(r6)]
impl RawR6 {
    pub fn new(r#type: i32) -> Self {
        Self { r#type }
    }
    pub fn r#type(&self) -> i32 {
        self.r#type
    }
    pub fn set_type(&mut self, r#type: i32) {
        self.r#type = r#type;
    }
    pub fn r#move(&mut self, r#where: i32) {
        self.r#type += r#where;
    }
}

#[derive(ExternalPtr)]
pub struct RawS3 {
    v: i32,
}

#[miniextendr(s3)]
impl RawS3 {
    pub fn new(r#type: i32) -> Self {
        Self { v: r#type }
    }
    pub fn r#type(&self) -> i32 {
        self.v
    }
    pub fn r#use(&self, r#mod: i32) -> i32 {
        self.v % r#mod
    }
}

#[derive(ExternalPtr)]
pub struct RawEnv {
    v: i32,
}

#[miniextendr(env)]
impl RawEnv {
    pub fn new(r#type: i32) -> Self {
        Self { v: r#type }
    }
    pub fn r#type(&self) -> i32 {
        self.v
    }
}

#[derive(ExternalPtr)]
pub struct RawS4 {
    v: i32,
}

#[miniextendr(s4)]
impl RawS4 {
    pub fn new(r#type: i32) -> Self {
        Self { v: r#type }
    }
    pub fn r#type(&self) -> i32 {
        self.v
    }
}

#[derive(ExternalPtr)]
pub struct RawS7 {
    v: i32,
}

#[miniextendr(s7)]
impl RawS7 {
    pub fn new(r#type: i32) -> Self {
        Self { v: r#type }
    }
    pub fn r#type(&self) -> i32 {
        self.v
    }
}

#[miniextendr]
pub trait RawTrait {
    fn r#loop(&self) -> i32;
    fn r#use(&self, r#where: i32) -> i32;
}

#[miniextendr(env)]
impl RawTrait for RawEnv {
    fn r#loop(&self) -> i32 {
        self.v
    }
    fn r#use(&self, r#where: i32) -> i32 {
        self.v + r#where
    }
}

#[miniextendr(s3)]
impl RawTrait for RawS3 {
    fn r#loop(&self) -> i32 {
        self.v
    }
    fn r#use(&self, r#where: i32) -> i32 {
        self.v + r#where
    }
}

#[derive(Clone, IntoList, DataFrameRow)]
pub struct RawRow {
    pub r#type: String,
    pub r#where: i32,
}

#[derive(Clone, DataFrameRow)]
pub enum RawRowEnum {
    A { r#type: i32 },
    B { r#where: f64 },
}

#[derive(IntoList, TryFromList)]
pub struct RawList {
    pub r#type: i32,
    pub r#match: String,
}

fn main() {}
