#![allow(rustdoc::private_intra_doc_links)]
//! Conversions from R SEXP to Rust types.
//!
//! This module provides [`TryFromSexp`] implementations for converting R values to Rust types:
//!
//! | R Type | Rust Type | Access Method |
//! |--------|-----------|---------------|
//! | INTSXP | `i32`, `&[i32]` | `INTEGER()` / `DATAPTR_RO` |
//! | REALSXP | `f64`, `&[f64]` | `REAL()` / `DATAPTR_RO` |
//! | LGLSXP | `RLogical`, `&[RLogical]` | `LOGICAL()` / `DATAPTR_RO` |
//! | RAWSXP | `u8`, `&[u8]` | `RAW()` / `DATAPTR_RO` |
//! | CPLXSXP | `Rcomplex` | `COMPLEX()` / `DATAPTR_RO` |
//! | STRSXP | `&str`, `String` | `STRING_ELT()` + encoding-aware UTF-8 extraction |
//!
//! # Submodules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`logical`] | `Rboolean`.string_elt(`bool`, `Option<bool>` |
//! | [`coerced_scalars`] | Multi-source numeric scalars (`i8`..`usize`) + large integers (`i64`, `u64`) |
//! | [`references`] | Borrowed views: `&T`, `&mut T`, `&[T]`, `Vec<&T>` |
//! | [`strings`] | `&str`, `String`, `char` from STRSXP |
//! | [`na_vectors`] | `Vec<Option<T>>`, `Box<[Option<T>]>` with NA awareness |
//! | [`collections`] | `HashMap`, `BTreeMap`, `HashSet`, `BTreeSet` |
//! | [`cow_and_paths`] | `Cow<[T]>`, `PathBuf`, `OsString`, string sets |
//!
//! # Choosing the right inbound conversion
//!
//! [`TryFromSexp`] is the strict inbound path: it returns `Result<T, SexpError>`
//! and rejects mismatched [`SEXPTYPE`]s outright (no silent coercion). When you
//! need to *accept* arguments coming from multiple R native types, reach for
//! the [`crate::coerce::Coerce`] / [`crate::coerce::TryCoerce`] traits instead
//! — those are the looser inbound path and the entry point for the multi-source
//! scalars handled in [`coerced_scalars`].
//!
//! The strict-vs-lax pairing for *outbound* conversion lives on
//! [`crate::into_r::IntoR`] (lax, default) vs [`crate::strict`] (`#[miniextendr(strict)]`).
//! There is intentionally no `TryFromSexpStrict` trait — inbound is already
//! strict-by-default because it returns `Result`.
//!
//! # Thread Safety
//!
//! The trait provides two methods:
//! - [`TryFromSexp::try_from_sexp`] - checked version with debug thread assertions
//! - [`TryFromSexp::try_from_sexp_unchecked`] - unchecked version for performance-critical paths
//!
//! Use `try_from_sexp_unchecked` when you're certain you're on the main thread:
//! - Inside ALTREP callbacks
//! - Inside standalone `#[miniextendr]` functions (they run on the main thread)
//! - Inside `extern "C-unwind"` functions called directly by R

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::altrep_traits::NA_REAL;
use crate::coerce::TryCoerce;
use crate::{RLogical, SEXP, SEXPTYPE, SexpExt};

/// Check if an f64 value is R's NA_real_ (a specific NaN bit pattern).
///
/// This is different from `f64::is_nan()` which returns true for ALL NaN values.
/// R's `NA_real_` is a specific NaN with a particular bit pattern, while regular
/// NaN values (e.g., from `0.0/0.0`) should be preserved as valid values.
#[inline]
pub(crate) fn is_na_real(value: f64) -> bool {
    value.to_bits() == NA_REAL.to_bits()
}

// region: CHARSXP to string conversion

/// Convert CHARSXP to `&str`, translating to UTF-8 when required.
///
/// Strings that R reports as UTF-8 (including ASCII and native strings in a
/// UTF-8 locale) borrow directly from the CHARSXP via `R_CHAR` + `LENGTH`.
/// Latin-1 and other translatable strings use `Rf_translateCharUTF8`; that
/// buffer lives on R's memory stack for the enclosing `.Call`. Strings marked
/// with R's `bytes` encoding are rejected because they are not text.
///
/// # Safety
///
/// - `charsxp` must be a valid CHARSXP (not NA_STRING, not null).
/// - The returned `&str` is only valid as long as R doesn't GC the CHARSXP.
#[inline]
pub(crate) unsafe fn charsxp_to_str(charsxp: SEXP) -> &'static str {
    unsafe {
        if matches!(crate::sys::Rf_charIsUTF8(charsxp), crate::Rboolean::TRUE) {
            return charsxp_utf8_bytes(charsxp.r_char(), charsxp.len());
        }
        reject_bytes_encoding(crate::sys::Rf_getCharCE(charsxp));
        translated_charsxp_to_str(crate::sys::Rf_translateCharUTF8(charsxp))
    }
}

/// Unchecked version of [`charsxp_to_str`] (skips R thread checks on R API calls).
#[inline]
pub(crate) unsafe fn charsxp_to_str_unchecked(charsxp: SEXP) -> &'static str {
    unsafe {
        if matches!(
            crate::sys::Rf_charIsUTF8_unchecked(charsxp),
            crate::Rboolean::TRUE
        ) {
            return charsxp_utf8_bytes(charsxp.r_char_unchecked(), charsxp.len_unchecked());
        }
        reject_bytes_encoding(crate::sys::Rf_getCharCE_unchecked(charsxp));
        translated_charsxp_to_str(crate::sys::Rf_translateCharUTF8_unchecked(charsxp))
    }
}

#[inline]
unsafe fn charsxp_utf8_bytes(ptr: *const std::os::raw::c_char, len: usize) -> &'static str {
    unsafe {
        let bytes = r_slice(ptr.cast::<u8>(), len);
        std::str::from_utf8(bytes).expect("R string marked as UTF-8 contains invalid UTF-8 bytes")
    }
}

/// Borrow directly from a CHARSXP that R classifies as UTF-8/ASCII.
///
/// Unlike [`charsxp_to_str`], this never returns R's temporary translation
/// buffer. Use it for views whose borrow may be tied to the source STRSXP
/// rather than merely to the enclosing `.Call`.
#[inline]
pub(crate) unsafe fn charsxp_to_borrowed_str(charsxp: SEXP) -> &'static str {
    unsafe {
        if matches!(crate::sys::Rf_charIsUTF8(charsxp), crate::Rboolean::TRUE) {
            return charsxp_utf8_bytes(charsxp.r_char(), charsxp.len());
        }
        reject_bytes_encoding(crate::sys::Rf_getCharCE(charsxp));
        panic!("cannot borrow a non-UTF-8 R string; use Cow<str> or String to translate it");
    }
}

#[inline]
fn reject_bytes_encoding(encoding: crate::cetype_t) {
    if matches!(encoding, crate::cetype_t::CE_BYTES) {
        panic!("cannot convert an R string marked as bytes to Rust UTF-8");
    }
}

#[inline]
unsafe fn translated_charsxp_to_str(ptr: *const std::os::raw::c_char) -> &'static str {
    assert!(!ptr.is_null(), "R returned a null UTF-8 translation");
    unsafe { std::ffi::CStr::from_ptr(ptr) }
        .to_str()
        .expect("R string translation produced invalid UTF-8")
}

/// Convert CHARSXP to `Cow<str>`.
///
/// UTF-8/ASCII strings borrow directly from R. Strings that require encoding
/// translation become owned Rust strings so the translated buffer cannot be
/// invalidated independently of the returned value.
#[inline]
pub(crate) unsafe fn charsxp_to_cow(charsxp: SEXP) -> std::borrow::Cow<'static, str> {
    unsafe {
        if matches!(crate::sys::Rf_charIsUTF8(charsxp), crate::Rboolean::TRUE) {
            return std::borrow::Cow::Borrowed(charsxp_utf8_bytes(charsxp.r_char(), charsxp.len()));
        }
        reject_bytes_encoding(crate::sys::Rf_getCharCE(charsxp));
        std::borrow::Cow::Owned(
            translated_charsxp_to_str(crate::sys::Rf_translateCharUTF8(charsxp)).to_owned(),
        )
    }
}

/// Convert CHARSXP to an owned, lossy `String`.
///
/// NA/null-defensive: returns `None` for `NA_character_`, `R_NilValue`, or a
/// null SEXP. Non-UTF-8 bytes are replaced (`CStr::to_string_lossy`) rather
/// than rejected. Unlike [`charsxp_to_str`] (the UTF-8-asserted hot path for
/// package-internal CHARSXPs), this is for defensive reads of *arbitrary* R
/// objects — S4 class attributes, `geterrmessage()` output, vctrs field
/// names, `tzone` attributes — where the CHARSXP's origin and encoding
/// aren't guaranteed.
///
/// # Safety
///
/// `charsxp` must be a valid SEXP. It may be `R_NilValue` or a null SEXP
/// (both map to `None`); if it is neither of those and not `NA_character_`,
/// it must actually be a CHARSXP.
#[inline]
pub(crate) unsafe fn charsxp_to_string_lossy(charsxp: SEXP) -> Option<String> {
    if charsxp.is_null_or_nil() || charsxp.is_na_string() {
        return None;
    }
    let ptr = charsxp.r_char();
    if ptr.is_null() {
        return None;
    }
    Some(
        unsafe { std::ffi::CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned(),
    )
}

/// Create a slice from an R data pointer, handling the zero-length case.
///
/// R returns a sentinel pointer (`0x1`) instead of null for empty vectors
/// (e.g., `LOGICAL(integer(0))` → `0x1`). Rust 1.93+ validates pointer
/// alignment in `slice::from_raw_parts` even for `len == 0`, so passing
/// R's sentinel directly causes a precondition-check abort.
///
/// This helper returns an empty slice for `len == 0` without touching the pointer.
///
/// # Safety
///
/// If `len > 0`, `ptr` must satisfy the requirements of [`std::slice::from_raw_parts`].
#[inline(always)]
pub(crate) unsafe fn r_slice<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

/// Mutable version of [`r_slice`] for `from_raw_parts_mut`.
///
/// # Safety
///
/// If `len > 0`, `ptr` must satisfy the requirements of [`std::slice::from_raw_parts_mut`].
#[inline(always)]
pub(crate) unsafe fn r_slice_mut<'a, T>(ptr: *mut T, len: usize) -> &'a mut [T] {
    if len == 0 {
        &mut []
    } else {
        unsafe { std::slice::from_raw_parts_mut(ptr, len) }
    }
}

#[derive(Debug, Clone, Copy)]
/// Error describing an unexpected R `SEXPTYPE`.
pub struct SexpTypeError {
    /// Expected R type.
    pub expected: SEXPTYPE,
    /// Actual R type encountered.
    pub actual: SEXPTYPE,
}

impl std::fmt::Display for SexpTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type mismatch: expected {:?}, got {:?}",
            self.expected, self.actual
        )
    }
}

impl std::error::Error for SexpTypeError {}

#[derive(Debug, Clone, Copy)]
/// Error describing an unexpected R object length.
pub struct SexpLengthError {
    /// Required length.
    pub expected: usize,
    /// Actual length encountered.
    pub actual: usize,
}

impl std::fmt::Display for SexpLengthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "length mismatch: expected {}, got {}",
            self.expected, self.actual
        )
    }
}

impl std::error::Error for SexpLengthError {}

#[derive(Debug, Clone, Copy)]
/// Error for NA values in conversions that require non-missing values.
pub struct SexpNaError {
    /// R type where an NA was found.
    pub sexp_type: SEXPTYPE,
}

impl std::fmt::Display for SexpNaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unexpected NA value in {:?}", self.sexp_type)
    }
}

impl std::error::Error for SexpNaError {}

#[derive(Debug, Clone)]
/// Unified conversion error when decoding an R `SEXP`.
pub enum SexpError {
    /// `SEXPTYPE` did not match the expected one.
    Type(SexpTypeError),
    /// Length did not match the expected one.
    Length(SexpLengthError),
    /// Missing value encountered where disallowed.
    Na(SexpNaError),
    /// Value is syntactically valid but semantically invalid (e.g. parse error).
    InvalidValue(String),
    /// A required field was missing from a named list.
    MissingField(String),
    /// A named list has duplicate non-empty names.
    DuplicateName(String),
    /// Failed to convert to `Either<L, R>` - both branches failed.
    ///
    /// Contains the error messages from attempting both conversions.
    #[cfg(feature = "either")]
    EitherConversion {
        /// Error from attempting to convert to the Left type
        left_error: String,
        /// Error from attempting to convert to the Right type
        right_error: String,
    },
}

impl std::fmt::Display for SexpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SexpError::Type(e) => write!(f, "{}", e),
            SexpError::Length(e) => write!(f, "{}", e),
            SexpError::Na(e) => write!(f, "{}", e),
            SexpError::InvalidValue(msg) => write!(f, "invalid value: {}", msg),
            SexpError::MissingField(name) => write!(f, "missing field: {}", name),
            SexpError::DuplicateName(name) => write!(f, "duplicate name in list: {:?}", name),
            #[cfg(feature = "either")]
            SexpError::EitherConversion {
                left_error,
                right_error,
            } => write!(
                f,
                "failed to convert to Either: Left failed ({}), Right failed ({})",
                left_error, right_error
            ),
        }
    }
}

impl std::error::Error for SexpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SexpError::Type(e) => Some(e),
            SexpError::Length(e) => Some(e),
            SexpError::Na(e) => Some(e),
            SexpError::InvalidValue(_) => None,
            SexpError::MissingField(_) => None,
            SexpError::DuplicateName(_) => None,
            #[cfg(feature = "either")]
            SexpError::EitherConversion { .. } => None,
        }
    }
}

impl From<SexpTypeError> for SexpError {
    fn from(e: SexpTypeError) -> Self {
        SexpError::Type(e)
    }
}

impl From<SexpLengthError> for SexpError {
    fn from(e: SexpLengthError) -> Self {
        SexpError::Length(e)
    }
}

impl From<SexpNaError> for SexpError {
    fn from(e: SexpNaError) -> Self {
        SexpError::Na(e)
    }
}

/// TryFrom-style trait for converting SEXP to Rust types.
///
/// Inbound counterpart of [`crate::into_r::IntoR`]. Strict by construction
/// (returns `Result`) — for a looser, multi-source coercion path use
/// [`crate::coerce::Coerce`] / [`crate::coerce::TryCoerce`].
///
/// # Examples
///
/// ```no_run
/// use miniextendr_api::SEXP;
/// use miniextendr_api::from_r::TryFromSexp;
///
/// fn example(sexp: SEXP) {
///     let value: i32 = TryFromSexp::try_from_sexp(sexp).unwrap();
///     let text: String = TryFromSexp::try_from_sexp(sexp).unwrap();
/// }
/// ```
pub trait TryFromSexp: Sized {
    /// The error type returned when conversion fails.
    type Error;

    /// Attempt to convert an R SEXP to this Rust type.
    ///
    /// In debug builds, may assert that we're on R's main thread.
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error>;

    /// Convert from SEXP without thread safety checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. In debug builds, this still
    /// calls the checked version by default, but implementations may
    /// skip thread assertions for performance.
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // Default: just call the checked version
        Self::try_from_sexp(sexp)
    }
}

// region: Box<[T]> delegates to Vec<T>
//
// A boxed slice converts exactly like the owned vector — read the vector, then
// `into_boxed_slice()` (an O(1), allocation-free shrink). This single blanket
// replaces every hand-rolled / macro-generated `Box<[X]>` impl: any element
// type whose `Vec<X>` is convertible gets `Box<[X]>` for free, inheriting the
// vector impl's error type and NA semantics by construction.

impl<T> TryFromSexp for Box<[T]>
where
    Vec<T>: TryFromSexp,
{
    type Error = <Vec<T> as TryFromSexp>::Error;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        <Vec<T> as TryFromSexp>::try_from_sexp(sexp).map(|v| v.into_boxed_slice())
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { <Vec<T> as TryFromSexp>::try_from_sexp_unchecked(sexp) }
            .map(|v| v.into_boxed_slice())
    }
}
// endregion

macro_rules! impl_try_from_sexp_scalar_native {
    ($t:ty, $sexptype:ident) => {
        impl TryFromSexp for $t {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::$sexptype {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::$sexptype,
                        actual,
                    }
                    .into());
                }
                let len = sexp.len();
                if len != 1 {
                    return Err(SexpLengthError {
                        expected: 1,
                        actual: len,
                    }
                    .into());
                }
                unsafe { sexp.as_slice::<$t>() }
                    .first()
                    .cloned()
                    .ok_or_else(|| {
                        SexpLengthError {
                            expected: 1,
                            actual: 0,
                        }
                        .into()
                    })
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::$sexptype {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::$sexptype,
                        actual,
                    }
                    .into());
                }
                let len = unsafe { sexp.len_unchecked() };
                if len != 1 {
                    return Err(SexpLengthError {
                        expected: 1,
                        actual: len,
                    }
                    .into());
                }
                unsafe { sexp.as_slice_unchecked::<$t>() }
                    .first()
                    .cloned()
                    .ok_or_else(|| {
                        SexpLengthError {
                            expected: 1,
                            actual: 0,
                        }
                        .into()
                    })
            }
        }
    };
}

// i32 has a bespoke impl that checks for NA_integer_ (i32::MIN).
// The shared macro is NOT used for i32 — it would silently pass NA through.
impl TryFromSexp for i32 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::INTSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::INTSXP,
                actual,
            }
            .into());
        }
        let len = sexp.len();
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }
        let v = unsafe { sexp.as_slice::<i32>() }
            .first()
            .cloned()
            .ok_or_else(|| {
                SexpError::from(SexpLengthError {
                    expected: 1,
                    actual: 0,
                })
            })?;
        if v == crate::altrep_traits::NA_INTEGER {
            return Err(SexpNaError {
                sexp_type: SEXPTYPE::INTSXP,
            }
            .into());
        }
        Ok(v)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::INTSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::INTSXP,
                actual,
            }
            .into());
        }
        let len = unsafe { sexp.len_unchecked() };
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }
        let v = unsafe { sexp.as_slice_unchecked::<i32>() }
            .first()
            .cloned()
            .ok_or_else(|| {
                SexpError::from(SexpLengthError {
                    expected: 1,
                    actual: 0,
                })
            })?;
        if v == crate::altrep_traits::NA_INTEGER {
            return Err(SexpNaError {
                sexp_type: SEXPTYPE::INTSXP,
            }
            .into());
        }
        Ok(v)
    }
}

impl_try_from_sexp_scalar_native!(f64, REALSXP);
impl_try_from_sexp_scalar_native!(u8, RAWSXP);
impl_try_from_sexp_scalar_native!(RLogical, LGLSXP);
impl_try_from_sexp_scalar_native!(crate::Rcomplex, CPLXSXP);

/// Pass-through conversion for raw SEXP values with ALTREP auto-materialization.
///
/// This allows `SEXP` to be used directly in `#[miniextendr]` function signatures.
/// When R passes an ALTREP vector (e.g., `1:10`, `seq_len(N)`),
/// [`ensure_materialized`](crate::altrep_sexp::ensure_materialized) is called
/// automatically to force materialization on the R main thread. After this,
/// the SEXP's data pointer is stable and safe to access from any thread.
///
/// # ALTREP handling
///
/// | Input | Result |
/// |---|---|
/// | Regular SEXP | Passed through unchanged |
/// | ALTREP SEXP | Materialized via `ensure_materialized`, then passed through |
///
/// To receive ALTREP without materializing, use
/// [`AltrepSexp`](crate::altrep_sexp::AltrepSexp) as the parameter type instead.
/// To receive the raw SEXP without any conversion (including no materialization),
/// use `extern "C-unwind"`.
///
/// # Safety
///
/// SEXP handles are only valid on R's main thread. Standalone `#[miniextendr]`
/// functions taking a `SEXP` parameter run on the main thread automatically.
impl TryFromSexp for SEXP {
    type Error = SexpError;

    /// Converts a SEXP, auto-materializing ALTREP vectors.
    ///
    /// If the input is ALTREP, [`ensure_materialized`](crate::altrep_sexp::ensure_materialized)
    /// is called to force materialization on the R main thread. After
    /// materialization the data pointer is stable and the SEXP can be safely
    /// sent to other threads.
    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        Ok(unsafe { crate::altrep_sexp::ensure_materialized(sexp) })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Ok(unsafe { crate::altrep_sexp::ensure_materialized(sexp) })
    }
}

impl TryFromSexp for Option<SEXP> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            Ok(Some(unsafe {
                crate::altrep_sexp::ensure_materialized(sexp)
            }))
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}
// endregion

mod logical;

mod coerced_scalars;
pub(crate) use coerced_scalars::coerce_value;

mod references;

// region: Blanket implementations for slices with arbitrary lifetimes

/// Blanket impl for `&[T]` where T: RNativeType
///
/// This replaces the macro-generated `&'static [T]` impls with a more composable
/// blanket impl that works for any lifetime. This enables containers like TinyVec
/// to use blanket impls without needing helper functions.
impl<T> TryFromSexp for &[T]
where
    T: crate::RNativeType + Copy,
{
    type Error = SexpTypeError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            });
        }
        Ok(unsafe { sexp.as_slice::<T>() })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            });
        }
        Ok(unsafe { sexp.as_slice_unchecked::<T>() })
    }
}

/// Blanket impl for `&mut [T]` where T: RNativeType
///
/// # Safety note (aliasing)
///
/// This impl hands out a `&mut` view over R's data pointer without copying, so
/// binding the same R vector to two slice parameters aliases one buffer. That is
/// undefined behavior whenever at least one borrow is mutable — two `&mut [T]`,
/// or a `&mut [T]` paired with a shared `&[T]` (which borrows the same buffer).
/// The macro-generated wrapper emits a `debug_assert!` catching this in debug
/// builds (#1104); in release the caller is responsible for not passing the same
/// SEXP to two such parameters when one is mutable.
impl<T> TryFromSexp for &mut [T]
where
    T: crate::RNativeType + Copy,
{
    type Error = SexpTypeError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            });
        }
        let len = sexp.len();
        let ptr = unsafe { T::dataptr_mut(sexp) };
        Ok(unsafe { r_slice_mut(ptr, len) })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            });
        }
        let len = unsafe { sexp.len_unchecked() };
        let ptr = unsafe { T::dataptr_mut(sexp) };
        Ok(unsafe { r_slice_mut(ptr, len) })
    }
}

/// Blanket impl for `Option<&[T]>` where T: RNativeType
impl<T> TryFromSexp for Option<&[T]>
where
    T: crate::RNativeType + Copy,
{
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp).map_err(SexpError::from)?;
        Ok(Some(slice))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let slice: &[T] =
            unsafe { TryFromSexp::try_from_sexp_unchecked(sexp).map_err(SexpError::from)? };
        Ok(Some(slice))
    }
}

/// Blanket impl for `Option<&mut [T]>` where T: RNativeType
impl<T> TryFromSexp for Option<&mut [T]>
where
    T: crate::RNativeType + Copy,
{
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let slice: &mut [T] = TryFromSexp::try_from_sexp(sexp).map_err(SexpError::from)?;
        Ok(Some(slice))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let slice: &mut [T] =
            unsafe { TryFromSexp::try_from_sexp_unchecked(sexp).map_err(SexpError::from)? };
        Ok(Some(slice))
    }
}
// endregion

mod strings;

// region: Result conversions (NULL -> Err(()))

impl<T> TryFromSexp for Result<T, ()>
where
    T: TryFromSexp,
    T::Error: Into<SexpError>,
{
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(Err(()));
        }
        let value = T::try_from_sexp(sexp).map_err(Into::into)?;
        Ok(Ok(value))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(Err(()));
        }
        let value = unsafe { T::try_from_sexp_unchecked(sexp).map_err(Into::into)? };
        Ok(Ok(value))
    }
}
// endregion

mod na_vectors;

mod collections;

mod tuples;

// region: Fixed-size array conversions

/// Blanket impl: Convert R vector to `[T; N]` where T: RNativeType.
///
/// Returns an error if the R vector length doesn't match N.
/// Useful for SHA hashes ([u8; 32]), fixed-size patterns, etc.
impl<T, const N: usize> TryFromSexp for [T; N]
where
    T: crate::RNativeType + Copy,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        if slice.len() != N {
            return Err(SexpLengthError {
                expected: N,
                actual: slice.len(),
            }
            .into());
        }

        // T: Copy, length verified above. Use MaybeUninit + copy_from_slice.
        let mut arr = std::mem::MaybeUninit::<[T; N]>::uninit();
        unsafe {
            // SAFETY: MaybeUninit<[T; N]> and [T; N] have the same layout.
            // We write all N elements via copy_from_slice, so assume_init is safe.
            let dst: &mut [T] = std::slice::from_raw_parts_mut(arr.as_mut_ptr().cast::<T>(), N);
            dst.copy_from_slice(&slice[..N]);
            Ok(arr.assume_init())
        }
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}
// endregion

// region: VecDeque conversions

use std::collections::VecDeque;

/// Blanket impl: Convert R vector to `VecDeque<T>` where T: RNativeType.
impl<T> TryFromSexp for VecDeque<T>
where
    T: crate::RNativeType + Copy,
{
    type Error = SexpTypeError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        Ok(VecDeque::from(slice.to_vec()))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(VecDeque::from(slice.to_vec()))
    }
}
// endregion

// region: BinaryHeap conversions

use std::collections::BinaryHeap;

/// Blanket impl: Convert R vector to `BinaryHeap<T>` where T: RNativeType + Ord.
///
/// Creates a binary heap from the R vector elements.
impl<T> TryFromSexp for BinaryHeap<T>
where
    T: crate::RNativeType + Copy + Ord,
{
    type Error = SexpTypeError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        Ok(BinaryHeap::from(slice.to_vec()))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(BinaryHeap::from(slice.to_vec()))
    }
}
// endregion

mod cow_and_paths;

// region: Option<Collection> conversions
//
// These convert NULL → None, and non-NULL to Some(collection).
// This differs from Option<scalar> which converts NA → None.

/// Convert R value to `Option<Vec<T>>`: NULL → None, otherwise Some(vec).
impl<T> TryFromSexp for Option<Vec<T>>
where
    Vec<T>: TryFromSexp,
    <Vec<T> as TryFromSexp>::Error: Into<SexpError>,
{
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            Vec::<T>::try_from_sexp(sexp).map(Some).map_err(Into::into)
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            unsafe {
                Vec::<T>::try_from_sexp_unchecked(sexp)
                    .map(Some)
                    .map_err(Into::into)
            }
        }
    }
}

macro_rules! impl_option_map_try_from_sexp {
    ($(#[$meta:meta])* $map_ty:ident) => {
        $(#[$meta])*
        impl<V: TryFromSexp> TryFromSexp for Option<$map_ty<String, V>>
        where
            V::Error: Into<SexpError>,
        {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    Ok(None)
                } else {
                    $map_ty::<String, V>::try_from_sexp(sexp).map(Some)
                }
            }
        }
    };
}

impl_option_map_try_from_sexp!(
    /// Convert R value to `Option<HashMap<String, V>>`: NULL -> None, otherwise Some(map).
    HashMap
);
impl_option_map_try_from_sexp!(
    /// Convert R value to `Option<BTreeMap<String, V>>`: NULL -> None, otherwise Some(map).
    BTreeMap
);

macro_rules! impl_option_set_try_from_sexp {
    ($(#[$meta:meta])* $set_ty:ident) => {
        $(#[$meta])*
        impl<T> TryFromSexp for Option<$set_ty<T>>
        where
            $set_ty<T>: TryFromSexp,
            <$set_ty<T> as TryFromSexp>::Error: Into<SexpError>,
        {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    Ok(None)
                } else {
                    $set_ty::<T>::try_from_sexp(sexp)
                        .map(Some)
                        .map_err(Into::into)
                }
            }
        }
    };
}

impl_option_set_try_from_sexp!(
    /// Convert R value to `Option<HashSet<T>>`: NULL -> None, otherwise Some(set).
    HashSet
);
impl_option_set_try_from_sexp!(
    /// Convert R value to `Option<BTreeSet<T>>`: NULL -> None, otherwise Some(set).
    BTreeSet
);
// endregion

// region: Nested vector conversions (list of vectors)

/// Convert R list (VECSXP) to `Vec<Vec<T>>`.
///
/// Each element of the R list must be convertible to `Vec<T>`.
impl<T> TryFromSexp for Vec<Vec<T>>
where
    Vec<T>: TryFromSexp,
    <Vec<T> as TryFromSexp>::Error: Into<SexpError>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        map_vecsxp_with(sexp, |_i, elem| {
            Vec::<T>::try_from_sexp(elem).map_err(Into::into)
        })
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe {
            map_vecsxp_with_unchecked(sexp, |_i, elem| {
                Vec::<T>::try_from_sexp_unchecked(elem).map_err(Into::into)
            })
        }
    }
}
// endregion

// region: Coerced wrapper - bridge between TryFromSexp and TryCoerce

use crate::coerce::Coerced;

/// Convert R value to `Coerced<T, R>` by reading `R` and coercing to `T`.
///
/// This enables reading non-native Rust types from R with coercion:
///
/// ```ignore
/// // Read i64 from R integer (i32)
/// let val: Coerced<i64, i32> = TryFromSexp::try_from_sexp(sexp)?;
/// let i64_val: i64 = val.into_inner();
///
/// // Works with collections too:
/// let vec: Vec<Coerced<i64, i32>> = ...;
/// let set: HashSet<Coerced<NonZeroU32, i32>> = ...;
/// ```
impl<T, R> TryFromSexp for Coerced<T, R>
where
    R: TryFromSexp,
    R: TryCoerce<T>,
    <R as TryFromSexp>::Error: Into<SexpError>,
    <R as TryCoerce<T>>::Error: std::fmt::Display,
{
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let r_val: R = R::try_from_sexp(sexp).map_err(Into::into)?;
        let value: T = r_val
            .try_coerce()
            .map_err(|e| SexpError::InvalidValue(format!("{e}")))?;
        Ok(Coerced::new(value))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let r_val: R = unsafe { R::try_from_sexp_unchecked(sexp).map_err(Into::into)? };
        let value: T = r_val
            .try_coerce()
            .map_err(|e| SexpError::InvalidValue(format!("{e}")))?;
        Ok(Coerced::new(value))
    }
}
// endregion

// region: Direct Vec coercion conversions
//
// These provide direct `TryFromSexp for Vec<T>` where T is not an R native type
// but can be coerced from one. This mirrors the `impl_into_r_via_coerce!` pattern
// in into_r.rs for the reverse direction.

/// Helper to coerce a slice element-wise into a Vec.
///
/// Walks the whole slice, accumulating every per-element coercion failure into
/// one batched [`SexpError::InvalidValue`] via [`BatchedErrors`] (`container`
/// names the target type, e.g. `"Vec<bool>"`) instead of bailing on the first
/// `Err`. The happy path allocates nothing for diagnostics, and a large
/// all-failing slice never builds more than [`BATCHED_ERROR_CAP`] messages.
#[inline]
fn coerce_slice_to_vec<R, T>(slice: &[R], container: &str) -> Result<Vec<T>, SexpError>
where
    R: Copy + TryCoerce<T>,
    <R as TryCoerce<T>>::Error: std::fmt::Display,
{
    let mut result = Vec::with_capacity(slice.len());
    let mut errors = BatchedErrors::default();
    for (i, v) in slice.iter().copied().enumerate() {
        match v.try_coerce() {
            Ok(x) => result.push(x),
            Err(e) => errors.push(|| format!("invalid value at index {i}: {e}")),
        }
    }
    if errors.is_empty() {
        Ok(result)
    } else {
        Err(errors.into_error(container))
    }
}

/// Drive a per-element coercion, batching every failure into one diagnostic.
///
/// Backs [`from_numeric_vec_with`]'s four SEXP-type branches: successes go into
/// the output `Vec`, failures accumulate as `"invalid value at index <i>: <err>"`
/// and are combined via [`BatchedErrors`] (matching the message grammar of the
/// `Vec<T>` arm of `try_from_sexp_via_str_parse!`). The per-element closure wraps
/// coercion failures as [`SexpError::InvalidValue`]; we unwrap that inner message
/// so the batched entry reads `"...: value out of range"` rather than the doubled
/// `"...: invalid value: value out of range"` that `SexpError`'s `Display` would
/// produce. The happy path allocates nothing for diagnostics, and a large
/// all-failing vector never builds more than [`BATCHED_ERROR_CAP`] messages.
#[inline]
fn collect_coerced<U>(
    container: &str,
    len: usize,
    iter: impl Iterator<Item = Result<U, SexpError>>,
) -> Result<Vec<U>, SexpError> {
    let mut result = Vec::with_capacity(len);
    let mut errors = BatchedErrors::default();
    for (i, item) in iter.enumerate() {
        match item {
            Ok(v) => result.push(v),
            Err(SexpError::InvalidValue(msg)) => {
                errors.push(|| format!("invalid value at index {i}: {msg}"))
            }
            Err(other) => errors.push(|| format!("invalid value at index {i}: {other}")),
        }
    }
    if errors.is_empty() {
        Ok(result)
    } else {
        Err(errors.into_error(container))
    }
}

/// Shared SEXP-dispatch shell for coerced numeric/logical/raw vectors.
///
/// Reads INTSXP/REALSXP/RAWSXP/LGLSXP and applies the per-element map. The only
/// behavioural axis (NA policy) lives entirely in the four closures the caller
/// passes, so the NA-unaware [`try_from_sexp_numeric_vec`] and the NA-aware
/// `try_from_sexp_numeric_option_vec` (in [`na_vectors`]) share one dispatch.
/// LGLSXP NA (`NA_LOGICAL`) and INTSXP NA (`i32::MIN`) round through as raw
/// sentinels here; any NA-to-`None` policy is the caller's closure to encode.
///
/// Per-element coercion failures are accumulated (not short-circuited) and
/// reported as one batched diagnostic; `container` names the target type for the
/// message (e.g. `"Vec<u32>"`, `"HashSet<i64>"`). See [`collect_coerced`].
#[inline]
pub(crate) fn from_numeric_vec_with<U, FI, FD, FR, FL>(
    sexp: SEXP,
    container: &str,
    map_i32: FI,
    map_f64: FD,
    map_u8: FR,
    map_lgl: FL,
) -> Result<Vec<U>, SexpError>
where
    FI: Fn(i32) -> Result<U, SexpError>,
    FD: Fn(f64) -> Result<U, SexpError>,
    FR: Fn(u8) -> Result<U, SexpError>,
    FL: Fn(RLogical) -> Result<U, SexpError>,
{
    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            let slice: &[i32] = unsafe { sexp.as_slice() };
            collect_coerced(container, slice.len(), slice.iter().copied().map(map_i32))
        }
        SEXPTYPE::REALSXP => {
            let slice: &[f64] = unsafe { sexp.as_slice() };
            collect_coerced(container, slice.len(), slice.iter().copied().map(map_f64))
        }
        SEXPTYPE::RAWSXP => {
            let slice: &[u8] = unsafe { sexp.as_slice() };
            collect_coerced(container, slice.len(), slice.iter().copied().map(map_u8))
        }
        SEXPTYPE::LGLSXP => {
            let slice: &[RLogical] = unsafe { sexp.as_slice() };
            collect_coerced(container, slice.len(), slice.iter().copied().map(map_lgl))
        }
        _ => Err(SexpError::InvalidValue(format!(
            "expected integer, numeric, logical, or raw; got {:?}",
            actual
        ))),
    }
}

/// Shared SEXP-dispatch shell for the STRSXP (character vector) walk.
///
/// String-side counterpart of [`from_numeric_vec_with`]: checks `STRSXP`,
/// walks each element via `string_elt`, and applies the per-element `map`
/// closure to the raw CHARSXP. NA/blank-string policy (`""` vs `None` vs
/// error) and the target representation (`String` vs `&str` vs `Cow`) are
/// entirely the caller's closure to encode — this only centralizes the type
/// check and the walk.
///
/// Mirrors `from_numeric_vec_with`'s choice to route both the checked and
/// unchecked `TryFromSexp` paths through the same (checked-FFI) walk — none
/// of the current STRSXP vector impls define a distinct unchecked fast path
/// (they fall back to `TryFromSexp`'s default `try_from_sexp_unchecked`, which
/// just calls the checked version), so there is no unchecked-FFI twin here.
#[inline]
pub(crate) fn map_strsxp_with<U>(
    sexp: SEXP,
    mut map: impl FnMut(SEXP /* charsxp */, usize) -> Result<U, SexpError>,
) -> Result<Vec<U>, SexpError> {
    let actual = sexp.type_of();
    if actual != SEXPTYPE::STRSXP {
        return Err(SexpTypeError {
            expected: SEXPTYPE::STRSXP,
            actual,
        }
        .into());
    }

    let len = sexp.len();
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let charsxp = sexp.string_elt(i as crate::R_xlen_t);
        result.push(map(charsxp, i)?);
    }
    Ok(result)
}

/// Shared scalar-STRSXP prologue: type-check + `len == 1` + `string_elt(0)`.
///
/// Returns the raw CHARSXP; NA (`SEXP::na_string()`) and blank-string
/// (`SEXP::blank_string()`) policy stay the caller's decision — some sites
/// error on NA, some map it to `""`, some to `None`, and `String`'s blank
/// handling has historically differed from `&str`'s (see audit D6 finding
/// #3), so this helper does not paper over that divergence.
#[inline]
pub(crate) fn scalar_charsxp(sexp: SEXP) -> Result<SEXP, SexpError> {
    let actual = sexp.type_of();
    if actual != SEXPTYPE::STRSXP {
        return Err(SexpTypeError {
            expected: SEXPTYPE::STRSXP,
            actual,
        }
        .into());
    }

    let len = sexp.len();
    if len != 1 {
        return Err(SexpLengthError {
            expected: 1,
            actual: len,
        }
        .into());
    }

    Ok(sexp.string_elt(0))
}

/// Unchecked-FFI variant of [`scalar_charsxp`] — uses `len_unchecked` /
/// `string_elt_unchecked`.
///
/// # Safety
///
/// Must be called from R's main thread (same contract as
/// [`TryFromSexp::try_from_sexp_unchecked`]).
#[inline]
pub(crate) unsafe fn scalar_charsxp_unchecked(sexp: SEXP) -> Result<SEXP, SexpError> {
    let actual = sexp.type_of();
    if actual != SEXPTYPE::STRSXP {
        return Err(SexpTypeError {
            expected: SEXPTYPE::STRSXP,
            actual,
        }
        .into());
    }

    let len = unsafe { sexp.len_unchecked() };
    if len != 1 {
        return Err(SexpLengthError {
            expected: 1,
            actual: len,
        }
        .into());
    }

    Ok(unsafe { sexp.string_elt_unchecked(0) })
}

/// Shared SEXP-dispatch shell for the VECSXP (list) walk.
///
/// List-side counterpart of [`from_numeric_vec_with`] / [`map_strsxp_with`]:
/// checks `VECSXP`, walks each element via `vector_elt`, and applies the
/// per-element `map` closure (which receives the element's index and SEXP).
/// Per-element policy (recursion, `NILSXP` → `None`, duplicate-pointer
/// aliasing checks, …) lives entirely in the closure.
#[inline]
pub(crate) fn map_vecsxp_with<U>(
    sexp: SEXP,
    mut map: impl FnMut(usize, SEXP) -> Result<U, SexpError>,
) -> Result<Vec<U>, SexpError> {
    let actual = sexp.type_of();
    if actual != SEXPTYPE::VECSXP {
        return Err(SexpTypeError {
            expected: SEXPTYPE::VECSXP,
            actual,
        }
        .into());
    }

    let len = sexp.len();
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let elem = sexp.vector_elt(i as crate::R_xlen_t);
        result.push(map(i, elem)?);
    }
    Ok(result)
}

/// Unchecked-FFI variant of [`map_vecsxp_with`] — uses `len_unchecked` /
/// `vector_elt_unchecked` for the type-check and walk.
///
/// # Safety
///
/// Must be called from R's main thread (same contract as
/// [`TryFromSexp::try_from_sexp_unchecked`]).
#[inline]
pub(crate) unsafe fn map_vecsxp_with_unchecked<U>(
    sexp: SEXP,
    mut map: impl FnMut(usize, SEXP) -> Result<U, SexpError>,
) -> Result<Vec<U>, SexpError> {
    let actual = sexp.type_of();
    if actual != SEXPTYPE::VECSXP {
        return Err(SexpTypeError {
            expected: SEXPTYPE::VECSXP,
            actual,
        }
        .into());
    }

    let len = unsafe { sexp.len_unchecked() };
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let elem = unsafe { sexp.vector_elt_unchecked(i as crate::R_xlen_t) };
        result.push(map(i, elem)?);
    }
    Ok(result)
}

/// Convert numeric/logical/raw vectors to `Vec<T>` with element-wise coercion.
///
/// NA-unaware: an R `NA` round-trips as the coerced sentinel rather than being
/// rejected. Bind `Vec<Option<T>>` (see [`na_vectors`]) when the caller can pass NA.
/// Per-element coercion failures batch into one diagnostic (`container` names the
/// target type for the message, e.g. `"Vec<u32>"`).
#[inline]
fn try_from_sexp_numeric_vec<T>(sexp: SEXP, container: &str) -> Result<Vec<T>, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    u8: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Display,
    <f64 as TryCoerce<T>>::Error: std::fmt::Display,
    <u8 as TryCoerce<T>>::Error: std::fmt::Display,
{
    from_numeric_vec_with(
        sexp,
        container,
        coerce_value,
        coerce_value,
        coerce_value,
        |v: RLogical| coerce_value(v.to_i32()),
    )
}

/// Implement `TryFromSexp for Vec<$target>` by coercing from integer/real/logical/raw.
macro_rules! impl_vec_try_from_sexp_numeric {
    ($target:ty) => {
        impl TryFromSexp for Vec<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                try_from_sexp_numeric_vec(sexp, concat!("Vec<", stringify!($target), ">"))
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                try_from_sexp_numeric_vec(sexp, concat!("Vec<", stringify!($target), ">"))
            }
        }
    };
}

impl_vec_try_from_sexp_numeric!(i8);
impl_vec_try_from_sexp_numeric!(i16);
impl_vec_try_from_sexp_numeric!(i64);
impl_vec_try_from_sexp_numeric!(isize);
impl_vec_try_from_sexp_numeric!(u16);
impl_vec_try_from_sexp_numeric!(u32);
impl_vec_try_from_sexp_numeric!(u64);
impl_vec_try_from_sexp_numeric!(usize);
impl_vec_try_from_sexp_numeric!(f32);

/// Convert R logical vector (LGLSXP) to `Vec<bool>` (errors on NA).
impl TryFromSexp for Vec<bool> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::LGLSXP,
                actual,
            }
            .into());
        }
        let slice: &[RLogical] = unsafe { sexp.as_slice() };
        coerce_slice_to_vec(slice, "Vec<bool>")
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}
// endregion

// region: Direct HashSet / BTreeSet coercion conversions

/// Convert numeric/logical/raw vectors to a set type with element-wise coercion.
///
/// Inherits [`try_from_sexp_numeric_vec`]'s batching; `container` names the target
/// set type for the message (e.g. `"HashSet<u32>"`).
#[inline]
fn try_from_sexp_numeric_set<T, S>(sexp: SEXP, container: &str) -> Result<S, SexpError>
where
    S: std::iter::FromIterator<T>,
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    u8: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Display,
    <f64 as TryCoerce<T>>::Error: std::fmt::Display,
    <u8 as TryCoerce<T>>::Error: std::fmt::Display,
{
    let vec = try_from_sexp_numeric_vec(sexp, container)?;
    Ok(vec.into_iter().collect())
}

macro_rules! impl_set_try_from_sexp_numeric {
    ($set_ty:ident, $target:ty) => {
        impl TryFromSexp for $set_ty<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                try_from_sexp_numeric_set(
                    sexp,
                    concat!(stringify!($set_ty), "<", stringify!($target), ">"),
                )
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                try_from_sexp_numeric_set(
                    sexp,
                    concat!(stringify!($set_ty), "<", stringify!($target), ">"),
                )
            }
        }
    };
}

impl_set_try_from_sexp_numeric!(HashSet, i8);
impl_set_try_from_sexp_numeric!(HashSet, i16);
impl_set_try_from_sexp_numeric!(HashSet, i64);
impl_set_try_from_sexp_numeric!(HashSet, isize);
impl_set_try_from_sexp_numeric!(HashSet, u16);
impl_set_try_from_sexp_numeric!(HashSet, u32);
impl_set_try_from_sexp_numeric!(HashSet, u64);
impl_set_try_from_sexp_numeric!(HashSet, usize);

impl_set_try_from_sexp_numeric!(BTreeSet, i8);
impl_set_try_from_sexp_numeric!(BTreeSet, i16);
impl_set_try_from_sexp_numeric!(BTreeSet, i64);
impl_set_try_from_sexp_numeric!(BTreeSet, isize);
impl_set_try_from_sexp_numeric!(BTreeSet, u16);
impl_set_try_from_sexp_numeric!(BTreeSet, u32);
impl_set_try_from_sexp_numeric!(BTreeSet, u64);
impl_set_try_from_sexp_numeric!(BTreeSet, usize);

macro_rules! impl_set_try_from_sexp_bool {
    ($set_ty:ident) => {
        impl TryFromSexp for $set_ty<bool> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let vec: Vec<bool> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(vec.into_iter().collect())
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                Self::try_from_sexp(sexp)
            }
        }
    };
}

impl_set_try_from_sexp_bool!(HashSet);
impl_set_try_from_sexp_bool!(BTreeSet);
// endregion

// region: ExternalPtr conversions

use crate::externalptr::{ExternalPtr, TypeMismatchError, TypedExternal};

/// Map a downcast [`TypeMismatchError`] to the [`SexpError`] surfaced from
/// `ExternalPtr<T>` argument conversion. Shared by the checked and unchecked
/// `TryFromSexp` paths below.
fn type_mismatch_to_sexp_error(e: TypeMismatchError) -> SexpError {
    match e {
        TypeMismatchError::NullPointer => {
            SexpError::InvalidValue("external pointer is null".to_string())
        }
        TypeMismatchError::InvalidTypeId => {
            SexpError::InvalidValue("external pointer has no valid type id".to_string())
        }
        TypeMismatchError::Mismatch { expected, found } => SexpError::InvalidValue(format!(
            "type mismatch: expected `{}`, found `{}`",
            expected, found
        )),
    }
}

/// Error for an `ExternalPtr<T>` argument that is neither a bare `EXTPTRSXP`
/// nor a class handle wrapping one (audit A9 — class-wrapped handles like
/// `Foo$new(...)` are unwrapped automatically; this fires only when *no*
/// `.ptr`/slot/attribute could be recovered at all).
fn not_a_handle_error(actual: SEXPTYPE) -> SexpError {
    SexpError::InvalidValue(format!(
        "expected an external pointer or a miniextendr class object wrapping one, got {:?}",
        actual
    ))
}

/// Convert R EXTPTRSXP to `ExternalPtr<T>`.
///
/// This enables using `ExternalPtr<T>` as parameter types in `#[miniextendr]` functions.
///
/// # Example
///
/// ```ignore
/// #[derive(ExternalPtr)]
/// struct MyData { value: i32 }
///
/// #[miniextendr]
/// fn process(data: ExternalPtr<MyData>) -> i32 {
///     data.value
/// }
/// ```
impl<T: TypedExternal + Send> TryFromSexp for ExternalPtr<T> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::EXTPTRSXP {
            // Not a bare pointer — try unwrapping a class-wrapped handle
            // (R6 `private$.ptr`, S4 `ptr` slot, S7 `.ptr` attribute; Env/S3
            // handles are already bare EXTPTRSXPs and never reach here).
            return match unsafe { crate::externalptr::unwrap_class_handle(sexp) } {
                Some(inner) => unsafe { ExternalPtr::wrap_sexp_with_error(inner) }
                    .map_err(type_mismatch_to_sexp_error),
                None => Err(not_a_handle_error(actual)),
            };
        }

        // Use ExternalPtr's type-checked constructor
        unsafe { ExternalPtr::wrap_sexp_with_error(sexp) }.map_err(type_mismatch_to_sexp_error)
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::EXTPTRSXP {
            return match unsafe { crate::externalptr::unwrap_class_handle(sexp) } {
                Some(inner) => {
                    unsafe { ExternalPtr::wrap_sexp_unchecked(inner) }.ok_or_else(|| {
                        SexpError::InvalidValue(
                            "failed to convert external pointer: type mismatch or null pointer"
                                .to_string(),
                        )
                    })
                }
                None => Err(not_a_handle_error(actual)),
            };
        }

        // Use ExternalPtr's type-checked constructor (unchecked variant)
        unsafe { ExternalPtr::wrap_sexp_unchecked(sexp) }.ok_or_else(|| {
            SexpError::InvalidValue(
                "failed to convert external pointer: type mismatch or null pointer".to_string(),
            )
        })
    }
}

impl<T: TypedExternal + Send> TryFromSexp for Option<ExternalPtr<T>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let ptr: ExternalPtr<T> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(Some(ptr))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let ptr: ExternalPtr<T> = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(Some(ptr))
    }
}

/// Convert an R list (VECSXP) of external pointers to `Vec<ExternalPtr<T>>`.
///
/// Each element must be an `EXTPTRSXP` carrying a `T`; conversion delegates to
/// [`ExternalPtr::<T>::try_from_sexp`] per element. This lets `#[miniextendr]`
/// functions accept an R `list()` of opaque handles (issue #827). The blanket
/// `impl_vec_try_from_sexp_list!` macro can't be used downstream for this — the
/// orphan rule rejects `impl TryFromSexp for Vec<ExternalPtr<T>>` in user crates
/// because both `Vec` and `TryFromSexp` are foreign there — so the impl lives
/// here, keyed on `ExternalPtr<T>` to avoid colliding with the atomic-vector impls.
impl<T: TypedExternal + Send> TryFromSexp for Vec<ExternalPtr<T>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        map_vecsxp_with(sexp, |_i, elem| {
            <ExternalPtr<T> as TryFromSexp>::try_from_sexp(elem)
        })
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe {
            map_vecsxp_with_unchecked(sexp, |_i, elem| {
                <ExternalPtr<T> as TryFromSexp>::try_from_sexp_unchecked(elem)
            })
        }
    }
}

/// Convert an R list (VECSXP) of external pointers / `NULL`s to
/// `Vec<Option<ExternalPtr<T>>>`. `NULL` elements map to `None`; every other
/// element must be an `EXTPTRSXP` carrying a `T` (issue #827).
impl<T: TypedExternal + Send> TryFromSexp for Vec<Option<ExternalPtr<T>>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        map_vecsxp_with(sexp, |_i, elem| {
            <Option<ExternalPtr<T>> as TryFromSexp>::try_from_sexp(elem)
        })
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe {
            map_vecsxp_with_unchecked(sexp, |_i, elem| {
                <Option<ExternalPtr<T>> as TryFromSexp>::try_from_sexp_unchecked(elem)
            })
        }
    }
}
// endregion

// region: R connections — TryFromSexp impls (issue #175, #176)

#[cfg(feature = "connections")]
mod connections_from_r {
    use std::ffi::CStr;

    use crate::connection::{RNullConnection, RStderr, RStdin, RStdout, Rconn};
    use crate::from_r::{SexpError, TryFromSexp};
    use crate::{Rboolean, SEXP};

    // Read the connection description and class fields from an Rconn handle.
    //
    // # Safety
    // - sexp must be a valid, open R connection SEXP.
    // - Must be called from the R main thread.
    unsafe fn conn_description(sexp: SEXP) -> Option<String> {
        unsafe {
            let handle = crate::sys::R_GetConnection(sexp);
            let conn = handle.cast::<Rconn>().cast_const();
            if (*conn).description.is_null() {
                None
            } else {
                Some(
                    CStr::from_ptr((*conn).description)
                        .to_string_lossy()
                        .into_owned(),
                )
            }
        }
    }

    unsafe fn conn_class(sexp: SEXP) -> Option<String> {
        unsafe {
            let handle = crate::sys::R_GetConnection(sexp);
            let conn = handle.cast::<Rconn>().cast_const();
            if (*conn).class.is_null() {
                None
            } else {
                Some(CStr::from_ptr((*conn).class).to_string_lossy().into_owned())
            }
        }
    }

    unsafe fn conn_canwrite(sexp: SEXP) -> bool {
        unsafe {
            let handle = crate::sys::R_GetConnection(sexp);
            let conn = handle.cast::<Rconn>().cast_const();
            (*conn).canwrite != Rboolean::FALSE
        }
    }

    unsafe fn conn_isopen(sexp: SEXP) -> bool {
        unsafe {
            let handle = crate::sys::R_GetConnection(sexp);
            let conn = handle.cast::<Rconn>().cast_const();
            (*conn).isopen != Rboolean::FALSE
        }
    }

    // Strict validation: confirm description == expected_desc and class == "terminal".
    unsafe fn validate_terminal(sexp: SEXP, expected_desc: &str) -> Result<(), SexpError> {
        let desc = unsafe { conn_description(sexp) }.unwrap_or_default();
        if desc != expected_desc {
            return Err(SexpError::InvalidValue(format!(
                "expected terminal connection with description {:?}, got {:?}",
                expected_desc, desc
            )));
        }
        let cls = unsafe { conn_class(sexp) }.unwrap_or_default();
        if cls != "terminal" {
            return Err(SexpError::InvalidValue(format!(
                "expected class \"terminal\", got {:?}",
                cls
            )));
        }
        Ok(())
    }

    impl TryFromSexp for RStdin {
        type Error = SexpError;

        fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
            unsafe { validate_terminal(sexp, "stdin") }?;
            Ok(RStdin)
        }

        unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
            Self::try_from_sexp(sexp)
        }
    }

    impl TryFromSexp for RStdout {
        type Error = SexpError;

        fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
            unsafe { validate_terminal(sexp, "stdout") }?;
            Ok(RStdout)
        }

        unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
            Self::try_from_sexp(sexp)
        }
    }

    impl TryFromSexp for RStderr {
        type Error = SexpError;

        fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
            unsafe { validate_terminal(sexp, "stderr") }?;
            Ok(RStderr)
        }

        unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
            Self::try_from_sexp(sexp)
        }
    }

    /// Accepts any open, write-capable connection — not just the null device.
    ///
    /// This is intentional: validating against `description == "/dev/null"` /
    /// `"NUL"` is brittle across platforms, and the type's value comes from the
    /// RAII close-on-drop, not the specific target. Substituting a `file()`
    /// connection for `RNullConnection` is supported.
    impl TryFromSexp for RNullConnection {
        type Error = SexpError;

        fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
            if !unsafe { conn_isopen(sexp) } {
                return Err(SexpError::InvalidValue(
                    "expected an open connection".to_string(),
                ));
            }
            if !unsafe { conn_canwrite(sexp) } {
                return Err(SexpError::InvalidValue(
                    "expected a write-capable connection".to_string(),
                ));
            }
            // Preserve the SEXP so it lives as long as this Rust struct.
            unsafe { crate::sys::R_PreserveObject(sexp) };
            Ok(unsafe { RNullConnection::from_preserved_sexp(sexp) })
        }

        unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
            Self::try_from_sexp(sexp)
        }
    }
}

// endregion

// region: txtProgressBar — TryFromSexp (issue #177)

#[cfg(feature = "connections")]
mod txt_progress_bar_from_r {
    use crate::from_r::{SexpError, TryFromSexp};
    use crate::sys::R_PreserveObject;
    use crate::txt_progress_bar::RTxtProgressBar;
    use crate::{SEXP, SexpExt};

    impl TryFromSexp for RTxtProgressBar {
        type Error = SexpError;

        fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
            // Must be a list (VECSXP) with class "txtProgressBar".
            if !sexp.inherits_class(c"txtProgressBar") {
                return Err(SexpError::InvalidValue(
                    "expected a SEXP with class \"txtProgressBar\"".to_string(),
                ));
            }
            // Pin on the precious list so GC cannot collect while Rust holds it.
            unsafe { R_PreserveObject(sexp) };
            Ok(unsafe { RTxtProgressBar::from_preserved_sexp(sexp) })
        }

        unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
            Self::try_from_sexp(sexp)
        }
    }
}

// endregion

// region: Helper macros for feature-gated modules

/// Implement `TryFromSexp for Option<T>` where T already implements TryFromSexp.
///
/// NULL → None, otherwise delegates to T::try_from_sexp and wraps in Some.
#[macro_export]
macro_rules! impl_option_try_from_sexp {
    ($t:ty) => {
        impl $crate::from_r::TryFromSexp for Option<$t> {
            type Error = $crate::from_r::SexpError;

            fn try_from_sexp(sexp: $crate::SEXP) -> Result<Self, Self::Error> {
                use $crate::{SEXPTYPE, SexpExt};
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                <$t as $crate::from_r::TryFromSexp>::try_from_sexp(sexp).map(Some)
            }

            unsafe fn try_from_sexp_unchecked(sexp: $crate::SEXP) -> Result<Self, Self::Error> {
                use $crate::{SEXPTYPE, SexpExt};
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                unsafe {
                    <$t as $crate::from_r::TryFromSexp>::try_from_sexp_unchecked(sexp).map(Some)
                }
            }
        }
    };
}

/// Implement `TryFromSexp for Vec<T>` from R list (VECSXP).
///
/// Each element is converted via T::try_from_sexp.
#[macro_export]
macro_rules! impl_vec_try_from_sexp_list {
    ($t:ty) => {
        impl $crate::from_r::TryFromSexp for Vec<$t> {
            type Error = $crate::from_r::SexpError;

            fn try_from_sexp(sexp: $crate::SEXP) -> Result<Self, Self::Error> {
                use $crate::from_r::SexpTypeError;
                use $crate::{SEXPTYPE, SexpExt};

                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut result = Vec::with_capacity(len);
                for i in 0..len {
                    let elem = sexp.vector_elt(i as $crate::R_xlen_t);
                    result.push(<$t as $crate::from_r::TryFromSexp>::try_from_sexp(elem)?);
                }
                Ok(result)
            }

            unsafe fn try_from_sexp_unchecked(sexp: $crate::SEXP) -> Result<Self, Self::Error> {
                use $crate::from_r::SexpTypeError;
                use $crate::{SEXPTYPE, SexpExt};

                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = unsafe { sexp.len_unchecked() };
                let mut result = Vec::with_capacity(len);
                for i in 0..len {
                    let elem = unsafe { sexp.vector_elt_unchecked(i as $crate::R_xlen_t) };
                    result.push(unsafe {
                        <$t as $crate::from_r::TryFromSexp>::try_from_sexp_unchecked(elem)?
                    });
                }
                Ok(result)
            }
        }
    };
}

/// Implement `TryFromSexp for Vec<Option<T>>` from R list (VECSXP).
///
/// NULL elements become None, others are converted via T::try_from_sexp.
#[macro_export]
macro_rules! impl_vec_option_try_from_sexp_list {
    ($t:ty) => {
        impl $crate::from_r::TryFromSexp for Vec<Option<$t>> {
            type Error = $crate::from_r::SexpError;

            fn try_from_sexp(sexp: $crate::SEXP) -> Result<Self, Self::Error> {
                use $crate::from_r::SexpTypeError;
                use $crate::{SEXPTYPE, SexpExt};

                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut result = Vec::with_capacity(len);
                for i in 0..len {
                    let elem = sexp.vector_elt(i as $crate::R_xlen_t);
                    if elem == $crate::SEXP::nil() {
                        result.push(None);
                    } else {
                        result.push(Some(<$t as $crate::from_r::TryFromSexp>::try_from_sexp(
                            elem,
                        )?));
                    }
                }
                Ok(result)
            }

            unsafe fn try_from_sexp_unchecked(sexp: $crate::SEXP) -> Result<Self, Self::Error> {
                use $crate::from_r::SexpTypeError;
                use $crate::{SEXPTYPE, SexpExt};

                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = unsafe { sexp.len_unchecked() };
                let mut result = Vec::with_capacity(len);
                for i in 0..len {
                    let elem = unsafe { sexp.vector_elt_unchecked(i as $crate::R_xlen_t) };
                    if elem == $crate::SEXP::nil() {
                        result.push(None);
                    } else {
                        result.push(Some(unsafe {
                            <$t as $crate::from_r::TryFromSexp>::try_from_sexp_unchecked(elem)?
                        }));
                    }
                }
                Ok(result)
            }
        }
    };
}

/// Cap on the number of per-element failures listed in a batched vector
/// conversion error; the remainder is summarized as `"and N more"`.
///
/// `pub(crate)` (rather than file-private) so [`crate::into_r_as`]'s
/// `StorageCoerceError::Batched` (#1097) shares the exact same cap instead of
/// drifting from the inbound [`BatchedErrors`] path.
pub(crate) const BATCHED_ERROR_CAP: usize = 10;

/// Bounded accumulator that folds indexed per-element conversion failures into
/// one batched [`SexpError::InvalidValue`].
///
/// Backs the `Vec<T>` / `Vec<Option<T>>` arms of
/// [`try_from_sexp_via_str_parse!`] (string-parse paths, #1143) **and** the
/// numeric-coercion vector shells [`from_numeric_vec_with`] / [`collect_coerced`]
/// / [`coerce_slice_to_vec`] (#1192): instead of bailing on the first NA or
/// coercion failure, those walk the whole vector and record each failure here.
///
/// Only the first [`BATCHED_ERROR_CAP`] messages are retained; every later
/// failure is counted but its message closure is never invoked. This keeps an
/// all-failing N-element vector from materialising N `String`s just to discard
/// all but 10 — the memory held is bounded regardless of input size. On
/// [`into_error`](Self::into_error) the retained entries are joined with `"; "`
/// and the remainder is summarized as `"and N more"`.
///
/// Public (but hidden) because `try_from_sexp_via_str_parse!` is
/// `#[macro_export]` and expands in downstream crates — not intended to be
/// used directly.
#[doc(hidden)]
#[derive(Default)]
pub struct BatchedErrors {
    listed: Vec<String>,
    total: usize,
}

impl BatchedErrors {
    /// Record one per-element failure. `msg` is evaluated (and its `String`
    /// allocated) only for the first [`BATCHED_ERROR_CAP`] failures; later ones
    /// are counted for the `"and N more"` tail but never formatted.
    #[inline]
    pub fn push(&mut self, msg: impl FnOnce() -> String) {
        if self.listed.len() < BATCHED_ERROR_CAP {
            self.listed.push(msg());
        }
        self.total += 1;
    }

    /// Whether any failure has been recorded.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.total == 0
    }

    /// Fold the recorded failures into one [`SexpError::InvalidValue`] under
    /// `container` (e.g. `"Vec<u32>"`).
    pub fn into_error(self, container: &str) -> SexpError {
        debug_assert!(self.total > 0, "batching zero conversion errors");
        let mut msg = format!("{container} conversion failed: {}", self.listed.join("; "));
        if self.total > self.listed.len() {
            use std::fmt::Write;
            let _ = write!(msg, "; and {} more", self.total - self.listed.len());
        }
        SexpError::InvalidValue(msg)
    }
}

/// Implement the four string-parse `TryFromSexp` impls (`T`, `Option<T>`,
/// `Vec<T>`, `Vec<Option<T>>`) for a type parsed from an R character vector.
///
/// Sibling of [`into_r_infallible!`](crate::into_r) for the reverse direction:
/// every "parse a scalar type out of an R string" integration (uuid, url,
/// regex, num-bigint) used to hand-write these four impls — some reinventing
/// the STRSXP validation `String`'s own `TryFromSexp` already performs.
/// This macro delegates to `Option<String>` / `Vec<Option<String>>`, so type,
/// length, and NA checks live in exactly one place.
///
/// Semantics:
/// - `T`: `NA_character_` / `NULL` → `SexpError::Na`; parse failure →
///   `InvalidValue("invalid <label>: <err>")`.
/// - `Option<T>`: `NA_character_` / `NULL` → `None`.
/// - `Vec<T>`: NA elements and parse failures are collected across the whole
///   vector into one batched `InvalidValue` (see [`BatchedErrors`]).
///   Per-element entries keep the `"NA at index <i> not allowed for Vec<T>"`
///   and `"invalid <label> at index <i>: <err>"` shapes; the first 10 are
///   listed and the remainder is summarized as `"and N more"`.
/// - `Vec<Option<T>>`: NA elements → `None`; parse failures batch as above.
///
/// The parse body is a closure-style `|s| expr` where `s: &str`, returning
/// `Result<T, E>` with `E: Display`.
///
/// ```ignore
/// try_from_sexp_via_str_parse!(Uuid, "UUID", |s| Uuid::parse_str(s));
/// ```
#[macro_export]
macro_rules! try_from_sexp_via_str_parse {
    ($ty:ty, $label:literal, |$s:ident| $parse:expr) => {
        impl $crate::from_r::TryFromSexp for $ty {
            type Error = $crate::from_r::SexpError;

            fn try_from_sexp(sexp: $crate::SEXP) -> Result<Self, Self::Error> {
                let opt: Option<String> = $crate::from_r::TryFromSexp::try_from_sexp(sexp)?;
                let s = opt.ok_or($crate::from_r::SexpError::Na($crate::from_r::SexpNaError {
                    sexp_type: $crate::SEXPTYPE::STRSXP,
                }))?;
                let $s: &str = &s;
                ($parse).map_err(|e| {
                    $crate::from_r::SexpError::InvalidValue(format!(
                        concat!("invalid ", $label, ": {}"),
                        e
                    ))
                })
            }
        }

        impl $crate::from_r::TryFromSexp for Option<$ty> {
            type Error = $crate::from_r::SexpError;

            fn try_from_sexp(sexp: $crate::SEXP) -> Result<Self, Self::Error> {
                let opt: Option<String> = $crate::from_r::TryFromSexp::try_from_sexp(sexp)?;
                match opt {
                    None => Ok(None),
                    Some(s) => {
                        let $s: &str = &s;
                        ($parse).map(Some).map_err(|e| {
                            $crate::from_r::SexpError::InvalidValue(format!(
                                concat!("invalid ", $label, ": {}"),
                                e
                            ))
                        })
                    }
                }
            }
        }

        impl $crate::from_r::TryFromSexp for Vec<$ty> {
            type Error = $crate::from_r::SexpError;

            fn try_from_sexp(sexp: $crate::SEXP) -> Result<Self, Self::Error> {
                let values: Vec<Option<String>> = $crate::from_r::TryFromSexp::try_from_sexp(sexp)?;
                let mut result = Vec::with_capacity(values.len());
                let mut errors = $crate::from_r::BatchedErrors::default();
                for (i, opt) in values.into_iter().enumerate() {
                    match opt {
                        None => errors.push(|| {
                            format!(
                                concat!(
                                    "NA at index {} not allowed for Vec<",
                                    stringify!($ty),
                                    ">"
                                ),
                                i
                            )
                        }),
                        Some(s) => {
                            let $s: &str = &s;
                            match ($parse) {
                                Ok(v) => result.push(v),
                                Err(e) => errors.push(|| {
                                    format!(concat!("invalid ", $label, " at index {}: {}"), i, e)
                                }),
                            }
                        }
                    }
                }
                if errors.is_empty() {
                    Ok(result)
                } else {
                    Err(errors.into_error(concat!("Vec<", stringify!($ty), ">")))
                }
            }
        }

        impl $crate::from_r::TryFromSexp for Vec<Option<$ty>> {
            type Error = $crate::from_r::SexpError;

            fn try_from_sexp(sexp: $crate::SEXP) -> Result<Self, Self::Error> {
                let values: Vec<Option<String>> = $crate::from_r::TryFromSexp::try_from_sexp(sexp)?;
                let mut result = Vec::with_capacity(values.len());
                let mut errors = $crate::from_r::BatchedErrors::default();
                for (i, opt) in values.into_iter().enumerate() {
                    match opt {
                        None => result.push(None),
                        Some(s) => {
                            let $s: &str = &s;
                            match ($parse) {
                                Ok(v) => result.push(Some(v)),
                                Err(e) => errors.push(|| {
                                    format!(concat!("invalid ", $label, " at index {}: {}"), i, e)
                                }),
                            }
                        }
                    }
                }
                if errors.is_empty() {
                    Ok(result)
                } else {
                    Err(errors.into_error(concat!("Vec<Option<", stringify!($ty), ">>")))
                }
            }
        }
    };
}
// endregion
