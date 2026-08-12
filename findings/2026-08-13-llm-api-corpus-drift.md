# The committed Rust API corpus lagged merged public APIs

## Finding

The tracked `rust-llm-docs/generated/` corpus was last refreshed in commit
`7b544174`, but later merged changes altered documented public surfaces without
regenerating it. A clean `just llm-docs` from current `origin/main` changed
seven generated files.

The semantic drift included:

- serde dataframe entry points and per-field enum-tag configuration added by
  #1371;
- the CLI scaffold R-version-floor helpers and constant added by #1372; and
- list-shaped cross-class return handling added by #1376.

The implementation inventories also no longer matched current rustdoc JSON.
Consequently, the LLM-facing API reference omitted supported entry points and
described an older macro surface.

## Resolution

Regenerate the complete tracked corpus from current source in one dedicated
change. Two consecutive generator runs produced identical hashes for the API
digest, API impl inventory, and macros digest. Keeping this refresh separate
from code changes restores a clean baseline so subsequent API PRs can show only
their own generated-documentation delta.
