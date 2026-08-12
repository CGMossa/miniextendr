//! Test fixtures for the encoding / locale-assertion path used by
//! `package_init` (see `miniextendr-api/src/init.rs`).
//!
//! `package_init` calls `miniextendr_assert_utf8_locale()` exactly once during
//! `R_init_*`. That call is the load-time gate that rejects sessions whose
//! locale isn't UTF-8 (the `from_r` decoders rely on native/unmarked CHARSXPs
//! having a UTF-8 interpretation; explicitly tagged strings are handled per
//! string).
//!
//! Because the assertion only runs at package load, the only way to exercise
//! it in process is to re-invoke it after `Sys.setlocale()` has flipped the
//! native encoding. The fixture below lets R-side tests do that.

use miniextendr_api::encoding;
use miniextendr_api::prelude::*;

/// Whether `miniextendr_encoding_init()` populated the cached snapshot.
///
/// On standard R-package builds this returns `FALSE` (encoding init is
/// disabled because the underlying symbols aren't exported from libR).
/// Embedded R builds with the `nonapi` feature can flip this to `TRUE`.
#[miniextendr]
pub fn encoding_info_available() -> bool {
    encoding::encoding_info().is_some()
}

/// Re-run the same UTF-8 assertion that fires inside `package_init`.
///
/// Returns `TRUE` on success. If the current locale isn't UTF-8 the
/// underlying `miniextendr_assert_utf8_locale()` calls `Rf_error`, which
/// surfaces in R as a regular error caught by `tryCatch`/`expect_error`.
///
/// Used by `tests/testthat/test-encoding.R` to verify both the success and
/// failure paths after `Sys.setlocale()` flips `LC_CTYPE`.
#[miniextendr]
pub fn assert_utf8_locale_now() -> bool {
    encoding::miniextendr_assert_utf8_locale();
    true
}
