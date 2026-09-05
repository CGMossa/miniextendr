# Proc-Macro and Lint Error Guide

This guide covers common error messages from `#[miniextendr]` proc macros and the `miniextendr-lint` static analysis tool.

## Running the Lint

```bash
just lint               # Run lint on rpkg
```

The lint also runs automatically during `cargo build`/`cargo check` via `build.rs`. To disable temporarily:

```bash
MINIEXTENDR_LINT=0 cargo check --manifest-path=rpkg/src/rust/Cargo.toml
```

## Lint Codes Reference

### Errors (CI-blocking)

| Code | Description | Fix |
|------|-------------|-----|
| **MXL001** | Reserved | (Legacy lint, no longer applicable) |
| **MXL002** | Reserved | (Legacy lint, no longer applicable) |
| **MXL003** | Reserved | (Legacy lint, no longer applicable) |
| **MXL004** | Reserved | (Legacy lint, no longer applicable) |
| **MXL005** | Reserved | (Legacy lint, no longer applicable) |
| **MXL006** | Reserved | (Legacy lint, no longer applicable) |
| **MXL007** | `impl Type;` requires ExternalPtr derive | Add `#[derive(ExternalPtr)]` or implement `TypedExternal` |
| **MXL008** | Trait impl class system incompatible | Ensure trait impl uses same class system as inherent impl |
| **MXL009** | Multiple impl blocks without labels | Add `#[miniextendr(label = "unique")]` to each impl block |
| **MXL010** | Duplicate labels on impl blocks | Use unique labels for each impl block |

### Warnings (P0: high impact)

| Code | Description | Fix |
|------|-------------|-----|
| **MXL100** | Duplicate entrypoint symbol | Rename one of the conflicting entries |
| **MXL101** | Duplicate registration entries | Remove the duplicate entry |
| **MXL102** | Trait impl missing TypedExternal | Implement `TypedExternal` for the type |
| **MXL103** | Generic concrete type in trait-ABI | Use concrete (non-generic) types for trait ABI |
| **MXL104** | `#[cfg]` mismatch between item and registration | Ensure `#[cfg]` attributes match on both |
| **MXL105** | Unreachable module file | Check file paths and module hierarchy |
| **MXL106** | Registered function is not `pub` | Add `pub` to the function definition |
| **MXL107** | Missing `#[miniextendr] impl Trait for Type` | Add the attribute to the trait impl |
| **MXL108** | Missing registration for trait impl | Add `#[miniextendr]` to the trait impl |
| **MXL109** | `#[cfg]` mismatch between `mod` declarations | Ensure `#[cfg]` attributes are consistent |
| **MXL110** | Parameter name is an R reserved word | Rename the parameter — generated R wrapper would fail to parse |
| **MXL111** | `s4_*` method on `#[miniextendr(s4)]` impl | Drop the `s4_` prefix — codegen auto-prepends it (you'd get `s4_s4_*`) |
| **MXL112** | Explicit lifetime parameter on `#[miniextendr]` fn or impl | Use owned types (`Vec<T>` / `String`) — `&[T]` and `&str` arguments work without explicit lifetime annotations |

### Warnings (P1: important)

| Code | Description | Fix |
|------|-------------|-----|
| **MXL200** | Trait tag collision preflight | Use unique trait names or labels |
| **MXL201** | Impl label mismatch | Check label spelling matches across impls |
| **MXL202** | Orphan child module reference | Remove the reference to non-existent child module |
| **MXL203** | `internal` + `noexport` redundancy | Use just `#[miniextendr(internal)]` (implies noexport) |
| **MXL204** | Multiple root-level registrations | Only one root registration per crate |

### Warnings (P2: safety)

| Code | Description | Fix |
|------|-------------|-----|
| **MXL300** | Direct `Rf_error`/`Rf_errorcall` call | Use `panic!()` or return `Err(...)` instead |
| **MXL301** | `_unchecked` FFI call outside guard context | Use the checked wrapper, or ensure you're inside `with_r_unwind_protect` |

#### MXL300: Direct Rf_error calls

`Rf_error()` and `Rf_errorcall()` perform a C `longjmp` that skips Rust destructors. This leaks memory and can corrupt state. The lint detects these calls via text scanning.

**Preferred alternatives:**

- `panic!("message")`: caught by miniextendr's unwind protection, produces a structured R condition
- `return Err(...)`: for `Result<T, E>` return types, produces a clean R error

**When Rf_error is intentional** (e.g., inside `with_r_unwind_protect` closures or test fixtures), suppress with `// mxl::allow(MXL300)`. See [Inline Suppression](#inline-suppression) below.

#### MXL301: Unchecked FFI calls

Functions like `Rf_ScalarInteger_unchecked()` bypass miniextendr's main-thread routing check. They are only safe when you are **certain** you're on R's main thread (inside ALTREP callbacks, `with_r_unwind_protect` closures, `extern "C-unwind"` functions called by R, etc.).

**Preferred:** Use the checked wrapper (without `_unchecked` suffix). It adds a debug-mode thread assertion.

**When unchecked is intentional**, suppress with `// mxl::allow(MXL301)`.

## Inline Suppression

Both MXL300 and MXL301 support inline suppression via `// mxl::allow(...)` comments. The suppression comment can appear:

1. **On the same line** (trailing comment):

```rust
Rf_error(c"%s".as_ptr(), c"intentional".as_ptr()) // mxl::allow(MXL300)
```

2. **On the immediately preceding line** (standalone comment):

```rust
// mxl::allow(MXL300)
Rf_error(c"%s".as_ptr(), c"intentional".as_ptr())
```

Multiple codes can be suppressed in one comment:

```rust
// mxl::allow(MXL300, MXL301)
Rf_error_unchecked(c"test".as_ptr())
```

**Important:** The comment must be on the exact same line or the line directly above. A comment two lines above will not suppress the diagnostic.

```rust
// mxl::allow(MXL300)   <-- too far away (2 lines above)
unsafe {
    Rf_error(...)        <-- NOT suppressed
}
```

Move the comment inside the block:

```rust
unsafe {
    // mxl::allow(MXL300)
    Rf_error(...)        <-- suppressed
}
```

### Suppression syntax

```rust
// mxl::allow(CODE)
// mxl::allow(CODE1, CODE2)
// mxl::allow(CODE1, CODE2, CODE3)
```

- Prefix: `// mxl::allow(`
- Codes: comma-separated, whitespace around commas is ignored
- Only `MXL300` and `MXL301` are currently suppressible

## Common Proc-Macro Errors

### "dots must be the last parameter"

The `...` (dots) argument must appear last in the function signature:

```rust
// Wrong
#[miniextendr]
fn bad(dots: &Dots, x: i32) -> i32 { x }

// Correct
#[miniextendr]
fn good(x: i32, dots: &Dots) -> i32 { x }
```

### "expected `pub` function"

Only `pub` functions get `@export` in R wrappers. If the function is intentionally private, use `#[miniextendr(noexport)]` or `#[miniextendr(internal)]`.

### "multiple impl blocks for Type need labels"

When a type has more than one `#[miniextendr]` impl block, each needs a unique label:

```rust
#[miniextendr(label = "core")]
impl MyType {
    fn method_a(&self) -> i32 { 0 }
}

#[miniextendr(label = "extra")]
impl MyType {
    fn method_b(&self) -> String { String::new() }
}
```

### "trait impl class system incompatible"

A trait impl's class system must match the inherent impl's class system:

```rust
#[miniextendr(s3)]
impl MyType { /* ... */ }

// This will error if the trait impl uses a different class system
#[miniextendr]  // Must inherit s3 from the inherent impl
impl Display for MyType { /* ... */ }
```

### "type must derive ExternalPtr or implement TypedExternal"

Types used in `#[miniextendr]` impl blocks need pointer identity:

```rust
#[derive(ExternalPtr)]
struct MyType { /* ... */ }

#[miniextendr]
impl MyType { /* ... */ }
// Registration is automatic via #[miniextendr].
```

### "takes `self` and is marked `constructor`"

`#[miniextendr(<system>(constructor))]` describes a receiverless `fn new(...) ->
Self`. A consuming builder step needs no marker: `fn step(self, ...) -> Self`
writes its result back into the same R object, `-> Result<Self, E>` /
`Option<Self>` run on a clone and overwrite on success (the type must be
`Clone`). Drop the attribute. See
[CLASS_SYSTEMS.md](CLASS_SYSTEMS.md#consuming-receivers-self).

### "unsupported receiver type"

The R handle stores the value itself, so a method can take `self`,
`self: Self`, `&self`, `&mut self`, or an `ExternalPtr<Self>` receiver.
`self: Box<Self>`, `Rc<Self>`, `Arc<Self>` and friends cannot be handed over;
unwrap to `self` or take `&self`.

### "`postfix` and `r_name` both set the R wrapper name"

`postfix = "_impl"` derives the R name from the Rust identifier; `r_name = "..."`
replaces it. Giving both is contradictory, so it is rejected rather than letting
one silently win. Keep `postfix` when the name should follow the Rust name,
`r_name` when it should not. The method-level variant says "R method name" and
also rejects `postfix` together with `generic = "..."`. See
[VISIBILITY.md](VISIBILITY.md#postfix-state-the-internal-entry-point-convention-once).

### "`postfix` cannot be used with `s3(generic = ..., class = ...)`"

Standalone S3 methods are always named `generic.class`, so there is nothing for
a postfix to append to. Rename through `generic` instead.

### "`call = caller` attributes conditions to the wrapper's caller, which is only meaningful for a package-internal entry point"

`#[miniextendr(call = caller)]` makes the generated wrapper report its caller's
call in conditions. That is right for a `noexport` / `internal` entry point
wrapped by a hand-written R function, and wrong for an exported function whose
caller is arbitrary user code. Add `noexport` or `internal`, or drop the option.
See [CALL_ATTRIBUTION.md](CALL_ATTRIBUTION.md#internal-entry-points-caller-attribution).

### "`call = caller` cannot be combined with `no_call_attribution` / `fast`"

Those options emit `.call = NULL` (no call captured at all), so there is no slot
for `call = caller` to redirect. Keep one: `call = caller` for attributed errors
from an internal entry point, `fast` for the no-attribution fast path.

### "`serde_error` cannot be used with `unwrap_in_r`"

`#[miniextendr(serde_error)]` classes the condition raised from a `Result`'s
`Err` arm. `unwrap_in_r` hands the whole `Result` to R as a value and never
raises, so there is nothing to class. Drop `unwrap_in_r` to raise a classed
error, or drop `serde_error` to return the `Result`. See
[CONDITIONS.md](CONDITIONS.md#deriving-the-classes-from-serde-with-serde_error).

### "`#[miniextendr(serde_error)]` requires a `Result<T, E>` return type"

The attribute only changes the generated `Err` arm. A function or method that
does not return `Result` has no `Err` arm, so the attribute would be a silent
no-op; it is rejected instead. Return `Result<T, E>` with
`E: serde::Serialize + Display`.

## Debugging Tips

1. **Run [`just lint`](https://github.com/A2-ai/miniextendr/blob/main/justfile)** before building: it catches attribute issues earlier than compile errors
2. **Check NAMESPACE**: if a function exists in Rust but not in R, run [`just rcmdinstall && just force-document`](https://github.com/A2-ai/miniextendr/blob/main/justfile) (`force-document` bypasses roxygen2's mtime cache, which can miss macro-layer wrapper changes)
3. **Feature-gated modules**: use `#[cfg]` on `mod` declarations for conditional compilation
