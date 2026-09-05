//! String conversions — STRSXP requires special handling via `STRING_ELT`.
//!
//! R stores strings as STRSXP (vector of CHARSXP). Each element requires
//! `STRING_ELT` + `R_CHAR` to extract, unlike numeric vectors which expose
//! a contiguous data pointer.
//!
//! Covers: `&str`, `String`, `char`, `Option<&str>`, `Option<String>`,
//! `Vec<String>`, `Vec<&str>`, `Box<[String]>`.
//!
//! # Tradeoff
//!
//! Prefer borrowed `&str` / `Vec<&str>` over owned `String` / `Vec<String>`
//! when the data only needs to live for the `.Call` — borrowed strings point
//! straight into R's CHARSXP pool with no allocation. Use `Option<String>`
//! / `Vec<Option<String>>` when callers may pass `NA_character_`; the plain
//! `String` / `Vec<String>` impls map NA to `""`, which is lossy. The optional
//! forms preserve the distinction as `None`.
//!
//! UTF-8/ASCII strings use a zero-copy fast path. Other text encodings are
//! translated to UTF-8; R strings marked as `bytes` are rejected. Outbound
//! counterparts: `String` / `&str` impls in [`crate::into_r`].

use crate::from_r::{
    SexpError, TryFromSexp, charsxp_to_str, charsxp_to_str_unchecked, scalar_charsxp,
    scalar_charsxp_unchecked,
};
use crate::{SEXP, SEXPTYPE, SexpExt};

/// Convert R character vector (STRSXP) to Rust &str.
///
/// Extracts the first element of the character vector and returns it as a UTF-8 string.
/// The Rust type uses a static lifetime for API convenience, but the borrow is
/// only valid while the source CHARSXP remains reachable.
///
/// # NA Handling
///
/// **Warning:** `NA_character_` is converted to empty string `""`. This is lossy!
/// If you need to distinguish between NA and empty strings, use `Option<String>` instead:
///
/// ```ignore
/// let maybe_str: Option<String> = sexp.try_into()?;
/// ```
///
/// # Safety
/// The returned &str is only valid as long as R doesn't garbage collect the CHARSXP.
/// In practice, this is safe within a single .Call invocation.
impl TryFromSexp for &'static str {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let charsxp = scalar_charsxp(sexp)?;

        // Check for NA_STRING or R_BlankString
        if charsxp == SEXP::na_string() {
            return Ok("");
        }
        if charsxp == SEXP::blank_string() {
            return Ok("");
        }

        // Use LENGTH-based conversion (O(1)) instead of CStr::from_ptr (O(n) strlen)
        Ok(unsafe { charsxp_to_str(charsxp) })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let charsxp = unsafe { scalar_charsxp_unchecked(sexp)? };

        // Check for NA_STRING or R_BlankString
        if charsxp == SEXP::na_string() {
            return Ok("");
        }
        if charsxp == SEXP::blank_string() {
            return Ok("");
        }

        // Use LENGTH-based conversion (O(1)) instead of CStr::from_ptr (O(n) strlen)
        Ok(unsafe { charsxp_to_str_unchecked(charsxp) })
    }
}

impl TryFromSexp for Option<&'static str> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }

        let charsxp = scalar_charsxp(sexp)?;
        if charsxp == SEXP::na_string() {
            return Ok(None);
        }
        if charsxp == SEXP::blank_string() {
            return Ok(Some(""));
        }

        Ok(Some(unsafe { charsxp_to_str(charsxp) }))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }

        let charsxp = unsafe { scalar_charsxp_unchecked(sexp)? };
        if charsxp == SEXP::na_string() {
            return Ok(None);
        }
        if charsxp == SEXP::blank_string() {
            return Ok(Some(""));
        }

        Ok(Some(unsafe { charsxp_to_str_unchecked(charsxp) }))
    }
}

/// Convert R character vector (STRSXP) to Rust char.
///
/// Extracts the first character of the first element of the character vector.
/// Returns an error if the string is empty, NA, or has more than one character.
impl TryFromSexp for char {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: &str = TryFromSexp::try_from_sexp(sexp)?;
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Ok(c),
            (None, _) => Err(SexpError::InvalidValue(
                "empty string cannot be converted to char".to_string(),
            )),
            (Some(_), Some(_)) => Err(SexpError::InvalidValue(
                "string has more than one character".to_string(),
            )),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: &str = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Ok(c),
            (None, _) => Err(SexpError::InvalidValue(
                "empty string cannot be converted to char".to_string(),
            )),
            (Some(_), Some(_)) => Err(SexpError::InvalidValue(
                "string has more than one character".to_string(),
            )),
        }
    }
}

/// Convert R character vector (STRSXP) to owned Rust String.
///
/// Extracts the first element and creates an owned copy.
///
/// # NA Handling
///
/// **Warning:** `NA_character_` is converted to empty string `""`. This is lossy!
/// If you need to distinguish between NA and empty strings, use `Option<String>` instead:
///
/// ```ignore
/// let maybe_str: Option<String> = sexp.try_into()?;
/// ```
impl TryFromSexp for String {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let charsxp = scalar_charsxp(sexp)?;

        if charsxp == SEXP::na_string() {
            return Ok(String::new());
        }

        Ok(unsafe { charsxp_to_str(charsxp) }.to_owned())
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let charsxp = unsafe { scalar_charsxp_unchecked(sexp)? };

        if charsxp == SEXP::na_string() {
            return Ok(String::new());
        }

        Ok(unsafe { charsxp_to_str_unchecked(charsxp) }.to_owned())
    }
}

/// NA-aware string conversion: returns `None` for `NA_character_`.
///
/// Use this when you need to distinguish between NA and empty strings:
/// ```ignore
/// let maybe_str: Option<String> = sexp.try_into()?;
/// match maybe_str {
///     Some(s) => println!("Got string: {}", s),
///     None => println!("Got NA"),
/// }
/// ```
impl TryFromSexp for Option<String> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // NULL -> None
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }

        let charsxp = scalar_charsxp(sexp)?;

        // Return None for NA_STRING
        if charsxp == SEXP::na_string() {
            return Ok(None);
        }

        Ok(Some(unsafe { charsxp_to_str(charsxp) }.to_owned()))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // For Option<String>, unchecked is same as checked (NA check is semantic, not safety)
        Self::try_from_sexp(sexp)
    }
}
// endregion
