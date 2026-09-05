//! M4 — error-path attribution.
//!
//! The R-side bench shows `demo_error("oops")` caught by tryCatch costs
//! ~21 μs total, vs ~7 μs for native `stop("oops")` — i.e. ~14 μs of
//! delta. This bench attributes that delta to:
//!
//! 1. C-side `make_rust_condition_value` — builds the 4-element tagged
//!    VECSXP + class attr + __rust_condition__ marker.
//! 2. C-side `with_r_unwind_protect` panic catch — Rust panic_payload_to_string
//!    + recognition logic.
//! 3. R-side `.miniextendr_raise_condition` — re-raises via
//!    `stop(structure(...))`. Measured separately by `scaffolding-error-bench.R`.
//!
//! Run:
//!   cargo bench --manifest-path=miniextendr-bench/Cargo.toml \
//!     --bench error_path_attribution

use miniextendr_api::SEXP;
use miniextendr_api::error_value::{kind, make_rust_condition_value};
use miniextendr_api::unwind_protect::with_r_unwind_protect;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// ---------------------------------------------------------------------------
// L1: make_rust_condition_value alone — pure C-side build cost.
// ---------------------------------------------------------------------------

#[divan::bench]
fn l1_make_condition_simple() -> SEXP {
    // SAFETY: the bench harness runs single-threaded on the R main thread.
    let s = unsafe { make_rust_condition_value("oops", kind::ERROR, None, None) };
    divan::black_box(s)
}

#[divan::bench]
fn l1_make_condition_with_class() -> SEXP {
    // SAFETY: the bench harness runs single-threaded on the R main thread.
    let s = unsafe { make_rust_condition_value("oops", kind::ERROR, Some("my_class"), None) };
    divan::black_box(s)
}

#[divan::bench]
fn l1_make_condition_warning() -> SEXP {
    // SAFETY: the bench harness runs single-threaded on the R main thread.
    let s = unsafe { make_rust_condition_value("hmm", kind::WARNING, None, None) };
    divan::black_box(s)
}

// ---------------------------------------------------------------------------
// L2: with_r_unwind_protect catching a panic!() — exercises the
// panic_payload_to_string + telemetry path.
// ---------------------------------------------------------------------------

#[divan::bench]
fn l2_panic_to_condition(bencher: divan::Bencher) {
    // Suppress the panic-hook noise during bench.
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    bencher.bench_local(|| {
        let out = with_r_unwind_protect(
            || -> SEXP {
                panic!("bench panic");
            },
            None,
        );
        divan::black_box(out);
    });
    std::panic::set_hook(prev);
}

// ---------------------------------------------------------------------------
// L3: with_r_unwind_protect catching an RCondition::Error — the user-visible
// path via the `error!()` macro.
// ---------------------------------------------------------------------------

#[divan::bench]
fn l3_rcondition_error(bencher: divan::Bencher) {
    use miniextendr_api::condition::RCondition;
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    bencher.bench_local(|| {
        let out = with_r_unwind_protect(
            || -> SEXP {
                std::panic::panic_any(RCondition::Error {
                    message: "oops".to_string(),
                    class: vec![],
                    data: None,
                });
            },
            None,
        );
        divan::black_box(out);
    });
    std::panic::set_hook(prev);
}

// ---------------------------------------------------------------------------
// L3b: same but with a custom class — closer to the demo_error_custom_class
// path. Tests that the class allocation overhead is measurable.
// ---------------------------------------------------------------------------

#[divan::bench]
fn l3b_rcondition_error_classed(bencher: divan::Bencher) {
    use miniextendr_api::condition::RCondition;
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    bencher.bench_local(|| {
        let out = with_r_unwind_protect(
            || -> SEXP {
                std::panic::panic_any(RCondition::Error {
                    message: "oops".to_string(),
                    class: vec!["my_class".to_string()],
                    data: None,
                });
            },
            None,
        );
        divan::black_box(out);
    });
    std::panic::set_hook(prev);
}
