# miniextendr-engine

Standalone R embedding engine for Rust binaries and tests.

This crate centralizes:
- Linking to `libR` (via `build.rs`).
- R initialization (`Rf_initialize_R`) and mainloop setup.
- A minimal runtime handle marking R as initialized for the process.

> Note: This crate uses **non-API** R internals (`Rembedded.h`,
> `Rinterface.h`). It is **not** intended for use inside R packages.

## When to use

- Rust-only binaries that need to embed R.
- Integration tests for crates that call into R.
- Benchmarks or tooling that want full control over R initialization.

## When not to use

- Code that will be built into an R package shared library.
- CRAN-facing packages (see Publishing to CRAN below).

## Usage

```rust
// SAFETY: Must be called once, from the main thread.
let _engine = unsafe {
    miniextendr_engine::REngine::build()
        .with_args(&["R", "--quiet", "--vanilla"])
        .init()
        .expect("Failed to initialize R")
};

// ... use R APIs from the main thread ...
// R remains initialized when the handle leaves scope.
```

R shutdown is intentionally skipped: `Rf_endEmbeddedR` is not reentrant-safe
and can crash if called during Drop or test teardown. Let the OS reclaim
resources on process exit. `REngine` has a private constructor, so obtaining a
handle proves that initialization completed successfully; dropping the handle
does not shut R down.

### Initialization details

- `R_HOME` is ensured (via `R RHOME`) if missing.
- `Rf_initialize_R` is used (not `Rf_initEmbeddedR`) to avoid double
  `setup_Rmainloop()` calls.
- `setup_Rmainloop()` is called exactly once after initialization.

### Why this crate exists

Embedding logic used to be duplicated across binaries/benches, which made it
easy for fixes to drift and hard to debug initialization problems. Centralizing
embedding here provides:

- **Consistent linking** – `build.rs` resolves `R_HOME` and emits `-lR` so
  dependents link correctly without re‑implementing discovery logic.
- **Safe initialization** – avoids double‑calling `setup_Rmainloop()` (a common
  source of crashes when `Rf_initEmbeddedR()` is followed by a manual call).
- **Single source of truth** – keeps `R_HOME` handling and initialization
  sequencing in one place.

### Runtime sentinel

The engine exposes a lightweight sentinel:

```rust
if miniextendr_engine::r_initialized_sentinel() {
    // R has been initialized in this process.
}
```

This checks the C stack markers set by `Rf_initialize_R`.

## Requirements

- R installed and `R` available on PATH.
- System linker able to link against `libR` (handled by `build.rs`).

## Publishing to CRAN

This crate **must not** be used by an R package that is intended for CRAN. It
relies on non-API R internals and performs embedding, which is not appropriate
inside an R process.

Acceptable uses in a CRAN workflow:
- As a **dev-dependency** for Rust tests or benchmarks that run outside the R
  package build.
- For **integration tests** executed by developers (not during CRAN checks).

For CRAN packages, use `miniextendr-api` and `miniextendr-macros` instead, and
keep the `nonapi` feature disabled.

## Maintainer

- Keep `build.rs` in sync with R installation behavior across platforms.
- Verify the initialization sequence against current R sources before changing
  any embedding logic.
- Update `r_initialized_sentinel()` if R changes how it initializes stack
  markers.
- Keep this README’s rationale and safety notes current when behavior changes.
- Ensure no `Rf_endEmbeddedR` is called in Drop (non-reentrant cleanup risk).
