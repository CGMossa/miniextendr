//! Cached R class attribute SEXPs.
//!
//! Frequently-used class vectors (like `c("POSIXct", "POSIXt")`) and attribute
//! values are allocated once and preserved permanently. This avoids repeated
//! `Rf_mkCharLenCE` hash lookups on hot paths.
//!
//! Two declarative macros handle the boilerplate:
//!
//! ```ignore
//! // Cache a symbol (Rf_install result):
//! cached_symbol!(pub(crate) fn tzone_symbol() = c"tzone");
//!
//! // Cache a STRSXP vector (class, names, etc.):
//! cached_strsxp!(pub(crate) fn posixct_class_sexp() = [c"POSIXct", c"POSIXt"]);
//! cached_strsxp!(pub(crate) fn date_class_sexp() = [c"Date"]);
//! ```
//!
//! Both expand to a function with a `static OnceLock<SEXP>` inside.
//! First call initializes; subsequent calls are a single atomic load.
//!
//! CHARSXPs are obtained via `Rf_install` + `PRINTNAME` — symbols are never
//! collected, so the CHARSXP is permanently valid. STRSXP vectors are kept
//! alive via `R_PreserveObject`.

// region: Macros

/// Cache an `Rf_install` symbol result.
///
/// Expands to a function that returns the cached SEXP. First call does the
/// `Rf_install`; subsequent calls are a single atomic load.
///
/// ```ignore
/// cached_symbol!(pub(crate) fn tzone_symbol() = c"tzone");
///
/// // With feature gate:
/// cached_symbol!(#[cfg(feature = "time")] pub(crate) fn tzone_symbol() = c"tzone");
/// ```
macro_rules! cached_symbol {
    ($(#[$meta:meta])* $vis:vis fn $name:ident() = $cstr:expr) => {
        $(#[$meta])*
        $vis fn $name() -> $crate::SEXP {
            static CACHE: ::std::sync::OnceLock<$crate::SEXP> = ::std::sync::OnceLock::new();
            *CACHE.get_or_init(|| unsafe { $crate::sys::Rf_install($cstr.as_ptr()) })
        }
    };
}
#[allow(unused_imports)] // exported for use by other modules
pub(crate) use cached_symbol;

/// Cache a STRSXP vector built from permanent CHARSXPs.
///
/// Each element is a `&CStr` literal routed through `Rf_install` + `PRINTNAME`
/// for a never-GC'd CHARSXP. The STRSXP itself is kept alive via
/// `R_PreserveObject`.
///
/// ```ignore
/// // Single-element class:
/// cached_strsxp!(pub(crate) fn date_class_sexp() = [c"Date"]);
///
/// // Multi-element class:
/// cached_strsxp!(pub(crate) fn posixct_class_sexp() = [c"POSIXct", c"POSIXt"]);
///
/// // With feature gate:
/// cached_strsxp!(
///     #[cfg(any(feature = "time", feature = "arrow"))]
///     pub(crate) fn posixct_class_sexp() = [c"POSIXct", c"POSIXt"]
/// );
/// ```
macro_rules! cached_strsxp {
    ($(#[$meta:meta])* $vis:vis fn $name:ident() = [$($cstr:expr),+ $(,)?]) => {
        $(#[$meta])*
        $vis fn $name() -> $crate::SEXP {
            static CACHE: ::std::sync::OnceLock<$crate::SEXP> = ::std::sync::OnceLock::new();
            *CACHE.get_or_init(|| unsafe {
                use $crate::SexpExt as _;
                let strings: &[&::std::ffi::CStr] = &[$($cstr),+];
                let sexp = $crate::sys::Rf_allocVector(
                    $crate::SEXPTYPE::STRSXP,
                    strings.len() as ::std::primitive::isize,
                );
                $crate::sys::R_PreserveObject(sexp);
                for (i, s) in strings.iter().enumerate() {
                    sexp.set_string_elt(
                        i as ::std::primitive::isize,
                        $crate::cached_class::permanent_charsxp(s),
                    );
                }
                sexp
            })
        }
    };
}
#[allow(unused_imports)] // exported for use by other modules
pub(crate) use cached_strsxp;

// endregion

// region: Permanent CHARSXP helper

/// Get a permanent CHARSXP for a string by going through `Rf_install` + `PRINTNAME`.
///
/// Symbols are never GC'd, so the CHARSXP from `PRINTNAME` is valid forever.
/// This avoids the `Rf_mkCharLenCE` hash lookup on repeated calls.
///
/// Used by [`cached_strsxp!`] — `pub(crate)` so the macro can reference it
/// from any module.
#[doc(hidden)]
#[inline]
pub(crate) fn permanent_charsxp(name: &std::ffi::CStr) -> crate::SEXP {
    use crate::SexpExt;
    unsafe { crate::sys::Rf_install(name.as_ptr()) }.printname()
}

// endregion

// region: Class vectors

cached_strsxp!(
    /// Cached `c("POSIXct", "POSIXt")` class STRSXP.
    #[cfg(any(feature = "time", feature = "jiff", feature = "arrow"))]
    pub(crate) fn posixct_class_sexp() = [c"POSIXct", c"POSIXt"]
);

cached_strsxp!(
    /// Cached `"Date"` class STRSXP.
    #[cfg(any(feature = "time", feature = "jiff", feature = "arrow"))]
    pub(crate) fn date_class_sexp() = [c"Date"]
);

cached_strsxp!(
    /// Cached `"data.frame"` class STRSXP.
    pub(crate) fn data_frame_class_sexp() = [c"data.frame"]
);

cached_strsxp!(
    /// Cached `"rust_condition_value"` class STRSXP — applied to every tagged
    /// SEXP produced by `make_rust_condition_value` to identify panic-transport
    /// values to the generated R wrappers.
    pub(crate) fn rust_condition_class_sexp() = [c"rust_condition_value"]
);

cached_strsxp!(
    /// Cached `c("error", "kind", "class", "call", "data")` names STRSXP for
    /// condition values.
    ///
    /// Used by `make_rust_condition_value` which writes a 5-element list: the
    /// error message, kind, optional user-supplied custom class, optional R
    /// call, and an optional named-list `data` payload (`NULL` when absent).
    pub(crate) fn condition_names_sexp() = [c"error", c"kind", c"class", c"call", c"data"]
);

// endregion

// region: Scalar strings

cached_strsxp!(
    /// Cached `"UTC"` scalar string SEXP for the `tzone` attribute.
    #[cfg(any(feature = "time", feature = "jiff"))]
    fn utc_tzone_sexp() = [c"UTC"]
);

// endregion

// region: Symbols

cached_symbol!(
    /// Cached `tzone` symbol.
    #[cfg(any(feature = "time", feature = "jiff", feature = "arrow"))]
    pub(crate) fn tzone_symbol() = c"tzone"
);

cached_symbol!(
    /// Cached `__rust_condition__` symbol — the marker attribute on tagged
    /// panic-transport SEXPs. Wrapper code asserts this is `TRUE` in addition
    /// to the class check before re-raising via `.miniextendr_raise_condition`.
    pub(crate) fn rust_condition_attr_symbol() = c"__rust_condition__"
);

cached_symbol!(
    /// Cached `mx_raw_type` symbol (for raw conversion type tags).
    #[cfg(feature = "raw_conversions")]
    pub(crate) fn mx_raw_type_symbol() = c"mx_raw_type"
);

cached_symbol!(
    /// Cached `ptype` symbol (vctrs list_of attribute).
    #[cfg(feature = "vctrs")]
    pub(crate) fn ptype_symbol() = c"ptype"
);

cached_symbol!(
    /// Cached `size` symbol (vctrs list_of attribute).
    #[cfg(feature = "vctrs")]
    pub(crate) fn size_symbol() = c"size"
);

// endregion

// region: Composite helpers

/// Set class = `c("POSIXct", "POSIXt")` and tzone = `"UTC"` on an SEXP.
///
/// Uses cached class vector + tzone string — zero allocations after first call.
///
/// # Safety
///
/// `sexp` must be a valid REALSXP. Must be called on R's main thread.
#[cfg(any(feature = "time", feature = "jiff"))]
pub fn set_posixct_utc(sexp: crate::SEXP) {
    use crate::SexpExt as _;
    sexp.set_class(posixct_class_sexp());
    sexp.set_attr(tzone_symbol(), utc_tzone_sexp());
}

/// Set class = `c("POSIXct", "POSIXt")` and tzone = `iana` on an SEXP.
///
/// Used by the jiff integration to round-trip `Zoned` timezone identity.
/// Callers pass `"UTC"` for zones without an IANA name (e.g., fixed-offset zones).
///
/// # Safety
///
/// `sexp` must be a valid REALSXP. Must be called on R's main thread.
#[cfg(feature = "jiff")]
pub fn set_posixct_tz(sexp: crate::SEXP, iana: &str) {
    use crate::SexpExt as _;
    sexp.set_class(posixct_class_sexp());
    // Build a one-element STRSXP for the tzone attribute.
    unsafe {
        let tzone_charsxp = crate::SEXP::charsxp(iana);
        // R's API requires a returned object to be protected immediately: the
        // STRSXP allocation below may collect a newly-created CHARSXP before
        // it is installed in the container.
        let _tzone_charsxp_guard = crate::OwnedProtect::new(tzone_charsxp);
        let tzone_sexp = crate::sys::Rf_allocVector(crate::SEXPTYPE::STRSXP, 1);
        let _tzone_guard = crate::OwnedProtect::new(tzone_sexp);
        tzone_sexp.set_string_elt(0, tzone_charsxp);
        sexp.set_attr(tzone_symbol(), tzone_sexp);
    }
}

// endregion
