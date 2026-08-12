# `RCall` did not root inline arguments

## Finding

`RCall` advertised automatic GC-safe call construction and its public examples
passed freshly allocated SEXPs directly to `.arg()`. The builder stored those
arguments as raw pointers in a Rust `Vec`, however, so later argument
allocations could collect and reuse earlier values before `build()` placed them
in an R pairlist.

The existing gctorture fixture manually wrapped every argument in
`OwnedProtect`, explicitly avoiding the documented ergonomic path. A public R
dogfood fixture that built `c(1L, ..., 8L)` with inline scalar allocations
returned this on the first pre-fix invocation under `gctorture(TRUE)`:

```text
[1] 2 2 8 8 8 8 8 8
```

The expected value was `1:8`. This is observable reuse of collected argument
objects.

## Resolution

`RCall` now owns a non-`Send` `ProtectPool` that roots its callable and every
argument until the builder is dropped. A temporary stack root also covers pool
allocation and growth before each value is inserted. Public R tests exercise
eight inline arguments repeatedly under GC torture, and the older expression
GC fixture now uses inline string arguments instead of protecting them by hand.
