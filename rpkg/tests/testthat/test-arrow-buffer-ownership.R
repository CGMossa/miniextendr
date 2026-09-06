test_that("Arrow buffers require registered R ownership", {
  for (fixture in c(
    "gc_stress_arrow_header_lookalike",
    "gc_stress_arrow_background_drop",
    "gc_stress_arrow_changed_nulls",
    "gc_stress_arrow_sliced_recordbatch"
  )) {
    expect_null(get(fixture, envir = asNamespace("miniextendr"))())
  }
})

test_that("DataFusion global aggregates materialize fresh buffers", {
  for (i in seq_len(20L)) {
    expect_null(miniextendr:::gc_stress_datafusion_global_aggregate())
  }
})
