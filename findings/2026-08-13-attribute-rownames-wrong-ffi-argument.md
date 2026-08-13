# RArray and List passed parent objects to `Rf_GetRowNames`

## Finding

`RArray::get_rownames` and `List::get_rownames` passed their wrapped array/list
SEXP directly to `Rf_GetRowNames`. R's implementation does not fetch an
object's `dimnames` attribute: it accepts the already extracted dimnames
VECSXP and returns `VECTOR_ELT(dimnames, 0)`.

The sibling column-name getters correctly extracted `dimnames` before calling
`Rf_GetColNames`, making the row/column behavior inconsistent.

## Runtime evidence

Public rpkg probes used dimnamed numeric and VECSXP matrices. Before the fix:

- both column getters returned `c("col-a", "col-b", "col-c")`;
- `RMatrix::get_rownames` returned `None` because the numeric parent was not a
  VECSXP; and
- `List::get_rownames` returned the first list-matrix data element, which then
  failed character conversion with `STRING_ELT() ... not ... 'integer'`.

This follows `background/r-svn/src/main/array.c`: `GetRowNames(SEXP dimnames)`
checks for VECSXP and indexes element zero.

## Resolution

Extract `dimnames` first in both row-name getters, return `None` when the
attribute is absent, then call `Rf_GetRowNames(dimnames)`. Dogfood both axes
through `RMatrix` and `List` against base R's `rownames()` / `colnames()`.
