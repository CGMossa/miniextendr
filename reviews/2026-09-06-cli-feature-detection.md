# CLI feature detection used an incompatible script format

Attempted: initialize feature detection with the real CLI, edit rules from
both CLI and R, execute the script, list rules as JSON, and add Cargo features
and optional dependencies in a package directory containing spaces.

Failure: the original CLI failed seven assertions. It wrote an inert helper
stub, did not patch configure or run autoconf, and used a rule format the R
parser/editor could not read. `--cargo-spec` and `--optional-dep` were ignored.

Root cause: handwritten CLI code duplicated a superseded design instead of
using the canonical R helpers. The adjacent workflow upgrade command also
only reran configure, leaving the build templates unchanged.

Fix: delegate feature setup, rule edits, and scaffold upgrades to their R
helpers. Read only the canonical marked rules section for native text/JSON
listing. The CI minirextendr job builds the CLI, installs the R helper for
subprocesses, and runs the real interoperability test on R or Rust changes.
Optional Git hooks and editor skills remain documented R-side setup.

The first fixed run exposed two transport problems. Rscript's `-e` processing
unescaped embedded backslashes in a quoted predicate, so rule values now travel
as process arguments and are read with `commandArgs()`. The R command logger
passed raw argument vectors to `system2()`, splitting a Cargo manifest path at
spaces; it now shell-quotes each argument at the execution boundary. A direct
regression also verifies that spaces, quotes, backslashes, and shell syntax
remain literal argument data. One initial test assertion incorrectly expected
a bare `Rscript` invocation in configure; the canonical helper uses quoted
`${R_HOME}/bin/Rscript` and `${srcdir}` paths, and the test now checks that form.

The real CLI/R test verifies feature output, bidirectional rule editing,
repeated setup/add, JSON expressions, optional dependencies, Cargo feature
specifications, legacy-file preservation, and template refresh without changing
user Rust sources or the Cargo manifest. See #1360.

The standalone YAML verification command initially used `$if`, which is invalid
R syntax for a reserved field name. Bracket indexing corrected that check; the
workflow parsed successfully and contained the intended CLI build/install steps.
