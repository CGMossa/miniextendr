//! Probes for struct shapes that CANNOT be lowered to R data via serde, or
//! that lower lossily. Each probe reports the actual runtime outcome so tests
//! assert observed behavior, not doc claims.
//!
//! The fourth class of unserializable struct is not representable here at
//! all: a struct holding a raw `SEXP` fails to compile under
//! `#[derive(Serialize)]` (no `Serialize` impl — nothing to lower). That is
//! the compile-time boundary; these probes cover the runtime boundary.
//!
//! `ExternalPtr<T>` fields used to sit in the same compile-time class, but
//! are RECTIFIED by the serde pass-through
//! (`miniextendr-api/src/serde/externalptr.rs`): when `T: Serialize` the
//! handle lowers by value as the pointee's encoding, and deserialization
//! rebuilds a fresh live handle. `ExternalPtr<T>` where `T` is not
//! `Serialize` still fails to compile. See `HandleHolder` below.

use crate::serde::{Deserialize, Serialize};
use miniextendr_api::prelude::SEXP;
use miniextendr_api::serde::{RSerdeError, from_r, to_r};
use miniextendr_api::{ExternalPtr, miniextendr};
use std::collections::HashMap;

/// Runtime-unserializable: serde's `Serializer` trait ships default
/// `serialize_i128`/`serialize_u128` hooks that error unless a serializer
/// opts in; `RSerializer` does not.
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct Hostile128 {
    pub big: u128,
}

/// Runtime-unserializable: R named lists require string names, so non-string
/// map keys must fail (`RSerdeError::NonStringKey`).
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct HostileIntKeys {
    pub by_id: HashMap<i32, String>,
}

/// Serializes but corrupts: `u64` lowers to REALSXP above i32 range, and
/// doubles are exact only up to 2^53. Values above that serialize AND
/// re-read without error — as a different number.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct HostileBigU64 {
    pub id: u64,
}

/// Both-None struct for pinning the serde None output encoding (NULL vs NA).
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct HostileNones {
    pub note: Option<String>,
    pub n: Option<i32>,
}

/// Pointee for the ExternalPtr serde pass-through.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ExternalPtr)]
#[serde(crate = "crate::serde")]
pub struct InnerPayload {
    pub name: String,
    pub score: f64,
}

/// Handle-holding struct — serializable by VALUE via the pass-through impl:
/// the `handle` field lowers as the pointee's named list; re-reading builds
/// a fresh live handle around the reconstructed pointee.
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct HandleHolder {
    pub label: String,
    pub handle: ExternalPtr<InnerPayload>,
}

/// Attempts to serialize a u128 field; reports Ok/Err.
#[miniextendr]
pub fn probe_serde_u128() -> String {
    let v = Hostile128 { big: u128::MAX };
    match to_r(&v) {
        Ok(_) => "Ok (unexpected)".to_string(),
        Err(e) => format!("Err: {e}"),
    }
}

/// Attempts to serialize a HashMap with i32 keys; reports Ok/Err.
#[miniextendr]
pub fn probe_serde_int_keys() -> String {
    let mut by_id = HashMap::new();
    by_id.insert(7, "seven".to_string());
    let v = HostileIntKeys { by_id };
    match to_r(&v) {
        Ok(_) => "Ok (unexpected)".to_string(),
        Err(e) => format!("Err: {e}"),
    }
}

/// Lowers `HostileBigU64 { id: 2^53 + 1 }` to R data. The value is already
/// corrupted in the returned SEXP (2^53 + 1 is not representable in f64).
#[miniextendr]
pub fn probe_serde_big_u64_to_r() -> Result<SEXP, String> {
    let v = HostileBigU64 {
        id: (1u64 << 53) + 1,
    };
    to_r(&v).map_err(|e: RSerdeError| e.to_string())
}

/// Re-reads lowered HostileBigU64 data; returns the id as a string so R can
/// compare it to the original without its own f64 precision limits.
/// @param data Named list produced by `probe_serde_big_u64_to_r()`.
#[miniextendr]
pub fn probe_serde_big_u64_read(data: SEXP) -> Result<String, String> {
    from_r::<HostileBigU64>(data)
        .map(|v| v.id.to_string())
        .map_err(|e: RSerdeError| e.to_string())
}

/// Lowers a struct whose Option fields are all None, so R can inspect what
/// the serde path emits for absence (NULL vs typed NA).
#[miniextendr]
pub fn probe_serde_none_to_r() -> Result<SEXP, String> {
    let v = HostileNones {
        note: None,
        n: None,
    };
    to_r(&v).map_err(|e: RSerdeError| e.to_string())
}

/// Re-reads HostileNones data; reports the parsed Options.
/// @param data Named list with `note` and `n` entries.
#[miniextendr]
pub fn probe_serde_none_read(data: SEXP) -> Result<String, String> {
    from_r::<HostileNones>(data)
        .map(|v| format!("note={:?} n={:?}", v.note, v.n))
        .map_err(|e: RSerdeError| e.to_string())
}

/// Lowers a live handle-holding struct to plain R data via the ExternalPtr
/// serde pass-through (by value: the `handle` entry is the pointee's list).
#[miniextendr]
pub fn probe_serde_handle_to_r() -> Result<SEXP, String> {
    let v = HandleHolder {
        label: "holder".to_string(),
        handle: ExternalPtr::new(InnerPayload {
            name: "inner".to_string(),
            score: 2.5,
        }),
    };
    to_r(&v).map_err(|e: RSerdeError| e.to_string())
}

/// Re-reads HandleHolder data, rebuilding a fresh live handle; reports the
/// pointee's fields read THROUGH the rebuilt handle.
/// @param data Named list produced by `probe_serde_handle_to_r()`.
#[miniextendr]
pub fn probe_serde_handle_read(data: SEXP) -> Result<String, String> {
    from_r::<HandleHolder>(data)
        .map(|v| {
            format!(
                "label={} name={} score={}",
                v.label, v.handle.name, v.handle.score
            )
        })
        .map_err(|e: RSerdeError| e.to_string())
}

/// Sentinel aliasing: R's INTSXP has no representation for i32::MIN — that
/// bit pattern IS NA_integer_. `scalar_integer` is raw `Rf_ScalarInteger`
/// with no guard, so serializing i32::MIN silently produces NA.
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct HostileI32Min {
    pub x: i32,
}

/// Option variant of [`HostileI32Min`] for the Some -> None flip probe.
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct HostileOptI32 {
    pub x: Option<i32>,
}

/// Option<f64> carrier for the NA_real_ payload aliasing probe.
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct HostileOptF64 {
    pub x: Option<f64>,
}

/// Lowers `{ x: i32::MIN }`; R receives `x = NA_integer_` without error.
#[miniextendr]
pub fn probe_serde_i32_min_to_r() -> Result<SEXP, String> {
    to_r(&HostileI32Min { x: i32::MIN }).map_err(|e: RSerdeError| e.to_string())
}

/// Re-reads i32::MIN data as a required i32; reports the outcome
/// (expected: unexpected-NA error — the value is unrecoverable).
#[miniextendr]
pub fn probe_serde_i32_min_read(data: SEXP) -> String {
    match from_r::<HostileI32Min>(data) {
        Ok(v) => format!("Ok: x={}", v.x),
        Err(e) => format!("Err: {e}"),
    }
}

/// Re-reads i32::MIN data as Option<i32>; reports the outcome
/// (expected: Some(i32::MIN) has silently become None).
#[miniextendr]
pub fn probe_serde_i32_min_read_opt(data: SEXP) -> String {
    match from_r::<HostileOptI32>(data) {
        Ok(v) => format!("Ok: x={:?}", v.x),
        Err(e) => format!("Err: {e}"),
    }
}

/// Macro output path: a bare `-> i32` return of i32::MIN reaches R as
/// NA_integer_ too — the aliasing is IntoR-level, not serde-specific.
#[miniextendr]
pub fn probe_macro_i32_min() -> i32 {
    i32::MIN
}

/// R's NA_real_ IS a NaN with payload 1954, so a Rust f64 carrying exactly
/// those bits round-trips Some -> (R sees NA) -> None, while an ordinary NaN
/// survives as Some(NaN).
#[miniextendr]
pub fn probe_serde_na_real_payload() -> String {
    use miniextendr_api::altrep_traits::NA_REAL;
    let na = to_r(&HostileOptF64 { x: Some(NA_REAL) })
        .map_err(|e: RSerdeError| e.to_string())
        .and_then(|s| from_r::<HostileOptF64>(s).map_err(|e: RSerdeError| e.to_string()));
    let nan = to_r(&HostileOptF64 { x: Some(f64::NAN) })
        .map_err(|e: RSerdeError| e.to_string())
        .and_then(|s| from_r::<HostileOptF64>(s).map_err(|e: RSerdeError| e.to_string()));
    format!(
        "na_payload={:?} plain_nan={:?}",
        na.map(|v| v.x),
        nan.map(|v| v.x)
    )
}

/// JSON contrast: serde_json writes u64 natively, so the JSON-string path
/// preserves values beyond 2^53 that the native SEXP path corrupts.
#[cfg(feature = "serde_json")]
#[miniextendr]
pub fn probe_json_big_u64_to_r() -> miniextendr_api::serde::AsJson<HostileBigU64> {
    miniextendr_api::serde::AsJson(HostileBigU64 {
        id: (1u64 << 53) + 1,
    })
}

/// Re-reads the JSON string; the id survives exactly.
/// @param data JSON string produced by `probe_json_big_u64_to_r()`.
#[cfg(feature = "serde_json")]
#[miniextendr]
pub fn probe_json_big_u64_read(data: miniextendr_api::serde::FromJson<HostileBigU64>) -> String {
    data.0.id.to_string()
}
