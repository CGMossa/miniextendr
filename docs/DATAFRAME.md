# Data Frame Conversion in miniextendr

miniextendr converts between Rust types and R data frames through a `DataFrame`
view, an owned `BuiltDataFrame` handle, and two conversion verbs that mirror the
scalar/vector surface (`IntoR` and `TryFromSexp`). One verb builds a rooted data
frame from rows; the other reads an R-owned frame back into rows. Both verbs
live on the data, errors are a single `DataFrameError`, and missing cells
round-trip as nullable fields.

## The two verbs

| Trait | Method | Direction | Scalar analogue |
|---|---|---|---|
| `IntoDataFrame` | `rows.into_dataframe()? -> BuiltDataFrame` | Rust → R | `IntoR` |
| `FromDataFrame` | `Vec::<Row>::from_dataframe(&df)? -> Vec<Row>` | R → Rust | `TryFromSexp` |

The verbs are implemented **on the data** (`Vec<Row>` / `&DataFrame`), not on a companion type. There is one error type (`DataFrameError`) and one NA contract: a missing cell maps to `None` in an `Option<T>` field, and `None` maps back to `NA`.

For the full design rationale and the orphan-rule mechanics behind the blanket impls, see [`DATAFRAME_INTERFACE.md`](DATAFRAME_INTERFACE.md).

## Quick start

`#[derive(DataFrameRow)]` is the primary path: derive it on a row struct, then
return `BuiltDataFrame` from a `#[miniextendr]` function.

```rust
use miniextendr_api::{BuiltDataFrame, DataFrame, DataFrameRow, IntoList, miniextendr};

#[derive(Clone, IntoList, DataFrameRow)]
struct Measurement {
    time: f64,
    value: f64,
    sensor: String,
}

#[miniextendr]
fn get_measurements() -> BuiltDataFrame {
    let rows = vec![
        Measurement { time: 1.0, value: 10.0, sensor: "A".into() },
        Measurement { time: 2.0, value: 20.0, sensor: "B".into() },
        Measurement { time: 3.0, value: 30.0, sensor: "C".into() },
    ];
    rows.into_dataframe().unwrap()
}
```

The reverse direction reads a `DataFrame` argument back into rows:

```rust
#[miniextendr]
fn round_trip(df: DataFrame) -> BuiltDataFrame {
    let rows: Vec<Measurement> = Vec::<Measurement>::from_dataframe(&df).unwrap();
    rows.into_dataframe().unwrap()
}
```

`DataFrame` implements `TryFromSexp`, so it is the inbound view type.
`BuiltDataFrame` implements `IntoR`, so a Rust-created frame is handed back to
R without exposing an unrooted intermediate.

## The `DataFrame` view and `BuiltDataFrame` handle

`DataFrame` is a validated, cheap `Copy` view over a bare SEXP. It has no GC
root of its own and is sound only while R's `.Call` frame, a `ProtectScope`, or
another owning handle keeps the object reachable. `BuiltDataFrame` owns a
precious-list root and dereferences to the view. Every Rust-side constructor
returns the handle.

The new-frame producers (`drop`, `select`, `select_rows`, `prepend_column`,
`with_column`) also return `BuiltDataFrame`, so constructor → edit chains are
rooted at every link and safe across R allocations (#1247). `rename` /
`strip_prefix` edit the `names` attribute in place; on a `BuiltDataFrame`
receiver the handle carries through the chain.

```rust
let df: BuiltDataFrame = rows.into_dataframe()?;

df.nrow();                       // row count
df.ncol();                       // column count
df.names();                      // Vec<String> of column names
df.contains_column("sensor");    // bool

let values: Vec<f64> = df.column("value").unwrap();   // typed column accessor
let raw: SEXP = df.column_raw("sensor").unwrap();      // untyped column SEXP

let df = df
    .rename("value", "reading")  // rename a column (in place)
    .drop("time")                // remove a column
    .select(&["sensor", "reading"]); // keep/reorder a subset — still a rooted BuiltDataFrame
```

Use `DataFrame::from_sexp(sexp)` to validate an arbitrary SEXP, and `as_sexp()` / `as_list()` to drop down to the raw representation when you need it.

## `#[derive(DataFrameRow)]` in depth

### Heterogeneous types

The derive supports different types in different fields; each field keeps its R type:

```rust
#[derive(Clone, IntoList, DataFrameRow)]
struct Person {
    name: String,      // character in R
    age: i32,          // integer in R
    height: f64,       // numeric in R
    is_student: bool,  // logical in R
}
```

### Collection expansion

Fixed-size arrays `[T; N]` are **automatically expanded** into N suffixed columns. `#[dataframe(expand)]` / `#[dataframe(unnest)]` request it explicitly, though arrays expand by default.

```rust
#[derive(Clone, DataFrameRow)]
struct Point3D {
    label: String,
    coords: [f64; 3],  // → coords_1, coords_2, coords_3
}
```

For `Vec<T>`, `Box<[T]>`, and `&[T]`, two expansion modes are available:

**Fixed width** (`width = N`): expands into exactly N columns at compile time.

```rust
#[derive(Clone, DataFrameRow)]
struct Scored {
    name: String,
    #[dataframe(width = 3)]
    scores: Vec<f64>,  // → scores_1, scores_2, scores_3 as Option<f64>
}
```

- Shorter vecs: padded with `NA`.
- **Longer vecs: truncated to N** (extra elements silently dropped).

**Auto-expand** (`expand` or `unnest`): column count determined at runtime from the maximum length across all rows.

```rust
#[derive(Clone, DataFrameRow)]
struct Measured {
    name: String,
    #[dataframe(expand)]       // or: #[dataframe(unnest)]
    readings: Vec<f64>,        // → readings_1, readings_2, ... (as many as needed)
}
```

- Shorter vecs: padded with `NA`.
- All elements preserved (no truncation).
- If all vecs are empty: no expansion columns produced.

`Box<[T]>` and `&[T]` work identically to `Vec<T>` for all expansion modes. Without `width` or `expand`/`unnest`, `Vec<T>`, `Box<[T]>`, and `&[T]` stay as opaque single columns (list columns in R).

**Note:** using `&[T]` introduces a lifetime parameter on both the row struct and the generated companion struct (e.g., `FooDataFrame<'a>`). This is zero-cost: `&[T]` is `Copy` (just a fat pointer), so pushing into the companion struct copies only the pointer, not the data.

### Field-level attributes

```rust
#[derive(Clone, DataFrameRow)]
struct Row {
    #[dataframe(skip)]           // Omit from data frame
    internal_id: u64,

    #[dataframe(rename = "lbl")] // Custom column name
    label: String,

    #[dataframe(as_list)]        // Suppress expansion (keep as single column)
    coords: [f64; 3],

    #[dataframe(width = 5)]      // Expand Vec to 5 columns
    scores: Vec<f64>,
}
```

| Attribute | Effect | Valid On |
|-----------|--------|----------|
| `skip` | Omit field from data frame | Any field |
| `rename = "name"` | Custom column name | Any field |
| `as_list` | Suppress expansion | `[T; N]`, `Vec<T>`, `Box<[T]>`, `&[T]` |
| `expand` | Explicit expansion (default for `[T; N]`; auto-expand for `Vec<T>`/`Box<[T]>`/`&[T]`) | `[T; N]`, `Vec<T>`, `Box<[T]>`, `&[T]` |
| `unnest` | Alias for `expand` | `[T; N]`, `Vec<T>`, `Box<[T]>`, `&[T]` |
| `width = N` | Pin expansion width (truncates longer vecs/slices) | `Vec<T>`, `Box<[T]>`, `&[T]` |

**Conflicts:** `as_list + expand`/`unnest`, `as_list + width` are compile errors.

**Round-tripping:** fixed-width and auto-expanded fields are read by regrouping
their suffixed columns. Trailing `NA` padding is removed, so a ragged `Vec<T>`
round-trips to its original length. Borrowed fields (`&[T]`) and explicitly
opaque `as_list` fields have no reader because owned R data cannot recreate the
borrow or infer the opaque element shape.

### Other collection types

Non-expanded collection fields work natively for both struct and enum `DataFrameRow`s:

```rust
use std::collections::{HashSet, BTreeSet};

#[derive(Clone, DataFrameRow)]
struct ComplexRow {
    measurements: Vec<f64>,      // opaque list column
    data: Box<[i32]>,            // opaque list column
    tags: HashSet<String>,       // opaque list column
    categories: BTreeSet<i32>,   // opaque list column
}
```

In **struct** `DataFrameRow`s the columns land as `Vec<C>` and convert to a VECSXP list-column. In **enum** `DataFrameRow`s they land as `Vec<Option<C>>` with `None` for variants that don't carry the field — these convert to a VECSXP list-column with `NULL` for absent rows. See [`CONVERSION_MATRIX.md`](CONVERSION_MATRIX.md#vecoptionc-for-collection-element-types) for the full set of supported `C`.

`HashMap<K, V>` / `BTreeMap<K, V>` variant fields expand to two parallel list-columns (see [Map fields](#map-fields-parallel-list-column-expansion) below). Struct-typed and nested-enum variant fields are covered in [Nested enum fields](#nested-enum-fields-flatten-and-opt-outs) below.

### Map fields: parallel list-column expansion

`HashMap<K, V>` and `BTreeMap<K, V>` fields on enum variants expand to two parallel list-columns named `<field>_keys` and `<field>_values`. Each cell holds a vector of K and a vector of V respectively, in the same entry order:

```rust
#[derive(Clone, DataFrameRow)]
#[dataframe(align, tag = "_type")]
enum Event {
    Tally { label: String, tally: BTreeMap<String, i32> },
    Empty { label: String },
}
// In R (BTreeMap, sorted key order):
//   _type   label  tally_keys    tally_values
//   Tally   "a"    list("a","b") list(1L, 2L)
//   Empty   "b"    NULL          NULL
```

Absent-variant rows produce `NULL` in both columns (not NA). An empty map produces `character(0)` / `integer(0)`, not `NULL`.

**HashMap ordering**: `HashMap` iteration order is non-deterministic. Keys and values are parallel within a single row, but the key order may differ across rows and runs. Use `setequal` or sort-based comparison in R tests, never `expect_equal` on unsorted key vectors.

**BTreeMap ordering**: keys are always in sorted order per the `BTreeMap` contract. `expect_equal` is safe.

**`as_list` opt-out**: annotate the field with `#[dataframe(as_list)]` to keep it as a single opaque named-list column (the pre-expansion behavior). Only use this when the named-list per-row shape is needed directly in R.

**Detection caveats**: `classify_field_type` detects `HashMap` / `BTreeMap` by matching the last path segment (`HashMap` or `BTreeMap`) and requiring exactly two generic type arguments. It also detects struct-typed fields by matching bare path types (single- or multi-segment, e.g. `Point` or `crate::geom::Point`) whose last segment has no generic arguments.

**Rejected wrapper types** — the following shapes produce a compile error (since #484) because they cannot be automatically expanded and would otherwise silently produce a confusing opaque list-column:

- `Option<T>` for a **non-scalar** `T` — `Option<HashMap<K,V>>`, `Option<UserStruct>`, `Option<Vec<T>>`, etc. (`Option<f64>`, `Option<String>` and the other scalar payloads are the NA contract and are accepted; see [Missing data](#missing-data))
- `Cow<T>`, `Rc<T>`, `Arc<T>`, `RefCell<T>`, `Cell<T>`, `Mutex<T>`, `RwLock<T>`

For all of these, use `#[dataframe(as_list)]` to opt into an explicit opaque list-column, or unwrap to the inner type (e.g. store `HashMap<K,V>` directly and use an empty map for the absent case):

```rust
#[derive(Clone, DataFrameRow)]
struct Row {
    id: i32,
    // `counts: Option<HashMap<String, i32>>` → compile error without `as_list`.
    #[dataframe(as_list)]
    counts: Option<HashMap<String, i32>>,
}
```

**Type aliases** are not automatically unwrapped — `type Counts = HashMap<String, i32>; field: Counts` has `Counts` as the last segment, so map expansion is not triggered. Use the concrete type directly (`field: HashMap<String, i32>`), or annotate with `#[dataframe(as_list)]`. See [#604](https://github.com/A2-ai/miniextendr/issues/604).

Multi-segment paths whose last segment does NOT implement `DataFrameRow` (e.g. `std::ffi::CString`) produce a clear compile-time error from the `_assert_inner_is_dataframe_row` assertion — this is intentional. Use `#[dataframe(as_list)]` on the field, or an import alias to a newtype wrapper, if a non-`DataFrameRow` stdlib type needs to be stored.

### Nested enum fields: flatten and opt-outs

A variant field whose type is itself a `DataFrameRow` enum flattens into prefixed columns by default. The inner enum must `#[derive(DataFrameRow)]`; the outer field's name acts as a prefix. The inner enum should use `#[dataframe(tag = "variant")]` so that its discriminant column merges cleanly as `<field>_variant`:

```rust
#[derive(Clone, DataFrameRow)]
#[dataframe(align, tag = "variant")]  // inner enum's own discriminant is "variant"
enum Status { Ok, Err { code: i32 } }

#[derive(Clone, DataFrameRow)]
#[dataframe(align, tag = "_type")]
enum Event {
    Tracked { id: i32, status: Status },
    Other   { id: i32 },
}
// Columns in R:
//   _type          character ("Tracked" / "Other")
//   id             integer
//   status_variant character ("Ok" / "Err" / NA for Other rows)
//   status_code    integer   (NA for Ok rows and Other rows; error code for Err rows)
```

Absent-variant rows (e.g. `Other` above, which has no `status` field) produce `NA` in all prefixed columns.

**Inner tag naming**: use `#[dataframe(tag = "variant")]` on the inner enum — the outer prefix then produces `<field>_variant` (single underscore). Using `#[dataframe(tag = "_variant")]` (with leading underscore) produces `<field>__variant` (double underscore). Avoid leading underscores on inner tags.

#### `as_factor` — unit-only inner enum

When the inner enum has only unit variants (no payload), annotate the field with `#[dataframe(as_factor)]` to emit a single R factor column instead of flattening. The inner enum does **not** need `DataFrameRow` for this path — only `UnitEnumFactor`, which is auto-emitted by `#[derive(DataFrameRow)]` for unit-only enums:

```rust
#[derive(Clone, Copy, DataFrameRow)]
#[dataframe(tag = "variant")]
enum Direction { North, South, East, West }

#[derive(Clone, DataFrameRow)]
#[dataframe(align, tag = "_type")]
enum Move {
    Step { id: i32, #[dataframe(as_factor)] dir: Direction },
    Stop { id: i32 },
}
// R column: dir — integer factor with levels c("North","South","East","West")
// Stop rows have NA in dir.
```

Factor levels are the variant idents in declaration order. `is.factor(df$dir)` returns `TRUE`. Annotating a payload-bearing enum with `as_factor` is a compile error (missing `UnitEnumFactor` implementation).

**Generic unit enums**: `#[derive(DataFrameRow)]` auto-emits `UnitEnumFactor` only when the enum has no generic type parameters (`impl_generics.is_empty()`). Generic unit enums must implement `UnitEnumFactor` manually if `as_factor` is needed.

#### `as_list` — opaque list-column

Use `#[dataframe(as_list)]` to keep any inner enum as a single opaque VECSXP list-column. Each present row gets a list cell; absent-variant rows get `NULL`:

```rust
enum Event {
    Move { id: i32, #[dataframe(as_list)] dir: Direction },
    Stop { id: i32 },
}
// R column: dir — list-column; Move rows have a list cell, Stop rows have NULL.
```

`as_list` works for any inner type (unit-only or payload-bearing, with or without `DataFrameRow`).

#### `<field>_variant` collision detection

When a field `kind: Inner` is flattened, the macro detects a compile-time collision if any sibling field in the same variant produces a column named `kind_variant` (the name that the inner enum's discriminant column will receive after prefixing). Rename the colliding field or change the inner enum's tag:

```rust
// ERROR: kind_variant is both the flatten discriminant and a sibling field name.
enum Bad {
    Wrap { kind: Inner, kind_variant: String },
}

// OK: rename sibling field, or change inner tag.
enum Good {
    Wrap { kind: Inner, #[dataframe(rename = "kind_type")] kind_type: String },
}
```

### Enum align mode

Enums build a unified schema where each variant's fields contribute columns. Fields absent in a variant are filled with `None` (→ NA in R):

```rust
#[derive(Clone, DataFrameRow)]
#[dataframe(tag = "_type")]
enum Event {
    Click { id: i64, x: f64, y: f64 },
    Impression { id: i64, slot: String },
    Error { id: i64, code: i32, message: String },
}

#[miniextendr]
fn get_events() -> BuiltDataFrame {
    let rows = vec![
        Event::Click { id: 1, x: 1.5, y: 2.5 },
        Event::Impression { id: 2, slot: "top_banner".into() },
        Event::Error { id: 3, code: 404, message: "not found".into() },
    ];
    rows.into_dataframe().unwrap()
}

// In R:
//   _type       id    x     y   slot        code  message
//   Click       1     1.5   2.5 NA          NA    NA
//   Impression  2     NA    NA  top_banner  NA    NA
//   Error       3     NA    NA  NA          404   not found
```

**Key points:**
- All enum columns are `Vec<Option<T>>` (absent fields get `None`).
- `tag = "col"` adds a variant discriminator column.
- `align` is implicit for enums (accepted but not required).
- Borrowed fields (`&'a str`, `&'a [T]`) work in enum variants — the same lifetime propagates through the companion struct. Explicit lifetime params on `#[miniextendr]` fns/impls are still rejected (MXL112); see CLAUDE.md.

#### Type conflicts across variants

If two variants use the same field name with different types, the derive fails by default. Use `conflicts = "string"` to coerce all conflicting columns to String:

```rust
#[derive(Clone, DataFrameRow)]
#[dataframe(conflicts = "string")]
enum Mixed {
    A { value: f64 },
    B { value: String },  // value column becomes String for all variants
}
```

#### Enum field attributes

All field-level attributes (`skip`, `rename`, `as_list`, `width`) work in enum variants too:

```rust
#[derive(Clone, DataFrameRow)]
#[dataframe(tag = "_type")]
enum Observation {
    Point { id: i32, coords: [f64; 2] },          // coords → coords_1, coords_2
    Measurement { id: i32, #[dataframe(width = 3)] readings: Vec<f64> },
}
```

### Enum split mode (`into_dataframe_split`)

Alongside the aligned form (`into_dataframe()`, which produces a single data frame with `NA`/`NULL` fill for variants that don't carry a field), enums also get `rows.into_dataframe_split()` — the `IntoDataFrameSplit` trait, re-exported from `miniextendr_api::prelude` next to `IntoDataFrame`/`FromDataFrame` — which partitions the rows by variant. Each partition is a data frame with **only that variant's own columns** — no `NA`-filled columns from sibling variants. Struct rows have no variants to partition, so the trait is implemented only for enum-derived `Vec<Row>`. It returns an `miniextendr_api::List` (a bare `data.frame` for a single-variant enum, a named list otherwise), so a `#[miniextendr]` function returns `List`:

| Variants × rows in input | R return |
|--------------------------|----------|
| **Single-variant enum**, any number of rows | bare `data.frame` |
| **Multi-variant enum**, mixed rows | named `list` of data frames, one per variant in `snake_case` |

```rust
use miniextendr_api::{IntoDataFrameSplit, List};

#[miniextendr]
fn split_events() -> List {
    let rows = vec![
        Event::Click { id: 1, x: 1.5, y: 2.5 },
        Event::Impression { id: 2, slot: "top_banner".into() },
        Event::Error { id: 3, code: 404, message: "not found".into() },
    ];
    rows.into_dataframe_split()
    // In R: list(click = <1-row df with id, x, y>,
    //            impression = <1-row df with id, slot>,
    //            error = <1-row df with id, code, message>)
}
```

Variants absent from the input still appear in the result as 0-row data frames carrying that variant's column shape. Unit variants produce a 0-column data frame with the correct row count. Tuple variants name positional columns `_0`, `_1`, … . See the cardinality matrix in `rpkg/tests/testthat/test-dataframe-enum-payload-matrix.R` for the full set of guarantees (PR #463).

### Container attributes

```rust
#[derive(DataFrameRow)]
#[dataframe(
    name = "Measurements",     // Custom companion type name (default: {StructName}DataFrame)
    tag = "_type",             // Add variant discriminator column (enums)
    align,                     // Unified schema with NA fill (implicit for enums)
    conflicts = "string",      // Coerce type conflicts to String (enums)
)]
struct Measurement { /* ... */ }
```

### Requirements

A struct row type must implement `IntoList`:

- Automatically via `#[derive(IntoList)]`.
- Via `#[derive(Serialize)]` when the `serde` feature is enabled (`Serialize` implies `IntoList`).
- Via a manual `impl IntoList` using `List::from_raw_pairs()` (for heterogeneous fields).

Enum `DataFrameRow`s generate their own `IntoList`; you don't add it separately.

## Reading data frames back (`FromDataFrame`)

`Vec::<Row>::from_dataframe(&df)?` transposes columns back into rows:

```rust
let rows: Vec<Measurement> = Vec::<Measurement>::from_dataframe(&df)?;
for m in &rows {
    println!("time {} value {}", m.time, m.value);
}
```

The reader covers scalar fields, fixed/auto expansion, nested struct flattening,
tagged enums (including flattened payload enums and `as_factor` unit enums),
and enum map columns. Shapes that intentionally discard reconstruction
information still return `DataFrameError::Conversion`: borrowed or `skip`
fields, `as_list` and other opaque non-scalar collection columns,
`#[dataframe(tag)]` structs, conflict-string enums, and tagless/skip-field
enums. See [The DataFrame interface](DATAFRAME_INTERFACE.md) for the exact
contract.

## Group-level iteration

Two rungs, cheapest first.

### Typed rows: `group_rows`

Once rows are extracted, grouping is plain Rust. `group_rows` makes the idiom
discoverable and gives NA-able keys a defined home (key on `Option<T>` —
`None` sorts first — or a custom enum):

```rust
let rows: Vec<Measurement> = Vec::<Measurement>::from_dataframe(&df)?;
let by_site: BTreeMap<String, Vec<Measurement>> =
    group_rows(rows, |m| m.site.clone());
```

The result is plain Rust data — no SEXP contact — so stepping over groups with
rayon is safe:

```rust
let summaries: Vec<(String, f64)> = by_site
    .into_iter()
    .par_bridge()                       // rayon: rows are Send, no SEXPs held
    .map(|(site, ms)| (site, ms.iter().map(|m| m.value).sum()))
    .collect();
```

### Untyped frames: `DataFrame::group_by`

For heterogeneous frames where extracting typed rows is wasteful,
`group_by(col)` computes group indices in one pass (main thread, no row
copies) and returns a `GroupedDataFrame`:

```rust
let grouped = df.group_by("site")?;

// (key, row-indices) — zero-copy
for (key, rows) in grouped.iter() { /* key: &GroupKey, rows: &[usize] */ }

// per-group sub-frames — root each one as you go (push protects it)
let mut out = NamedDataFrameListBuilder::with_capacity(grouped.len());
for (key, sub) in grouped.frames() {
    out = out.push(key.label(), *sub);
}
let named_list_of_frames = out.build();

// or: one typed extraction, then an index move-partition (rayon-safe after)
let by_group: Vec<(GroupKey, Vec<Measurement>)> = grouped.extract()?;
```

Supported key columns: **factor** (fast path — levels are the keys, level
order kept, empty levels included, like `split()`), **character** (byte-order
sort), **integer** (numeric sort), **logical** (`FALSE`, `TRUE`). **Double
columns error** — grouping on floating point is a footgun; `cut()` or
`factor()` the column in R first.

**NA keys form one group, ordered last.** This deliberately deviates from R's
`split()`, which silently drops NA-keyed rows.

### Composite keys: `DataFrame::group_by_multi`

`group_by_multi(&["a", "b"])` groups on several columns at once, keying each
group by a `GroupKey::Tuple` of the per-column scalar keys (same supported types
and errors as `group_by`):

```rust
let grouped = df.group_by_multi(&["site", "day"])?;
for (key, sub) in grouped.frames() {
    out = out.push(key.label(), *sub); // label(): "site.day", "." separator
}
```

Non-NA groups match `split(df, interaction(a, b, …, drop = TRUE))` — the first
column varies fastest (R `interaction()`'s default, `lex.order = FALSE`), each
column ordered as `group_by` would order it alone. For character keys the match
is exact for keys whose byte order coincides with the session's collation —
always true in the C locale; single-case ASCII in practice (`en_US.UTF-8`
collates `a A b B` where byte order gives `A B a b`). This is inherited from
`group_by`'s byte-order sort for character keys (see above). Labels join
element labels with `.` (interaction's default separator).

**NA extends the single-column convention.** `interaction()` maps any row with
an NA in *any* component to NA and `split()` drops it; `group_by_multi` instead
keeps such rows, forming one trailing group per distinct NA-containing tuple, in
first-encounter row order. An **empty** column slice is an error; a
**single-column** slice delegates to `group_by` and yields scalar keys (never
1-tuples), so downstream matches on `GroupKey` never have to special-case them.

### Honoring a dplyr `grouped_df`: `DataFrame::group_by_metadata`

`group_by` / `group_by_multi` **compute** grouping from your chosen columns.
When a `#[miniextendr]` function receives a frame that dplyr has **already**
grouped (a `grouped_df`), `group_by_metadata()` reads that existing grouping —
**respecting the caller's grouping instead of recomputing it**:

```rust
// df arrived as dplyr::group_by(data, site, day)
let grouped = df.group_by_metadata()?;
for (key, sub) in grouped.frames() {
    out = out.push(key.label(), *sub); // one entry per dplyr group, in dplyr order
}
```

dplyr stores the grouping in `attr(df, "groups")`: a data frame whose leading
columns are the group-key columns (one row per group, in dplyr's order) and
whose trailing `.rows` list-column holds each group's **1-based** row indices.
`group_by_metadata` ingests that verbatim into the same `GroupedDataFrame` the
compute verbs return — single key column → scalar `GroupKey`, multiple → a
`.`-joined `GroupKey::Tuple` (same supported key types as `group_by`). It
preserves the `groups`-frame order exactly (no re-sorting, no NA reordering),
converts every `.rows` index from 1-based to 0-based, and **keeps** `.drop =
FALSE` empty groups as groups with empty index vectors (mirroring `group_by`'s
retention of empty factor levels).

Unlike `group_by`, this does **no** recomputation: a plain (non-grouped) frame
is a `NotGroupedDataFrame` error — callers who want the framework to compute
grouping use `group_by` / `group_by_multi`. Other errors: `MissingGroupRows`
(the `groups` frame lacks a `.rows` column), `BadGroupRows` (a `.rows` element
is not an integer/integerish vector), and `GroupIndexOutOfRange` (a `.rows`
index is `< 1` or `> nrow`).

Degenerate edge: a `groups` frame with **zero** key columns (e.g. a dplyr
`rowwise_df`, whose `groups` frame is just `.rows`) yields empty-`Tuple` keys
with empty labels — that shape is outside the documented `grouped_df` scope, so
don't pass rowwise frames here.

## Parallel fast paths (`feature = "rayon"`)

Explicit `_par` variants produce the **same** `BuiltDataFrame` / `Vec<Row>` as the sequential verbs — parallelism is an opt-in method, not a hidden threshold:

```rust
let df   = rows.into_dataframe_par()?;             // parallel (column, row-range) fill
let rows = Vec::<Measurement>::from_dataframe_par(&df)?; // off-main-thread row assembly
```

Dropping `_par` (building without the `rayon` feature) degrades cleanly to the sequential call — the verb name is stable across feature sets.

```rust
#[miniextendr]
fn big_points() -> BuiltDataFrame {
    let points: Vec<Point> = (0..100_000)
        .map(|i| Point { x: i as f64, y: (i * 2) as f64 })
        .collect();
    points.into_dataframe_par().unwrap()  // explicit parallel fill
}
```

Parallel fill is most beneficial for large row counts (10k+), wide data frames (many fields), or expensive per-field conversions. For small data frames, prefer the sequential `into_dataframe()` to avoid rayon overhead.

## Heterogeneous columns without a row type

When you are filling columns directly (not transposing a `Vec<Row>`), use `DataFrame::builder(nrow)`. `column::<T>` takes a native element type (`f64` / `i32` / `RLogical` / `u8` / `Rcomplex`) and a chunk-fill closure; `column_str` builds a character column from a per-row closure returning `Option<String>` (`None` → `NA_character_`). `build()` yields a rooted `BuiltDataFrame`:

```rust
let df = DataFrame::builder(nrow)
    .column::<f64>("x", |chunk, offset| {
        for (i, slot) in chunk.iter_mut().enumerate() {
            *slot = (offset + i) as f64;
        }
    })
    .column_str("label", |i| Some(format!("p{i}")))
    .build();
```

The builder is always available. With the `rayon` feature, column buffers are
filled in parallel over disjoint row ranges; without it, each column is filled
serially. Both paths assemble the same rooted `data.frame` on the R thread.

## serde rows

Types that derive `serde::Serialize` / `Deserialize` convert through the `SerdeRows<T>` newtype, which keeps the serde path from colliding with the derive's concrete `Vec<Row>` conversions:

```rust
use miniextendr_api::serde::SerdeRows;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct LogEntry {
    timestamp: f64,
    level: String,
    message: String,
}

#[miniextendr]
fn get_logs() -> BuiltDataFrame {
    let logs = vec![
        LogEntry { timestamp: 1.0, level: "INFO".into(), message: "started".into() },
        LogEntry { timestamp: 2.0, level: "ERROR".into(), message: "failed".into() },
    ];
    SerdeRows(logs).into_dataframe().unwrap()
}

// Reading back:
//   let logs = SerdeRows::<LogEntry>::from_dataframe(&df)?.into_inner();
```

Column types are inferred from serde field types:

| Rust type | R column |
|-----------|----------|
| `bool` | logical |
| `i8` / `i16` / `i32` | integer |
| `i64` / `u64` / `f32` / `f64` | numeric |
| `String` / `&str` | character |
| `Option<T>` | same type, `NA` for `None` |

This is useful when you already have serde-serializable types and don't want to add `IntoList` + `DataFrameRow` derives. For new types, prefer `#[derive(DataFrameRow)]`, which gives you a typed companion type and the reverse-direction reader.

Requires the `serde` feature.

## Missing data

Use `Option<T>` for nullable **scalar** fields. `None` becomes the typed `NA`
in R (`NA_real_`, `NA_integer_`, `NA_character_`, `NA`), and `NA` reads back
as `None`; the column type is unchanged, so an all-`None` column is still a
double / character / logical / integer vector:

```rust
#[derive(Clone, IntoList, DataFrameRow)]
struct Record {
    id: i32,
    value: Option<f64>,  // NA in R when None
}
```

Accepted payloads: `bool`, `f32`, `f64`, `i8`..`i64`, `u8`..`u64`, `isize`,
`usize`, `String` (the reader covers the same set as bare scalars: `u64`,
`isize`, `usize` are write-only). `Option` around a map, a nested
`DataFrameRow` struct, or a collection is rejected at compile time (see
[Rejected wrapper types](#rejected-wrapper-types)); in **enum** row types the
variant fields are already `Option` columns (absent for other variants), so
write the bare scalar there.

## `DataFrameError`

A single error type covers every failure mode of both verbs:

| Variant | Meaning |
|---------|---------|
| `NotList(msg)` | The SEXP is not a VECSXP. |
| `NotDataFrame` | The object does not inherit from `data.frame`. |
| `NoNames` | The list has no `names` attribute (columns must be named). |
| `BadRowNames(msg)` | Could not extract `nrow` from the `row.names` attribute. |
| `UnequalLengths { expected, column, actual }` | Columns have unequal lengths. |
| `UnnamedColumns` | A row could not be turned into named columns. |
| `NoSuchColumn(name)` | `group_by` referenced a column name that does not exist. |
| `EmptyGroupColumns` | `group_by_multi` was called with an empty column slice. |
| `UnsupportedGroupColumn { column, type_of }` | A key column has no grouping semantics (double, list-column, …). |
| `NotGroupedDataFrame` | `group_by_metadata` on a frame with no dplyr `groups` metadata. |
| `MissingGroupRows` | The `groups` frame has no `.rows` list-column. |
| `BadGroupRows { group, type_of }` | A `.rows` element is not an integer/integerish index vector. |
| `GroupIndexOutOfRange { group, value, nrow }` | A `.rows` index is `< 1` or `> nrow`. |
| `Conversion(msg)` | A serde or other conversion failure, carried as a message (also covers "this shape has no reader"). |

It implements `std::error::Error` and `From<RSerdeError>`, so `?` works in functions that mix serde and data-frame conversions.

## Migration from the legacy surface

The redundant public types below were **removed** (#781) — there is no backwards-compat shim. If you have older code, map it to the façade:

| Was | Now |
|---|---|
| `DataFrameView`, `convert::DataFrame<T>` | `DataFrame` view + `BuiltDataFrame` owner |
| `DataFrame::from_rows(rows)` (typed row buffer) | `rows.into_dataframe()?` |
| `ToDataFrame<Companion>` wrapper + `value.to_data_frame()` | `rows.into_dataframe()?` |
| `convert::SerializeDataFrame<T>` / `AsSerializeRow<T>` / `from_serialize()` | `serde::SerdeRows(rows).into_dataframe()?` |
| `impl IntoDataFrame for X { fn into_data_frame(self) -> List }` | derive `DataFrameRow`, or build a `BuiltDataFrame` via `DataFrame::builder(n)` |
| `Row::try_from_dataframe(sexp)` (bare `String` error) | `Vec::<Row>::from_dataframe(&df)?` (`DataFrameError`) |
| `RDataFrameBuilder::new(n)` | `DataFrame::builder(n)` |
| four conversion error types | one `DataFrameError` |

The companion type that `#[derive(DataFrameRow)]` generates (`{Name}DataFrame`,
with `to_dataframe` / `from_rows` / `from_rows_par` / `from_dataframe` and
`IntoIterator`) still exists as the engine the façade verbs delegate to. The
serde columnar path (`serde::vec_to_dataframe` and friends) produces the same
`BuiltDataFrame` handle — there is no separate `ColumnarDataFrame` type; the
naming convergence was completed in #783.

## Feature flags

- **Base functionality**: no features required.
- **`serde`**: enables `impl IntoList for T: Serialize`, the `SerdeRows<T>` wrapper, and `#[derive(Serialize, DataFrameRow)]`.
- **`rayon`**: enables the `_par` verbs and parallelizes `DataFrame::builder`;
  the builder itself remains available without this feature.

## Examples

See [rpkg/src/rust/dataframe_examples.rs](../rpkg/src/rust/dataframe_examples.rs) for complete working examples.
