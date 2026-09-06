//! Package initialization for miniextendr R packages.
//!
//! [`package_init`](crate::init::package_init) consolidates all initialization steps that were previously
//! scattered across `entrypoint.c.in`. The `miniextendr_init!` proc macro
//! generates the `R_init_*` entry point that calls this function.
//!
//! # Usage
//!
//! In your crate's `lib.rs`:
//!
//! ```ignore
//! miniextendr_init!(mypkg);
//! ```
//!
//! This expands to an `extern "C-unwind" fn R_init_mypkg(dll)` that calls
//! [`package_init`](crate::init::package_init) with the appropriate package name.

use crate::Rboolean;
use crate::sys::{DllInfo, R_forceSymbols, R_useDynamicSymbols};
use std::ffi::CStr;

/// Env var the Makevars wrapper-gen recipe sets before `dyn.load`ing the
/// freshly-built shared object to generate `R/*-wrappers.R`.
///
/// Loading the installed `.so`/`.dll` runs `R_init_<pkg>` on every platform, but
/// during wrapper-gen that image is `dyn.unload`ed immediately afterwards. So when
/// this var is present, init takes a minimal path: [`package_init`] skips the
/// panic hook / locale / ALTREP+mx_abi setup, and [`miniextendr_register_routines`]
/// skips ALTREP *class* registration — none of which must plant a pointer into an
/// about-to-be-unloaded image (doing so risks the "malloc unsorted double linked
/// list" heap corruption that macOS hides but Linux R aborts on).
///
/// SINGLE SOURCE OF TRUTH: both read-sites go through [`wrapper_gen_mode`] so the
/// name can't drift between them. Presence-based — any value (even empty) enables
/// it — so it MUST NOT leak into a real package-load environment, or the package
/// loads silently degraded (no panic hook, no ALTREP classes, no mx_abi).
///
/// [`miniextendr_register_routines`]: crate::registry::miniextendr_register_routines
pub(crate) const WRAPPER_GEN_ENV: &str = "MINIEXTENDR_WRAPPER_GEN";

/// `true` when the package was loaded purely for wrapper generation — see
/// [`WRAPPER_GEN_ENV`].
pub(crate) fn wrapper_gen_mode() -> bool {
    std::env::var_os(WRAPPER_GEN_ENV).is_some()
}

/// Initialize a miniextendr R package.
///
/// This performs all initialization steps in the correct order:
///
/// 1. Install panic hook for better error messages
/// 2. Record main thread ID (and optionally spawn worker thread)
/// 3. Assert UTF-8 locale
/// 4. Set ALTREP package name
/// 5. Register mx_abi C-callables for cross-package trait dispatch
/// 6. Register all `#[miniextendr]` routines and ALTREP classes
/// 7. Lock down dynamic symbols
///
/// # Safety
///
/// Must be called from R's main thread during `R_init_*`.
/// `dll` must be a valid pointer provided by R.
/// `pkg_name` must be a valid null-terminated C string that lives for the
/// duration of the R session (typically a string literal).
pub unsafe fn package_init(dll: *mut DllInfo, pkg_name: &CStr) {
    unsafe {
        // When loaded purely for wrapper generation (Makevars dyn.load()s the
        // installed .so/.dll, which runs this init), skip full init. Only routine
        // registration is needed so .Call(miniextendr_write_wrappers) works.
        // See WRAPPER_GEN_ENV. The env var is set by Makevars before dyn.load().
        let wrapper_gen = wrapper_gen_mode();

        // 1. Record main thread ID (and optionally spawn worker thread)
        // Always needed: checked FFI variants (R_useDynamicSymbols, etc.)
        // route through with_r_thread() which requires runtime_init.
        crate::worker::miniextendr_runtime_init();

        if !wrapper_gen {
            // 2. Install panic hook for better error messages
            // Skipped during wrapper-gen: on Windows, set_hook during DLL init
            // can fail with "failed to initiate panic, error 5" because the
            // panic infrastructure isn't fully available during DLL loading.
            crate::backtrace::miniextendr_panic_hook();

            // 3. Assert UTF-8 locale
            crate::encoding::miniextendr_assert_utf8_locale();

            // 3b. Install R console logger (if log feature enabled)
            #[cfg(feature = "log")]
            crate::optionals::log_impl::install_r_logger();

            // 4. Set ALTREP package name and DllInfo
            crate::miniextendr_set_altrep_pkg_name(pkg_name.as_ptr());
            crate::set_altrep_dll_info(dll);

            // 5. Register mx_abi C-callables
            crate::mx_abi::mx_abi_register(pkg_name);
        }

        // 6. Register .Call routines (and ALTREP classes, unless wrapper-gen)
        crate::registry::miniextendr_register_routines(dll);

        // 7. Lock down dynamic symbols
        R_useDynamicSymbols(dll, Rboolean::FALSE);
        R_forceSymbols(dll, Rboolean::TRUE);
    }
}
