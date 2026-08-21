# R6 active-binding codegen emitted roxygen ownership warnings

## What was attempted

Regenerate package documentation after an unrelated fixture export change.

## What went wrong

roxygen2 emitted fourteen `@field` / automatically generated `@backref`
warnings saying it could not find matching R6 methods for seven active
bindings.

## Root cause

The generator put each `@field` block directly above a dynamic
`Class$set("active", ...)` call. roxygen2 8.0.0's `$set()` parser records that
target as an R6 method regardless of the `"active"` member kind, so its method
pass cannot attach the field tags.

## Fix

Move generated active-binding `@field` tags into the class documentation block,
where roxygen2's field resolver consumes them. Leave no roxygen block adjacent
to dynamic active-binding calls and retain an explicit regression assertion for
that output shape.
