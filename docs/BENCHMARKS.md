# Benchmarks

Performance-investigation tooling for miniextendr: how to run the
bench suite, how to capture a baseline, and what the published
numbers actually mean.

`miniextendr-bench` embeds R through `miniextendr-engine`, exercises
every FFI path the runtime cares about (conversions, ALTREP,
ExternalPtr, worker routing, trait ABI, unwind protection, GC
protection, class dispatch, …), and prints numbers that engineers can
compare against an earlier baseline before landing a change.

The crate is `publish = false` and is **never** shipped in a CRAN
tarball. It is for maintainer and contributor use only.

## Why benchmarks exist

miniextendr's selling points are safety, small FFI overhead, and
ALTREP throughput. Those are empirical claims. The only way to keep
them true across refactors is to measure them. Every non-trivial
change to a hot path (conversions, ALTREP callbacks, unwind protect,
worker dispatch, class-system codegen) should be accompanied by at
least one `just bench`/`just bench-save` run and, ideally, a
`just bench-drift` check against the prior baseline.

### The FFI model being measured

miniextendr exposes Rust to R the same way [cpp11](https://cpp11.r-lib.org/)
exposes C++: a shared library (`.so` / `.dylib` / `.dll`) is loaded
into the R session, and R calls into it via registered `.Call()`
entry points. R hands the callee an array of `SEXP` arguments and
receives a `SEXP` back. No marshalling, no serialisation, no IPC.
Most of what these benchmarks measure is the thin work that happens
on top of that already-fast call: argument conversion, GC protection,
unwind guarding, ALTREP materialisation, and the occasional hop to
the worker thread. If a benchmark moves, it is one of those layers
that moved, not the `.Call()` dispatch itself.

## Running benchmarks

Prerequisites:

- R installed and reachable on `PATH`
- `rustup` with the workspace-pinned toolchain
- `cargo-limit` (optional, via `just dev-tools-install`)

### Everyday recipes

```bash
# Full Rust suite (all targets in miniextendr-bench/benches/)
just bench

# A single target (e.g. trait_abi dispatch microbenchmarks)
just bench --bench trait_abi

# Core high-signal subset (ffi_calls, into_r, from_r, translate,
# strings, externalptr, worker, unwind_protect)
just bench-core

# Feature-gated (connections, rayon)
just bench-features

# Full matrix: core + feature targets
just bench-full
```

### Specialised targets

```bash
# Lint-scan benchmarks (miniextendr-lint/benches/lint_scan.rs)
just bench-lint

# Macro compile-time benchmark (synthetic crates, cold/warm/incr)
just bench-compile

# R-side benchmarks (requires rpkg installed + the R `bench` package)
just bench-r
```

### Filtering inside a target

Everything after `--` is forwarded to `cargo bench`, and everything
after a second `--` is forwarded to the `divan` harness. Divan
supports regex filtering and baseline save/compare:

```bash
# Only run the `altrep_int_dataptr` benchmarks inside the altrep target
just bench --bench altrep -- altrep_int_dataptr

# Save a divan baseline named "main" for the worker target
cargo bench --manifest-path=miniextendr-bench/Cargo.toml \
  --bench worker -- --save-baseline main

# …make changes, then compare against that baseline
cargo bench --manifest-path=miniextendr-bench/Cargo.toml \
  --bench worker -- --baseline main
```

## Structured baselines and drift detection

The repository ships a small wrapper around divan that stores raw
output, a machine-readable CSV, and git metadata under
`miniextendr-bench/baselines/`.

| Recipe | What it does |
|--------|--------------|
| `just bench-save` | Run the suite, parse output, write `bench-<timestamp>.txt` + `bench-<timestamp>.csv` + `bench-<timestamp>.meta.json` to `miniextendr-bench/baselines/` |
| `just bench-compare [file.csv]` | Print the slowest/fastest benchmarks from the most recent (or specified) baseline |
| `just bench-drift [--threshold=20]` | Compare the two most recent baselines; fail if any median regressed beyond the threshold percentage |
| `just bench-info` | List saved baselines with git-SHA, R version, CPU, date |

The CSV schema is:

```
timestamp,bench_target,group,name,args,median_ns,unit
```

Drift detection uses `median_ns` so unit-changes (ns ↔ us ↔ ms)
never produce false positives. The implementation lives in
`tests/perf/bench_baseline.sh`.

## Environment setup for reproducible runs

Noise is the enemy. Before a baseline run:

1. Close heavy background workloads (editors with LSP servers, Docker,
   video calls). A 10% jitter floor is normal; 30%+ usually means
   something else is hitting the CPU.
2. Disable CPU turbo and frequency scaling if possible.
3. On macOS, disable App Nap for the shell running the bench.
4. On Linux, pin to specific cores via `taskset -c <N>` and disable
   hyper-thread siblings. `DIVAN_SKIP_SLOW=1` skips the 50 K-size
   end of the size matrix for faster iteration while developing.

Before saving a baseline, capture the environment so a later reader
can reproduce it:

```bash
# System / OS
uname -a
sysctl -n machdep.cpu.brand_string            # macOS
cat /proc/cpuinfo | grep -m1 'model name'     # Linux

# R
R --version | head -1
Rscript -e 'sessionInfo()'

# Rust
rustc --version
cargo --version

# miniextendr
git describe --always --dirty
```

`just bench-save` captures most of this automatically into the
`.meta.json` sidecar.

### Environment variables

| Variable | Effect |
|----------|--------|
| `R_HOME` | Selects the R installation the embedder uses |
| `DIVAN_SKIP_SLOW=1` | Skips the largest size in each size matrix |
| `RAYON_NUM_THREADS` | Thread count for the `rayon` benchmarks |
| `CARGO_INCREMENTAL=0` | Matches CI; avoids per-run hash noise in sccache |

## What each target measures

Each file under `miniextendr-bench/benches/` focuses on one subsystem
so iterations stay fast and failures point at a single owner. The
high-level plan lives in `miniextendr-bench/src/bench_plan.rs`.

| Target | Measures |
|--------|----------|
| `allocator` | R allocator vs system allocator across size matrix |
| `altrep` / `altrep_advanced` / `altrep_iter` | ALTREP element access, `DATAPTR` materialisation, guard modes, string ALTREP, zero-alloc patterns |
| `coerce` | R-side `Rf_coerceVector` vs Rust-side coercion |
| `connections` | Custom R connections: build/open, read, write, burst write (feature `connections`) |
| `dataframe` | `Vec<Struct>` → R data.frame transpose + full pipelines |
| `externalptr` | Creation, access, downcast, N-ptr churn; vs plain `Box` baseline |
| `factor` | Cached vs uncached level lookup; `FactorVec<T>` throughput |
| `ffi_calls` | Raw `Rf_*` calls (`ScalarInteger`, `allocVector`, `protect`/`unprotect`, `install`) |
| `from_r` | `TryFromSexp` scalar/slice/map/set paths |
| `gc_protect` | `OwnedProtect`, `ProtectScope`, manual `PROTECT`/`UNPROTECT` |
| `gc_protection_compare` | Seven protection mechanisms head-to-head (stack, vec pool, slotmap, precious list, DLL preserve, reprotect-slot, DLL reinsert) |
| `into_r` | `IntoR` scalar and vector (incl. 1 M-element scale benchmarks) |
| `list` | List construction, named lookup, derive-based builders |
| `native_vs_coerce` | RNative memcpy path vs element-wise coercion |
| `panic_telemetry` | Panic-hook `RwLock` read cost, fire cost |
| `preserve` | Preserve list insert/release vs `PROTECT`/`UNPROTECT` |
| `raw_access` | `r_slice`, unchecked slice, raw pointer access across INTSXP/REALSXP |
| `rarray` | `RArray`/`RMatrix` row/col access patterns |
| `rayon` | `rayon_bridge` parallel helpers (feature `rayon`) |
| `refcount_protect` | `RefCountedArena` vs `ProtectScope` vs raw preserve |
| `rffi_checked` | `#[r_ffi_checked]` overhead vs `*_unchecked` variant |
| `sexp_ext` | `SexpExt` helpers vs raw pointer deref |
| `strict` | Strict-mode scalar/vector conversions vs normal |
| `strings` | UTF-8/Latin-1 extraction, `mkCharLen`, `translateCharUTF8` |
| `trait_abi` | `mx_erased` vtable query + dispatch (single-method and 5-method) |
| `translate` | `R_CHAR` vs `translateCharUTF8` extraction costs |
| `typed_list` | `typed_list!` validation across field count |
| `unwind_protect` | `with_r_unwind_protect` overhead; nested layers; panic path |
| `worker` | Worker-thread round-trip, channel saturation, batching, payload size |
| `wrappers` | Generated R wrapper dispatch (plain, env, R6, S3, S4, S7); requires rpkg installed |

Each benchmark uses the shared `miniextendr_bench::init()` harness
from `miniextendr-bench/src/lib.rs`, which embeds R on the init
thread and asserts thread affinity on every call. The canonical size
matrix is `[1, 16, 256, 4096, 65536]`; named-list benchmarks use
`[16, 256, 4096]`.

## Interpreting results

Divan prints four numbers per case:

- **fastest**: the single fastest sample. Useful for latency-bound
  comparisons but noisy across runs.
- **slowest**: the slowest sample. A sudden slowest spike hints at GC
  pauses, OS scheduling, or an allocator interaction worth chasing.
- **median**: the canonical number for drift detection. Not skewed
  by a single spike.
- **mean / std-dev**: use when comparing two allocation-heavy paths
  where allocator variance matters.

Rules of thumb:

- For anything under ~50 ns, absolute deltas are almost always below
  measurement noise. Compare *ratios* across a size matrix instead.
- For ALTREP and worker paths, always report both the cold path
  (first call, includes class/channel init) and the hot path.
- String and CHARSXP allocation dominates large-vector conversion
  timings. Changes in `strings`/`into_r` that don't touch
  `mkCharLenCE` should not move those numbers.

## Reference baseline: 2026-04-20 (Apple M3 Max)

Full raw data, all tables, and methodology notes are in
[`miniextendr-bench/BENCH_RESULTS_2026-04-20.md`](../miniextendr-bench/BENCH_RESULTS_2026-04-20.md).
The headline numbers below are a subset for quick reference.

Environment: `rustc 1.95.0`, R 4.5.2, macOS 25.4.0 arm64, 36 GB RAM.
Commit `00df79d4` on `main`.

### Quick reference

| Subsystem | Operation | Median | Notes |
|-----------|-----------|--------|-------|
| Worker thread | `run_on_worker_no_r` | 0.46 ns | compiles to inline call |
| Worker thread | `with_r_thread` (on main) | 11.6 ns | near free |
| Unwind protect | `with_r_unwind_protect` | 32.6 ns | overhead vs direct |
| Unwind protect | nested 5 layers | 161 ns | linear |
| `catch_unwind` | success path | 0.46 ns | |
| `catch_unwind` | panic caught | 5.4 µs | panic + catch |
| ExternalPtr | create (8 B) | 209 ns | Box baseline 18 ns (~12×) |
| ExternalPtr | create (64 KB) | 209 ns | Box baseline 667 ns (allocator fast) |
| Trait ABI | `mx_query_vtable` | 2.3 ns | cache-hot |
| Trait ABI | single-method dispatch | 55–60 ns | view + call |
| Trait ABI | all-5-method dispatch | 417 ns | multi-method hot loop |
| R allocator | small (8 B) | 18.0 ns | system 16.5 ns |
| R allocator | large (64 KB) | 797 ns | system 82 ns |

### Type conversions (64 K elements unless noted)

| Type | `into_sexp` median | `try_from_sexp` median |
|------|--------------------|-------------------------|
| `i32` scalar | 11.7 ns | 23.8 ns |
| `f64` scalar | 12.0 ns | 21.9 ns |
| `Vec<i32>` | 9.0 µs | - |
| `Vec<f64>` | 16.6 µs | - |
| `Vec<String>` | 3.87 ms | - |
| `Vec<Option<i32>>` (50% NA) | 29.2 µs | - |
| `slice_i32` / `slice_f64` | - | 21 ns (zero-copy) |
| `HashSet<i32>` from 64 K | - | 1.49 ms |

**1 M element scale:** `i32` 106 µs; `f64` 233 µs; `String` 94.6 ms
(CHARSXP allocation dominates); `Option<i32>` 440 µs.

### ALTREP (64 K elements)

| Operation | ALTREP | Plain INTSXP |
|-----------|--------|--------------|
| element access (elt) | 15.7 µs | 9.1 ns |
| `DATAPTR` materialisation | 17.1 µs | 9.3 ns |
| full scan via elt loop | 4.79 ms | - |
| full scan via `DATAPTR` | 21.5 µs | 9.4 ns |

**Guard modes (64 K full scan):** `unsafe` 1.07 ms • `rust_unwind`
(default) 4.71 ms • `r_unwind` 4.70 ms • plain INTSXP 256 µs.
`unsafe` is ~4× faster than the default guard; `r_unwind` and default
are similar (both use `catch_unwind` internally; `r_unwind` adds
`R_UnwindProtect` per callback).

**String ALTREP (64 K strings):** create 2.51 ms • elt access 2.81 ms •
elt with NA 2.72 ms • force materialise 6.22 ms • plain STRSXP elt
4.47 ms.

### GC protection (per-op)

| Mechanism | Single op | Notes |
|-----------|-----------|-------|
| Protect stack | 1.9 ns | array write + counter decrement |
| Vec pool (VECSXP) | 13.0 ns | + free list |
| Slotmap pool | 14.4 ns | + generational safety check |
| Precious list | 15.3 ns | CONS alloc + prepend |
| DLL preserve | 29.1 ns | CONS alloc + doubly-linked splice |

**Replace-in-loop (10 K iterations):** Vec pool 1.14 ms •
DLL preserve 1.30 ms • Precious churn 74 ms
(O(n²); do not use for reassignment).

### Lint (miniextendr-lint)

| Benchmark | Scale | Median |
|-----------|-------|--------|
| `full_scan` | 10 modules | 2.2 ms |
| `full_scan` | 100 modules | 19.8 ms |
| `full_scan` | 500 modules | 114 ms |
| `impl_scan` | 10 types | 2.4 ms |
| `impl_scan` | 100 types | 22.2 ms |
| `scaling` | 500 fns / 10 files | 8.6 ms |
| `scaling` | 500 fns / 500 files | 76.8 ms |

Linear in both module count and file count.

### Connections (feature `connections`)

| Operation | Size | Median |
|-----------|------|--------|
| `connection_build` + open | - | 625 ns |
| `connection_write` | - | 28 ns |
| `connection_read` | 64 B | 24 ns |
| `connection_read` | 16 KB | 273 ns |
| `connection_write_sized` | 16 KB | 922 ns |
| `connection_burst_write` | 50 items | 1.04 µs |

## Publishing new baselines

When a PR changes numbers that show up in this document, attach a
fresh baseline to the PR and point `BENCH_RESULTS_<date>.md` at it.
The workflow:

1. `just bench-save` on a quiet machine.
2. Commit the new raw results file under
   `miniextendr-bench/BENCH_RESULTS_<YYYY-MM-DD>.md` (re-using the
   template from the 2026-02-18 file).
3. Update the "Reference baseline" section in this document to point
   at the new file.
4. Mention in the PR description which subsystem moved and by how
   much. If the reference baseline's machine differs from the one
   used for the run, note it. Don't silently mix hardware.

Old baselines stay in the tree. `miniextendr-bench/baselines/` plus
the dated `BENCH_RESULTS_*.md` files are the historical record used
by `just bench-drift`.

## Skipped / known issues

- **`wrappers`**: Requires `rpkg` installed to provide R class
  methods. Skipped on a cold checkout; run
  `just rcmdinstall && just bench --bench wrappers`.
- **R-side benchmarks**: Need rpkg installed and the R `bench`
  package. Run `just bench-r`.
- **Macro compile-time**: Generates synthetic crates on a
  temp-dir. Run `just bench-compile`; the results are written to the
  temp-dir and summarised on stdout.
- **Feature combinations that conflict with `--all-features`**:
  `r6-default` and `s7-default` are mutually exclusive, so
  `--all-features` fails. Use the explicit feature list from
  `CLAUDE.md`'s "Reproducing CI clippy before PR" section when you
  need the CI-equivalent set.

## Reference

- Raw results file: [`miniextendr-bench/BENCH_RESULTS_2026-04-20.md`](../miniextendr-bench/BENCH_RESULTS_2026-04-20.md)
- Previous baseline: [`miniextendr-bench/BENCH_RESULTS_2026-02-18.md`](../miniextendr-bench/BENCH_RESULTS_2026-02-18.md)
- Benchmark plan: `miniextendr-bench/src/bench_plan.rs`
- Harness and fixtures: `miniextendr-bench/src/lib.rs`
- Baseline tooling: `tests/perf/bench_baseline.sh`
- Cross-reference analysis (GC protection):
  `analysis/gc-protection-strategies.md` and
  `analysis/gc-protection-benchmarks-results.md`
