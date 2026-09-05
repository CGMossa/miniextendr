# Visibility and Export Control

`#[miniextendr]` functions exist at three scopes: the Rust binary (C symbol), the package
namespace (callable inside the package), and the public API (importable by other packages).
Rust `pub` and the `#[miniextendr]` export attributes control which scope each function
occupies. This document explains the mapping.

---

## Three-Tier Model

Every `#[miniextendr]` function always gets a C symbol registered in `R_CallMethodDef`.
What varies is whether the generated R wrapper carries `@export`, `@keywords internal`,
or neither.

| Rust visibility | `#[miniextendr]` option | `@export` | `@keywords internal` | NAMESPACE |
|-----------------|------------------------|-----------|----------------------|-----------|
| `pub fn` | (none) | yes | no | exported |
| `pub fn` | `noexport` | no | no | not in NAMESPACE |
| `pub fn` | `internal` | no | yes | not in NAMESPACE |
| non-`pub fn` | (none) | no | no | not in NAMESPACE |
| non-`pub fn` | `export` | yes | no | exported |

Non-`pub` functions behave identically to `pub` + `noexport`: an R wrapper is generated and
the C symbol is registered, but no `@export` is emitted.

---

## The C Symbol Is Always Registered

Every `#[miniextendr]` function — regardless of Rust visibility or export flags —
produces a `R_CallMethodDef` entry and an R wrapper. This means
`.Call(C_<crate>_fn_name, ...)` works from any R code inside the package, even for
non-exported functions. (C symbols are prefixed with the crate's name — `mypkg`
in the examples below — for webR cross-package uniqueness, see `docs/WEBR.md`.)

NAMESPACE controls importability from _outside_ the package. It has no effect on
`.Call()` visibility within the package itself.

```rust
// C_mypkg_internal_helper is callable via .Call() from R code in the same package,
// but does not appear in NAMESPACE and cannot be imported by downstream packages.
fn internal_helper(x: i32) -> i32 {
    x + 1
}
```

---

## Function Attribute Reference

| Attribute | Effect on R wrapper |
|-----------|---------------------|
| `noexport` | Omit `@export`; wrapper exists and C symbol is registered |
| `internal` | Omit `@export`; add `@keywords internal`; appears in `?help` only when searched directly |
| `export` | Force `@export` on a non-`pub` function |
| `r_name = "..."` | Rename the R wrapper (e.g. `r_name = "is.widget"`); does not affect NAMESPACE membership |
| `postfix = "..."` | Append a suffix to the Rust name for the R wrapper (`postfix = "_impl"` on `fn f` gives `f_impl`); states the "hand-written `f()` delegates to generated `f_impl()`" convention once. Exclusive with `r_name` and `s3(...)` |
| `call = caller` | Attribute conditions to the wrapper's caller (the hand-written R function delegating to this internal entry point) instead of the wrapper's own call. Requires `noexport` or `internal` |
| `c_symbol = "..."` | Rename the C symbol used in `.Call()` and `R_CallMethodDef`. The value is used verbatim — no crate prefix is added, so **you** own its cross-package uniqueness on webR (see `docs/WEBR.md`) |

### When to use each option

| Goal | Use |
|------|-----|
| Public API function | `pub fn` (default) |
| `pub` for Rust trait bounds, but package-internal | `#[miniextendr(noexport)]` |
| Internal helper that should appear in `?help` search | `#[miniextendr(internal)]` |
| Non-`pub` function that must be exported | `#[miniextendr(export)]` (rare; prefer making the fn `pub`) |
| Rename the R-facing name | `r_name = "my.function"` |
| Internal entry point behind a hand-written R function | `#[miniextendr(noexport, postfix = "_impl")]` |
| Internal entry point whose errors should name the public R function | `#[miniextendr(noexport, call = caller)]` |
| Rename the C symbol (e.g. to avoid collision) | `c_symbol = "pkg_my_fn"` |

### Mutually exclusive combinations

The following combinations are compile errors:

- `internal + noexport` — `internal` is a strict superset; drop `noexport`
- `export + noexport` — contradictory
- `export + internal` — contradictory
- `postfix + r_name` — both set the R wrapper name; use one
- `postfix + s3(...)` — S3 method names are always `generic.class`
- `call = caller` without `noexport` / `internal` — caller attribution is for internal entry points
- `call = caller + no_call_attribution` (or `fast`) — `.call = NULL` leaves no slot to redirect

---

## Examples

### Default: exported public function

```rust
#[miniextendr]
pub fn add(x: i32, y: i32) -> i32 {
    x + y
}
```

```r
# Generated:
#' @export
add <- function(x, y) .Call(C_mypkg_add, x, y)
```

`add` appears in `NAMESPACE` and is importable by downstream packages.

### noexport: suppress NAMESPACE entry

```rust
// pub needed so this type's method can call it via a trait bound,
// but we don't want it in the public API.
#[miniextendr(noexport)]
pub fn validate_internal(x: i32) -> bool {
    x > 0
}
```

No `@export` is emitted. The R wrapper exists and `.Call(C_mypkg_validate_internal, ...)` works
inside the package, but `validate_internal` is not in NAMESPACE.

### internal: hide from normal help search

```rust
#[miniextendr(internal)]
pub fn debug_repr(x: i32) -> String {
    format!("debug: {}", x)
}
```

```r
# Generated:
#' @keywords internal
debug_repr <- function(x) .Call(C_mypkg_debug_repr, x)
```

`@keywords internal` hides the function from `help.search()` results by default. It remains
accessible to R users who know the name (`?debug_repr` still works) but does not clutter
discovery.

### export: force-export a non-pub function

```rust
// Rare — prefer making the function pub instead.
#[miniextendr(export)]
fn legacy_compat() -> i32 {
    42
}
```

`@export` is emitted despite the function not being `pub` in Rust.

### r_name: rename the R wrapper

```rust
#[miniextendr(r_name = "is.widget")]
pub fn is_widget(x: i32) -> bool {
    x == 1
}
```

The R wrapper is named `is.widget` (valid R identifier style). The C symbol remains
derived from the Rust identifier (`C_mypkg_is_widget`). NAMESPACE gets `export(is.widget)`.

### postfix: state the internal-entry-point convention once

A common layout keeps the documented R function hand-written (argument checking,
defaults, `match.arg()`) and has it delegate to the generated wrapper. With
`r_name` every such entry point spells its own name again in a string; `postfix`
derives it from the Rust name:

```rust
#[miniextendr(noexport, postfix = "_impl")]
pub fn summarise_widget(x: i32) -> i32 {
    x * 2
}
```

```r
# Generated (not exported):
summarise_widget_impl <- function(x) .Call(C_mypkg_summarise_widget, x)

# Hand-written, in R/:
#' @export
summarise_widget <- function(x) {
  stopifnot(is.numeric(x), length(x) == 1L)
  summarise_widget_impl(as.integer(x))
}
```

The value is appended verbatim to the Rust identifier and must be a valid R
identifier fragment (letters, digits, `_`, `.`). The C symbol is unchanged.
Inherent impl methods accept it too (`obj$bump_impl()` on R6/Env,
`bump_impl.Widget` on S3, the generic name on S4/S7); trait impls take their R
names from the trait declaration and do not. There is no crate-level default:
proc-macro invocations share no state, so the attribute is per item (#1454
sketches a `[package.metadata]` alternative).

### call = caller: attribute conditions to the hand-written caller

```rust
#[miniextendr(noexport, call = caller)]
pub fn summarise_impl(x: i32) -> Result<i32, String> {
    if x <= 0 { return Err(format!("x must be positive, got {x}")); }
    Ok(x)
}
```

```r
# Hand-written, in R/:
#' @export
summarise <- function(value) summarise_impl(as.integer(value))

summarise(-1)
#> Error in summarise(value = -1) : x must be positive, got -1
```

Without the option the error would read `Error in summarise_impl(x = as.integer(value))`,
leaking the bridge that `noexport` exists to hide. See
[CALL_ATTRIBUTION.md](CALL_ATTRIBUTION.md#internal-entry-points-caller-attribution) for
how the frame is chosen and the top-level fallback.

---

## Impl and Class-Level Export Control

The `noexport` and `internal` flags can be applied to an entire `impl` block, suppressing
or marking all methods at once.

```rust
// All methods in this impl are internal
#[miniextendr(internal)]
impl DebugType {
    pub fn dump(&self) -> String { ... }
    pub fn inspect(&self) -> i32 { ... }
}
```

```rust
// Suppress @export on the whole class
#[miniextendr(noexport)]
impl InternalHelper {
    pub fn run(&self) -> i32 { ... }
}
```

Individual methods can override the impl-level flag:

```rust
#[miniextendr(noexport)]
impl MyType {
    pub fn internal_method(&self) -> i32 { ... }

    // Override: this one is exported despite the block-level noexport
    #[miniextendr(export)]
    pub fn public_method(&self) -> String { ... }
}
```

---

## Lint Rules

### MXL106: non-`pub` function not exported

```
warning[MXL106]: registered function `my_helper` is not `pub`
```

This fires when a `#[miniextendr]` function is not `pub`. The C symbol is registered
and the R wrapper exists, but users cannot call it from outside the package. Fix: add
`pub`, or add `#[miniextendr(export)]` if you intentionally want to export a non-`pub` fn.

### MXL203: `internal + noexport` redundancy

```
warning[MXL203]: `internal` and `noexport` are redundant together
```

`internal` already suppresses `@export` _and_ adds `@keywords internal`. Drop `noexport`.

```rust
// Bad
#[miniextendr(internal, noexport)]
pub fn helper() -> i32 { 42 }

// Good
#[miniextendr(internal)]
pub fn helper() -> i32 { 42 }
```

---

## Quick Decision Guide

```
Is the function part of the package's public API?
├── Yes → pub fn (default)
├── No — should it appear in help search?
│   ├── Yes, but not exported → pub fn + #[miniextendr(internal)]
│   └── No → non-pub fn  (or pub + #[miniextendr(noexport)])
└── Need pub for Rust trait bounds only? → pub fn + #[miniextendr(noexport)]
```

---

## See Also

- [MINIEXTENDR_ATTRIBUTE.md](MINIEXTENDR_ATTRIBUTE.md) — complete `#[miniextendr]` option reference
- [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md) — class-level export control details
- [MACRO_ERRORS.md](MACRO_ERRORS.md) — MXL106, MXL203, and other lint details
