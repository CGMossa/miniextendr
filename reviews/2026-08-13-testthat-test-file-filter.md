# `test_file()` does not accept a test-name filter

## What was attempted

After the complete factor test file passed, the Arrow adapter file was invoked
with `testthat::test_file(..., filter = "Factor")` to select only its three
factor cases.

## What went wrong

`test_file()` rejected the unused `filter` argument before running the Arrow
file.

## Root cause

The testthat filtering option belongs to directory/package runners, not
`test_file()`. This was test-harness misuse, not a package failure.

## Fix

Run the complete Arrow adapter test file. This is a stronger public dogfood
oracle and avoids custom source/eval filtering that could diverge from testthat
semantics.
