# Jiff timezone gctorture oracle

## What was attempted

Audit `cached_class.rs` against its documentation, Jiff call sites, package
fixtures, testthat coverage, and R's garbage-collection rules.

## What went wrong

The Jiff stress fixture said it verified the `set_posixct_tz` protection path,
but it only read numeric ALTREP elements. It never inspected the dynamic
timezone CHARSXP that was actually at risk. The public constructor using this
path had no R test.

## Root cause

The container received an `OwnedProtect`, but only after a dynamic CHARSXP had
already crossed the container allocation unprotected. The test oracle covered
the surrounding object, not the vulnerable metadata.

## Fix

Protect both allocations in dependency order, assert `tzone` inside the stress
fixture, and add package-level constructor and validation tests.
