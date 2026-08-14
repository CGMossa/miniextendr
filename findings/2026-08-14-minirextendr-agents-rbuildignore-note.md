# `minirextendr` ships `AGENTS.md` as a non-standard top-level file

**Tracked as:** https://github.com/A2-ai/miniextendr/issues/1409

## Summary

`CI=true just minirextendr-check` completes with an R CMD check NOTE because
the built `minirextendr` source package contains `AGENTS.md` at its top level:

```text
Non-standard file/directory found at top level:
  'AGENTS.md'
```

The repository requires every directory containing `CLAUDE.md` to have a
sibling `AGENTS.md`, so removing the file is not an option. The package's
`.Rbuildignore` excludes `CLAUDE.md` but does not exclude `AGENTS.md`:

```text
^CLAUDE\.md$
```

## Reproduction

With the R version pinned by `rproject.toml` (currently R 4.6.0), run:

```text
CI=true just minirextendr-check
```

Inspect the built tarball or `00check.log`; `AGENTS.md` is present and produces
the top-level-file NOTE.

## Suggested fix

Add an anchored `AGENTS.md` entry to `minirextendr/.Rbuildignore`, add a
regression assertion for the built tarball contents, and rerun
`just minirextendr-check`.

## Overlap audit

Issue #1151 covered unifying `.Rbuildignore` and `.gitignore` behavior in
generated scaffolds. Issue #1253 covered enforcing sibling `CLAUDE.md` and
`AGENTS.md` files. Neither covers excluding the required maintainer file from
the `minirextendr` package artifact.
