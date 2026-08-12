# Custom connections truncate long formatted writes

## Finding

The custom-connection builder replaced the `vfprintf` callback installed by
R with a Rust trampoline backed by a fixed 10,000-byte stack buffer. Formatted
output longer than 9,999 bytes was truncated before it reached
`RConnectionImpl::write`, while the callback returned the original formatted
length. R therefore treated the operation as successful and users received
silently incomplete data.

The public API reproduced the defect without internal helpers:

```r
con <- memory_connection()
x <- strrep("x", 12000L)
writeLines(x, con, sep = "")
seek(con, 0)
nchar(readLines(con, warn = FALSE), type = "bytes")
# before: 9999
# expected: 12000
```

No repository implementation overrides `RConnectionImpl::vfprintf`. R's own
`dummy_vfprintf`, installed by `R_new_custom_connection`, already handles the
platform-specific `va_list`, grows its buffer when necessary, and forwards the
formatted bytes through the connection's `write` callback.

## Resolution

Preserve R's `dummy_vfprintf` callback and remove the unused Rust `vfprintf`
override surface and fixed-buffer trampoline. A public R regression test now
writes and reads back a 12,000-byte line exactly.
