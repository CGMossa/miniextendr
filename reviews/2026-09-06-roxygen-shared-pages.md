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

CI's cross-package ABI tests passed, but its final drift gate failed because
the tracked producer/consumer wrappers still used the old function order.
The rpkg sync gate does not cover those separate package artifacts. Regenerate
with `just cross-install` and `just cross-document`, then commit both wrappers.
The wrapper blocks are unchanged except for order; all 387 cross-package
assertions pass without warnings or failures.

Linux R CMD check exposed duplicate aliases in old fixtures: their hand-written
`@aliases` listed other functions that already have generated help on another
page once explicit `@name` is honored. Remove these redundant lists; each
function retains its own generated alias. The help regression now rejects
aliases claimed by multiple pages. CI's `load_all()` test mode also registers
the source package path without an installed help database, so read generated
source Rd in that mode and retain installed-database checks for installed tests.

Reordering the tracked cross-package wrappers also exposed trailing spaces
in generated blank roxygen lines. All three tag renderers now emit `#'` for
blank lines, preserving paragraph boundaries without adding trailing whitespace.
