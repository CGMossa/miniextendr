//! Columnar serializer for converting `Vec<T>` directly to R data.frames.
//!
//! Instead of serializing each struct as a named list (row-oriented), this
//! module transposes the data into column-oriented R vectors — one atomic
//! vector per field. This is more memory-efficient and produces native R
//! data.frames directly.
//!
//! Nested structs are recursively flattened into prefixed columns
//! (e.g., `metadata_size`). `#[serde(flatten)]` and `#[serde(skip_serializing_if)]`
//! are handled correctly.

use std::collections::HashMap;

use super::error::RSerdeError;
use crate::altrep_traits::{NA_LOGICAL, NA_REAL};
use crate::dataframe::{BuiltDataFrame, NamedDataFrameListBuilder};
use crate::{SEXP, SEXPTYPE, SexpExt};
use serde::ser::{self, Serialize};

/// Generate serde `Serializer` error stubs for methods that should reject non-struct input.
/// Accepts `struct`/`map` to allow, and an error message for the rest.
macro_rules! reject_non_struct {
    ($msg:expr, allow_some_none) => {
        fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), RSerdeError> {
            value.serialize(self)
        }
        fn serialize_none(self) -> Result<(), RSerdeError> {
            Err(RSerdeError::Message(concat!($msg, " (got None)").into()))
        }
        reject_non_struct!(@primitives $msg);
    };
    ($msg:expr) => {
        fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<(), RSerdeError> {
            Err(RSerdeError::Message($msg.into()))
        }
        fn serialize_none(self) -> Result<(), RSerdeError> {
            Err(RSerdeError::Message($msg.into()))
        }
        reject_non_struct!(@primitives $msg);
    };
    (@primitives $msg:expr) => {
        fn serialize_bool(self, _: bool) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_i8(self, _: i8) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_i16(self, _: i16) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_i32(self, _: i32) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_i64(self, _: i64) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_u8(self, _: u8) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_u16(self, _: u16) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_u32(self, _: u32) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_u64(self, _: u64) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_f32(self, _: f32) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_f64(self, _: f64) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_char(self, _: char) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_str(self, _: &str) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_bytes(self, _: &[u8]) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_unit(self) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_unit_struct(self, _: &'static str) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _: &'static str, _: &T) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_newtype_variant<T: ?Sized + Serialize>(self, _: &'static str, _: u32, _: &'static str, _: &T) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeTupleStruct, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_tuple_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeTupleVariant, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_struct_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeStructVariant, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
    };
}

/// Convert a slice of serializable structs to an R
/// [`DataFrame`](crate::dataframe::DataFrame) in columnar layout.
///
/// Each field of `T` becomes a column (R atomic vector). Nested structs are
/// recursively flattened into prefixed columns (`parent_child` naming).
///
/// This is the serde column path's `Rust → R` entry point. It produces the same
/// rooted [`BuiltDataFrame`] the rest of the
/// unified interface uses (the same type returned by
/// [`IntoDataFrame`](crate::dataframe::IntoDataFrame)), so the result supports
/// post-assembly editing — every link of the chain is GC-rooted (#1247):
///
/// ```ignore
/// vec_to_dataframe(&rows)?
///     .rename("hashes_blake3", "hash")
///     .with_column("status", status_sexp)
///     .drop("internal_id")
/// ```
///
/// # Supported Field Types
///
/// | Rust Type | R Column Type |
/// |-----------|---------------|
/// | `bool` | `logical` |
/// | `i8/i16/i32` | `integer` |
/// | `i64/u64/f32/f64` | `numeric` |
/// | `String/&str` | `character` |
/// | `Option<T>` | Same type with NA for `None` |
/// | `Option<T>` (every row `None`) | `logical` NA column — R coerces to the surrounding type on first use (`c(NA, 1L)` → integer, `c(NA, "x")` → character) |
/// | Nested struct | Recursively flattened with `parent_child` naming |
/// | Nested struct's `Option<T>` sub-field | Column type is the union across **all** rows, independent of row order — a sub-field that is `None` on the first row (or first several rows) but typed on a later one still gets that atomic type (`character`, `integer`, …), not a `list` column. Only if the sub-field is `None` in *every* row does it fall back to the `logical` NA downgrade above (nothing to union from). |
/// | Other | Falls back to per-element list column |
pub fn vec_to_dataframe<T: Serialize>(rows: &[T]) -> Result<BuiltDataFrame, RSerdeError> {
    if rows.is_empty() {
        // SAFETY: `empty_dataframe` returns a well-formed 0-row data.frame SEXP.
        return Ok(unsafe { BuiltDataFrame::adopt_sexp(empty_dataframe()) });
    }

    // Phase 1: Discover schema from ALL rows (union of fields across enum variants)
    let mut acc = SchemaAccumulator::new(SchemaMode::Union);
    for row in rows {
        // Skip rows that fail discovery (e.g., top-level None) — Union mode tolerates partial probes.
        let _ = acc.feed(row);
    }
    let schema = acc.finalize()?;
    let ncol = schema.fields.len();
    let nrow = rows.len();

    if ncol == 0 {
        return Err(RSerdeError::Message(
            "vec_to_dataframe: type has no fields".into(),
        ));
    }

    // Phase 2: Allocate column buffers
    let mut columns: Vec<ColumnBuffer> = schema
        .fields
        .iter()
        .map(|f| ColumnBuffer::new(f.col_type, nrow))
        .collect();

    // Roots every SEXP that lands in a `ColumnBuffer::Generic` (via
    // `RSerializer::serialize`) for the duration of this call. Without
    // this, the unprotected SEXPs in the generic-list buffer can be
    // GC'd before `assemble_dataframe` reads them — surfaced under
    // `gctorture(TRUE)` and on glibc-strict R runtimes.
    let scope = unsafe { crate::ProtectScope::new() };

    // Phase 3: Fill columns from all rows
    let mut filled = vec![false; ncol];
    for row in rows {
        let filler = ColumnFiller {
            columns: &mut columns,
            field_map: &schema.field_map,
            filled: &mut filled,
            col_start: 0,
            col_count: ncol,
            is_top_level: true,
            pending_key: None,
            scope: &scope,
            strict: false,
        };
        row.serialize(filler)?;
    }

    // Phase 4: Assemble data.frame. `assemble_with_scope` sequences
    // `assemble_dataframe(...)` followed by `drop(scope)` so the
    // protected SEXPs in `ColumnBuffer::Generic` cells are still rooted
    // while `set_vector_elt` copies them into the parent VECSXP.
    Ok(unsafe { assemble_with_scope(&schema, &columns, nrow, scope) })
}

// region: Streaming serialize (iter_to_dataframe + SerdeRowBuilder)

/// Stream rows from an iterator into a columnar data.frame.
///
/// Schema is taken from the **first row**; subsequent rows must match that
/// schema. If a later row introduces a field not seen in the first row,
/// returns [`RSerdeError::Message`]. Fields present in the first row but
/// missing from a later row are NA-padded.
///
/// `nrow_hint` lets callers pre-size column buffers; `None` is fine — buffers
/// grow exponentially via `Vec::push`.
///
/// # When to use this vs [`vec_to_dataframe`]
///
/// Use [`vec_to_dataframe`] when you already have a `&[T]` in memory. Use
/// `iter_to_dataframe` when the rows arrive incrementally (a file iterator,
/// a DB cursor, a generator) — materialising into a `Vec` first would defeat
/// the purpose.
///
/// # Errors
///
/// - A row fails to serialize.
/// - A row introduces a field not present in the first row's schema.
/// - Column assembly fails.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(serde::Serialize)]
/// struct Row { id: i32, name: String }
///
/// let rows = (0..10).map(|i| Row { id: i, name: format!("item_{i}") });
/// let df = iter_to_dataframe(rows, Some(10))?;
/// ```
pub fn iter_to_dataframe<T, I>(
    iter: I,
    nrow_hint: Option<usize>,
) -> Result<BuiltDataFrame, RSerdeError>
where
    T: Serialize,
    I: IntoIterator<Item = T>,
{
    let mut builder = SerdeRowBuilder::<T>::new(nrow_hint);
    for row in iter {
        builder.push(row)?;
    }
    builder.finish()
}

/// Parallel counterpart to [`iter_to_dataframe`]: fan row→column serialisation
/// out across rayon for CPU-bound row work.
///
/// # Strategy: per-thread scratch + merge
///
/// 1. **Materialise + discover** (main thread). The iterator is collected into
///    a `Vec<T>` and the schema is discovered from the **first row** — the same
///    homogeneous-schema contract as [`iter_to_dataframe`]. The collection is
///    necessary: rayon needs an indexed source to split into ordered chunks,
///    and the schema must be shared by every worker.
/// 2. **Fan out** (worker threads, zero R API). Rows are split into contiguous
///    chunks; each worker fills a *local* `Vec<ColumnBuffer>` against the shared
///    schema using pure-Rust serde extraction — no SEXP allocation, no
///    `ProtectScope`, no R main-thread contact. This mirrors the invariant of
///    the row-oriented serde paths: the parallel region touches only Rust data.
/// 3. **Merge in row order** (main thread). Chunk results come back ordered
///    (rayon's [`collect`](rayon::iter::ParallelIterator::collect) on an `IndexedParallelIterator` preserves index order), so
///    concatenating each chunk's column buffers reproduces the original row
///    order exactly. The merged buffers are assembled into a [`DataFrame`](crate::dataframe::DataFrame).
///
/// # Schema scope (homogeneous only)
///
/// Like [`iter_to_dataframe`], the schema is fixed from the first row. Two
/// shapes are **rejected** here because they need the R main thread or a
/// reconciliation step that defeats the per-thread-merge model:
///
/// - **Generic (list) columns** — a column whose values serialise to arbitrary
///   SEXPs (the `Generic` fallback) must allocate on the R main thread. Such a
///   schema returns an error pointing back to [`iter_to_dataframe`].
/// - **Growing / heterogeneous schema** — rows that introduce new fields under
///   parallelism would produce divergent per-thread schemas needing a union
///   merge. Out of scope for this homogeneous variant; use
///   [`par_iter_to_dataframe_growing`] (union-schema, still parallel) or
///   [`SerdeRowBuilder::grow_schema`] sequentially.
///
/// # Equivalence
///
/// For any input whose schema is fully atomic (no `Generic` column), the result
/// is identical column-for-column and row-for-row to
/// `iter_to_dataframe(rows, nrow_hint)`.
///
/// # Errors
///
/// - A row fails to serialize.
/// - A row introduces a field not present in the first row's schema.
/// - The discovered schema contains a `Generic` (list) column.
/// - Column assembly fails.
///
/// # Example
///
/// ```rust,ignore
/// use rayon::prelude::*;
///
/// #[derive(serde::Serialize)]
/// struct Row { id: i32, name: String }
///
/// let rows: Vec<Row> = (0..10_000)
///     .map(|i| Row { id: i, name: format!("item_{i}") })
///     .collect();
/// let df = par_iter_to_dataframe(rows, Some(10_000))?;
/// ```
#[cfg(feature = "rayon")]
pub fn par_iter_to_dataframe<T, I>(
    iter: I,
    nrow_hint: Option<usize>,
) -> Result<BuiltDataFrame, RSerdeError>
where
    // `Sync` (not just `Send`): rows are borrowed across rayon workers via
    // `par_chunks`, so `&T: Send`, i.e. `T: Sync`.
    T: Serialize + Send + Sync,
    I: IntoIterator<Item = T>,
{
    crate::optionals::parallel::ensure_pool();

    // 1. Materialise. rayon needs an indexed source for ordered chunking.
    let rows: Vec<T> = iter.into_iter().collect();
    let Some((schema, merged, nrow)) = par_build_columns(&rows, nrow_hint)? else {
        // Empty input: well-formed 0-row data.frame.
        // SAFETY: `empty_dataframe` returns a well-formed 0-row data.frame SEXP.
        return Ok(unsafe { BuiltDataFrame::adopt_sexp(empty_dataframe()) });
    };

    // Assembly allocates SEXPs and must run on the R main thread. The merged
    // buffers are pure Rust at this point; a fresh ProtectScope guards the
    // assembly's own allocations (no Generic cells are held — rejected by
    // `par_build_columns`).
    // SAFETY: caller invokes this on the R main thread; column lengths all
    // equal `nrow` (each row contributed exactly one element per column).
    let scope = unsafe { crate::ProtectScope::new() };
    Ok(unsafe { assemble_with_scope(&schema, &merged, nrow, scope) })
}

/// Parallel, growing-schema counterpart to [`par_iter_to_dataframe`]: the
/// rayon-backed analogue of [`vec_to_dataframe`]'s union-schema path (#936).
///
/// Where [`par_iter_to_dataframe`] fixes the schema from the first row and
/// rejects rows that introduce new fields, this variant computes the **union**
/// of fields across *all* rows — rows may freely introduce fields the others
/// lack (heterogeneous structs via untagged enums, maps with divergent keys).
/// Fields a row doesn't carry are NA-padded, exactly like
/// [`vec_to_dataframe`].
///
/// # How it stays parallel
///
/// Divergent per-thread schemas are the hard part of a growing schema under
/// fan-out: one worker may discover field `x` while another discovers `y`,
/// and neither sees the other's columns. Instead of reconciling per-thread
/// column sets after the fact (re-indexing + NA back-fill per chunk), the
/// build separates discovery from fill:
///
/// 1. **Parallel union discovery.** Each worker probes its chunk's rows into a
///    local `SchemaAccumulator` (pure Rust, no R contact).
/// 2. **Global resolution** (main thread, cheap). The per-chunk accumulators
///    are merged in chunk (= row) order and resolved through the same
///    candidate lattice as [`vec_to_dataframe`] — so cross-chunk type clashes
///    behave identically to the sequential union path: first-seen wins,
///    a typed probe beats a None-only (`Generic`) probe regardless of which
///    chunk saw it.
/// 3. **Parallel fill against the shared union schema.** Identical to the
///    homogeneous fan-out: every worker fills a local column set with one slot
///    per union field, NA-padding fields its rows lack. The ordered merge then
///    needs no reconciliation at all.
///
/// # Schema scope
///
/// **Generic (list) columns are rejected**, same as [`par_iter_to_dataframe`]:
/// per-cell SEXP allocation needs the R main thread. This includes the
/// all-`None` case — a field that is `None` in *every* row resolves to
/// `Generic` (nothing typed it) and is rejected here, where
/// [`vec_to_dataframe`] would downgrade it to a logical-NA column at assembly.
/// Fall back to [`vec_to_dataframe`] for such schemas.
///
/// # Equivalence
///
/// For any input whose union schema is fully atomic (no `Generic` column), the
/// result is identical column-for-column and row-for-row to
/// `vec_to_dataframe(&rows)`. Note this is the *union* semantics — it differs
/// from sequential [`SerdeRowBuilder::grow_schema`], which types each column
/// from its first-seen probe only (an early all-`None` window can lock a
/// column to `Generic` there; the union path resolves it from any later row).
///
/// # Errors
///
/// - A row fails to serialize during the fill pass.
/// - No row yields any fields (e.g. every row is a top-level `None`).
/// - The union schema contains a `Generic` (list) column.
/// - Column assembly fails.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(serde::Serialize)]
/// #[serde(untagged)]
/// enum Row {
///     Old { id: i32 },
///     New { id: i32, score: f64 },
/// }
///
/// let rows: Vec<Row> = load_mixed_rows();
/// // Columns: id, score — `score` is NA on every `Old` row.
/// let df = par_iter_to_dataframe_growing(rows, None)?;
/// ```
#[cfg(feature = "rayon")]
pub fn par_iter_to_dataframe_growing<T, I>(
    iter: I,
    nrow_hint: Option<usize>,
) -> Result<BuiltDataFrame, RSerdeError>
where
    // `Sync` (not just `Send`): rows are borrowed across rayon workers via
    // `par_chunks`, so `&T: Send`, i.e. `T: Sync`.
    T: Serialize + Send + Sync,
    I: IntoIterator<Item = T>,
{
    crate::optionals::parallel::ensure_pool();

    let rows: Vec<T> = iter.into_iter().collect();
    let Some((schema, merged, nrow)) = par_build_columns_growing(&rows, nrow_hint)? else {
        // SAFETY: `empty_dataframe` returns a well-formed 0-row data.frame SEXP.
        return Ok(unsafe { BuiltDataFrame::adopt_sexp(empty_dataframe()) });
    };

    // SAFETY: caller invokes this on the R main thread; column lengths all
    // equal `nrow` (fill pads every union column on every row).
    let scope = unsafe { crate::ProtectScope::new() };
    Ok(unsafe { assemble_with_scope(&schema, &merged, nrow, scope) })
}

/// Pure-Rust core of [`par_iter_to_dataframe`]: discover the schema, fan the
/// row→column fill out across rayon, and merge the per-thread chunk buffers
/// back into one column set in row order.
///
/// Returns `Ok(None)` for empty input (the caller emits a 0-row data.frame).
/// Contains **zero R-API contact** — no SEXP allocation, no `ProtectScope` —
/// so it is fully exercisable in a unit test without the R main thread, and the
/// rayon region is sound (all `ColumnBuffer` variants reached here are
/// `Send`-safe `Vec`s; `Generic` is rejected up front).
#[cfg(feature = "rayon")]
#[allow(clippy::type_complexity)]
fn par_build_columns<T>(
    rows: &[T],
    nrow_hint: Option<usize>,
) -> Result<Option<(Schema, Vec<ColumnBuffer>, usize)>, RSerdeError>
where
    T: Serialize + Sync,
{
    if rows.is_empty() {
        return Ok(None);
    }

    // Discover schema from the first row (homogeneous-schema contract). Schema
    // discovery is pure-Rust; doing it here keeps the Generic check before any
    // fan-out.
    let mut acc = SchemaAccumulator::new(SchemaMode::SingleRow);
    acc.feed(&rows[0])?;
    let schema = acc.finalize()?;

    // Reject schemas the parallel path can't honour: any `Generic` column needs
    // the R main thread to allocate per-cell SEXPs.
    if schema
        .fields
        .iter()
        .any(|f| f.col_type == ColumnType::Generic)
    {
        return Err(RSerdeError::Message(
            "par_iter_to_dataframe: schema contains a generic (list) column, which requires \
             the R main thread; use the sequential iter_to_dataframe instead"
                .into(),
        ));
    }

    let merged = par_fill_merge(rows, &schema, nrow_hint, /* strict */ true)?;
    Ok(Some((schema, merged, rows.len())))
}

/// Pure-Rust core of [`par_iter_to_dataframe_growing`]: union-discover the
/// schema across all rows in parallel, resolve it globally, then fan the fill
/// out and merge — see the public fn's docstring for the three-phase design.
///
/// Returns `Ok(None)` for empty input. Zero R-API contact, like
/// [`par_build_columns`].
#[cfg(feature = "rayon")]
#[allow(clippy::type_complexity)]
fn par_build_columns_growing<T>(
    rows: &[T],
    nrow_hint: Option<usize>,
) -> Result<Option<(Schema, Vec<ColumnBuffer>, usize)>, RSerdeError>
where
    T: Serialize + Sync,
{
    use rayon::prelude::*;

    if rows.is_empty() {
        return Ok(None);
    }

    // 1. Parallel union discovery: each worker probes its chunk into a local
    //    accumulator. Per-row probe failures (e.g. top-level None) are
    //    swallowed exactly like `vec_to_dataframe`'s Phase 1.
    let chunk_size = par_chunk_size(rows.len());
    let chunk_accs: Vec<SchemaAccumulator> = rows
        .par_chunks(chunk_size)
        .map(|chunk| {
            let mut acc = SchemaAccumulator::new(SchemaMode::Union);
            for row in chunk {
                let _ = acc.feed(row);
            }
            acc
        })
        .collect();

    // 2. Global resolution: merge per-chunk candidates in chunk (= row) order,
    //    then resolve through the same lattice as the sequential union path.
    //    Merging *unresolved* candidates (rather than per-chunk schemas) makes
    //    the result identical to feeding every row into one accumulator.
    let mut acc = SchemaAccumulator::new(SchemaMode::Union);
    for chunk_acc in chunk_accs {
        acc.merge(chunk_acc);
    }
    let schema = acc.finalize().map_err(|_| {
        RSerdeError::Message(
            "par_iter_to_dataframe_growing: no fields discovered from any row".into(),
        )
    })?;

    // Reject Generic columns: per-cell SEXP allocation needs the R main
    // thread. A union-resolved Generic means either a genuinely generic field
    // (e.g. Vec<i32>) or a field that was None in every row — both belong on
    // the sequential path.
    if schema
        .fields
        .iter()
        .any(|f| f.col_type == ColumnType::Generic)
    {
        return Err(RSerdeError::Message(
            "par_iter_to_dataframe_growing: union schema contains a generic (list) column \
             (a list-typed field, or a field that is None in every row), which requires \
             the R main thread; use the sequential vec_to_dataframe instead"
                .into(),
        ));
    }

    // 3. Fill against the shared union schema. Non-strict, mirroring
    //    `vec_to_dataframe`: the union absorbed every row's keys, so an
    //    unmapped field can only be a discarded Compound shape (the documented
    //    recursive-union limitation) — skip it rather than error.
    let merged = par_fill_merge(rows, &schema, nrow_hint, /* strict */ false)?;
    Ok(Some((schema, merged, rows.len())))
}

/// Chunk size for the parallel fan-outs: ~4 chunks per rayon thread for load
/// balancing, but never tiny chunks.
#[cfg(feature = "rayon")]
fn par_chunk_size(nrow: usize) -> usize {
    let nthreads = rayon::current_num_threads().max(1);
    nrow.div_ceil(nthreads * 4).max(1)
}

/// Shared fill + merge fan-out for [`par_build_columns`] /
/// [`par_build_columns_growing`]: split `rows` into contiguous chunks, fill a
/// local `Vec<ColumnBuffer>` per chunk against the shared `schema`, then
/// concatenate the chunk columns back in row order.
///
/// `strict` mirrors [`ColumnFiller`]'s flag: when true, a row touching a field
/// absent from `schema` errors (homogeneous first-row contract); when false it
/// is silently skipped (union schema — misses can only be discarded Compound
/// shapes).
#[cfg(feature = "rayon")]
fn par_fill_merge<T>(
    rows: &[T],
    schema: &Schema,
    nrow_hint: Option<usize>,
    strict: bool,
) -> Result<Vec<ColumnBuffer>, RSerdeError>
where
    T: Serialize + Sync,
{
    use rayon::prelude::*;

    let ncol = schema.fields.len();
    let nrow = rows.len();
    let chunk_size = par_chunk_size(nrow);

    // Fan out. Split into contiguous chunks; each worker fills a local
    // Vec<ColumnBuffer> against the shared schema with zero R API contact.
    // `par_chunks` + `map` + `collect::<Vec<_>>` preserves chunk order
    // (rayon's IndexedParallelIterator::collect is order-preserving).
    let chunk_results: Vec<Result<Vec<ColumnBuffer>, RSerdeError>> = rows
        .par_chunks(chunk_size)
        .map(|chunk| {
            let mut filler = ParChunkFiller::new(schema, chunk.len(), strict);
            for row in chunk {
                filler.push(row)?;
            }
            Ok(filler.into_columns())
        })
        .collect();

    // Merge in row order. Seed with empty columns of the right variants,
    // then append each chunk's columns in iteration (= row) order.
    let mut merged: Vec<ColumnBuffer> = schema
        .fields
        .iter()
        .map(|f| ColumnBuffer::new(f.col_type, nrow_hint.unwrap_or(nrow)))
        .collect();

    for chunk in chunk_results {
        let chunk_cols = chunk?;
        debug_assert_eq!(chunk_cols.len(), ncol, "chunk column count mismatch");
        for (dst, src) in merged.iter_mut().zip(chunk_cols) {
            dst.append(src);
        }
    }

    Ok(merged)
}

/// Off-thread, scope-free column filler for [`par_iter_to_dataframe`] and
/// [`par_iter_to_dataframe_growing`].
///
/// Mirrors [`ColumnFiller`] but operates on an owned `Vec<ColumnBuffer>` with no
/// `ProtectScope` and rejects the `Generic` path — so it carries no R-side
/// state and is safe to run on a rayon worker. `strict` carries
/// [`ColumnFiller`]'s flag: the homogeneous path validates every row against
/// the first-row schema (an unknown field errors); the growing path fills
/// against a union schema where an unmapped field can only be a discarded
/// Compound shape, skipped like `vec_to_dataframe` does.
#[cfg(feature = "rayon")]
struct ParChunkFiller<'a> {
    columns: Vec<ColumnBuffer>,
    field_map: &'a FieldMap,
    /// Per-row "did this column get a value?" tracking; reset each row by
    /// `pad_unfilled`.
    filled: Vec<bool>,
    ncol: usize,
    strict: bool,
}

#[cfg(feature = "rayon")]
impl<'a> ParChunkFiller<'a> {
    fn new(schema: &'a Schema, capacity: usize, strict: bool) -> Self {
        let columns = schema
            .fields
            .iter()
            .map(|f| ColumnBuffer::new(f.col_type, capacity))
            .collect();
        let ncol = schema.fields.len();
        Self {
            columns,
            field_map: &schema.field_map,
            filled: vec![false; ncol],
            ncol,
            strict,
        }
    }

    fn push<T: ?Sized + Serialize>(&mut self, row: &T) -> Result<(), RSerdeError> {
        let sub = ParColumnFiller {
            columns: &mut self.columns,
            field_map: self.field_map,
            filled: &mut self.filled,
            col_start: 0,
            col_count: self.ncol,
            is_top_level: true,
            pending_key: None,
            strict: self.strict,
        };
        row.serialize(sub)
    }

    fn into_columns(self) -> Vec<ColumnBuffer> {
        self.columns
    }
}

/// Borrowing serde `Serializer` that drives one row into a [`ParChunkFiller`]'s
/// column buffers. The scope-free twin of [`ColumnFiller`].
#[cfg(feature = "rayon")]
struct ParColumnFiller<'a> {
    columns: &'a mut [ColumnBuffer],
    field_map: &'a FieldMap,
    filled: &'a mut Vec<bool>,
    col_start: usize,
    col_count: usize,
    is_top_level: bool,
    pending_key: Option<String>,
    strict: bool,
}

#[cfg(feature = "rayon")]
impl ParColumnFiller<'_> {
    fn fill_field<T: ?Sized + Serialize>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        match self.field_map.map.get(key) {
            Some(FieldMapping::Scalar { col_idx }) => {
                self.columns[*col_idx].push_value_pure(value)?;
                self.filled[*col_idx] = true;
            }
            Some(FieldMapping::Compound {
                sub_fields,
                col_start,
                col_count,
                ..
            }) => {
                let sub = ParColumnFiller {
                    columns: self.columns,
                    field_map: sub_fields,
                    filled: self.filled,
                    col_start: *col_start,
                    col_count: *col_count,
                    is_top_level: false,
                    pending_key: None,
                    strict: self.strict,
                };
                value.serialize(sub)?;
            }
            None => {
                // See ColumnFiller::fill_field — strict enforces the
                // homogeneous first-row contract; the union (growing) path
                // skips, since its schema absorbed every row's keys.
                if self.strict {
                    return Err(RSerdeError::Message(format!(
                        "par_iter_to_dataframe: row introduced field {key:?} not in initial schema"
                    )));
                }
            }
        }
        Ok(())
    }

    fn pad_unfilled(&mut self) {
        let start = self.field_map.col_start;
        let end = start + self.field_map.total_cols;
        if self.is_top_level {
            for i in start..end {
                if !self.filled[i] {
                    self.columns[i].push_na();
                }
                self.filled[i] = false; // reset for next row
            }
        } else {
            for i in start..end {
                if !self.filled[i] {
                    self.columns[i].push_na();
                }
                self.filled[i] = true;
            }
        }
    }
}

#[cfg(feature = "rayon")]
impl<'a> ser::Serializer for ParColumnFiller<'a> {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Ok(self)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), RSerdeError> {
        if self.is_top_level {
            return Err(RSerdeError::Message("expected struct".into()));
        }
        value.serialize(self)
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        if self.is_top_level {
            return Err(RSerdeError::Message("expected struct".into()));
        }
        for i in self.col_start..self.col_start + self.col_count {
            self.columns[i].push_na();
            self.filled[i] = true;
        }
        Ok(())
    }

    reject_non_struct!(@primitives "expected struct");
}

#[cfg(feature = "rayon")]
impl ser::SerializeStruct for ParColumnFiller<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.fill_field(key, value)
    }

    fn end(mut self) -> Result<(), RSerdeError> {
        self.pad_unfilled();
        Ok(())
    }
}

#[cfg(feature = "rayon")]
impl ser::SerializeMap for ParColumnFiller<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        let mut extractor = ValueExtractor::default();
        key.serialize(&mut extractor)?;
        self.pending_key = match extractor.value {
            ExtractedValue::Str(s) => Some(s),
            _ => Some(String::new()),
        };
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        let key = self.pending_key.take().unwrap_or_default();
        self.fill_field(&key, value)
    }

    fn end(mut self) -> Result<(), RSerdeError> {
        self.pad_unfilled();
        Ok(())
    }
}

/// User-facing column type descriptor for [`SerdeRowBuilder::with_schema`].
///
/// Maps onto the internal `ColumnType` and unlocks an NA-tolerance hint via
/// `Optional(_)`. The wrapper does **not** change the underlying column type —
/// `Optional(Integer)` produces an integer column where `None` lands as
/// `NA_INTEGER`. Without the hint, an all-`None` column discovered from the
/// first row would otherwise degrade to a logical-NA column (see
/// `vec_to_dataframe` doc).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSpec {
    /// R `logical` column (`bool`).
    Logical,
    /// R `integer` column (`i8`/`i16`/`i32`).
    Integer,
    /// R `numeric` column (`f32`/`f64`/`i64`/`u64`).
    Real,
    /// R `character` column (`String`/`&str`).
    Character,
    /// R generic list column (per-element SEXP fallback).
    Generic,
    /// NA-tolerance hint wrapping a base type. `Optional(Integer)` is an
    /// integer column where `None` is `NA_INTEGER`.
    Optional(Box<TypeSpec>),
}

impl TypeSpec {
    /// Collapse the user-facing spec to the internal `ColumnType`. The
    /// `Optional` hint is discarded — column types are already NA-tolerant
    /// at the R level (`NA_INTEGER` / `NA_REAL` / `NA_character_` are part
    /// of the corresponding atomic vector types).
    fn into_column_type(self) -> ColumnType {
        match self {
            TypeSpec::Logical => ColumnType::Logical,
            TypeSpec::Integer => ColumnType::Integer,
            TypeSpec::Real => ColumnType::Real,
            TypeSpec::Character => ColumnType::Character,
            TypeSpec::Generic => ColumnType::Generic,
            TypeSpec::Optional(inner) => inner.into_column_type(),
        }
    }
}

/// Stream a `Result`-yielding iterator into a named `list(ok = df, err = df)`.
///
/// Maintains two [`SerdeRowBuilder`]s and dispatches each row to the
/// appropriate one based on its `Result` variant. The output names default to
/// `"ok"` / `"err"`; pass [`DispatchNames`] to override.
///
/// Unlike a manual `iter.partition(Result::is_ok)` + two `iter_to_dataframe`
/// calls, this preserves the streaming property — rows are dispatched as they
/// arrive, no double-materialisation.
///
/// # Empty sides
///
/// A side that received zero rows produces a 0-row, 0-column data.frame in
/// the returned named list. The list always has both slots so downstream R
/// code can rely on a stable shape (`res$ok` / `res$err`).
///
/// # When to use this vs [`result_to_dataframe`]
///
/// Use [`super::result_to_dataframe`] when you already have a `&[Result<T, E>]`
/// in memory; it offers richer shape control (`Auto` / `Collated` / `Split`
/// with custom sentinel). Use `dispatch_to_dataframes` when the rows arrive
/// incrementally and you specifically want the streaming two-builder shape.
///
/// # Errors
///
/// - A row fails to serialize (propagates from either builder).
/// - Schema mismatch on later rows of the same variant.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::serde::dispatch_to_dataframes;
///
/// #[derive(Serialize)] struct Ok_ { id: i32, val: f64 }
/// #[derive(Serialize)] struct Err_ { id: i32, reason: String }
///
/// let rows = (0..10).map(|i| if i % 3 == 0 {
///     Err(Err_ { id: i, reason: "skip".into() })
/// } else {
///     Ok(Ok_ { id: i, val: i as f64 * 0.5 })
/// });
///
/// let named = dispatch_to_dataframes(rows, None, DispatchNames::default())?;
/// // named$ok  -> id, val
/// // named$err -> id, reason
/// ```
pub fn dispatch_to_dataframes<O, E, I>(
    iter: I,
    nrow_hint: Option<usize>,
    names: DispatchNames,
) -> Result<crate::list::List, RSerdeError>
where
    O: Serialize,
    E: Serialize,
    I: IntoIterator<Item = Result<O, E>>,
{
    let mut ok_builder = SerdeRowBuilder::<O>::new(nrow_hint);
    let mut err_builder = SerdeRowBuilder::<E>::new(nrow_hint);

    for row in iter {
        match row {
            Ok(o) => ok_builder.push(o)?,
            Err(e) => err_builder.push(e)?,
        }
    }

    // Both finishes return an owned, GC-rooted `BuiltDataFrame`: `ok_df` stays
    // rooted (by its own handle) across `err_builder.finish()`'s allocation, so
    // the manual `OwnedProtect` guards this used to need are gone (#1128). The
    // `*` derefs each handle to its `DataFrame` view for `push` (which re-roots
    // it in the builder's own scope); the handles live to the end of the
    // statement, so they remain rooted across the whole assembly.
    let ok_df = ok_builder.finish()?;
    let err_df = err_builder.finish()?;

    Ok(NamedDataFrameListBuilder::with_capacity(2)
        .push(names.ok, *ok_df)
        .push(names.err, *err_df)
        .build())
}

/// Custom slot names for [`dispatch_to_dataframes`]'s output list.
///
/// Defaults to `ok = "ok"`, `err = "err"`. Override either or both via
/// `DispatchNames { ok: "results".into(), err: "errors".into() }`.
#[derive(Debug, Clone)]
pub struct DispatchNames {
    pub ok: String,
    pub err: String,
}

impl Default for DispatchNames {
    fn default() -> Self {
        Self {
            ok: "ok".to_string(),
            err: "err".to_string(),
        }
    }
}

/// Builder for incremental data.frame assembly.
///
/// Three schema modes:
///
/// 1. **Default** ([`SerdeRowBuilder::new`]) — schema discovered from the
///    first [`push`](Self::push); subsequent rows that introduce new fields
///    are rejected.
/// 2. **Pre-declared** ([`SerdeRowBuilder::with_schema`]) — schema fixed at
///    construction; first push skips discovery; later pushes must conform.
/// 3. **Growing** ([`SerdeRowBuilder::grow_schema`]) — new fields seen in
///    later rows are added on-the-fly and back-filled with NA on prior rows.
///    Composes with [`with_schema`](Self::with_schema) to start from a
///    declared partial schema.
///
/// Call [`finish`](Self::finish) to produce a rooted
/// [`BuiltDataFrame`].
///
/// Use [`iter_to_dataframe`] when an iterator suffices; reach for this when
/// you need explicit control over push points (conditional skipping,
/// streaming from multiple sources, custom NA strategies).
///
/// # Examples
///
/// ```rust,ignore
/// # use miniextendr_api::serde::{SerdeRowBuilder, TypeSpec};
/// # use serde::Serialize;
/// #[derive(Serialize)]
/// struct Row { id: i32, label: Option<String> }
///
/// // Pre-declared schema. Optional(Character) keeps the column character-typed
/// // even if the first row's label is None.
/// let mut b = SerdeRowBuilder::<Row>::with_schema(
///     [
///         ("id", TypeSpec::Integer),
///         ("label", TypeSpec::Optional(Box::new(TypeSpec::Character))),
///     ],
///     None,
/// );
/// b.push(Row { id: 1, label: None }).unwrap();
/// b.push(Row { id: 2, label: Some("two".into()) }).unwrap();
/// let df = b.finish().unwrap();
/// ```
pub struct SerdeRowBuilder<T: Serialize> {
    /// Schema set on first push (default mode), at construction
    /// ([`with_schema`](Self::with_schema)), or grown lazily ([`grow_schema`](Self::grow_schema)).
    schema: Option<Schema>,
    /// One buffer per column; lengths grow with each row.
    columns: Vec<ColumnBuffer>,
    /// Per-row "did this column get a value?" tracking; resized to ncol on first push.
    filled: Vec<bool>,
    /// Number of rows pushed so far.
    nrow: usize,
    /// Initial capacity hint; if `Some`, columns are allocated with this capacity.
    nrow_hint: Option<usize>,
    /// Holds protected SEXPs from `ColumnBuffer::Generic` cells until `finish` assembles them.
    scope: crate::ProtectScope,
    /// When true, each push runs a Union-mode discovery pass and adds any
    /// previously-unseen fields (back-filling NA on existing rows).
    grow: bool,
    _marker: core::marker::PhantomData<fn(T)>,
}

impl<T: Serialize> SerdeRowBuilder<T> {
    /// Create a new builder with schema discovered on first [`push`](Self::push).
    ///
    /// `nrow_hint` pre-sizes column buffers; `None` is acceptable.
    pub fn new(nrow_hint: Option<usize>) -> Self {
        Self {
            schema: None,
            columns: Vec::new(),
            filled: Vec::new(),
            nrow: 0,
            nrow_hint,
            // SAFETY: builder must be constructed on the R main thread.
            // ProtectScope carries NoSendSync; the builder cannot escape.
            scope: unsafe { crate::ProtectScope::new() },
            grow: false,
            _marker: core::marker::PhantomData,
        }
    }

    /// Create a builder with a pre-declared flat schema.
    ///
    /// Skips the first-row discovery pass. All later pushes are validated
    /// against this schema by the strict `ColumnFiller`; fields not in
    /// the schema produce an error (unless [`grow_schema`](Self::grow_schema)
    /// is chained, in which case new fields are added on the fly).
    ///
    /// `schema` is an iterable of `(name, TypeSpec)` pairs. Order is
    /// preserved in the resulting data.frame's column layout.
    ///
    /// **Limitation**: this constructor takes a flat schema only — nested
    /// struct flattening (`parent_child` columns) is not supported here.
    /// Callers who need flattened nested structs either let default
    /// discovery handle it, or pre-flatten the names themselves
    /// (`"parent_child"` strings).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use miniextendr_api::serde::{SerdeRowBuilder, TypeSpec};
    /// # use serde::Serialize;
    /// #[derive(Serialize)]
    /// struct R { id: i32, name: String }
    ///
    /// let mut b = SerdeRowBuilder::<R>::with_schema(
    ///     [("id", TypeSpec::Integer), ("name", TypeSpec::Character)],
    ///     Some(100),
    /// );
    /// for i in 0..100 {
    ///     b.push(R { id: i, name: format!("row_{i}") }).unwrap();
    /// }
    /// let df = b.finish().unwrap();
    /// ```
    pub fn with_schema<S, I>(schema: I, nrow_hint: Option<usize>) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = (S, TypeSpec)>,
    {
        let mut fields: Vec<FieldInfo> = Vec::new();
        let mut map: HashMap<String, FieldMapping> = HashMap::new();
        for (name, spec) in schema {
            let name = name.into();
            let col_idx = fields.len();
            let col_type = spec.into_column_type();
            fields.push(FieldInfo {
                name: name.clone(),
                col_type,
            });
            map.insert(name, FieldMapping::Scalar { col_idx });
        }
        let total = fields.len();
        let cap = nrow_hint.unwrap_or(0);
        let columns: Vec<ColumnBuffer> = fields
            .iter()
            .map(|f| ColumnBuffer::new(f.col_type, cap))
            .collect();
        let filled = vec![false; total];
        let schema = Schema {
            fields,
            field_map: FieldMap {
                map,
                col_start: 0,
                total_cols: total,
            },
        };
        Self {
            schema: Some(schema),
            columns,
            filled,
            nrow: 0,
            nrow_hint,
            // SAFETY: builder must be constructed on the R main thread.
            scope: unsafe { crate::ProtectScope::new() },
            grow: false,
            _marker: core::marker::PhantomData,
        }
    }

    /// Enable growing-schema mode: new fields discovered in later rows are
    /// added on the fly and back-filled with NA on prior rows.
    ///
    /// Composes with [`with_schema`](Self::with_schema) — call
    /// `SerdeRowBuilder::with_schema(...).grow_schema()` to start with a
    /// declared partial schema and let new fields appear as rows arrive.
    ///
    /// Cost: O(new_fields × existing_nrow) on each push that introduces a
    /// new field. For row-by-row growing types this is amortised
    /// O(nrow × ncols) — the same shape as `vec_to_dataframe` today.
    ///
    /// **Type clashes**: a later row writing a `String` to a column whose
    /// first-seen value was an `Integer` follows today's union-path
    /// behaviour — the value is coerced or NA-filled by
    /// `ColumnBuffer::push_value`. No new error is raised. If your data
    /// is genuinely heterogeneous, declare the column as
    /// `TypeSpec::Generic` to get a list-column.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use miniextendr_api::serde::SerdeRowBuilder;
    /// # use std::collections::BTreeMap;
    /// // Heterogeneous rows: each row is a map; later rows introduce new keys.
    /// let mut b = SerdeRowBuilder::<BTreeMap<String, i32>>::new(None).grow_schema();
    ///
    /// let r1: BTreeMap<String, i32> = [("a".into(), 1)].into_iter().collect();
    /// let r2: BTreeMap<String, i32> = [("a".into(), 2), ("b".into(), 3)].into_iter().collect();
    /// b.push(r1).unwrap();
    /// b.push(r2).unwrap();  // adds column "b", back-fills NA on row 0
    /// let df = b.finish().unwrap();
    /// ```
    pub fn grow_schema(mut self) -> Self {
        self.grow = true;
        self
    }

    /// Append a row.
    ///
    /// In default mode the first call discovers the schema. In
    /// [`with_schema`](Self::with_schema) mode the schema is fixed at
    /// construction. In [`grow_schema`](Self::grow_schema) mode each push
    /// also runs a discovery pass and absorbs any new fields, back-filling
    /// NA on prior rows.
    pub fn push(&mut self, row: T) -> Result<(), RSerdeError> {
        if self.schema.is_none() {
            let mut acc = SchemaAccumulator::new(SchemaMode::SingleRow);
            acc.feed(&row)?;
            let schema = acc.finalize()?;
            let ncol = schema.fields.len();
            let cap = self.nrow_hint.unwrap_or(0);
            self.columns = schema
                .fields
                .iter()
                .map(|f| ColumnBuffer::new(f.col_type, cap))
                .collect();
            self.filled = vec![false; ncol];
            self.schema = Some(schema);
        } else if self.grow {
            self.absorb_new_fields(&row)?;
        }

        let schema = self.schema.as_ref().expect("schema set above");
        let ncol = schema.fields.len();

        let filler = ColumnFiller {
            columns: &mut self.columns,
            field_map: &schema.field_map,
            filled: &mut self.filled,
            col_start: 0,
            col_count: ncol,
            is_top_level: true,
            pending_key: None,
            scope: &self.scope,
            strict: true,
        };
        // ColumnFiller::SerializeStruct::end calls pad_unfilled, which
        // NA-pads any column the row didn't touch and resets `filled` for
        // the next push — same flow as vec_to_dataframe.
        row.serialize(filler)?;

        self.nrow += 1;
        Ok(())
    }

    /// Run a Union-mode discovery pass over `row` and absorb any fields not
    /// present in the current schema. New columns get a fresh
    /// [`ColumnBuffer`] back-filled with `self.nrow` NA values so the
    /// per-column length invariant is preserved going into the fill step.
    ///
    /// Compound (nested-struct) discoveries are flattened by
    /// [`SchemaAccumulator::finalize`]; each leaf field appears as its own
    /// top-level entry in the per-row schema and is added here as a flat
    /// scalar column under that name.
    fn absorb_new_fields(&mut self, row: &T) -> Result<(), RSerdeError> {
        let mut acc = SchemaAccumulator::new(SchemaMode::Union);
        // Union mode tolerates rows that probe to no fields (e.g. top-level
        // None); we propagate hard serialization errors.
        let _ = acc.feed(row);
        // If the row produced no fields at all, `finalize` would error; treat
        // that as "no new fields" rather than failing the push.
        let discovered = match acc.finalize() {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };

        let cap = self.nrow_hint.unwrap_or(0);
        let schema = self.schema.as_mut().expect("schema set by push gating");

        for field in discovered.fields {
            if schema.field_map.map.contains_key(&field.name) {
                continue;
            }
            // Allocate a fresh column and back-fill prior rows with NA so
            // the column length matches the existing nrow.
            let mut buf = ColumnBuffer::new(field.col_type, cap);
            for _ in 0..self.nrow {
                buf.push_na();
            }
            let col_idx = schema.fields.len();
            self.columns.push(buf);
            self.filled.push(false);
            schema
                .field_map
                .map
                .insert(field.name.clone(), FieldMapping::Scalar { col_idx });
            schema.field_map.total_cols = col_idx + 1;
            schema.fields.push(field);
        }
        Ok(())
    }

    /// Number of rows pushed so far.
    pub fn len(&self) -> usize {
        self.nrow
    }

    /// Whether no rows have been pushed yet.
    pub fn is_empty(&self) -> bool {
        self.nrow == 0
    }

    /// Consume the builder and produce the data.frame.
    ///
    /// An empty builder produces an empty 0-row 0-column data.frame
    /// (matching `vec_to_dataframe(&[])`).
    pub fn finish(self) -> Result<BuiltDataFrame, RSerdeError> {
        let Some(schema) = self.schema else {
            // SAFETY: `empty_dataframe` returns a well-formed 0-row data.frame SEXP.
            return Ok(unsafe { BuiltDataFrame::adopt_sexp(empty_dataframe()) });
        };
        // SAFETY: nrow columns × matching lengths invariant maintained by push.
        // `assemble_with_scope` runs `assemble_dataframe` then drops the scope —
        // see its docstring for the drop-order rationale.
        Ok(unsafe { assemble_with_scope(&schema, &self.columns, self.nrow, self.scope) })
    }
}

// endregion

// region: Field mapping (recursive name → column routing)

/// Maps a field name to its column location in the flat column array.
#[derive(Clone)]
enum FieldMapping {
    /// Scalar field: writes directly to one column.
    Scalar { col_idx: usize },
    /// Compound field (flattened nested struct): spans multiple columns.
    Compound {
        col_start: usize,
        col_count: usize,
        sub_fields: FieldMap,
    },
}

/// Name-to-column mapping for one level of struct fields.
#[derive(Clone)]
struct FieldMap {
    map: HashMap<String, FieldMapping>,
    col_start: usize,
    total_cols: usize,
}

// endregion

// region: Schema discovery

#[derive(Debug, Clone, Copy, PartialEq)]
enum ColumnType {
    Logical,
    Integer,
    Real,
    Character,
    Generic,
}

#[derive(Clone)]
struct FieldInfo {
    name: String,
    col_type: ColumnType,
}

struct Schema {
    fields: Vec<FieldInfo>,
    field_map: FieldMap,
}

/// A candidate mapping for a single key, extracted from one probe row.
enum Candidate {
    Scalar(ColumnType),
    Compound {
        fields: Vec<FieldInfo>,
        sub_map: FieldMap,
    },
}

/// Resolve a slice of candidates for one key into the best single candidate.
///
/// Lattice (highest wins):
/// - `Compound` beats everything (has concrete shape).
/// - `Scalar(non-Generic)` beats `Scalar(Generic)`.
/// - `Scalar(Generic)` is the bottom (None-only probes land here).
///
/// Two `Scalar(non-Generic)` of different types: keep the first seen (no widening).
/// Two `Compound` of different shapes: **recursively unioned** key-by-key
/// (#1370) — see [`merge_compound_shapes`]. This is what lets a nested
/// sub-field whose first-row probe is `None` (`Scalar(Generic)`) still
/// upgrade to an atomic type when a later row types it, instead of locking
/// the whole nested column to `Generic` (list).
fn resolve_candidates(candidates: &mut Vec<Candidate>) -> Candidate {
    let mut drained = candidates.drain(..);
    let mut acc = drained
        .next()
        .expect("resolve_candidates: called with an empty candidate list");
    for next in drained {
        acc = combine_two_candidates(acc, next);
    }
    acc
}

/// Fold two candidates (in row order — `a` seen no later than `b`) into one,
/// per the lattice documented on [`resolve_candidates`].
fn combine_two_candidates(a: Candidate, b: Candidate) -> Candidate {
    match (a, b) {
        (
            Candidate::Compound {
                fields: af,
                sub_map: am,
            },
            Candidate::Compound {
                fields: bf,
                sub_map: bm,
            },
        ) => {
            let (fields, sub_map) = merge_compound_shapes(af, am, bf, bm);
            Candidate::Compound { fields, sub_map }
        }
        // Compound beats Scalar regardless of which side it's on.
        (Candidate::Compound { fields, sub_map }, Candidate::Scalar(_))
        | (Candidate::Scalar(_), Candidate::Compound { fields, sub_map }) => {
            Candidate::Compound { fields, sub_map }
        }
        // Scalar(non-Generic) beats Scalar(Generic).
        (Candidate::Scalar(ColumnType::Generic), Candidate::Scalar(t))
            if t != ColumnType::Generic =>
        {
            Candidate::Scalar(t)
        }
        // Otherwise keep the first-seen scalar (no widening between two non-Generic
        // types; both-Generic stays Generic).
        (Candidate::Scalar(t), Candidate::Scalar(_)) => Candidate::Scalar(t),
    }
}

/// A resolved slot for one key inside a nested-`Compound` shape, keeping its
/// already fully-qualified column name(s) (`process_field` prefixes names
/// with `{parent_key}_` as it flattens, so a leaf's [`FieldInfo::name`] is
/// already e.g. `"meta_variant"` by the time it reaches here). Mirrors
/// [`Candidate`] — the only difference is that the scalar leaf carries its
/// `FieldInfo` (name + type) rather than just a bare [`ColumnType`], since
/// [`Candidate::Scalar`] normally gets its name for free from the enclosing
/// per-key `HashMap` key, which nested recursion doesn't have.
enum SubShape {
    Scalar(FieldInfo),
    Compound(Vec<FieldInfo>, FieldMap),
}

/// Pull the (bare-key) child `key`'s shape out of a `Compound` candidate's
/// `(fields, field_map)` pair. `fields` holds every leaf under this Compound in
/// flat, already-prefixed-name order; `field_map.col_start` is the absolute
/// column index of `fields[0]`, so `col_idx - field_map.col_start` recovers the
/// local index into `fields` for any mapping found in `field_map`.
fn extract_sub_shape(fields: &[FieldInfo], field_map: &FieldMap, key: &str) -> Option<SubShape> {
    match field_map.map.get(key)? {
        FieldMapping::Scalar { col_idx } => {
            let local = col_idx - field_map.col_start;
            Some(SubShape::Scalar(fields[local].clone()))
        }
        FieldMapping::Compound {
            col_start,
            col_count,
            sub_fields,
        } => {
            let local_start = col_start - field_map.col_start;
            let child_fields = fields[local_start..local_start + col_count].to_vec();
            Some(SubShape::Compound(child_fields, sub_fields.clone()))
        }
    }
}

/// The first-seen key order of one level of a [`FieldMap`], recovered from
/// each mapping's column position (field declaration order is stable across
/// probes of the same Rust type, so sorting by position reconstructs it;
/// `FieldMap.map` itself is a `HashMap` with no ordering of its own).
fn field_map_key_order(field_map: &FieldMap) -> Vec<String> {
    let mut entries: Vec<(usize, &String)> = field_map
        .map
        .iter()
        .map(|(k, v)| {
            let pos = match v {
                FieldMapping::Scalar { col_idx } => *col_idx,
                FieldMapping::Compound { col_start, .. } => *col_start,
            };
            (pos, k)
        })
        .collect();
    entries.sort_by_key(|(pos, _)| *pos);
    entries.into_iter().map(|(_, k)| k.clone()).collect()
}

/// Fold two shapes for the *same* key into one, applying the same lattice as
/// [`combine_two_candidates`] (recursing into [`merge_compound_shapes`] for
/// Compound-vs-Compound instead of keeping the first seen).
fn combine_sub_shapes(a: SubShape, b: SubShape) -> SubShape {
    match (a, b) {
        (SubShape::Compound(af, am), SubShape::Compound(bf, bm)) => {
            let (fields, sub_map) = merge_compound_shapes(af, am, bf, bm);
            SubShape::Compound(fields, sub_map)
        }
        (SubShape::Compound(fields, sub_map), SubShape::Scalar(_))
        | (SubShape::Scalar(_), SubShape::Compound(fields, sub_map)) => {
            SubShape::Compound(fields, sub_map)
        }
        (SubShape::Scalar(a_info), SubShape::Scalar(b_info)) => {
            if a_info.col_type == ColumnType::Generic && b_info.col_type != ColumnType::Generic {
                SubShape::Scalar(b_info)
            } else {
                SubShape::Scalar(a_info)
            }
        }
    }
}

/// Append a resolved key's shape into a fresh flat `(fields, field_map)` pair
/// under construction, assigning it the next contiguous column slot(s).
fn place_sub_shape(
    key: String,
    shape: SubShape,
    merged_fields: &mut Vec<FieldInfo>,
    merged_map: &mut HashMap<String, FieldMapping>,
) {
    match shape {
        SubShape::Scalar(info) => {
            let col_idx = merged_fields.len();
            merged_fields.push(info);
            merged_map.insert(key, FieldMapping::Scalar { col_idx });
        }
        SubShape::Compound(fields, sub_fields) => {
            let col_start = merged_fields.len();
            let col_count = fields.len();
            let old_base = sub_fields.col_start;
            let remapped = remap_field_map(sub_fields, old_base, col_start);
            merged_fields.extend(fields);
            merged_map.insert(
                key,
                FieldMapping::Compound {
                    col_start,
                    col_count,
                    sub_fields: remapped,
                },
            );
        }
    }
}

/// Recursively union two `Compound` candidates' shapes key-by-key (#1370).
///
/// Instead of discarding `b` wholesale (the pre-#1370 "keep first seen"
/// behaviour), each key present in either side is resolved through the same
/// lattice as [`resolve_candidates`] — recursing for nested `Compound`
/// sub-fields — so a sub-field that probed as `Generic` in `a` (e.g. a `None`
/// row) can still pick up a concrete type from `b`, and a key present only in
/// `b` (a genuinely different shape, e.g. a `#[serde(skip_serializing_if)]`
/// field omitted in `a`'s row but present in `b`'s) is no longer dropped.
///
/// Key order is first-seen union: `a`'s keys in `a`'s own order, then any of
/// `b`'s keys not already in `a`, in `b`'s own order — mirroring the
/// top-level `SchemaAccumulator` union order.
fn merge_compound_shapes(
    a_fields: Vec<FieldInfo>,
    a_map: FieldMap,
    b_fields: Vec<FieldInfo>,
    b_map: FieldMap,
) -> (Vec<FieldInfo>, FieldMap) {
    let a_order = field_map_key_order(&a_map);
    let b_order = field_map_key_order(&b_map);

    let mut merged_fields: Vec<FieldInfo> = Vec::new();
    let mut merged_map: HashMap<String, FieldMapping> = HashMap::new();

    for key in a_order {
        let a_shape = extract_sub_shape(&a_fields, &a_map, &key)
            .expect("key drawn from a_map's own key order");
        let shape = match extract_sub_shape(&b_fields, &b_map, &key) {
            Some(b_shape) => combine_sub_shapes(a_shape, b_shape),
            None => a_shape,
        };
        place_sub_shape(key, shape, &mut merged_fields, &mut merged_map);
    }
    for key in b_order {
        if a_map.map.contains_key(&key) {
            continue; // already resolved above (both sides had it)
        }
        let b_shape = extract_sub_shape(&b_fields, &b_map, &key)
            .expect("key drawn from b_map's own key order");
        place_sub_shape(key, b_shape, &mut merged_fields, &mut merged_map);
    }

    let total_cols = merged_fields.len();
    let field_map = FieldMap {
        map: merged_map,
        col_start: 0,
        total_cols,
    };
    (merged_fields, field_map)
}

/// Mode controls accumulation strictness and the final empty-schema error wording.
///
/// `SingleRow` produces the `SerdeRowBuilder: first row has no fields` error; `Union`
/// produces the `vec_to_dataframe: no fields discovered from any row` error.
/// Field collection itself is identical — both feed candidates into the same lattice
/// resolver. SingleRow callers feed exactly once; Union callers feed per row.
#[derive(Debug, Clone, Copy, PartialEq)]
enum SchemaMode {
    SingleRow,
    Union,
}

/// Accumulates per-row probes into per-key candidate lists, then resolves them
/// into a unified [`Schema`].
///
/// One limitation vs. a fully-typed schema (applies to Union mode; SingleRow sees
/// ≤1 candidate per key so the lattice degenerates):
///
/// - **Truly-all-None nested `Option<Struct>`**: when every row has `None` for an
///   `Option<UserStruct>`, the probe never sees the inner struct's fields. The key lands
///   as `Scalar(Generic)` (no Compound ever seen), which the assembly-time all-None
///   downgrade converts to a single logical-NA column. Structurally unfixable without a
///   type-level hint on stable Rust.
///
/// Two rows producing different `Compound` shapes for the same key (e.g., a nested
/// sub-field probed as `None` — `Scalar(Generic)` — on the first row and typed on a
/// later one, or a `#[serde(skip_serializing_if)]` sub-field present in some rows'
/// shape but not others') are no longer resolved by keeping the first seen: they are
/// **recursively unioned** key-by-key through the same lattice, so the merged shape
/// carries every sub-field each side saw, each at its best-seen type. See
/// [`resolve_candidates`] / [`merge_compound_shapes`]
/// ([#1370](https://github.com/A2-ai/miniextendr/issues/1370), fixed).
struct SchemaAccumulator {
    candidates: HashMap<String, Vec<Candidate>>,
    key_order: Vec<String>,
    mode: SchemaMode,
}

impl SchemaAccumulator {
    fn new(mode: SchemaMode) -> Self {
        Self {
            candidates: HashMap::new(),
            key_order: Vec::new(),
            mode,
        }
    }

    /// Probe one row and append its per-key candidates. Caller controls whether
    /// to propagate or swallow the error (Union mode treats per-row failures
    /// like top-level `None` as expected and skips them).
    fn feed<T: ?Sized + Serialize>(&mut self, row: &T) -> Result<(), RSerdeError> {
        let mut discoverer = SchemaDiscoverer::new(0);
        row.serialize(&mut discoverer)?;

        for key in &discoverer.key_order {
            if !self.candidates.contains_key(key) {
                self.key_order.push(key.clone());
                self.candidates.insert(key.clone(), Vec::new());
            }

            let Some(mapping) = discoverer.mappings.remove(key) else {
                continue;
            };

            let candidate = match mapping {
                FieldMapping::Scalar { col_idx } => {
                    Candidate::Scalar(discoverer.fields[col_idx].col_type)
                }
                FieldMapping::Compound {
                    col_start,
                    col_count,
                    sub_fields,
                } => {
                    let fields: Vec<FieldInfo> = (col_start..col_start + col_count)
                        .map(|i| FieldInfo {
                            name: discoverer.fields[i].name.clone(),
                            col_type: discoverer.fields[i].col_type,
                        })
                        .collect();
                    Candidate::Compound {
                        fields,
                        sub_map: sub_fields,
                    }
                }
            };
            self.candidates.get_mut(key).unwrap().push(candidate);
        }
        Ok(())
    }

    /// Absorb another accumulator's *unresolved* candidates, preserving
    /// first-seen key order (`self`'s keys first, then `other`'s new keys).
    ///
    /// Feeding rows chunk-by-chunk into per-chunk accumulators and merging
    /// them in chunk order is equivalent to feeding every row into one
    /// accumulator: per-key candidate lists concatenate in row order, and
    /// key first-appearance order is preserved. Used by the parallel union
    /// discovery in [`par_build_columns_growing`].
    #[cfg(feature = "rayon")]
    fn merge(&mut self, other: SchemaAccumulator) {
        // Register other's new keys in first-seen order. Every row behind
        // `self` precedes every row behind `other`, so appending other's
        // unseen keys after self's preserves global first-appearance order.
        for key in &other.key_order {
            if !self.candidates.contains_key(key) {
                self.key_order.push(key.clone());
                self.candidates.insert(key.clone(), Vec::new());
            }
        }
        // Move the candidate lists over. HashMap iteration order is fine
        // here: per-key lists are independent, and within a key self's
        // (earlier-row) candidates already precede the appended ones.
        for (key, mut cands) in other.candidates {
            self.candidates
                .get_mut(&key)
                .expect("key registered above")
                .append(&mut cands);
        }
    }

    /// Resolve each key's candidate list into the best single candidate and
    /// build the unified [`Schema`]. Errors with a mode-specific message when
    /// no fields were collected.
    fn finalize(mut self) -> Result<Schema, RSerdeError> {
        let mut unified_fields: Vec<FieldInfo> = Vec::new();
        let mut unified_mappings: HashMap<String, FieldMapping> = HashMap::new();

        for key in &self.key_order {
            let candidates = self.candidates.get_mut(key).unwrap();
            if candidates.is_empty() {
                continue;
            }
            let new_start = unified_fields.len();
            match resolve_candidates(candidates) {
                Candidate::Scalar(col_type) => {
                    unified_fields.push(FieldInfo {
                        name: key.clone(),
                        col_type,
                    });
                    unified_mappings
                        .insert(key.clone(), FieldMapping::Scalar { col_idx: new_start });
                }
                Candidate::Compound { fields, sub_map } => {
                    let col_count = fields.len();
                    // sub_map indices were relative to col_offset=0 in the per-row probe;
                    // remap them to the actual position in the unified layout.
                    let old_base = sub_map.col_start;
                    for field in fields {
                        unified_fields.push(field);
                    }
                    let remapped = remap_field_map(sub_map, old_base, new_start);
                    unified_mappings.insert(
                        key.clone(),
                        FieldMapping::Compound {
                            col_start: new_start,
                            col_count,
                            sub_fields: remapped,
                        },
                    );
                }
            }
        }

        if unified_fields.is_empty() {
            return Err(RSerdeError::Message(match self.mode {
                SchemaMode::SingleRow => "SerdeRowBuilder: first row has no fields".into(),
                SchemaMode::Union => "vec_to_dataframe: no fields discovered from any row".into(),
            }));
        }

        let total = unified_fields.len();
        Ok(Schema {
            fields: unified_fields,
            field_map: FieldMap {
                map: unified_mappings,
                col_start: 0,
                total_cols: total,
            },
        })
    }
}

/// Remap all column indices in a FieldMap from old base to new base.
fn remap_field_map(old: FieldMap, old_base: usize, new_base: usize) -> FieldMap {
    FieldMap {
        map: old
            .map
            .into_iter()
            .map(|(k, v)| (k, remap_field_mapping(v, old_base, new_base)))
            .collect(),
        col_start: new_base,
        total_cols: old.total_cols,
    }
}

fn remap_field_mapping(m: FieldMapping, old_base: usize, new_base: usize) -> FieldMapping {
    match m {
        FieldMapping::Scalar { col_idx } => FieldMapping::Scalar {
            col_idx: col_idx - old_base + new_base,
        },
        FieldMapping::Compound {
            col_start,
            col_count,
            sub_fields,
        } => {
            let new_col_start = col_start - old_base + new_base;
            FieldMapping::Compound {
                col_start: new_col_start,
                col_count,
                sub_fields: remap_field_map(sub_fields, col_start, new_col_start),
            }
        }
    }
}

/// Try to recursively discover and flatten a nested struct value.
/// Returns (flat_fields, field_map) if the value serializes as a struct.
fn try_discover_nested<T: ?Sized + Serialize>(
    value: &T,
    col_offset: usize,
) -> Option<(Vec<FieldInfo>, FieldMap)> {
    // Use single-row discovery for nested values (they're not enums at this level)
    let mut discoverer = SchemaDiscoverer::new(col_offset);
    if value.serialize(&mut discoverer).is_ok() && !discoverer.fields.is_empty() {
        let total = discoverer.fields.len();
        Some((
            discoverer.fields,
            FieldMap {
                map: discoverer.mappings,
                col_start: col_offset,
                total_cols: total,
            },
        ))
    } else {
        None
    }
}

struct SchemaDiscoverer {
    fields: Vec<FieldInfo>,
    mappings: HashMap<String, FieldMapping>,
    key_order: Vec<String>,
    col_offset: usize,
}

impl SchemaDiscoverer {
    fn new(col_offset: usize) -> Self {
        Self {
            fields: Vec::new(),
            mappings: HashMap::new(),
            key_order: Vec::new(),
            col_offset,
        }
    }

    /// Process a field: try to flatten as nested struct, else probe as scalar.
    fn process_field<T: ?Sized + Serialize>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.key_order.push(key.to_string());
        let abs_col = self.col_offset + self.fields.len();
        if let Some((sub_fields, sub_map)) = try_discover_nested(value, abs_col) {
            let count = sub_fields.len();
            for mut field in sub_fields {
                field.name = format!("{key}_{}", field.name);
                self.fields.push(field);
            }
            self.mappings.insert(
                key.to_string(),
                FieldMapping::Compound {
                    col_start: abs_col,
                    col_count: count,
                    sub_fields: sub_map,
                },
            );
        } else {
            let mut type_probe = TypeProbe {
                col_type: ColumnType::Generic,
            };
            let _ = value.serialize(&mut type_probe);
            self.fields.push(FieldInfo {
                name: key.to_string(),
                col_type: type_probe.col_type,
            });
            self.mappings
                .insert(key.to_string(), FieldMapping::Scalar { col_idx: abs_col });
        }
        Ok(())
    }
}

impl<'a> ser::Serializer for &'a mut SchemaDiscoverer {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = SchemaMapDiscoverer<'a>;
    type SerializeStruct = SchemaStructDiscoverer<'a>;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SchemaStructDiscoverer { parent: self })
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Ok(SchemaMapDiscoverer {
            parent: self,
            pending_key: None,
        })
    }

    reject_non_struct!("vec_to_dataframe: expected struct", allow_some_none);
}

struct SchemaStructDiscoverer<'a> {
    parent: &'a mut SchemaDiscoverer,
}

impl ser::SerializeStruct for SchemaStructDiscoverer<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.parent.process_field(key, value)
    }

    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

/// Map-based schema discoverer for structs using `#[serde(flatten)]`.
struct SchemaMapDiscoverer<'a> {
    parent: &'a mut SchemaDiscoverer,
    pending_key: Option<String>,
}

impl ser::SerializeMap for SchemaMapDiscoverer<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        let mut extractor = ValueExtractor::default();
        key.serialize(&mut extractor)?;
        self.pending_key = match extractor.value {
            ExtractedValue::Str(s) => Some(s),
            _ => Some(String::new()),
        };
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        let key = self.pending_key.take().unwrap_or_default();
        self.parent.process_field(&key, value)
    }

    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}
// endregion

// region: Type probe (discovers column type from a single value)

struct TypeProbe {
    col_type: ColumnType,
}

impl ser::Serializer for &mut TypeProbe {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = ser::Impossible<(), RSerdeError>;
    type SerializeStruct = ser::Impossible<(), RSerdeError>;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_bool(self, _: bool) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Logical;
        Ok(())
    }
    fn serialize_i8(self, _: i8) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_i16(self, _: i16) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_i32(self, _: i32) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_i64(self, _: i64) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_u8(self, _: u8) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_u16(self, _: u16) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_u32(self, _: u32) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_u64(self, _: u64) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_f32(self, _: f32) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_f64(self, _: f64) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_char(self, _: char) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Character;
        Ok(())
    }
    fn serialize_str(self, _: &str) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Character;
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Generic;
        Ok(())
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        // Keep existing type (handles Option<T> where first element is None)
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), RSerdeError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Generic;
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Generic;
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Character;
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), RSerdeError> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Generic;
        Ok(())
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
}
// endregion

// region: Column buffers

enum ColumnBuffer {
    Logical(Vec<i32>),
    Integer(Vec<i32>),
    Real(Vec<f64>),
    Character(Vec<Option<String>>),
    Generic(Vec<Option<SEXP>>),
}

impl ColumnBuffer {
    fn new(col_type: ColumnType, capacity: usize) -> Self {
        match col_type {
            ColumnType::Logical => ColumnBuffer::Logical(Vec::with_capacity(capacity)),
            ColumnType::Integer => ColumnBuffer::Integer(Vec::with_capacity(capacity)),
            ColumnType::Real => ColumnBuffer::Real(Vec::with_capacity(capacity)),
            ColumnType::Character => ColumnBuffer::Character(Vec::with_capacity(capacity)),
            ColumnType::Generic => ColumnBuffer::Generic(Vec::with_capacity(capacity)),
        }
    }

    fn push_na(&mut self) {
        match self {
            ColumnBuffer::Logical(v) => v.push(i32::MIN),
            ColumnBuffer::Integer(v) => v.push(i32::MIN),
            ColumnBuffer::Real(v) => v.push(NA_REAL),
            ColumnBuffer::Character(v) => v.push(None),
            ColumnBuffer::Generic(v) => v.push(None),
        }
    }

    fn push_value<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
        scope: &crate::ProtectScope,
    ) -> Result<(), RSerdeError> {
        match self {
            ColumnBuffer::Logical(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Bool(b) => i32::from(b),
                    ExtractedValue::None => i32::MIN, // NA_LOGICAL
                    _ => i32::MIN,
                });
            }
            ColumnBuffer::Integer(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Int(i) => i,
                    ExtractedValue::None => i32::MIN, // NA_INTEGER
                    _ => i32::MIN,
                });
            }
            ColumnBuffer::Real(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Real(f) => f,
                    ExtractedValue::Int(i) => f64::from(i),
                    ExtractedValue::None => NA_REAL,
                    _ => NA_REAL,
                });
            }
            ColumnBuffer::Character(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Str(s) => Some(s),
                    ExtractedValue::None => None, // NA_character_
                    _ => None,
                });
            }
            ColumnBuffer::Generic(v) => {
                // Fall back to full serde serialization for this element.
                // The returned SEXP is freshly allocated and unrooted; root it
                // in the caller's ProtectScope so it survives the GC pressure
                // from subsequent rows' fills and from `assemble_dataframe`'s
                // own allocations.
                let sexp = value.serialize(super::ser::RSerializer)?;
                let rooted = unsafe { scope.protect_raw(sexp) };
                v.push(Some(rooted));
            }
        }
        Ok(())
    }

    /// Push a value without a `ProtectScope`. Used by the parallel fill path
    /// ([`par_iter_to_dataframe`]), which runs off the R main thread where no
    /// protection scope exists and the R API must not be touched.
    ///
    /// The atomic variants are pure-Rust (they extract primitives via
    /// [`ValueExtractor`] and push into a `Vec`), so they work off-thread. The
    /// `Generic` variant calls into the R API (`RSerializer`) and therefore
    /// returns an error here — the parallel path rejects any schema containing
    /// a `Generic` column before fanning out, so this arm is unreachable in
    /// practice and exists only to make the no-scope contract total.
    #[cfg(feature = "rayon")]
    fn push_value_pure<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        match self {
            ColumnBuffer::Generic(_) => Err(RSerdeError::Message(
                "par_iter_to_dataframe: generic (list) columns require the R main thread; \
                 use the sequential iter_to_dataframe for this schema"
                    .into(),
            )),
            // SAFETY of soundness, not memory: a dummy scope is never read by
            // the non-Generic arms of `push_value`, so we route through the
            // same extraction logic without allocating one. We cannot construct
            // a `ProtectScope` off the main thread, so re-implement the atomic
            // arms inline here instead.
            ColumnBuffer::Logical(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Bool(b) => i32::from(b),
                    _ => i32::MIN, // NA_LOGICAL
                });
                Ok(())
            }
            ColumnBuffer::Integer(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Int(i) => i,
                    _ => i32::MIN, // NA_INTEGER
                });
                Ok(())
            }
            ColumnBuffer::Real(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Real(f) => f,
                    ExtractedValue::Int(i) => f64::from(i),
                    _ => NA_REAL,
                });
                Ok(())
            }
            ColumnBuffer::Character(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Str(s) => Some(s),
                    _ => None, // NA_character_
                });
                Ok(())
            }
        }
    }

    /// Append all elements of `other` onto `self`, consuming `other`.
    ///
    /// Both buffers must be the same variant (guaranteed by the parallel merge,
    /// which builds every chunk's columns from the same shared schema). Used to
    /// concat per-thread chunk buffers back into one column in row order.
    #[cfg(feature = "rayon")]
    fn append(&mut self, other: ColumnBuffer) {
        match (self, other) {
            (ColumnBuffer::Logical(a), ColumnBuffer::Logical(mut b)) => a.append(&mut b),
            (ColumnBuffer::Integer(a), ColumnBuffer::Integer(mut b)) => a.append(&mut b),
            (ColumnBuffer::Real(a), ColumnBuffer::Real(mut b)) => a.append(&mut b),
            (ColumnBuffer::Character(a), ColumnBuffer::Character(mut b)) => a.append(&mut b),
            (ColumnBuffer::Generic(a), ColumnBuffer::Generic(mut b)) => a.append(&mut b),
            _ => unreachable!("ColumnBuffer::append: mismatched variants from shared schema"),
        }
    }
}
// endregion

// region: Value extraction

#[derive(Default)]
struct ValueExtractor {
    value: ExtractedValue,
}

#[derive(Default)]
enum ExtractedValue {
    #[default]
    None,
    Bool(bool),
    Int(i32),
    Real(f64),
    Str(String),
}

impl ser::Serializer for &mut ValueExtractor {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = ser::Impossible<(), RSerdeError>;
    type SerializeStruct = ser::Impossible<(), RSerdeError>;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_bool(self, v: bool) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Bool(v);
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(i32::from(v));
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(i32::from(v));
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(v);
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Real(v as f64);
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(i32::from(v));
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(i32::from(v));
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<(), RSerdeError> {
        if let Ok(i) = i32::try_from(v) {
            self.value = ExtractedValue::Int(i);
        } else {
            self.value = ExtractedValue::Real(f64::from(v));
        }
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Real(v as f64);
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Real(f64::from(v));
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Real(v);
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Str(v.to_string());
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Str(v.to_string());
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::None;
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), RSerdeError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::None;
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::None;
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        v: &'static str,
    ) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Str(v.to_string());
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), RSerdeError> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
}
// endregion

// region: Column filling (name-based, handles skip_serializing_if)

/// Column filler. Dispatches each field by name to the correct column(s).
///
/// When `is_top_level` is true, this is the top-level row filler: `pad_unfilled`
/// resets filled flags for the next row. When false, this is a sub-filler for
/// nested struct fields: `pad_unfilled` marks columns as filled (the top-level
/// filler handles the reset), and `serialize_some`/`serialize_none` support
/// Option-wrapped nested structs with NA-fill logic.
struct ColumnFiller<'a> {
    columns: &'a mut [ColumnBuffer],
    field_map: &'a FieldMap,
    filled: &'a mut Vec<bool>,
    col_start: usize,
    col_count: usize,
    is_top_level: bool,
    pending_key: Option<String>,
    scope: &'a crate::ProtectScope,
    /// When true, fields not in the schema produce an error instead of being
    /// silently ignored. `SerdeRowBuilder` sets this to enforce first-row
    /// schema; `vec_to_dataframe` leaves it false because its
    /// schema is the union of all rows (so misses can't happen).
    strict: bool,
}

impl ColumnFiller<'_> {
    fn fill_field<T: ?Sized + Serialize>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        match self.field_map.map.get(key) {
            Some(FieldMapping::Scalar { col_idx }) => {
                self.columns[*col_idx].push_value(value, self.scope)?;
                self.filled[*col_idx] = true;
            }
            Some(FieldMapping::Compound {
                sub_fields,
                col_start,
                col_count,
                ..
            }) => {
                let sub = ColumnFiller {
                    columns: self.columns,
                    field_map: sub_fields,
                    filled: self.filled,
                    col_start: *col_start,
                    col_count: *col_count,
                    is_top_level: false,
                    pending_key: None,
                    scope: self.scope,
                    strict: self.strict,
                };
                value.serialize(sub)?;
            }
            None => {
                // Field not in schema. The union-schema path (from_rows) never
                // hits this because the schema absorbed every row's keys; the
                // streaming path (SerdeRowBuilder, strict=true) does, when a
                // later row introduces a new field.
                if self.strict {
                    return Err(RSerdeError::Message(format!(
                        "SerdeRowBuilder: row introduced field {key:?} not in initial schema"
                    )));
                }
            }
        }
        Ok(())
    }

    fn pad_unfilled(&mut self) {
        let start = self.field_map.col_start;
        let end = start + self.field_map.total_cols;
        if self.is_top_level {
            for i in start..end {
                if !self.filled[i] {
                    self.columns[i].push_na();
                }
                self.filled[i] = false; // reset for next row
            }
        } else {
            for i in start..end {
                if !self.filled[i] {
                    self.columns[i].push_na();
                }
                // Don't reset — the top-level filler handles reset
                self.filled[i] = true;
            }
        }
    }
}

impl<'a> ser::Serializer for ColumnFiller<'a> {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Ok(self)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), RSerdeError> {
        if self.is_top_level {
            return Err(RSerdeError::Message("expected struct".into()));
        }
        value.serialize(self)
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        if self.is_top_level {
            return Err(RSerdeError::Message("expected struct".into()));
        }
        // Fill all columns owned by this sub-filler with NA
        for i in self.col_start..self.col_start + self.col_count {
            self.columns[i].push_na();
            self.filled[i] = true;
        }
        Ok(())
    }

    reject_non_struct!(@primitives "expected struct");
}

impl ser::SerializeStruct for ColumnFiller<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.fill_field(key, value)
    }

    fn end(mut self) -> Result<(), RSerdeError> {
        self.pad_unfilled();
        Ok(())
    }
}

impl ser::SerializeMap for ColumnFiller<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        let mut extractor = ValueExtractor::default();
        key.serialize(&mut extractor)?;
        self.pending_key = match extractor.value {
            ExtractedValue::Str(s) => Some(s),
            _ => Some(String::new()),
        };
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        let key = self.pending_key.take().unwrap_or_default();
        self.fill_field(&key, value)
    }

    fn end(mut self) -> Result<(), RSerdeError> {
        self.pad_unfilled();
        Ok(())
    }
}
// endregion

// region: Data.frame assembly

/// Build an empty R data.frame (0 rows, 0 columns).
fn empty_dataframe() -> SEXP {
    unsafe {
        let scope = crate::ProtectScope::new();
        let list = crate::ListBuilder::new(&scope, 0).into_sexp();

        // Set class = "data.frame"
        list.set_class(crate::cached_class::data_frame_class_sexp());

        // Set compact row.names: c(NA_integer_, 0)
        let (row_names, rn) = crate::into_r::alloc_r_vector::<i32>(2);
        scope.protect_raw(row_names);
        rn[0] = i32::MIN; // NA_integer_
        rn[1] = 0;
        list.set_row_names(row_names);

        list
    }
}

/// Assemble column buffers into a data.frame, root it as a [`BuiltDataFrame`],
/// then drop `scope` only after the assembled VECSXP has copied every protected
/// element. Rooting happens *before* the scope drop, so the frame is never
/// unrooted — not even momentarily.
///
/// Centralises the protect-scope drop discipline shared by
/// [`vec_to_dataframe`] and [`SerdeRowBuilder::finish`].
///
/// # Safety
///
/// Must be called from the R main thread. All column buffers must have
/// exactly `nrow` elements.
unsafe fn assemble_with_scope(
    schema: &Schema,
    columns: &[ColumnBuffer],
    nrow: usize,
    scope: crate::ProtectScope,
) -> BuiltDataFrame {
    // SAFETY: `assemble_dataframe` builds a well-formed data.frame SEXP from the
    // column buffers; the caller upholds the main-thread + length invariants.
    // `adopt_sexp` roots the SEXP before `scope` releases its per-column
    // protects, so the frame is continuously rooted.
    let built =
        unsafe { BuiltDataFrame::adopt_sexp(assemble_dataframe(&schema.fields, columns, nrow)) };
    drop(scope);
    built
}

/// Assemble column buffers into an R data.frame SEXP.
///
/// # Safety
///
/// Must be called from the R main thread. All column buffers must have
/// exactly `nrow` elements.
unsafe fn assemble_dataframe(fields: &[FieldInfo], columns: &[ColumnBuffer], nrow: usize) -> SEXP {
    let ncol: usize = fields.len();

    unsafe {
        let scope = crate::ProtectScope::new();
        let builder = crate::ListBuilder::new(&scope, ncol);
        let list = builder.as_sexp();

        // Build each column and set into list. `set_protected` protects the
        // freshly-built column SEXP across `set_vector_elt`, then drops the
        // guard once the parent VECSXP owns it — balancing automatically.
        for (i, col) in columns.iter().enumerate() {
            let idx: isize = i.try_into().expect("column index exceeds isize::MAX");
            let col_sexp = column_to_sexp(col, nrow);
            builder.set_protected(idx, col_sexp);
        }

        // Set names. `StrVecBuilder` allocates the STRSXP rooted in `scope`.
        let names_builder = crate::StrVecBuilder::new(&scope, ncol);
        for (i, field) in fields.iter().enumerate() {
            let idx: isize = i.try_into().expect("field index exceeds isize::MAX");
            names_builder.set_str(idx, &field.name);
        }
        list.set_names(names_builder.into_sexp());

        // Set class = "data.frame"
        list.set_class(crate::cached_class::data_frame_class_sexp());

        // Set compact row.names: c(NA_integer_, -nrow)
        let (row_names, rn) = crate::into_r::alloc_r_vector::<i32>(2);
        scope.protect_raw(row_names);
        rn[0] = i32::MIN; // NA_integer_
        rn[1] = -i32::try_from(nrow).expect("data.frame row count exceeds i32::MAX");
        list.set_row_names(row_names);

        list
    }
}

/// Convert a single column buffer into an R SEXP vector.
unsafe fn column_to_sexp(col: &ColumnBuffer, nrow: usize) -> SEXP {
    use crate::into_r::alloc_r_vector;

    unsafe {
        match col {
            ColumnBuffer::Logical(v) => {
                let (sexp, dst) = alloc_r_vector::<crate::RLogical>(nrow);
                let dst_i32: &mut [i32] =
                    std::slice::from_raw_parts_mut(dst.as_mut_ptr().cast::<i32>(), nrow);
                dst_i32.copy_from_slice(v);
                sexp
            }
            ColumnBuffer::Integer(v) => {
                let (sexp, dst) = alloc_r_vector::<i32>(nrow);
                dst.copy_from_slice(v);
                sexp
            }
            ColumnBuffer::Real(v) => {
                let (sexp, dst) = alloc_r_vector::<f64>(nrow);
                dst.copy_from_slice(v);
                sexp
            }
            ColumnBuffer::Character(v) => {
                let nrow_r: isize = nrow.try_into().expect("nrow exceeds isize::MAX");
                // PROTECT discipline: SEXP::charsxp (Rf_mkCharLenCE) allocates
                // and can trigger GC under gctorture, which would reclaim our
                // unprotected STRSXP. The OwnedProtect guard protects the
                // freshly-allocated STRSXP and drops (UNPROTECT(1)) on return.
                let guard = crate::OwnedProtect::new(SEXP::alloc(SEXPTYPE::STRSXP, nrow_r));
                let sexp = guard.get();
                for (i, val) in v.iter().enumerate() {
                    let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                    match val {
                        // Preserve the R_BlankString short-circuit: skips the
                        // CHARSXP hash-lookup on the interned empty string.
                        Some(s) if s.is_empty() => {
                            sexp.set_string_elt(idx, SEXP::blank_string());
                        }
                        Some(s) => {
                            sexp.set_string_elt(idx, SEXP::charsxp(s));
                        }
                        None => {
                            sexp.set_string_elt(idx, SEXP::na_string());
                        }
                    }
                }
                sexp
            }
            ColumnBuffer::Generic(v) => {
                let nrow_r: isize = nrow.try_into().expect("nrow exceeds isize::MAX");
                // If every entry is None or Some(NULL) — meaning all rows had
                // `Option<T> = None` (which serializes as NULL) or the column
                // was always NA-padded — emit a logical NA vector instead of
                // list(NULL, …).  R coerces logical NA to the surrounding type
                // on first use, so this is invisible downstream:
                //   c(NA, 1L) → integer,  c(NA, "x") → character, etc.
                //
                // `push_na` (pad for missing rows) stores `None`.
                // `push_value(&None::<T>)` stores `Some(SEXP::nil())` via
                // RSerializer::serialize_none.
                // Both are "NA-like" in the generic-list context.
                let all_null = v.iter().all(|e| match e {
                    None => true,
                    Some(s) => s.is_nil(),
                });
                if all_null {
                    let (sexp, dst) = alloc_r_vector::<crate::RLogical>(nrow);
                    let dst_i32: &mut [i32] =
                        std::slice::from_raw_parts_mut(dst.as_mut_ptr().cast::<i32>(), nrow);
                    dst_i32.fill(NA_LOGICAL);
                    return sexp;
                }
                // No allocation occurs in the loop below — every element is a
                // pre-existing SEXP from the buffer or the R_NilValue singleton
                // — so the freshly-allocated VECSXP needs no protection here.
                let sexp = SEXP::alloc(SEXPTYPE::VECSXP, nrow_r);
                for (i, val) in v.iter().enumerate() {
                    let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                    if let Some(elem) = val {
                        sexp.set_vector_elt(idx, *elem);
                    } else {
                        sexp.set_vector_elt(idx, SEXP::nil());
                    }
                }
                sexp
            }
        }
    }
}
// endregion

// region: Enum split (vec_to_dataframe_split)

struct VariantInfo {
    name: String,
    is_unit: bool,
    tag_field: Option<String>,
}

fn extract_variant_info<T: Serialize>(row: &T) -> Option<VariantInfo> {
    let mut ext = VariantNameExtractor::default();
    let _ = row.serialize(&mut ext);
    ext.name.map(|name| VariantInfo {
        name,
        is_unit: ext.is_unit,
        tag_field: ext.tag_field,
    })
}

// region: VariantNameExtractor

#[derive(Default)]
struct VariantNameExtractor {
    name: Option<String>,
    is_unit: bool,
    tag_field: Option<String>,
    /// Set when the value serialized via `serialize_none` (an `Option::None`).
    /// Lets a single extraction pass distinguish `None` (no variant, NA-fill)
    /// from a genuine non-enum value without a second probe serializer.
    is_none: bool,
}

struct NoopStructVariant;
impl ser::SerializeStructVariant for NoopStructVariant {
    type Ok = ();
    type Error = RSerdeError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

struct NoopTupleVariant;
impl ser::SerializeTupleVariant for NoopTupleVariant {
    type Ok = ();
    type Error = RSerdeError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, _: &T) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

struct TagStructCapture<'a> {
    parent: &'a mut VariantNameExtractor,
    first_done: bool,
}

impl ser::SerializeStruct for TagStructCapture<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        if !self.first_done {
            self.first_done = true;
            let mut ve = ValueExtractor::default();
            let _ = value.serialize(&mut ve);
            if let ExtractedValue::Str(s) = ve.value {
                self.parent.name = Some(s);
                self.parent.tag_field = Some(key.to_string());
            }
        }
        Ok(())
    }

    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

// Defends against custom `Serialize` impls that emit internally-tagged enums via
// `serialize_map` rather than `serialize_struct`. `#[derive(Serialize)]` always
// uses `serialize_struct` for internally-tagged enums, so this path doesn't fire
// for derive-generated impls — but hand-written serializers may use a map.
struct TagMapCapture<'a> {
    parent: &'a mut VariantNameExtractor,
    pending_key: Option<String>,
    first_done: bool,
}

impl ser::SerializeMap for TagMapCapture<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        if !self.first_done {
            let mut ve = ValueExtractor::default();
            let _ = key.serialize(&mut ve);
            if let ExtractedValue::Str(s) = ve.value {
                self.pending_key = Some(s);
            }
        }
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        if !self.first_done {
            self.first_done = true;
            let key = self.pending_key.take().unwrap_or_default();
            let mut ve = ValueExtractor::default();
            let _ = value.serialize(&mut ve);
            if let ExtractedValue::Str(s) = ve.value {
                self.parent.name = Some(s);
                self.parent.tag_field = Some(key);
            }
        }
        Ok(())
    }

    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

impl<'a> ser::Serializer for &'a mut VariantNameExtractor {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = NoopTupleVariant;
    type SerializeMap = TagMapCapture<'a>;
    type SerializeStruct = TagStructCapture<'a>;
    type SerializeStructVariant = NoopStructVariant;

    fn serialize_bool(self, _: bool) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_i8(self, _: i8) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_i16(self, _: i16) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_i32(self, _: i32) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_i64(self, _: i64) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_u8(self, _: u8) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_u16(self, _: u16) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_u32(self, _: u32) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_u64(self, _: u64) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_f32(self, _: f32) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_f64(self, _: f64) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_char(self, _: char) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_str(self, _: &str) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        self.is_none = true;
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<(), RSerdeError> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<(), RSerdeError> {
        self.name = Some(variant.to_string());
        self.is_unit = true;
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), RSerdeError> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        self.name = Some(variant.to_string());
        Ok(())
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, RSerdeError> {
        Err(RSerdeError::Message("seq in variant extractor".into()))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, RSerdeError> {
        Err(RSerdeError::Message("tuple in variant extractor".into()))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, RSerdeError> {
        Err(RSerdeError::Message(
            "tuple_struct in variant extractor".into(),
        ))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, RSerdeError> {
        self.name = Some(variant.to_string());
        Ok(NoopTupleVariant)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Ok(TagMapCapture {
            parent: self,
            pending_key: None,
            first_done: false,
        })
    }
    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, RSerdeError> {
        Ok(TagStructCapture {
            parent: self,
            first_done: false,
        })
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, RSerdeError> {
        self.name = Some(variant.to_string());
        Ok(NoopStructVariant)
    }
}
// endregion

// region: VariantStrippingSerializer

struct VariantPayload<T>(T);

impl<T: Serialize> Serialize for VariantPayload<T> {
    fn serialize<S: ser::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(VariantStrippingSerializer { inner: s })
    }
}

struct VariantStrippingSerializer<S: ser::Serializer> {
    inner: S,
}

struct VariantAsStruct<S: ser::SerializeStruct>(S);

impl<S: ser::SerializeStruct> ser::SerializeStructVariant for VariantAsStruct<S> {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), S::Error> {
        self.0.serialize_field(key, value)
    }
    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

struct VariantAsTupleStruct<S: ser::SerializeTupleStruct>(S);

impl<S: ser::SerializeTupleStruct> ser::SerializeTupleVariant for VariantAsTupleStruct<S> {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), S::Error> {
        self.0.serialize_field(value)
    }
    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

impl<S: ser::Serializer> ser::Serializer for VariantStrippingSerializer<S> {
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = S::SerializeSeq;
    type SerializeTuple = S::SerializeTuple;
    type SerializeTupleStruct = S::SerializeTupleStruct;
    type SerializeTupleVariant = VariantAsTupleStruct<S::SerializeTupleStruct>;
    type SerializeMap = S::SerializeMap;
    type SerializeStruct = S::SerializeStruct;
    type SerializeStructVariant = VariantAsStruct<S::SerializeStruct>;

    fn serialize_bool(self, v: bool) -> Result<S::Ok, S::Error> {
        self.inner.serialize_bool(v)
    }
    fn serialize_i8(self, v: i8) -> Result<S::Ok, S::Error> {
        self.inner.serialize_i8(v)
    }
    fn serialize_i16(self, v: i16) -> Result<S::Ok, S::Error> {
        self.inner.serialize_i16(v)
    }
    fn serialize_i32(self, v: i32) -> Result<S::Ok, S::Error> {
        self.inner.serialize_i32(v)
    }
    fn serialize_i64(self, v: i64) -> Result<S::Ok, S::Error> {
        self.inner.serialize_i64(v)
    }
    fn serialize_u8(self, v: u8) -> Result<S::Ok, S::Error> {
        self.inner.serialize_u8(v)
    }
    fn serialize_u16(self, v: u16) -> Result<S::Ok, S::Error> {
        self.inner.serialize_u16(v)
    }
    fn serialize_u32(self, v: u32) -> Result<S::Ok, S::Error> {
        self.inner.serialize_u32(v)
    }
    fn serialize_u64(self, v: u64) -> Result<S::Ok, S::Error> {
        self.inner.serialize_u64(v)
    }
    fn serialize_f32(self, v: f32) -> Result<S::Ok, S::Error> {
        self.inner.serialize_f32(v)
    }
    fn serialize_f64(self, v: f64) -> Result<S::Ok, S::Error> {
        self.inner.serialize_f64(v)
    }
    fn serialize_char(self, v: char) -> Result<S::Ok, S::Error> {
        self.inner.serialize_char(v)
    }
    fn serialize_str(self, v: &str) -> Result<S::Ok, S::Error> {
        self.inner.serialize_str(v)
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<S::Ok, S::Error> {
        self.inner.serialize_bytes(v)
    }
    fn serialize_none(self) -> Result<S::Ok, S::Error> {
        self.inner.serialize_none()
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<S::Ok, S::Error> {
        self.inner.serialize_some(v)
    }
    fn serialize_unit(self) -> Result<S::Ok, S::Error> {
        self.inner.serialize_unit()
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<S::Ok, S::Error> {
        self.inner.serialize_unit_struct(name)
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<S::Ok, S::Error> {
        self.inner.serialize_unit_struct(variant)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        v: &T,
    ) -> Result<S::Ok, S::Error> {
        self.inner.serialize_newtype_struct(name, v)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        v: &T,
    ) -> Result<S::Ok, S::Error> {
        v.serialize(self.inner)
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<S::SerializeSeq, S::Error> {
        self.inner.serialize_seq(len)
    }
    fn serialize_tuple(self, len: usize) -> Result<S::SerializeTuple, S::Error> {
        self.inner.serialize_tuple(len)
    }
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<S::SerializeTupleStruct, S::Error> {
        self.inner.serialize_tuple_struct(name, len)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, S::Error> {
        let ts = self.inner.serialize_tuple_struct(variant, len)?;
        Ok(VariantAsTupleStruct(ts))
    }
    fn serialize_map(self, len: Option<usize>) -> Result<S::SerializeMap, S::Error> {
        self.inner.serialize_map(len)
    }
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<S::SerializeStruct, S::Error> {
        self.inner.serialize_struct(name, len)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, S::Error> {
        let s = self.inner.serialize_struct(variant, len)?;
        Ok(VariantAsStruct(s))
    }
}
// endregion

// region: 0-column data.frame for unit variants

fn unit_variant_dataframe(nrow: usize) -> SEXP {
    unsafe {
        let scope = crate::ProtectScope::new();
        let list = crate::ListBuilder::new(&scope, 0).into_sexp();
        list.set_class(crate::cached_class::data_frame_class_sexp());
        let (row_names, rn) = crate::into_r::alloc_r_vector::<i32>(2);
        scope.protect_raw(row_names);
        rn[0] = i32::MIN;
        rn[1] = -i32::try_from(nrow).expect("nrow overflow");
        list.set_row_names(row_names);
        list
    }
}
// endregion

// region: vec_to_dataframe_split

/// Output-shape selector for [`vec_to_dataframe_split`].
///
/// Configures whether per-variant data.frames carry an explicit variant-tag
/// column, and whether the result is one list per variant or a single
/// collated data.frame with the variant name on every row.
///
/// The variant name on the R side is whatever serde emits (PascalCase by
/// default). Override with `#[serde(rename_all = "snake_case")]` (or
/// similar) on the enum definition.
pub enum SplitShape {
    /// `list(VariantA = df, VariantB = df, …)` — historical behaviour.
    ///
    /// Single-variant input short-circuits to a bare data.frame instead of a
    /// one-element list. Per-variant data.frames do not carry the variant
    /// name — it lives on the list-element name.
    PerVariantList,

    /// Same shape as [`PerVariantList`](Self::PerVariantList) but each
    /// per-variant data.frame gets a leading column whose name is
    /// `column` and whose values are the variant name repeated nrow times.
    ///
    /// Use when the per-variant data.frame is going to be passed through
    /// `bind_rows` / `rbind` downstream and the variant tag needs to
    /// survive that pass.
    PerVariantListWithTag { column: String },

    /// Single collated data.frame containing the union of every variant's
    /// fields plus a leading variant-tag column named `column`. Fields
    /// belonging to other variants land as NA per row.
    ///
    /// Returns an error on empty input — the variant set is unknowable
    /// from zero rows so the union schema cannot be inferred.
    Collated { column: String },
}

/// Categorical return shape for the dataframe-helpers family
/// ([`vec_to_dataframe_split`] / [`result_to_dataframe`]).
///
/// Carries enough type information that downstream Rust code can `match`
/// on the variant without dispatching on SEXP type. Convert to a SEXP via
/// the [`crate::IntoR`] impl (or just return it from a `#[miniextendr]`
/// fn), which collapses every variant to the equivalent R value (bare
/// data.frame / named list of data.frames / `list(results=, error=)`).
///
/// # GC discipline (rooted by construction, #1265)
///
/// Every frame inside the shape is an owned, GC-rooted
/// [`BuiltDataFrame`], and the
/// [`SplitResults::None`] sentinel carries its own [`RootedSentinel`]
/// root — so holding a `DataFrameShape` across R allocations is safe by
/// construction; there is no convert-immediately contract. The
/// [`crate::IntoR`] conversion consumes the handles, re-rooting each
/// child in the assembled R value.
pub enum DataFrameShape {
    /// Single data.frame.
    ///
    /// Used by:
    /// - [`vec_to_dataframe_split`] when only one variant is present (and
    ///   [`PerVariantList`](SplitShape::PerVariantList) or
    ///   [`PerVariantListWithTag`](SplitShape::PerVariantListWithTag) is
    ///   selected — the single-variant short-circuit) or always under
    ///   [`Collated`](SplitShape::Collated).
    /// - [`result_to_dataframe`] under [`Auto`](ResultShape::Auto) when
    ///   every row is `Ok`, and always under
    ///   [`Collated`](ResultShape::Collated).
    Bare(BuiltDataFrame),

    /// `list(results = <df | sentinel>, error = df)`.
    ///
    /// Produced by [`result_to_dataframe`] under
    /// [`Auto`](ResultShape::Auto) when at least one `Err` is present, and
    /// always under [`Split`](ResultShape::Split).
    Split {
        /// The Ok partition.
        results: SplitResults,
        /// The error partition (always present, possibly zero-row).
        error: BuiltDataFrame,
    },

    /// `list(VariantA = df, VariantB = df, …)`.
    ///
    /// Produced by [`vec_to_dataframe_split`] under
    /// [`PerVariantList`](SplitShape::PerVariantList) /
    /// [`PerVariantListWithTag`](SplitShape::PerVariantListWithTag) when
    /// the input contains more than one variant. Order matches first-seen
    /// order in the input slice.
    PerVariantList(Vec<(String, BuiltDataFrame)>),
}

/// Result partition for [`DataFrameShape::Split`].
///
/// Used to distinguish "no Ok rows at all" (which lets the caller supply
/// a sentinel value such as `NULL`, `NA`, `FALSE`, …) from a real
/// zero-row data.frame.
pub enum SplitResults {
    /// At least one `Ok` row — partition has a concrete data.frame.
    Some(BuiltDataFrame),
    /// No `Ok` rows — sentinel value supplied by the caller via
    /// `empty_ok_sentinel`, GC-rooted by its [`RootedSentinel`] handle.
    ///
    /// The handle is consumed by the [`crate::IntoR`] impl on
    /// [`DataFrameShape`]; until then its own root keeps the sentinel
    /// reachable (#1265) — no immediate-conversion contract for this leg.
    None(RootedSentinel),
}

/// GC-rooted holder for the empty-`Ok` sentinel in [`SplitResults::None`].
///
/// The sentinel is an arbitrary caller-supplied R value (`NULL`, `NA`,
/// `FALSE`, a zero-row data.frame, …), so it cannot ride in a
/// [`BuiltDataFrame`]; this is the same
/// RAII pattern (`R_PreserveObject` on construction, `R_ReleaseObject` on
/// drop, `!Send`) for one bare SEXP. It exists so every leg of
/// [`DataFrameShape`] is rooted by construction (#1265).
pub struct RootedSentinel {
    sexp: SEXP,
    /// `!Send + !Sync`: rooting/unrooting mutates R's global precious
    /// list, which is R-main-thread state.
    _not_send: std::marker::PhantomData<*mut ()>,
}

impl RootedSentinel {
    /// Convert `value` to a SEXP and take ownership of a fresh root.
    ///
    /// Rooting is immediate: no allocation happens between the
    /// `into_sexp` return and `R_PreserveObject`, and `R_PreserveObject`
    /// keeps its argument reachable across its own cons-cell allocation.
    pub fn new(value: impl crate::IntoR) -> Self {
        let sexp = value.into_sexp();
        // SAFETY: `into_sexp` just ran on the R main thread and returned a
        // valid SEXP; rooting is immediate (see above).
        unsafe {
            crate::sys::R_PreserveObject(sexp);
        }
        Self {
            sexp,
            _not_send: std::marker::PhantomData,
        }
    }

    /// The rooted SEXP. Stays reachable for as long as this handle lives.
    pub fn as_sexp(&self) -> SEXP {
        self.sexp
    }
}

impl Drop for RootedSentinel {
    fn drop(&mut self) {
        // SAFETY: `!Send` guarantees drop runs on the constructing (R main)
        // thread; release the exact root added in `new`. `R_ReleaseObject`
        // does not allocate.
        unsafe { crate::sys::R_ReleaseObject(self.sexp) };
    }
}

impl crate::IntoR for DataFrameShape {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        match self {
            DataFrameShape::Bare(df) => Ok(df.into_sexp()),
            DataFrameShape::Split { results, error } => {
                // Protect both children via the NamedDataFrameListBuilder's
                // scope so neither is reaped between the set_vector_elt /
                // CHARSXP allocations in from_raw_pairs. Each owned handle
                // stays alive until after its child is protected in the
                // builder's scope, then drops alloc-free — the child is
                // rooted at every instant.
                let mut builder = NamedDataFrameListBuilder::with_capacity(2);
                builder = match results {
                    // `push` protects the view; the `df` handle's own root
                    // releases at the end of this arm, after the protect.
                    SplitResults::Some(df) => builder.push("results", *df),
                    SplitResults::None(sentinel) => {
                        // SAFETY: `sentinel` wraps a valid SEXP kept alive by
                        // its own root; `push_raw` protects it in the
                        // builder's scope before the handle drops at the end
                        // of this arm.
                        unsafe { builder.push_raw("results", sentinel.as_sexp()) }
                    }
                };
                // As above: `error`'s root holds until this arm ends, past
                // the protect inside `push`.
                builder = builder.push("error", *error);
                Ok(builder.build().into_sexp())
            }
            DataFrameShape::PerVariantList(pairs) => {
                let mut builder = NamedDataFrameListBuilder::with_capacity(pairs.len());
                for (name, df) in pairs {
                    // `push` protects the view; the `df` handle's own root
                    // releases at the end of the iteration, after the protect.
                    builder = builder.push(name, *df);
                }
                Ok(builder.build().into_sexp())
            }
        }
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
}

/// Partition a slice of serializable enum rows into per-variant data.frames.
///
/// The output shape is selected via [`SplitShape`]:
///
/// - [`PerVariantList`](SplitShape::PerVariantList) returns the historical
///   `list(VariantA = df, …)` shape (single-variant short-circuit to a
///   bare data.frame).
/// - [`PerVariantListWithTag`](SplitShape::PerVariantListWithTag) is the
///   same shape but each per-variant data.frame carries a leading
///   variant-tag column. Use when downstream `rbind`/`bind_rows` needs
///   the tag to survive.
/// - [`Collated`](SplitShape::Collated) returns one data.frame with all
///   variants stacked, plus a leading variant-tag column. Other-variant
///   fields are NA-filled per row.
///
/// Each variant's per-variant data.frame contains only that variant's
/// fields. For internally-tagged enums (`#[serde(tag = "...")]`), the
/// implicit tag column is dropped from each partition before any explicit
/// tag column is added back.
///
/// Variant-name casing: whatever serde emits. PascalCase by default;
/// override with `#[serde(rename_all = "snake_case")]` on the enum.
///
/// # Errors
///
/// - Any row serializes without a variant name (i.e. it's not an enum) —
///   use [`vec_to_dataframe`] for plain structs instead.
/// - [`Collated`](SplitShape::Collated) on empty input — the variant set
///   is unknowable.
/// - Underlying column-buffer assembly fails.
pub fn vec_to_dataframe_split<T: Serialize>(
    rows: &[T],
    shape: SplitShape,
) -> Result<DataFrameShape, RSerdeError> {
    if rows.is_empty() {
        return match shape {
            SplitShape::PerVariantList | SplitShape::PerVariantListWithTag { .. } => {
                Ok(DataFrameShape::PerVariantList(Vec::new()))
            }
            SplitShape::Collated { .. } => Err(RSerdeError::Message(
                "vec_to_dataframe_split(Collated): empty input — variant set is unknowable".into(),
            )),
        };
    }

    // Phase 1: extract variant info for each row.
    let infos: Vec<VariantInfo> = {
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            match extract_variant_info(row) {
                Some(info) => out.push(info),
                None => {
                    return Err(RSerdeError::Message(
                        "vec_to_dataframe_split: row has no variant — use vec_to_dataframe for plain structs".into(),
                    ));
                }
            }
        }
        out
    };

    // Detect internally-tagged enum style from the first row carrying one.
    let tag_field: Option<&str> = infos.iter().find_map(|i| i.tag_field.as_deref());

    match shape {
        SplitShape::Collated { column } => {
            // One data.frame, union schema across variants, leading tag column.
            // We synthesize TaggedVariantRow wrappers that emit the tag column
            // first and then route the row's payload through the existing
            // variant-stripping serializer so externally-tagged enums work too.
            let wrapped: Vec<TaggedVariantRow<'_, T>> = rows
                .iter()
                .zip(infos.iter())
                .map(|(row, info)| TaggedVariantRow {
                    tag_column: column.as_str(),
                    tag_value: info.name.as_str(),
                    inner: row,
                    tag_field: tag_field.map(str::to_string),
                })
                .collect();
            // `DataFrameShape::Bare` carries the owned, GC-rooted handle
            // (#1265) — the frame stays rooted for as long as the caller
            // holds the shape.
            Ok(DataFrameShape::Bare(vec_to_dataframe(&wrapped)?))
        }
        SplitShape::PerVariantList | SplitShape::PerVariantListWithTag { .. } => {
            // Group indices by variant name (preserve first-seen order).
            let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
            for (i, info) in infos.iter().enumerate() {
                if let Some(grp) = groups.iter_mut().find(|(n, _)| n == &info.name) {
                    grp.1.push(i);
                } else {
                    groups.push((info.name.clone(), vec![i]));
                }
            }

            let tag_column: Option<String> = match &shape {
                SplitShape::PerVariantListWithTag { column } => Some(column.clone()),
                _ => None,
            };

            // Build per-partition data.frames. We don't use NamedDataFrameListBuilder
            // here because we want to return owned frames, not a List.
            // Single-variant short-circuit needs to inspect the count below.
            //
            // Each partition rides in an owned, GC-rooted `BuiltDataFrame`
            // (#1265): later iterations allocate (vec_to_dataframe,
            // make_strsxp_repeat, prepend_column), and the handles keep
            // earlier partitions reachable — no ProtectScope re-rooting
            // dance, and the roots travel with the returned shape instead of
            // releasing when this fn returns.
            let mut partitions: Vec<(String, BuiltDataFrame)> = Vec::with_capacity(groups.len());

            for (name, indices) in &groups {
                let is_unit = infos[indices[0]].is_unit;

                let df: BuiltDataFrame = if is_unit {
                    let sexp = unit_variant_dataframe(indices.len());
                    // SAFETY: `unit_variant_dataframe` returns a well-formed,
                    // freshly-built data.frame SEXP; rooting is immediate (no
                    // allocation before `R_PreserveObject`).
                    unsafe { BuiltDataFrame::adopt_sexp(sexp) }
                } else if tag_field.is_some() {
                    let refs: Vec<&T> = indices.iter().map(|&i| &rows[i]).collect();
                    let built = vec_to_dataframe(&refs)?;
                    if let Some(tf) = tag_field {
                        // `drop` (the inherent `BuiltDataFrame` forward, #1247)
                        // consumes `built` and returns a rooted handle for the
                        // edited frame.
                        built.drop(tf)
                    } else {
                        built
                    }
                } else {
                    let wrapped: Vec<VariantPayload<&T>> =
                        indices.iter().map(|&i| VariantPayload(&rows[i])).collect();
                    vec_to_dataframe(&wrapped)?
                };

                let df = if let Some(col_name) = tag_column.as_deref() {
                    // SAFETY: R main thread. `make_strsxp_repeat` returns an
                    // unprotected STRSXP; protect across `prepend_column`'s
                    // internal allocations (drop → alloc_list → alloc_strsxp).
                    let tag_protect = unsafe {
                        crate::OwnedProtect::new(make_strsxp_repeat(name, indices.len()))
                    };
                    // `prepend_column` (the inherent forward, #1247) consumes
                    // `df` — whose root keeps the input frame reachable
                    // throughout the edit — and returns a rooted handle for
                    // the tagged frame. `tag_protect` drops right after.
                    df.prepend_column(col_name, *tag_protect)
                } else {
                    df
                };

                partitions.push((name.clone(), df));
            }

            // Single-variant short-circuit: collapse to a bare data.frame. The
            // variant name lives in the tag column already if requested; for
            // PerVariantList it lives on neither side (historical behaviour).
            if partitions.len() == 1 {
                let (_name, df) = partitions.into_iter().next().expect("len == 1");
                Ok(DataFrameShape::Bare(df))
            } else {
                Ok(DataFrameShape::PerVariantList(partitions))
            }
        }
    }
}

/// Allocate a STRSXP of length `n` filled with `value`.
///
/// # Safety
///
/// Must be called from the R main thread.
unsafe fn make_strsxp_repeat(value: &str, n: usize) -> SEXP {
    unsafe {
        let n_r: isize = n.try_into().expect("nrow exceeds isize::MAX");
        // Protect the freshly-allocated STRSXP across `SEXP::charsxp`
        // (Rf_mkCharLenCE) — which can trigger GC under gctorture. The
        // OwnedProtect guard drops (UNPROTECT(1)) on return.
        let guard = crate::OwnedProtect::new(SEXP::alloc(SEXPTYPE::STRSXP, n_r));
        let sexp = guard.get();
        // Allocate the CHARSXP exactly once, then reuse it for every slot.
        // Preserve the R_BlankString short-circuit for the empty string.
        let charsxp = if value.is_empty() {
            SEXP::blank_string()
        } else {
            SEXP::charsxp(value)
        };
        for i in 0..n_r {
            sexp.set_string_elt(i, charsxp);
        }
        sexp
    }
}
// endregion
// endregion

// region: TaggedVariantRow (collated emit for vec_to_dataframe_split + result_to_dataframe)

/// Wrapper that prepends a `(tag_column, tag_value)` field to a struct row
/// when serialized through the columnar pipeline.
///
/// For externally-tagged enums (the default) the inner row goes through
/// [`VariantStrippingSerializer`] to flatten `Variant { fields… }` into a
/// plain struct so schema discovery sees the fields directly. For
/// internally-tagged enums the implicit tag field is captured but the
/// caller is expected to suppress it by routing through
/// `SuppressFieldSerializer` (we use the existing tag-drop on the
/// resulting data.frame instead).
struct TaggedVariantRow<'a, T> {
    tag_column: &'a str,
    tag_value: &'a str,
    inner: &'a T,
    /// `Some(field_name)` for internally-tagged enums — that field will be
    /// suppressed in the emitted struct so it doesn't collide with the
    /// explicit `tag_column`.
    tag_field: Option<String>,
}

impl<T: Serialize> Serialize for TaggedVariantRow<'_, T> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // We don't know the inner struct's field count up front, but
        // serialize_struct accepts a hint — use a reasonable upper bound
        // (the value is advisory).
        use ser::SerializeMap as _;
        // Use a map under the hood: the columnar discoverer treats both
        // serialize_struct and serialize_map as struct-like input. Maps
        // give us the freedom to forward arbitrary inner field counts.
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry(self.tag_column, self.tag_value)?;
        // Re-emit the inner row's fields. We funnel them through a
        // serializer that strips the variant wrapper (externally-tagged
        // enums) and skips the implicit tag field (internally-tagged).
        let routed = RoutedInner {
            inner: self.inner,
            suppress_field: self.tag_field.as_deref(),
        };
        // RoutedInner emits its serialize_struct via map.serialize_entry,
        // so we don't need to call serialize_entry per field here.
        routed.serialize_into_map(&mut map)?;
        map.end()
    }
}

/// Internal carrier that emits the inner row's fields one-by-one into a
/// parent serde map.
struct RoutedInner<'a, T> {
    inner: &'a T,
    suppress_field: Option<&'a str>,
}

impl<T: Serialize> RoutedInner<'_, T> {
    fn serialize_into_map<M: ser::SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
        // Drive the inner Serialize through a forwarder that captures each
        // (key, value) and forwards it to the parent map.
        let forwarder = MapForwarder {
            map,
            suppress: self.suppress_field,
            prefix: None,
        };
        self.inner
            .serialize(VariantStrippingMapForwarder { forwarder })
    }
}

/// Forwarder serializer that re-emits a struct's fields as map entries on
/// a *parent* map serializer, skipping `suppress` if set.
///
/// When `prefix` is `Some(p)`, every emitted key is rewritten to `<p>_<key>`.
/// This is how [`FlattenEnumFieldsRow`] namespaces a nested enum field's
/// payload columns under the parent field name (`action_file`, `action_path`,
/// …) without touching the inner enum's serde representation.
struct MapForwarder<'m, M: ser::SerializeMap> {
    map: &'m mut M,
    suppress: Option<&'m str>,
    prefix: Option<&'m str>,
}

impl<M: ser::SerializeMap> MapForwarder<'_, M> {
    fn emit<V: ?Sized + Serialize>(&mut self, key: &str, value: &V) -> Result<(), M::Error> {
        if Some(key) == self.suppress {
            return Ok(());
        }
        match self.prefix {
            Some(p) => self.map.serialize_entry(&format!("{p}_{key}"), value),
            None => self.map.serialize_entry(key, value),
        }
    }
}

/// Serializer that flattens the inner row (handling enum-variant wrapping)
/// and forwards each (key, value) to a [`MapForwarder`].
struct VariantStrippingMapForwarder<'m, M: ser::SerializeMap> {
    forwarder: MapForwarder<'m, M>,
}

impl<'m, M: ser::SerializeMap> ser::Serializer for VariantStrippingMapForwarder<'m, M> {
    type Ok = ();
    type Error = M::Error;
    type SerializeSeq = ser::Impossible<(), M::Error>;
    type SerializeTuple = ser::Impossible<(), M::Error>;
    type SerializeTupleStruct = ser::Impossible<(), M::Error>;
    type SerializeTupleVariant = ForwardingMapEmitter<'m, M>;
    type SerializeMap = ForwardingMapEmitter<'m, M>;
    type SerializeStruct = ForwardingMapEmitter<'m, M>;
    type SerializeStructVariant = ForwardingMapEmitter<'m, M>;

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(ForwardingMapEmitter {
            forwarder: self.forwarder,
            tuple_idx: 0,
        })
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(ForwardingMapEmitter {
            forwarder: self.forwarder,
            tuple_idx: 0,
        })
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(ForwardingMapEmitter {
            forwarder: self.forwarder,
            tuple_idx: 0,
        })
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(ForwardingMapEmitter {
            forwarder: self.forwarder,
            tuple_idx: 0,
        })
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), Self::Error> {
        v.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        v: &T,
    ) -> Result<(), Self::Error> {
        v.serialize(self)
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<(), Self::Error> {
        // Unit variants emit no fields — handled by the caller's unit-variant
        // branch when building per-variant DFs. Collated-shape unit variants
        // arrive here and contribute zero columns aside from the tag column.
        Ok(())
    }

    // Primitives + simple values cannot appear at the top of a row (the row
    // is required to be a struct/map). Fail informatively.
    fn serialize_bool(self, _: bool) -> Result<(), Self::Error> {
        Err(serde::ser::Error::custom(
            "TaggedVariantRow: expected struct or map at top level",
        ))
    }
    fn serialize_i8(self, _: i8) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_i16(self, _: i16) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_i32(self, _: i32) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_i64(self, _: i64) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_u8(self, _: u8) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_u16(self, _: u16) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_u32(self, _: u32) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_u64(self, _: u64) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_f32(self, _: f32) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_f64(self, _: f64) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_char(self, _: char) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_str(self, _: &str) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_none(self) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<(), Self::Error> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Self::Error> {
        Ok(())
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(serde::ser::Error::custom(
            "TaggedVariantRow: expected struct or map at top level",
        ))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(serde::ser::Error::custom(
            "TaggedVariantRow: expected struct or map at top level",
        ))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(serde::ser::Error::custom(
            "TaggedVariantRow: expected struct or map at top level",
        ))
    }
}

/// Sink that re-emits struct fields / map entries as map entries on the
/// parent serializer. Tuple-variant indices are mapped to `_0`, `_1`, …
struct ForwardingMapEmitter<'m, M: ser::SerializeMap> {
    forwarder: MapForwarder<'m, M>,
    tuple_idx: u32,
}

impl<M: ser::SerializeMap> ser::SerializeStruct for ForwardingMapEmitter<'_, M> {
    type Ok = ();
    type Error = M::Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.forwarder.emit(key, value)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<M: ser::SerializeMap> ser::SerializeStructVariant for ForwardingMapEmitter<'_, M> {
    type Ok = ();
    type Error = M::Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.forwarder.emit(key, value)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<M: ser::SerializeMap> ser::SerializeMap for ForwardingMapEmitter<'_, M> {
    type Ok = ();
    type Error = M::Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, _key: &T) -> Result<(), Self::Error> {
        // We don't have a way to extract the key string here without a side
        // channel; the TaggedVariantRow's collated path only feeds derive-
        // generated impls (serialize_struct or serialize_struct_variant),
        // so this map-path is dead for them. Hand-written Serialize impls
        // emitting maps with non-string keys aren't supported.
        Err(serde::ser::Error::custom(
            "TaggedVariantRow: hand-written serialize_map not supported in collated shape",
        ))
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), Self::Error> {
        Err(serde::ser::Error::custom(
            "TaggedVariantRow: hand-written serialize_map not supported in collated shape",
        ))
    }

    fn serialize_entry<K: ?Sized + Serialize, V: ?Sized + Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error> {
        // Extract the key string via a sibling extractor.
        let mut ve = ValueExtractor::default();
        let _ = key.serialize(&mut ve);
        let key_str = match ve.value {
            ExtractedValue::Str(s) => s,
            _ => return Ok(()),
        };
        self.forwarder.emit(&key_str, value)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<M: ser::SerializeMap> ser::SerializeTupleVariant for ForwardingMapEmitter<'_, M> {
    type Ok = ();
    type Error = M::Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = format!("_{}", self.tuple_idx);
        self.tuple_idx += 1;
        self.forwarder.emit(&key, value)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// endregion

// region: vec_to_dataframe_flatten_enums (#1056)

/// Like [`vec_to_dataframe`], but each field named in `flatten_enum_fields`
/// whose value is an enum is flattened *into columns* instead of landing as a
/// per-row list-column.
///
/// The default [`vec_to_dataframe`] recursively flattens nested *structs* into
/// `parent_child` columns, but a nested *enum* field cannot be flattened the
/// same way (different rows carry different variant shapes), so it falls back
/// to a `ColumnType::Generic` list-column. This helper is the nested-field
/// analogue of [`SplitShape::Collated`](crate::serde::SplitShape): for each
/// targeted field it emits
///
/// - a leading `<field>_variant` **tag** column holding the variant name, and
/// - the union of the variants' payload fields, each prefixed `<field>_`
///   (`<field>_<subfield>`), NA-filled on rows whose variant lacks that field.
///
/// This matches the `#[derive(DataFrameRow)]` nested-enum naming convention
/// (`<outer_field>_variant` tag + prefixed payload). Fields *not* in the set
/// keep their default behaviour (nested structs flatten to `parent_child`;
/// nested enums stay list-columns).
///
/// # Serde-inheritance only
///
/// This is a pure `Serialize` adapter over the existing row type — it requires
/// **no** flattened "view" struct and does **not** change the enum's serde
/// representation (an externally-tagged enum stays externally tagged on the
/// wire). You opt in per-call by naming the fields to flatten.
///
/// # Variant shapes
///
/// - **Struct variant** `V { a, b }` → `<field>_a`, `<field>_b`.
/// - **Newtype variant wrapping a struct** `V(Inner { a, b })` → the inner
///   struct's fields flatten under the prefix (`<field>_a`, `<field>_b`).
/// - **Tuple variant** `V(x, y)` → `<field>__0`, `<field>__1` (the `_N`
///   tuple-index convention, prefixed).
/// - **Unit variant** `V` → only the `<field>_variant` tag (no payload).
/// - **`Option<Enum>` = `None`** → the row contributes nothing for this field;
///   Union discovery + NA-fill leave the tag and all payload columns NA.
///
/// # Casing
///
/// Variant names are whatever serde emits (PascalCase by default; override
/// with `#[serde(rename_all = "snake_case")]` on the enum).
///
/// # Limitations
///
/// - **Externally-tagged / untagged data enums** are the intended target. If a
///   targeted field holds an *internally-tagged* enum (`#[serde(tag = "kind")]`),
///   its implicit tag arrives as an ordinary field and becomes a normal
///   `<field>_kind` column (no dedicated `<field>_variant` is synthesised);
///   this is accepted, not special-cased.
/// - **Reject-non-enum is best-effort.** A scalar or sequence value errors, but
///   a *struct*-shaped value cannot be told apart from an internally-tagged enum
///   under serde, so a plain struct targeted here is flattened in place (its
///   fields land as `<field>_<sub>`, no `<field>_variant`) rather than rejected.
///   Name only true enum fields; a struct field is already flattened to
///   `<field>_<sub>` by plain [`vec_to_dataframe`] anyway.
/// - **Compound-vs-compound payload divergence.** Payload subfields are emitted
///   *flat* into the parent map, so each leaf key goes through scalar Union
///   resolution independently — variants whose payloads differ in shape merge
///   per-leaf-key. (Plain [`vec_to_dataframe`]'s nested-struct `Compound`
///   candidates recursively union the same way as of
///   [#1370](https://github.com/A2-ai/miniextendr/issues/1370); this bullet
///   just documents that this function's flat-leaf-key path always did.)
///
/// # Errors
///
/// - A targeted field's value serializes as a scalar or sequence (and is not
///   `Option::None`) — there is no struct/enum shape to flatten. (A struct-shaped
///   value is flattened, not rejected — see Limitations.)
/// - The row does not serialize as a struct or map.
/// - Underlying column-buffer assembly fails.
///
/// # Example
///
/// ```ignore
/// #[derive(Serialize)]
/// enum Action { Add { file: f64 }, Init { path: String } }
/// #[derive(Serialize)]
/// struct AuditRow { id: i32, action: Action }
///
/// let df = vec_to_dataframe_flatten_enums(&rows, &["action"])?;
/// // Columns: id, action_variant, action_file, action_path
/// ```
///
/// # Reading back
///
/// The layout round-trips to `Vec<T>` via
/// [`dataframe_to_vec`](crate::serde::dataframe_to_vec) (default tag names) —
/// or [`dataframe_to_vec_with_enum_tags`](crate::serde::dataframe_to_vec_with_enum_tags)
/// when custom tag names were used (#1060).
pub fn vec_to_dataframe_flatten_enums<T: Serialize>(
    rows: &[T],
    flatten_enum_fields: &[&str],
) -> Result<BuiltDataFrame, RSerdeError> {
    vec_to_dataframe_flatten_enums_with_tags(rows, flatten_enum_fields, &[])
}

/// Like [`vec_to_dataframe_flatten_enums`], but lets the caller name each
/// field's variant-tag column (#1061).
///
/// `tags` maps a flattened field to the tag-column name to emit for it
/// (`field → column`); fields in `flatten_enum_fields` but absent from `tags`
/// keep the default `<field>_variant`. This mirrors the top-level
/// [`SplitShape::Collated { column }`](SplitShape), which already lets the
/// caller name its tag column.
///
/// The payload columns are unaffected — they stay `<field>_<subfield>`
/// regardless of the tag name. Read back with
/// [`dataframe_to_vec_with_enum_tags`](crate::serde::dataframe_to_vec_with_enum_tags),
/// passing the same mapping.
///
/// # Example
///
/// ```ignore
/// // `state` gets a `state_tag` column instead of `state_variant`.
/// let df = vec_to_dataframe_flatten_enums_with_tags(&rows, &["state"], &[("state", "state_tag")])?;
/// // Columns: id, state_tag, state_<payload…>
/// ```
pub fn vec_to_dataframe_flatten_enums_with_tags<T: Serialize>(
    rows: &[T],
    flatten_enum_fields: &[&str],
    tags: &[(&str, &str)],
) -> Result<BuiltDataFrame, RSerdeError> {
    let wrapped: Vec<FlattenEnumFieldsRow<'_, T>> = rows
        .iter()
        .map(|row| FlattenEnumFieldsRow {
            inner: row,
            fields: flatten_enum_fields,
            tags,
        })
        .collect();
    vec_to_dataframe(&wrapped)
}

/// Adapter that re-serializes a row as a serde map, flattening each enum field
/// named in `fields` into a `<field>_variant` (or `tags`-overridden) tag plus
/// prefixed payload columns.
struct FlattenEnumFieldsRow<'a, T> {
    inner: &'a T,
    fields: &'a [&'a str],
    tags: &'a [(&'a str, &'a str)],
}

impl<T: Serialize> Serialize for FlattenEnumFieldsRow<'_, T> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use ser::SerializeMap as _;
        // A map gives us freedom over the emitted field count + names; the
        // columnar discoverer treats serialize_map like a struct.
        let mut map = serializer.serialize_map(None)?;
        self.inner.serialize(FieldSelectingForwarder {
            map: &mut map,
            fields: self.fields,
            tags: self.tags,
        })?;
        map.end()
    }
}

/// Resolve the tag-column name for `field`: the `tags` override if present,
/// else the default `<field>_variant`.
fn resolve_tag_column(field: &str, tags: &[(&str, &str)]) -> String {
    tags.iter()
        .find(|(f, _)| *f == field)
        .map(|(_, t)| (*t).to_owned())
        .unwrap_or_else(|| format!("{field}_variant"))
}

/// Drives the inner row's `serialize_struct`/`serialize_map` and re-emits each
/// `(key, value)` onto a parent map, routing keys in `fields` through the
/// enum-flattener and passing the rest through unchanged.
struct FieldSelectingForwarder<'m, M: ser::SerializeMap> {
    map: &'m mut M,
    fields: &'m [&'m str],
    tags: &'m [(&'m str, &'m str)],
}

impl<'m, M: ser::SerializeMap> ser::Serializer for FieldSelectingForwarder<'m, M> {
    type Ok = ();
    type Error = M::Error;
    type SerializeSeq = ser::Impossible<(), M::Error>;
    type SerializeTuple = ser::Impossible<(), M::Error>;
    type SerializeTupleStruct = ser::Impossible<(), M::Error>;
    type SerializeTupleVariant = ser::Impossible<(), M::Error>;
    type SerializeMap = SelectingMapEmitter<'m, M>;
    type SerializeStruct = SelectingMapEmitter<'m, M>;
    type SerializeStructVariant = ser::Impossible<(), M::Error>;

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SelectingMapEmitter {
            map: self.map,
            fields: self.fields,
            tags: self.tags,
            pending_key: None,
        })
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SelectingMapEmitter {
            map: self.map,
            fields: self.fields,
            tags: self.tags,
            pending_key: None,
        })
    }

    fn serialize_some<V: ?Sized + Serialize>(self, v: &V) -> Result<(), Self::Error> {
        v.serialize(self)
    }
    fn serialize_newtype_struct<V: ?Sized + Serialize>(
        self,
        _: &'static str,
        v: &V,
    ) -> Result<(), Self::Error> {
        v.serialize(self)
    }

    // A flatten-target row must be a struct/map at the top level — everything
    // else is an error (mirrors VariantStrippingMapForwarder's posture).
    fn serialize_none(self) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_bool(self, _: bool) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_i8(self, _: i8) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_i16(self, _: i16) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_i32(self, _: i32) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_i64(self, _: i64) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_u8(self, _: u8) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_u16(self, _: u16) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_u32(self, _: u32) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_u64(self, _: u64) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_f32(self, _: f32) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_f64(self, _: f64) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_char(self, _: char) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_str(self, _: &str) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_unit(self) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_newtype_variant<V: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &V,
    ) -> Result<(), Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(top_level_struct_error())
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(top_level_struct_error())
    }
}

fn top_level_struct_error<E: ser::Error>() -> E {
    ser::Error::custom("vec_to_dataframe_flatten_enums: expected struct or map at top level")
}

/// Sink that re-emits the inner row's `(key, value)` pairs onto the parent map,
/// routing keys named in `fields` through [`flatten_enum_field`].
struct SelectingMapEmitter<'m, M: ser::SerializeMap> {
    map: &'m mut M,
    fields: &'m [&'m str],
    tags: &'m [(&'m str, &'m str)],
    /// For the map path: the most recent key awaiting its value.
    pending_key: Option<String>,
}

impl<M: ser::SerializeMap> SelectingMapEmitter<'_, M> {
    fn route<V: ?Sized + Serialize>(&mut self, key: &str, value: &V) -> Result<(), M::Error> {
        if self.fields.contains(&key) {
            let tag_column = resolve_tag_column(key, self.tags);
            flatten_enum_field(self.map, key, value, &tag_column)
        } else {
            self.map.serialize_entry(key, value)
        }
    }
}

impl<M: ser::SerializeMap> ser::SerializeStruct for SelectingMapEmitter<'_, M> {
    type Ok = ();
    type Error = M::Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.route(key, value)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<M: ser::SerializeMap> ser::SerializeMap for SelectingMapEmitter<'_, M> {
    type Ok = ();
    type Error = M::Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        let mut ve = ValueExtractor::default();
        let _ = key.serialize(&mut ve);
        self.pending_key = match ve.value {
            ExtractedValue::Str(s) => Some(s),
            _ => None,
        };
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        match self.pending_key.take() {
            Some(key) => self.route(&key, value),
            None => Err(ser::Error::custom(
                "vec_to_dataframe_flatten_enums: map key was not a string",
            )),
        }
    }

    fn serialize_entry<K: ?Sized + Serialize, V: ?Sized + Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error> {
        let mut ve = ValueExtractor::default();
        let _ = key.serialize(&mut ve);
        match ve.value {
            ExtractedValue::Str(s) => self.route(&s, value),
            _ => Err(ser::Error::custom(
                "vec_to_dataframe_flatten_enums: map key was not a string",
            )),
        }
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Route a single targeted field `value` (expected to be an enum or
/// `Option<Enum>`) into the parent `map`: emit the `<field>_variant` tag, then
/// the variant payload prefixed with `<field>_`.
///
/// `None` (a `None` `Option<Enum>`) emits nothing — Union discovery + NA-fill
/// leaves the tag + payload columns NA for that row.
fn flatten_enum_field<M: ser::SerializeMap, V: ?Sized + Serialize>(
    map: &mut M,
    field: &str,
    value: &V,
    tag_column: &str,
) -> Result<(), M::Error> {
    // Classify the value in a single pass. An externally-tagged / untagged enum
    // yields a variant `name` (and no `tag_field`); an internally-tagged enum —
    // or, indistinguishably under serde, a plain struct with a leading string
    // field — yields `name` *and* `tag_field`; `Option::None` yields neither but
    // sets `is_none`; a scalar / sequence yields nothing at all.
    let mut ext = VariantNameExtractor::default();
    let _ = value.serialize(&mut ext);

    let name = match ext.name {
        Some(name) => name,
        None => {
            // No variant: `Option::None` emits nothing (Union discovery +
            // NA-fill leaves the row's tag + payload columns NA); any other
            // non-struct value (scalar / sequence) is a usage error.
            return if ext.is_none {
                Ok(())
            } else {
                Err(ser::Error::custom(format!(
                    "vec_to_dataframe_flatten_enums: field '{field}' is not an enum \
                     (only enum / Option<enum> fields can be flattened)"
                )))
            };
        }
    };

    // Synthesize the tag column (`<field>_variant` by default, or a caller
    // override) only for externally-tagged / untagged enums. For an
    // internally-tagged enum (`tag_field` is `Some`) the tag already rides
    // along inside the payload and lands as `<field>_<tagfield>`, so a
    // synthetic tag column would just duplicate it.
    if ext.tag_field.is_none() {
        map.serialize_entry(tag_column, &name)?;
    }

    if ext.is_unit {
        // Unit variant: tag only, no payload.
        return Ok(());
    }

    // Payload: drive the value through the variant-stripping forwarder with a
    // key-prefixing MapForwarder so each subfield lands as `<field>_<sub>`.
    let forwarder = MapForwarder {
        map,
        suppress: None,
        prefix: Some(field),
    };
    value.serialize(VariantStrippingMapForwarder { forwarder })
}

// endregion

// region: map_to_dataframe (closes #700)

/// Serialize a [`BTreeMap`](std::collections::BTreeMap) to an R data.frame
/// with the keys as one column and the value struct's fields as the rest.
///
/// Output column order: `<key_column>` first, then `V`'s flattened serde
/// fields in declaration order. Nested struct flattening, `#[serde(flatten)]`,
/// and `#[serde(skip_serializing_if)]` all work the same way as in
/// [`vec_to_dataframe`].
///
/// `BTreeMap`'s ordered iteration gives a deterministic row order. For
/// [`HashMap`], see [`hashmap_to_dataframe`].
///
/// # Errors
///
/// - `V` does not serialize as a struct or map.
/// - Underlying column-buffer assembly fails.
///
/// # Example
///
/// ```ignore
/// use std::collections::BTreeMap;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Summary {
///     cmax: f64,
///     tmax: f64,
/// }
///
/// let summary: BTreeMap<i32, Summary> = /* … */;
/// let df = map_to_dataframe(&summary, "subject")?;
/// // Columns: subject, cmax, tmax
/// ```
pub fn map_to_dataframe<K, V>(
    map: &std::collections::BTreeMap<K, V>,
    key_column: &str,
) -> Result<BuiltDataFrame, RSerdeError>
where
    K: Serialize,
    V: Serialize,
{
    let rows: Vec<MapEntry<'_, K, V>> = map
        .iter()
        .map(|(k, v)| MapEntry {
            key_column,
            key: k,
            value: v,
        })
        .collect();
    vec_to_dataframe(&rows)
}

/// Serialize a [`HashMap`] to an R data.frame.
///
/// Keys are sorted by their `Ord` impl to produce a deterministic row order.
/// For maps with non-`Ord` keys (or callers happy with insertion-order
/// non-determinism), wrap into a `BTreeMap` first or convert manually.
///
/// Output column order matches [`map_to_dataframe`]: `<key_column>` first,
/// then `V`'s flattened serde fields in declaration order.
///
/// # Errors
///
/// Same as [`map_to_dataframe`].
pub fn hashmap_to_dataframe<K, V>(
    map: &std::collections::HashMap<K, V>,
    key_column: &str,
) -> Result<BuiltDataFrame, RSerdeError>
where
    K: Serialize + Ord,
    V: Serialize,
{
    // Collect references and sort by key for deterministic output order.
    let mut entries: Vec<(&K, &V)> = map.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let rows: Vec<MapEntry<'_, K, V>> = entries
        .into_iter()
        .map(|(k, v)| MapEntry {
            key_column,
            key: k,
            value: v,
        })
        .collect();
    vec_to_dataframe(&rows)
}

/// Wrapper that serializes a single `(K, V)` map entry as a struct/map
/// whose first field is the user's key column and whose remaining fields
/// are flattened from `V`.
struct MapEntry<'a, K, V> {
    key_column: &'a str,
    key: &'a K,
    value: &'a V,
}

impl<K: Serialize, V: Serialize> Serialize for MapEntry<'_, K, V> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use ser::SerializeMap as _;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry(self.key_column, self.key)?;
        // Forward V's fields directly into the map. V must serialize as a
        // struct or map; primitives at the top level fail explicitly.
        let routed = RoutedInner {
            inner: self.value,
            suppress_field: None,
        };
        routed.serialize_into_map(&mut map)?;
        map.end()
    }
}

// endregion

// region: result_to_dataframe (closes #697)

/// Shape selector for [`result_to_dataframe`].
///
/// Configures whether the helper returns a bare data.frame, a split
/// `list(results=, error=)`, or a collated single-data.frame with an
/// `is_error` column and the union of Ok and Err fields.
pub enum ResultShape<S> {
    /// All-Ok input → bare data.frame; otherwise → `list(results=, error=)`.
    ///
    /// `empty_ok_sentinel` is unused when at least one `Ok` is present.
    Auto {
        /// Returned in the `results` slot when *every* row was `Err`.
        empty_ok_sentinel: S,
    },
    /// Single collated data.frame: every row, with an `is_error` LGLSXP
    /// column plus the union of Ok and Err fields. Other-variant fields
    /// land as NA per row.
    Collated,
    /// Always `list(results=, error=)`, even when all rows are `Ok` (in
    /// which case `error` is a zero-row data.frame) or all are `Err` (in
    /// which case `results` is `empty_ok_sentinel`).
    Split {
        /// Returned in the `results` slot when *every* row was `Err`.
        empty_ok_sentinel: S,
    },
}

/// Partition a slice of `Result<T, E>` into Ok and Err data.frames.
///
/// The shape of the return is controlled by [`ResultShape`]. The output is
/// always a [`DataFrameShape`]; convert via [`crate::IntoR`] to get the
/// equivalent R-side SEXP at the `#[miniextendr]` function boundary.
///
/// # Sentinel
///
/// For [`Auto`](ResultShape::Auto) and [`Split`](ResultShape::Split), the
/// `empty_ok_sentinel` field is the value placed in the `results` slot
/// when every row is `Err`. Any [`crate::IntoR`] type works — `NULL`,
/// `FALSE`, `NA`, an empty zero-row data.frame, etc. The sentinel is only
/// allocated when needed.
///
/// # GC discipline
///
/// The returned [`DataFrameShape`] is rooted by construction (#1265):
/// every partition rides in an owned, GC-rooted
/// [`BuiltDataFrame`] handle and the
/// empty-`Ok` sentinel carries its own [`RootedSentinel`] root, so the
/// shape is safe to hold across R allocations. The [`crate::IntoR`]
/// conversion consumes the handles.
///
/// # Errors
///
/// - `T` or `E` does not serialize as a struct or map.
/// - Underlying column-buffer assembly fails.
///
/// # Example
///
/// ```ignore
/// # use miniextendr_api::serde::{result_to_dataframe, ResultShape};
/// # use serde::Serialize;
/// # #[derive(Serialize)] struct Obs { id: i32, value: f64 }
/// # #[derive(Serialize)] struct Err { id: i32, reason: String }
/// let rows: Vec<Result<Obs, Err>> = /* … */ vec![];
/// // Default dispatch: bare on all-Ok, list otherwise.
/// let shape = result_to_dataframe(&rows, ResultShape::Auto { empty_ok_sentinel: () })?;
/// ```
pub fn result_to_dataframe<T, E, S>(
    rows: &[Result<T, E>],
    shape: ResultShape<S>,
) -> Result<DataFrameShape, RSerdeError>
where
    T: Serialize,
    E: Serialize,
    S: crate::IntoR,
{
    match shape {
        ResultShape::Collated => {
            // Union-schema across T and E, with is_error column.
            let wrapped: Vec<CollatedResultRow<'_, T, E>> =
                rows.iter().map(CollatedResultRow::from).collect();
            // `DataFrameShape::Bare` carries the owned, GC-rooted handle
            // (#1265) — the frame stays rooted for as long as the caller
            // holds the shape.
            Ok(DataFrameShape::Bare(vec_to_dataframe(&wrapped)?))
        }
        ResultShape::Auto { empty_ok_sentinel } => {
            let (oks, errs) = partition_results(rows);
            if errs.is_empty() {
                Ok(DataFrameShape::Bare(vec_to_dataframe(&oks)?))
            } else {
                build_split(oks, errs, empty_ok_sentinel)
            }
        }
        ResultShape::Split { empty_ok_sentinel } => {
            let (oks, errs) = partition_results(rows);
            build_split(oks, errs, empty_ok_sentinel)
        }
    }
}

fn partition_results<T, E>(rows: &[Result<T, E>]) -> (Vec<&T>, Vec<&E>) {
    let mut oks: Vec<&T> = Vec::new();
    let mut errs: Vec<&E> = Vec::new();
    for row in rows {
        match row {
            Ok(t) => oks.push(t),
            Err(e) => errs.push(e),
        }
    }
    (oks, errs)
}

fn build_split<T, E, S>(
    oks: Vec<&T>,
    errs: Vec<&E>,
    empty_ok_sentinel: S,
) -> Result<DataFrameShape, RSerdeError>
where
    T: Serialize,
    E: Serialize,
    S: crate::IntoR,
{
    // `error_df` is an owned, GC-rooted handle: it stays rooted across the
    // `oks`/sentinel allocation below (pre-#1128 it was a bare view reaped
    // mid-build under gctorture) and rides in the returned shape (#1265), so
    // the root travels with the caller instead of releasing here.
    let error_df = vec_to_dataframe(&errs)?;
    let results = if oks.is_empty() {
        // `RootedSentinel::new` converts and immediately roots the
        // caller-supplied sentinel — every leg of the shape carries its own
        // root (#1265).
        SplitResults::None(RootedSentinel::new(empty_ok_sentinel))
    } else {
        SplitResults::Some(vec_to_dataframe(&oks)?)
    };
    Ok(DataFrameShape::Split {
        results,
        error: error_df,
    })
}

/// Wrapper that serializes one `Result<T, E>` as a flat struct with the
/// fields of the variant plus a leading `is_error` boolean.
struct CollatedResultRow<'a, T, E> {
    is_error: bool,
    payload: CollatedPayload<'a, T, E>,
}

enum CollatedPayload<'a, T, E> {
    Ok(&'a T),
    Err(&'a E),
}

impl<'a, T, E> From<&'a Result<T, E>> for CollatedResultRow<'a, T, E> {
    fn from(r: &'a Result<T, E>) -> Self {
        match r {
            Ok(t) => CollatedResultRow {
                is_error: false,
                payload: CollatedPayload::Ok(t),
            },
            Err(e) => CollatedResultRow {
                is_error: true,
                payload: CollatedPayload::Err(e),
            },
        }
    }
}

impl<T: Serialize, E: Serialize> Serialize for CollatedResultRow<'_, T, E> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use ser::SerializeMap as _;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("is_error", &self.is_error)?;
        match &self.payload {
            CollatedPayload::Ok(t) => {
                let routed = RoutedInner {
                    inner: *t,
                    suppress_field: None,
                };
                routed.serialize_into_map(&mut map)?;
            }
            CollatedPayload::Err(e) => {
                let routed = RoutedInner {
                    inner: *e,
                    suppress_field: None,
                };
                routed.serialize_into_map(&mut map)?;
            }
        }
        map.end()
    }
}

// endregion

// region: Tests

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    // region: TypeSpec → ColumnType collapse

    /// Each `TypeSpec` variant maps to the expected `ColumnType`. `Optional`
    /// unwraps to its inner column type — the NA-tolerance hint never
    /// changes the underlying R atomic type.
    #[test]
    fn typespec_collapses_to_column_type() {
        assert_eq!(TypeSpec::Logical.into_column_type(), ColumnType::Logical);
        assert_eq!(TypeSpec::Integer.into_column_type(), ColumnType::Integer);
        assert_eq!(TypeSpec::Real.into_column_type(), ColumnType::Real);
        assert_eq!(
            TypeSpec::Character.into_column_type(),
            ColumnType::Character
        );
        assert_eq!(TypeSpec::Generic.into_column_type(), ColumnType::Generic);
        assert_eq!(
            TypeSpec::Optional(Box::new(TypeSpec::Integer)).into_column_type(),
            ColumnType::Integer
        );
        // Nested Optional collapses transitively.
        assert_eq!(
            TypeSpec::Optional(Box::new(TypeSpec::Optional(Box::new(TypeSpec::Real))))
                .into_column_type(),
            ColumnType::Real
        );
    }

    // endregion

    // region: SerdeRowBuilder::with_schema (#693)

    #[derive(Serialize)]
    struct WideRow {
        a: i32,
        b: Option<String>,
    }

    /// `with_schema` populates the schema before any push, allocates the
    /// expected number of column buffers, and seeds `filled` to the matching
    /// length. First push therefore skips discovery — the schema is fixed.
    #[test]
    fn with_schema_allocates_columns_up_front() {
        let b = SerdeRowBuilder::<WideRow>::with_schema(
            [
                ("a", TypeSpec::Integer),
                ("b", TypeSpec::Optional(Box::new(TypeSpec::Character))),
            ],
            Some(4),
        );
        let schema = b.schema.as_ref().expect("schema set by with_schema");
        assert_eq!(schema.fields.len(), 2);
        assert_eq!(schema.fields[0].name, "a");
        assert_eq!(schema.fields[0].col_type, ColumnType::Integer);
        assert_eq!(schema.fields[1].name, "b");
        // Optional(Character) collapses to Character — NA-tolerance is implicit.
        assert_eq!(schema.fields[1].col_type, ColumnType::Character);
        assert_eq!(b.columns.len(), 2);
        assert_eq!(b.filled.len(), 2);
        assert!(!b.grow);
    }

    /// `with_schema` + `Optional(Integer)` + first row's value `None` lands
    /// in the integer buffer as `NA_INTEGER`, not as a logical-NA column
    /// (which is what default discovery would produce when the first row's
    /// `Option<i32>` is `None`).
    ///
    /// Inspects the buffer state directly; doesn't call `finish` (avoids R
    /// allocation in unit tests).
    #[test]
    fn with_schema_optional_first_none_keeps_declared_type() {
        let mut b = SerdeRowBuilder::<WideRow>::with_schema(
            [
                ("a", TypeSpec::Integer),
                ("b", TypeSpec::Optional(Box::new(TypeSpec::Character))),
            ],
            None,
        );
        b.push(WideRow { a: 1, b: None })
            .expect("first push should succeed");
        assert_eq!(b.nrow, 1);
        // Column "a" is integer with value 1.
        match &b.columns[0] {
            ColumnBuffer::Integer(v) => assert_eq!(v, &vec![1]),
            _ => panic!("expected Integer buffer"),
        }
        // Column "b" is character with NA (None) — *not* a logical column.
        match &b.columns[1] {
            ColumnBuffer::Character(v) => assert_eq!(v, &vec![None]),
            _ => panic!("expected Character buffer for Optional(Character)"),
        }
    }

    /// Default mode: first push discovers schema; a later push with a new
    /// field is rejected by the strict filler.
    #[test]
    fn default_mode_rejects_new_field_on_later_push() {
        #[derive(Serialize)]
        struct R1 {
            a: i32,
        }
        #[derive(Serialize)]
        struct R2 {
            a: i32,
            b: i32,
        }
        // Push R1 then R2 — but builder is parameterised over a single T.
        // Use a wrapper enum to allow heterogeneous serialization.
        #[derive(Serialize)]
        #[serde(untagged)]
        enum Either {
            One(R1),
            Two(R2),
        }
        let mut b = SerdeRowBuilder::<Either>::new(None);
        b.push(Either::One(R1 { a: 1 })).expect("first push");
        let err = b
            .push(Either::Two(R2 { a: 2, b: 99 }))
            .expect_err("strict mode should reject field 'b' not in schema");
        match err {
            RSerdeError::Message(m) => {
                assert!(m.contains("row introduced field"), "unexpected error: {m}");
            }
            other => panic!("expected Message variant, got {other:?}"),
        }
    }

    /// `with_schema` mode: an unknown field on push is rejected.
    #[test]
    fn with_schema_rejects_unknown_field() {
        #[derive(Serialize)]
        struct R {
            x: i32,
            extra: i32,
        }
        let mut b = SerdeRowBuilder::<R>::with_schema([("x", TypeSpec::Integer)], None);
        let err = b
            .push(R { x: 1, extra: 9 })
            .expect_err("'extra' not declared");
        match err {
            RSerdeError::Message(m) => assert!(m.contains("extra"), "unexpected error: {m}"),
            other => panic!("expected Message variant, got {other:?}"),
        }
    }

    // endregion

    // region: SerdeRowBuilder::grow_schema (#692)

    /// `grow_schema()` lets a later row introduce a new field. The new
    /// column is back-filled with `self.nrow` NA values so its length
    /// matches the existing rows.
    #[test]
    fn grow_schema_back_fills_na_on_new_field() {
        use std::collections::BTreeMap;
        let mut b = SerdeRowBuilder::<BTreeMap<String, i32>>::new(None).grow_schema();
        let r1: BTreeMap<String, i32> = [("a".to_string(), 1)].into_iter().collect();
        let r2: BTreeMap<String, i32> = [("a".to_string(), 2), ("b".to_string(), 3)]
            .into_iter()
            .collect();
        b.push(r1).expect("first push");
        assert_eq!(b.nrow, 1);
        assert_eq!(b.columns.len(), 1, "only 'a' so far");
        b.push(r2).expect("growth push");
        assert_eq!(b.nrow, 2);
        assert_eq!(b.columns.len(), 2, "'b' added");
        match &b.columns[0] {
            ColumnBuffer::Integer(v) => assert_eq!(v, &vec![1, 2]),
            _ => panic!("expected Integer for 'a'"),
        }
        // Column 'b' was allocated on row 1; row 0 back-filled to NA_INTEGER.
        match &b.columns[1] {
            ColumnBuffer::Integer(v) => assert_eq!(v, &vec![i32::MIN, 3]),
            _ => panic!("expected Integer for 'b'"),
        }
    }

    /// `grow_schema()` composed with `with_schema(...)`: pre-declared
    /// columns stay typed; new columns from later rows are appended.
    #[test]
    fn grow_schema_combined_with_with_schema() {
        use std::collections::BTreeMap;
        let mut b =
            SerdeRowBuilder::<BTreeMap<String, i32>>::with_schema([("a", TypeSpec::Integer)], None)
                .grow_schema();
        let r1: BTreeMap<String, i32> = [("a".to_string(), 10)].into_iter().collect();
        let r2: BTreeMap<String, i32> = [("a".to_string(), 20), ("c".to_string(), 99)]
            .into_iter()
            .collect();
        b.push(r1).expect("push 1");
        b.push(r2).expect("push 2");
        assert_eq!(b.nrow, 2);
        // Original declared column 'a' preserved.
        let schema = b.schema.as_ref().unwrap();
        assert_eq!(schema.fields[0].name, "a");
        assert_eq!(schema.fields[0].col_type, ColumnType::Integer);
        // New column 'c' added after first push that introduced it.
        assert_eq!(schema.fields.len(), 2);
        assert_eq!(schema.fields[1].name, "c");
        assert_eq!(schema.fields[1].col_type, ColumnType::Integer);
        match &b.columns[1] {
            ColumnBuffer::Integer(v) => assert_eq!(v, &vec![i32::MIN, 99]),
            _ => panic!("expected Integer for 'c'"),
        }
    }

    /// `grow_schema()` plus a row that introduces no new fields is a no-op
    /// on schema state — the discovery pass runs but nothing is added.
    #[test]
    fn grow_schema_noop_when_no_new_fields() {
        use std::collections::BTreeMap;
        let mut b = SerdeRowBuilder::<BTreeMap<String, i32>>::new(None).grow_schema();
        let r: BTreeMap<String, i32> = [("k".to_string(), 5)].into_iter().collect();
        b.push(r.clone()).unwrap();
        let pre_len = b.schema.as_ref().unwrap().fields.len();
        b.push(r).unwrap();
        let post_len = b.schema.as_ref().unwrap().fields.len();
        assert_eq!(pre_len, post_len);
        assert_eq!(b.nrow, 2);
    }

    // endregion

    // region: par_iter_to_dataframe (#690)

    /// Build the merged column buffers the *sequential* way, for equivalence
    /// comparison. Mirrors `iter_to_dataframe` minus the final R assembly:
    /// drives a `SerdeRowBuilder` and hands back its raw `ColumnBuffer`s.
    #[cfg(feature = "rayon")]
    fn seq_columns<T: Serialize>(rows: Vec<T>) -> (Vec<String>, Vec<ColumnBuffer>) {
        let mut b = SerdeRowBuilder::<T>::new(None);
        for r in rows {
            b.push(r).expect("sequential push");
        }
        let schema = b.schema.expect("schema discovered");
        let names = schema.fields.iter().map(|f| f.name.clone()).collect();
        (names, b.columns)
    }

    /// Two `ColumnBuffer` slices are element-for-element equal. (Generic
    /// columns never appear on the parallel path, so they're unreachable here.)
    #[cfg(feature = "rayon")]
    fn columns_eq(a: &[ColumnBuffer], b: &[ColumnBuffer]) {
        assert_eq!(a.len(), b.len(), "column count mismatch");
        for (i, (ca, cb)) in a.iter().zip(b).enumerate() {
            match (ca, cb) {
                (ColumnBuffer::Logical(x), ColumnBuffer::Logical(y)) => {
                    assert_eq!(x, y, "logical column {i}")
                }
                (ColumnBuffer::Integer(x), ColumnBuffer::Integer(y)) => {
                    assert_eq!(x, y, "integer column {i}")
                }
                (ColumnBuffer::Real(x), ColumnBuffer::Real(y)) => {
                    // Bitwise: NA_REAL is a NaN, so `==` would reject equal
                    // columns containing NA cells.
                    assert_eq!(x.len(), y.len(), "real column {i} length");
                    assert!(
                        x.iter().zip(y).all(|(a, b)| a.to_bits() == b.to_bits()),
                        "real column {i}"
                    );
                }
                (ColumnBuffer::Character(x), ColumnBuffer::Character(y)) => {
                    assert_eq!(x, y, "character column {i}")
                }
                _ => panic!("column {i}: variant mismatch or unexpected Generic"),
            }
        }
    }

    /// The parallel build is equivalent to the sequential build, column for
    /// column and row for row — including row order. Uses a large input so the
    /// fan-out splits into multiple chunks across the rayon pool.
    #[cfg(feature = "rayon")]
    #[test]
    fn par_build_equivalent_to_sequential() {
        #[derive(Serialize, Clone)]
        struct Row {
            id: i32,
            ratio: f64,
            name: String,
            flag: bool,
            maybe: Option<i32>,
        }

        let rows: Vec<Row> = (0..5_000)
            .map(|i| Row {
                id: i,
                ratio: f64::from(i) * 0.5,
                name: format!("item_{i}"),
                flag: i % 2 == 0,
                // Row 0 must be `Some` so first-row schema discovery types this
                // as an Integer column (an all-None first row would degrade to
                // a Generic/list column, which the parallel path rejects).
                // Later `None`s land as NA_INTEGER in the integer buffer.
                maybe: if i != 0 && i % 3 == 0 {
                    None
                } else {
                    Some(i * 10)
                },
            })
            .collect();

        let (seq_names, seq_cols) = seq_columns(rows.clone());
        let (schema, par_cols, nrow) = par_build_columns(&rows, None)
            .expect("par build ok")
            .expect("non-empty");

        assert_eq!(nrow, 5_000);
        let par_names: Vec<String> = schema.fields.iter().map(|f| f.name.clone()).collect();
        assert_eq!(par_names, seq_names, "column names + order must match");
        columns_eq(&par_cols, &seq_cols);

        // Spot-check row order survived the chunk merge: id column is 0..5000
        // in order.
        match &par_cols[0] {
            ColumnBuffer::Integer(v) => {
                assert_eq!(v.len(), 5_000);
                assert!(
                    v.iter().enumerate().all(|(i, &x)| x == i as i32),
                    "id column not in row order after merge"
                );
            }
            _ => panic!("expected integer id column"),
        }
    }

    /// Row order is preserved even when the chunk boundary falls mid-stream:
    /// the character column reads back in exact iteration order.
    #[cfg(feature = "rayon")]
    #[test]
    fn par_build_preserves_row_order() {
        #[derive(Serialize)]
        struct Row {
            label: String,
        }
        let rows: Vec<Row> = (0..1_000)
            .map(|i| Row {
                label: format!("r{i:04}"),
            })
            .collect();
        let (_schema, cols, nrow) = par_build_columns(&rows, Some(1_000))
            .expect("par build ok")
            .expect("non-empty");
        assert_eq!(nrow, 1_000);
        match &cols[0] {
            ColumnBuffer::Character(v) => {
                for (i, cell) in v.iter().enumerate() {
                    assert_eq!(cell.as_deref(), Some(format!("r{i:04}").as_str()));
                }
            }
            _ => panic!("expected character column"),
        }
    }

    /// Empty input yields `Ok(None)` (the public fn turns this into a 0-row
    /// data.frame without touching R).
    #[cfg(feature = "rayon")]
    #[test]
    fn par_build_empty_is_none() {
        #[derive(Serialize)]
        struct Row {
            x: i32,
        }
        let rows: Vec<Row> = Vec::new();
        let out = par_build_columns(&rows, None).expect("par build ok");
        assert!(out.is_none(), "empty input should produce None");
    }

    /// A schema with a generic (list) column is rejected — the parallel path
    /// can't allocate per-cell SEXPs off the R main thread.
    #[cfg(feature = "rayon")]
    #[test]
    fn par_build_rejects_generic_column() {
        // A nested Vec<i32> field has no atomic-column mapping → Generic column.
        #[derive(Serialize)]
        struct Row {
            id: i32,
            tags: Vec<i32>,
        }
        let rows = vec![
            Row {
                id: 1,
                tags: vec![1, 2],
            },
            Row {
                id: 2,
                tags: vec![3],
            },
        ];
        let err = par_build_columns(&rows, None)
            .err()
            .expect("generic column must be rejected");
        match err {
            RSerdeError::Message(m) => {
                assert!(m.contains("generic"), "unexpected error: {m}")
            }
            other => panic!("expected Message variant, got {other:?}"),
        }
    }

    /// A row introducing a field absent from the first row's schema is rejected
    /// (strict homogeneous-schema contract, same as sequential).
    #[cfg(feature = "rayon")]
    #[test]
    fn par_build_rejects_new_field() {
        #[derive(Serialize)]
        struct R1 {
            a: i32,
        }
        #[derive(Serialize)]
        struct R2 {
            a: i32,
            b: i32,
        }
        #[derive(Serialize)]
        #[serde(untagged)]
        enum Either {
            One(R1),
            Two(R2),
        }
        let rows = vec![Either::One(R1 { a: 1 }), Either::Two(R2 { a: 2, b: 9 })];
        let err = par_build_columns(&rows, None)
            .err()
            .expect("new field must be rejected");
        match err {
            RSerdeError::Message(m) => {
                assert!(m.contains("row introduced field"), "unexpected error: {m}")
            }
            other => panic!("expected Message variant, got {other:?}"),
        }
    }

    // endregion

    // region: par_iter_to_dataframe_growing (#936)

    /// Build the merged column buffers the *sequential union* way, for
    /// equivalence comparison. Mirrors `vec_to_dataframe`'s Phases 1–3 minus
    /// the final R assembly. Only valid for Generic-free schemas (no SEXP is
    /// ever pushed, so the ProtectScope stays empty).
    #[cfg(feature = "rayon")]
    fn seq_union_columns<T: Serialize>(rows: &[T]) -> (Vec<String>, Vec<ColumnBuffer>) {
        let mut acc = SchemaAccumulator::new(SchemaMode::Union);
        for row in rows {
            let _ = acc.feed(row);
        }
        let schema = acc.finalize().expect("union schema");
        let ncol = schema.fields.len();
        let mut columns: Vec<ColumnBuffer> = schema
            .fields
            .iter()
            .map(|f| ColumnBuffer::new(f.col_type, rows.len()))
            .collect();
        // SAFETY (test): no Generic column → the scope is never written to.
        let scope = unsafe { crate::ProtectScope::new() };
        let mut filled = vec![false; ncol];
        for row in rows {
            let filler = ColumnFiller {
                columns: &mut columns,
                field_map: &schema.field_map,
                filled: &mut filled,
                col_start: 0,
                col_count: ncol,
                is_top_level: true,
                pending_key: None,
                scope: &scope,
                strict: false,
            };
            row.serialize(filler).expect("sequential fill");
        }
        let names = schema.fields.iter().map(|f| f.name.clone()).collect();
        (names, columns)
    }

    /// Heterogeneous rows for the growing tests: `score` appears only on
    /// `New` rows, `legacy` only on `Old` rows.
    #[cfg(feature = "rayon")]
    #[derive(Serialize)]
    #[serde(untagged)]
    enum MixedRow {
        Old { id: i32, legacy: String },
        New { id: i32, score: f64 },
    }

    /// The growing build matches `vec_to_dataframe`'s union semantics column
    /// for column and row for row, with fields introduced mid-stream (across
    /// chunk boundaries) NA-padded on the rows that lack them.
    #[cfg(feature = "rayon")]
    #[test]
    fn par_growing_equivalent_to_sequential_union() {
        // 5_000 rows so the fan-out splits into multiple chunks; the variant
        // flips at i % 3 so every chunk sees both shapes, and `legacy` /
        // `score` each first appear early.
        let rows: Vec<MixedRow> = (0..5_000)
            .map(|i| {
                if i % 3 == 0 {
                    MixedRow::Old {
                        id: i,
                        legacy: format!("v{i}"),
                    }
                } else {
                    MixedRow::New {
                        id: i,
                        score: f64::from(i) * 0.25,
                    }
                }
            })
            .collect();

        let (seq_names, seq_cols) = seq_union_columns(&rows);
        let (schema, par_cols, nrow) = par_build_columns_growing(&rows, None)
            .expect("growing build ok")
            .expect("non-empty");

        assert_eq!(nrow, 5_000);
        let par_names: Vec<String> = schema.fields.iter().map(|f| f.name.clone()).collect();
        assert_eq!(par_names, seq_names, "column names + order must match");
        columns_eq(&par_cols, &seq_cols);
    }

    /// A field that first appears in the *last* chunk back-fills NA across
    /// every earlier chunk, and key order follows first appearance.
    #[cfg(feature = "rayon")]
    #[test]
    fn par_growing_backfills_field_from_late_chunk() {
        let rows: Vec<MixedRow> = (0..4_000)
            .map(|i| {
                if i < 3_999 {
                    MixedRow::New {
                        id: i,
                        score: f64::from(i),
                    }
                } else {
                    // Only the final row carries `legacy`.
                    MixedRow::Old {
                        id: i,
                        legacy: "tail".into(),
                    }
                }
            })
            .collect();

        let (schema, cols, nrow) = par_build_columns_growing(&rows, None)
            .expect("growing build ok")
            .expect("non-empty");

        assert_eq!(nrow, 4_000);
        let names: Vec<&str> = schema.fields.iter().map(|f| f.name.as_str()).collect();
        // `legacy` was discovered last → appended after first-seen fields.
        assert_eq!(names, ["id", "score", "legacy"]);
        match &cols[2] {
            ColumnBuffer::Character(v) => {
                assert_eq!(v.len(), 4_000);
                assert!(v[..3_999].iter().all(Option::is_none), "expected NA prefix");
                assert_eq!(v[3_999].as_deref(), Some("tail"));
            }
            _ => panic!("expected character `legacy` column"),
        }
        // `score` is NA on the one Old row.
        match &cols[1] {
            ColumnBuffer::Real(v) => assert!(v[3_999].is_nan(), "score must be NA on Old row"),
            _ => panic!("expected real `score` column"),
        }
    }

    /// Global candidate resolution: a field that is `None` for every row a
    /// chunk sees (which a chunk-local schema would type as Generic) still
    /// resolves to its typed column from another chunk's probe — same as the
    /// sequential union path, and the case a per-chunk-schema merge would get
    /// wrong.
    #[cfg(feature = "rayon")]
    #[test]
    fn par_growing_resolves_none_window_against_late_typed_probe() {
        #[derive(Serialize)]
        struct Row {
            id: i32,
            maybe: Option<i32>,
        }
        // `maybe` is None for the first 3_999 rows (many whole chunks) and
        // typed only by the final row.
        let rows: Vec<Row> = (0..4_000)
            .map(|i| Row {
                id: i,
                maybe: (i == 3_999).then_some(7),
            })
            .collect();

        let (schema, cols, _) = par_build_columns_growing(&rows, None)
            .expect("growing build ok")
            .expect("non-empty");

        assert_eq!(schema.fields[1].col_type, ColumnType::Integer);
        match &cols[1] {
            ColumnBuffer::Integer(v) => {
                assert!(
                    v[..3_999].iter().all(|&x| x == i32::MIN),
                    "expected NA prefix"
                );
                assert_eq!(v[3_999], 7);
            }
            _ => panic!("expected integer `maybe` column"),
        }
    }

    /// A union schema containing a Generic column (here: a field that is None
    /// in every row) is rejected with a pointer back to the sequential path.
    #[cfg(feature = "rayon")]
    #[test]
    fn par_growing_rejects_generic_column() {
        #[derive(Serialize)]
        struct Row {
            id: i32,
            maybe: Option<i32>,
        }
        let rows: Vec<Row> = (0..100).map(|i| Row { id: i, maybe: None }).collect();
        let err = par_build_columns_growing(&rows, None)
            .err()
            .expect("all-None column must be rejected");
        match err {
            RSerdeError::Message(m) => {
                assert!(m.contains("generic"), "unexpected error: {m}")
            }
            other => panic!("expected Message variant, got {other:?}"),
        }
    }

    /// Cross-chunk type clash follows the sequential union behaviour:
    /// first-seen type wins; mismatched later values land as NA.
    #[cfg(feature = "rayon")]
    #[test]
    fn par_growing_type_clash_first_seen_wins() {
        #[derive(Serialize)]
        #[serde(untagged)]
        enum Row {
            I { x: i32 },
            S { x: String },
        }
        // Integers fill the first chunks; strings only appear near the end.
        let rows: Vec<Row> = (0..4_000)
            .map(|i| {
                if i < 3_998 {
                    Row::I { x: i }
                } else {
                    Row::S { x: format!("s{i}") }
                }
            })
            .collect();

        let (seq_names, seq_cols) = seq_union_columns(&rows);
        let (schema, par_cols, _) = par_build_columns_growing(&rows, None)
            .expect("growing build ok")
            .expect("non-empty");

        assert_eq!(schema.fields[0].col_type, ColumnType::Integer);
        let par_names: Vec<String> = schema.fields.iter().map(|f| f.name.clone()).collect();
        assert_eq!(par_names, seq_names);
        columns_eq(&par_cols, &seq_cols);
        match &par_cols[0] {
            ColumnBuffer::Integer(v) => {
                assert_eq!(v[0], 0);
                assert_eq!(v[3_999], i32::MIN, "string cell must coerce to NA");
            }
            _ => panic!("expected integer column"),
        }
    }

    /// Empty input yields `Ok(None)`, same as the homogeneous core.
    #[cfg(feature = "rayon")]
    #[test]
    fn par_growing_empty_is_none() {
        #[derive(Serialize)]
        struct Row {
            x: i32,
        }
        let rows: Vec<Row> = Vec::new();
        let out = par_build_columns_growing(&rows, None).expect("growing build ok");
        assert!(out.is_none(), "empty input should produce None");
    }

    /// Merging per-chunk accumulators is candidate-for-candidate equivalent to
    /// feeding all rows into one accumulator (key order and resolved types).
    #[cfg(feature = "rayon")]
    #[test]
    fn schema_accumulator_merge_matches_single_feed() {
        let rows: Vec<MixedRow> = (0..60)
            .map(|i| {
                if i % 2 == 0 {
                    MixedRow::Old {
                        id: i,
                        legacy: "x".into(),
                    }
                } else {
                    MixedRow::New { id: i, score: 1.0 }
                }
            })
            .collect();

        let mut single = SchemaAccumulator::new(SchemaMode::Union);
        for row in &rows {
            let _ = single.feed(row);
        }
        let single_schema = single.finalize().expect("single schema");

        let mut merged = SchemaAccumulator::new(SchemaMode::Union);
        for chunk in rows.chunks(7) {
            let mut acc = SchemaAccumulator::new(SchemaMode::Union);
            for row in chunk {
                let _ = acc.feed(row);
            }
            merged.merge(acc);
        }
        let merged_schema = merged.finalize().expect("merged schema");

        let single_fields: Vec<(String, ColumnType)> = single_schema
            .fields
            .iter()
            .map(|f| (f.name.clone(), f.col_type))
            .collect();
        let merged_fields: Vec<(String, ColumnType)> = merged_schema
            .fields
            .iter()
            .map(|f| (f.name.clone(), f.col_type))
            .collect();
        assert_eq!(merged_fields, single_fields);
    }

    // endregion
}

// endregion
