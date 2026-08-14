#![allow(dead_code)]

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{OnceLock, mpsc};

use miniextendr_api::thread::RThreadBuilder;

type Job = Box<dyn FnOnce() + Send + 'static>;

static R_THREAD: OnceLock<mpsc::Sender<Job>> = OnceLock::new();

fn initialize_r() {
    unsafe {
        let _engine = miniextendr_engine::REngine::build()
            .with_args(&["R", "--quiet", "--vanilla"])
            .init()
            .expect("Failed to initialize R");
        // Initialize in same order as package_init() (called by miniextendr_init!)
        miniextendr_api::backtrace::miniextendr_panic_hook();
        miniextendr_api::worker::miniextendr_runtime_init();
        disable_r_stack_checking();
        assert!(
            miniextendr_engine::r_initialized_sentinel(),
            "Rf_initialize_R did not set C stack sentinels"
        );
    }
}

/// Disable R's stack checking for test threads.
///
/// # Why This Exists
///
/// R's stack overflow detection uses `R_CStackLimit` which is calibrated for
/// the main thread. When running R on a dedicated test thread (as we do in
/// `cargo test`), these checks produce false positives. Setting `R_CStackLimit`
/// to `usize::MAX` disables R's checks while the OS stack limit still applies.
///
/// # CRAN Compliance
///
/// **This code is never part of the R package.** It exists only in the `tests/`
/// directory and is only compiled for `cargo test`. The R package (`rpkg`):
///
/// 1. Only vendors `miniextendr-api/src/` (not `tests/`)
/// 2. Does not include `miniextendr-engine` (the R embedding crate)
/// 3. Uses the `nonapi` feature gate for any `R_CStackLimit` access in library code
///
/// The direct `extern "C"` access to `R_CStackLimit` here (when `nonapi` feature
/// is disabled) is acceptable because this code path only runs in `cargo test`,
/// never in the CRAN package.
fn disable_r_stack_checking() {
    #[cfg(feature = "nonapi")]
    {
        miniextendr_api::thread::disable_stack_checking_permanently();
    }

    #[cfg(not(feature = "nonapi"))]
    unsafe {
        unsafe extern "C" {
            static mut R_CStackLimit: usize;
        }
        R_CStackLimit = usize::MAX;
    }
}

pub fn with_r_thread<T, F>(f: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let sender = R_THREAD.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Job>();
        RThreadBuilder::new()
            .name("r-test-main".to_string())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                initialize_r();
                for job in rx {
                    job();
                }
            })
            .expect("Failed to spawn R test thread");
        tx
    });

    let (result_tx, result_rx) = mpsc::sync_channel(0);
    let job: Job = Box::new(move || {
        let result = catch_unwind(AssertUnwindSafe(f));
        let _ = result_tx.send(result);
    });

    sender.send(job).expect("R test thread stopped");
    match result_rx
        .recv()
        .expect("R test thread dropped the response")
    {
        Ok(value) => value,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}
