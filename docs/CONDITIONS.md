# Condition system: error!, warning!, message!, condition!

miniextendr provides four macros for raising structured R conditions from Rust.
They ride the tagged-condition transport that every `#[miniextendr]` function
uses.

## Quick reference

| Macro | R equivalent | Default class | Unhandled behaviour |
|---|---|---|---|
| `error!(...)` | `stop()` | `rust_error` | terminates execution |
| `warning!(...)` | `warning()` | `rust_warning` | prints, continues |
| `message!(...)` | `message()` | `rust_message` | prints, continues |
| `condition!(...)` | `signalCondition()` | `rust_condition` | silent no-op |

All four support an optional `class = ...` argument to prepend custom classes
for programmatic catching (one string or a vector, most specific first), and an
optional `data = ...` argument to attach structured named fields readable as
`e$<name>` in handlers.
`Result<T, E>` returns get the same treatment through the
[`RConditionError`](#classed-result-errors-with-rconditionerror-and-rerror) trait,
or, for error enums that already derive `serde::Serialize`, through
[`#[miniextendr(serde_error)]`](#deriving-the-classes-from-a-serde-error-type).

> **Import note.** `error!` and `condition!` are shadowed by the crate-root
> modules `error` / `condition`, so `use miniextendr_api::*;` (or a direct
> `use miniextendr_api::error;`) resolves to the module, not the macro. Use the
> collision-free aliases `rust_error!` / `rust_condition!` (identical
> expansions), or invoke the bare names fully qualified. `warning!` / `message!`
> have no such clash.

## How it works

Each macro calls `std::panic::panic_any(RCondition::...)`. The panic is caught by
`with_r_unwind_protect` before Rust destructors have unwound, which recognises
the `RCondition` payload and converts it to a tagged SEXP (5-element list:
`error`, `kind`, `class`, `call`, `data`). The generated R wrapper reads the
SEXP and dispatches to the appropriate R signal function.

The `class` slot carries the optional user-supplied class. When non-NULL it is
prepended to the standard layered vector. The `data` slot carries the optional
named-list payload; the R helper splices its fields into the condition object
alongside `message` / `call` / `kind`.

## Class layering

```r
class(e)
# error!("...")         → c("rust_error",     "simpleError",     "error",   "condition")
# warning!("...")       → c("rust_warning",   "simpleWarning",   "warning", "condition")
# message!("...")       → c("rust_message",   "simpleMessage",   "message", "condition")
# condition!("...")     → c("rust_condition", "simpleCondition",            "condition")

# With class = "my_err":
class(e)
# error!(class = "my_err", "...") → c("my_err", "rust_error", "simpleError", "error", "condition")

# With a class vector (member first, family second), so handlers can catch either:
# error!(class = ["pkg_error_missing_field", "pkg_error"], "...")
#   → c("pkg_error_missing_field", "pkg_error", "rust_error", "simpleError", "error", "condition")
```

`class =` accepts anything implementing `ConditionClass`: `&str`, `String`,
`[&str; N]`, `Vec<String>`, slices. The same vector form is available on
`warning!` and `condition!`.

## Runnable examples

### `error!()`

```r
library(miniextendr)

# Raised by: error!("something went wrong: {x}")

e <- tryCatch(demo_error("oops"), error = function(e) e)
class(e)
# [1] "rust_error"  "simpleError" "error"       "condition"
conditionMessage(e)
# [1] "oops"

# Specific handler:
tryCatch(demo_error("x"), rust_error = function(e) "caught by rust_error handler")
# [1] "caught by rust_error handler"
```

### `error!()` with custom class

```r
# Raised by: error!(class = "my_error", "missing field: {name}")

tryCatch(
  demo_error_custom_class("my_error", "missing field: x"),
  my_error   = function(e) paste("custom:", conditionMessage(e)),
  rust_error = function(e) paste("rust:",   conditionMessage(e))
)
# [1] "custom: missing field: x"
```

### `error!()` with structured `data` payloads

Rust-side, the macros accept `data = ("name", value)` for a single field or
`data = [("a", v1), ("b", v2)]` for several (rlang `abort(data = list(...))`
style). Argument order is fixed: `class = ...` (optional), then `data = ...`
(optional), then the format message:

```rust
// Single field:
error!(class = "range_error", data = ("value", value), "value {value} out of range");

// Multiple fields:
error!(
    class = "validation_error",
    data = [("value", value), ("code", code), ("label", label), ("fatal", true)],
    "validation failed for {label}"
);
```

R-side, handlers read the fields directly from the condition object:

```r
# Raised by: error!(class = "range_error", data = ("value", value), "value {value} out of range")

e <- tryCatch(demo_error_data_scalar(150L), range_error = function(e) e)
e$value
# [1] 150

# Programmatic recovery — clamp instead of parsing the message:
tryCatch(
  demo_error_data_scalar(150L),
  range_error = function(e) min(max(e$value, 0L), 100L)
)
# [1] 100
```

#### Supported `data` value types

| Rust value | R field type |
|---|---|
| `i32` | `integer(1)` |
| `f64` | `double(1)` |
| `bool` | `logical(1)` |
| `&str` / `String` | `character(1)` |
| `Vec<i32>` | `integer(n)` |
| `Vec<f64>` | `double(n)` |
| `Vec<bool>` | `logical(n)` |
| `Vec<String>` / `Vec<&str>` | `character(n)` |
| `Option<T>` / `Vec<Option<T>>` for the scalar families above | typed `NA` |
| `i64` / `u32` | integer when it fits; otherwise double |
| `RValue::Null` | `NULL` |
| `RValue::Complex`, `RValue::Raw` | complex / raw vector |
| `RValue::List` | recursively nested, optionally named list |

`RValue` is the owned, `Send`, R-native value tree used by the condition
transport. Build it directly for nested or heterogeneous values, or use
`RValue::debug(value)` to attach an eager `Debug` rendering when no native R
mapping is appropriate.

#### Worker-thread note

The payload travels through `panic_any`, which requires `Send` — and the macro
may fire on the worker thread, where a live `SEXP` is illegal to carry. Each
field is therefore converted at the call site into `RValue`, and a multi-field
payload becomes `ConditionData` (`Vec<(String, RValue)>`). The actual R objects
are materialised on R's main thread at the unwind boundary. Consequently,
`data = ...` works identically from worker-thread and main-thread code, but a
live `SEXP` or arbitrary `IntoR` value cannot ride along.

#### Reserved names

`message`, `call` and `kind` are the condition's own slots (the R helper
splices `data` over them with `utils::modifyList`), so they are **rejected**
as field names: at compile time when the name is a literal or a bare
identifier (`data = ("kind", 1)`, `data = { kind = 1 }`), at runtime otherwise
(a plain `rust_error` explaining the clash, instead of the former silent
overwrite).

There is no escape hatch, deliberately: `message` and `call` are what R's
`conditionMessage()` / `conditionCall()` read, and `kind` is the framework's
transport tag, so a field with one of those names is a naming clash to fix
where the payload is built (a different key, `#[serde(rename)]` on a derived
struct, a rename in your `RConditionError::data()` impl). Renaming every field
with a prefix to rescue one would make `e$<name>` access inconsistent across
conditions. When the payload is an error type's own serde shape that other
consumers read, do the renaming at the condition boundary instead:
`#[miniextendr(serde_error)]` takes `skip(..)` / `rename(..)` and drops a
`message` field that merely echoes `Display` (see
[Payload fields named `message`](#payload-fields-named-message)).

```rust
error!(
    class = "pkg_sparse_rule",
    data = { rule = kind, column = column },
    "sparse rule on `{column}`"
);
```

```r
e <- tryCatch(f(), pkg_sparse_rule = function(e) e)
e$rule      # the field
e$kind      # still "error" (the transport tag)
```

### `warning!()`

```r
# Raised by: warning!("x is large: {x}")

# tryCatch absorbs the warning and returns the handler result:
tryCatch(demo_warning("watch out"), rust_warning = function(w) "caught!")
# [1] "caught!"

# withCallingHandlers resumes execution after the handler:
result <- withCallingHandlers(
  {
    demo_warning("note")
    42L
  },
  warning = function(w) {
    cat("saw:", conditionMessage(w), "\n")
    invokeRestart("muffleWarning")
  }
)
# saw: note
result
# [1] 42
```

### `message!()`

```r
# Raised by: message!("step {n} complete")

demo_message("hello")
# hello

suppressMessages(demo_message("silenced"))
# (no output)

# withCallingHandlers — muffleMessage restart stops the default printing:
withCallingHandlers(
  demo_message("intercepted"),
  message = function(m) {
    cat("caught:", conditionMessage(m))
    invokeRestart("muffleMessage")
  }
)
# caught: intercepted
```

### `condition!()`

```r
# Raised by: condition!("step 1 of 10")
# Without a handler, signalCondition returns NULL invisibly.

demo_condition("silent signal")
# NULL

# With a handler:
withCallingHandlers(
  demo_condition("progress event"),
  condition = function(c) cat("progress:", conditionMessage(c), "\n")
)
# progress: progress event
# NULL

# With a custom class:
withCallingHandlers(
  demo_condition_custom_class("my_progress", "step 3"),
  my_progress = function(c) cat("progress:", conditionMessage(c), "\n")
)
# progress: step 3
# NULL
```

## Classed `Result` errors with `RConditionError` and `RError`

A `#[miniextendr]` function or method returning `Result<T, E>` raises `Err(e)`
as an R error. By default that error is a bare `rust_error` whose message is
`format!("{e:?}")` and whose `kind` is `"result_err"`. To give R handlers
something to dispatch on without giving up `?` composition, implement
`miniextendr_api::condition::RConditionError` for the error type:

```rust
use miniextendr_api::condition::{ConditionData, RConditionError};

#[derive(Debug, thiserror::Error)]
pub enum PkgError {
    #[error("field `{field}` is missing")]
    MissingField { field: String },
    #[error("{value} exceeds the maximum {max}")]
    OutOfRange { value: f64, max: f64 },
}

impl RConditionError for PkgError {
    fn message(&self) -> String { self.to_string() }
    fn class(&self) -> Vec<String> {
        let member = match self {
            PkgError::MissingField { .. } => "pkg_error_missing_field",
            PkgError::OutOfRange { .. } => "pkg_error_out_of_range",
        };
        vec![member.into(), "pkg_error".into()]
    }
    fn data(&self) -> Option<ConditionData> {
        Some(match self {
            PkgError::MissingField { field } => vec![("field".into(), field.as_str().into())],
            PkgError::OutOfRange { value, max } =>
                vec![("value".into(), (*value).into()), ("max".into(), (*max).into())],
        })
    }
}

#[miniextendr]
pub fn check(value: f64) -> Result<f64, PkgError> {
    if value > 100.0 { return Err(PkgError::OutOfRange { value, max: 100.0 }); }
    Ok(value)
}
```

```r
e <- tryCatch(check(150), error = function(e) e)
class(e)
# [1] "pkg_error_out_of_range" "pkg_error" "rust_error" "simpleError" "error" "condition"
e$value; e$max
# [1] 150
# [1] 100
tryCatch(check(150), pkg_error = function(e) "family handler")
# [1] "family handler"
```

Detection is by trait, not by attribute: the generated `Err` arm probes
`E: RConditionError` first and falls back to the `Debug` rendering otherwise,
so existing `Result<T, String>` functions behave exactly as before. `kind`
stays `"result_err"`. Field names follow the reserved-name rule above; a
reserved name from `data()` raises a `rust_error` describing the clash.

For one-off cases, `RError` is a ready-made classed value. Any
`std::error::Error` converts into it with `?` or `From` (the message keeps the
`caused by:` chain), and builders add the R-facing parts:

```rust
use miniextendr_api::condition::RError;

#[miniextendr]
pub fn parse_port(s: &str) -> Result<i32, RError> {
    let port: i32 = s
        .parse()
        .map_err(|e| RError::from(e).class(["pkg_bad_port", "pkg_error"]).data("input", s))?;
    Ok(port)
}
```

`RError` implements `Display` (the message) but not `std::error::Error`, which
keeps the blanket `From<E: Error>` coherent; it works with
`#[miniextendr(unwrap_in_r)]` too.

### Deriving the classes from a serde error type

When the error type already derives `serde::Serialize` (for logging, JSON
transport, or a downstream client), the same information can drive the R
condition without an `RConditionError` impl. `#[miniextendr(serde_error)]`
serializes the `Err` value: the enum variant becomes the member class
`<prefix>_<variant>`, the variant's fields become `e$<name>`, and the message
comes from `Display`. Requires the `serde` feature.

```rust
#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EngineError {
    #[error("field `{field}` is missing")]
    MissingField { field: String },
    #[error("{value} exceeds the maximum {max}")]
    OutOfRange { value: f64, max: f64 },
}

#[miniextendr(serde_error)]
pub fn check(value: f64) -> Result<f64, EngineError> {
    if value > 100.0 { return Err(EngineError::OutOfRange { value, max: 100.0 }); }
    Ok(value)
}
```

```r
e <- tryCatch(check(150), error = function(e) e)
class(e)
# [1] "mypkg_error_out_of_range" "mypkg_error" "rust_error" "simpleError" "error" "condition"
e$value; e$max
# [1] 150
# [1] 100
```

- The family class defaults to `<crate>_error`, from the Rust crate name at
  compile time. Override it with `serde_error(prefix = "engine")`.
- Internally tagged enums (`#[serde(tag = "kind")]`) have the tag field
  consumed as the variant, so it never collides with the framework's own
  `e$kind` (`"result_err"`). Name a different tag with
  `serde_error(tag = "type")`. Externally tagged enums (serde's default)
  report the variant name verbatim (`mypkg_error_OutOfRange`; add
  `#[serde(rename_all = "snake_case")]` for snake_case).
- Struct variants contribute their fields. A newtype variant with a struct
  payload contributes that struct's fields; any other newtype or tuple payload
  lands under `e$value`. Unit variants carry classes only. A value that
  serializes without variant information (a plain struct, a string) gets just
  the family class.
- Values follow the `RValue` mapping (`RValue::from_serde`, see
  [SERDE_R.md](SERDE_R.md#owned-values-rvaluefrom_serde)): scalars,
  homogeneous `Vec<T>` as atomic vectors, nested structs as named lists,
  `None` as `NULL`. The reserved names above apply to the payload fields; a
  clash raises a `rust_error` describing it, exactly as for `data()`, with
  the exceptions below.
- `serde_error` needs a `Result` return type and cannot be combined with
  `unwrap_in_r`; both are compile errors.

#### Payload fields named `message`

A wrapped parser or foreign error is usually `Variant { message: String }`,
and renaming that field at the source changes a serialization other consumers
read. Three rules keep the zero-boilerplate path open for that shape:

```rust
#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParserError {
    #[error("line {line}: {message}")]
    Parse { message: String, line: u32 },
    #[error("{message}")]
    Wrapped { message: String },
}

#[miniextendr(serde_error(skip("message")))]
pub fn parse(text: String) -> Result<f64, ParserError> { /* ... */ }

#[miniextendr(serde_error(rename(message = "detail")))]
pub fn parse_keep(text: String) -> Result<f64, ParserError> { /* ... */ }
```

- `serde_error(skip("message", ...))` drops the named payload fields before
  the reserved-name check. `Parse` reaches R as `mypkg_error_parse` with
  `e$line` and the `Display` text as `conditionMessage(e)`; the wrapped text
  survives only where `Display` includes it.
- `serde_error(rename(message = "detail", ...))` splices the field under the
  new name (`e$detail`). The source may be an identifier or, for a serde-renamed
  field whose name is not one, a string literal: `rename("kebab-name" = "kebab")`.
  A target may not be `message`, `call` or `kind`; that is a compile error.
- With no option, a `message` field whose value is exactly the `Display` text
  is dropped as redundant with the condition's own `message` slot. `Wrapped`
  above therefore needs nothing; `Parse` (whose `Display` adds the line) still
  raises the reserved-name error unless skipped or renamed.

Both options name the field as it serializes and are a no-op for variants
that lack it, so one attribute covers an enum whose variants differ. The macro
cannot see the error type's fields at expansion time; only the option grammar
is checked there. A field named `call` or `kind` has no equality rule and
must be skipped or renamed.
- Choose `RConditionError` when the message or class vector needs to differ
  from the serde shape; `serde_error` is the zero-boilerplate path for enums
  that are already serde-tagged.

## Trait-ABI and ALTREP error class layering

Cross-package trait method panics and ALTREP `r_unwind` callback panics
**do** receive `rust_*` class layering, even though there is no R wrapper
to inspect a tagged SEXP. Two different mechanisms cover the two contexts:

- **Trait-ABI shims**: the vtable shim returns a tagged SEXP on panic; the
  generated View method wrapper inspects the result and re-panics with the
  reconstructed [`RCondition`]. The consumer's outer `with_r_unwind_protect`
  guard (every `#[miniextendr]` fn has one) catches the re-panic and produces
  the tagged SEXP for the consumer's R wrapper. End-to-end behavior is identical
  to a same-package call: `tryCatch(rust_error = h, ...)` matches; user
  classes from `error!(class = "...", ...)` match before `rust_error`. Structured
  `data =` fields (tagged SEXP slot [4]) are also preserved across the re-panic
  boundary — `e$field_name` is accessible in R handlers even when the error
  crossed a package boundary (see #996 path-1).

- **ALTREP `r_unwind` callbacks**: the guard raises the R condition by
  evaluating `stop(structure(list(message, call, ...), class = c(...)))`
  directly (no R wrapper required). `tryCatch(rust_error = h, ...)` matches;
  user classes match before `rust_error`. Structured `data =` fields are
  spliced into the condition list after `message`/`call`, so `e$field_name`
  works from ALTREP-raised errors too (see #996 path-2).

### Remaining limitations

Two narrow cases still degrade:

- `warning!()` / `message!()` / `condition!()` from an ALTREP `r_unwind`
  callback. There is no mechanism to suspend execution to deliver a
  non-fatal signal from inside R's vector-dispatch machinery. These produce
  an R error with the message: *"warning!/message!/condition! from ALTREP
  callback context cannot be raised as non-fatal signals; use error!()
  instead"*.

- A trait View method (`view.method()`) called from Rust code that is not
  wrapped in `with_r_unwind_protect` (e.g., a manual call from a test harness
  or init callback). The re-panic from the View has no outer guard to catch it,
  so the worker thread's `catch_unwind` boundary converts it to an R error
  without `rust_*` class layering. In practice, every `#[miniextendr]` fn
  already provides the outer guard, so this only affects unusual call sites.

Functions that explicitly opt out via `#[miniextendr(unwrap_in_r)]` deliver
`Result<T, E>` to R as a list with an `$error` slot rather than treating `Err`
as a Rust-origin failure — `Err` never traverses the condition pipeline.

## `AsRError` — wrapping `std::error::Error`

For functions that return `Result<T, E>` where `E: std::error::Error`,
`AsRError<E>` wraps the error and formats its full cause chain into the
message:

```rust
use miniextendr_api::condition::AsRError;
use miniextendr_api::miniextendr;

#[miniextendr]
fn parse_number(s: &str) -> Result<i32, AsRError<std::num::ParseIntError>> {
    s.parse::<i32>().map_err(AsRError)
}
```

```r
tryCatch(parse_number("abc"), error = function(e) e$message)
# [1] "invalid digit found in string"
```

For errors with a source chain, all causes appear in the message separated by
`\n  caused by: ...`.
