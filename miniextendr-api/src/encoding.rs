//! Encoding / locale probing utilities.
//!
//! This module exists mainly for debugging + experiments around R's string
//! encodings. R's runtime has both:
//! - per-CHARSXP encoding tags (UTF-8 / Latin-1 / bytes / native)
//! - global/locale-level settings (native encoding, UTF-8 locale flags)
//!
//! # Availability
//!
//! The global signals are **non-API** (from `Defn.h`) and require the `nonapi`
//! feature. Only the locale flags R's shared library exports are read
//! (`utf8locale`, `mbcslocale`, `known_to_be_latin1`); the hidden ones
//! (`known_to_be_utf8`, `latin1locale`, `R_nativeEncoding`) must not even be
//! *referenced* — an eager data relocation against a hidden symbol aborts
//! `dyn.load` of the whole package (see `sys::nonapi_encoding`).
//!
//! `miniextendr_encoding_init()` is not called by `package_init`; it exists
//! for debugging and for standalone Rust applications embedding R.

use std::sync::OnceLock;

/// Cached snapshot of R's encoding / locale state at init time.
#[derive(Debug, Clone)]
pub struct REncodingInfo {
    #[cfg(feature = "nonapi")]
    /// Whether R thinks the current locale is UTF-8 (non-API `utf8locale`).
    pub utf8_locale: Option<bool>,
    #[cfg(feature = "nonapi")]
    /// Whether the current locale is multi-byte (non-API `mbcslocale`).
    pub mbcs_locale: Option<bool>,
    #[cfg(feature = "nonapi")]
    /// Whether R treats unknown-encoding strings as Latin-1 (non-API
    /// `known_to_be_latin1`).
    pub known_to_be_latin1: Option<bool>,
}

static ENCODING_INFO: OnceLock<REncodingInfo> = OnceLock::new();

/// Return the cached encoding info (if `miniextendr_encoding_init()` has run).
#[inline]
pub fn encoding_info() -> Option<&'static REncodingInfo> {
    ENCODING_INFO.get()
}

/// Assert that R's locale is UTF-8.
///
/// Called once from `R_init_*` (package init). Errors if the R session
/// does not use UTF-8, so native/unmarked CHARSXPs have an unambiguous UTF-8
/// interpretation. Explicitly tagged Latin-1 strings are translated by the
/// conversion layer, while strings marked as `bytes` are rejected as non-text.
///
/// Uses `l10n_info()[["UTF-8"]]` which is public R API.
///
/// ABI: must be `extern "C-unwind"`, not `extern "C"` — the failure branch
/// raises via `Rf_error`, and under webR (Emscripten builds R with
/// `-sSUPPORT_LONGJMP=wasm`) that longjmp is a real wasm exception that
/// unwinds through this frame. A non-unwind `extern "C"` ABI makes rustc
/// insert an abort guard at the boundary, which turns the load-gate error
/// into an `unreachable` trap that kills the whole wasm runtime (observed
/// in the tier-3 testthat pass via `assert_utf8_locale_now()`). On native
/// targets `longjmp` never touches landing pads, which is why this only
/// bit on wasm32.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_assert_utf8_locale() {
    debug_assert!(
        crate::worker::is_r_main_thread(),
        "must be called from R main thread"
    );
    use crate::SexpExt;
    use crate::sys::{R_BaseEnv, Rf_eval, Rf_install};

    unsafe {
        // Read `l10n_info()[["UTF-8"]]` inside a scope so the temporaries
        // (call, info) are unprotected *before* the error branch below —
        // matching the original ordering (a longjmp out of `Rf_error` would
        // skip the scope's Drop).
        let is_utf8 = {
            let scope = crate::ProtectScope::new();
            // Call l10n_info()
            let call = scope.protect_raw(crate::sys::Rf_lang1(Rf_install(c"l10n_info".as_ptr())));
            let info = scope.protect_raw(Rf_eval(call, R_BaseEnv));

            // Find the "UTF-8" element by name
            let names = info.get_names();
            let n = info.xlength();
            let mut is_utf8 = false;
            for i in 0..n {
                let name_charsxp = names.string_elt(i);
                if name_charsxp.r_char_str() == Some("UTF-8") {
                    let elt = info.vector_elt(i);
                    is_utf8 = elt.logical_elt(0) != 0;
                    break;
                }
            }
            is_utf8
        };

        if !is_utf8 {
            crate::sys::Rf_error_unchecked(
                c"%s".as_ptr(),
                c"miniextendr requires a UTF-8 locale (R >= 4.2.0 uses UTF-8 by default)".as_ptr(),
            );
        }
    }
}

/// Initialize / snapshot R's encoding state.
///
/// Intended to be called once from `R_init_*` (package init).
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_encoding_init() {
    let _ = ENCODING_INFO.get_or_init(|| {
        #[cfg(feature = "nonapi")]
        unsafe {
            use crate::Rboolean;
            use crate::sys::nonapi_encoding;

            let utf8_locale = Some(nonapi_encoding::utf8locale != Rboolean::FALSE);
            let mbcs_locale = Some(nonapi_encoding::mbcslocale != Rboolean::FALSE);
            let known_to_be_latin1 = Some(nonapi_encoding::known_to_be_latin1 != Rboolean::FALSE);

            let info = REncodingInfo {
                utf8_locale,
                mbcs_locale,
                known_to_be_latin1,
            };

            if std::env::var_os("MINIEXTENDR_ENCODING_DEBUG").is_some() {
                let msg = format!("[miniextendr] encoding init: {info:?}\n");
                if let Ok(c_msg) = std::ffi::CString::new(msg) {
                    crate::sys::REprintf_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
                }
            }

            info
        }

        #[cfg(not(feature = "nonapi"))]
        {
            REncodingInfo {}
        }
    });
}
