# Long custom-connection writes were truncated

## What was attempted

The public `memory_connection()` fixture was exercised with a formatted write
larger than the framework's apparent 10,000-byte formatting boundary.

## What went wrong

Writing 12,000 bytes with `writeLines()` and reading them back returned only
9,999 bytes. The operation did not report an error.

## Root cause

The builder overwrote R's `dummy_vfprintf` callback with a Rust trampoline
that formatted into a fixed `[u8; 10_000]`. It forwarded the truncated prefix
to `write` but returned the full length reported by `vsnprintf`, so R could not
detect the short write.

## Fix

The builder now leaves R's callback installed. R safely owns and copies the
platform-specific `va_list`, grows the formatting buffer when required, and
forwards the complete output to the Rust `write` trampoline. The custom Rust
`vfprintf` hook was removed because it had no users and exposed a non-portable
ABI surface.
