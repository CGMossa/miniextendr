//! Integration with the Apache Arrow columnar format.
//!
//! This module provides zero-copy (where possible) conversions between R
//! vectors/data.frames and Arrow arrays/RecordBatch.
//!
//! # Zero-Copy Conversions (R → Arrow)
//!
//! | R Type | Arrow Type | Method |
//! |--------|-----------|--------|
//! | `numeric` (REALSXP) | `Float64Array` | Zero-copy via shared buffer |
//! | `integer` (INTSXP) | `Int32Array` | Zero-copy via shared buffer |
//! | `raw` (RAWSXP) | `UInt8Array` | Zero-copy via shared buffer |
//!
//! # Copy Conversions
//!
//! | R Type | Arrow Type | Notes |
//! |--------|-----------|-------|
//! | `logical` (LGLSXP) | `BooleanArray` | R uses i32 per element, Arrow uses bit-packed |
//! | `character` (STRSXP) | `StringArray` | Different string representations |
//! | `data.frame` | `RecordBatch` | Column-wise conversion |
//!
//! # NA Handling
//!
//! R's NA values (`NA_integer_`, `NA_real_`, `NA_character_`) are converted to
//! Arrow's null bitmap. For zero-copy primitives, the NA sentinel values remain
//! in the data buffer — Arrow ignores them because the null bitmap marks those
//! positions as null.
//!
//! # Features
//!
//! Enable with `features = ["arrow"]`:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["arrow"] }
//! ```

use std::ops::Deref;
use std::sync::Arc;

pub use arrow_array::{
    self, Array, ArrayRef, BooleanArray, Date32Array, DictionaryArray, Float64Array, Int32Array,
    RecordBatch, StringArray, TimestampSecondArray, UInt8Array,
    types::{Date32Type, Float64Type, Int32Type, TimestampSecondType, UInt8Type},
};
pub use arrow_buffer;
pub use arrow_schema::{self, DataType, Field, Schema};

use arrow_array::types::ArrowPrimitiveType;

use crate::from_r::{SexpError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;
use crate::sys;
use crate::{R_xlen_t, RNativeType, SEXP, SEXPTYPE, SexpExt};

// region: RSourced trait — R buffer provenance for Arrow types

/// Trait for Arrow types that may be backed by R memory.
///
/// Types implementing this trait carry optional provenance information:
/// the original R SEXP whose data buffer backs the Arrow array. When
/// converting back to R, this enables zero-copy return of the original
/// vector instead of allocating and copying.
pub trait RSourced {
    /// The original R SEXP if this value is zero-copy from R.
    fn r_source(&self) -> Option<SEXP>;

    /// Whether nulls came from R sentinel values (NA_integer_, NA_real_).
    ///
    /// When true, the R vector's data buffer already contains the correct
    /// sentinel values and can be returned as-is. When false, Arrow
    /// operations may have added/changed nulls — the null bitmap must
    /// be materialized back as R sentinels before returning.
    fn nulls_from_sentinels(&self) -> bool;
}

// endregion

// region: RPrimitive<T> — R-backed primitive Arrow array

/// A primitive Arrow array that may be backed by R memory.
///
/// `RPrimitive<T>` wraps a [`PrimitiveArray<T>`](arrow_array::PrimitiveArray) and optionally carries the
/// source R SEXP. When the array came from R (via `TryFromSexp`), converting
/// back to R is zero-copy — the original SEXP is returned directly.
///
/// All Arrow APIs work transparently via `Deref<Target = PrimitiveArray<T>>`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::optionals::arrow_impl::{RPrimitive, Float64Type};
///
/// #[miniextendr]
/// pub fn passthrough(x: RPrimitive<Float64Type>) -> RPrimitive<Float64Type> {
///     x  // zero-copy round-trip: R REALSXP → Arrow → same REALSXP
/// }
///
/// #[miniextendr]
/// pub fn doubled(x: RPrimitive<Float64Type>) -> RPrimitive<Float64Type> {
///     let result: Float64Array = x.iter().map(|v| v.map(|f| f * 2.0)).collect();
///     RPrimitive::from_arrow(result)  // no R source → copies on IntoR
/// }
/// ```
pub struct RPrimitive<T: ArrowPrimitiveType> {
    array: arrow_array::PrimitiveArray<T>,
    source: Option<SEXP>,
    sentinel_nulls: bool,
}

impl<T: ArrowPrimitiveType> RPrimitive<T> {
    /// Wrap a computed Arrow array (no R source). IntoR will copy.
    pub fn from_arrow(array: arrow_array::PrimitiveArray<T>) -> Self {
        Self {
            array,
            source: None,
            sentinel_nulls: false,
        }
    }

    /// Wrap an Arrow array with a known R source SEXP.
    ///
    /// # Safety
    ///
    /// The caller must ensure `sexp` is a valid R vector whose data buffer
    /// backs `array` (i.e., the array was created via `sexp_to_arrow_buffer`).
    pub unsafe fn from_r(array: arrow_array::PrimitiveArray<T>, sexp: SEXP) -> Self {
        Self {
            array,
            source: Some(sexp),
            sentinel_nulls: true,
        }
    }

    /// Get the inner Arrow array, discarding provenance.
    pub fn into_inner(self) -> arrow_array::PrimitiveArray<T> {
        self.array
    }
}

impl<T: ArrowPrimitiveType> RSourced for RPrimitive<T> {
    fn r_source(&self) -> Option<SEXP> {
        self.source
    }
    fn nulls_from_sentinels(&self) -> bool {
        self.sentinel_nulls
    }
}

impl<T: ArrowPrimitiveType> Deref for RPrimitive<T> {
    type Target = arrow_array::PrimitiveArray<T>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl<T: ArrowPrimitiveType> AsRef<arrow_array::PrimitiveArray<T>> for RPrimitive<T> {
    #[inline]
    fn as_ref(&self) -> &arrow_array::PrimitiveArray<T> {
        &self.array
    }
}

impl<T: ArrowPrimitiveType> AsRef<dyn Array + 'static> for RPrimitive<T> {
    #[inline]
    fn as_ref(&self) -> &(dyn Array + 'static) {
        &self.array
    }
}

impl<T: ArrowPrimitiveType> std::fmt::Debug for RPrimitive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RPrimitive")
            .field("array", &self.array)
            .field("r_backed", &self.source.is_some())
            .finish()
    }
}

// endregion

// region: RStringArray — R-backed string Arrow array

/// A string Arrow array that may be backed by an R STRSXP.
///
/// R's STRSXP and Arrow's StringArray have incompatible memory layouts,
/// so R→Arrow always copies string data. However, for unmodified round-trips,
/// the original STRSXP is returned on IntoR without rebuilding it.
pub struct RStringArray {
    array: StringArray,
    source: Option<SEXP>,
}

impl RStringArray {
    /// Wrap a computed StringArray (no R source).
    pub fn from_arrow(array: StringArray) -> Self {
        Self {
            array,
            source: None,
        }
    }

    /// Wrap a StringArray with a known R source STRSXP.
    ///
    /// # Safety
    ///
    /// The caller must ensure `sexp` is a valid STRSXP that was the source
    /// for the StringArray's data (i.e., the array was built from this STRSXP).
    pub unsafe fn from_r(array: StringArray, sexp: SEXP) -> Self {
        Self {
            array,
            source: Some(sexp),
        }
    }

    /// Get the inner StringArray, discarding provenance.
    pub fn into_inner(self) -> StringArray {
        self.array
    }
}

impl RSourced for RStringArray {
    fn r_source(&self) -> Option<SEXP> {
        self.source
    }
    fn nulls_from_sentinels(&self) -> bool {
        // String arrays are always copies from R, so this flag is about
        // whether the *source STRSXP* can be returned as-is.
        self.source.is_some()
    }
}

impl Deref for RStringArray {
    type Target = StringArray;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl AsRef<StringArray> for RStringArray {
    #[inline]
    fn as_ref(&self) -> &StringArray {
        &self.array
    }
}

impl AsRef<dyn Array + 'static> for RStringArray {
    #[inline]
    fn as_ref(&self) -> &(dyn Array + 'static) {
        &self.array
    }
}

impl std::fmt::Debug for RStringArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RStringArray")
            .field("array", &self.array)
            .field("r_backed", &self.source.is_some())
            .finish()
    }
}

// endregion

// endregion

mod r_buffers;
pub(crate) use r_buffers::drain_pending_r_buffer_releases;
use r_buffers::{RBufferOwner, original_r_vector};

// region: Zero-copy buffer helpers

/// Create an Arrow Buffer backed by R vector memory (zero-copy).
///
/// The returned Buffer holds a GC guard (via `R_PreserveObject`) that keeps
/// the R SEXP alive. When all references to the Buffer are dropped, the
/// guard releases the R object for GC. Background-thread drops defer release
/// until the next R unwind boundary.
///
/// Returns `None` if the SEXP's data pointer is null (e.g., ALTREP types
/// that return null from `DATAPTR_RO` when they cannot expose a contiguous
/// buffer, such as Arrow arrays with null bitmasks after deserialization).
///
/// # Safety
///
/// - `sexp` must be a valid R vector with contiguous data of type `T`
/// - Must be called on R's main thread (for `R_PreserveObject`)
unsafe fn sexp_to_arrow_buffer<T: RNativeType>(sexp: SEXP) -> Option<arrow_buffer::Buffer> {
    let _guard = unsafe { crate::OwnedProtect::new(sexp) };
    let len = sexp.len();
    if len == 0 {
        return Some(arrow_buffer::Buffer::from(Vec::<u8>::new()));
    }

    let ptr = unsafe { sys::DATAPTR_RO(sexp) }.cast::<u8>().cast_mut();

    // ALTREP types with null bitmasks (e.g., deserialized Arrow arrays) may
    // return null from DATAPTR_RO when they cannot provide a contiguous buffer.
    if ptr.is_null() {
        return None;
    }

    // Preserve the R object so it won't be GC'd while Arrow holds a reference
    let guard = Arc::new(unsafe { RBufferOwner::new::<T>(sexp, ptr) });

    let byte_len = len * std::mem::size_of::<T>();

    // SAFETY: R vectors have contiguous memory. The guard keeps the SEXP alive.
    Some(unsafe {
        arrow_buffer::Buffer::from_custom_allocation(
            std::ptr::NonNull::new_unchecked(ptr),
            byte_len,
            guard,
        )
    })
}

/// Allocate an Arrow Buffer backed by a new R vector.
///
/// The returned buffer points into a freshly allocated R vector (REALSXP,
/// INTSXP, or RAWSXP depending on `T`). When this buffer is later used in
/// an Arrow array and that array is converted back to R via `IntoR`, the
/// SEXP pointer recovery will find the original R vector — zero-copy
/// round-trip for the Rust→Arrow→R direction.
///
/// Returns `(buffer, sexp)` so callers can also work with the SEXP directly.
///
/// # Safety
///
/// Must be called on R's main thread.
///
/// # Allocation failure
///
/// If `Rf_allocVector` cannot satisfy the request, R longjmps from inside
/// the allocation. `R_UnwindProtect` catches the longjmp, the framework
/// recognises the R-origin path via `RErrorMarker`, and `R_ContinueUnwind`
/// re-raises R's original `Error: vector memory exhausted (limit reached?)`
/// verbatim. The error message is R's, not Rust's — no tagged-SEXP synthesis
/// happens on this path (cf. `with_r_unwind_protect`). The two
/// `.expect()` calls in the body are therefore unreachable in production —
/// they guard against logic errors (wrong `len` encoding or unexpected null
/// pointer), not against OOM.
///
/// # Example
///
/// ```ignore
/// let (buffer, _sexp) = unsafe { alloc_r_backed_buffer::<f64>(1000) };
/// let values = arrow_buffer::ScalarBuffer::<f64>::from(buffer);
/// // Fill values via unsafe mutable access, then:
/// let array = Float64Array::new(values, None);
/// // array.into_sexp() → returns the original REALSXP (zero-copy)
/// ```
pub unsafe fn alloc_r_backed_buffer<T: RNativeType>(len: usize) -> (arrow_buffer::Buffer, SEXP) {
    if len == 0 {
        return (
            arrow_buffer::Buffer::from(Vec::<u8>::new()),
            SEXP(std::ptr::null_mut()),
        );
    }
    let len_isize: isize = len.try_into().expect("vector length exceeds isize::MAX");
    let sexp = unsafe { sys::Rf_allocVector(T::SEXP_TYPE, len_isize) };
    // freshly allocated R vectors always have a valid data pointer
    let buffer = unsafe { sexp_to_arrow_buffer::<T>(sexp) }
        .expect("freshly allocated R vector must have a valid data pointer");
    (buffer, sexp)
}

// endregion

// region: NA bitmap construction

use crate::altrep_traits::{NA_INTEGER, NA_REAL};

/// Check if an f64 value is R's NA_real_ (specific NaN bit pattern).
#[inline]
fn is_na_real(value: f64) -> bool {
    value.to_bits() == NA_REAL.to_bits()
}

/// Scan an R integer vector for `NA_integer_` and build an Arrow NullBuffer.
///
/// Returns `None` if no NAs found (all values valid).
fn build_i32_null_buffer(slice: &[i32]) -> Option<arrow_buffer::NullBuffer> {
    if !slice.contains(&NA_INTEGER) {
        return None;
    }
    let mut builder = arrow_buffer::BooleanBufferBuilder::new(slice.len());
    for &v in slice {
        builder.append(v != NA_INTEGER);
    }
    Some(arrow_buffer::NullBuffer::new(builder.finish()))
}

/// Scan an R real vector for `NA_real_` and build an Arrow NullBuffer.
///
/// Returns `None` if no NAs found (all values valid).
fn build_f64_null_buffer(slice: &[f64]) -> Option<arrow_buffer::NullBuffer> {
    let has_any_na = slice.iter().any(|&v| is_na_real(v));
    if !has_any_na {
        return None;
    }
    let mut builder = arrow_buffer::BooleanBufferBuilder::new(slice.len());
    for &v in slice {
        builder.append(!is_na_real(v));
    }
    Some(arrow_buffer::NullBuffer::new(builder.finish()))
}

// endregion

// region: TryFromSexp — zero-copy primitives (R → Arrow)

impl TryFromSexp for Float64Array {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // Derive the expected tag from the element type instead of hardwiring it,
        // so it can't drift from the `sexp_to_arrow_buffer::<f64>` / `f64::elt`
        // calls below that read the same buffer (#882, follow-up to #881).
        let actual = sexp.type_of();
        if actual != <f64 as RNativeType>::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: <f64 as RNativeType>::SEXP_TYPE,
                actual,
            }
            .into());
        }
        let len = sexp.len();
        if len == 0 {
            return Ok(Float64Array::from(Vec::<f64>::new()));
        }

        // Try zero-copy: wrap R's data buffer directly
        if let Some(buffer) = unsafe { sexp_to_arrow_buffer::<f64>(sexp) } {
            let scalar_buffer = arrow_buffer::ScalarBuffer::<f64>::from(buffer);

            // Scan for NAs to build null bitmap (data stays untouched)
            let slice: &[f64] = unsafe { sexp.as_slice() };
            let null_buffer = build_f64_null_buffer(slice);

            return Ok(Float64Array::new(scalar_buffer, null_buffer));
        }

        // Fallback: DATAPTR_RO returned null (e.g., ALTREP Arrow array with nulls).
        // Read element-by-element via elt() which works for all SEXP types.
        let values: Vec<Option<f64>> = (0..len)
            .map(|i| {
                let v = f64::elt(sexp, i as R_xlen_t);
                if is_na_real(v) { None } else { Some(v) }
            })
            .collect();
        Ok(Float64Array::from(values))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

impl TryFromSexp for Int32Array {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // Tag derived from the element type (`i32`) — see Float64Array above (#882).
        let actual = sexp.type_of();
        if actual != <i32 as RNativeType>::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: <i32 as RNativeType>::SEXP_TYPE,
                actual,
            }
            .into());
        }
        let len = sexp.len();
        if len == 0 {
            return Ok(Int32Array::from(Vec::<i32>::new()));
        }

        // Try zero-copy: wrap R's data buffer directly
        if let Some(buffer) = unsafe { sexp_to_arrow_buffer::<i32>(sexp) } {
            let scalar_buffer = arrow_buffer::ScalarBuffer::<i32>::from(buffer);

            let slice: &[i32] = unsafe { sexp.as_slice() };
            let null_buffer = build_i32_null_buffer(slice);

            return Ok(Int32Array::new(scalar_buffer, null_buffer));
        }

        // Fallback: DATAPTR_RO returned null (e.g., ALTREP Arrow array with nulls).
        // Read element-by-element via elt() which works for all SEXP types.
        let values: Vec<Option<i32>> = (0..len)
            .map(|i| {
                let v = i32::elt(sexp, i as R_xlen_t);
                if v == NA_INTEGER { None } else { Some(v) }
            })
            .collect();
        Ok(Int32Array::from(values))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

impl TryFromSexp for UInt8Array {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // Tag derived from the element type (`u8`) — see Float64Array above (#882).
        let actual = sexp.type_of();
        if actual != <u8 as RNativeType>::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: <u8 as RNativeType>::SEXP_TYPE,
                actual,
            }
            .into());
        }
        let len = sexp.len();
        if len == 0 {
            return Ok(UInt8Array::from(Vec::<u8>::new()));
        }

        // Zero-copy, no NA concept for raw vectors
        if let Some(buffer) = unsafe { sexp_to_arrow_buffer::<u8>(sexp) } {
            let scalar_buffer = arrow_buffer::ScalarBuffer::<u8>::from(buffer);
            return Ok(UInt8Array::new(scalar_buffer, None));
        }

        // Fallback: DATAPTR_RO returned null. Read element-by-element via elt().
        let values: Vec<u8> = (0..len).map(|i| u8::elt(sexp, i as R_xlen_t)).collect();
        Ok(UInt8Array::from(values))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// endregion

// region: TryFromSexp — copy conversions (BooleanArray, StringArray)

impl TryFromSexp for BooleanArray {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // Literal is the source of truth here (#882): Arrow's BooleanArray is
        // bit-packed and built element-by-element via `logical_elt`, so there is
        // no `RLogical`-typed buffer path alongside this guard to desync from.
        let actual = sexp.type_of();
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::LGLSXP,
                actual,
            }
            .into());
        }
        let slice: &[crate::RLogical] = unsafe { sexp.as_slice() };
        let mut builder = arrow_array::builder::BooleanBuilder::with_capacity(slice.len());

        for &val in slice {
            if val.is_na() {
                builder.append_null();
            } else {
                builder.append_value(val.to_i32() != 0);
            }
        }

        Ok(builder.finish())
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

impl TryFromSexp for StringArray {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }
        let len = sexp.len();
        let mut builder = arrow_array::builder::StringBuilder::with_capacity(len, len * 8);

        for i in 0..len {
            let charsxp = sexp.string_elt(i as R_xlen_t);
            if charsxp.is_na_string() {
                builder.append_null();
            } else {
                let s = unsafe { crate::from_r::charsxp_to_str(charsxp) };
                builder.append_value(s);
            }
        }

        Ok(builder.finish())
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// endregion

// region: TryFromSexp — Factor, Date, POSIXct (R class-aware conversions)

/// Type alias for dictionary-encoded string arrays (Arrow equivalent of R factors).
pub type StringDictionaryArray = DictionaryArray<Int32Type>;

/// Check if an R SEXP has a specific class (checks "class" attribute).
///
/// Check if an R SEXP is a factor (INTSXP with "levels" attribute).
fn is_factor(sexp: SEXP) -> bool {
    sexp.type_of() == SEXPTYPE::INTSXP && sexp.inherits_class(c"factor")
}

/// Check if an R SEXP is a Date (REALSXP with class "Date").
fn is_date(sexp: SEXP) -> bool {
    sexp.type_of() == SEXPTYPE::REALSXP && sexp.inherits_class(c"Date")
}

/// Check if an R SEXP is POSIXct (REALSXP with class "POSIXct").
fn is_posixct(sexp: SEXP) -> bool {
    sexp.type_of() == SEXPTYPE::REALSXP && sexp.inherits_class(c"POSIXct")
}

/// Convert R factor to Arrow `DictionaryArray<Int32Type>` with string values.
///
/// R factors are INTSXP with 1-based indices into a "levels" character vector.
/// Arrow DictionaryArray uses 0-based indices, so we subtract 1.
/// NA in factor (NA_integer_) → null in the dictionary keys.
impl TryFromSexp for StringDictionaryArray {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if !is_factor(sexp) {
            return Err(SexpError::InvalidValue(
                "expected R factor (integer with levels attribute)".into(),
            ));
        }

        let n = sexp.len();
        let levels_sexp = sexp.get_levels();
        if levels_sexp.type_of() != SEXPTYPE::STRSXP {
            return Err(SexpError::InvalidValue(
                "factor missing levels attribute".into(),
            ));
        }

        // Build the dictionary (levels)
        let n_levels = levels_sexp.len();
        let mut dict_builder =
            arrow_array::builder::StringBuilder::with_capacity(n_levels, n_levels * 8);
        for i in 0..n_levels {
            let charsxp = levels_sexp.string_elt(i as R_xlen_t);
            let s = unsafe { crate::from_r::charsxp_to_str(charsxp) };
            dict_builder.append_value(s);
        }
        let dictionary = dict_builder.finish();

        // Build the keys (1-based → 0-based, NA → null)
        let slice: &[i32] = unsafe { sexp.as_slice() };
        let mut keys_builder = arrow_array::builder::Int32Builder::with_capacity(n);
        for &v in slice {
            if v == NA_INTEGER {
                keys_builder.append_null();
            } else {
                keys_builder.append_value(v - 1); // R is 1-based, Arrow is 0-based
            }
        }
        let keys = keys_builder.finish();

        DictionaryArray::try_new(keys, Arc::new(dictionary))
            .map_err(|e| SexpError::InvalidValue(e.to_string()))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

/// Convert R Date to Arrow Date32Array.
///
/// R Date values are doubles (days since 1970-01-01). Arrow Date32 is i32 (same epoch).
impl TryFromSexp for Date32Array {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if !is_date(sexp) {
            return Err(SexpError::InvalidValue(
                "expected R Date object (numeric with class 'Date')".into(),
            ));
        }

        let n = sexp.len();
        let slice: &[f64] = unsafe { sexp.as_slice() };
        let mut builder = arrow_array::builder::Date32Builder::with_capacity(n);

        for &v in slice {
            if is_na_real(v) {
                builder.append_null();
            } else {
                // Arrow Date32 stores i32 days; R Date is an f64 day count.
                // Truncation is intentional (fractional days are dropped) and
                // the cast saturates for out-of-range values (no UB).
                #[allow(clippy::cast_possible_truncation)]
                let days = v as i32; // f64 days → i32 days
                builder.append_value(days);
            }
        }

        Ok(builder.finish())
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

/// Convert R POSIXct to Arrow TimestampSecondArray.
///
/// R POSIXct values are doubles (seconds since Unix epoch, possibly fractional).
/// Arrow TimestampSecondArray uses i64 seconds. Fractional seconds are truncated.
/// Timezone from R's "tzone" attribute is preserved if present.
pub fn posixct_to_timestamp(sexp: SEXP) -> Result<TimestampSecondArray, SexpError> {
    if !is_posixct(sexp) {
        return Err(SexpError::InvalidValue(
            "expected R POSIXct object (numeric with class 'POSIXct')".into(),
        ));
    }

    let n = sexp.len();
    let slice: &[f64] = unsafe { sexp.as_slice() };

    // Extract timezone if present
    let tzone_sexp = sexp.get_attr(crate::cached_class::tzone_symbol());
    let tz: Option<Arc<str>> = if tzone_sexp.type_of() == SEXPTYPE::STRSXP && tzone_sexp.len() > 0 {
        let charsxp = tzone_sexp.string_elt(0);
        let s = unsafe { crate::from_r::charsxp_to_str(charsxp) };
        if s.is_empty() {
            None
        } else {
            Some(Arc::from(s))
        }
    } else {
        None
    };

    let mut builder = arrow_array::builder::TimestampSecondBuilder::with_capacity(n);
    for &v in slice {
        if is_na_real(v) {
            builder.append_null();
        } else {
            // Arrow stores i64 seconds; R POSIXct is f64 seconds. Fractional
            // seconds are intentionally truncated (documented above); the cast
            // saturates for out-of-range values (no UB).
            #[allow(clippy::cast_possible_truncation)]
            let secs = v as i64; // f64 seconds → i64 seconds
            builder.append_value(secs);
        }
    }

    let mut arr = builder.finish();
    if let Some(tz) = tz {
        arr = arr.with_timezone(tz);
    }
    Ok(arr)
}

// endregion

// region: IntoR — Factor, Date, POSIXct

/// Convert Arrow `DictionaryArray<Int32Type>` to R factor.
impl IntoR for StringDictionaryArray {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        use arrow_array::cast::AsArray;

        let n = self.len();
        let keys = self.keys();
        let values = self.values().as_string::<i32>();

        unsafe {
            let scope = crate::gc_protect::ProtectScope::new();

            // Create integer vector for factor codes (0-based → 1-based)
            let (codes, codes_dst) = crate::into_r::alloc_r_vector::<i32>(n);
            scope.protect_raw(codes);
            for (i, slot) in codes_dst.iter_mut().enumerate() {
                *slot = if self.is_null(i) {
                    NA_INTEGER
                } else {
                    keys.value(i) + 1 // Arrow 0-based → R 1-based
                };
            }

            // Create levels character vector
            let n_levels = arrow_array::Array::len(&values);
            let levels = scope.alloc_character(n_levels).into_raw();
            for i in 0..n_levels {
                let s = values.value(i);
                let charsxp = SEXP::charsxp(s);
                levels.set_string_elt(i as R_xlen_t, charsxp);
            }

            // Set levels and class attributes
            codes.set_levels(levels);
            codes.set_class(crate::factor::factor_class_sexp());

            codes
        }
    }
}

/// Convert Arrow Date32Array to R Date.
impl IntoR for Date32Array {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        let n = self.len();
        unsafe {
            let scope = crate::gc_protect::ProtectScope::new();

            let (sexp, dst) = crate::into_r::alloc_r_vector::<f64>(n);
            scope.protect_raw(sexp);
            for (i, slot) in dst.iter_mut().enumerate() {
                *slot = if self.is_null(i) {
                    NA_REAL
                } else {
                    // Date32 value is i32 days; widening to f64 is lossless.
                    f64::from(self.value(i))
                };
            }

            // Set class = "Date"
            sexp.set_class(crate::cached_class::date_class_sexp());

            sexp
        }
    }
}

/// Convert Arrow TimestampSecondArray to R POSIXct.
impl IntoR for TimestampSecondArray {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        let n = self.len();
        let tz = match self.data_type() {
            DataType::Timestamp(_, Some(tz)) => Some(tz.clone()),
            _ => None,
        };

        unsafe {
            let scope = crate::gc_protect::ProtectScope::new();

            let (sexp, dst) = crate::into_r::alloc_r_vector::<f64>(n);
            scope.protect_raw(sexp);
            for (i, slot) in dst.iter_mut().enumerate() {
                *slot = if self.is_null(i) {
                    NA_REAL
                } else {
                    self.value(i) as f64
                };
            }

            // Set class = c("POSIXct", "POSIXt")
            sexp.set_class(crate::cached_class::posixct_class_sexp());

            // Set tzone attribute if present
            if let Some(tz) = tz {
                let tz_str = scope.alloc_character(1).into_raw();
                tz_str.set_string_elt(0, SEXP::charsxp(&tz));
                sexp.set_attr(crate::cached_class::tzone_symbol(), tz_str);
            }

            sexp
        }
    }
}

// endregion

// region: TryFromSexp — RecordBatch from data.frame (class-aware dispatch)

/// Convert a single R column SEXP to an Arrow ArrayRef.
///
/// Dispatches on R class attributes first (factor, Date, POSIXct), then
/// falls back to TYPEOF for plain vectors.
fn sexp_column_to_arrow(col_sexp: SEXP, col_name: &str) -> Result<(Field, ArrayRef), SexpError> {
    // Check class-based types first (before plain TYPEOF dispatch)
    if is_factor(col_sexp) {
        let arr = StringDictionaryArray::try_from_sexp(col_sexp)?;
        let nullable = arr.logical_null_count() > 0;
        return Ok((
            Field::new(
                col_name,
                DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8)),
                nullable,
            ),
            Arc::new(arr),
        ));
    }
    if is_date(col_sexp) {
        let arr = Date32Array::try_from_sexp(col_sexp)?;
        let nullable = arr.null_count() > 0;
        return Ok((
            Field::new(col_name, DataType::Date32, nullable),
            Arc::new(arr),
        ));
    }
    if is_posixct(col_sexp) {
        let arr = posixct_to_timestamp(col_sexp)?;
        let nullable = arr.null_count() > 0;
        return Ok((
            Field::new(col_name, arr.data_type().clone(), nullable),
            Arc::new(arr),
        ));
    }

    // Plain TYPEOF dispatch
    let (field, array): (Field, ArrayRef) = match col_sexp.type_of() {
        SEXPTYPE::REALSXP => {
            let arr = Float64Array::try_from_sexp(col_sexp)?;
            let nullable = arr.null_count() > 0;
            (
                Field::new(col_name, DataType::Float64, nullable),
                Arc::new(arr),
            )
        }
        SEXPTYPE::INTSXP => {
            let arr = Int32Array::try_from_sexp(col_sexp)?;
            let nullable = arr.null_count() > 0;
            (
                Field::new(col_name, DataType::Int32, nullable),
                Arc::new(arr),
            )
        }
        SEXPTYPE::LGLSXP => {
            let arr = BooleanArray::try_from_sexp(col_sexp)?;
            let nullable = arr.null_count() > 0;
            (
                Field::new(col_name, DataType::Boolean, nullable),
                Arc::new(arr),
            )
        }
        SEXPTYPE::STRSXP => {
            let arr = StringArray::try_from_sexp(col_sexp)?;
            let nullable = arr.null_count() > 0;
            (
                Field::new(col_name, DataType::Utf8, nullable),
                Arc::new(arr),
            )
        }
        SEXPTYPE::RAWSXP => {
            let arr = UInt8Array::try_from_sexp(col_sexp)?;
            (Field::new(col_name, DataType::UInt8, false), Arc::new(arr))
        }
        other => {
            return Err(SexpError::InvalidValue(format!(
                "unsupported column type for Arrow conversion: {other:?}"
            )));
        }
    };
    Ok((field, array))
}

impl TryFromSexp for RecordBatch {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::VECSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::VECSXP,
                actual,
            }
            .into());
        }

        let ncol = sexp.len();

        // Get column names
        let names_sexp = sexp.get_names();
        let names: Vec<String> = if names_sexp.type_of() == SEXPTYPE::STRSXP {
            (0..ncol)
                .map(|i| {
                    let charsxp = names_sexp.string_elt(i as R_xlen_t);
                    unsafe { crate::from_r::charsxp_to_str(charsxp) }.to_string()
                })
                .collect()
        } else {
            (0..ncol).map(|i| format!("V{}", i + 1)).collect()
        };

        let mut fields = Vec::with_capacity(ncol);
        let mut columns: Vec<ArrayRef> = Vec::with_capacity(ncol);

        for (i, name) in names.iter().enumerate() {
            let col_sexp = sexp.vector_elt(i as R_xlen_t);
            let (field, array) = sexp_column_to_arrow(col_sexp, name)?;
            fields.push(field);
            columns.push(array);
        }

        // Validate that all columns have the same length before calling
        // RecordBatch::try_new, which would otherwise emit a generic error.
        if let Some(first) = columns.first() {
            let nrow = first.len();
            for (i, col) in columns.iter().enumerate().skip(1) {
                if col.len() != nrow {
                    return Err(SexpError::InvalidValue(format!(
                        "column '{}' has length {} but column '{}' has length {}",
                        names[i],
                        col.len(),
                        names[0],
                        nrow,
                    )));
                }
            }
        }

        let schema = Arc::new(Schema::new(fields));
        RecordBatch::try_new(schema, columns).map_err(|e| SexpError::InvalidValue(e.to_string()))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// endregion

// region: TryFromSexp — ArrayRef dynamic dispatch

impl TryFromSexp for ArrayRef {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        match sexp.type_of() {
            SEXPTYPE::REALSXP => Ok(Arc::new(Float64Array::try_from_sexp(sexp)?)),
            SEXPTYPE::INTSXP => Ok(Arc::new(Int32Array::try_from_sexp(sexp)?)),
            SEXPTYPE::LGLSXP => Ok(Arc::new(BooleanArray::try_from_sexp(sexp)?)),
            SEXPTYPE::STRSXP => Ok(Arc::new(StringArray::try_from_sexp(sexp)?)),
            SEXPTYPE::RAWSXP => Ok(Arc::new(UInt8Array::try_from_sexp(sexp)?)),
            other => Err(SexpError::InvalidValue(format!(
                "unsupported R type for Arrow conversion: {other:?}"
            ))),
        }
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// endregion

// region: IntoR — Arrow arrays to R vectors

impl IntoR for Float64Array {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        // Only a registered, complete R buffer can return its source. A new
        // Arrow null bitmap may require replacing values with R sentinels.
        if let Some(sexp) = original_r_vector::<f64>(self.values().inner(), self.len())
            && (self.null_count() == 0
                || (0..self.len()).all(|i| !self.is_null(i) || is_na_real(self.value(i))))
        {
            return sexp;
        }

        // Fallback: allocate and copy
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector::<f64>(self.len());
            if self.null_count() == 0 {
                dst.copy_from_slice(self.values().as_ref());
            } else {
                for (i, slot) in dst.iter_mut().enumerate() {
                    *slot = if self.is_null(i) {
                        NA_REAL
                    } else {
                        self.value(i)
                    };
                }
            }
            sexp
        }
    }
}

impl IntoR for Int32Array {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        // Only a registered, complete R buffer can return its source. A new
        // Arrow null bitmap may require replacing values with R sentinels.
        if let Some(sexp) = original_r_vector::<i32>(self.values().inner(), self.len())
            && (self.null_count() == 0
                || (0..self.len()).all(|i| !self.is_null(i) || self.value(i) == NA_INTEGER))
        {
            return sexp;
        }

        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector::<i32>(self.len());
            if self.null_count() == 0 {
                dst.copy_from_slice(self.values().as_ref());
            } else {
                for (i, slot) in dst.iter_mut().enumerate() {
                    *slot = if self.is_null(i) {
                        NA_INTEGER
                    } else {
                        self.value(i)
                    };
                }
            }
            sexp
        }
    }
}

impl IntoR for UInt8Array {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        // Only a registered, complete R buffer can return its source. A new
        // Arrow null bitmap may require replacing values with R sentinels.
        if let Some(sexp) = original_r_vector::<u8>(self.values().inner(), self.len())
            && (self.null_count() == 0
                || (0..self.len()).all(|i| !self.is_null(i) || self.value(i) == 0))
        {
            return sexp;
        }

        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector::<u8>(self.len());
            if self.null_count() == 0 {
                dst.copy_from_slice(self.values().as_ref());
            } else {
                for (i, slot) in dst.iter_mut().enumerate() {
                    *slot = if self.is_null(i) { 0 } else { self.value(i) };
                }
            }
            sexp
        }
    }
}

impl IntoR for BooleanArray {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector::<crate::RLogical>(self.len());
            for (i, slot) in dst.iter_mut().enumerate() {
                *slot = if self.is_null(i) {
                    crate::RLogical::NA
                } else if self.value(i) {
                    crate::RLogical::TRUE
                } else {
                    crate::RLogical::FALSE
                };
            }
            sexp
        }
    }
}

impl IntoR for StringArray {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        let n = Array::len(&self);
        unsafe {
            // STRSXP literal is the source of truth (#882): R strings are
            // write-barriered CHARSXP-pointer arrays, not a `RNativeType` element
            // type, so there is no `alloc_r_vector::<T>` collapse here.
            let sexp = sys::Rf_allocVector(SEXPTYPE::STRSXP, n as R_xlen_t);
            let guard = crate::gc_protect::OwnedProtect::new(sexp);
            for i in 0..n {
                if self.is_null(i) {
                    guard.get().set_string_elt(i as R_xlen_t, SEXP::na_string());
                } else {
                    let s = self.value(i);
                    guard.get().set_string_elt(i as R_xlen_t, SEXP::charsxp(s));
                }
            }
            guard.get()
        }
    }
}

// endregion

// region: TryFromSexp + IntoR — R-backed types (RPrimitive, RStringArray, RRecordBatch)

// RPrimitive<Float64Type>

impl TryFromSexp for RPrimitive<Float64Type> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let array = Float64Array::try_from_sexp(sexp)?;
        Ok(unsafe { RPrimitive::from_r(array, sexp) })
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// RPrimitive IntoR: use stored R source directly when available,
// otherwise fall back to inner array's IntoR (which does pointer recovery).
macro_rules! impl_rprimitive_into_r {
    ($prim_type:ty) => {
        impl IntoR for RPrimitive<$prim_type> {
            type Error = std::convert::Infallible;

            fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
                Ok(self.into_sexp())
            }

            fn into_sexp(self) -> SEXP {
                if let Some(sexp) = self.source {
                    return sexp;
                }
                self.array.into_sexp()
            }
        }
    };
}

impl_rprimitive_into_r!(Float64Type);
impl_rprimitive_into_r!(Int32Type);
impl_rprimitive_into_r!(UInt8Type);

// RPrimitive TryFromSexp

impl TryFromSexp for RPrimitive<Int32Type> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let array = Int32Array::try_from_sexp(sexp)?;
        Ok(unsafe { RPrimitive::from_r(array, sexp) })
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

impl TryFromSexp for RPrimitive<UInt8Type> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let array = UInt8Array::try_from_sexp(sexp)?;
        Ok(unsafe { RPrimitive::from_r(array, sexp) })
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// RStringArray

impl TryFromSexp for RStringArray {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let array = StringArray::try_from_sexp(sexp)?;
        Ok(unsafe { RStringArray::from_r(array, sexp) })
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

impl IntoR for RStringArray {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        if let Some(sexp) = self.source {
            return sexp;
        }
        self.array.into_sexp()
    }
}

// Note: RRecordBatch removed — RecordBatch.into_sexp() already calls
// arrow_array_to_sexp() per column, which delegates to the individual array
// IntoR impls that do automatic SEXP pointer recovery. No wrapper needed.

// endregion

// region: IntoR — RecordBatch to data.frame

/// Convert an Arrow ArrayRef to an R SEXP, dispatching on DataType.
fn arrow_array_to_sexp(array: &ArrayRef) -> SEXP {
    use arrow_array::cast::AsArray;

    match array.data_type() {
        DataType::Float64 => array.as_primitive::<Float64Type>().clone().into_sexp(),
        DataType::Float32 => {
            // Widen f32 → f64 for R
            let arr = array.as_primitive::<arrow_array::types::Float32Type>();
            let widened: Float64Array = arr.iter().map(|v| v.map(f64::from)).collect();
            widened.into_sexp()
        }
        DataType::Int32 => array.as_primitive::<Int32Type>().clone().into_sexp(),
        DataType::Int64 => {
            // R has no i64 — convert to f64 (may lose precision for values > 2^53)
            let arr = array.as_primitive::<arrow_array::types::Int64Type>();
            let converted: Float64Array = arr.iter().map(|v| v.map(|x| x as f64)).collect();
            converted.into_sexp()
        }
        DataType::Int16 => {
            let arr = array.as_primitive::<arrow_array::types::Int16Type>();
            let widened: Int32Array = arr.iter().map(|v| v.map(i32::from)).collect();
            widened.into_sexp()
        }
        DataType::Int8 => {
            let arr = array.as_primitive::<arrow_array::types::Int8Type>();
            let widened: Int32Array = arr.iter().map(|v| v.map(i32::from)).collect();
            widened.into_sexp()
        }
        DataType::UInt8 => array.as_primitive::<UInt8Type>().clone().into_sexp(),
        DataType::Boolean => array.as_boolean().clone().into_sexp(),
        DataType::Utf8 => array.as_string::<i32>().clone().into_sexp(),
        DataType::Date32 => array.as_primitive::<Date32Type>().clone().into_sexp(),
        DataType::Timestamp(arrow_schema::TimeUnit::Second, _) => array
            .as_primitive::<TimestampSecondType>()
            .clone()
            .into_sexp(),
        DataType::Dictionary(key_type, _) if key_type.as_ref() == &DataType::Int32 => {
            array
                .as_any()
                .downcast_ref::<StringDictionaryArray>()
                .cloned()
                .map(|a| a.into_sexp())
                .unwrap_or_else(|| {
                    // Not a string dictionary, fall through to default
                    let n = array.len();
                    unsafe {
                        let sexp = sys::Rf_allocVector(SEXPTYPE::STRSXP, n as R_xlen_t);
                        let guard = crate::gc_protect::OwnedProtect::new(sexp);
                        for i in 0..n {
                            guard.get().set_string_elt(i as R_xlen_t, SEXP::na_string());
                        }
                        guard.get()
                    }
                })
        }
        // Fallback for unsupported types: return character vector of NA
        _ => {
            let n = array.len();
            unsafe {
                let sexp = sys::Rf_allocVector(SEXPTYPE::STRSXP, n as R_xlen_t);
                let guard = crate::gc_protect::OwnedProtect::new(sexp);
                for i in 0..n {
                    guard.get().set_string_elt(i as R_xlen_t, SEXP::na_string());
                }
                guard.get()
            }
        }
    }
}

impl IntoR for RecordBatch {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        let ncol = self.num_columns();
        let nrow = self.num_rows();
        let schema = self.schema();

        unsafe {
            let scope = crate::gc_protect::ProtectScope::new();

            // Create list for columns
            let list = scope.alloc_vecsxp(ncol).into_raw();

            // Create names vector
            let names = scope.alloc_character(ncol).into_raw();

            for (i, (col, field)) in self.columns().iter().zip(schema.fields()).enumerate() {
                let col_sexp = arrow_array_to_sexp(col);
                list.set_vector_elt(i as R_xlen_t, col_sexp);

                let name = field.name();
                names.set_string_elt(i as R_xlen_t, SEXP::charsxp(name));
            }

            // Set names attribute
            list.set_names(names);

            // Set class = "data.frame"
            list.set_class(crate::cached_class::data_frame_class_sexp());

            // Set compact row.names: c(NA_integer_, -nrow)
            let (rownames, rn) = crate::into_r::alloc_r_vector::<i32>(2);
            scope.protect_raw(rownames);
            rn[0] = NA_INTEGER;
            rn[1] = -i32::try_from(nrow).expect("data frame nrow exceeds i32::MAX");
            list.set_row_names(rownames);

            list
        }
    }
}

// endregion

// region: IntoR — ArrayRef dynamic dispatch

impl IntoR for ArrayRef {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        arrow_array_to_sexp(&self)
    }
}

// endregion

// region: ALTREP support for Arrow arrays (Lazy<T>)
//
// These impls allow `Lazy<Float64Array>`, `Lazy<Int32Array>`, etc. to return
// Arrow data as ALTREP vectors. R reads elements on demand; for null-free
// arrays the Dataptr callback returns the Arrow buffer pointer directly
// (true zero-copy, O(1)).

use crate::altrep_data::{
    AltIntegerData, AltLogicalData, AltRawData, AltRealData, AltrepDataptr, AltrepLen, Logical,
};
use crate::externalptr::TypedExternal;

// region: TypedExternal impls (required for ExternalPtr wrapping in ALTREP)

impl TypedExternal for Float64Array {
    const TYPE_NAME: &'static str = "arrow::Float64Array";
    const TYPE_NAME_CSTR: &'static [u8] = b"arrow::Float64Array\0";
    const TYPE_ID_CSTR: &'static [u8] = b"arrow::Float64Array\0";
}

impl TypedExternal for Int32Array {
    const TYPE_NAME: &'static str = "arrow::Int32Array";
    const TYPE_NAME_CSTR: &'static [u8] = b"arrow::Int32Array\0";
    const TYPE_ID_CSTR: &'static [u8] = b"arrow::Int32Array\0";
}

impl TypedExternal for UInt8Array {
    const TYPE_NAME: &'static str = "arrow::UInt8Array";
    const TYPE_NAME_CSTR: &'static [u8] = b"arrow::UInt8Array\0";
    const TYPE_ID_CSTR: &'static [u8] = b"arrow::UInt8Array\0";
}

impl TypedExternal for BooleanArray {
    const TYPE_NAME: &'static str = "arrow::BooleanArray";
    const TYPE_NAME_CSTR: &'static [u8] = b"arrow::BooleanArray\0";
    const TYPE_ID_CSTR: &'static [u8] = b"arrow::BooleanArray\0";
}

impl TypedExternal for StringArray {
    const TYPE_NAME: &'static str = "arrow::StringArray";
    const TYPE_NAME_CSTR: &'static [u8] = b"arrow::StringArray\0";
    const TYPE_ID_CSTR: &'static [u8] = b"arrow::StringArray\0";
}

// endregion

// region: AltrepLen impls

impl AltrepLen for Float64Array {
    fn len(&self) -> usize {
        Array::len(self)
    }
}

impl AltrepLen for Int32Array {
    fn len(&self) -> usize {
        Array::len(self)
    }
}

impl AltrepLen for UInt8Array {
    fn len(&self) -> usize {
        Array::len(self)
    }
}

impl AltrepLen for BooleanArray {
    fn len(&self) -> usize {
        Array::len(self)
    }
}

// endregion

// region: ALTREP data trait impls

impl AltRealData for Float64Array {
    fn elt(&self, i: usize) -> f64 {
        if self.is_null(i) {
            NA_REAL
        } else {
            self.value(i)
        }
    }

    fn as_slice(&self) -> Option<&[f64]> {
        // Zero-copy: return Arrow buffer pointer directly if no nulls.
        // Arrow's null bitmap marks nulls but data buffer has garbage there,
        // so we can only expose the raw buffer when null_count == 0.
        if self.null_count() == 0 {
            Some(self.values().as_ref())
        } else {
            None
        }
    }

    fn no_na(&self) -> Option<bool> {
        Some(self.null_count() == 0)
    }
}

impl AltIntegerData for Int32Array {
    fn elt(&self, i: usize) -> i32 {
        if self.is_null(i) {
            crate::altrep_traits::NA_INTEGER
        } else {
            self.value(i)
        }
    }

    fn as_slice(&self) -> Option<&[i32]> {
        if self.null_count() == 0 {
            Some(self.values().as_ref())
        } else {
            None
        }
    }

    fn no_na(&self) -> Option<bool> {
        Some(self.null_count() == 0)
    }
}

impl AltRawData for UInt8Array {
    fn elt(&self, i: usize) -> u8 {
        if self.is_null(i) { 0 } else { self.value(i) }
    }

    fn as_slice(&self) -> Option<&[u8]> {
        if self.null_count() == 0 {
            Some(self.values().as_ref())
        } else {
            None
        }
    }
}

impl AltLogicalData for BooleanArray {
    fn elt(&self, i: usize) -> Logical {
        if self.is_null(i) {
            Logical::Na
        } else if self.value(i) {
            Logical::True
        } else {
            Logical::False
        }
    }

    fn no_na(&self) -> Option<bool> {
        Some(self.null_count() == 0)
    }
}

// endregion

// region: AltrepDataptr impls (zero-copy when no nulls)

impl AltrepDataptr<f64> for Float64Array {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut f64> {
        // Arrow buffers are immutable — can't provide writable pointer.
        // Return read-only pointer cast to mut (R handles const-correctness).
        if self.null_count() == 0 {
            Some(self.values().as_ptr().cast_mut())
        } else {
            None
        }
    }

    fn dataptr_or_null(&self) -> Option<*const f64> {
        if self.null_count() == 0 {
            Some(self.values().as_ptr())
        } else {
            None
        }
    }
}

impl AltrepDataptr<i32> for Int32Array {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        if self.null_count() == 0 {
            Some(self.values().as_ptr().cast_mut())
        } else {
            None
        }
    }

    fn dataptr_or_null(&self) -> Option<*const i32> {
        if self.null_count() == 0 {
            Some(self.values().as_ptr())
        } else {
            None
        }
    }
}

impl AltrepDataptr<u8> for UInt8Array {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut u8> {
        if self.null_count() == 0 {
            Some(self.values().as_ptr().cast_mut())
        } else {
            None
        }
    }

    fn dataptr_or_null(&self) -> Option<*const u8> {
        if self.null_count() == 0 {
            Some(self.values().as_ptr())
        } else {
            None
        }
    }
}

// endregion

// region: ALTREP bridge traits + InferBase + RegisterAltrep
//
// impl_alt*_from_data! generates the bridge traits (Altrep, AltVec, AltReal/etc.)
// AND calls impl_inferbase_*! to register the class creation.

// Serialize support: materialize Arrow array into native R vector for saveRDS.
// On readRDS, the native vector is loaded directly (no Rust/Arrow needed).

// Arrow serialized_state impls allocate and copy directly instead of calling
// self.clone().into_sexp(). The IntoR path for Float64Array/Int32Array/UInt8Array
// includes try_recover_r_sexp which speculatively probes whether the Arrow buffer
// is R-backed. In serialized_state, the data is always Rust-owned (no R SEXP to
// recover), so the speculative probe would read garbage memory for no benefit.
// Bypassing it avoids false positives that can cause segfaults on some platforms.

impl crate::altrep_data::AltrepSerialize for Float64Array {
    fn serialized_state(&self) -> SEXP {
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector::<f64>(self.len());
            if self.null_count() == 0 {
                dst.copy_from_slice(self.values().as_ref());
            } else {
                for (i, slot) in dst.iter_mut().enumerate() {
                    *slot = if self.is_null(i) {
                        NA_REAL
                    } else {
                        self.value(i)
                    };
                }
            }
            sexp
        }
    }
    fn unserialize(state: SEXP) -> Option<Self> {
        TryFromSexp::try_from_sexp(state).ok()
    }
}

impl crate::altrep_data::AltrepSerialize for Int32Array {
    fn serialized_state(&self) -> SEXP {
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector::<i32>(self.len());
            if self.null_count() == 0 {
                dst.copy_from_slice(self.values().as_ref());
            } else {
                for (i, slot) in dst.iter_mut().enumerate() {
                    *slot = if self.is_null(i) {
                        NA_INTEGER
                    } else {
                        self.value(i)
                    };
                }
            }
            sexp
        }
    }
    fn unserialize(state: SEXP) -> Option<Self> {
        TryFromSexp::try_from_sexp(state).ok()
    }
}

impl crate::altrep_data::AltrepSerialize for UInt8Array {
    fn serialized_state(&self) -> SEXP {
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector::<u8>(self.len());
            if self.null_count() == 0 {
                dst.copy_from_slice(self.values().as_ref());
            } else {
                for (i, slot) in dst.iter_mut().enumerate() {
                    *slot = if self.is_null(i) { 0 } else { self.value(i) };
                }
            }
            sexp
        }
    }
    fn unserialize(state: SEXP) -> Option<Self> {
        TryFromSexp::try_from_sexp(state).ok()
    }
}

impl crate::altrep_data::AltrepSerialize for BooleanArray {
    fn serialized_state(&self) -> SEXP {
        self.clone().into_sexp()
    }
    fn unserialize(state: SEXP) -> Option<Self> {
        TryFromSexp::try_from_sexp(state).ok()
    }
}

impl crate::altrep_data::AltrepSerialize for StringArray {
    fn serialized_state(&self) -> SEXP {
        self.clone().into_sexp()
    }
    fn unserialize(state: SEXP) -> Option<Self> {
        TryFromSexp::try_from_sexp(state).ok()
    }
}

crate::impl_altreal_from_data!(Float64Array, dataptr, serialize);
crate::impl_altinteger_from_data!(Int32Array, dataptr, serialize);
crate::impl_altraw_from_data!(UInt8Array, dataptr, serialize);
crate::impl_altlogical_from_data!(BooleanArray, serialize);

use crate::altrep::RegisterAltrep;

impl RegisterAltrep for Float64Array {
    fn get_or_init_class() -> crate::sys::altrep::R_altrep_class_t {
        use std::sync::OnceLock;
        static CLASS: OnceLock<crate::sys::altrep::R_altrep_class_t> = OnceLock::new();
        *CLASS.get_or_init(|| {
            let cls = unsafe {
                <Float64Array as crate::altrep_data::InferBase>::make_class(
                    c"arrow_Float64Array".as_ptr(),
                    crate::AltrepPkgName::as_ptr(),
                )
            };
            unsafe {
                <Float64Array as crate::altrep_data::InferBase>::install_methods(cls);
            }
            cls
        })
    }
}

impl RegisterAltrep for Int32Array {
    fn get_or_init_class() -> crate::sys::altrep::R_altrep_class_t {
        use std::sync::OnceLock;
        static CLASS: OnceLock<crate::sys::altrep::R_altrep_class_t> = OnceLock::new();
        *CLASS.get_or_init(|| {
            let cls = unsafe {
                <Int32Array as crate::altrep_data::InferBase>::make_class(
                    c"arrow_Int32Array".as_ptr(),
                    crate::AltrepPkgName::as_ptr(),
                )
            };
            unsafe {
                <Int32Array as crate::altrep_data::InferBase>::install_methods(cls);
            }
            cls
        })
    }
}

impl RegisterAltrep for UInt8Array {
    fn get_or_init_class() -> crate::sys::altrep::R_altrep_class_t {
        use std::sync::OnceLock;
        static CLASS: OnceLock<crate::sys::altrep::R_altrep_class_t> = OnceLock::new();
        *CLASS.get_or_init(|| {
            let cls = unsafe {
                <UInt8Array as crate::altrep_data::InferBase>::make_class(
                    c"arrow_UInt8Array".as_ptr(),
                    crate::AltrepPkgName::as_ptr(),
                )
            };
            unsafe {
                <UInt8Array as crate::altrep_data::InferBase>::install_methods(cls);
            }
            cls
        })
    }
}

impl RegisterAltrep for BooleanArray {
    fn get_or_init_class() -> crate::sys::altrep::R_altrep_class_t {
        use std::sync::OnceLock;
        static CLASS: OnceLock<crate::sys::altrep::R_altrep_class_t> = OnceLock::new();
        *CLASS.get_or_init(|| {
            let cls = unsafe {
                <BooleanArray as crate::altrep_data::InferBase>::make_class(
                    c"arrow_BooleanArray".as_ptr(),
                    crate::AltrepPkgName::as_ptr(),
                )
            };
            unsafe {
                <BooleanArray as crate::altrep_data::InferBase>::install_methods(cls);
            }
            cls
        })
    }
}

// StringArray ALTREP — Elt creates CHARSXP on demand, no Dataptr (not contiguous).

impl AltrepLen for StringArray {
    fn len(&self) -> usize {
        Array::len(self)
    }
}

impl crate::altrep_data::AltStringData for StringArray {
    fn elt(&self, i: usize) -> Option<&str> {
        if self.is_null(i) {
            None
        } else {
            Some(self.value(i))
        }
    }

    fn no_na(&self) -> Option<bool> {
        Some(self.null_count() == 0)
    }
}

crate::impl_altstring_from_data!(StringArray, serialize);

impl RegisterAltrep for StringArray {
    fn get_or_init_class() -> crate::sys::altrep::R_altrep_class_t {
        use std::sync::OnceLock;
        static CLASS: OnceLock<crate::sys::altrep::R_altrep_class_t> = OnceLock::new();
        *CLASS.get_or_init(|| {
            let cls = unsafe {
                <StringArray as crate::altrep_data::InferBase>::make_class(
                    c"arrow_StringArray".as_ptr(),
                    crate::AltrepPkgName::as_ptr(),
                )
            };
            unsafe {
                <StringArray as crate::altrep_data::InferBase>::install_methods(cls);
            }
            cls
        })
    }
}

// region: Linkme registration entries for Arrow ALTREP classes
//
// Each entry here mirrors what `__impl_builtin_altrep_registration!` emits for
// the standard builtins.  Arrow types are in a separate module (optionals/) and
// behind `#[cfg(feature = "arrow")]`, so they cannot be reached by the macro
// call sites in `altrep_impl.rs` — the entries are emitted here instead.
//
// The `#[cfg_attr(not(wasm32), distributed_slice)]` pattern matches the pattern
// used by user-defined ALTREP types (see `miniextendr-macros/src/altrep.rs`).

#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C" fn __mx_altrep_reg_builtin_arrow_Float64Array() {
    Float64Array::get_or_init_class();
}

#[cfg_attr(
    not(target_arch = "wasm32"),
    crate::linkme::distributed_slice(crate::registry::MX_ALTREP_REGISTRATIONS),
    linkme(crate = crate::linkme)
)]
#[doc(hidden)]
#[allow(non_upper_case_globals)]
static __MX_ALTREP_REG_ENTRY_builtin_arrow_Float64Array: crate::registry::AltrepRegistration =
    crate::registry::AltrepRegistration {
        register: __mx_altrep_reg_builtin_arrow_Float64Array,
        symbol: "__mx_altrep_reg_builtin_arrow_Float64Array",
    };

#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C" fn __mx_altrep_reg_builtin_arrow_Int32Array() {
    Int32Array::get_or_init_class();
}

#[cfg_attr(
    not(target_arch = "wasm32"),
    crate::linkme::distributed_slice(crate::registry::MX_ALTREP_REGISTRATIONS),
    linkme(crate = crate::linkme)
)]
#[doc(hidden)]
#[allow(non_upper_case_globals)]
static __MX_ALTREP_REG_ENTRY_builtin_arrow_Int32Array: crate::registry::AltrepRegistration =
    crate::registry::AltrepRegistration {
        register: __mx_altrep_reg_builtin_arrow_Int32Array,
        symbol: "__mx_altrep_reg_builtin_arrow_Int32Array",
    };

#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C" fn __mx_altrep_reg_builtin_arrow_UInt8Array() {
    UInt8Array::get_or_init_class();
}

#[cfg_attr(
    not(target_arch = "wasm32"),
    crate::linkme::distributed_slice(crate::registry::MX_ALTREP_REGISTRATIONS),
    linkme(crate = crate::linkme)
)]
#[doc(hidden)]
#[allow(non_upper_case_globals)]
static __MX_ALTREP_REG_ENTRY_builtin_arrow_UInt8Array: crate::registry::AltrepRegistration =
    crate::registry::AltrepRegistration {
        register: __mx_altrep_reg_builtin_arrow_UInt8Array,
        symbol: "__mx_altrep_reg_builtin_arrow_UInt8Array",
    };

#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C" fn __mx_altrep_reg_builtin_arrow_BooleanArray() {
    BooleanArray::get_or_init_class();
}

#[cfg_attr(
    not(target_arch = "wasm32"),
    crate::linkme::distributed_slice(crate::registry::MX_ALTREP_REGISTRATIONS),
    linkme(crate = crate::linkme)
)]
#[doc(hidden)]
#[allow(non_upper_case_globals)]
static __MX_ALTREP_REG_ENTRY_builtin_arrow_BooleanArray: crate::registry::AltrepRegistration =
    crate::registry::AltrepRegistration {
        register: __mx_altrep_reg_builtin_arrow_BooleanArray,
        symbol: "__mx_altrep_reg_builtin_arrow_BooleanArray",
    };

#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C" fn __mx_altrep_reg_builtin_arrow_StringArray() {
    StringArray::get_or_init_class();
}

#[cfg_attr(
    not(target_arch = "wasm32"),
    crate::linkme::distributed_slice(crate::registry::MX_ALTREP_REGISTRATIONS),
    linkme(crate = crate::linkme)
)]
#[doc(hidden)]
#[allow(non_upper_case_globals)]
static __MX_ALTREP_REG_ENTRY_builtin_arrow_StringArray: crate::registry::AltrepRegistration =
    crate::registry::AltrepRegistration {
        register: __mx_altrep_reg_builtin_arrow_StringArray,
        symbol: "__mx_altrep_reg_builtin_arrow_StringArray",
    };

// endregion
