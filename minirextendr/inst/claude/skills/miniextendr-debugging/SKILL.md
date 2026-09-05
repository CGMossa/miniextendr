---
name: miniextendr-debugging
description: Use when a miniextendr-backed R package misbehaves — "could not find function" after adding Rust code, library() exposes nothing, configure or autoconf errors, cargo/link failures, dyn.load symbol-not-found, segfaults, suspected garbage-collector bugs, MXL lint failures, or a build that suddenly ignores your Rust edits.
---

# Debugging a miniextendr package

Work through the section that matches the symptom. When in doubt, start with:

```r
minirextendr::miniextendr_doctor()
```

It checks the Rust toolchain, `src/rust/Cargo.toml`, stale
`inst/vendor.tar.xz`, missing `src/rust/.cargo/config.toml`, stale generated
files, `NAMESPACE`, and git hooks — most "weird" states show up here.

## "could not find function" / new function invisible in R

Check in order:

1. The function is `pub` **and** has `#[miniextendr]`.
2. Its module is reachable from `lib.rs` (`mod my_module;` present —
   unreachable modules are silently skipped).
3. Rebuild the whole chain: `minirextendr::miniextendr_build()`. A new export
   needs wrappers regenerated *and* `NAMESPACE` updated *and* a reinstall;
   `miniextendr_build()` does all three (including the second install when
   NAMESPACE changed).
4. Restart R. An already-loaded package image doesn't pick up the new `.so`.

## `library(pkg)` exposes nothing (empty namespace)

Almost always the **vendor latch**: `inst/vendor.tar.xz` is present, so
`./configure` selected offline *tarball* mode, which skips wrapper generation
and expects pre-shipped `R/<pkg>-wrappers.R`. On a package that has none, you
get an empty namespace.

- This can happen when a build-producing frontend runs `bootstrap.R` on a fresh
  package before wrappers exist. A direct `R CMD INSTALL .` remains in source
  mode because configure never vendors.
- Fix: `minirextendr::miniextendr_clean_vendor_leak()` (removes the stale
  tarball), then `minirextendr::miniextendr_build()`.
- `MINIEXTENDR_FORCE_WRAPPER_GEN=1` forces wrapper generation even in tarball
  mode (debugging escape hatch).

Related symptom of the same latch: **your Rust edits are silently ignored**
(the build compiles vendored copies, not your tree). Same fix.

## configure problems

- `configure: command not found` → generate it: `autoconf` (or
  `minirextendr::miniextendr_autoconf()`), then `bash ./configure`.
- Always invoke as **`bash ./configure`** — bare `./configure` runs under
  `#!/bin/sh` and produces spurious errors in the config-command passthrough.
- autoconf missing → `brew install autoconf automake` /
  `apt-get install autoconf automake`.
- `src/Makevars` or `src/rust/.cargo/config.toml` missing → run
  `bash ./configure` again; they are generated per install mode.

## Cargo build / link failures

- Local `path = "../.."` dependency fails under `R CMD INSTALL` but works with
  `cargo build`: R copies the package to a temp dir before compiling, so
  **relative** path deps break. Use an absolute path in `src/rust/Cargo.toml`,
  or vendor the crate (`minirextendr::use_vendor_lib()`).
- `Cargo.lock` shape errors in tarball mode →
  `minirextendr::miniextendr_repair_lock()`.

### dyn.load: "symbol not found in flat namespace" (macOS) / "undefined symbol" (Linux)

R links the Rust staticlib into `<pkg>.so` with `-undefined dynamic_lookup`,
and `dyn.load()` resolves *every* referenced symbol eagerly (`RTLD_NOW`).
A symbol that a dependency references but never calls (e.g. a C API removed
by a newer toolchain) passes `cargo test` yet breaks `dyn.load`.

Diagnose — symbols undefined in the archive and provided by nothing you link:

```sh
LIB=$(find . -name 'lib*.a' -path '*rust*' | head -1)
comm -23 <(nm -u "$LIB" | awk '{print $NF}' | sort -u) \
         <(nm -g --defined-only "$LIB" | awk '{print $NF}' | sort -u)
```

Fix: add abort-on-call stubs to `src/stub.c` for the genuinely unreachable
symbols:

```c
#include <stdlib.h>
void TheMissingSymbol(void) { abort(); }
```

The `.so` then loads; if the "impossible" path is ever hit, it aborts loudly.
Remove the stub when the upstream dependency drops the reference.

## Segfaults

```sh
R -d lldb -e 'mypkg::crashing_fn(args)'
# at the (lldb) prompt: run
# after the crash: bt          — backtrace
#                  frame select N / p variable
```

On Linux use `R -d gdb`. If the backtrace lands in R's garbage collector or
in allocator corruption (frames mentioning gc, SET_VECTOR_ELT on a freed
object, or `malloc` internals), suspect a protection bug — next section.

## GC bugs (use-after-free of R objects held in Rust)

Signature: crashes that appear "sometimes", only under load, or only on CI
(`malloc(): unsorted double linked list corrupted`, `caught segfault`).
Cause: an SEXP held in Rust state (a `Vec<SEXP>`, cached field, builder)
without R protection, collected when a later allocation triggers GC.

Deterministic reproduction with GC torture — **load the package first, then
enable torture** (enabling first crashes unrelated package-load paths):

```r
library(mypkg)
gctorture(TRUE)
mypkg::suspect_fn(args)   # repeat a few times
gctorture(FALSE)
```

For a whole-suite sweep use `gctorture2(step = 100)` (≈100× slowdown, still
catches most bugs) around `testthat::test_dir()`. In Rust, fix by rooting the
value: `OwnedProtect` / `ProtectScope` from `miniextendr_api` (RAII
protect/unprotect), never a raw unprotected SEXP across allocations.

## MXL lint failures (from `build.rs`)

The framework lints your crate during `cargo build`. The rules you will
actually meet:

- **MXL300** — direct `Rf_error`/`Rf_errorcall` call. Use `panic!()`; the
  framework converts panics to R errors *and* runs Rust destructors first.
- **MXL301** — `_unchecked` FFI call outside a known-safe context (ALTREP
  callback, `with_r_unwind_protect`, `with_r_thread`). Use the checked
  variant, or move the call into one of those contexts.
- **MXL106** — `#[miniextendr]` function that isn't `pub`.
- **MXL110** — parameter named after an R reserved word (`if`, `function`,
  `TRUE`, …) — the generated R wrapper would not parse. Rename it.

Escape hatch for exploration only: `MINIEXTENDR_LINT=0 cargo check`.
Violations are real defects — fix before committing.

## Panics show the wrong location / errors look generic

Rust panics become classed R conditions (`rust_condition_value` →
`.miniextendr_raise_condition`). Use `tryCatch(..., error = function(e) ...)`
normally; `conditionMessage(e)` carries the panic message. If you see a raw
pointer or corrupted object instead of an error, you are calling a generated
wrapper that was hand-edited — regenerate (`miniextendr_build()`), never edit
`R/<pkg>-wrappers.R`.

## R CMD check hangs at "checking tests" (Windows)

A dependency spun up a multi-threaded Tokio runtime; worker threads keep
Rterm's stdout pipe open forever. Build a `current_thread` runtime instead:

```rust
let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
rt.block_on(async { /* ... */ });
```

## Escalation checklist

Before filing an issue against miniextendr:

1. `minirextendr::miniextendr_doctor()` output attached.
2. Reproduced after `miniextendr_clean_vendor_leak()` + fresh
   `miniextendr_build()`.
3. For crashes: `lldb`/`gdb` backtrace and, if GC-shaped, a
   `gctorture(TRUE)` minimal repro.

Full manual: https://a2-ai.github.io/miniextendr
