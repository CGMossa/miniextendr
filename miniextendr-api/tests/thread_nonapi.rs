//! Feature-gated tests for nonapi thread utilities.
//!
//! These tests require actual R initialization via miniextendr-engine.

#![cfg(feature = "nonapi")]

use std::sync::OnceLock;

use miniextendr_api::thread::{
    RThreadBuilder, StackCheckGuard, get_stack_config, is_stack_checking_disabled,
    with_stack_checking_disabled,
};

/// Global R initialization state.
/// R can only be initialized once per process.
static R_INITIALIZED: OnceLock<Result<(), String>> = OnceLock::new();

/// Initialize R once for all tests in this file.
fn ensure_r_initialized() -> Result<(), String> {
    R_INITIALIZED
        .get_or_init(|| {
            // SAFETY: We're initializing R once at the start of the test process.
            // This must be called from the main thread before any R operations.
            unsafe {
                let result = miniextendr_engine::REngine::build()
                    .with_args(&["R", "--quiet", "--vanilla"])
                    .init();

                match result {
                    Ok(_engine) => {
                        // Initialize miniextendr-api components
                        miniextendr_api::backtrace::miniextendr_panic_hook();
                        miniextendr_api::worker::miniextendr_runtime_init();

                        // Verify R is properly initialized
                        if !miniextendr_engine::r_initialized_sentinel() {
                            return Err("Rf_initialize_R did not set C stack sentinels".to_string());
                        }

                        Ok(())
                    }
                    Err(e) => Err(format!("Failed to initialize R: {}", e)),
                }
            }
        })
        .clone()
}

#[test]
fn nonapi_thread_suite() {
    // Initialize R (or get cached result)
    if let Err(e) = ensure_r_initialized() {
        // Skip test if R initialization fails (e.g., R not available)
        eprintln!("Skipping test: {}", e);
        return;
    }

    // Test stack configuration
    let (start, limit, dir) = get_stack_config();
    assert!(dir == -1 || dir == 1, "stack direction should be -1 or 1");
    assert_ne!(start, 0, "stack start should not be zero");
    assert_ne!(limit, usize::MAX, "stack limit should not be usize::MAX");

    // Test stack checking guard
    assert!(
        !is_stack_checking_disabled(),
        "stack checking should be enabled initially"
    );
    {
        let _guard = StackCheckGuard::disable();
        assert!(
            is_stack_checking_disabled(),
            "stack checking should be disabled with guard"
        );
        assert!(
            StackCheckGuard::active_count() >= 1,
            "at least one guard should be active"
        );
    }
    assert!(
        !is_stack_checking_disabled(),
        "stack checking should be re-enabled after guard drops"
    );

    // Test with_stack_checking_disabled helper
    let value = with_stack_checking_disabled(|| 123);
    assert_eq!(
        value, 123,
        "with_stack_checking_disabled should return closure result"
    );
}

#[test]
fn nonapi_rthread_builder() {
    // This test just verifies RThreadBuilder can be created
    // (doesn't require R initialization)
    let builder = RThreadBuilder::new()
        .name("test-thread".to_string())
        .stack_size(8 * 1024 * 1024);

    // We don't actually spawn because that would require complex cleanup
    // Just verify the builder API works
    let _builder = builder;
}
