# Encoding and Locale

This document covers miniextendr's UTF-8 locale requirement and encoding
probing utilities.

**Source:** `miniextendr-api/src/encoding.rs`

## UTF-8 Locale Assertion

miniextendr requires a UTF-8 locale. The `miniextendr_assert_utf8_locale()`
function is called during package initialization (`R_init_*`) and terminates
with an R error if the session is not UTF-8.

This makes the meaning of native/unmarked CHARSXPs unambiguous. A UTF-8 process
locale does **not** make every CHARSXP UTF-8: R can still carry explicitly
tagged Latin-1 and `bytes` strings. The conversion layer therefore handles the
per-string tag as well:

- UTF-8, ASCII, and native strings in a UTF-8 locale use the zero-copy
  `R_CHAR` + `LENGTH` path.
- Latin-1 and other translatable text is converted with
  `Rf_translateCharUTF8`.
- `bytes` strings are rejected because arbitrary bytes cannot be represented
  by Rust's UTF-8 `str` type.

R >= 4.2.0 uses UTF-8 by default on all mainstream platforms, so the locale
gate only fails on old or misconfigured installations.

### How It Works

1. Calls `l10n_info()` (public R API) during `R_init_*`
2. Reads the `"UTF-8"` element from the result
3. If `FALSE`, raises an R error:
   `"miniextendr requires a UTF-8 locale (R >= 4.2.0 uses UTF-8 by default)"`

### Initialization Integration

The assertion is called automatically by `package_init()` (via `miniextendr_init!`):

```rust
// lib.rs - the macro handles UTF-8 assertion automatically
miniextendr_api::miniextendr_init!(mypkg);
```

No user action is required. `miniextendr_init!` includes the UTF-8 locale
check as part of the standard initialization sequence.

## Encoding Info (Non-API, Embedding Only)

The `miniextendr_encoding_init()` function snapshots R's internal encoding
state into a static `REncodingInfo` struct. This is **only available when
embedding R** (via `miniextendr-engine`), not in R packages.

### Why Not in R Packages

`miniextendr_encoding_init()` reads non-API symbols from R's `Defn.h`
(`utf8locale`, `mbcslocale`, `known_to_be_latin1`). These symbols are not a
supported package API, so this snapshot is reserved for embedding builds.

### REncodingInfo

When the `nonapi` feature is enabled and `miniextendr_encoding_init()` has run:

```rust
use miniextendr_api::encoding;

if let Some(info) = encoding::encoding_info() {
    println!("UTF-8 locale: {:?}", info.utf8_locale);
    println!("multibyte locale: {:?}", info.mbcs_locale);
    println!("unknown strings are Latin-1: {:?}", info.known_to_be_latin1);
}
```

| Field | Type | Description |
|-------|------|-------------|
| `utf8_locale` | `Option<bool>` | Whether R considers the locale UTF-8 |
| `mbcs_locale` | `Option<bool>` | Whether R considers the locale multibyte |
| `known_to_be_latin1` | `Option<bool>` | Whether R treats unknown strings as Latin-1 |

All fields require the `nonapi` feature. Without it, `REncodingInfo` is an
empty struct and `encoding_info()` returns `Some(&REncodingInfo {})` after init.

### Debug Output

Set `MINIEXTENDR_ENCODING_DEBUG=1` to print the encoding snapshot at init time:

```bash
MINIEXTENDR_ENCODING_DEBUG=1 R -e 'library(miniextendr)'
# [miniextendr] encoding init: REncodingInfo { utf8_locale: Some(true), ... }
```

This is only useful when embedding R or on platforms where the non-API symbols
happen to be exported.

## R's Encoding Model

For background, R has two layers of encoding:

1. **Per-CHARSXP tags** -- each R string carries an encoding mark (UTF-8,
   Latin-1, bytes, or native). Functions like `Rf_mkCharCE` and
   `Rf_translateCharUTF8` work with these tags.

2. **Global locale state** -- R tracks whether the session locale is UTF-8 or
   Latin-1. This affects how "native" strings are interpreted.

miniextendr requires a UTF-8 locale for native/unmarked strings and still
honors each explicit CHARSXP encoding tag. This keeps the common path
zero-copy without treating the process locale as a property of every string.
