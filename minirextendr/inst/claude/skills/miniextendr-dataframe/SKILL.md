---
name: miniextendr-dataframe
description: Use when moving data.frames between R and Rust in a miniextendr package — #[derive(DataFrameRow)], converting Vec<Row> to a data.frame and back, the DataFrame type, column builders, serde-based rows, enum/tagged rows, list-columns, or NA cells in tabular data.
---

# data.frames ↔ Rust rows

miniextendr has one owned `DataFrame` type and two traits that mirror the
scalar conversion surface:

| Trait | Call | Direction |
|---|---|---|
| `IntoDataFrame` | `rows.into_dataframe()?` | `Vec<Row>` → R data.frame |
| `FromDataFrame` | `Vec::<Row>::from_dataframe(&df)?` | R data.frame → `Vec<Row>` |

Both verbs live on the data itself. All errors are one type,
`DataFrameError`. Missing cells round-trip as `Option<T>` fields for scalar
`T` (`Option<f64>`, `Option<String>`, `Option<bool>`, `Option<i32>`, …):
`None` is the typed `NA` and `NA` reads back as `None`. That is the entire
NA contract for struct row types.

## The 90% case: derive a row type

```rust
use miniextendr_api::dataframe::{BuiltDataFrame, DataFrame, FromDataFrame, IntoDataFrame};
use miniextendr_api::{miniextendr, DataFrameRow, IntoList};

#[derive(Clone, IntoList, DataFrameRow)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub label: String,
    pub weight: Option<f64>,   // NA-able column
}

// Take an R-supplied frame as the cheap `DataFrame` view; return the freshly
// built frame as `BuiltDataFrame` (an owned, GC-rooted handle — the return type
// of every Rust-side constructor like `into_dataframe`). It Derefs to
// `DataFrame` and converts back to R automatically.
#[miniextendr]
pub fn shift_points(df: DataFrame, dx: f64) -> BuiltDataFrame {
    let mut rows = Vec::<Point>::from_dataframe(&df).unwrap();
    for p in &mut rows {
        p.x += dx;
    }
    rows.into_dataframe().unwrap()
}
```

```r
df <- data.frame(x = 1:3, y = 4:6, label = c("a","b","c"), weight = c(1, NA, 3))
shift_points(df, 10)
```

`DataFrame` implements the ordinary conversion traits, so it appears directly
in `#[miniextendr]` signatures. It wraps a *validated* `data.frame` SEXP —
handing it a list that isn't a data.frame errors cleanly.

## Field shapes the derive understands

- **Scalars** (`f64`, `i32`, `String`, `bool`, …) → one column each;
  `Option<scalar>` for NA-able columns. `Option` around a map, a nested
  struct or a collection is a compile error: use `#[dataframe(as_list)]`
  or store the inner type. Enum variant fields are already `Option`
  columns (absent for other variants), so write the bare scalar there.
- **Fixed arrays** `[T; N]` and `Vec<T>` + `#[dataframe(width = N)]` →
  expanded to `field_1 … field_N` columns (ragged `Vec` pads trailing `NA`,
  and round-trips losslessly).
- **Nested structs** deriving `DataFrameRow` → flattened with a
  `<field>_` prefix, arbitrarily deep.
- **Un-annotated `Vec<scalar>` / `Box<[scalar]>`** → an opaque list-column
  (one R list element per row).
- **`HashMap`/`BTreeMap`** (scalar keys/values) → paired
  `<field>_keys` / `<field>_values` list-columns, zipped back on read.
- **`#[dataframe(skip)]`** → field not written (and therefore not readable
  back — `from_dataframe` on such a shape returns an error, not garbage).

**Tagged enums** work as row types too: give the enum
`#[dataframe(tag = "kind")]` and each variant's fields become columns, with
the tag column dispatching per-row on the way back in. Unit-only nested enums
can render as factors with `#[dataframe(as_factor)]`.

A few shapes are write-only (reading back returns a clear `DataFrameError`):
borrowed fields (`&str`/`&[T]`), skipped fields, opaque non-scalar
collections, and tagless enums. If `from_dataframe` errors on your type,
check the field shapes first.

## Filling columns directly (no row type)

For heterogeneous frames built column-wise, use the builder:

```rust
let df = DataFrame::builder(nrow)
    .column::<f64>("x", |chunk, offset| {
        for (i, slot) in chunk.iter_mut().enumerate() {
            *slot = (offset + i) as f64;
        }
    })
    .column_str("label", |i| Some(format!("row{i}")))
    .build();
```

The builder fills serially by default and in parallel when the crate is built
with the `rayon` feature — same API either way.

## Parallel conversions (`rayon` feature)

`_par` variants produce identical results, just faster on large data:

```rust
let df   = rows.into_dataframe_par()?;
let rows = Vec::<Point>::from_dataframe_par(&df)?;
```

Row structs are plain Rust data (no R pointers), so after extraction you can
process them with rayon freely: `rows.par_iter().map(...)`. See the
`miniextendr-parallel` skill for the threading rules.

## serde rows (when you already derive Serialize)

Types with `serde::Serialize`/`Deserialize` can skip `DataFrameRow` and
convert through the `SerdeRows` newtype:

```rust
use miniextendr_api::serde::SerdeRows;

let df   = SerdeRows(rows).into_dataframe()?;
let rows = SerdeRows::<Row>::from_dataframe(&df)?.into_inner();
```

There are also free functions `vec_to_dataframe(...)` and
`vec_to_dataframe_flatten_enums(...)` (flattens nested enum fields into
columns) in `miniextendr_api::serde`. Prefer the `DataFrameRow` derive when
you control the type — it is checked at compile time and reads back; the
serde path shines for third-party types you can't annotate.

## Pitfalls

- **All-`None` columns land as logical NA columns** (that's R's convention
  for "unknown type, all missing"); R coerces them to the right type on first
  combine. Mixed Some/None columns are unaffected.
- **Column order for `HashMap` fields is non-deterministic** (use `BTreeMap`
  for byte-stable output); key→value pairing is always preserved.
- **factors**: a factor column arrives as its integer codes through the
  scalar path. Convert at the boundary (`as.character()` in R) or model it as
  a unit enum + `as_factor` on the Rust side.
- **Don't hold the `DataFrame` across long computations** if you can extract
  rows instead — rows are plain Rust data and are safe on any thread; the
  `DataFrame` itself wraps R memory and must stay on the main thread.
- **One error type**: match on `DataFrameError` for diagnostics; it reports
  the offending column and row where applicable.

Full manual (DataFrame chapters): https://a2-ai.github.io/miniextendr
