# Arrow Integration

Zero-copy conversions between R vectors and Apache Arrow arrays.

## Quick Reference

```rust
use miniextendr_api::{miniextendr, SEXP};
use miniextendr_api::optionals::arrow_impl::*;

// R numeric → Arrow Float64Array → back to R: zero-copy both directions
#[miniextendr]
pub fn passthrough_numeric(x: Float64Array) -> Float64Array {
    x
}

// R integer → Arrow Int32Array → back to R: zero-copy both directions
#[miniextendr]
pub fn passthrough_integer(x: Int32Array) -> Int32Array {
    x
}

// Compute on Arrow, return to R (copies on return - new data)
#[miniextendr]
pub fn doubled(x: Float64Array) -> Float64Array {
    x.iter().map(|v| v.map(|f| f * 2.0)).collect()
}

// RecordBatch round-trip: primitive columns zero-copy per-column
#[miniextendr]
pub fn passthrough_df(df: RecordBatch) -> RecordBatch {
    df
}
```

## Zero-Copy String Vectors

R stores strings as STRSXP (array of CHARSXP pointers). Each CHARSXP is interned,
GC-managed, and has a known `LENGTH`. Instead of copying into `String`, borrow directly.

### `Cow<'static, str>`: scalar

```rust
#[miniextendr]
pub fn greet(name: Cow<'static, str>) -> String {
    // name is Cow::Borrowed - points directly into R's CHARSXP data
    // No allocation unless you call .to_mut()
    format!("Hello, {}!", name)
}
```

### `Vec<Cow<'static, str>>`: vector, zero-copy per element

```rust
#[miniextendr]
pub fn upper_first(words: Vec<Cow<'static, str>>) -> Vec<String> {
    // Each element is Cow::Borrowed (zero-copy from R's CHARSXP pool)
    words.iter().map(|w| {
        let mut s = w.to_string();
        if let Some(c) = s.get_mut(0..1) {
            c.make_ascii_uppercase();
        }
        s
    }).collect()
}

// NA-aware variant: None for NA_character_
#[miniextendr]
pub fn count_non_na(words: Vec<Option<Cow<'static, str>>>) -> i32 {
    words.iter().filter(|w| w.is_some()).count() as i32
}
```

### `Cow<'static, [T]>`: numeric slices

```rust
#[miniextendr]
pub fn sum_cow(x: Cow<'static, [f64]>) -> f64 {
    // Cow::Borrowed - x points directly into R's REALSXP data
    x.iter().sum()
}

// Round-trip: input is zero-copy (Cow::Borrowed into R's INTSXP), but the
// IntoR direction copies into a fresh R vector. Unlike Arrow buffers, a bare
// &[T] carries no provenance to prove it points at an R vector start, so a
// borrowed sub-slice can't be told apart from a full borrow — the copy is the
// only sound choice (see #880).
#[miniextendr]
pub fn passthrough_cow(x: Cow<'static, [i32]>) -> Cow<'static, [i32]> {
    x  // zero-copy in; copy back out
}
```

### `RCow<'static, T>`: safe zero-copy round-trip

When you want the round-trip back to R to *also* be zero-copy, use `RCow`
instead of `Cow`. `RCow`'s borrowed arm remembers the source SEXP it was read
from, so `IntoR` returns that exact R object — no copy, and no speculative
pointer recovery (the reason `Cow<[T]>` can't do this safely; see #880):

```rust
use miniextendr_api::RCow;

// Zero-copy BOTH directions: returns the original R vector unchanged.
#[miniextendr]
pub fn passthrough_rcow(x: RCow<'static, f64>) -> RCow<'static, f64> {
    x
}

// Mutation triggers copy-on-write, then materializes a fresh R vector.
#[miniextendr]
pub fn doubled_rcow(mut x: RCow<'static, f64>) -> RCow<'static, f64> {
    for v in x.to_mut() {
        *v *= 2.0;
    }
    x
}
```

A borrowed `RCow` can only be produced by reading an R vector (its fields are
private), so — unlike `Cow::Borrowed(&slice[2..5])` — it can never be a
sub-slice. That structural invariant is what makes returning the stored SEXP
sound. Slice it via `Deref` to get a plain `&[T]`; use `to_mut()` /
`into_owned()` when you need to mutate or keep the data past the call.

### `ProtectedStrVec` vs `StrVec`

`ProtectedStrVec` and `StrVec` both wrap an R STRSXP and provide zero-copy
`&str` access to its elements. They differ in GC safety:

| | `StrVec` | `ProtectedStrVec` |
|---|---|---|
| Size | 1 word (just the SEXP) | 3 words (SEXP + len + OwnedProtect) |
| Copy | `Copy` | `!Copy` (owns protection guard) |
| GC protection | None (caller's responsibility) | `OwnedProtect` keeps STRSXP alive |
| Borrow lifetime | `&'static str` (lie) | `&'a str` tied to `&'a self` |
| Iterator | `StrVecIter` (`Option<&'static str>`) | `ProtectedStrVecIter<'a>` (`Option<&'a str>`) |

**The key difference is lifetime safety.** `ProtectedStrVec` ties all borrows
to the struct's lifetime. The compiler catches use-after-free:

```rust
let dangling: &str;
{
    let sv = unsafe { ProtectedStrVec::new(sexp) };
    dangling = sv.get_str(0).unwrap(); // borrows &sv
} // sv dropped → SEXP unprotected
// dangling is now invalid - COMPILER ERROR: sv doesn't live long enough
```

With `StrVec` or `Vec<&'static str>`, the same code **compiles silently** and
produces a dangling pointer. The `'static` lifetime is a lie: the data is only
valid while R protects the SEXP).

**When to use which:**

- **`StrVec`** / **`Vec<&'static str>`**: inside a `#[miniextendr]` function
  where R protects the `.Call` argument. Lightweight, fine. The SEXP won't be
  GC'd during the call.
- **`ProtectedStrVec`**: when you store the string vector beyond the immediate
  scope, pass it to a closure, or want the compiler to catch lifetime bugs.
  The `OwnedProtect` guard keeps the STRSXP alive until the struct is dropped.

**Usage examples:**

```rust
use miniextendr_api::ProtectedStrVec;
use std::collections::HashSet;

#[miniextendr]
pub fn count_unique(strings: ProtectedStrVec) -> i32 {
    // Lifetimes tied to &self - compiler enforces GC safety
    let unique: HashSet<&str> = strings.iter()
        .filter_map(|s| s)  // skip NA
        .collect();
    unique.len() as i32
}

// Can't return &str - ProtectedStrVec is consumed by IntoR, so there's
// nothing to borrow from. Return String or the whole ProtectedStrVec.
#[miniextendr]
pub fn first_non_na(strings: ProtectedStrVec) -> String {
    strings.iter()
        .find_map(|s| s)
        .unwrap_or("")
        .to_owned()
}
```

```rust
use miniextendr_api::StrVec;

#[miniextendr]
pub fn has_empty(strings: StrVec) -> bool {
    // StrVec is Copy - just a SEXP wrapper. R protects .Call arguments,
    // so this is safe within the function body.
    strings.iter().any(|opt| opt == Some(""))
}
```

## Arrow Arrays

### R → Arrow (already zero-copy for primitives)

```rust
use miniextendr_api::optionals::arrow_impl::*;

#[miniextendr]
pub fn arrow_mean(x: Float64Array) -> f64 {
    // x.values() points directly into R's REALSXP data (zero-copy)
    // NA values are tracked in Arrow's null bitmap, not in the data
    let sum: f64 = x.iter().flatten().sum();
    let count = x.len() - x.null_count();
    sum / count as f64
}

#[miniextendr]
pub fn arrow_filter_positive(x: Int32Array) -> Int32Array {
    // Arrow compute - result is a new array (Rust-allocated)
    x.iter()
        .map(|v| v.filter(|&i| i > 0))
        .collect()
}
```

### Arrow → R (automatic SEXP recovery)

When an Arrow array's data buffer came from R (via `sexp_to_arrow_buffer`),
`IntoR` automatically recovers the original SEXP using pointer arithmetic.
No wrapper types needed.

```rust
// This is zero-copy BOTH directions:
#[miniextendr]
pub fn identity(x: Float64Array) -> Float64Array {
    x  // R→Arrow (zero-copy) → Arrow→R (pointer recovery, zero-copy)
}

// This copies on return (new data, not from R):
#[miniextendr]
pub fn squares(x: Float64Array) -> Float64Array {
    x.iter().map(|v| v.map(|f| f * f)).collect()
}
```

### RecordBatch (data.frame)

```rust
use arrow_array::cast::AsArray;

#[miniextendr]
pub fn df_add_column(df: RecordBatch) -> RecordBatch {
    let col0: &Float64Array = df.column(0).as_primitive();

    // Compute new column
    let new_col: Float64Array = col0.iter()
        .map(|v| v.map(|f| f * 2.0))
        .collect();

    // Build new batch - original columns return to R zero-copy,
    // new column copies (it's Rust-allocated)
    let mut fields = df.schema().fields().to_vec();
    fields.push(Arc::new(Field::new("doubled", DataType::Float64, true)));
    let schema = Arc::new(Schema::new(fields));

    let mut columns = df.columns().to_vec();
    columns.push(Arc::new(new_col));

    RecordBatch::try_new(schema, columns).unwrap()
}
```

### `alloc_r_backed_buffer`: Rust→Arrow→R zero-copy

Allocate an Arrow buffer backed by R memory from the start. Write through
the raw SEXP pointer, then wrap in Arrow types. When the array is later
converted to R, pointer recovery finds the original SEXP.

```rust
use miniextendr_api::optionals::arrow_impl::alloc_r_backed_buffer;

#[miniextendr]
pub fn generate_sequence(n: i32) -> SEXP {
    use miniextendr_api::IntoR;
    let n = n as usize;

    // Allocate buffer as R REALSXP - data lives in R's heap
    let (buffer, sexp) = unsafe { alloc_r_backed_buffer::<f64>(n) };

    // Fill through the SEXP's raw pointer (before wrapping in Arrow)
    unsafe {
        let ptr = miniextendr_api::sys::REAL(sexp);
        for i in 0..n {
            *ptr.add(i) = i as f64;
        }
    }

    // Wrap as Arrow array
    let values = arrow_buffer::ScalarBuffer::<f64>::from(buffer);
    let array = Float64Array::new(values, None);

    // IntoR → pointer recovery → returns the same REALSXP (zero-copy)
    array.into_sexp()
}
```

### `RStringArray`: string round-trip tracking

Arrow's StringArray and R's STRSXP have incompatible layouts (contiguous data+offsets
vs per-element CHARSXPs). Automatic pointer recovery can't work for strings.
`RStringArray` explicitly tracks the source STRSXP.

```rust
use miniextendr_api::optionals::arrow_impl::RStringArray;

#[miniextendr]
pub fn string_passthrough(x: RStringArray) -> RStringArray {
    // x.source is Some(strsxp) - IntoR returns original STRSXP
    x
}

#[miniextendr]
pub fn string_lengths(x: RStringArray) -> Vec<i32> {
    // Deref to StringArray - all Arrow APIs work
    x.iter().map(|opt| opt.map(|s| s.len() as i32).unwrap_or(-1)).collect()
}
```

### ALTREP for Cow string vectors

`Vec<Cow<'static, str>>` supports ALTREP with seamless serialization:

```rust
use miniextendr_api::IntoRAltrep;
use std::borrow::Cow;

#[miniextendr]
pub fn lazy_strings(prefix: &str, n: i32) -> SEXP {
    let strings: Vec<Cow<'static, str>> = (0..n)
        .map(|i| Cow::Owned(format!("{}_{}", prefix, i)))
        .collect();
    strings.into_sexp_altrep()
    // R sees a character vector; elements computed on demand via ALTREP Elt
    // saveRDS/readRDS works - serializes to STRSXP, deserializes back to Vec<Cow>
}
```

## How It Works

### SEXP Pointer Recovery (`r_memory` module)

R stores vector data at a fixed offset from the SEXP header:

```text
[VECTOR_SEXPREC header (48 bytes on 64-bit)] [data...]
 ^                                            ^
 SEXP                                         DATAPTR_RO(sexp)
```

All R vector types (REALSXP, INTSXP, RAWSXP, STRSXP, VECSXP) use the same
`VECTOR_SEXPREC` header. Non-vector types use larger `SEXPREC` but don't have
data pointers.

At package init, we measure the offset on a real R vector. Then in `IntoR`:

```text
candidate_sexp = data_ptr - offset
verify: TYPEOF(candidate) == expected AND LENGTH(candidate) == expected AND DATAPTR_RO(candidate) == data_ptr
```

**Safety consideration**: For Rust-allocated buffers, `data_ptr - offset` points to
arbitrary heap memory. The 4-byte type-tag read at that address is technically undefined
behavior in Rust's abstract model (the pointer wasn't derived from an R allocation).
In practice, this is safe. The address is in mapped heap memory and the read is
immediately validated by the triple check (type + length + DATAPTR_RO round-trip),
which makes false positives impossible. ALTREP vectors also fail safely (the
DATAPTR_RO round-trip check catches them, since ALTREP data isn't at a fixed offset).

### String conversion (`charsxp_to_str`)

`charsxp_to_str()` uses `R_CHAR` + `LENGTH` (O(1)) when R reports that the
CHARSXP is UTF-8/ASCII. Explicitly tagged Latin-1 strings are translated with
`Rf_translateCharUTF8`, and `bytes` strings are rejected. `charsxp_to_cow()`
returns `Cow::Borrowed` for the fast path and `Cow::Owned` when translation is
required.

## Type Decision Tree

```text
Need strings from R?
├── Scalar → Cow<'static, str>          (zero-copy for UTF-8/ASCII)
├── Vector, need ownership → Vec<String> (copies, lossy NA→"")
├── Vector, read-only → Vec<Cow<'static, str>>  (zero-copy per UTF-8/ASCII element)
├── Vector, NA-aware → Vec<Option<Cow<'static, str>>>
├── View with GC safety → ProtectedStrVec
└── Lightweight view → StrVec           (Copy, caller manages GC)

Need numerics from R?
├── As Rust slice → &[f64] / &[i32]    (zero-copy, 'static lifetime)
├── Copy-on-write → Cow<'static, [f64]> (zero-copy, copies on .to_mut())
├── As Arrow array → Float64Array       (zero-copy both directions)
└── Owned copy → Vec<f64>              (copies)

Need data frames?
├── As Arrow → RecordBatch             (primitive cols zero-copy both ways)
└── As Arrow (string cols too) → use RStringArray per column
```

The `'static` lifetimes above are API-level convenience types, not GC roots.
The borrows remain valid only while the source SEXP/CHARSXP is reachable;
normally keep them inside the generated `.Call` frame or use a rooted wrapper.
