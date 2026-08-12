# CHARSXP locale assumption

## What was attempted

Audit the encoding module through its Rust implementation, R package fixtures,
testthat coverage, documentation, and the vendored R source.

## What went wrong

A valid Latin-1 CHARSXP in a UTF-8 R session failed the existing `String`
round-trip fixture. The tests covered UTF-8 process locales and ordinary UTF-8
strings, but never combined a UTF-8 session with a non-UTF-8 per-string tag.

## Root cause

The zero-copy refactor replaced encoding-aware translation with `R_CHAR` and
asserted that the process locale made every CHARSXP UTF-8. R stores encoding
metadata per CHARSXP, so the global locale cannot establish that invariant.
`from_utf8_unchecked` then turned the mistaken assumption into a release-build
soundness problem.

## Fix

Dispatch on R's per-string encoding classification: borrow and validate
UTF-8/ASCII, translate Latin-1 text, own translated Cow values, reject `bytes`,
and retain regression tests at the actual R boundary.
