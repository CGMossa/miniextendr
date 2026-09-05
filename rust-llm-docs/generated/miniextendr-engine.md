# miniextendr_engine v0.1.0

miniextendr-engine: standalone R embedding for Rust binaries and tests.

This crate centralizes `libR` linking (via `build.rs`), R initialization, and
a minimal runtime handle. It is intended for Rust-only executables and
integration tests that embed R.

**Not for R packages:** this crate uses non-API R internals
(`Rembedded.h`, `Rinterface.h`). For R packages, depend on `miniextendr-api`
and keep `nonapi` disabled.

## When to use
- Rust binaries that embed R.
- Integration tests or benchmarks that need full control over R startup.

## Quick start

```no_run
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

## Initialization details
- Ensures `R_HOME` (via `R RHOME`) if missing.
- Calls `Rf_initialize_R` directly to avoid double `setup_Rmainloop()`.
- Calls `setup_Rmainloop()` exactly once after initialization.

## Runtime sentinel

```no_run
if miniextendr_engine::r_initialized_sentinel() {
    // R has been initialized in this process.
}
```

## Safety

- Must only be initialized once per process.
- Must be called from the main thread.
- No shutdown: `Rf_endEmbeddedR` is intentionally not called because the
  cleanup path is not reentrant-safe. The OS reclaims resources on exit.

---

## Structs

### `REngine`

```rust
pub struct REngine
```

Handle to an initialized R runtime.

This is a marker type indicating R has been initialized for this process.
R cleanup (via `Rf_endEmbeddedR`) is intentionally NOT called because it
performs non-reentrant operations that can crash if called during Drop
or concurrent with other cleanup. The OS reclaims all resources on process exit.

The handle cannot be constructed directly; only a successful
[`REngineBuilder::init`] can create it.

```compile_fail
let _forged = miniextendr_engine::REngine;
```

**Inherent associated items:**

#### `build`

```rust
fn build() -> REngineBuilder
```

Create a new builder for configuring R initialization.

### `REngineBuilder`

```rust
pub struct REngineBuilder
```

Builder for configuring and initializing the R runtime.

#### Example

```no_run
# fn main() -> Result<(), miniextendr_engine::REngineError> {
let _engine = unsafe {
    miniextendr_engine::REngine::build()
        .with_args(&["R", "--quiet", "--no-save"])
        .interactive(false)
        .signal_handlers(false)
        .init()?
};
# Ok(())
# }
```

**Inherent associated items:**

#### `init`

```rust
unsafe fn init(self: Self) -> Result<REngine, REngineError>
```

Initialize the R runtime with the configured settings.

##### Safety

- Must only be called once per process
- Must be called from the main thread
- R cannot be safely shutdown and reinitialized

##### Errors

Returns an error if R initialization fails.

#### `interactive`

```rust
fn interactive(self: Self, interactive: bool) -> Self
```

Set whether R should run in interactive mode.

Default is `false`.

#### `new`

```rust
fn new() -> Self
```

Create a new R engine builder with default settings.

#### `signal_handlers`

```rust
fn signal_handlers(self: Self, enable: bool) -> Self
```

Set whether R should install signal handlers.

Default is `false`. Set to `true` if you want R to handle Ctrl+C etc.

#### `with_args`

```rust
fn with_args(self: Self, args: &[&str]) -> Self
```

Set the command-line arguments for R initialization.

Default is `["R", "--quiet", "--vanilla"]`.

---

## Enums

### `REngineError`

```rust
pub enum REngineError
```

Errors that can occur during R engine initialization.

**Variants:**

- `RHomeNotFound { stderr: Option<String> }`
  - Could not determine / set `R_HOME` for embedding.
- `InitializationFailed`
  - R initialization failed.
- `AlreadyInitialized`
  - R is already initialized. Re-initialization is not supported.

---

## Functions

### `r_initialized_sentinel`

```rust
fn r_initialized_sentinel() -> bool
```

Check whether `Rf_initialize_R` has run by inspecting stack sentinels.

`R_CStackStart`/`R_CStackDir` are set during R initialization on the main
thread. A zero or `usize::MAX` value indicates "not initialized".
