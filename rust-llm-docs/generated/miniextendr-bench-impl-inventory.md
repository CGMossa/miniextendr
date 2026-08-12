# Trait impl inventory

Source: `target/doc/miniextendr_bench.json`

Traits with impls: 39

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `From` | 10 | 1 |
| `Any` | 9 | 0 |
| `Borrow` | 9 | 0 |
| `BorrowMut` | 9 | 0 |
| `Freeze` | 9 | 0 |
| `Into` | 9 | 0 |
| `IntoEither` | 9 | 0 |
| `Pointable` | 9 | 0 |
| `RefUnwindSafe` | 9 | 0 |
| `Send` | 9 | 0 |
| `Sync` | 9 | 0 |
| `TryFrom` | 9 | 0 |
| `TryInto` | 9 | 0 |
| `Unpin` | 9 | 0 |
| `UnsafeUnpin` | 9 | 0 |
| `UnwindSafe` | 9 | 0 |
| `Drop` | 4 | 4 |
| `Clone` | 2 | 2 |
| `CloneToUninit` | 2 | 0 |
| `Copy` | 2 | 2 |
| `Equivalent` | 2 | 0 |
| `RClone` | 2 | 0 |
| `RCopy` | 2 | 0 |
| `ToOwned` | 2 | 0 |
| `Comparable` | 1 | 0 |
| `Debug` | 1 | 1 |
| `Default` | 1 | 1 |
| `Eq` | 1 | 1 |
| `Hash` | 1 | 1 |
| `Key` | 1 | 1 |
| `Ord` | 1 | 1 |
| `PartialEq` | 1 | 1 |
| `PartialOrd` | 1 | 1 |
| `RDebug` | 1 | 0 |
| `RDefault` | 1 | 0 |
| `RHash` | 1 | 0 |
| `ROrd` | 1 | 0 |
| `RPartialOrd` | 1 | 0 |
| `StructuralPartialEq` | 1 | 1 |

## `From` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ProtectKey` | `` | concrete | 1 | miniextendr-bench/src/pool_prototypes.rs:174 |

## `Drop` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `DequePool` | `` | concrete | 1 | miniextendr-bench/src/pool_prototypes.rs:162 |
| `SlotmapPool` | `` | concrete | 1 | miniextendr-bench/src/pool_prototypes.rs:253 |
| `KeyedBacking` | `` | concrete | 1 | miniextendr-bench/src/pool_prototypes.rs:335 |
| `VecPool` | `` | concrete | 1 | miniextendr-bench/src/pool_prototypes.rs:87 |

## `Clone` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Fixtures` | `` | concrete | 1 | miniextendr-bench/src/lib.rs:53 |
| `ProtectKey` | `` | concrete | 1 | miniextendr-bench/src/pool_prototypes.rs:174 |

## `Copy` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Fixtures` | `` | concrete | 0 | miniextendr-bench/src/lib.rs:53 |
| `ProtectKey` | `` | concrete | 0 | miniextendr-bench/src/pool_prototypes.rs:174 |

## `Debug` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ProtectKey` | `` | concrete | 1 | miniextendr-bench/src/pool_prototypes.rs:174 |

## `Default` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ProtectKey` | `` | concrete | 1 | miniextendr-bench/src/pool_prototypes.rs:174 |

## `Eq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ProtectKey` | `` | concrete | 0 | miniextendr-bench/src/pool_prototypes.rs:174 |

## `Hash` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ProtectKey` | `` | concrete | 1 | miniextendr-bench/src/pool_prototypes.rs:174 |

## `Key` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ProtectKey` | `` | concrete | 1 | miniextendr-bench/src/pool_prototypes.rs:174 |

## `Ord` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ProtectKey` | `` | concrete | 1 | miniextendr-bench/src/pool_prototypes.rs:174 |

## `PartialEq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ProtectKey` | `` | concrete | 1 | miniextendr-bench/src/pool_prototypes.rs:174 |

## `PartialOrd` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ProtectKey` | `` | concrete | 1 | miniextendr-bench/src/pool_prototypes.rs:174 |

## `StructuralPartialEq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ProtectKey` | `` | concrete | 0 | miniextendr-bench/src/pool_prototypes.rs:174 |
