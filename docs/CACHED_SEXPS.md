# Cached SEXPs

R strings (CHARSXPs), symbols, and class vectors are immutable once created.
Cache values that are needed repeatedly, especially on hot paths like
vectorized conversions, and reuse the pointer.

## Macros

Two declarative macros in `cached_class.rs` handle all the boilerplate.
Adding a new cached value is a one-liner:

```rust
use crate::cached_class::{cached_symbol, cached_strsxp};

// Cache a symbol (Rf_install result):
cached_symbol!(pub(crate) fn tzone_symbol() = c"tzone");

// Cache a single-element class vector:
cached_strsxp!(pub(crate) fn date_class_sexp() = [c"Date"]);

// Cache a multi-element class vector:
cached_strsxp!(pub(crate) fn posixct_class_sexp() = [c"POSIXct", c"POSIXt"]);

// Cache a names vector:
cached_strsxp!(
    pub(crate) fn condition_names_sexp() = [c"error", c"kind", c"class", c"call", c"data"]
);

// With feature gates:
cached_symbol!(
    #[cfg(feature = "vctrs")]
    pub(crate) fn ptype_symbol() = c"ptype"
);
```

Each macro expands to a function with a `static OnceLock<SEXP>` inside.
First call initializes; subsequent calls are a single atomic load.

### `cached_symbol!`

Caches the result of `Rf_install`. Symbols are never GC'd, so no
`R_PreserveObject` is needed.

### `cached_strsxp!`

Allocates a STRSXP, fills it with permanent CHARSXPs (via `Rf_install` +
`PRINTNAME`), and preserves it with `R_PreserveObject`. Works for class
vectors, names vectors, and scalar strings: anything that's a fixed STRSXP.

## How it works

### Permanent CHARSXPs via symbols

R's symbol table is never garbage-collected. A symbol's `PRINTNAME` is a
CHARSXP that lives as long as the R session:

```rust
/// Symbol → CHARSXP (never GC'd).
unsafe fn permanent_charsxp(name: &std::ffi::CStr) -> SEXP {
    unsafe { PRINTNAME(Rf_install(name.as_ptr())) }
}
```

This is cheaper than `Rf_mkCharLenCE` on repeated calls because it skips
the global string hash lookup after the first call (the `OnceLock` caches
the result).

### Manual pattern (for reference)

The macros expand to this pattern:

```rust
use std::sync::OnceLock;

pub(crate) fn my_class_sexp() -> SEXP {
    static CACHE: OnceLock<SEXP> = OnceLock::new();
    *CACHE.get_or_init(|| unsafe {
        let class = Rf_allocVector(SEXPTYPE::STRSXP, 1);
        R_PreserveObject(class);
        class.set_string_elt(0, permanent_charsxp(c"my_class"));
        class
    })
}
```

Use the macros instead of writing this by hand.

## When to cache

**Do cache:**

- Class vectors set on every conversion (`c("POSIXct", "POSIXt")`,
  `"data.frame"`, `"Date"`, `"factor"`)
- Attribute symbols used per-element or per-vector (`tzone`, `mx_raw_type`,
  `ptype`, `size`)
- Names vectors with fixed structure (`c("error", "kind", "class", "call", "data")`)

**Don't cache:**

- Dynamic strings (user-provided column names, error messages with variable
  content)
- One-shot setup code (connection version checks, package init)
- Strings only used behind a cold `if` branch

## Where caches live

All cached SEXPs are in `miniextendr-api/src/cached_class.rs`. Feature-gated
items use the narrowest `#[cfg]` that covers their callers:

| Cached value | Feature gate |
|---|---|
| `data_frame_class_sexp()` | (none - always available) |
| `rust_condition_class_sexp()` | (none) |
| `condition_names_sexp()` | (none) |
| `rust_condition_attr_symbol()` | (none) |
| `posixct_class_sexp()` | `any(time, jiff, arrow)` |
| `date_class_sexp()` | `any(time, jiff, arrow)` |
| `tzone_symbol()` | `any(time, jiff, arrow)` |
| `set_posixct_utc()` | `any(time, jiff)` |
| `utc_tzone_sexp()` | `any(time, jiff)` |
| `set_posixct_tz()` | `jiff` |
| `mx_raw_type_symbol()` | `raw_conversions` |
| `ptype_symbol()` | `vctrs` |
| `size_symbol()` | `vctrs` |
| `factor_class_sexp()` | (none - in `factor.rs`) |

## Safety

- `R_PreserveObject` prevents GC for the lifetime of the R session.
  `OnceLock` ensures single initialization.
- The cached STRSXP is shared across all callers. This is safe because
  `set_class` / `Rf_setAttrib` attach it as an attribute - they don't
  mutate the value itself.
- Symbols (`Rf_install`) are never GC'd - no `R_PreserveObject` needed
  for the symbol itself, but the accessor caches it to avoid repeated
  hash lookups.
