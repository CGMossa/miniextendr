# Row-name getters failed public dimnamed-matrix probes

## What was attempted

Public rpkg wrappers exercised `RMatrix::get_rownames`,
`RMatrix::get_colnames`, `List::get_rownames`, and `List::get_colnames` using
numeric and VECSXP matrices with explicit row and column names.

## What went wrong

Both column probes returned the expected names. The RMatrix row probe reported
that no row names existed, while the List row probe returned its first integer
data element and failed when that value was read as a character vector.

## Root cause

The row getters passed the parent object to `Rf_GetRowNames`; the column getters
passed the extracted dimnames list. R's `Rf_GetRowNames` accepts a dimnames
VECSXP and returns its first element—it does not retrieve the attribute itself.

## Fix

Make both row getters mirror the column getters: retrieve `dimnames`, return
`None` if absent, and pass the dimnames VECSXP to `Rf_GetRowNames`. Keep public
fixtures for both wrapper types and both axes.
