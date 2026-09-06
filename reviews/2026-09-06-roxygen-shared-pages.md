# Explicit roxygen pages and wrapped tag values (#1476)

The new shared-page fixture reproduced both failures: Rust extraction dropped
`@describeIn` continuation text, and emitted wrappers also contained the default
file-stem `@rdname`. Roxygen reported that `@describeIn` cannot be combined with
`@rdname`. Its documentation command nevertheless returned success, so checking
only the process status missed the incomplete page. A warning-condition handler
also missed these message diagnostics; inspect the output and generated Rd.

Respect explicit `@name`, `@rdname`, `@describeIn`, and `@noRd` before automatic
page grouping. Preserve prose continuations. Roxygen's installed tag parsers
show that inheritance, keyword, and concept values can require a single logical
line, so fold their wrapped words with spaces instead of introducing warnings.

Record each wrapper's source line and order standalone functions by file and line.
Keep class/trait priority ordering stable, including the S7 inheritance sort.
This makes shared-page usage and descriptions follow the author's source order.
The generated help regression checks the real installed Rd page and runtime
calls for both ordinary functions and an S3 method.
