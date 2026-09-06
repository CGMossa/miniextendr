# Doctor export attribution with whole-package imports (#1368)

The regression suite reproduced a missed stale export when NAMESPACE contained
`import(stats)`: doctor skipped the entire check, including a removed local
function. Whole-package imports were treated as unknowable because enumerating
loaded namespace exports would execute dependency code.

Read explicit exports from installed `Meta/nsInfo.rds`, falling back to the
shipped NAMESPACE when the cache is absent or unreadable. Union these with local
definitions and selective imports, applying `except` only to that dependency's
contribution. Missing packages, unreadable namespace data, and imported export
patterns retain explicit skip diagnostics.

Tests use metadata-only fake dependencies whose R code fails if executed, check
that their namespaces remain unloaded, and cover cache precedence, fallback,
multiple imports, exclusions, unavailable export sets, and real `stats` imports.

The first multi-dependency fixture replaced `.libPaths()` instead of prefixing
it, hiding both the prior fake dependency and testthat's diff renderer. Set
`action = "prefix"` on the temporary library scope; no packages needed reinstalling.

CI passed the minirextendr suite, Linux R CMD check, CRAN-like check, and
cross-package ABI checks. The separate Linux R test job segfaulted while
waldo compared `test_df_global_agg()`'s `max_x` column with `5L` (address
`0x91`, exit 139). This matches the returned-column aggregation crash
already tracked in #1326; the root cause remains under investigation there.
The doctor changes do not execute in that rpkg test.

Failure log: https://github.com/A2-ai/miniextendr/actions/runs/34059914832/job/101558534126
GitHub refused a job rerun while the other workflow jobs were still running.
Keep this occurrence linked from the PR to #1326 instead of hiding it as a
passing check or filing a duplicate issue.
