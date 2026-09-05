# New probe wrappers were installed before NAMESPACE regeneration

## What was attempted

The newly added `#[miniextendr]` row/column-name probes were called through
ordinary exported package lookup after one `just rcmdinstall`.

## What went wrong

R reported that `rarray_matrix_colnames` could not be found.

## Root cause

The install generated and installed the R wrapper, but the existing tracked
`NAMESPACE` did not yet export the new function. As documented in the project
workflow, new exports require `rcmdinstall && force-document && rcmdinstall` to
be callable through exported lookup.

## Fix

Use `miniextendr:::` to run the already installed wrappers for the pre-fix
reproduction. After the runtime fix, run the required document-and-second-
install loop and test through normal exported lookup.
