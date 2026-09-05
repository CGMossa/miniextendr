//! Cross-session persistence fixture: an S7 class whose Rust state derives
//! `Serialize`/`Deserialize`.
//!
//! Answers the question "can a pointer-backed S7 object be saved to
//! .rds/.rda and read back in a NEW R session?" — exercised by
//! `rpkg/tests/testthat/test-s7-serde-cross-session.R`:
//!
//! - Saving the S7 object itself does NOT survive: R serialization nulls the
//!   ExternalPtr address (even within the same session), so the class shell
//!   round-trips but the first method call on the loaded object errors.
//! - The serde bridge DOES survive: `s7_persist_to_r()` lowers the full Rust
//!   state (including fields with no R analog) to plain R data, which
//!   `saveRDS`/`save` handle natively; `S7SerdePersist_from_r_data()` rebuilds
//!   a live object in the new session.

use crate::serde::{Deserialize, Serialize};
use miniextendr_api::prelude::SEXP;
use miniextendr_api::serde::{RSerdeError, from_r, to_r};
use miniextendr_api::{ExternalPtr, miniextendr};
use std::collections::BTreeMap;

/// S7-wrapped struct mixing R-native-representable fields (`String`,
/// `Vec<f64>`, `Option<String>`) with Rust-native ones (`u64`, `BTreeMap`).
/// All state lives behind the ExternalPtr — nothing is stored R-side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ExternalPtr)]
#[serde(crate = "crate::serde")]
pub struct S7SerdePersist {
    label: String,
    values: Vec<f64>,
    maybe_note: Option<String>,
    /// Rust-native: exceeds R's integer range (round-trips via double).
    id: u64,
    /// Rust-native: deterministic iteration order keeps serialization stable.
    lookup: BTreeMap<String, i32>,
}

/// S7 class for cross-session serde persistence tests.
#[miniextendr(s7, internal)]
impl S7SerdePersist {
    /// Creates a persistence fixture; `keys`/`counts` are zipped into the
    /// Rust-side lookup map.
    /// @param label Character scalar label.
    /// @param values Numeric vector payload.
    /// @param maybe_note Optional character note (NULL for none).
    /// @param keys Character vector of lookup keys.
    /// @param counts Integer vector of lookup values (parallel to keys).
    pub fn new(
        label: String,
        values: Vec<f64>,
        maybe_note: Option<String>,
        keys: Vec<String>,
        counts: Vec<i32>,
    ) -> Self {
        let lookup: BTreeMap<String, i32> = keys.into_iter().zip(counts).collect();
        S7SerdePersist {
            label,
            values,
            maybe_note,
            id: 0xDEAD_BEEF,
            lookup,
        }
    }

    /// Returns the label.
    pub fn s7_persist_label(&self) -> String {
        self.label.clone()
    }

    /// Returns the numeric payload.
    pub fn s7_persist_values(&self) -> Vec<f64> {
        self.values.clone()
    }

    /// Returns the optional note. Note: as a class METHOD, `Option::None`
    /// raises the typed NONE_ERR condition instead of returning NA — the
    /// method path diverges from the standalone-fn absence contract (#1415).
    pub fn s7_persist_note(&self) -> Option<String> {
        self.maybe_note.clone()
    }

    /// Returns the u64 id formatted as a string (value exceeds R's i32).
    pub fn s7_persist_id(&self) -> String {
        self.id.to_string()
    }

    /// Looks up a key in the Rust-side map. A missing key raises the typed
    /// NONE_ERR condition ("returned no value") rather than returning NA —
    /// the method path diverges from the standalone-fn absence contract
    /// (#1415); the cross-session test pins this behavior.
    /// @param key Character scalar key.
    pub fn s7_persist_lookup_get(&self, key: String) -> Option<i32> {
        self.lookup.get(&key).copied()
    }

    /// Lowers the full Rust state to plain R data (a named list) via serde —
    /// safe to `saveRDS`/`save` and reload in another session.
    pub fn s7_persist_to_r(&self) -> Result<SEXP, String> {
        to_r(self).map_err(|e: RSerdeError| e.to_string())
    }

    /// Rebuilds a live object from serde-lowered R data; wraps into a classed
    /// S7 object exactly like the constructor does.
    /// @param data Named list produced by `s7_persist_to_r()`.
    pub fn from_r_data(data: SEXP) -> Result<Self, String> {
        from_r(data).map_err(|e: RSerdeError| e.to_string())
    }
}
