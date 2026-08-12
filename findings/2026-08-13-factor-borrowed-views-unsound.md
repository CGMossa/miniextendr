# Factor borrowed views had unconstrained lifetimes and aliasing

## Finding

The public `Factor<'a>` and `FactorMut<'a>` constructors accepted a copyable raw
`SEXP` by value but returned Rust references with an unconstrained caller-chosen
lifetime. Safe code could therefore manufacture a factor view that outlived the
R object:

```rust
fn escape(sexp: SEXP) -> Factor<'static> {
    Factor::try_new(sexp).unwrap()
}
```

`FactorMut::try_new` had an additional aliasing violation. Because `SEXP` is
`Copy` and the constructor was safe, callers could create two simultaneous
mutable slices over the same R integer vector:

```rust
let left = FactorMut::try_new(sexp).unwrap();
let right = FactorMut::try_new(sexp).unwrap();
```

Both patterns violate Rust's reference validity rules. `PhantomData` recorded a
lifetime in the output type but did not tie it to an input borrow or GC root.

## Repository evidence

Neither type had a production consumer, fixture, or runtime test anywhere in
the repository. The supported enum/factor paths use `RFactor`, `FactorVec`,
`FactorOptionVec`, and `UnitEnumFactor` instead. The only other references were
their own documentation, the root re-export, and generated API inventories.

## Resolution

Remove both unused borrowed-view types and their root re-exports. This avoids
adding an owned/rooted factor handle without a demonstrated consumer and keeps
the supported factor conversion API unchanged. Generated API documentation is
regenerated so the removed surface cannot remain discoverable.
