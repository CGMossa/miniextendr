# NOTES

`_R0` means that it does 0-indexing
`_EX` means that it takes ALTREP into account.

# Macro parsing through LLVM

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
