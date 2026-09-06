---
name: miniextendr-altrep
description: "Use when the user asks about implementing ALTREP (Alternative Representation) vectors in miniextendr, including derive macros (#[derive(AltrepInteger)], #[derive(AltrepReal)], etc.), guard modes (unsafe/rust_unwind/r_unwind), the manual derive path, AltrepExtract trait, no_lowlevel opt-out, option validation rules (subset/dataptr/serialize), the single-struct pattern, why #[miniextendr] on 1-field structs is removed, or compute-on-access iterator patterns."
---

# miniextendr ALTREP

ALTREP (Alternative Representation) lets R vectors be backed by arbitrary Rust data structures without materializing the full vector unless required. miniextendr generates all the R-side registration and C trampoline boilerplate from a derive macro on a single struct.

## When to use this skill

- "How do I create a lazy/compute-on-access R vector from Rust?"
- "Which ALTREP derive macro should I use?"
- "What is the difference between `rust_unwind` and `r_unwind` guard modes?"
- "How do I use `#[altrep(manual)]`?"
- "Why does `#[altrep(subset)]` fail for my type?"
- "What is `AltrepExtract` and do I need to implement it?"
- "How do I implement a string ALTREP with NA support?"
- "Why is `#[miniextendr]` on a 1-field struct rejected?"
- "What is the sparse iterator ALTREP pattern?"
- "ALTREP callbacks on worker thread vs main thread?"

## Key concepts

### The single-struct pattern

The old pattern placed `#[miniextendr]` on a 1-field wrapper struct. That pattern is removed. Use the derive macros instead.

Each ALTREP type is a plain Rust struct with one or more fields. The derive macro generates:

1. Low-level trait implementations (`Altrep`, `AltVec`, `Alt*` family traits).
2. The `RegisterAltrep` trait impl that creates the R class handle, installs method trampolines, and caches the handle in a `OnceLock`.
3. A `#[distributed_slice]` entry so the class is registered automatically in `R_init_<pkg>`.

The struct is **not** wrapped in another struct. The macro generates everything around the struct itself.

### Derive macros by family

| R type | Derive macro |
|--------|-------------|
| Integer (`INTSXP`) | `#[derive(AltrepInteger)]` |
| Real (`REALSXP`) | `#[derive(AltrepReal)]` |
| Logical (`LGLSXP`) | `#[derive(AltrepLogical)]` |
| Raw (`RAWSXP`) | `#[derive(AltrepRaw)]` |
| String (`STRSXP`) | `#[derive(AltrepString)]` |
| Complex (`CPLXSXP`) | `#[derive(AltrepComplex)]` |
| List (`VECSXP`) | `#[derive(AltrepList)]` |

All derives accept the same `#[altrep(...)]` attribute keys documented below.

### `#[altrep(...)]` attribute keys

| Key | Description |
|-----|-------------|
| `len = "field"` | Field holding the vector length. Auto-detected if named `len` or `length`. |
| `elt = "field"` | Field to return as element value (constant-value vector). |
| `elt_delegate = "field"` | Field whose `.elt(i)` method is called (delegation pattern). |
| `manual` | Skip `AltrepLen` and `Alt*Data` generation. User writes those trait impls. Registration still emitted automatically. |
| `no_lowlevel` | Suppress `impl_alt*_from_data!` macro invocation. Use when you write all traits by hand. |
| `dataptr` | Enable `Dataptr` method — R gets a direct pointer to underlying data. Mutually exclusive with `subset`. Not for List. |
| `serialize` | Enable `Serialized_state` and `Unserialize` for ALTREP serialization. |
| `subset` | Enable `Extract_subset` method. Mutually exclusive with `dataptr`. Only integer and complex. Not for List. |
| `unsafe` | Guard mode: no panic protection on callbacks. |
| `rust_unwind` | Guard mode: `catch_unwind` only — safe for pure Rust, unsafe if callbacks call R API. |
| `r_unwind` | Guard mode: `with_r_unwind_protect` (default) — safe for callbacks that call R API. |
| `class = "name"` | Override ALTREP class name (default: struct name). |

### Guard modes

The guard mode controls how panics and R longjmps inside ALTREP callback trampolines are handled. The `const GUARD: AltrepGuard` associated constant on the `Altrep` trait is set by the derive macro based on which key you choose. Because it is a const, the compiler eliminates dead branches at monomorphization time — zero runtime overhead.

The default across all families is `r_unwind` (i.e., `AltrepGuard::RUnwind`).

Declared in `miniextendr-api/src/altrep_traits.rs`:

```
AltrepGuard::Unsafe      — no protection
AltrepGuard::RustUnwind  — catch_unwind
AltrepGuard::RUnwind     — with_r_unwind_protect (default)
```

The trampoline dispatch lives in `miniextendr-api/src/altrep_bridge.rs` in `guarded_altrep_call`. For `RUnwind`, it delegates to `with_r_unwind_protect_sourced` with `PanicSource::Altrep`.

### The `AltrepExtract` trait

`AltrepExtract` abstracts how the macro-generated trampolines retrieve `&T` (and `&mut T`) from a SEXP. There is a blanket implementation for any `T: TypedExternal` (i.e., any type using the `ExternalPtr` mechanism). If your struct is backed by an `ExternalPtr`, you do not need to implement `AltrepExtract` yourself.

If you are using a custom storage scheme (e.g., data2 in an integer ALTREP without ExternalPtr), you must implement `AltrepExtract` manually.

Declared in `miniextendr-api/src/altrep_ext.rs`.

### Option validation rules

The derive macro enforces:

- `dataptr` and `subset` are mutually exclusive (you cannot have both).
- `subset` is only valid for integer and complex families.
- List rejects `dataptr`, `serialize`, and `subset` — list ALTREP has different semantics.
- String ALTREP that calls R API in its `elt` (which is common, since `elt` must return a CHARSXP allocated with `Rf_mkCharLenCE`) requires at least `r_unwind` guard mode. The string family default is `RUnwind`.

### String ALTREP and NA

Use `Vec<Option<String>>` (not `Vec<String>`) for string ALTREP vectors that need to preserve `NA_character_`. With `Vec<String>`, NAs cannot be round-tripped because `String` has no sentinel value.

The `elt` callback for string ALTREP must return a `CHARSXP` allocated with `Rf_mkCharLenCE` (or `Rf_mkCharCE`). These are R API calls, which is why string ALTREP defaults to `r_unwind` guard mode: `elt` may trigger R's allocator, which can longjmp.

### ALTREP callbacks and threads

ALTREP callbacks (length, elt, dataptr, serialized_state, etc.) are always called by R on the main R thread. They receive raw `SEXP` arguments, which are not `Send`. Do not attempt to route ALTREP callbacks to the worker thread. See `miniextendr-worker` for the threading model.

### `impl_altinteger_from_data!` and the `no_lowlevel` escape hatch

When you use `#[altrep(manual)]`, the derive skips generating `AltrepLen` and `Alt*Data` impls but still emits `impl_alt*_from_data!(YourType)`. If you also want to suppress that macro call (for example, because you want to write all six low-level traits by hand), add `#[altrep(no_lowlevel)]`.

Typical use: custom storage that needs to implement `AltrepExtract` differently AND write hand-tuned trait impls.

## How it works

### Field-based derive (common case)

Annotate your struct:

```rust
use miniextendr_api::prelude::*;

#[derive(AltrepInteger)]
#[altrep(len = "length", elt = "value", class = "ConstantInt")]
pub struct ConstantIntVector {
    pub length: usize,
    pub value: i32,
}
```

The prelude re-exports the full ALTREP surface (#1230): the seven per-family
derives (`AltrepInteger`/`Real`/`Logical`/`Raw`/`String`/`Complex`/`List`),
the `AltrepLen`/`Alt*Data` traits, and `AltrepExtract`/`RegisterAltrep` —
so `use miniextendr_api::prelude::*;` alone is enough.

The derive emits:
- `impl AltrepLen for ConstantIntVector` reading `self.length`.
- `impl AltIntegerData for ConstantIntVector` returning `self.value` for every index.
- `impl_altinteger_from_data!(ConstantIntVector)` — the macro that implements `Altrep`, `AltVec`, `AltInteger` for the low-level trampoline layer.
- `impl RegisterAltrep for ConstantIntVector` — creates and caches the R class handle.
- A `#[distributed_slice(MX_ALTREP_REGISTRATIONS)]` entry.

To expose a constructor from R:

```rust
#[miniextendr]
pub fn make_constant_int(value: i32, length: i32) -> SEXP {
    ConstantIntVector {
        length: length.max(0) as usize,
        value,
    }
    .into_sexp()
}
```

### Manual derive path

Use `#[altrep(manual)]` when your element logic is more complex than a single field lookup.

```rust
#[derive(AltrepReal)]
#[altrep(manual, class = "FibonacciReal")]
pub struct FibonacciVector {
    pub len: usize,
}

impl AltrepLen for FibonacciVector {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltRealData for FibonacciVector {
    fn elt(&self, i: usize) -> f64 {
        // custom fibonacci computation
        fib(i) as f64
    }
    fn as_slice(&self) -> Option<&[f64]> {
        None  // cannot provide contiguous slice
    }
}
```

The macro still emits `impl_altreal_from_data!(FibonacciVector)` and `RegisterAltrep`. You do not call those macros yourself.

### Sparse iterator pattern

For very large vectors where only a few elements will be accessed, use the sparse iterator data types from `miniextendr-api/src/altrep_data/`. They store only accessed elements in a `BTreeMap` keyed by index, using `Iterator::nth()` to skip. Skipped elements return NA/default and cannot be retrieved later.

See `docs/SPARSE_ITERATOR_ALTREP.md` for the complete walkthrough including the `SparseIterIntData`, `SparseIterRealData` types, prefix-caching vs sparse trade-off table, and construction reference.

## Decision trees

### Which guard mode?

Start here: do your ALTREP callbacks call any R API functions?

- Yes, they call R API (`Rf_mkCharLenCE`, `Rf_allocVector`, `Rf_eval`, etc.):
  - Use `r_unwind` (default). The R API can longjmp; you need `R_UnwindProtect` to run Rust destructors.
- No, they are pure Rust (field reads, BTreeMap lookups, arithmetic):
  - Do you need to save ~2ns per callback?
    - Yes: use `rust_unwind`. Wraps in `catch_unwind`, catches Rust panics, but cannot catch R longjmps (safe because you confirmed no R API calls).
    - No: leave the default `r_unwind`. The compile-time const eliminates the overhead anyway.
- Do you have trivial callbacks that provably cannot panic and call no R API?
  - Use `unsafe`. Reserve for hot paths only; if your callback ever panics, behavior is undefined.

String ALTREP: always use `r_unwind` (or leave default). String `elt` must allocate CHARSXP.

### Field-based vs manual derive?

- Is your element value a single struct field or a delegation to an inner type? Use field-based with `elt = "field"` or `elt_delegate = "field"`.
- Do you need computed elements, conditional NA, or custom `get_region` logic? Use `#[altrep(manual)]` and implement `AltrepLen` + `Alt*Data` yourself.
- Do you want to write all low-level traits by hand (rare)? Add `no_lowlevel` on top of `manual`.

### Which options to enable?

- Always providing a contiguous `&[T]` (e.g., `Vec<T>` backing)? Enable `dataptr` for maximum R compatibility.
- Supporting efficient random subsetting via `[indices]`? Enable `subset` (integer/complex only).
- Need `saveRDS()` / `readRDS()` to work without materializing? Enable `serialize`.
- Using a lazy or sparse iterator where you cannot provide a contiguous slice? Enable neither `dataptr` nor `subset`.

Never enable `dataptr` and `subset` together — they are mutually exclusive by design.

## Key files

- `miniextendr-api/src/altrep_traits.rs` — `AltrepGuard` enum, `Altrep` base trait with `const GUARD`, all type-family traits (`AltInteger`, `AltReal`, `AltString`, `AltList`, etc.).
- `miniextendr-api/src/altrep_bridge.rs` — `guarded_altrep_call` dispatch and all `extern "C-unwind"` trampolines.
- `miniextendr-api/src/altrep.rs` — `AltrepClass`, `RegisterAltrep`, `validate_altrep_class`, `make_class_by_base`.
- `miniextendr-api/src/altrep_ext.rs` — `AltrepExtract` trait and blanket impl for `TypedExternal`.
- `miniextendr-api/src/altrep_impl.rs` — `impl_altinteger_from_data!` and sibling macros for each family.
- `miniextendr-api/src/altrep_data.rs` — `AltrepLen`, `AltIntegerData`, `AltRealData`, and friends.
- `miniextendr-macros/src/altrep_derive.rs` — derive macro implementation, `AltrepAttrs` parser, `AltrepFamilyConfig`.
- `miniextendr-api/src/ffi_guard.rs` — `guarded_ffi_call` and `guarded_ffi_call_with_fallback` (used by `guarded_altrep_call`).
- `docs/SPARSE_ITERATOR_ALTREP.md` — compute-on-access iterator pattern guide.

## Common pitfalls

- **`#[miniextendr]` on a 1-field struct is rejected**: this pattern was removed. Use `#[derive(Altrep*)]` instead. The error message points you to the correct derive.

- **`subset` on a real vector**: `Extract_subset` is only defined for integer and complex in R's ALTREP API. Enabling `subset` on a real ALTREP is a compile-time error.

- **`dataptr` + `subset` together**: mutually exclusive. Choose one or neither.

- **String ALTREP with `Vec<String>` loses NA**: use `Vec<Option<String>>`. `String` has no sentinel; `Option<String>` with `None` maps to `NA_character_`.

- **ALTREP callback calls R API but guard is `rust_unwind`**: if R's allocator longjmps inside your callback (e.g., out-of-memory), `catch_unwind` does not catch longjmps, and Rust destructors do not run. Use `r_unwind` for any callback that calls R API.

- **Forgetting `AltrepExtract` for custom storage**: the trampolines call `altrep_extract_ref(x)` and `altrep_extract_mut(x)` to get `&T` and `&mut T` from the SEXP. If your type is not backed by `TypedExternal`, the blanket impl does not apply; you must implement `AltrepExtract` yourself.

- **GC discipline in `elt` callbacks**: the `elt` callback for string ALTREP allocates a fresh CHARSXP with `Rf_mkCharLenCE`. If you then store that CHARSXP in a temporary before returning, it must be protected across any subsequent R API call that could trigger GC. Use `OwnedProtect` or pass directly to the return. See `miniextendr-ffi` for the protection model.

- **ALTREP callbacks are on the main thread**: you cannot call `run_on_worker` from inside an ALTREP callback. The callback is already on the main thread. Use `with_r_thread` only if you are on the worker and need R API access; inside ALTREP callbacks, call R API directly (with appropriate guard mode set).

- **gctorture discipline**: if your ALTREP stores SEXPs across allocations (e.g., in a `Vec<SEXP>` sidecar), run a `gctorture(TRUE)` sweep before committing. CI's stricter R-devel GC is more aggressive and aborts with `malloc(): unsorted double linked list corrupted` on protect violations. See `docs/GCTORTURE_TESTING.md` for the harness pattern.

## Related skills

- `miniextendr-worker` — threading model, why ALTREP callbacks stay on main thread.
- `miniextendr-ffi` — `#[r_ffi_checked]`, `_unchecked` variants, GC protection, unwind protection.
- `miniextendr-externalptr` — `TypedExternal`, `AltrepExtract` blanket impl, pointer provenance.
- `miniextendr-macros` — how `#[miniextendr]` on functions and the ALTREP derives fit into the broader macro system.
