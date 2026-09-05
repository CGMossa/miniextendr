//! Direct R serialization via serde (no JSON intermediate).
//!
//! This module provides efficient type-preserving conversions between Rust types
//! and native R objects using serde's `Serialize` and `Deserialize` traits.
//!
//! # Features
//!
//! Enable this module with the `serde` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["serde"] }
//! ```
//!
//! # Type Mappings
//!
//! ## Serialization (Rust → R)
//!
//! | Rust Type | R Type |
//! |-----------|--------|
//! | `bool` | `logical(1)` |
//! | `i8/i16/i32` | `integer(1)` |
//! | `i64/u64/f32/f64` | `numeric(1)` |
//! | `String/&str` | `character(1)` |
//! | `Option<T>::None` | NA or NULL |
//! | `Vec<primitive>` | atomic vector (smart dispatch) |
//! | `Vec<struct>` | list of lists |
//! | `HashMap<String, T>` | named list |
//! | `struct { fields }` | named list |
//! | `()` / unit struct | NULL |
//! | unit enum variant | character scalar |
//! | data enum variant | tagged list `list(tag = value)` |
//!
//! ## Deserialization (R → Rust)
//!
//! | R Type | Rust Type |
//! |--------|-----------|
//! | `logical(1)` | `bool` |
//! | `integer(1)` | `i32` |
//! | `numeric(1)` | `f64` |
//! | `character(1)` | `String` |
//! | NA values | `Option<T>::None` |
//! | atomic vectors | `Vec<primitive>` or `Box<[primitive]>` |
//! | lists | `Vec<T>` or struct |
//! | named lists | struct or `HashMap<String, T>` |
//! | NULL | `()` or `Option<T>::None` |
//!
//! # Smart Vec Dispatch
//!
//! When serializing `Vec<T>`, the serializer uses smart dispatch:
//!
//! - `Vec<i32>` → `integer` vector
//! - `Vec<f64>` → `numeric` vector
//! - `Vec<bool>` → `logical` vector
//! - `Vec<String>` → `character` vector
//! - `Vec<struct>` → list of lists
//!
//! # NA Roundtrip
//!
//! NA values are preserved through `Option<T>`:
//!
//! ```rust,ignore
//! // Serialization
//! let v: Option<i32> = None;
//! // → NA_integer_
//!
//! // Deserialization
//! let v: Option<i32> = from_r(na_integer_sexp)?;
//! assert!(v.is_none());
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use serde::{Serialize, Deserialize};
//! use miniextendr_api::serde::{RSerializeNative, RDeserializeNative};
//!
//! #[derive(Serialize, Deserialize, Clone, ExternalPtr)]
//! struct Point {
//!     x: f64,
//!     y: f64,
//! }
//!
//! #[miniextendr]
//! impl RSerializeNative for Point {}
//!
//! #[miniextendr]
//! impl RDeserializeNative for Point {}
//! ```
//!
//! In R:
//!
//! ```r
//! # Create a Point
//! p <- Point$new(1.0, 2.0)
//!
//! # Serialize to R list
//! data <- p$to_r()
//! # list(x = 1.0, y = 2.0)
//!
//! # Deserialize from R list
//! p2 <- Point$from_r(list(x = 3.0, y = 4.0))
//! p2$x  # 3.0
//! p2$y  # 4.0
//! ```
//!
//! # Remote Derive for External Types
//!
//! When you need to serialize types from external crates that don't implement
//! `Serialize`/`Deserialize`, use serde's [remote derive] pattern.
//!
//! [remote derive]: https://serde.rs/remote-derive.html
//!
//! ## Basic Pattern
//!
//! 1. Define a "shadow" type that mirrors the external type's structure
//! 2. Add `#[serde(remote = "ExternalType")]` to it
//! 3. Use `#[serde(with = "ShadowType")]` on fields containing the external type
//!
//! ```rust,ignore
//! use serde::{Serialize, Deserialize};
//! use miniextendr_api::serde::to_r;
//!
//! // External type you don't control (e.g., from another crate)
//! // pub struct Duration { secs: u64, nanos: u32 }
//!
//! // Define a shadow type for serialization
//! #[derive(Serialize, Deserialize)]
//! #[serde(remote = "std::time::Duration")]
//! struct DurationDef {
//!     #[serde(getter = "std::time::Duration::as_secs")]
//!     secs: u64,
//!     #[serde(getter = "std::time::Duration::subsec_nanos")]
//!     nanos: u32,
//! }
//!
//! // Use the shadow type in your serializable struct
//! #[derive(Serialize)]
//! struct TimingData {
//!     name: String,
//!     #[serde(with = "DurationDef")]
//!     elapsed: std::time::Duration,
//! }
//!
//! #[miniextendr]
//! fn get_timing() -> SEXP {
//!     let data = TimingData {
//!         name: "benchmark".into(),
//!         elapsed: std::time::Duration::new(5, 123_000_000),
//!     };
//!     to_r(&data).unwrap()
//!     // Returns: list(name = "benchmark", elapsed = list(secs = 5, nanos = 123000000))
//! }
//! ```
//!
//! ## Standalone Conversion
//!
//! To convert an external type directly (not as a field), wrap it in a newtype:
//!
//! ```rust,ignore
//! // Newtype wrapper for standalone serialization
//! #[derive(Serialize)]
//! struct DurationR(#[serde(with = "DurationDef")] std::time::Duration);
//!
//! #[miniextendr]
//! fn duration_to_r(secs: f64) -> SEXP {
//!     let dur = std::time::Duration::from_secs_f64(secs);
//!     to_r(&DurationR(dur)).unwrap()
//! }
//! ```
//!
//! ## Implementing IntoR via Remote Derive
//!
//! To make external types work seamlessly with `IntoR`:
//!
//! ```rust,ignore
//! use miniextendr_api::into_r::IntoR;
//!
//! // Wrapper that implements IntoR
//! struct DurationR(std::time::Duration);
//!
//! // Shadow type for serde
//! #[derive(Serialize)]
//! #[serde(remote = "std::time::Duration")]
//! struct DurationDef {
//!     #[serde(getter = "std::time::Duration::as_secs")]
//!     secs: u64,
//!     #[serde(getter = "std::time::Duration::subsec_nanos")]
//!     nanos: u32,
//! }
//!
//! // Helper for serialization
//! #[derive(Serialize)]
//! struct DurationSer<'a>(#[serde(with = "DurationDef")] &'a std::time::Duration);
//!
//! impl IntoR for DurationR {
//!     fn into_sexp(self) -> SEXP {
//!         miniextendr_api::serde::to_r(&DurationSer(&self.0))
//!             .expect("Duration serialization failed")
//!     }
//! }
//!
//! // Now DurationR can be returned from #[miniextendr] functions
//! #[miniextendr]
//! fn make_duration(secs: f64) -> DurationR {
//!     DurationR(std::time::Duration::from_secs_f64(secs))
//! }
//! ```
//!
//! # Comparison with `serde_json` Feature
//!
//! | Feature | `serde_json` | `serde` (Native) |
//! |---------|--------------|------------------|
//! | Intermediate format | JSON string | None |
//! | Type preservation | No (all numbers → f64) | Yes (i32 stays i32) |
//! | NA handling | Limited | Full support |
//! | Performance | Extra parse/stringify | Direct conversion |
//! | External tools | JSON interop | R-native only |
//!
//! Use native `serde` when:
//! - You need efficient Rust ↔ R conversion
//! - Type preservation matters (integers vs doubles)
//! - You want NA/NULL handling
//!
//! Use `serde_json` when:
//! - You need JSON string output for files/APIs
//! - You want to share data with non-R systems
//! - You need JSON inspection/manipulation

pub mod columnar;
pub mod dataframe_de;
mod de;
mod error;
#[cfg(feature = "serde_json")]
pub mod json_string;
pub mod rvalue_ser;
mod ser;
mod traits;

// Re-export the serde crate for convenience
pub use ::serde::{Deserialize, Serialize};

pub use columnar::{
    DataFrameShape, DispatchNames, ResultShape, RootedSentinel, SerdeRowBuilder, SplitResults,
    SplitShape, TypeSpec, dispatch_to_dataframes, hashmap_to_dataframe, iter_to_dataframe,
    map_to_dataframe, result_to_dataframe, vec_to_dataframe, vec_to_dataframe_flatten_enums,
    vec_to_dataframe_flatten_enums_with_tags, vec_to_dataframe_split,
};
#[cfg(feature = "rayon")]
pub use columnar::{par_iter_to_dataframe, par_iter_to_dataframe_growing};
pub use dataframe_de::{
    BorrowedRows, SerdeRows, dataframe_to_vec, dataframe_to_vec_borrowed,
    dataframe_to_vec_collated, dataframe_to_vec_with_enum_tags,
    dataframe_to_vec_with_struct_fields, with_dataframe_rows,
};
pub use de::RDeserializer;
pub use error::RSerdeError;
#[cfg(feature = "serde_json")]
pub use json_string::{AsJson, AsJsonPretty, AsJsonVec, FromJson};
pub use rvalue_ser::{RValueSerializer, to_rvalue};
pub use ser::RSerializer;
pub use traits::{AsSerialize, RDeserializeNative, RSerializeNative, from_r, to_r};
