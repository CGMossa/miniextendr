# Inline `RCall` arguments were reused by GC

## What was attempted

The fluent builder form shown in the expression API documentation was
dogfooded through R: eight freshly allocated integer scalars were passed inline
to `RCall::arg`, then evaluated as `c(1L, ..., 8L)` under `gctorture(TRUE)`.

## What went wrong

The first invocation returned `c(2L, 2L, 8L, 8L, 8L, 8L, 8L, 8L)` instead of
`1:8`. A longer loop terminated before its success marker.

## Root cause

`RCall` stored callable and argument SEXPs as unrooted raw pointers. Later
scalar allocations triggered GC before the builder constructed its R pairlist,
so earlier argument objects were reclaimed and their memory reused. The
existing GC fixture manually protected all arguments and therefore could not
detect the defect.

## Fix

The builder now owns a VECSXP-backed root pool for its callable and arguments.
Each insertion first takes a temporary stack root so pool initialization or
growth cannot collect the incoming value. The builder is consequently
`!Send + !Sync` and releases its single backing root when dropped.
