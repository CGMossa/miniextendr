# Feature Flags Reference

miniextendr-api uses Cargo feature flags to enable optional integrations.
Only `default` features are enabled automatically.

## Quick Reference

| Feature | What it enables | Dependencies added |
|---------|----------------|-------------------|
| **Default** | | |
| `doc-lint` | Build-time lint checking `#[miniextendr]` source-level attributes | (forwarded to miniextendr-macros) |
| **Core / R Integration** | | |
| `nonapi` | Non-API R symbols (stack controls, mutable `DATAPTR`) | (none) |
| `rayon` | Parallel iterators via Rayon | rayon |
| `worker-thread` | Infrastructure for opt-in worker dispatch | (none) |
| `connections` | Experimental custom R connection framework | (none) |
| `indicatif` | Progress bars via R connections / console | indicatif (implies `connections`, `nonapi`) |
| `vctrs` | vctrs C API + `#[derive(Vctrs)]` macro | (forwarded to miniextendr-macros) |
| **Serialization** | | |
| `serde` | Direct Rust-R serialization (`RSerializeNative`, `RDeserializeNative`) | serde |
| `serde_json` | JSON string serialization (`RSerialize`, `RDeserialize`) | serde, serde_json |
| `toml` | TOML value conversions | toml |
| **Matrix / Array** | | |
| `ndarray` | N-dimensional array conversions (`Array1`..`Array6`, views) | ndarray |
| `nalgebra` | Linear algebra types (`DVector`, `DMatrix`, `SVector`, `SMatrix`) | nalgebra |
| **Numeric Types** | | |
| `num-bigint` | Arbitrary-precision integers (`BigInt`, `BigUint`) | num-bigint, num-integer |
| `rust_decimal` | Fixed-point decimals (`Decimal`) | rust_decimal |
| `ordered-float` | NaN-orderable floats (`OrderedFloat<f64>`) | ordered-float |
| `num-complex` | Complex numbers (`Complex<f64>`) | num-complex |
| **Adapter Traits** | | |
| `num-traits` | Generic numeric operations (`RNum`, `RSigned`, `RFloat`) | num-traits |
| `bytes` | Byte buffer operations (`RBuf`, `RBufMut`) | bytes |
| **String / Text** | | |
| `uuid` | UUID conversions (`Uuid`, `Vec<Uuid>`) | uuid (with `v4` feature) |
| `regex` | Compiled regex from R patterns (`Regex`) | regex |
| `url` | Validated URL conversions (`Url`, `Vec<Url>`) | url |
| `aho-corasick` | Fast multi-pattern string search | aho-corasick |
| `globset` | Shell-style glob matching (`GlobSet`, path-aware defaults) | globset |
| **Date / Time** | | |
| `time` | `OffsetDateTime`, `Date`, `Duration` conversions | time (with formatting/parsing/macros) |
| `jiff` | `Timestamp`/`Zoned`/`civil::Date`/`SignedDuration` conversions + ALTREP + vctrs | jiff 0.2 (bundled tzdb) |
| **Random Number Generation** | | |
| `rand` | Wraps R's RNG with `rand` traits (`RRng`) | rand |
| `rand_distr` | Additional distributions (Normal, Exp, etc.) | rand, rand_distr |
| **Collections** | | |
| `indexmap` | Order-preserving maps (`IndexMap<String, T>`) | indexmap |
| `tinyvec` | Small-vector optimization (`TinyVec`, `ArrayVec`) | tinyvec (with `alloc`) |
| **Either / Sum Types** | | |
| `either` | `Either<L, R>` sum type conversions | either |
| **Binary Serialization** | | |
| `borsh` | Borsh binary serialization (`Borsh<T>` wrapper) | borsh (with `derive`) |
| **Bit Manipulation** | | |
| `bitflags` | Bitflags-integer conversions (`RFlags<T>`) | bitflags |
| `bitvec` | Bit vector-logical conversions (`RBitVec`) | bitvec |
| **Binary Data** | | |
| `raw_conversions` | POD types via bytemuck (`Raw<T>`, `RawSlice<T>`) | bytemuck (with `derive`) |
| `sha2` | SHA-256/SHA-512 hashing helpers | sha2 |
| `blake3` | BLAKE3 hashing helpers (hex + 32-byte digest) | blake3 (default-features off) |
| `md5` | MD5 hashing helpers (interop only — broken crypto) | md5 |
| `zstd` | Whole-buffer zstd compress/decompress | zstd (bundled C via zstd-sys) |
| **Formatting** | | |
| `tabled` | ASCII/Unicode table formatting | tabled |
| **Arrow / DataFusion** | | |
| `arrow` | Zero-copy R vector / data.frame to Apache Arrow array conversions | arrow-array, arrow-buffer, arrow-schema, arrow-select |
| `datafusion` | SQL query engine on R data frames via DataFusion (implies `arrow`) | datafusion, tokio |
| **Logging** | | |
| `log` | Routes Rust `log` macros (`info!`, `warn!`, `error!`) to R console | log |
| **Diagnostics** | | |
| `macro-coverage` | Macro expansion coverage module for auditing | (none) |
| `growth-debug` | Collection-growth counters and reports | (none) |
| **Project defaults** | | |
| `strict-default`, `coerce-default`, `fast-default` | Default conversion/wrapper policies | (forwarded to miniextendr-macros) |
| `r6-default`, `s7-default` | Default class system (mutually exclusive) | (forwarded to miniextendr-macros) |
| `worker-default` | Default worker dispatch | `worker-thread` |

---

## Default Features

### `doc-lint`

Enables the build-time lint that checks `#[miniextendr]` source-level attributes
for consistency. Warns on missing or mismatched annotations.

The advisories are delivered through rustc's `deprecated` lint (the macro emits a
`#[deprecated]` const that the generated code reads), so a crate that sets
`#![deny(deprecated)]` — or builds with `-D warnings` / `RUSTFLAGS=-Dwarnings` —
escalates them to hard errors. That is a reasonable strict mode, but be aware the
escalation is controlled by your lint configuration, not by this feature.

Forwarded to `miniextendr-macros/doc-lint`. Disable with `default-features = false` if
the lint causes issues during development.

---

## Core / R Integration Features

### `nonapi`

Enables access to non-API R symbols that are not part of R's public C API. These
symbols may change between R versions and will cause `R CMD check` warnings.

**What it unlocks:**
- `DATAPTR` -- mutable data pointer (prefer `DATAPTR_RO` when possible)
- `R_curErrorBuf` -- current R error message buffer
- `R_CStackStart`, `R_CStackLimit`, `R_CStackDir` -- process-global stack-check controls
- `scope_with_r()`, `spawn_with_r()`, `with_stack_checking_disabled()` --
  legacy stack-configuration utilities; they do not make off-main R API calls safe

See [NONAPI.md](NONAPI.md) for the full tracking list.

R's API remains main-thread-only. Package code must not change these stack
globals or use the helpers as a substitute for `with_r_thread`; removal or
relocation of that public surface is tracked in #1352.

### `worker-thread`

Enables the dedicated worker thread infrastructure. Without this feature, `run_on_worker()`
and `with_r_thread()` are lightweight inline stubs that execute closures directly on the
calling thread (no thread dispatch).

**With the feature enabled:**
- `miniextendr_runtime_init()` spawns a dedicated worker thread with bidirectional channels
- `run_on_worker(f)` dispatches `f` to the worker thread, returns `Result<T, String>`
- `with_r_thread(f)` routes `f` back to R's main thread from the worker

**Without the feature (the default):**
- `miniextendr_runtime_init()` only records the main thread ID
- `run_on_worker(f)` → `Ok(f())` (inline)
- `with_r_thread(f)` → `f()` (inline, panics if not on main thread)

The `worker-default` feature implies `worker-thread`.

### `rayon`

Parallel iterators via the Rayon crate, with R-safe interop.

**Provides:**
- `RParallelIterator` -- adapter trait for exposing parallel iterators to R
- `RParallelExtend` -- parallel collection building
- `with_r_vec()` -- zero-copy parallel fill into R vectors
- `with_r_matrix()` -- parallel matrix fill
- `reduce()` -- parallel reductions returning R scalars

See [RAYON.md](RAYON.md) for the full guide.

### `connections`

Experimental R connection framework. Wraps R's internal connection system for
creating custom readable/writable connections from Rust types.

**Warning:** R explicitly reserves the right to change the connection ABI without
a compatibility layer. A compile-time version check catches mismatches. Gated behind
this feature flag to make the instability opt-in.

**Provides:**
- `RConnectionImpl` trait -- implement to define custom connection behavior
- `RCustomConnection` builder -- configure and create R connection objects
- `std::io` adapters (`IoRead`, `IoWrite`, `IoReadWrite`, `IoReadWriteSeek`, `IoBufRead`)
- `RConnectionIo` builder -- auto-wraps any `std::io` type with zero boilerplate
- Helper functions: `get_connection()`, `read_connection()`, `write_connection()`

See [CONNECTIONS.md](CONNECTIONS.md) for the full guide with examples.

### `indicatif`

Progress bars rendered in the R console via the `indicatif` crate. Output is routed
through `ptr_R_WriteConsoleEx` (a non-API symbol), so this feature implies `nonapi`.

All output is a no-op when called off the R main thread.

**Provides:**
- `progress::RTerm` -- `TermLike` implementation for R console output
- `progress::RStream` -- stdout/stderr target selection
- Convenience constructors: `term_like_stdout()`, `term_like_stderr()`, `term_like_*_with_hz()`

See [PROGRESS.md](PROGRESS.md) for the full guide with examples.

### `vctrs`

Access to the vctrs R package's maturing C API, plus the `#[derive(Vctrs)]` proc macro
for defining custom vctrs-compatible classes.

**C API wrappers:**
- `init_vctrs()` -- load function pointers via `R_GetCCallable`
- `obj_is_vector()` -- check if object is a vctrs vector
- `short_vec_size()` -- get vector size
- `short_vec_recycle()` -- recycle to target size

**Derive macro:**
- `#[derive(Vctrs)]` with `Vctr`, `Rcrd`, `ListOf` kinds
- `#[miniextendr(vctrs)]` impl blocks for methods
- `coerce = "type"` attribute for `vec_ptype2`/`vec_cast` generation
- `extends = "parent"` attribute for inheritance: prepends the parent into the
  class vector and generates child↔parent `vec_ptype2`/`vec_cast` stubs (the
  parent, as the supertype, wins coercion)

Requires the `vctrs` R package to be installed. See [VCTRS.md](VCTRS.md) for the full guide.

---

## Serialization Features

### `serde`

Direct Rust-R serialization with no JSON intermediate. Converts Rust structs to/from
native R objects (named lists, atomic vectors, etc.) using serde's `Serialize` and
`Deserialize` traits.

**Provides:**
- `RSerializeNative` / `RDeserializeNative` traits
- `AsSerialize<T>` wrapper for returning `Serialize` types from `#[miniextendr]` functions

**Type mappings:** structs become named lists, `Vec<primitive>` becomes atomic vectors,
`Option::None` becomes NA or NULL. See [SERDE_R.md](SERDE_R.md) for details.

### `serde_json`

JSON string serialization via `serde_json`. Implies `serde`.

**Provides:**
- `RSerialize` / `RDeserialize` traits (JSON-based)
- `JsonOptions`, `NaHandling`, `FactorHandling`, `SpecialFloatHandling` configuration
- `json_from_sexp()`, `json_into_sexp()` and variants (strict, permissive, custom)
- `JsonValue` / `RJsonValueOps` for working with JSON values

### `toml`

TOML value conversions between R lists/strings and TOML.

**Provides:**
- `TomlValue` / `RTomlOps` type and adapter trait
- `toml_from_str()`, `toml_to_string()`, `toml_to_string_pretty()` helpers

---

## Matrix / Array Features

### `ndarray`

N-dimensional array conversions between R vectors/matrices and the `ndarray` crate.

**Supported types:**
- Owned: `Array0` through `Array6`, `ArrayD` (dynamic dimensions)
- Views: `ArrayView0`..`ArrayView6`, `ArrayViewD`
- Mutable views: `ArrayViewMut0`..`ArrayViewMut6`, `ArrayViewMutD`
- Shared: `ArcArray1`, `ArcArray2`

**Adapter traits:** `RNdArrayOps`, `RNdIndex`, `RNdSlice`, `RNdSlice2D`

### `nalgebra`

Linear algebra type conversions between R vectors/matrices and `nalgebra`.

**Supported types:**
- Dynamic: `DVector`, `DMatrix`
- Static: `SVector<T, N>`, `SMatrix<T, R, C>`

**Adapter traits:** `RVectorOps`, `RMatrixOps`

---

## Numeric Type Features

### `num-bigint`

Arbitrary-precision integers via character string representation.

| Rust Type | R Type | Conversion |
|-----------|--------|------------|
| `BigInt` | `character(1)` | String parsing (signed) |
| `BigUint` | `character(1)` | String parsing (unsigned) |

**Adapter traits:** `RBigIntOps`, `RBigIntBitOps`, `RBigUintOps`, `RBigUintBitOps`

#### Example

```rust
use num_bigint::BigInt;

#[miniextendr]
fn factorial_big(n: i32) -> BigInt {
    (1..=n).fold(BigInt::from(1), |acc, x| acc * BigInt::from(x))
}

#[miniextendr]
fn bigint_add(a: BigInt, b: BigInt) -> BigInt {
    a + b
}
```

In R:

```r
factorial_big(50)
#> [1] "30414093201713378043612608166979581188299763898377..."

bigint_add("12345678901234567890", "98765432109876543210")
#> [1] "111111111011111111100"
```

### `rust_decimal`

Fixed-point decimal numbers via character string representation.

| Rust Type | R Type | Conversion |
|-----------|--------|------------|
| `Decimal` | `character(1)` | String parsing |

**Adapter trait:** `RDecimalOps`

#### Example

```rust
use rust_decimal::Decimal;
use std::str::FromStr;

/// Lossless addition -- pass values as strings to avoid f64 rounding.
#[miniextendr]
fn decimal_add(a: Decimal, b: Decimal) -> Decimal {
    a + b
}

#[miniextendr]
fn decimal_round(value: Decimal, dp: i32) -> Decimal {
    value.round_dp(dp.max(0) as u32)
}
```

In R:

```r
decimal_add("0.1", "0.2")
#> [1] "0.3"

decimal_round("3.14159", 2L)
#> [1] "3.14"
```

### `ordered-float`

NaN-orderable floating-point wrapper.

| Rust Type | R Type | Notes |
|-----------|--------|-------|
| `OrderedFloat<f64>` | `numeric` | Panics on NaN |

**Adapter trait:** `ROrderedFloatOps`

#### Example

```rust
use ordered_float::OrderedFloat;

/// Sort floats with total ordering (NaN sorts last).
#[miniextendr]
fn sort_floats(x: Vec<OrderedFloat<f64>>) -> Vec<OrderedFloat<f64>> {
    let mut v = x;
    v.sort();
    v
}

/// Find the minimum value (NaN-safe).
#[miniextendr]
fn safe_min(x: Vec<OrderedFloat<f64>>) -> OrderedFloat<f64> {
    x.into_iter().min().unwrap_or(OrderedFloat(f64::NAN))
}
```

In R:

```r
sort_floats(c(3.1, 1.4, 2.7))
#> [1] 1.4 2.7 3.1

safe_min(c(5.0, 2.0, 8.0))
#> [1] 2
```

### `num-complex`

Complex number support using R's native `CPLXSXP`.

| Rust Type | R Type | Notes |
|-----------|--------|-------|
| `Complex<f64>` | `complex` | Native R complex type |

**Adapter trait:** `RComplexOps`

#### Example

```rust
use num_complex::Complex;

#[miniextendr]
fn complex_magnitude(z: Complex<f64>) -> f64 {
    z.norm()
}

#[miniextendr]
fn complex_multiply(a: Complex<f64>, b: Complex<f64>) -> Complex<f64> {
    a * b
}
```

In R:

```r
complex_magnitude(3+4i)
#> [1] 5

complex_multiply(1+2i, 3+4i)
#> [1] -5+10i
```

---

## Adapter Trait Features

### `num-traits`

Generic numeric operations via blanket implementations over `num_traits` traits.

**Provides:**
- `RNum` -- basic numeric operations (add, sub, mul, div, rem, pow)
- `RSigned` -- signed number operations (abs, signum)
- `RFloat` -- floating-point operations (floor, ceil, round, sqrt, etc.)

#### Example

```rust
use miniextendr_api::num_traits_impl::RFloat;

/// Clamp a value to [0, 1] and return its square root.
#[miniextendr]
fn safe_sqrt(x: f64) -> f64 {
    let clamped = if x < 0.0 { 0.0 } else { x };
    RFloat::sqrt(&clamped)
}

/// Check if a number is a normal finite value (not zero, subnormal, inf, or NaN).
#[miniextendr]
fn is_normal(x: f64) -> bool {
    RFloat::is_normal(&x)
}
```

In R:

```r
safe_sqrt(2.0)
#> [1] 1.414214

is_normal(0.0)
#> [1] FALSE
```

### `bytes`

Byte buffer operations via the `bytes` crate.

**Provides:**
- `RBuf` -- read-only buffer adapter (wraps `Bytes`)
- `RBufMut` -- mutable buffer adapter (wraps `BytesMut`)
- Re-exports: `Buf`, `BufMut`, `Bytes`, `BytesMut`

#### Example

```rust
use bytes::Bytes;

/// Accept a raw vector and return its length.
#[miniextendr]
fn byte_length(data: Bytes) -> i32 {
    data.len() as i32
}

/// Concatenate two raw vectors.
#[miniextendr]
fn concat_bytes(a: Vec<u8>, b: Vec<u8>) -> Bytes {
    let mut combined = a;
    combined.extend_from_slice(&b);
    Bytes::from(combined)
}
```

In R:

```r
byte_length(charToRaw("hello"))
#> [1] 5

concat_bytes(as.raw(1:3), as.raw(4:6))
#> [1] 01 02 03 04 05 06
```

---

## String / Text Features

### `uuid`

UUID conversions between R character vectors and the `uuid` crate. Enables UUID v4
generation.

| Rust Type | R Type | Notes |
|-----------|--------|-------|
| `Uuid` | `character(1)` | Standard UUID format |
| `Vec<Uuid>` | `character` | Vector of UUIDs |

**Adapter trait:** `RUuidOps`
**Helpers:** `uuid_helpers` module

#### Example

```rust
use uuid::Uuid;

#[miniextendr]
fn generate_id() -> Uuid {
    Uuid::new_v4()
}

#[miniextendr]
fn batch_ids(n: i32) -> Vec<Uuid> {
    (0..n).map(|_| Uuid::new_v4()).collect()
}

#[miniextendr]
fn parse_uuid(s: String) -> Option<Uuid> {
    Uuid::parse_str(&s).ok()
}
```

In R:

```r
generate_id()
#> [1] "550e8400-e29b-41d4-a716-446655440000"

batch_ids(3)
#> [1] "a1b2c3d4-..." "e5f6a7b8-..." "c9d0e1f2-..."

parse_uuid("not-a-uuid")
#> [1] NA
```

### `regex`

Compiled regular expressions from R character patterns.

| Rust Type | R Type | Notes |
|-----------|--------|-------|
| `Regex` | `character(1)` | Compiled on conversion |

**Adapter traits:** `RRegexOps`, `RCaptureGroups`
**Types:** `CaptureGroups`

#### Example

```rust
use regex::Regex;

/// Pattern is compiled automatically from an R string.
#[miniextendr]
fn extract_numbers(pattern: Regex, text: String) -> Vec<String> {
    pattern.find_iter(&text).map(|m| m.as_str().to_string()).collect()
}

/// Split text on whitespace runs.
#[miniextendr]
fn split_whitespace(text: String) -> Vec<String> {
    let re = Regex::new(r"\s+").unwrap();
    re.split(&text).map(|s| s.to_string()).collect()
}
```

In R:

```r
extract_numbers("\\d+", "abc123def456")
#> [1] "123" "456"

split_whitespace("hello   world  test")
#> [1] "hello" "world" "test"
```

**Note:** `Regex` does not implement `IntoR` -- it is input-only. If you need to
reuse a compiled regex across calls, wrap it in an `ExternalPtr<Regex>`.

### `url`

Validated URL parsing via the `url` crate.

| Rust Type | R Type | Notes |
|-----------|--------|-------|
| `Url` | `character(1)` | Validated on conversion |
| `Vec<Url>` | `character` | Vector of validated URLs |

**Adapter trait:** `RUrlOps`
**Helpers:** `url_helpers` module

#### Example

```rust
use url::Url;

/// Extract the host from a URL, validating the input.
#[miniextendr]
fn get_host(url: Url) -> Option<String> {
    url.host_str().map(|s| s.to_string())
}

/// Filter a vector of URLs to only HTTPS.
#[miniextendr]
fn keep_https(urls: Vec<Url>) -> Vec<Url> {
    urls.into_iter().filter(|u| u.scheme() == "https").collect()
}

/// Join a relative path onto a base URL.
#[miniextendr]
fn join_path(base: Url, path: String) -> Result<Url, String> {
    base.join(&path).map_err(|e| e.to_string())
}
```

In R:

```r
get_host("https://example.com:8080/path")
#> [1] "example.com"

keep_https(c("https://a.com", "http://b.com", "https://c.com"))
#> [1] "https://a.com/" "https://c.com/"

join_path("https://example.com/api/", "v2/users")
#> [1] "https://example.com/api/v2/users"
```

### `aho-corasick`

Fast multi-pattern string search via the Aho-Corasick algorithm.

**Provides:**
- `AhoCorasick` type
- `aho_compile()` -- build automaton from patterns
- `aho_is_match()`, `aho_find_first()`, `aho_find_all()`, `aho_find_all_flat()`
- `aho_count_matches()`, `aho_replace_all()`

**Adapter trait:** `RAhoCorasickOps`

#### Example

```rust
use miniextendr_api::aho_corasick_impl::{aho_compile, aho_is_match, aho_replace_all};

#[miniextendr]
fn contains_any(patterns: Vec<String>, text: String) -> bool {
    let ac = aho_compile(&patterns).unwrap();
    aho_is_match(&ac, &text)
}

#[miniextendr]
fn censor_words(words: Vec<String>, text: String) -> String {
    let ac = aho_compile(&words).unwrap();
    aho_replace_all(&ac, &text, "***")
}
```

In R:

```r
contains_any(c("foo", "bar"), "hello foobar")
#> [1] TRUE

censor_words(c("bad", "ugly"), "the bad and the ugly")
#> [1] "the *** and the ***"
```

---

### `globset`

Shell-style glob matching (`*.R`, `**/*.rs`, `{foo,bar}/*`).

**Provides:**
- `GlobOptions` -- builder options (`literal_separator`, `case_insensitive`, `backslash_escape`)
- `build_globset(patterns, opts) -> Result<GlobSet, Error>`
- `globset_is_match(set, paths) -> Vec<bool>` -- TRUE if any pattern matches
- `globset_matches(set, path) -> Vec<i32>` -- 1-based indices of matching patterns
- `TryFromSexp for GlobSet` -- character vector of patterns, path-aware defaults

**Default semantics are path-aware** (`literal_separator = TRUE`): `*.R`
matches `a.R` but not `sub/a.R`. This differs from raw globset. Backslash
escaping is on for all platforms.

#### Example

```r
globset_is_match("*.R", c("a.R", "a.Rmd", "sub/a.R"))
#> [1]  TRUE FALSE FALSE
```

---

## Date / Time Features

### `time`

Date and time conversions via the `time` crate. Enables formatting, parsing, and
macro features.

| Rust Type | R Type | Notes |
|-----------|--------|-------|
| `OffsetDateTime` | `POSIXct` | Timezone-aware datetime |
| `Date` | `Date` | Calendar date |
| `Duration` | `difftime` | Time duration |

**Types:** `RDateTimeFormat`, `RDuration`

---

### `jiff`

Date and time conversions via the `jiff 0.2` crate with first-class IANA timezone
support. Uses a bundled timezone database (`tzdb-bundle-always`) so no system tzdata
is required.

| Rust Type | R Type | Notes |
|-----------|--------|-------|
| `Timestamp` | `POSIXct` (UTC) | Nanosecond-precision Unix timestamp |
| `Zoned` | `POSIXct` + `tzone` attr | Preserves IANA timezone identity |
| `civil::Date` | `Date` | Calendar date (no timezone) |
| `SignedDuration` | `difftime` (secs) | Signed nanosecond-precision duration |

**Adapter traits** (for `ExternalPtr` wrapping):
`RTimestamp`, `RDate`, `RZoned`, `RSignedDuration`, `RSpan`, `RDateTime`, `RTime`

**ALTREP:**
- `JiffTimestampVec` — lazy `Vec<Timestamp>` backed by `Arc`; elements projected on access, no upfront materialization.
- `JiffZonedVec` — lazy `Vec<Zoned>` backed by `Arc`; single-timezone strict (construction rejects heterogeneous timezones). Produces `POSIXct` with `tzone` attribute.

**vctrs rcrd constructors** (requires `vctrs` feature):
`span_vec_to_rcrd`, `zoned_vec_to_rcrd`, `datetime_vec_to_rcrd`, `time_vec_to_rcrd`

> **Timezone policy:** `Zoned → R` uses the element's IANA name; `R → Zoned`
> refuses unknown timezone names (no silent UTC fallback). `JiffZonedVec` enforces
> single-timezone invariant at construction time.

---

## Random Number Generation Features

### `rand`

Wraps R's built-in RNG with the `rand` crate's `RngCore` trait, allowing use of
any `rand`-compatible distribution with R's RNG state.

**Provides:**
- `RRng` -- R's RNG implementing `rand::RngCore`
- `RDistributions` -- direct access to R's native distributions (Normal, Uniform, etc.)

**Adapter traits:** `RRngOps`, `RDistributionOps`

### `rand_distr`

Re-exports the `rand_distr` crate for additional probability distributions
(Normal, Exponential, Gamma, etc.) that work with `RRng`. Implies `rand`.

---

## Collection Features

### `indexmap`

Order-preserving hash map conversions.

| Rust Type | R Type | Notes |
|-----------|--------|-------|
| `IndexMap<String, T>` | named `list` | Preserves insertion order |

**Adapter trait:** `RIndexMapOps`

### `tinyvec`

Small-vector optimization types that avoid heap allocation for small collections.

| Rust Type | R Type | Notes |
|-----------|--------|-------|
| `TinyVec<[T; N]>` | vector | Inline up to N, then spills to heap |
| `ArrayVec<[T; N]>` | vector | Fixed capacity N, never allocates |

---

## Either / Sum Type Features

### `either`

The `Either<L, R>` sum type from the `either` crate, with `TryFromSexp` and `IntoR` conversions.

| Rust Type | R Type | Notes |
|-----------|--------|-------|
| `Either<L, R>` | depends on variant | Left/Right converted via their own `IntoR`/`TryFromSexp` |

---

## Binary Serialization Features

### `borsh`

Binary Object Representation Serializer for Hashing (Borsh). Provides a `Borsh<T>` wrapper
for converting between Borsh-serialized binary data and R raw vectors.

| Rust Type | R Type | Notes |
|-----------|--------|-------|
| `Borsh<T>` | `raw` | Binary serialization via borsh derive |

---

## Bit Manipulation Features

### `bitflags`

Integer-bitflag conversions via the `bitflags` crate.

**Provides:**
- `RFlags<T>` -- wrapper for `Flags` types with integer conversion
- Re-exports `Flags` trait

### `bitvec`

Bit vector to logical vector conversions.

| Rust Type | R Type | Notes |
|-----------|--------|-------|
| `RBitVec` | `logical` | Backed by `BitVec` |

**Types:** `BitVec`, `Lsb0`, `Msb0`

---

## Binary Data Features

### `raw_conversions`

POD (Plain Old Data) type conversions via the `bytemuck` crate. Provides zero-copy
(when aligned) conversions between Rust structs and R raw vectors.

**Types:**
- `Raw<T>` -- single POD value (headerless)
- `RawSlice<T>` -- sequence of POD values (headerless)
- `RawTagged<T>` / `RawSliceTagged<T>` -- with header metadata

**Helpers:** `raw_from_bytes()`, `raw_to_bytes()`, `raw_slice_from_bytes()`, `raw_slice_to_bytes()`

**Re-exports:** `Pod`, `Zeroable` derive macros from bytemuck

**Note:** Not portable across architectures (native byte order, no endian conversion).

### `sha2`

Cryptographic hashing helpers.

**Provides:**
- `sha256_str(data) -> String (hex)` -- SHA-256 as 64-char hex string
- `sha256_bytes(data) -> String (hex)` -- SHA-256 of bytes, as 64-char hex string
- `sha512_str(data) -> String (hex)` -- SHA-512 as 128-char hex string
- `sha512_bytes(data) -> String (hex)` -- SHA-512 of bytes, as 128-char hex string

#### Example

```rust
use miniextendr_api::sha2_impl::{sha256_str, sha256_bytes};

#[miniextendr]
fn hash_string(s: String) -> String {
    sha256_str(&s)
}

#[miniextendr]
fn hash_raw(data: Vec<u8>) -> String {
    sha256_bytes(&data)
}
```

In R:

```r
hash_string("hello world")
#> [1] "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"

hash_raw(charToRaw("hello world"))
#> [1] "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
```

---

### `blake3`

Fast cryptographic hashing (SIMD-accelerated, 32-byte digests).

**Provides:**
- `blake3_str(s) -> String` -- BLAKE3 of a string, as 64-char hex
- `blake3_hex(data) -> String` -- BLAKE3 of bytes, as 64-char hex
- `blake3_bytes(data) -> Vec<u8>` -- BLAKE3 of bytes, raw 32-byte digest

#### Example

```rust
use miniextendr_api::blake3_impl::{blake3_str, blake3_bytes};

#[miniextendr]
fn hash_string(s: String) -> String {
    blake3_str(&s)
}

#[miniextendr]
fn hash_raw(data: Vec<u8>) -> Vec<u8> {
    blake3_bytes(&data)
}
```

In R:

```r
hash_string("")
#> [1] "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
```

---

### `md5`

Legacy MD5 hashing.

> **Security note:** MD5 is cryptographically broken. This feature exists for
> interoperability only (ETags, cache keys, `digest::digest(algo = "md5")`
> compatibility). Use `sha2` or `blake3` for anything security-relevant.

**Provides:**
- `md5_str(s) -> String` -- MD5 of a string, as 32-char hex
- `md5_hex(data) -> String` -- MD5 of bytes, as 32-char hex
- `md5_bytes(data) -> Vec<u8>` -- MD5 of bytes, raw 16-byte digest

#### Example

```r
md5_str("")
#> [1] "d41d8cd98f00b204e9800998ecf8427e"
md5_str("abc")
#> [1] "900150983cd24fb0d6963f7d28e17f72"
```

---

### `zstd`

Whole-buffer zstd compression — fills the gap left by `memCompress()` (which
covers gzip/bzip2/xz only).

**Provides:**
- `zstd_compress(data, level) -> Result<Vec<u8>, String>` -- level `0` = default (3); valid levels `1..=22`
- `zstd_decompress(data) -> Result<Vec<u8>, String>`

Builds the bundled zstd C library via `zstd-sys` (`cc` crate, pregenerated
bindings) — no cmake or pkg-config needed.

#### Example

```r
compressed <- zstd_compress(serialize(mtcars, NULL), 3L)
identical(unserialize(zstd_decompress(compressed)), mtcars)
#> [1] TRUE
```

---

## Formatting Features

### `tabled`

Table formatting for producing ASCII/Unicode tables from data.

**Provides:**
- `table_to_string()`, `table_to_string_styled()`, `table_to_string_opts()`
- `table_from_vecs()`, `builder_to_string()`
- Re-exports: `Table`, `Tabled`, `Builder`

#### Example

```rust
use tabled::Tabled;
use miniextendr_api::tabled_impl::table_to_string_styled;

#[derive(Tabled)]
struct Record { name: String, score: i32 }

#[miniextendr]
fn format_scores(names: Vec<String>, scores: Vec<i32>) -> String {
    let rows: Vec<Record> = names.into_iter().zip(scores)
        .map(|(name, score)| Record { name, score })
        .collect();
    table_to_string_styled(&rows, "markdown")
}
```

In R:

```r
cat(format_scores(c("Alice", "Bob"), c(95, 87)))
#> | name  | score |
#> |-------|-------|
#> | Alice | 95    |
#> | Bob   | 87    |
```

---

## Arrow / DataFusion Features

### `arrow`

Zero-copy conversions between R vectors/data.frames and Apache Arrow arrays/RecordBatch.
Foundation for DataFusion and other Arrow-based tools.

**Supported types (zero-copy where R memory layout matches Arrow):**
- Scalar arrays: `Float64Array`, `Int32Array`, `BooleanArray`, `StringArray`, etc.
- `RecordBatch` -- round-trips R data.frames column by column

See [ARROW.md](ARROW.md) for the full guide.

### `datafusion`

SQL query engine on R data frames via Apache DataFusion. Depends on `arrow`.
Provides `RSessionContext` for running SQL queries on R data frames.

Uses Tokio internally (`current_thread` / `block_on`) -- NOT `rt-multi-thread`.

**Provides:**
- `RSessionContext` -- register R data frames as tables, execute SQL
- `execute_sql()`, `collect_to_r()` helpers

---

## Logging Features

### `log`

Routes Rust `log` macros (`info!`, `warn!`, `error!`, `debug!`, `trace!`) to R's
console output (`Rprintf`/`REprintf`). The logger is installed automatically during
`package_init()` -- no setup needed beyond enabling the feature.

```rust
use log::{info, warn};

#[miniextendr]
pub fn process_data(n: i32) -> i32 {
    info!("Processing {} rows", n);
    if n > 10000 {
        warn!("Large input -- this may be slow");
    }
    n * 2
}
```

---

## Diagnostic Features

### `macro-coverage`

Enables the `macro_coverage` module used for `cargo expand` auditing. This is a
development/testing feature for verifying macro expansion coverage across all
supported attribute combinations.

### `growth-debug`

Enables the `track_growth!` and `report_growth!` instrumentation used to find
unexpected collection reallocations. Both macros are no-ops when the feature is
disabled.

### Aggregate features (maintainer use)

The crate exposes four aggregates for CI and local maintenance:

| Feature | Contents |
|---|---|
| `full-integrations` | Every optional dependency integration |
| `full-codegen` | Runtime/codegen selectors using the `r6-default` side of the class-system mutex |
| `full-codegen-s7` | The same selectors using `s7-default` instead |
| `full` | `full-integrations` + `full-codegen` + diagnostic features |

Package authors should enable the individual features they use. `full` is a
local-check convenience, not a deployment recommendation, and cannot include
both `r6-default` and `s7-default`.

---

## Project-Wide Default Features

These features set project-wide defaults for `#[miniextendr]` options, so you don't
need to annotate every function. Individual items can opt out with `no_` prefixed keywords.

See [FEATURE_DEFAULTS.md](FEATURE_DEFAULTS.md) for the full guide with examples.

| Feature | Effect | Opt-out |
|---------|--------|---------|
| `strict-default` | Strict checked conversions for lossy types | `no_strict` |
| `coerce-default` | Auto-coerce parameters | `no_coerce` |
| `fast-default` | Remove R-side wrapper validation and prefer unchecked fast paths | `no_fast` |
| `r6-default` | R6 class system for impl blocks | `env`, `s7`, etc. |
| `s7-default` | S7 class system for impl blocks | `env`, `r6`, etc. |
| `worker-default` | Force worker thread execution (implies `worker-thread`) | `no_worker` |

**Note:** The tagged-condition transport (panics, `Err`, `None` → tagged SEXP → R wrapper
raises a structured `rust_*` condition) and main thread execution are **hardcoded
defaults** with no opt-out. The `unwrap_in_r` attribute is orthogonal (Result-as-value
vs Result-as-error-boundary). Opt into the worker thread per-function with `worker`.

**Mutual exclusivity:** `r6-default`/`s7-default` cannot be enabled simultaneously.

---

## Usage

Enable features in your `Cargo.toml`:

```toml
[dependencies]
miniextendr-api = { version = "0.1", features = ["rayon", "serde", "ndarray"] }
```

To disable default features:

```toml
[dependencies]
miniextendr-api = { version = "0.1", default-features = false, features = ["rayon"] }
```

### Prelude and crate re-exports

The prelude (`use miniextendr_api::prelude::*`) re-exports both the miniextendr
adapter types **and** the upstream dependency crates for every enabled feature.
You do **not** need to add optional crates as direct dependencies:

```toml
[dependencies]
# This is enough - no need for uuid = "1" or ndarray = "0.17"
miniextendr-api = { version = "0.1", features = ["uuid", "ndarray"] }
```

```rust
use miniextendr_api::prelude::*;

// Uuid type is available directly
let id = Uuid::new_v4();

// Full upstream crate is also accessible for advanced usage
let parsed = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
```

Feature implications (automatically enabled):

| Feature | Also enables |
|---------|-------------|
| `serde_json` | `serde` |
| `rand_distr` | `rand` |
| `indicatif` | `connections`, `nonapi` |
| `datafusion` | `arrow` |
| `worker-default` | `worker-thread` |

---

## Known Limitations

- **`connections` is experimental.** R reserves the right to change the connection ABI without backward compatibility. Always check `R_CONNECTIONS_VERSION`. See [GAPS.md](GAPS.md#41-r-connections-api-experimental).
- **Feature-gated modules** require path-based module switching with `#[cfg]` on `mod` declarations. See [GAPS.md](GAPS.md#13-feature-gated-module-entries).

See [GAPS.md](GAPS.md) for the full catalog of known limitations.

---

## See Also

- [TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md) -- How feature-gated types convert to/from R
- [FEATURE_DEFAULTS.md](FEATURE_DEFAULTS.md) -- Project-wide defaults via Cargo features
- [THREADS.md](THREADS.md) -- Worker-thread routing and optional non-API stack controls
