# NOTES

From R-internals repo:
> Note that most modern C libraries encode strings as UTF-8.
> This means you should typically use `Rf_mkCharCE()` or
> `Rf_mkCharLenCE()`, and avoid the other string
> creation methods including `Rf_mkString()`

## `NORET`

`Rf_error`
`UNIMPLEMENTED`
`WrongArgCount`
`VECTOR_PTR`
`R_ContinueUnwind`
`Rf_errorcall`

Where `VECTOR_PTR` is deprecated.

## Globals

`_WIN32` means 32-bit.
But `WIN32` maybe just means windows.

## `allowlist.txt`

Before allowlist: 7548
After allowlist:  2843

But the allowlist has a problem with annonymous structs.
They need to be handled separately.

E.g.

```c
enum {SORTED_DECR_NA_1ST = -2,
      SORTED_DECR = -1,
      UNKNOWN_SORTEDNESS = INT_MIN, /*INT_MIN is NA_INTEGER! */
      SORTED_INCR = 1,
      SORTED_INCR_NA_1ST = 2,
      KNOWN_UNSORTED = 0};
```

from `Rinternals.h`.

Most notably all the math functions used in r disappears here.

## Post and prefix in the C-API

`_R0` means that it does 0-indexing
`_EX` means that it takes ALTREP into account.

## Defining `WIN32`

- `R_ExpandFileNameUTF8`
- `R_WaitEvent`
-

## Macro parsing through LLVM

- [ ] Cannot detect

```c
#define CHAR(x) R_CHAR(x)
const char *(R_CHAR)(SEXP x);
```

second line gets evaded.

Search only a few files:

```
rsys/r/include/**/{R.h,Rconfig.h,R_ext/Arith.h,R_ext/libextern.h,R_ext/Boolean.h,R_ext/Complex.h,R_ext/Constants.h,R_ext/Error.h,R_ext/Memory.h,R_ext/Print.h,R_ext/Random.h,R_ext/Utils.h,R_ext/RS.h,Rinternals.h,R_ext/Rdynload.h}
```

## References

[](https://github.com/hadley/r-internals)
[](https://cran.r-project.org/doc/manuals/r-release/R-ints.html)
[](https://cpp11.r-lib.org/articles/internals.html)
[](https://github.com/hadley/adv-r/blob/master/C-interface.Rmd)
