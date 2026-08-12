# Trait impl inventory

Source: `target/doc/miniextendr.json`

Traits with impls: 31

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `Any` | 21 | 0 |
| `Borrow` | 21 | 0 |
| `BorrowMut` | 21 | 0 |
| `Freeze` | 21 | 0 |
| `From` | 21 | 0 |
| `Into` | 21 | 0 |
| `RefUnwindSafe` | 21 | 0 |
| `Send` | 21 | 0 |
| `Sync` | 21 | 0 |
| `TryFrom` | 21 | 0 |
| `TryInto` | 21 | 0 |
| `Unpin` | 21 | 0 |
| `UnsafeUnpin` | 21 | 0 |
| `UnwindSafe` | 21 | 0 |
| `FromArgMatches` | 14 | 14 |
| `Subcommand` | 12 | 12 |
| `Clone` | 6 | 6 |
| `CloneToUninit` | 6 | 0 |
| `ToOwned` | 6 | 0 |
| `Debug` | 5 | 5 |
| `Copy` | 3 | 3 |
| `Args` | 2 | 2 |
| `Display` | 2 | 2 |
| `Equivalent` | 2 | 0 |
| `Serialize` | 2 | 2 |
| `ToString` | 2 | 0 |
| `CommandFactory` | 1 | 1 |
| `Eq` | 1 | 1 |
| `Parser` | 1 | 1 |
| `PartialEq` | 1 | 1 |
| `StructuralPartialEq` | 1 | 1 |

## `FromArgMatches` — 14 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `InitCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:117 |
| `WorkflowCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:169 |
| `StatusCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:227 |
| `CargoBuildOpts` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:241 |
| `CargoCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:263 |
| `Command` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:28 |
| `Cli` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:4 |
| `VendorCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:427 |
| `FeatureCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:477 |
| `FeatureDetectCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:498 |
| `FeatureRuleCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:506 |
| `RenderCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:533 |
| `RustCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:554 |
| `ConfigCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:573 |

## `Subcommand` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `InitCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:117 |
| `WorkflowCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:169 |
| `StatusCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:227 |
| `CargoCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:263 |
| `Command` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:28 |
| `VendorCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:427 |
| `FeatureCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:477 |
| `FeatureDetectCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:498 |
| `FeatureRuleCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:506 |
| `RenderCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:533 |
| `RustCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:554 |
| `ConfigCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:573 |

## `Clone` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `` | concrete | 1 | miniextendr-cli/src/cli.rs:241 |
| `Config` | `` | concrete | 1 | miniextendr-cli/src/commands/config.rs:11 |
| `ProjectContext` | `` | concrete | 1 | miniextendr-cli/src/project.rs:62 |
| `Render` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:221 |
| `Dest` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:325 |
| `PlanEntry` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:336 |

## `Debug` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `` | concrete | 1 | miniextendr-cli/src/cli.rs:241 |
| `ProjectContext` | `` | concrete | 1 | miniextendr-cli/src/project.rs:62 |
| `Render` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:221 |
| `Dest` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:325 |
| `PlanEntry` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:336 |

## `Copy` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Render` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:221 |
| `Dest` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:325 |
| `PlanEntry` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:336 |

## `Args` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:241 |
| `Cli` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:4 |

## `Display` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Config` | `` | concrete | 1 | miniextendr-cli/src/commands/config.rs:21 |
| `HasResult` | `` | concrete | 1 | miniextendr-cli/src/commands/status.rs:18 |

## `Serialize` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Config` | `` | concrete | 1 | miniextendr-cli/src/commands/config.rs:11 |
| `HasResult` | `` | concrete | 1 | miniextendr-cli/src/commands/status.rs:9 |

## `CommandFactory` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cli` | `` | concrete | 2 | miniextendr-cli/src/cli.rs:4 |

## `Eq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Render` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:221 |

## `Parser` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cli` | `` | concrete | 0 | miniextendr-cli/src/cli.rs:4 |

## `PartialEq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Render` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:221 |

## `StructuralPartialEq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Render` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:221 |
