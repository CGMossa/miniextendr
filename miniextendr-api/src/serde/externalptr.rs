//! Serde pass-through for [`ExternalPtr<T>`]: serialize the pointee by value,
//! deserialize by rebuilding a fresh self-rooted handle.
//!
//! This makes handle-holding structs serde-lowerable: a field of type
//! `ExternalPtr<T>` where `T: Serialize` contributes the pointee's own
//! encoding (a named list for a struct `T`), indistinguishable from a plain
//! `T` field on the R side. Deserialization builds a brand-new
//! [`ExternalPtr::new`] around the reconstructed `T`.
//!
//! # Semantics — snapshot, not reference
//!
//! - **By value.** The lowered form is a copy of the pointee's state at
//!   serialization time. Later mutations through the original handle are not
//!   reflected in the data.
//! - **Identity collapses.** Two fields aliasing the same handle serialize as
//!   two independent copies and deserialize as two distinct handles. R's own
//!   serialization preserves reference identity for EXTPTRSXP within one
//!   `serialize()` call (via its ref table); this pass-through does not.
//! - **Liveness.** A held `ExternalPtr<T>` is live by construction
//!   (`TryFromSexp` rejects null/foreign pointers before user code runs), so
//!   serialization normally cannot observe a dead handle. The one exception
//!   is a cleared handle, which reports a serde error rather than panicking.
//!
//! # Threading
//!
//! Serialization reads the pointee through the cached pointer — plain Rust
//! memory, no R API. Deserialization allocates an EXTPTRSXP via
//! [`ExternalPtr::new`] and must run on the R main thread (true for the
//! `from_r`/`RDeserializer` path, which executes inside `.Call`).

use serde::de::{Deserialize, Deserializer};
use serde::ser::{Error as _, Serialize, Serializer};

use crate::externalptr::{ExternalPtr, TypedExternal};

impl<T: TypedExternal + Serialize> Serialize for ExternalPtr<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match Self::as_ref(self) {
            Some(inner) => inner.serialize(serializer),
            None => Err(S::Error::custom(
                "cannot serialize a null or cleared ExternalPtr",
            )),
        }
    }
}

impl<'de, T: TypedExternal + Deserialize<'de>> Deserialize<'de> for ExternalPtr<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(ExternalPtr::new)
    }
}
