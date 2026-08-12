# CHARSXP encoding tags bypassed the locale invariant

## Finding

Inbound string conversion treated a UTF-8 R process locale as proof that every
CHARSXP contained UTF-8 bytes. That is false: an R session can be UTF-8 while a
particular CHARSXP is explicitly tagged Latin-1 or `bytes`.

The hot path read `R_CHAR` and constructed `str` with
`from_utf8_unchecked`. Debug builds panicked on a valid Latin-1 input; release
builds could construct an invalid Rust `str`, violating Rust's safety contract.

## Reproduction

On R 4.6.0 with `l10n_info()[["UTF-8"]] == TRUE`:

```r
x <- rawToChar(as.raw(c(0x66, 0x61, 0xe7, 0x61, 0x64, 0x65)))
Encoding(x) <- "latin1"
miniextendr::conv_string_arg(x)
```

The call failed with `CHARSXP contains non-UTF-8 bytes (locale assertion may
have been skipped)`.

R's own source confirms that `R_CHAR` returns stored bytes unchanged, while
`Rf_translateCharUTF8` consults `IS_UTF8`, `IS_LATIN1`, `IS_BYTES`, and locale
state per CHARSXP.

## Resolution

- Keep the zero-copy `R_CHAR` + `LENGTH` path when `Rf_charIsUTF8` is true.
- Validate those bytes before constructing `str`; never use
  `from_utf8_unchecked` for arbitrary R input.
- Translate other text encodings with `Rf_translateCharUTF8`.
- Return owned `Cow<str>` values when translation is required.
- Reject R's `bytes` encoding as non-text.
- Exercise the behavior through real R fixtures for `String`, `&str`, scalar
  and vector `Cow<str>`, plus invalid byte strings.

## Documentation drift found alongside it

`docs/ENCODING.md` also described four removed `REncodingInfo` fields and
omitted two current fields. The guide is corrected to match the struct and the
per-string encoding behavior.
