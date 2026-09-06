# Trait impl inventory

Source: `target/doc/miniextendr_macros.json`

Traits with impls: 28

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `Any` | 98 | 0 |
| `Borrow` | 98 | 0 |
| `BorrowMut` | 98 | 0 |
| `Freeze` | 98 | 0 |
| `From` | 98 | 0 |
| `Into` | 98 | 0 |
| `RefUnwindSafe` | 98 | 0 |
| `Send` | 98 | 0 |
| `Sync` | 98 | 0 |
| `TryFrom` | 98 | 0 |
| `TryInto` | 98 | 0 |
| `Unpin` | 98 | 0 |
| `UnsafeUnpin` | 98 | 0 |
| `UnwindSafe` | 98 | 0 |
| `Debug` | 31 | 31 |
| `Clone` | 22 | 22 |
| `CloneToUninit` | 22 | 0 |
| `ToOwned` | 22 | 0 |
| `Default` | 20 | 20 |
| `Parse` | 13 | 13 |
| `Eq` | 12 | 12 |
| `PartialEq` | 12 | 12 |
| `StructuralPartialEq` | 12 | 12 |
| `Copy` | 11 | 11 |
| `FromStr` | 2 | 2 |
| `Display` | 1 | 1 |
| `ParsedImplExt` | 1 | 1 |
| `ToString` | 1 | 0 |

## `Debug` — 31 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `ReturnHandling` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:95 |
| `SlotKind` | `` | concrete | 1 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleSpec` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:140 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 1 | miniextendr-macros/src/method_return_builder.rs:92 |
| `ROnExit` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1277 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1315 |
| `VariadicDots` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:145 |
| `ParamAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:603 |
| `ClassSystem` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `VctrsAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:358 |
| `ReceiverKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `ParsedMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:426 |
| `R6MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:458 |
| `S7MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:509 |
| `MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:531 |
| `ParsedImpl` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:702 |
| `ImplAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:783 |
| `TraitMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:120 |
| `TraitConst` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:296 |
| `MethodInfo` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_trait.rs:846 |
| `LowerCall` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:61 |
| `LowerFun` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:70 |
| `LowerArg` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:79 |
| `LowerAtom` | `` | concrete | 1 | miniextendr-macros/src/r_macro/lowering.rs:86 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:341 |
| `SeveralOkContainer` | `` | concrete | 1 | miniextendr-macros/src/type_inspect.rs:149 |

## `Clone` — 22 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `ReturnHandling` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:95 |
| `VariantShape` | `` | concrete | 1 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 1 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleSpec` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:140 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 1 | miniextendr-macros/src/method_return_builder.rs:92 |
| `ROnExit` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1277 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1315 |
| `VariadicDots` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:145 |
| `ReturnPref` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1562 |
| `ParamAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:603 |
| `ClassSystem` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `VctrsAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:358 |
| `ReceiverKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `TraitMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:120 |
| `PreconditionOptions` | `` | concrete | 1 | miniextendr-macros/src/r_preconditions.rs:83 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:341 |
| `SeveralOkContainer` | `` | concrete | 1 | miniextendr-macros/src/type_inspect.rs:149 |

## `Default` — 20 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `FieldAttrs` | `` | concrete | 1 | miniextendr-macros/src/dataframe_derive.rs:128 |
| `RFactorAttrs` | `` | concrete | 1 | miniextendr-macros/src/factor_derive.rs:61 |
| `LifecycleSpec` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:140 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `MatchArgAttrs` | `` | concrete | 1 | miniextendr-macros/src/match_arg_derive.rs:46 |
| `MiniextendrFnAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1147 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1315 |
| `ReturnPref` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1562 |
| `PerParamMiniextendrAttr` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:427 |
| `ParamAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:603 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `VctrsAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:358 |
| `R6MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:458 |
| `S7MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:509 |
| `MethodAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:531 |
| `PreconditionOptions` | `` | concrete | 1 | miniextendr-macros/src/r_preconditions.rs:83 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:341 |
| `RoxygenBuilder` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:635 |
| `RustConversionBuilder` | `` | concrete | 1 | miniextendr-macros/src/rust_conversion_builder.rs:621 |

## `Parse` — 13 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ListInput` | `` | concrete | 1 | miniextendr-macros/src/list_macro.rs:62 |
| `ListEntry` | `` | concrete | 1 | miniextendr-macros/src/list_macro.rs:77 |
| `RenamePair` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1499 |
| `MiniextendrFnAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1597 |
| `MiniextendrFunctionParsed` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:632 |
| `ImplAttrs` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:855 |
| `TpieInput` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:548 |
| `TpieMethod` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl_trait.rs:656 |
| `TypedDataframeField` | `` | concrete | 1 | miniextendr-macros/src/typed_dataframe.rs:108 |
| `TypedDataframeInput` | `` | concrete | 1 | miniextendr-macros/src/typed_dataframe.rs:56 |
| `ParsedTypeSpec` | `` | concrete | 1 | miniextendr-macros/src/typed_list.rs:121 |
| `TypedListInput` | `` | concrete | 1 | miniextendr-macros/src/typed_list.rs:39 |
| `ParsedEntry` | `` | concrete | 1 | miniextendr-macros/src/typed_list.rs:79 |

## `Eq` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `VariantShape` | `` | concrete | 0 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 0 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleStage` | `` | concrete | 0 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 0 | miniextendr-macros/src/method_return_builder.rs:92 |
| `SerdeErrorSpec` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_fn.rs:1315 |
| `ClassSystem` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 0 | miniextendr-macros/src/r_wrapper_builder.rs:341 |

## `PartialEq` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 1 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `VariantShape` | `` | concrete | 1 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 1 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 1 | miniextendr-macros/src/method_return_builder.rs:92 |
| `SerdeErrorSpec` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_fn.rs:1315 |
| `ClassSystem` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 1 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 1 | miniextendr-macros/src/r_wrapper_builder.rs:341 |

## `StructuralPartialEq` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ErrPartsMode` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:187 |
| `SliceBorrow` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `VariantShape` | `` | concrete | 0 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 0 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleStage` | `` | concrete | 0 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 0 | miniextendr-macros/src/method_return_builder.rs:92 |
| `SerdeErrorSpec` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_fn.rs:1315 |
| `ClassSystem` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 0 | miniextendr-macros/src/r_wrapper_builder.rs:341 |

## `Copy` — 11 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SliceBorrow` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:22 |
| `ThreadStrategy` | `` | concrete | 0 | miniextendr-macros/src/c_wrapper_builder.rs:64 |
| `VariantShape` | `` | concrete | 0 | miniextendr-macros/src/dataframe_derive.rs:3112 |
| `SlotKind` | `` | concrete | 0 | miniextendr-macros/src/externalptr_derive.rs:253 |
| `LifecycleStage` | `` | concrete | 0 | miniextendr-macros/src/lifecycle.rs:32 |
| `ReturnStrategy` | `` | concrete | 0 | miniextendr-macros/src/method_return_builder.rs:92 |
| `ReturnPref` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_fn.rs:1562 |
| `ClassSystem` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:263 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:327 |
| `ReceiverKind` | `` | concrete | 0 | miniextendr-macros/src/miniextendr_impl.rs:373 |
| `CallAttribution` | `` | concrete | 0 | miniextendr-macros/src/r_wrapper_builder.rs:341 |

## `FromStr` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ClassSystem` | `` | concrete | 2 | miniextendr-macros/src/miniextendr_impl.rs:310 |
| `VctrsKind` | `` | concrete | 2 | miniextendr-macros/src/miniextendr_impl.rs:341 |

## `Display` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LifecycleStage` | `` | concrete | 1 | miniextendr-macros/src/lifecycle.rs:126 |

## `ParsedImplExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `crate::miniextendr_impl::ParsedImpl` | `` | concrete | 6 | miniextendr-macros/src/r_class_formatter.rs:1038 |
