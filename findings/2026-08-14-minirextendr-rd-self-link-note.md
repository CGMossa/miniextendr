# `minirextendr` uses a self-package Rd link that R cannot resolve

**Tracked as:** https://github.com/A2-ai/miniextendr/issues/1410

## Summary

`CI=true just minirextendr-check` completes with an R CMD check NOTE:

```text
Unknown package 'minirextendr' in Rd xrefs
```

The source is the roxygen comment in
`minirextendr/R/use-release-workflow.R`:

```text
\code{\link[miniextendr]{assert_utf8_locale_now}}
```

Roxygen preserves that as
`\link[miniextendr]{assert_utf8_locale_now}` in
`minirextendr/man/use_release_workflow.Rd`. The bracket form names an external
package, so checking `minirextendr` itself reports the package as unknown.

## Reproduction

With the R version pinned by `rproject.toml` (currently R 4.6.0), run:

```text
CI=true just minirextendr-check
```

The check log reports the unknown-package cross-reference and points to the
generated help database.

## Suggested fix

Use a same-package topic link, for example
`\link[=assert_utf8_locale_now]{assert_utf8_locale_now}`, regenerate the Rd
file with `just minirextendr-document`, and add a check that the package tarball
passes the Rd cross-reference check without this NOTE.

## Overlap audit

Issue #1188 covered invalid URL warnings caused by Rustdoc-style links in the
main R package. Closed issues #1154 and #1261 covered different historical R
CMD check failures. None covers this self-package Rd cross-reference.
