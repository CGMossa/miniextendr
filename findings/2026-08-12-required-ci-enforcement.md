# `main` does not require the aggregate CI check

## Finding

The repository defines `CI Success` as the stable aggregate for normal pull
request validation, but GitHub does not require that check before merging to
`main`.

## Evidence

On 2026-08-12:

- `GET /repos/A2-ai/miniextendr/branches/main/protection` returned
  `404 Branch not protected`;
- `GET /repos/A2-ai/miniextendr/rulesets` returned an empty list;
- open and closed issue searches for branch protection, rulesets, and required
  status checks found no existing owner.

## Impact

The merge button can remain available while `CI Success` is pending or
failing. Documentation can establish maintainer policy, but it cannot prevent
an accidental bypass.

## Recommended implementation

Add a repository ruleset targeting `main` that requires pull requests and a
successful `CI Success` check. Require the aggregate rather than every matrix
job so path-conditional skips and future workflow refactors do not require
continuous repository-setting changes.

Tracked by [#1391](https://github.com/A2-ai/miniextendr/issues/1391).
