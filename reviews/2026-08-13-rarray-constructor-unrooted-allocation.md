# RArray construction failed under an allocating gctorture initializer

## What was attempted

Construct a 2-by-3 numeric `RMatrix` through the public `RMatrix::new` API,
force an R allocation inside its initializer closure, and verify dimensions and
values for 100 iterations under gctorture.

## What went wrong

The first pre-fix iteration returned an object whose dimensions were no longer
identical to `c(2L, 3L)`.

## Root cause

The constructor left the data SEXP unprotected while `set_dims` allocated an
integer vector and while the caller-provided initializer could allocate. The R
collector could reclaim the data object before construction finished.

## Fix

Wrap the data SEXP in `OwnedProtect` immediately after allocation and keep the
guard alive through dimension assignment and initializer execution.
