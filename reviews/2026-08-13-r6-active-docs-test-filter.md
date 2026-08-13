# Focused macro test command used multiple filters

## What was attempted

Run the three active-binding unit tests and their snapshot test in one
`cargo ltest` invocation by listing each fully qualified test name.

## What went wrong

Cargo rejected the second test name as an unexpected argument before compiling
or running any tests.

## Root cause

`cargo test` accepts one positional test-name filter, not an arbitrary list of
filters.

## Fix

Use the shared `r6_active_binding` substring as one filter for the unit tests,
then run the snapshot test with its own single filter.
