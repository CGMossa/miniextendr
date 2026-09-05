# Docs index check recipe-name mistake

## What was attempted

After editing README and maintainer documentation, I ran
`just docs-index-check` alongside `just site-check`.

## What went wrong

The command failed immediately because the justfile has no
`docs-index-check` recipe.

## Root cause

I inferred a recipe name from CI's “Check docs index coverage” step instead of
reading the workflow first. That check is an inline shell block in
`.github/workflows/ci.yml`, not a reusable just recipe.

## Fix

I read the live CI step, ran its exact check directly, and kept `just
site-check` as the repository-supported documentation verifier. Future checks
must confirm a recipe exists before invoking it; if it does not, use the
documented command rather than inventing a shortcut.
