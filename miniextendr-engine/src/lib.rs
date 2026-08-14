//! miniextendr-engine: standalone R embedding for Rust binaries and tests.
//!
//! This crate centralizes `libR` linking (via `build.rs`), R initialization, and
//! a minimal runtime handle. It is intended for Rust-only executables and
//! integration tests that embed R.
//!
//! **Not for R packages:** this crate uses non-API R internals
//! (`Rembedded.h`, `Rinterface.h`). For R packages, depend on `miniextendr-api`
//! and keep `nonapi` disabled.
//!
//! ## When to use
//! - Rust binaries that embed R.
//! - Integration tests or benchmarks that need full control over R startup.
//!
//! ## Quick start
//!
//! ```no_run
//! // SAFETY: Must be called once, from the main thread.
//! let _engine = unsafe {
//!     miniextendr_engine::REngine::build()
//!         .with_args(&["R", "--quiet", "--vanilla"])
//!         .init()
//!         .expect("Failed to initialize R")
//! };
//!
//! // ... use R APIs from the main thread ...
//! // R remains initialized when the handle leaves scope.
//! ```
//!
//! ## Initialization details
//! - Ensures `R_HOME` (via `R RHOME`) if missing.
//! - Calls `Rf_initialize_R` directly to avoid double `setup_Rmainloop()`.
//! - Calls `setup_Rmainloop()` exactly once after initialization.
//!
//! ## Runtime sentinel
//!
//! ```no_run
//! if miniextendr_engine::r_initialized_sentinel() {
//!     // R has been initialized in this process.
//! }
//! ```
//!
//! ## Safety
//!
//! - Must only be initialized once per process.
//! - Must be called from the main thread.
//! - No shutdown: `Rf_endEmbeddedR` is intentionally not called because the
//!   cleanup path is not reentrant-safe. The OS reclaims resources on exit.

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::process::Command;

// Note: This entire crate uses non-API R functions (Rembedded.h, Rinterface.h)
// for embedding R. It is not intended for use in R packages.
unsafe extern "C" {
    // R initialization (from Rembedded.h - non-API)
    fn Rf_initialize_R(argc: c_int, argv: *mut *mut c_char) -> c_int;
    #[allow(dead_code)]
    fn Rf_endEmbeddedR(fatal: c_int);

    // Setup functions
    fn setup_Rmainloop();

    // Global state from Rinterface.h (non-API)
    // Use UnsafeCell for interior mutability without static mut
    static R_Interactive: std::cell::UnsafeCell<c_int>;
    static R_SignalHandlers: std::cell::UnsafeCell<c_int>;
    static R_CStackStart: usize;
    static R_CStackDir: c_int;
    static mut R_CStackLimit: usize;
}

/// Write to R's global `R_Interactive` flag.
///
/// # Safety
/// Must be called from the main thread during R initialization.
#[inline]
unsafe fn set_r_interactive(value: c_int) {
    unsafe {
        *R_Interactive.get() = value;
    }
}

/// Write to R's global `R_SignalHandlers` flag.
///
/// # Safety
/// Must be called from the main thread during R initialization.
#[inline]
unsafe fn set_r_signal_handlers(value: c_int) {
    unsafe {
        *R_SignalHandlers.get() = value;
    }
}

/// Check whether `Rf_initialize_R` has run by inspecting stack sentinels.
///
/// `R_CStackStart`/`R_CStackDir` are set during R initialization on the main
/// thread. A zero or `usize::MAX` value indicates "not initialized".
#[inline]
pub fn r_initialized_sentinel() -> bool {
    unsafe {
        let start = R_CStackStart;
        let dir = R_CStackDir;
        dir != 0 && start != 0 && start != usize::MAX
    }
}

/// Builder for configuring and initializing the R runtime.
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), miniextendr_engine::REngineError> {
/// let _engine = unsafe {
///     miniextendr_engine::REngine::build()
///         .with_args(&["R", "--quiet", "--no-save"])
///         .interactive(false)
///         .signal_handlers(false)
///         .init()?
/// };
/// # Ok(())
/// # }
/// ```
pub struct REngineBuilder {
    args: Vec<String>,
    interactive: bool,
    signal_handlers: bool,
}

impl Default for REngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl REngineBuilder {
    /// Create a new R engine builder with default settings.
    pub fn new() -> Self {
        Self {
            // Default to a non-interactive-safe setup: R requires an explicit
            // save/no-save choice when not running interactively.
            args: vec![
                "R".to_string(),
                "--quiet".to_string(),
                "--vanilla".to_string(),
            ],
            interactive: false,
            signal_handlers: false,
        }
    }

    /// Set the command-line arguments for R initialization.
    ///
    /// Default is `["R", "--quiet", "--vanilla"]`.
    pub fn with_args(mut self, args: &[&str]) -> Self {
        self.args = args.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set whether R should run in interactive mode.
    ///
    /// Default is `false`.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Set whether R should install signal handlers.
    ///
    /// Default is `false`. Set to `true` if you want R to handle Ctrl+C etc.
    pub fn signal_handlers(mut self, enable: bool) -> Self {
        self.signal_handlers = enable;
        self
    }

    /// Initialize the R runtime with the configured settings.
    ///
    /// # Safety
    ///
    /// - Must only be called once per process
    /// - Must be called from the main thread
    /// - R cannot be safely shutdown and reinitialized
    ///
    /// # Errors
    ///
    /// Returns an error if R initialization fails.
    pub unsafe fn init(self) -> Result<REngine, REngineError> {
        // Guard against re-initialization
        if r_initialized_sentinel() {
            return Err(REngineError::AlreadyInitialized);
        }

        ensure_r_home_env()?;

        // Convert args to C strings
        let c_args: Vec<CString> = self
            .args
            .iter()
            .map(|s| CString::new(s.as_str()).unwrap())
            .collect();

        let mut c_ptrs: Vec<*mut c_char> = c_args.iter().map(|s| s.as_ptr().cast_mut()).collect();

        let argc = c_ptrs.len() as c_int;
        let argv = c_ptrs.as_mut_ptr();

        // Initialize R.
        //
        // Note: `Rf_initEmbeddedR()` already calls `setup_Rmainloop()`.
        // We want tighter control (and to avoid double-calling the setup),
        // so we call `Rf_initialize_R()` directly and then `setup_Rmainloop()`.
        let result = unsafe { Rf_initialize_R(argc, argv) };
        if result != 0 {
            return Err(REngineError::InitializationFailed);
        }

        unsafe {
            // Set global flags *after* initialization, mirroring R's own
            // `Rf_initEmbeddedR()` order (but respecting our builder flags).
            set_r_interactive(if self.interactive { 1 } else { 0 });
            set_r_signal_handlers(if self.signal_handlers { 1 } else { 0 });

            // Disable R's C-stack overflow check before `setup_Rmainloop()`
            // evaluates any R code. `Rf_initialize_R` calibrates
            // `R_CStackStart` for the *process* main thread (glibc uses
            // `__libc_stack_end`), so when R is initialized on any other
            // thread — as the test harness's dedicated `r-test-main` thread
            // does — the computed usage is garbage and R dies with
            // "C stack usage <huge> is too close to the limit" during
            // startup evaluation. macOS calibrates per-thread
            // (`pthread_get_stackaddr_np`), which is why this only bites on
            // Linux. Disabling the check (limit = -1, per Writing R
            // Extensions §8) is standard embedded-R-on-a-thread practice;
            // the OS guard page still catches real overflows.
            R_CStackLimit = usize::MAX;

            setup_Rmainloop();

            // Note: We do NOT register an atexit handler for Rf_endEmbeddedR.
            // The R runtime cleanup operations (KillAllDevices, RunExitFinalizers, etc.)
            // are complex and can crash if other cleanup is happening concurrently.
            // For short-lived programs (tests, benchmarks), letting the OS reclaim
            // resources on process exit is safer and sufficient.
        }

        Ok(REngine { _private: () })
    }
}

/// Handle to an initialized R runtime.
///
/// This is a marker type indicating R has been initialized for this process.
/// R cleanup (via `Rf_endEmbeddedR`) is intentionally NOT called because it
/// performs non-reentrant operations that can crash if called during Drop
/// or concurrent with other cleanup. The OS reclaims all resources on process exit.
///
/// The handle cannot be constructed directly; only a successful
/// [`REngineBuilder::init`] can create it.
///
/// ```compile_fail
/// let _forged = miniextendr_engine::REngine;
/// ```
pub struct REngine {
    _private: (),
}

impl REngine {
    /// Create a new builder for configuring R initialization.
    pub fn build() -> REngineBuilder {
        REngineBuilder::new()
    }
}

// Note: We intentionally DO NOT provide shutdown or Drop implementations.
//
// Rf_endEmbeddedR performs non-reentrant cleanup operations.
// Here's what it does (from R 4.5.2 source):
//
// Unix/Linux version (src/unix/Rembedded.c):
// ```c
// void Rf_endEmbeddedR(int fatal)
// {
//     R_RunExitFinalizers();    // Runs .Last and exit handlers (NOT reentrant!)
//     CleanEd();                // Editor cleanup
//     if(!fatal) KillAllDevices();  // Graphics devices (NOT reentrant!)
//     R_CleanTempDir();         // File system cleanup
//     if(!fatal && R_CollectWarnings)
//         PrintWarnings();      // Console I/O
//     fpu_setup(FALSE);         // FPU state
// }
// ```
//
// Windows version (src/gnuwin32/embeddedR.c):
// ```c
// void Rf_endEmbeddedR(int fatal)
// {
//     R_RunExitFinalizers();
//     CleanEd();
//     R_CleanTempDir();
//     if(!fatal){
//         Rf_KillAllDevices();
//         AllDevicesKilled = TRUE;
//     }
//     if(!fatal && R_CollectWarnings)
//         PrintWarnings();
//     app_cleanup();           // Application-specific cleanup
// }
// ```
//
// These operations are NOT reentrant and must run exactly once at process exit.
// Calling during Drop (e.g., test cleanup) causes crashes.
//
// **Solution:** We intentionally do NOT call Rf_endEmbeddedR. For short-lived
// programs (tests, benchmarks), the OS reclaims all resources on process exit.
// This avoids crashes from double-cleanup or reentrant calls.

/// Errors that can occur during R engine initialization.
#[derive(Debug)]
pub enum REngineError {
    /// Could not determine / set `R_HOME` for embedding.
    RHomeNotFound {
        /// Optional stderr from `R RHOME` command for diagnostics.
        stderr: Option<String>,
    },
    /// R initialization failed.
    InitializationFailed,
    /// R is already initialized. Re-initialization is not supported.
    AlreadyInitialized,
}

impl std::fmt::Display for REngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            REngineError::RHomeNotFound { stderr } => {
                write!(f, "R_HOME is not set and `R RHOME` could not be resolved")?;
                if let Some(stderr) = stderr
                    && !stderr.is_empty()
                {
                    write!(f, "\nstderr: {}", stderr)?;
                }
                Ok(())
            }
            REngineError::InitializationFailed => write!(f, "R initialization failed"),
            REngineError::AlreadyInitialized => {
                write!(
                    f,
                    "R is already initialized. Multiple calls to REngineBuilder::init() are not supported."
                )
            }
        }
    }
}

impl std::error::Error for REngineError {}

fn ensure_r_home_env() -> Result<(), REngineError> {
    // If R_HOME is already set, use it
    if std::env::var_os("R_HOME").is_some() {
        return Ok(());
    }

    // Auto-detect via `R RHOME`
    let output = Command::new("R")
        .args(["RHOME"])
        .output()
        .map_err(|_| REngineError::RHomeNotFound { stderr: None })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(REngineError::RHomeNotFound {
            stderr: Some(stderr),
        });
    }

    let r_home = String::from_utf8(output.stdout)
        .map_err(|_| REngineError::RHomeNotFound { stderr: None })?;
    let r_home = r_home.trim();
    if r_home.is_empty() {
        return Err(REngineError::RHomeNotFound { stderr: None });
    }

    // SAFETY: We call this during single-threaded startup (before initializing
    // R and before spawning any worker threads).
    unsafe {
        std::env::set_var("R_HOME", r_home);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
