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
