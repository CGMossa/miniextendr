# Trait impl inventory

Source: `cargo-revendor/target/doc/cargo_revendor.json`

Traits with impls: 28

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `Any` | 9 | 0 |
| `Borrow` | 9 | 0 |
| `BorrowMut` | 9 | 0 |
| `Freeze` | 9 | 0 |
| `From` | 9 | 0 |
| `Into` | 9 | 0 |
| `RefUnwindSafe` | 9 | 0 |
| `Same` | 9 | 0 |
| `Send` | 9 | 0 |
| `Sync` | 9 | 0 |
| `TryFrom` | 9 | 0 |
| `TryInto` | 9 | 0 |
| `Unpin` | 9 | 0 |
| `UnsafeUnpin` | 9 | 0 |
| `UnwindSafe` | 9 | 0 |
| `Clone` | 5 | 5 |
| `CloneToUninit` | 5 | 0 |
| `ToOwned` | 5 | 0 |
| `Debug` | 4 | 4 |
| `Copy` | 2 | 2 |
| `Args` | 1 | 1 |
| `CommandFactory` | 1 | 1 |
| `Drop` | 1 | 1 |
| `FromArgMatches` | 1 | 1 |
| `Parser` | 1 | 1 |
| `PartialEq` | 1 | 1 |
| `Serialize` | 1 | 1 |
| `StructuralPartialEq` | 1 | 1 |

## `Clone` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Mode` | `` | concrete | 1 | src/main.rs:261 |
| `Verbosity` | `` | concrete | 1 | src/main.rs:55 |
| `LocalPackage` | `` | concrete | 1 | src/metadata.rs:8 |
| `StripConfig` | `` | concrete | 1 | src/strip.rs:8 |
| `LockPackage` | `` | concrete | 1 | src/verify.rs:23 |

## `Debug` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Verbosity` | `` | concrete | 1 | src/main.rs:55 |
| `LocalPackage` | `` | concrete | 1 | src/metadata.rs:8 |
| `StripConfig` | `` | concrete | 1 | src/strip.rs:8 |
| `LockPackage` | `` | concrete | 1 | src/verify.rs:23 |

## `Copy` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Mode` | `` | concrete | 0 | src/main.rs:261 |
| `Verbosity` | `` | concrete | 0 | src/main.rs:55 |

## `Args` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cli` | `` | concrete | 3 | src/main.rs:70 |

## `CommandFactory` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cli` | `` | concrete | 2 | src/main.rs:70 |

## `Drop` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ManifestGuard` | `` | concrete | 1 | src/manifest_guard.rs:58 |

## `FromArgMatches` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cli` | `` | concrete | 4 | src/main.rs:70 |

## `Parser` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cli` | `` | concrete | 0 | src/main.rs:70 |

## `PartialEq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Mode` | `` | concrete | 1 | src/main.rs:261 |

## `Serialize` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `JsonOutput` | `` | concrete | 1 | src/main.rs:384 |

## `StructuralPartialEq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Mode` | `` | concrete | 0 | src/main.rs:261 |
