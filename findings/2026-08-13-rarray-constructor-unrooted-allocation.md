# RArray constructor leaves its data SEXP unrooted across allocations

## Finding

`RArray::new` allocated the data vector and then left it unprotected while
allocating the dimension vector and executing an arbitrary initializer closure.
Both operations may trigger R's garbage collector, and the caller cannot root
the data object because the constructor has not returned it yet.

## User impact

Under gctorture, a public `RMatrix::new` probe that allocates once inside its
initializer returned an object whose `dim` attribute was already corrupt on the
first iteration. Subsequent slice writes are a use-after-free risk.

## Resolution

Protect the newly allocated data SEXP immediately and retain that guard across
dimension assignment and the initializer closure. Return the same SEXP only
after all GC-capable constructor work has completed.

The public regression fixture passes 100 constructor iterations under
gctorture, including an R allocation inside every initializer invocation.
