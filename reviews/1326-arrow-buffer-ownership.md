# Arrow buffer ownership and background release (#1326)

DataFusion aggregation intermittently returned invalid R columns in Linux CI,
including `test_df_global_agg` on PR #1484. The capacity/offset gate was assumed
to exclude fresh Arrow buffers before speculative R-header recovery.

A custom Arrow buffer holding a copied R header reproduced the error reliably:
its exact-size, unsliced Rust allocation passed the gate and was returned as a
SEXP. Reading before an unregistered allocation was undefined even when the
header check rejected it. R's `memory.c` also shows `R_ReleaseObject` mutating
the preserve list with `R_CHECK_THREAD`, without the mutex claimed by the old
allocation owner; Arrow can destroy that owner on a background thread.

Replaced header probing with ownership registration at R-to-Arrow construction.
Complete registered non-ALTREP vectors retain zero-copy return; fresh buffers,
slices, and changed nulls copy. Independent owners count separately and clones
share one owner. Background drops queue their preserve roots for release on
R's main thread at an unwind boundary. Constructor protection covers ALTREP
materialization and the allocation performed by `R_PreserveObject`.

Regression fixtures cover fake R headers, changed validity, independent owners
and background drops, sliced record batches, and fresh DataFusion aggregate
results. The nightly Miri job now exercises the actual registry implementation
instead of substituting a no-op for unsafe recovery.

The first Miri run passed both registry tests but exposed nightly warnings:
unused inherited package metadata, `arrow-select` enabled without DataFusion,
integration-only dev-dependencies in the unit-test target, a deprecated
`core::f64` module import, and unreachable forwarding on `Infallible`. Removed
the redundant metadata/import, narrowed the dependency feature, acknowledged
the integration-only dependencies, and used exhaustive `match self {}` bodies.

Documenting before the first install left stale wrappers: the roxygen build
deliberately skips wrapper generation and updates its timestamp. Making the
shared library newer triggered wrapper regeneration on the next install, which
then failed its load check because the old namespace still exported
`zero_copy_sexprec_offset`. Documenting the regenerated wrappers and reinstalling
removes that obsolete export and installs the new private regression wrappers.
