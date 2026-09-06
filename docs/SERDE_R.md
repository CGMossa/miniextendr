# Native Rust-R serialization with serde

The `serde` feature provides direct serialization between Rust types and native
R objects without an intermediate format such as JSON. The API lives in
`miniextendr_api::serde` and preserves R's native data types.

## Overview

| Feature | `serde_json` (JSON) | `serde` (native R) |
|---------|----------------|-------------------|
| Intermediate format | JSON string | None |
| Type preservation | No (all numbers → f64) | Yes (i32 stays i32) |
| NA handling | Limited | Full support via `Option<T>` |
| Performance | Extra parse/stringify | Direct conversion |
| Smart Vec dispatch | No | Yes (Vec<i32> → integer vector) |

## Enabling the Feature

```toml
# Cargo.toml
[dependencies]
miniextendr-api = { version = "0.1", features = ["serde"] }

# Or for both JSON and native R serialization:
miniextendr-api = { version = "0.1", features = ["serde_json"] }
```

## Type Mappings

### Serialization (Rust → R)

| Rust Type | R Type | Notes |
|-----------|--------|-------|
| `bool` | `logical(1)` | Scalar |
| `i8/i16/i32` | `integer(1)` | Widened to i32 |
| `i64/u64/f32/f64` | `numeric(1)` | Converted to f64 |
| `String/&str` | `character(1)` | UTF-8 preserved |
| `Option<T>::Some(v)` | T | Transparent |
| `Option<T>::None` | `NULL` | |
| `Vec<i32>` | `integer` vector | Smart dispatch |
| `Vec<f64>` | `numeric` vector | Smart dispatch |
| `Vec<bool>` | `logical` vector | Smart dispatch |
| `Vec<String>` | `character` vector | Smart dispatch |
| `Vec<struct>` | `list` of lists | Heterogeneous |
| `HashMap<String, T>` | named `list` | Keys become names |
| `BTreeMap<String, T>` | named `list` | Sorted keys |
| `struct { fields }` | named `list` | Field names preserved |
| `()` / unit struct | `NULL` | |
| unit enum variant | `character(1)` | Variant name |
| newtype variant | `list(Variant = value)` | Tagged |
| tuple variant | `list(Variant = list(...))` | Tagged list |
| struct variant | `list(Variant = list(a=..., b=...))` | Tagged named list |

### Deserialization (R → Rust)

| R Type | Rust Type | Notes |
|--------|-----------|-------|
| `logical(1)` | `bool` | Bare (non-`Option`) target: NA is an error |
| `integer(1)` | `i32` | Bare (non-`Option`) target: NA is an error |
| `numeric(1)` | `f64` | Bare (non-`Option`) target: NA is an error |
| `character(1)` | `String` | Bare (non-`Option`) target: NA is an error |
| `integer` vector | `Vec<i32>` | |
| `numeric` vector | `Vec<f64>` | |
| `logical` vector | `Vec<bool>` | |
| `character` vector | `Vec<String>` | |
| `raw` vector | `Vec<u8>` / `&[u8]` | |
| named `list` | struct / `HashMap` | Field matching |
| unnamed `list` | `Vec<T>` / tuple | Positional |
| `NULL` | `()` / `Option::None` | |
| `NA` (any of the four above) or `NULL` | `Option<T>::None` | See [NA and NULL handling](#na-and-null-handling) below |

**Input-side contract (audit A5):** a typed scalar `NA` reaching a *bare*
(non-`Option`) field is a genuine missingness error and is rejected — it is
never silently coerced. An `Option<T>` field accepts *either* `NA` or `NULL`
as `None`. This matches the macro `TryFromSexp` convention documented in
[`CONVERSION_MATRIX.md`](CONVERSION_MATRIX.md) (see its `Option<T>` table),
so the two conversion layers now agree on what counts as "missing" on input.
Output-side conventions remain per-layer and unchanged: native serde's `to_r()`
always serializes `None` to `NULL` (see the Serialization table above), while
the macro's scalar `IntoR` serializes `None` to `NA`.

## Basic Usage

### Defining Serializable Types

```rust
use serde::{Serialize, Deserialize};
use miniextendr_api::{miniextendr, ExternalPtr};
use miniextendr_api::serde::{RDeserializeNative, RSerializeNative};

#[derive(Serialize, Deserialize, Clone, ExternalPtr)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[miniextendr]
impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}

// Register the adapter traits
#[miniextendr]
impl RSerializeNative for Point {}

#[miniextendr]
impl RDeserializeNative for Point {}

// Registration is automatic via #[miniextendr].
```

### Using from R

```r
# Create a Point
p <- Point$new(1.0, 2.0)

# Serialize to R list
data <- p$to_r()
# list(x = 1.0, y = 2.0)

# Access fields
data$x  # 1.0
data$y  # 2.0

# Deserialize from R list
p2 <- Point$from_r(list(x = 3.0, y = 4.0))
p2$x  # 3.0
p2$y  # 4.0

# Round-trip
original <- Point$new(5.0, 6.0)
restored <- Point$from_r(original$to_r())
identical(original$x, restored$x)  # TRUE
```

## Smart Vec Dispatch

One of the key features of the native serde bridge is smart vector dispatch.
When serializing `Vec<T>`, the serializer automatically chooses the most
efficient R representation:

```rust
// Vec<i32> -> integer vector (atomic)
let ints = vec![1, 2, 3, 4, 5];
// Serializes to: c(1L, 2L, 3L, 4L, 5L)

// Vec<f64> -> numeric vector (atomic)
let floats = vec![1.1, 2.2, 3.3];
// Serializes to: c(1.1, 2.2, 3.3)

// Vec<String> -> character vector (atomic)
let strings = vec!["a".to_string(), "b".to_string()];
// Serializes to: c("a", "b")

// Vec<Point> -> list of lists (heterogeneous)
let points = vec![Point { x: 1.0, y: 2.0 }];
// Serializes to: list(list(x = 1.0, y = 2.0))
```

## NA and NULL handling

### Option<T> for NA Support

Use `Option<T>` to represent potentially missing values:

```rust
#[derive(Serialize, Deserialize, ExternalPtr)]
pub struct Record {
    pub id: i32,                    // Required
    pub name: Option<String>,       // Optional (can be NULL or NA_character_)
    pub value: Option<f64>,         // Optional (can be NULL or NA_real_)
}
```

From R:
```r
# Create with all values
r1 <- Record$from_r(list(id = 1L, name = "test", value = 3.14))

# Create with missing values -- NULL and the type-appropriate NA sentinel
# are equivalent inputs for an Option<T> field (audit A5).
r2 <- Record$from_r(list(id = 2L, name = NULL, value = NULL))
r3 <- Record$from_r(list(id = 3L, name = NA_character_, value = NA_real_))

# Serialize back -- native serde's output convention is always NULL for None,
# regardless of whether NA or NULL was the input.
r2$to_r()
# list(id = 2L, name = NULL, value = NULL)
r3$to_r()
# list(id = 3L, name = NULL, value = NULL)

# `id` is a bare (non-Option) i32: a typed NA there is a genuine
# missingness error, not an absence signal, so this still fails.
Record$from_r(list(id = NA_integer_, name = "test", value = 3.14))
# Error: unexpected NA value
```

## Nested Structures

The native serde bridge handles arbitrarily nested structures:

```rust
#[derive(Serialize, Deserialize)]
pub struct Level3 {
    pub data: Vec<f64>,
    pub flag: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Level2 {
    pub level3: Level3,
    pub values: Vec<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct Level1 {
    pub level2: Level2,
    pub name: String,
}

#[derive(Serialize, Deserialize, ExternalPtr)]
pub struct DeepNest {
    pub level1: Level1,
}
```

From R:
```r
# Create from deeply nested R list
deep <- list(
  level1 = list(
    level2 = list(
      level3 = list(
        data = c(1.0, 2.0, 3.0),
        flag = TRUE
      ),
      values = c(10L, 20L, 30L)
    ),
    name = "nested"
  )
)

dn <- DeepNest$from_r(deep)
```

## Enum Serialization

### Unit Variants

Unit enum variants serialize to character strings:

```rust
#[derive(Serialize, Deserialize)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}
```

From R:
```r
# Unit variant -> character
status <- "Active"  # Deserializes to Status::Active
```

### Data Variants

Data-carrying variants serialize to tagged lists:

```rust
#[derive(Serialize, Deserialize)]
pub enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}
```

From R:
```r
# Circle { radius: 5.0 } serializes to:
list(Circle = list(radius = 5.0))

# Rectangle { width: 10.0, height: 20.0 } serializes to:
list(Rectangle = list(width = 10.0, height = 20.0))
```

## HashMap/BTreeMap

Maps with string keys become named R lists:

```rust
use std::collections::HashMap;

#[derive(Serialize, Deserialize, ExternalPtr)]
pub struct Config {
    pub settings: HashMap<String, i32>,
    pub metadata: HashMap<String, String>,
}
```

From R:
```r
cfg <- Config$from_r(list(
  settings = list(timeout = 30L, retries = 3L),
  metadata = list(author = "test", version = "1.0")
))

data <- cfg$to_r()
data$settings$timeout  # 30L
data$metadata$author   # "test"
```

## Standalone Functions

For one-off conversions without registering types:

```rust
use miniextendr_api::serde::{from_r, to_r};

#[miniextendr]
pub fn convert_to_r() -> SEXP {
    let data = vec![1, 2, 3, 4, 5];
    to_r(&data).expect("serialize")
}

#[miniextendr]
pub fn convert_from_r(sexp: SEXP) -> Vec<i32> {
    from_r(sexp).expect("deserialize")
}
```

### Owned values: `RValue::from_serde`

`to_r` allocates a `SEXP`, so it needs the R main thread. When a value has to
be built somewhere else first (a worker thread, a panic payload, a condition's
`data =`), serialize into the owned, `Send` `RValue` tree instead and
materialise it later:

```rust
use miniextendr_api::RValue;

let v: RValue = RValue::from_serde(&point)?;
// equivalently: miniextendr_api::serde::to_rvalue(&point)?
```

The mapping is the table above: homogeneous `Vec<scalar>` coalesces to an
atomic vector, structs and string-keyed maps become named lists, unit variants
become strings, data variants become `list(Variant = ...)`. `None` becomes
`NULL`, not a typed `NA`, because the serializer never sees the absent inner
type. The `Err` arm of a `Result<T, E>` uses this to turn a serde-tagged error
enum into a classed R condition; see
[CONDITIONS.md](CONDITIONS.md#deriving-the-classes-from-a-serde-error-type).

## Columnar `data.frame` Assembly

For `&[T: Serialize]`, `vec_to_dataframe` produces a column-oriented R
`data.frame` where each field of `T` becomes one atomic column. Nested structs
are recursively flattened into prefixed columns (`point_x`, `point_y`);
`#[serde(flatten)]` fields appear without a prefix; `#[serde(skip_serializing_if)]`
fills NA. `Option<Struct>` fills NA across all sub-columns when `None`.

```rust
use miniextendr_api::dataframe::BuiltDataFrame;
use miniextendr_api::serde::vec_to_dataframe;

#[derive(Serialize)]
struct Row {
    id: i32,
    point: Point,           // flattened to point_x, point_y
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,   // NA when None
}

#[miniextendr]
pub fn rows_as_df(rows: Vec<Row>) -> BuiltDataFrame {
    vec_to_dataframe(&rows).unwrap()
        .rename("point_x", "x")
        .rename("point_y", "y")
        .drop("note")
}
```

`BuiltDataFrame` implements `IntoR`, so return it directly from a
`#[miniextendr]` function. No explicit `.build()` or `into_sexp()` call is
needed. The handle keeps every constructor/editing chain GC-rooted until the
SEXP is handed to R.

### Streaming rows

Use `iter_to_dataframe` for a one-pass iterator. Use `SerdeRowBuilder<T>` when
rows arrive incrementally or the schema needs to be declared or allowed to
grow:

```rust
use miniextendr_api::serde::{SerdeRowBuilder, TypeSpec};

let mut builder = SerdeRowBuilder::<Row>::with_schema(
    [
        ("id", TypeSpec::Integer),
        ("note", TypeSpec::Optional(Box::new(TypeSpec::Character))),
    ],
    None,
).grow_schema();

builder.push(row)?;
let df: BuiltDataFrame = builder.finish()?;
```

`finish()` returns a rooted `BuiltDataFrame`, including for the empty
0-row/0-column case.

### Results and enum output shapes

`result_to_dataframe` turns `&[Result<T, E>]` into a typed `DataFrameShape`:

- `ResultShape::Auto` returns a bare data frame when every row is `Ok`, and a
  `list(results = ..., error = ...)` when any row is `Err`.
- `ResultShape::Split` always returns that two-slot list.
- `ResultShape::Collated` returns one data frame with an `is_error` column and
  the union of the `T` and `E` fields.

```rust
use miniextendr_api::serde::{DataFrameShape, ResultShape, result_to_dataframe};

#[derive(serde::Serialize)]
struct ErrorRow {
    id: i32,
    reason: String,
}

#[miniextendr]
fn results_df(rows: Vec<Result<Row, ErrorRow>>) -> Result<DataFrameShape, String> {
    result_to_dataframe(
        &rows,
        ResultShape::Auto { empty_ok_sentinel: () }, // NULL if every row is Err
    )
    .map_err(|error| error.to_string())
}
```

Every frame inside `DataFrameShape` is a rooted `BuiltDataFrame`. When an
all-error split uses a caller-supplied sentinel, the shape keeps it alive in a
`RootedSentinel` until `IntoR` consumes the result. The shape is therefore safe
to hold across intervening R allocations; it is not a convert-immediately view.

For streaming `Result<T, E>` rows, `dispatch_to_dataframes` incrementally fills
two serde builders and always returns `list(ok = <df>, err = <df>)`; customize
the names with `DispatchNames`. For enums, `vec_to_dataframe_split` selects a
per-variant list or collated frame via `SplitShape`.

## Error Handling

Deserialization can fail for various reasons:

```rust
use miniextendr_api::serde::from_r;

#[miniextendr]
pub fn safe_deserialize(sexp: SEXP) -> Result<Point, String> {
    from_r::<Point>(sexp).map_err(|e| e.to_string())
}
```

Error types include:
- `TypeMismatch` - Wrong R type for target Rust type
- `MissingField` - Required struct field not in list
- `InvalidVariant` - Unknown enum variant name
- `LengthMismatch` - Wrong length for tuple/array
- `UnexpectedNa` - NA where not allowed
- `Overflow` - Numeric overflow in conversion

## Integration with R Object Systems

### With R6

```r
library(R6)

MyClass <- R6Class("MyClass",
  public = list(
    x = NULL,
    y = NULL,
    initialize = function(x, y) {
      self$x <- x
      self$y <- y
    },
    to_list = function() list(x = self$x, y = self$y)
  )
)

obj <- MyClass$new(1.0, 2.0)
point <- Point$from_r(obj$to_list())
```

### With S4

```r
setClass("S4Point", slots = c(x = "numeric", y = "numeric"))
s4obj <- new("S4Point", x = 3.0, y = 4.0)

# Extract slots as list
point <- Point$from_r(list(x = s4obj@x, y = s4obj@y))
```

### With S7

```r
library(S7)

S7Point <- new_class("S7Point",
  properties = list(x = class_double, y = class_double)
)

s7obj <- S7Point(x = 5.0, y = 6.0)
point <- Point$from_r(list(x = prop(s7obj, "x"), y = prop(s7obj, "y")))
```

### With Environments

```r
e <- new.env()
e$x <- 7.0
e$y <- 8.0

point <- Point$from_r(as.list(e))
```

## Comparison with IntoList Derive

miniextendr also provides `#[derive(IntoList)]` for simpler struct-to-list conversion. Here's how they compare:

| Feature | `IntoList` | native serde |
|---------|-----------|-----------|
| Derive macro | Yes | Needs serde derives |
| Deserialization | No (one-way) | Yes (bidirectional) |
| Enum support | No | Yes |
| Smart Vec dispatch | No | Yes |
| HashMap/BTreeMap | No | Yes |
| Option/NA | No | Yes |
| Nested structs | Yes | Yes |

Use `IntoList` for simple one-way struct-to-list conversion. Use native serde
when you need full bidirectional serialization, enum support, or smart vector
handling.

## Satellite crates: R interop for a serde-only crate

A crate that already derives `serde::{Serialize, Deserialize}` gets R interop
**without ever depending on miniextendr**. Keep your data crate (the
"satellite") miniextendr-free and do all the bridging in the R-package crate
that already links miniextendr.

```
  satellite/                    rpkg/src/rust/  (the R package crate)
  ┌────────────────────┐        ┌──────────────────────────────────┐
  │ serde only          │        │ depends on miniextendr-api        │
  │ #[derive(Serialize, │  path  │ + satellite (path dep)            │
  │   Deserialize)]     │◄───────│                                   │
  │ struct Reading {…}  │  dep   │ #[miniextendr]                    │
  │                     │        │ fn readings_df() -> BuiltDataFrame {
  │ NO miniextendr,     │        │   vec_to_dataframe(&readings())   │
  │ NO FFI, NO R        │        │ }   // the ONLY bridge code        │
  └────────────────────┘        └──────────────────────────────────┘
```

### Why split it this way

- The satellite crate stays portable: no FFI, no R, nothing from miniextendr in
  its dependency tree. It compiles and tests on its own and is reusable outside
  R entirely.
- All R-specific glue lives in one place — the package crate — and every helper
  is generic over `T: Serialize` / `T: Deserialize`, so a new type costs a few
  lines, not a conversion impl.

### Layout

The satellite is a normal path dependency, sealed as its own workspace so it
is excluded from the package's workspace:

```toml
# satellite/Cargo.toml — serde and nothing else
[package]
name = "satellite"
edition = "2024"
[workspace]                      # sealed: not a member of any outer workspace
[dependencies]
serde = { version = "1", features = ["derive"] }
```

```toml
# package crate Cargo.toml
[workspace]
exclude = ["satellite"]          # don't treat the path dep as a nested member
[dependencies]
satellite = { path = "satellite" }
```

`serde` unifies across the two crates (one `^1` resolution), so
`satellite::Reading` implements the *same* `serde::Serialize` that
miniextendr's bridge functions require — no shared-trait wiring needed.

### Troubleshooting duplicate serde versions

The bridge requires more than two traits with the same spelling: the
satellite derive and `miniextendr-api` must use the **same resolved serde crate
instance**. Cargo normally gives them one instance when both dependency
requirements are compatible with `serde = "1"`. An incompatible major, or
non-overlapping exact pins within the same major, can instead put two serde
versions in the package graph. Traits from those versions have different crate
identities and are not interchangeable.

The resulting compiler error is usually an E0277 at a bridge call, for example
that `satellite::Reading: serde::Serialize` is not satisfied even though
`Reading` visibly derives `Serialize`. Inspect duplicate versions and their
reverse dependency paths from the R-package crate:

```bash
cargo tree -d
cargo tree -i serde@<version-reported-by-cargo-tree>
```

Fix the dependency graph rather than adding another derive or a conversion
shim:

- Prefer compatible requirements such as `serde = { version = "1", features =
  ["derive"] }` in the satellite.
- Remove unnecessary exact pins and update the lockfile so Cargo can select one
  version satisfying both crates.
- If the satellite must use an incompatible serde major, expose a feature that
  selects serde 1 for this integration or define a package-owned data adapter
  whose fields use serde-1-compatible types, then convert explicitly between
  the adapter and the satellite value. A derive against one serde dependency
  does not satisfy the trait from a different serde crate.

### The bridge (the only miniextendr-aware code)

```rust
use miniextendr_api::{miniextendr, SEXP};
use miniextendr_api::dataframe::BuiltDataFrame;
use miniextendr_api::serde::{AsSerialize, from_r, vec_to_dataframe};

// Vec<struct> → columnar data.frame (nested structs flatten, Option → NA).
#[miniextendr]
fn readings_df() -> Result<BuiltDataFrame, String> {
    vec_to_dataframe(&satellite::sample_readings()).map_err(|e| e.to_string())
}

// struct → R list (row-oriented).
#[miniextendr]
fn readings_list() -> AsSerialize<Vec<satellite::Reading>> {
    AsSerialize(satellite::sample_readings())
}

// R → Rust → R: deserialize an R list into the satellite type, round-trip back.
#[miniextendr]
fn echo_reading(x: SEXP) -> Result<AsSerialize<satellite::Reading>, String> {
    Ok(AsSerialize(from_r::<satellite::Reading>(x).map_err(|e| e.to_string())?))
}
```

### What you get for free (data interchange)

| Capability | Bridge entry point |
|---|---|
| `struct` ↔ named R `list` | `AsSerialize` / `from_r` |
| `Vec<struct>` → columnar `data.frame` | `vec_to_dataframe` |
| nested struct → flattened columns | `vec_to_dataframe` (`site_lat`, `site_lon`) |
| `Option<T>` → `NA` (round-trips) | any serde path |
| `enum` → tagged list / per-variant `data.frame` | `vec_to_dataframe_split`, `result_to_dataframe` |
| nested `enum` field → `<field>_variant` tag + `<field>_<sub>` columns | `vec_to_dataframe_flatten_enums`, `…_with_tags` (custom tag name) |
| `HashMap`/`BTreeMap` → named list / `data.frame` | `map_to_dataframe`, `hashmap_to_dataframe` |
| `data.frame` → `Vec<struct>` | `dataframe_to_vec` / `SerdeRows` (`dataframe_to_vec_with_struct_fields` for the #1320 tag-collision opt-out) |
| collated / flattened enum `data.frame` → `Vec<enum>` (collated and flattened shapes) | `dataframe_to_vec_collated` (top-level), `dataframe_to_vec` / `dataframe_to_vec_with_enum_tags` (nested fields) |

The **collated and flattened** enum shapes are bidirectional: nested enum
fields written by `vec_to_dataframe_flatten_enums` read back via plain
`dataframe_to_vec` (default `<field>_variant` tag), and a top-level
`SplitShape::Collated { column }` frame reads back via
`dataframe_to_vec_collated(sexp, column)`. When the writer used custom
tag-column names (`vec_to_dataframe_flatten_enums_with_tags(rows, fields,
&[(field, tag)])`), pass the **same** mapping to
`dataframe_to_vec_with_enum_tags(sexp, &[(field, tag)])` so the reader finds
each field's tag column. Unknown variant strings and missing tag columns
surface a clear `RSerdeError`.

The other enum writer shapes remain **write-only**: `PerVariantList` /
`PerVariantListWithTag` (and `result_to_dataframe`'s split output) produce
per-variant frame *lists* no reader consumes, and internally-tagged flattened
fields (`<field>_<tagfield>`, no `_variant` column) are not covered by the
reader path — see [#1321](https://github.com/A2-ai/miniextendr/issues/1321).

### Reader caveat: the `_variant` tag-column collision (#1320)

The reader has no type information when it meets an `Option<T>` field with no
bare column — it cannot tell `Option<NestedStruct>` from `Option<Enum>`. To
read `Option<Enum>` `None` rows back, it probes the would-be variant-tag
column — `<field>_variant` by default, or the configured
`dataframe_to_vec_with_enum_tags` override — and an NA character/factor cell
there means `None`. The `_variant` suffix (or whatever tag name the reader is
configured with) is therefore effectively **reserved** under
`Option<nested struct>` fields.

**Silent loss** occurs when all of these hold:

1. the field is `Option<NestedStruct>` (flattened columns, no bare column);
2. the nested struct has a sub-field whose flattened column name equals the
   would-be tag column (a sub-field literally named `variant` under the
   default, e.g. `meta.variant` → `meta_variant`);
3. that column is character or factor, and NA at the row.

The whole struct then reads back as `None` — the other sub-fields'
values on that row are dropped without an error. Fixes:

| Approach | How | Trade-off |
|---|---|---|
| Struct opt-out (**preferred**) | `dataframe_to_vec_with_struct_fields(sexp, &["meta"])` | The heuristic never fires for `meta`; it is always read as a struct. An actual `None` struct row errors (non-`Option` sub-field) or reads as all-`None` `Some` — the writer emits no presence signal to recover it. |
| Rename the colliding sub-field | `#[serde(rename = "kind")] variant: Option<String>` on the inner struct field | Changes the emitted column name (`meta_kind`) on both write and read. |
| Per-field tag override | `dataframe_to_vec_with_enum_tags(sexp, &[("meta", "<unused column>")])` | Points the tag probe at a non-existent column so it never fires — works today, but expresses the intent poorly; prefer the opt-out. |

`Option<Enum>` fields are unaffected by the opt-out unless listed: fields not
named in `dataframe_to_vec_with_struct_fields` keep the tag heuristic, so
`None` enum rows still round-trip.

### What serde alone cannot give you

The serde bridge moves **values**. Anything about R-object **identity or
behaviour** needs miniextendr-native code in the package crate (a
`#[derive(...)]` or `#[miniextendr] impl` on a type the package crate owns) —
it cannot come from a serde-only satellite:

- **Live mutable handles** (`ExternalPtr`): returning a Rust object R holds and
  mutates in place, rather than a copy of its data.
- **R class systems** (R6 / S3 / S4 / S7) and **methods** callable on the type.
- **ALTREP** vectors, **custom connections**, Rust errors surfaced as **R
  conditions**, and **`...` (dots)** handling.

The dividing line is data vs. behaviour: a satellite crate ships data; objects,
methods, and classes live in the package crate.

### The irreducible per-type cost

You still write one `#[miniextendr]` free function per exported conversion —
`extern "C"` exports can't be generic, so each must name the concrete satellite
type. That function *is* the entire glue: name the type, call `vec_to_dataframe`
/ `AsSerialize` / `from_r`. (A `TryFrom<&[T]> for BuiltDataFrame` sugar would let you
write `rows.try_into()` inside the body, but you'd still need the named export,
so it saves nothing for this pattern.)
