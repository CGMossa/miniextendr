# GitHub Actions workflows were not linted in CI

## Finding

The repository did not run `actionlint` in CI. Running actionlint 1.7.12 against
all workflows on `origin/main` reported ten diagnostics:

- seven ShellCheck SC2086 findings for unquoted environment-file paths or the
  macOS `$USER` argument; and
- two invalid `needs.r-check-{macos,windows}.result` expressions embedded in
  shell comments even though neither job is in the summary job's `needs` list;
  and
- one ShellCheck SC2162 finding in `.github/workflows/pages.yml`, where `read`
  lacked `-r` and could mangle backslashes in permission-warning paths.

The stale `needs` expressions are easy to miss because GitHub expression
interpolation still applies inside the multiline `run` value even when the
resulting shell line is a comment.

## Resolution

Quote each shell expansion, replace the stale expression-bearing comments with
plain text, make the Pages loop preserve backslashes, and add an always-on
actionlint 1.7.12 job to the CI summary fan-in. This makes the current workflows
clean and prevents the same class of drift from returning unnoticed.
