# Trait impl inventory

Source: `target/doc/miniextendr_lint.json`

Traits with impls: 31

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `Any` | 14 | 0 |
| `Borrow` | 14 | 0 |
| `BorrowMut` | 14 | 0 |
| `Freeze` | 14 | 0 |
| `From` | 14 | 0 |
| `Into` | 14 | 0 |
| `RefUnwindSafe` | 14 | 0 |
| `Send` | 14 | 0 |
| `Sync` | 14 | 0 |
| `TryFrom` | 14 | 0 |
| `TryInto` | 14 | 0 |
| `Unpin` | 14 | 0 |
| `UnsafeUnpin` | 14 | 0 |
| `UnwindSafe` | 14 | 0 |
| `Debug` | 11 | 11 |
| `Clone` | 8 | 8 |
| `CloneToUninit` | 8 | 0 |
| `ToOwned` | 8 | 0 |
| `Eq` | 5 | 5 |
| `PartialEq` | 5 | 5 |
| `Copy` | 4 | 4 |
| `StructuralPartialEq` | 4 | 4 |
| `Default` | 3 | 3 |
| `Display` | 3 | 3 |
| `Hash` | 3 | 3 |
| `ToString` | 3 | 0 |
| `Ord` | 2 | 2 |
| `PartialOrd` | 2 | 2 |
| `IntoIterator` | 1 | 0 |
| `Iterator` | 1 | 1 |
| `MetaToString` | 1 | 1 |

## `Debug` — 11 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintItem` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:101 |
| `AttributedTraitImpl` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:148 |
| `FileData` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:163 |
| `MethodReceiverKind` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:23 |
| `ImplMethodEntry` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:75 |
| `LintKind` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:92 |
| `Diagnostic` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:30 |
| `Severity` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:9 |
| `MiniextendrImplAttrs` | `` | concrete | 1 | miniextendr-lint/src/helpers.rs:61 |
| `LintReport` | `` | concrete | 1 | miniextendr-lint/src/lib.rs:93 |
| `LintCode` | `` | concrete | 1 | miniextendr-lint/src/lint_code.rs:10 |

## `Clone` — 8 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintItem` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:101 |
| `AttributedTraitImpl` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:148 |
| `MethodReceiverKind` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:23 |
| `ImplMethodEntry` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:75 |
| `LintKind` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:92 |
| `Diagnostic` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:30 |
| `Severity` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:9 |
| `LintCode` | `` | concrete | 1 | miniextendr-lint/src/lint_code.rs:10 |

## `Eq` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintItem` | `` | concrete | 0 | miniextendr-lint/src/crate_index.rs:115 |
| `MethodReceiverKind` | `` | concrete | 0 | miniextendr-lint/src/crate_index.rs:23 |
| `LintKind` | `` | concrete | 0 | miniextendr-lint/src/crate_index.rs:92 |
| `Severity` | `` | concrete | 0 | miniextendr-lint/src/diagnostic.rs:9 |
| `LintCode` | `` | concrete | 0 | miniextendr-lint/src/lint_code.rs:10 |

## `PartialEq` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintItem` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:109 |
| `MethodReceiverKind` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:23 |
| `LintKind` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:92 |
| `Severity` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:9 |
| `LintCode` | `` | concrete | 1 | miniextendr-lint/src/lint_code.rs:10 |

## `Copy` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `MethodReceiverKind` | `` | concrete | 0 | miniextendr-lint/src/crate_index.rs:23 |
| `LintKind` | `` | concrete | 0 | miniextendr-lint/src/crate_index.rs:92 |
| `Severity` | `` | concrete | 0 | miniextendr-lint/src/diagnostic.rs:9 |
| `LintCode` | `` | concrete | 0 | miniextendr-lint/src/lint_code.rs:10 |

## `StructuralPartialEq` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `MethodReceiverKind` | `` | concrete | 0 | miniextendr-lint/src/crate_index.rs:23 |
| `LintKind` | `` | concrete | 0 | miniextendr-lint/src/crate_index.rs:92 |
| `Severity` | `` | concrete | 0 | miniextendr-lint/src/diagnostic.rs:9 |
| `LintCode` | `` | concrete | 0 | miniextendr-lint/src/lint_code.rs:10 |

## `Default` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `FileData` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:163 |
| `MiniextendrImplAttrs` | `` | concrete | 1 | miniextendr-lint/src/helpers.rs:61 |
| `LintReport` | `` | concrete | 1 | miniextendr-lint/src/lib.rs:93 |

## `Display` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Severity` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:19 |
| `Diagnostic` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:76 |
| `LintCode` | `` | concrete | 1 | miniextendr-lint/src/lint_code.rs:55 |

## `Hash` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintItem` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:117 |
| `LintKind` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:92 |
| `LintCode` | `` | concrete | 1 | miniextendr-lint/src/lint_code.rs:10 |

## `Ord` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Severity` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:9 |
| `LintCode` | `` | concrete | 1 | miniextendr-lint/src/lint_code.rs:10 |

## `PartialOrd` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Severity` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:9 |
| `LintCode` | `` | concrete | 1 | miniextendr-lint/src/lint_code.rs:10 |

## `Iterator` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SplitTopLevelCommas<'a>` | `<'a>` | concrete | 2 | miniextendr-lint/src/helpers.rs:145 |

## `MetaToString` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `syn::Meta` | `` | concrete | 1 | miniextendr-lint/src/helpers.rs:275 |
