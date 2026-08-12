//! Cow, PathBuf, OsString, and string collection conversions.
//!
//! - `Cow<'static, [T]>` — zero-copy borrow of R native vectors
//! - `Cow<'static, str>` — zero-copy borrow of R character scalars
//! - `PathBuf` / `OsString` — from STRSXP via `String` intermediary
//! - `HashSet<String>` / `BTreeSet<String>` — string set conversions
//!
//! # Tradeoff
//!
//! These [`TryFromSexp`] impls reject mismatched
//! [`SEXPTYPE`](crate::SEXPTYPE)s — there is no looser coercion path for `Cow` / `PathBuf` /
//! `OsString`. The `'static` lifetime on `Cow` borrows is valid only for the
//! duration of the enclosing `.Call`; if you need an owned value that
//! outlives R's GC, take `String` or `Vec<T>` instead (see
//! [`strings`](crate::from_r::strings) and [`references`](crate::from_r::references)).

use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};
use std::ffi::OsString;
use std::path::PathBuf;

use crate::SEXP;
use crate::from_r::{
    SexpError, SexpTypeError, TryFromSexp, charsxp_to_cow, charsxp_to_str, map_strsxp_with,
};

/// Blanket impl: Convert R vector to `Cow<'static, [T]>` where T: RNativeType.
///
/// Returns `Cow::Borrowed` — the slice points directly into R's SEXP data with
/// no copy. The `'static` lifetime is valid for the duration of the `.Call`
/// invocation (R protects the SEXP from GC while Rust code is running).
///
/// **Important:** Do not send the borrowed `Cow` to another thread or store it
/// past the `.Call` return — the underlying R memory is only valid while
/// R's protection stack guards this SEXP.
impl<T> TryFromSexp for Cow<'static, [T]>
where
    T: crate::RNativeType + Copy + Clone,
{
    type Error = SexpTypeError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        Ok(Cow::Borrowed(slice))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(Cow::Borrowed(slice))
    }
}

/// Convert R character scalar to `Cow<'static, str>`.
///
/// Returns `Cow::Borrowed` when the CHARSXP is already UTF-8/ASCII. Other text
/// encodings are translated to an owned UTF-8 string. The `'static` lifetime
/// on a borrowed value is valid for the duration of the `.Call` invocation.
///
/// Use `Cow` when your code may need to mutate the string later — `to_mut()`
/// copies UTF-8/ASCII inputs on write, while translated inputs are already owned.
impl TryFromSexp for Cow<'static, str> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let charsxp = crate::from_r::scalar_charsxp(sexp)?;
        if charsxp == SEXP::na_string() || charsxp == SEXP::blank_string() {
            return Ok(Cow::Borrowed(""));
        }
        Ok(unsafe { charsxp_to_cow(charsxp) })
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

/// Convert R character vector to `Vec<Cow<'static, str>>` — zero-copy per element.
///
/// UTF-8/ASCII elements borrow directly from R's CHARSXP data. Other text
/// encodings are translated and stored as `Cow::Owned`.
///
/// # NA Handling
///
/// **Warning:** `NA_character_` is converted to `Cow::Borrowed("")`. This is lossy!
/// Use `Vec<Option<Cow<'static, str>>>` to distinguish NA from empty strings.
impl TryFromSexp for Vec<Cow<'static, str>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        map_strsxp_with(sexp, |charsxp, _i| {
            if charsxp == SEXP::na_string() || charsxp == SEXP::blank_string() {
                Ok(Cow::Borrowed(""))
            } else {
                Ok(unsafe { charsxp_to_cow(charsxp) })
            }
        })
    }
}

/// Convert R character vector to `Vec<Option<Cow<'static, str>>>` — zero-copy, NA-aware.
///
/// `NA_character_` → `None`, valid strings → `Some(Cow::Borrowed(&str))`.
impl TryFromSexp for Vec<Option<Cow<'static, str>>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        map_strsxp_with(sexp, |charsxp, _i| {
            if charsxp == SEXP::na_string() {
                Ok(None)
            } else {
                // charsxp_to_cow returns Cow::Borrowed("") for R_BlankString-equivalent
                Ok(Some(unsafe { charsxp_to_cow(charsxp) }))
            }
        })
    }
}

/// Convert R character vector to `Vec<String>`.
///
/// # NA and Encoding Handling
///
/// **Warning:** This conversion is lossy for NA values and encoding failures:
/// - `NA_character_` values are converted to empty string `""`
/// - Encoding translation failures become empty string `""`
/// - Invalid UTF-8 (after translation) becomes empty string `""`
///
/// If you need to preserve NA semantics, use `Vec<Option<String>>` instead:
///
/// ```ignore
/// let strings: Vec<Option<String>> = sexp.try_into()?;
/// // NA values will be None, valid strings will be Some(s)
/// ```
///
/// This design choice prioritizes convenience over strict correctness for the
/// common case where strings are known to be non-NA and properly encoded.
impl TryFromSexp for Vec<String> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        map_strsxp_with(sexp, |charsxp, _i| {
            let s = if charsxp == SEXP::na_string() {
                String::new()
            } else {
                unsafe { charsxp_to_str(charsxp) }.to_owned()
            };
            Ok(s)
        })
    }
}

/// Convert R character vector to `Vec<&str>`.
///
/// **Warning:** `NA_character_` values are converted to empty string `""`.
impl TryFromSexp for Vec<&'static str> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        map_strsxp_with(sexp, |charsxp, _i| {
            if charsxp == SEXP::na_string() || charsxp == SEXP::blank_string() {
                return Ok("");
            }
            Ok(unsafe { charsxp_to_str(charsxp) })
        })
    }
}

/// Convert R character vector to `Vec<Option<&str>>`.
impl TryFromSexp for Vec<Option<&'static str>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        map_strsxp_with(sexp, |charsxp, _i| {
            if charsxp == SEXP::na_string() {
                return Ok(None);
            }
            if charsxp == SEXP::blank_string() {
                return Ok(Some(""));
            }
            Ok(Some(unsafe { charsxp_to_str(charsxp) }))
        })
    }
}

macro_rules! impl_set_string_try_from_sexp {
    ($(#[$meta:meta])* $set_ty:ident) => {
        $(#[$meta])*
        impl TryFromSexp for $set_ty<String> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let vec: Vec<String> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(vec.into_iter().collect())
            }
        }
    };
}

impl_set_string_try_from_sexp!(
    /// Convert R character vector to `HashSet<String>`.
    HashSet
);
impl_set_string_try_from_sexp!(
    /// Convert R character vector to `BTreeSet<String>`.
    BTreeSet
);
// endregion

// region: String-wrapper type conversions (PathBuf, OsString)

/// Generate TryFromSexp impls for types that are `From<String>` (scalar, Option,
/// Vec, `Vec<Option>`). Used for PathBuf and OsString which delegate to String conversion.
macro_rules! impl_string_wrapper_try_from_sexp {
    (
        $(#[$scalar_meta:meta])*
        scalar: $ty:ty;
        $(#[$option_meta:meta])*
        option: $ty2:ty;
        $(#[$vec_meta:meta])*
        vec: $ty3:ty;
        $(#[$vec_option_meta:meta])*
        vec_option: $ty4:ty;
    ) => {
        $(#[$scalar_meta])*
        impl TryFromSexp for $ty {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let s: String = TryFromSexp::try_from_sexp(sexp)?;
                Ok(<$ty>::from(s))
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let s: String = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(<$ty>::from(s))
            }
        }

        $(#[$option_meta])*
        impl TryFromSexp for Option<$ty> {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let opt: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(opt.map(<$ty>::from))
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let opt: Option<String> = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(opt.map(<$ty>::from))
            }
        }

        $(#[$vec_meta])*
        impl TryFromSexp for Vec<$ty> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let vec: Vec<String> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(vec.into_iter().map(<$ty>::from).collect())
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let vec: Vec<String> = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(vec.into_iter().map(<$ty>::from).collect())
            }
        }

        $(#[$vec_option_meta])*
        impl TryFromSexp for Vec<Option<$ty>> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let vec: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(vec.into_iter().map(|opt| opt.map(<$ty>::from)).collect())
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let vec: Vec<Option<String>> = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(vec.into_iter().map(|opt| opt.map(<$ty>::from)).collect())
            }
        }
    };
}

impl_string_wrapper_try_from_sexp!(
    /// Convert R character scalar (STRSXP of length 1) to `PathBuf`.
    ///
    /// # NA Handling
    ///
    /// **Warning:** `NA_character_` is converted to empty path `""`. This is lossy!
    /// If you need to distinguish between NA and empty strings, use `Option<PathBuf>` instead.
    scalar: PathBuf;
    /// NA-aware PathBuf conversion: returns `None` for `NA_character_` or `NULL`.
    option: PathBuf;
    /// Convert R character vector (STRSXP) to `Vec<PathBuf>`.
    ///
    /// # NA Handling
    ///
    /// **Warning:** `NA_character_` elements are converted to empty paths.
    /// Use `Vec<Option<PathBuf>>` if you need to preserve NA values.
    vec: PathBuf;
    /// Convert R character vector (STRSXP) to `Vec<Option<PathBuf>>` with NA support.
    ///
    /// `NA_character_` elements are converted to `None`.
    vec_option: PathBuf;
);

impl_string_wrapper_try_from_sexp!(
    /// Convert R character scalar (STRSXP of length 1) to `OsString`.
    ///
    /// Since R strings are converted to UTF-8, the resulting `OsString` contains
    /// valid UTF-8 data.
    ///
    /// # NA Handling
    ///
    /// **Warning:** `NA_character_` is converted to empty string. This is lossy!
    /// If you need to distinguish between NA and empty strings, use `Option<OsString>` instead.
    scalar: OsString;
    /// NA-aware OsString conversion: returns `None` for `NA_character_` or `NULL`.
    option: OsString;
    /// Convert R character vector (STRSXP) to `Vec<OsString>`.
    ///
    /// # NA Handling
    ///
    /// **Warning:** `NA_character_` elements are converted to empty strings.
    /// Use `Vec<Option<OsString>>` if you need to preserve NA values.
    vec: OsString;
    /// Convert R character vector (STRSXP) to `Vec<Option<OsString>>` with NA support.
    ///
    /// `NA_character_` elements are converted to `None`.
    vec_option: OsString;
);
// endregion
