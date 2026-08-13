# Row-name probe used a non-root-reexported helper

## What was attempted

The new public row/column-name probes converted returned CHARSXPs with
`miniextendr_api::charsxp_to_str`.

## What went wrong

The rpkg fixture failed to compile because that helper is not re-exported at the
crate root.

## Root cause

The fixture assumed a root re-export that the public API does not provide. This
was probe code, before exercising the row-name getters.

## Fix

Use the supported `SEXP::r_char()` plus `CStr::from_ptr` conversion pattern
already used by production list code, then rerun the cached package build.
