# Trait impl inventory

Source: `target/doc/miniextendr_api.json`

Traits with impls: 216

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `TryFromSexp` | 469 | 469 |
| `From` | 389 | 74 |
| `IntoR` | 353 | 353 |
| `TryFrom` | 320 | 11 |
| `Borrow` | 310 | 1 |
| `BorrowMut` | 310 | 1 |
| `Send` | 309 | 10 |
| `Pointable` | 309 | 0 |
| `FmtForward` | 309 | 0 |
| `Into` | 309 | 0 |
| `SupersetOf` | 309 | 0 |
| `VZip` | 309 | 0 |
| `Unpin` | 309 | 0 |
| `RefUnwindSafe` | 309 | 1 |
| `UnsafeUnpin` | 309 | 0 |
| `IntoEither` | 309 | 0 |
| `Freeze` | 309 | 0 |
| `Tap` | 309 | 0 |
| `TryInto` | 309 | 0 |
| `Conv` | 309 | 0 |
| `Pipe` | 309 | 0 |
| `Sync` | 309 | 13 |
| `Any` | 309 | 0 |
| `UnwindSafe` | 309 | 0 |
| `TryConv` | 309 | 0 |
| `Same` | 309 | 0 |
| `Allocation` | 218 | 0 |
| `TypedExternal` | 181 | 181 |
| `Equivalent` | 160 | 0 |
| `IntoRAs` | 135 | 135 |
| `RDebug` | 109 | 1 |
| `Debug` | 108 | 108 |
| `RClone` | 101 | 1 |
| `ToOwned` | 100 | 0 |
| `Clone` | 100 | 100 |
| `CloneToUninit` | 100 | 0 |
| `TryCoerce` | 95 | 93 |
| `AltrepLen` | 64 | 64 |
| `RCopy` | 59 | 1 |
| `Copy` | 58 | 58 |
| `Coerce` | 53 | 53 |
| `InferBase` | 45 | 45 |
| `Altrep` | 45 | 45 |
| `AltVec` | 45 | 45 |
| `PartialEq` | 44 | 44 |
| `Scalar` | 43 | 0 |
| `StructuralPartialEq` | 43 | 43 |
| `Eq` | 40 | 40 |
| `DynEq` | 40 | 0 |
| `RegisterAltrep` | 33 | 33 |
| `AltrepDataptr` | 27 | 27 |
| `AltrepSerialize` | 27 | 27 |
| `RDefault` | 25 | 1 |
| `Default` | 24 | 24 |
| `Drop` | 23 | 23 |
| `RDisplay` | 23 | 1 |
| `Display` | 22 | 22 |
| `ToString` | 22 | 0 |
| `Receiver` | 21 | 0 |
| `Deref` | 21 | 21 |
| `Error` | 20 | 20 |
| `RError` | 19 | 1 |
| `RHash` | 16 | 1 |
| `AltIntegerData` | 16 | 16 |
| `AltRealData` | 16 | 16 |
| `Hash` | 15 | 15 |
| `DynHash` | 15 | 0 |
| `MultiUnzip` | 13 | 0 |
| `TraitView` | 12 | 12 |
| `AltReal` | 11 | 11 |
| `IteratorRandom` | 10 | 0 |
| `AltInteger` | 10 | 10 |
| `AltStringData` | 10 | 10 |
| `Serializer` | 10 | 10 |
| `AtomicElement` | 9 | 9 |
| `AltString` | 9 | 9 |
| `AltRawData` | 8 | 8 |
| `DerefMut` | 8 | 8 |
| `SerializeStruct` | 7 | 7 |
| `WidensToF64` | 7 | 7 |
| `IoCaps` | 7 | 7 |
| `TryRngCore` | 7 | 0 |
| `RConnectionImpl` | 7 | 7 |
| `IntoIterator` | 7 | 2 |
| `RngCore` | 7 | 0 |
| `SerializeMap` | 7 | 7 |
| `Rng` | 7 | 0 |
| `AltLogicalData` | 7 | 7 |
| `RSerializeNative` | 6 | 1 |
| `Formattable` | 6 | 0 |
| `AsRef` | 6 | 6 |
| `Parsable` | 6 | 0 |
| `CryptoRng` | 6 | 0 |
| `RSerialize` | 6 | 1 |
| `TryCryptoRng` | 6 | 0 |
| `Deserializer` | 6 | 6 |
| `AltComplexData` | 6 | 6 |
| `AltRaw` | 6 | 6 |
| `RNdArrayOps` | 6 | 6 |
| `AltLogical` | 5 | 5 |
| `AltrepExtract` | 5 | 1 |
| `RPartialOrd` | 5 | 1 |
| `ExactSizeIterator` | 5 | 5 |
| `Serialize` | 5 | 5 |
| `IntoList` | 5 | 5 |
| `ROrd` | 5 | 1 |
| `ProgressIterator` | 5 | 0 |
| `Iterator` | 5 | 5 |
| `TreeNodeIterator` | 5 | 0 |
| `RNativeType` | 5 | 5 |
| `Itertools` | 5 | 0 |
| `SeqAccess` | 5 | 5 |
| `TryFromList` | 5 | 5 |
| `SerializeTupleVariant` | 4 | 4 |
| `RngExt` | 4 | 0 |
| `AltComplex` | 4 | 4 |
| `TryRng` | 4 | 1 |
| `PartialOrd` | 4 | 4 |
| `SerializeStructVariant` | 4 | 4 |
| `Ord` | 4 | 4 |
| `WidensToI32` | 4 | 4 |
| `Comparable` | 4 | 0 |
| `Write` | 3 | 3 |
| `IntoRAltrep` | 3 | 1 |
| `VariantAccess` | 3 | 3 |
| `AsNamedListExt` | 3 | 3 |
| `AsNamedVectorExt` | 3 | 3 |
| `AsRNativeExt` | 3 | 1 |
| `EnumAccess` | 3 | 3 |
| `RNdSlice2D` | 2 | 2 |
| `AsMut` | 2 | 2 |
| `Protector` | 2 | 2 |
| `IntoDataFrame` | 2 | 2 |
| `FromDataFrame` | 2 | 2 |
| `RSourced` | 2 | 2 |
| `NodeTrait` | 2 | 0 |
| `RNdIndex` | 2 | 2 |
| `RNdSlice` | 2 | 2 |
| `MapAccess` | 2 | 2 |
| `AltrepClass` | 2 | 2 |
| `Storage` | 2 | 2 |
| `AsDataFrameExt` | 2 | 1 |
| `RDateTimeFormat` | 2 | 2 |
| `UnitEnumFactor` | 1 | 1 |
| `ROrderedFloatOps` | 1 | 1 |
| `RawStorageMut` | 1 | 1 |
| `RDate` | 1 | 1 |
| `ElementIterator` | 1 | 0 |
| `RTomlOps` | 1 | 1 |
| `IsContiguous` | 1 | 1 |
| `RDeserialize` | 1 | 1 |
| `Log` | 1 | 1 |
| `CheckedBitPattern` | 1 | 0 |
| `SerializeTupleStruct` | 1 | 1 |
| `RVectorOps` | 1 | 1 |
| `SexpExt` | 1 | 1 |
| `BitOr` | 1 | 1 |
| `RRegexOps` | 1 | 1 |
| `RToVec` | 1 | 1 |
| `Pod` | 1 | 1 |
| `RUrlOps` | 1 | 1 |
| `PairListExt` | 1 | 1 |
| `RZoned` | 1 | 1 |
| `RSigned` | 1 | 1 |
| `GlobalAlloc` | 1 | 1 |
| `IntoDataFrameSplit` | 1 | 1 |
| `RFloat` | 1 | 1 |
| `RawStorage` | 1 | 1 |
| `RAhoCorasickOps` | 1 | 1 |
| `RDateTime` | 1 | 1 |
| `RCaptureGroups` | 1 | 1 |
| `RIndexMapOps` | 1 | 1 |
| `RDistributions` | 1 | 1 |
| `RTime` | 1 | 1 |
| `Read` | 1 | 1 |
| `TermLike` | 1 | 1 |
| `AnyBitPattern` | 1 | 0 |
| `RSignedDuration` | 1 | 1 |
| `RBorshOps` | 1 | 1 |
| `NoUninit` | 1 | 0 |
| `AltListData` | 1 | 1 |
| `RTimestamp` | 1 | 1 |
| `RDecimalOps` | 1 | 1 |
| `BitAnd` | 1 | 1 |
| `AsListExt` | 1 | 1 |
| `MatchArg` | 1 | 1 |
| `IntoRecords` | 1 | 0 |
| `AsExternalPtrExt` | 1 | 1 |
| `Not` | 1 | 1 |
| `StorageMut` | 1 | 0 |
| `RDuration` | 1 | 1 |
| `RBigUintBitOps` | 1 | 1 |
| `RSpan` | 1 | 1 |
| `FusedIterator` | 1 | 1 |
| `RJsonValueOps` | 1 | 1 |
| `RFromStr` | 1 | 1 |
| `RJsonBridge` | 1 | 1 |
| `RBigUintOps` | 1 | 1 |
| `RMatrixOps` | 1 | 1 |
| `AltrepSexpExt` | 1 | 1 |
| `SerializeSeq` | 1 | 1 |
| `RUuidOps` | 1 | 1 |
| `SerializeTuple` | 1 | 1 |
| `RDeserializeNative` | 1 | 1 |
| `ParCollectR` | 1 | 1 |
| `RComplexOps` | 1 | 1 |
| `RBigIntBitOps` | 1 | 1 |
| `RFromIter` | 1 | 1 |
| `Pointer` | 1 | 1 |
| `BitXor` | 1 | 1 |
| `RNum` | 1 | 1 |
| `Zeroable` | 1 | 1 |
| `AsVctrsExt` | 1 | 1 |
| `RBigIntOps` | 1 | 1 |
| `IntoRVecElement` | 1 | 1 |
| `DoubleEndedIterator` | 1 | 1 |

## `TryFromSexp` — 469 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `AltrepSexp` | `` | concrete | 3 | miniextendr-api/src/altrep_sexp.rs:282 |
| `AsFromStr<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/convert.rs:1025 |
| `AsFromStrVec<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/convert.rs:1067 |
| `DataFrame` | `` | concrete | 2 | miniextendr-api/src/dataframe.rs:711 |
| `Factor<'a>` | `<'a>` | concrete | 2 | miniextendr-api/src/factor.rs:222 |
| `FactorVec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/factor.rs:517 |
| `FactorOptionVec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/factor.rs:570 |
| `crate::coerce::Coerced<T, R>` | `<T, R> +3wc` | concrete | 3 | miniextendr-api/src/from_r.rs:1020 |
| `Vec<i8>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1370 |
| `Vec<i16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1371 |
| `Vec<i64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1372 |
| `Vec<isize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1373 |
| `Vec<u16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1374 |
| `Vec<u32>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1375 |
| `Vec<u64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1376 |
| `Vec<usize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1377 |
| `Vec<f32>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1378 |
| `Vec<bool>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1381 |
| `std::collections::HashSet<i8>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1446 |
| `std::collections::HashSet<i16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1447 |
| `std::collections::HashSet<i64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1448 |
| `std::collections::HashSet<isize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1449 |
| `std::collections::HashSet<u16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1450 |
| `std::collections::HashSet<u32>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1451 |
| `std::collections::HashSet<u64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1452 |
| `std::collections::HashSet<usize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1453 |
| `std::collections::BTreeSet<i8>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1455 |
| `std::collections::BTreeSet<i16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1456 |
| `std::collections::BTreeSet<i64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1457 |
| `std::collections::BTreeSet<isize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1458 |
| `std::collections::BTreeSet<u16>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1459 |
| `std::collections::BTreeSet<u32>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1460 |
| `std::collections::BTreeSet<u64>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1461 |
| `std::collections::BTreeSet<usize>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1462 |
| `std::collections::HashSet<bool>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1481 |
| `std::collections::BTreeSet<bool>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1482 |
| `crate::externalptr::ExternalPtr<T>` | `<T>` | concrete | 3 | miniextendr-api/src/from_r.rs:1533 |
| `Option<crate::externalptr::ExternalPtr<T>>` | `<T>` | concrete | 3 | miniextendr-api/src/from_r.rs:1578 |
| `Vec<crate::externalptr::ExternalPtr<T>>` | `<T>` | concrete | 3 | miniextendr-api/src/from_r.rs:1607 |
| `Vec<Option<crate::externalptr::ExternalPtr<T>>>` | `<T>` | concrete | 3 | miniextendr-api/src/from_r.rs:1628 |
| `crate::connection::RStdin` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1725 |
| `crate::connection::RStdout` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1738 |
| `crate::connection::RStderr` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1751 |
| `crate::connection::RNullConnection` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1770 |
| `crate::txt_progress_bar::RTxtProgressBar` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:1806 |
| `Box<[T]>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:368 |
| `i32` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:457 |
| `f64` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:533 |
| `u8` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:534 |
| `crate::RLogical` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:535 |
| `crate::Rcomplex` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:536 |
| `crate::SEXP` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:562 |
| `Option<crate::SEXP>` | `` | concrete | 3 | miniextendr-api/src/from_r.rs:582 |
| `&[T]` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:617 |
| `&mut [T]` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:659 |
| `Option<&[T]>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:695 |
| `Option<&mut [T]>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:722 |
| `Result<T, ()>` | `<T> +2wc` | concrete | 3 | miniextendr-api/src/from_r.rs:753 |
| `[T; N]` | `<T, N> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:792 |
| `std::collections::VecDeque<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:830 |
| `std::collections::BinaryHeap<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r.rs:855 |
| `Option<Vec<T>>` | `<T> +2wc` | concrete | 3 | miniextendr-api/src/from_r.rs:881 |
| `Option<std::collections::HashMap<String, V>>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r.rs:932 |
| `Option<std::collections::BTreeMap<String, V>>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r.rs:936 |
| `Option<std::collections::HashSet<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/from_r.rs:965 |
| `Option<std::collections::BTreeSet<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/from_r.rs:969 |
| `Vec<Vec<T>>` | `<T> +2wc` | concrete | 3 | miniextendr-api/src/from_r.rs:980 |
| `i8` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:269 |
| `i16` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:283 |
| `u16` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:297 |
| `u32` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:311 |
| `f32` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:325 |
| `Option<i8>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:339 |
| `Option<i16>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:353 |
| `Option<u16>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:367 |
| `Option<u32>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:381 |
| `Option<f32>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:395 |
| `i64` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:446 |
| `u64` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:464 |
| `Option<i64>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:479 |
| `Option<u64>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:493 |
| `usize` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:507 |
| `Option<usize>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:521 |
| `isize` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:535 |
| `Option<isize>` | `` | concrete | 3 | miniextendr-api/src/from_r/coerced_scalars.rs:549 |
| `Vec<std::collections::HashMap<String, V>>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:144 |
| `Vec<std::collections::BTreeMap<String, V>>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:148 |
| `std::collections::HashSet<i32>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:176 |
| `std::collections::HashSet<u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:177 |
| `std::collections::HashSet<crate::RLogical>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:178 |
| `std::collections::BTreeSet<i32>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:179 |
| `std::collections::BTreeSet<u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:180 |
| `Vec<i32>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:195 |
| `Vec<f64>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:196 |
| `Vec<u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:197 |
| `Vec<crate::RLogical>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:198 |
| `Vec<crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:199 |
| `std::collections::HashMap<String, V>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:38 |
| `std::collections::BTreeMap<String, V>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/from_r/collections.rs:45 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:103 |
| `Vec<String>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:136 |
| `Vec<&'static str>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:154 |
| `Vec<Option<&'static str>>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:168 |
| `std::collections::HashSet<String>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:198 |
| `std::collections::BTreeSet<String>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:202 |
| `Vec<Option<std::path::PathBuf>>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:289 |
| `std::path::PathBuf` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:289 |
| `Vec<std::path::PathBuf>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:289 |
| `Option<std::path::PathBuf>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:289 |
| `std::ffi::OsString` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:312 |
| `Vec<std::ffi::OsString>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:312 |
| `Option<std::ffi::OsString>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:312 |
| `Vec<Option<std::ffi::OsString>>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:312 |
| `std::borrow::Cow<'static, [T]>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:36 |
| `std::borrow::Cow<'static, str>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:62 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 2 | miniextendr-api/src/from_r/cow_and_paths.rs:86 |
| `Option<bool>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:118 |
| `Option<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:142 |
| `Option<i32>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:164 |
| `Option<f64>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:196 |
| `Option<u8>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:228 |
| `Option<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:250 |
| `crate::Rboolean` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:32 |
| `Option<crate::Rboolean>` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:62 |
| `bool` | `` | concrete | 3 | miniextendr-api/src/from_r/logical.rs:92 |
| `Vec<crate::Rboolean>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:101 |
| `Vec<Option<crate::Rboolean>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:131 |
| `Vec<crate::altrep_data::Logical>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:162 |
| `Vec<Option<crate::RLogical>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:185 |
| `Vec<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:210 |
| `Vec<Option<u8>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:225 |
| `Vec<Option<i8>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:302 |
| `Vec<Option<i16>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:303 |
| `Vec<Option<u16>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:304 |
| `Vec<Option<u32>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:305 |
| `Vec<Option<i64>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:306 |
| `Vec<Option<u64>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:307 |
| `Vec<Option<isize>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:308 |
| `Vec<Option<usize>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:309 |
| `Vec<Option<f32>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:310 |
| `Vec<Option<f64>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:62 |
| `Vec<Option<i32>>` | `` | concrete | 2 | miniextendr-api/src/from_r/na_vectors.rs:63 |
| `Vec<Option<bool>>` | `` | concrete | 3 | miniextendr-api/src/from_r/na_vectors.rs:66 |
| `&'static mut i32` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:321 |
| `&'static i32` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:321 |
| `Option<&'static mut i32>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<&'static i32>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<Option<&'static i32>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<&'static mut i32>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<Option<&'static mut i32>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<&'static [i32]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<Option<&'static [i32]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<&'static mut [i32]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<Option<&'static mut [i32]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Option<&'static i32>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:321 |
| `&'static f64` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:322 |
| `Option<&'static mut f64>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<&'static f64>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<Option<&'static f64>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<&'static mut f64>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<Option<&'static mut f64>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<&'static [f64]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<Option<&'static [f64]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<&'static mut [f64]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<Option<&'static mut [f64]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Option<&'static f64>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:322 |
| `&'static mut f64` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:322 |
| `&'static u8` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:323 |
| `Option<&'static mut u8>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<&'static u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<Option<&'static u8>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<&'static mut u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<Option<&'static mut u8>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<&'static [u8]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<Option<&'static [u8]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<&'static mut [u8]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<Option<&'static mut [u8]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Option<&'static u8>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:323 |
| `&'static mut u8` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:323 |
| `&'static mut crate::RLogical` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:324 |
| `&'static crate::RLogical` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:324 |
| `Option<&'static mut crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<&'static crate::RLogical>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<Option<&'static crate::RLogical>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<&'static mut crate::RLogical>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<Option<&'static mut crate::RLogical>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<&'static [crate::RLogical]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<Option<&'static [crate::RLogical]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<&'static mut [crate::RLogical]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<Option<&'static mut [crate::RLogical]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Option<&'static crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:324 |
| `&'static mut crate::Rcomplex` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:325 |
| `Option<&'static mut crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<&'static crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<Option<&'static crate::Rcomplex>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<&'static mut crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<Option<&'static mut crate::Rcomplex>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<&'static [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<Option<&'static [crate::Rcomplex]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<&'static mut [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<Option<&'static mut [crate::Rcomplex]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `&'static crate::Rcomplex` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:325 |
| `Option<&'static crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:325 |
| `char` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:125 |
| `String` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:171 |
| `Option<String>` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:207 |
| `&'static str` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:47 |
| `Option<&'static str>` | `` | concrete | 3 | miniextendr-api/src/from_r/strings.rs:83 |
| `(A,)` | `<A> +1wc` | concrete | 3 | miniextendr-api/src/from_r/tuples.rs:111 |
| `(A, B)` | `<A, B> +2wc` | concrete | 3 | miniextendr-api/src/from_r/tuples.rs:112 |
| `(A, B, C)` | `<A, B, C> +3wc` | concrete | 3 | miniextendr-api/src/from_r/tuples.rs:113 |
| `(A, B, C, D)` | `<A, B, C, D> +4wc` | concrete | 3 | miniextendr-api/src/from_r/tuples.rs:114 |
| `(A, B, C, D, E)` | `<A, B, C, D, E> +5wc` | concrete | 3 | miniextendr-api/src/from_r/tuples.rs:115 |
| `(A, B, C, D, E, F)` | `<A, B, C, D, E, F> +6wc` | concrete | 3 | miniextendr-api/src/from_r/tuples.rs:116 |
| `(A, B, C, D, E, F, G)` | `<A, B, C, D, E, F, G> +7wc` | concrete | 3 | miniextendr-api/src/from_r/tuples.rs:117 |
| `(A, B, C, D, E, F, G, H)` | `<A, B, C, D, E, F, G, H> +8wc` | concrete | 3 | miniextendr-api/src/from_r/tuples.rs:118 |
| `List` | `` | concrete | 2 | miniextendr-api/src/list.rs:1246 |
| `Option<List>` | `` | concrete | 2 | miniextendr-api/src/list.rs:1301 |
| `Option<ListMut>` | `` | concrete | 2 | miniextendr-api/src/list.rs:1313 |
| `ListMut` | `` | concrete | 2 | miniextendr-api/src/list.rs:1325 |
| `NamedList` | `` | concrete | 2 | miniextendr-api/src/list/named.rs:165 |
| `Option<NamedList>` | `` | concrete | 2 | miniextendr-api/src/list/named.rs:175 |
| `Missing<T>` | `<T> +2wc` | concrete | 3 | miniextendr-api/src/missing.rs:253 |
| `NamedVector<std::collections::HashMap<String, V>>` | `<V>` | concrete | 2 | miniextendr-api/src/named_vector.rs:328 |
| `NamedVector<std::collections::BTreeMap<String, V>>` | `<V>` | concrete | 2 | miniextendr-api/src/named_vector.rs:343 |
| `Vec<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/newtype.rs:120 |
| `Option<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/newtype.rs:145 |
| `Vec<Option<T>>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/newtype.rs:165 |
| `Option<AhoCorasick>` | `` | concrete | 3 | miniextendr-api/src/optionals/aho_corasick_impl.rs:102 |
| `Vec<AhoCorasick>` | `` | concrete | 3 | miniextendr-api/src/optionals/aho_corasick_impl.rs:103 |
| `Vec<Option<AhoCorasick>>` | `` | concrete | 3 | miniextendr-api/src/optionals/aho_corasick_impl.rs:104 |
| `AhoCorasick` | `` | concrete | 2 | miniextendr-api/src/optionals/aho_corasick_impl.rs:60 |
| `RecordBatch` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1062 |
| `ArrayRef` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1130 |
| `RPrimitive<Float64Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1311 |
| `RPrimitive<Int32Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1351 |
| `RPrimitive<UInt8Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1364 |
| `RStringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1379 |
| `Float64Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:487 |
| `Int32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:534 |
| `UInt8Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:578 |
| `BooleanArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:616 |
| `StringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:650 |
| `StringDictionaryArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:712 |
| `Date32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:765 |
| `RFlags<T>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:176 |
| `Option<RFlags<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:220 |
| `Vec<RFlags<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:265 |
| `Vec<Option<RFlags<T>>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:309 |
| `BitVec<u8, Msb0>` | `` | concrete | 2 | miniextendr-api/src/optionals/bitvec_impl.rs:142 |
| `Option<BitVec<u8, Msb0>>` | `` | concrete | 3 | miniextendr-api/src/optionals/bitvec_impl.rs:172 |
| `RBitVec` | `` | concrete | 2 | miniextendr-api/src/optionals/bitvec_impl.rs:65 |
| `Option<RBitVec>` | `` | concrete | 3 | miniextendr-api/src/optionals/bitvec_impl.rs:95 |
| `Borsh<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/borsh_impl.rs:56 |
| `Bytes` | `` | concrete | 3 | miniextendr-api/src/optionals/bytes_impl.rs:381 |
| `BytesMut` | `` | concrete | 3 | miniextendr-api/src/optionals/bytes_impl.rs:415 |
| `Option<Bytes>` | `` | concrete | 3 | miniextendr-api/src/optionals/bytes_impl.rs:430 |
| `Option<BytesMut>` | `` | concrete | 3 | miniextendr-api/src/optionals/bytes_impl.rs:451 |
| `RSessionContext` | `` | concrete | 3 | miniextendr-api/src/optionals/datafusion_impl.rs:198 |
| `Either<L, R>` | `<L, R> +4wc` | concrete | 3 | miniextendr-api/src/optionals/either_impl.rs:96 |
| `GlobSet` | `` | concrete | 2 | miniextendr-api/src/optionals/globset_impl.rs:115 |
| `Option<GlobSet>` | `` | concrete | 3 | miniextendr-api/src/optionals/globset_impl.rs:133 |
| `Vec<GlobSet>` | `` | concrete | 3 | miniextendr-api/src/optionals/globset_impl.rs:134 |
| `Vec<Option<GlobSet>>` | `` | concrete | 3 | miniextendr-api/src/optionals/globset_impl.rs:135 |
| `IndexMap<String, T>` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/optionals/indexmap_impl.rs:62 |
| `Zoned` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:129 |
| `Option<Zoned>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:206 |
| `Vec<Zoned>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:261 |
| `Vec<Option<Zoned>>` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:365 |
| `SignedDuration` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `Vec<SignedDuration>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `Option<SignedDuration>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `Vec<Option<SignedDuration>>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `Vec<Option<Timestamp>>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Timestamp` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Vec<Timestamp>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Option<Timestamp>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `JiffTimestampVecMut` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffTimestampVecRef` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffZonedVecRef` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `JiffZonedVecMut` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `Option<Date>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Vec<Option<Date>>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Date` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Vec<Date>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `JiffZonedVec` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:985 |
| `log::LevelFilter` | `` | concrete | 2 | miniextendr-api/src/optionals/log_impl.rs:319 |
| `RDVector<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1307 |
| `RDMatrix<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1320 |
| `SMatrix<T, R, C>` | `<T, R, C> +1wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:246 |
| `Option<SMatrix<T, R, C>>` | `<T, R, C> +1wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:312 |
| `DVector<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:78 |
| `DVector<crate::coerce::Coerced<T, R>>` | `<T, R> +4wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:854 |
| `DMatrix<crate::coerce::Coerced<T, R>>` | `<T, R> +4wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:885 |
| `DMatrix<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:96 |
| `Array0<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:122 |
| `Array1<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:143 |
| `Array2<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:158 |
| `Array3<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:179 |
| `Array4<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:200 |
| `Array5<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:233 |
| `Array6<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:268 |
| `ArrayD<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:303 |
| `RndVec<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3205 |
| `RndMat<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3350 |
| `Array1<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array4<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `ArrayD<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array3<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array6<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array2<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array5<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array4<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `ArrayD<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array3<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array6<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array2<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array5<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array1<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array3<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array6<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array2<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array5<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array1<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array4<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `ArrayD<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array2<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array5<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array1<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array4<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `ArrayD<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array3<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array6<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array2<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array5<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array1<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array4<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `ArrayD<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array3<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array6<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array1<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:673 |
| `Array4<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:673 |
| `ArrayD<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:673 |
| `Array3<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:673 |
| `Array6<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:673 |
| `Array2<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:673 |
| `Array5<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:673 |
| `ArrayD<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:674 |
| `Array3<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:674 |
| `Array6<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:674 |
| `Array2<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:674 |
| `Array5<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:674 |
| `Array1<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:674 |
| `Array4<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:674 |
| `Array3<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:675 |
| `Array6<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:675 |
| `Array2<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:675 |
| `Array5<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:675 |
| `Array1<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:675 |
| `Array4<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:675 |
| `ArrayD<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:675 |
| `Array2<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:679 |
| `Array5<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:679 |
| `Array1<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:679 |
| `Array4<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:679 |
| `ArrayD<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:679 |
| `Array3<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:679 |
| `Array6<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:679 |
| `Array2<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:683 |
| `Array5<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:683 |
| `Array1<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:683 |
| `Array4<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:683 |
| `ArrayD<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:683 |
| `Array3<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:683 |
| `Array6<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:683 |
| `Array0<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `Array3<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `Array6<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `Array2<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `Array5<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `Array1<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `Array4<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `ArrayD<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `Array5<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `Array1<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `Array4<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `ArrayD<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `Array0<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `Array3<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `Array6<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `Array2<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `BigInt` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:193 |
| `Option<BigInt>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:193 |
| `Vec<BigInt>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:193 |
| `Vec<Option<BigInt>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:193 |
| `BigUint` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:194 |
| `Option<BigUint>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:194 |
| `Vec<BigUint>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:194 |
| `Vec<Option<BigUint>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:194 |
| `Option<Complex<f64>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_complex_impl.rs:129 |
| `Vec<Complex<f64>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_complex_impl.rs:171 |
| `Vec<Option<Complex<f64>>>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_complex_impl.rs:218 |
| `Complex<f64>` | `` | concrete | 2 | miniextendr-api/src/optionals/num_complex_impl.rs:92 |
| `OrderedFloat<f64>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:128 |
| `OrderedFloat<f32>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:136 |
| `Option<OrderedFloat<f64>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:144 |
| `Option<OrderedFloat<f32>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:153 |
| `Vec<OrderedFloat<f64>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:164 |
| `Vec<OrderedFloat<f32>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:182 |
| `Vec<Option<OrderedFloat<f64>>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:203 |
| `Vec<Option<OrderedFloat<f32>>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:212 |
| `Option<Regex>` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:67 |
| `Vec<Regex>` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:67 |
| `Vec<Option<Regex>>` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:67 |
| `Regex` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:67 |
| `Decimal` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:146 |
| `Option<Decimal>` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:179 |
| `Vec<Decimal>` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:206 |
| `Vec<Option<Decimal>>` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:251 |
| `JsonValue` | `` | concrete | 2 | miniextendr-api/src/optionals/serde_impl.rs:578 |
| `Option<JsonValue>` | `` | concrete | 3 | miniextendr-api/src/optionals/serde_impl.rs:590 |
| `Vec<JsonValue>` | `` | concrete | 3 | miniextendr-api/src/optionals/serde_impl.rs:591 |
| `Vec<Option<JsonValue>>` | `` | concrete | 3 | miniextendr-api/src/optionals/serde_impl.rs:592 |
| `Option<OffsetDateTime>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Vec<Option<OffsetDateTime>>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `OffsetDateTime` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Vec<OffsetDateTime>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Date` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Vec<Date>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Option<Date>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Vec<Option<Date>>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `ArrayVec<[T; N]>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:102 |
| `Option<TinyVec<[T; N]>>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:178 |
| `Option<ArrayVec<[T; N]>>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:227 |
| `TinyVec<[crate::coerce::Coerced<T, R>; N]>` | `<T, R, N> +4wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:316 |
| `ArrayVec<[crate::coerce::Coerced<T, R>; N]>` | `<T, R, N> +4wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:350 |
| `TinyVec<[T; N]>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:78 |
| `TomlValue` | `` | concrete | 2 | miniextendr-api/src/optionals/toml_impl.rs:122 |
| `Option<TomlValue>` | `` | concrete | 3 | miniextendr-api/src/optionals/toml_impl.rs:167 |
| `Vec<TomlValue>` | `` | concrete | 2 | miniextendr-api/src/optionals/toml_impl.rs:170 |
| `Vec<Option<TomlValue>>` | `` | concrete | 2 | miniextendr-api/src/optionals/toml_impl.rs:193 |
| `Option<Url>` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:48 |
| `Vec<Url>` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:48 |
| `Vec<Option<Url>>` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:48 |
| `Url` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:48 |
| `Option<Uuid>` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:43 |
| `Vec<Uuid>` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:43 |
| `Vec<Option<Uuid>>` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:43 |
| `Uuid` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:43 |
| `RArray<T, NDIM>` | `<T, NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:671 |
| `RArray<i8, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:778 |
| `RArray<i16, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:779 |
| `RArray<i64, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:780 |
| `RArray<isize, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:781 |
| `RArray<u16, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:782 |
| `RArray<u32, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:783 |
| `RArray<u64, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:784 |
| `RArray<usize, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:785 |
| `RArray<f32, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:788 |
| `RArray<bool, NDIM>` | `<NDIM>` | concrete | 3 | miniextendr-api/src/rarray.rs:791 |
| `Raw<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:497 |
| `RawSlice<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:508 |
| `RawTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:519 |
| `RawSliceTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:555 |
| `RCow<'a, T>` | `<'a, T>` | concrete | 3 | miniextendr-api/src/rcow.rs:174 |
| `RValue` | `` | concrete | 2 | miniextendr-api/src/rvalue.rs:107 |
| `FromJson<T>` | `<T>` | concrete | 3 | miniextendr-api/src/serde/json_string.rs:97 |
| `StrVec<'a>` | `<'a>` | concrete | 2 | miniextendr-api/src/strvec.rs:499 |
| `ProtectedStrVec` | `` | concrete | 2 | miniextendr-api/src/strvec.rs:741 |

### `TryFromSexp` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/from_r/references.rs:325** (12 impls): `&'static mut crate::Rcomplex`, `Option<&'static mut crate::Rcomplex>`, `Vec<&'static crate::Rcomplex>`, `Vec<Option<&'static crate::Rcomplex>>`, `Vec<&'static mut crate::Rcomplex>`, `Vec<Option<&'static mut crate::Rcomplex>>`, `Vec<&'static [crate::Rcomplex]>`, `Vec<Option<&'static [crate::Rcomplex]>>`, `Vec<&'static mut [crate::Rcomplex]>`, `Vec<Option<&'static mut [crate::Rcomplex]>>`, `&'static crate::Rcomplex`, `Option<&'static crate::Rcomplex>`
- **miniextendr-api/src/from_r/references.rs:321** (12 impls): `&'static mut i32`, `&'static i32`, `Option<&'static mut i32>`, `Vec<&'static i32>`, `Vec<Option<&'static i32>>`, `Vec<&'static mut i32>`, `Vec<Option<&'static mut i32>>`, `Vec<&'static [i32]>`, `Vec<Option<&'static [i32]>>`, `Vec<&'static mut [i32]>`, `Vec<Option<&'static mut [i32]>>`, `Option<&'static i32>`
- **miniextendr-api/src/from_r/references.rs:323** (12 impls): `&'static u8`, `Option<&'static mut u8>`, `Vec<&'static u8>`, `Vec<Option<&'static u8>>`, `Vec<&'static mut u8>`, `Vec<Option<&'static mut u8>>`, `Vec<&'static [u8]>`, `Vec<Option<&'static [u8]>>`, `Vec<&'static mut [u8]>`, `Vec<Option<&'static mut [u8]>>`, `Option<&'static u8>`, `&'static mut u8`
- **miniextendr-api/src/from_r/references.rs:324** (12 impls): `&'static mut crate::RLogical`, `&'static crate::RLogical`, `Option<&'static mut crate::RLogical>`, `Vec<&'static crate::RLogical>`, `Vec<Option<&'static crate::RLogical>>`, `Vec<&'static mut crate::RLogical>`, `Vec<Option<&'static mut crate::RLogical>>`, `Vec<&'static [crate::RLogical]>`, `Vec<Option<&'static [crate::RLogical]>>`, `Vec<&'static mut [crate::RLogical]>`, `Vec<Option<&'static mut [crate::RLogical]>>`, `Option<&'static crate::RLogical>`
- **miniextendr-api/src/from_r/references.rs:322** (12 impls): `&'static f64`, `Option<&'static mut f64>`, `Vec<&'static f64>`, `Vec<Option<&'static f64>>`, `Vec<&'static mut f64>`, `Vec<Option<&'static mut f64>>`, `Vec<&'static [f64]>`, `Vec<Option<&'static [f64]>>`, `Vec<&'static mut [f64]>`, `Vec<Option<&'static mut [f64]>>`, `Option<&'static f64>`, `&'static mut f64`
- **miniextendr-api/src/optionals/ndarray_impl.rs:867** (8 impls): `Array5<Option<String>>`, `Array1<Option<String>>`, `Array4<Option<String>>`, `ArrayD<Option<String>>`, `Array0<Option<String>>`, `Array3<Option<String>>`, `Array6<Option<String>>`, `Array2<Option<String>>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:866** (8 impls): `Array0<String>`, `Array3<String>`, `Array6<String>`, `Array2<String>`, `Array5<String>`, `Array1<String>`, `Array4<String>`, `ArrayD<String>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:683** (7 impls): `Array2<bool>`, `Array5<bool>`, `Array1<bool>`, `Array4<bool>`, `ArrayD<bool>`, `Array3<bool>`, `Array6<bool>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:669** (7 impls): `Array4<i16>`, `ArrayD<i16>`, `Array3<i16>`, `Array6<i16>`, `Array2<i16>`, `Array5<i16>`, `Array1<i16>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:672** (7 impls): `Array2<u16>`, `Array5<u16>`, `Array1<u16>`, `Array4<u16>`, `ArrayD<u16>`, `Array3<u16>`, `Array6<u16>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:674** (7 impls): `ArrayD<u64>`, `Array3<u64>`, `Array6<u64>`, `Array2<u64>`, `Array5<u64>`, `Array1<u64>`, `Array4<u64>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:675** (7 impls): `Array3<usize>`, `Array6<usize>`, `Array2<usize>`, `Array5<usize>`, `Array1<usize>`, `Array4<usize>`, `ArrayD<usize>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:670** (7 impls): `Array3<i64>`, `Array6<i64>`, `Array2<i64>`, `Array5<i64>`, `Array1<i64>`, `Array4<i64>`, `ArrayD<i64>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:673** (7 impls): `Array1<u32>`, `Array4<u32>`, `ArrayD<u32>`, `Array3<u32>`, `Array6<u32>`, `Array2<u32>`, `Array5<u32>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:668** (7 impls): `Array1<i8>`, `Array4<i8>`, `ArrayD<i8>`, `Array3<i8>`, `Array6<i8>`, `Array2<i8>`, `Array5<i8>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:679** (7 impls): `Array2<f32>`, `Array5<f32>`, `Array1<f32>`, `Array4<f32>`, `ArrayD<f32>`, `Array3<f32>`, `Array6<f32>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:671** (7 impls): `Array2<isize>`, `Array5<isize>`, `Array1<isize>`, `Array4<isize>`, `ArrayD<isize>`, `Array3<isize>`, `Array6<isize>`
- **miniextendr-api/src/optionals/num_bigint_impl.rs:194** (4 impls): `BigUint`, `Option<BigUint>`, `Vec<BigUint>`, `Vec<Option<BigUint>>`
- **miniextendr-api/src/optionals/time_impl.rs:69** (4 impls): `Option<OffsetDateTime>`, `Vec<Option<OffsetDateTime>>`, `OffsetDateTime`, `Vec<OffsetDateTime>`
- **miniextendr-api/src/optionals/jiff_impl.rs:476** (4 impls): `SignedDuration`, `Vec<SignedDuration>`, `Option<SignedDuration>`, `Vec<Option<SignedDuration>>`
- **miniextendr-api/src/from_r/cow_and_paths.rs:312** (4 impls): `std::ffi::OsString`, `Vec<std::ffi::OsString>`, `Option<std::ffi::OsString>`, `Vec<Option<std::ffi::OsString>>`
- **miniextendr-api/src/optionals/time_impl.rs:97** (4 impls): `Date`, `Vec<Date>`, `Option<Date>`, `Vec<Option<Date>>`
- **miniextendr-api/src/optionals/url_impl.rs:48** (4 impls): `Option<Url>`, `Vec<Url>`, `Vec<Option<Url>>`, `Url`
- **miniextendr-api/src/optionals/jiff_impl.rs:81** (4 impls): `Vec<Option<Timestamp>>`, `Timestamp`, `Vec<Timestamp>`, `Option<Timestamp>`
- **miniextendr-api/src/optionals/jiff_impl.rs:98** (4 impls): `Option<Date>`, `Vec<Option<Date>>`, `Date`, `Vec<Date>`
- **miniextendr-api/src/optionals/num_bigint_impl.rs:193** (4 impls): `BigInt`, `Option<BigInt>`, `Vec<BigInt>`, `Vec<Option<BigInt>>`
- **miniextendr-api/src/from_r/cow_and_paths.rs:289** (4 impls): `Vec<Option<std::path::PathBuf>>`, `std::path::PathBuf`, `Vec<std::path::PathBuf>`, `Option<std::path::PathBuf>`
- **miniextendr-api/src/optionals/regex_impl.rs:67** (4 impls): `Option<Regex>`, `Vec<Regex>`, `Vec<Option<Regex>>`, `Regex`
- **miniextendr-api/src/optionals/uuid_impl.rs:43** (4 impls): `Option<Uuid>`, `Vec<Uuid>`, `Vec<Option<Uuid>>`, `Uuid`
- **miniextendr-api/src/optionals/jiff_impl.rs:915** (2 impls): `JiffZonedVecRef`, `JiffZonedVecMut`
- **miniextendr-api/src/optionals/jiff_impl.rs:874** (2 impls): `JiffTimestampVecMut`, `JiffTimestampVecRef`

## `From` — 74 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Logical` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:107 |
| `Logical` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:114 |
| `crate::RLogical` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:121 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:168 |
| `Sortedness` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:182 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:85 |
| `Logical` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:96 |
| `AsRError<E>` | `<E>` | concrete | 1 | miniextendr-api/src/condition.rs:872 |
| `AsList<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:103 |
| `AsExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:365 |
| `AsRNative<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:417 |
| `AsDataFrame<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:487 |
| `AsVctrs<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:534 |
| `AsNamedList<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:606 |
| `AsNamedVector<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:719 |
| `DataFrameError` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:175 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1775 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1782 |
| `FactorVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:482 |
| `FactorOptionVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:540 |
| `SexpError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:303 |
| `SexpError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:309 |
| `SexpError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:315 |
| `Altrep<T>` | `<T>` | concrete | 1 | miniextendr-api/src/into_r/altrep.rs:125 |
| `CoerceErrorKind` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:306 |
| `CoerceErrorKind` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:317 |
| `ListFromSexpError` | `` | concrete | 1 | miniextendr-api/src/list.rs:1240 |
| `crate::from_r::SexpError` | `` | concrete | 1 | miniextendr-api/src/match_arg.rs:101 |
| `Missing<T>` | `<T>` | concrete | 1 | miniextendr-api/src/missing.rs:221 |
| `Missing<T>` | `<T>` | concrete | 1 | miniextendr-api/src/missing.rs:228 |
| `Option<T>` | `<T>` | concrete | 1 | miniextendr-api/src/missing.rs:238 |
| `NamedVector<M>` | `<M>` | concrete | 1 | miniextendr-api/src/named_vector.rs:201 |
| `RFlags<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/bitflags_impl.rs:126 |
| `ArrayView1<'a, T>` | `<'a, T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1738 |
| `ArrayView2<'a, T>` | `<'a, T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1764 |
| `ArrayView3<'a, T>` | `<'a, T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1776 |
| `ArrayViewD<'a, T>` | `<'a, T, NDIM>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1788 |
| `Array1<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1818 |
| `Array2<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1829 |
| `Array3<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1841 |
| `ArrayD<T>` | `<T, NDIM>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:1854 |
| `RCoerceError` | `` | concrete | 1 | miniextendr-api/src/r_coerce.rs:158 |
| `RCoerceError` | `` | concrete | 1 | miniextendr-api/src/r_coerce.rs:164 |
| `RCow<'_, T>` | `<T>` | concrete | 1 | miniextendr-api/src/rcow.rs:159 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:184 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:189 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:194 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:199 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:204 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:209 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:214 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:219 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:224 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:229 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:243 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:248 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:253 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:258 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:263 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:268 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:273 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:282 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:287 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:292 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:301 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:311 |
| `CollatedResultRow<'a, T, E>` | `<'a, T, E>` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:4598 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:255 |
| `SEXP` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:482 |
| `*mut SEXPREC` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:489 |
| `RLogical` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:202 |
| `Rboolean` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:330 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:339 |
| `TypedListError` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:265 |

## `IntoR` — 353 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `AsList<T>` | `<T>` | concrete | 4 | miniextendr-api/src/convert.rs:109 |
| `Collect<I>` | `<I, T> +2wc` | concrete | 5 | miniextendr-api/src/convert.rs:1152 |
| `CollectStrings<I>` | `<I> +1wc` | concrete | 3 | miniextendr-api/src/convert.rs:1208 |
| `CollectNA<I>` | `<I> +1wc` | concrete | 5 | miniextendr-api/src/convert.rs:1249 |
| `CollectNAInt<I>` | `<I> +1wc` | concrete | 5 | miniextendr-api/src/convert.rs:1294 |
| `AsExternalPtr<T>` | `<T>` | concrete | 4 | miniextendr-api/src/convert.rs:371 |
| `AsRNative<T>` | `<T>` | concrete | 5 | miniextendr-api/src/convert.rs:423 |
| `AsDataFrame<T>` | `<T>` | concrete | 3 | miniextendr-api/src/convert.rs:493 |
| `AsVctrs<T>` | `<T>` | concrete | 3 | miniextendr-api/src/convert.rs:541 |
| `AsNamedList<Vec<(K, V)>>` | `<K, V>` | concrete | 4 | miniextendr-api/src/convert.rs:612 |
| `AsNamedList<[(K, V); N]>` | `<K, V, N>` | concrete | 4 | miniextendr-api/src/convert.rs:641 |
| `AsNamedList<&[(K, V)]>` | `<K, V>` | concrete | 4 | miniextendr-api/src/convert.rs:667 |
| `AsNamedVector<Vec<(K, V)>>` | `<K, V>` | concrete | 4 | miniextendr-api/src/convert.rs:725 |
| `AsNamedVector<[(K, V); N]>` | `<K, V, N>` | concrete | 4 | miniextendr-api/src/convert.rs:741 |
| `AsNamedVector<&[(K, V)]>` | `<K, V>` | concrete | 4 | miniextendr-api/src/convert.rs:757 |
| `AsDisplay<T>` | `<T>` | concrete | 3 | miniextendr-api/src/convert.rs:964 |
| `AsDisplayVec<T>` | `<T>` | concrete | 3 | miniextendr-api/src/convert.rs:991 |
| `DataFrame` | `` | concrete | 4 | miniextendr-api/src/dataframe.rs:719 |
| `BuiltDataFrame` | `` | concrete | 4 | miniextendr-api/src/dataframe.rs:927 |
| `FactorVec<T>` | `<T>` | concrete | 4 | miniextendr-api/src/factor.rs:501 |
| `FactorOptionVec<T>` | `<T>` | concrete | 4 | miniextendr-api/src/factor.rs:610 |
| `BTreeSet<i8>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1038 |
| `HashSet<i8>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1038 |
| `HashSet<i16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1039 |
| `BTreeSet<i16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1039 |
| `HashSet<u16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1040 |
| `BTreeSet<u16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1040 |
| `Option<Vec<T>>` | `<T>` | concrete | 5 | miniextendr-api/src/into_r.rs:1049 |
| `Option<Vec<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1076 |
| `Option<std::collections::HashMap<String, V>>` | `<V>` | concrete | 5 | miniextendr-api/src/into_r.rs:1103 |
| `Option<std::collections::BTreeMap<String, V>>` | `<V>` | concrete | 5 | miniextendr-api/src/into_r.rs:1130 |
| `Option<std::collections::HashSet<T>>` | `<T>` | concrete | 5 | miniextendr-api/src/into_r.rs:1157 |
| `Option<std::collections::BTreeSet<T>>` | `<T>` | concrete | 5 | miniextendr-api/src/into_r.rs:1184 |
| `Option<std::collections::HashSet<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1241 |
| `Option<std::collections::BTreeSet<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1245 |
| `Vec<String>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1323 |
| `&[String]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1341 |
| `&[&str]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1359 |
| `Vec<std::borrow::Cow<'_, str>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1377 |
| `Vec<Option<std::borrow::Cow<'_, str>>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1396 |
| `Vec<Option<&str>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1418 |
| `Vec<&str>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1436 |
| `Vec<Vec<T>>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:1514 |
| `Vec<&[T]>` | `<T>` | concrete | 5 | miniextendr-api/src/into_r.rs:1537 |
| `Vec<&[String]>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1556 |
| `crate::SEXP` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:157 |
| `Vec<Vec<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1573 |
| `Vec<Option<f64>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1635 |
| `Vec<Option<i32>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1636 |
| `Vec<Option<i64>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1674 |
| `Vec<Option<u64>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1675 |
| `Vec<Option<isize>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1676 |
| `Vec<Option<usize>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1678 |
| `Vec<Option<i8>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1697 |
| `Vec<Option<i16>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1698 |
| `Vec<Option<u16>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1699 |
| `Vec<Option<u32>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1700 |
| `Vec<Option<f32>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1701 |
| `Vec<bool>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1734 |
| `&[bool]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1754 |
| `Vec<crate::Rboolean>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1774 |
| `&[crate::Rboolean]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1794 |
| `Vec<Option<bool>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1837 |
| `Vec<Option<crate::Rboolean>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1846 |
| `Vec<Option<crate::RLogical>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1854 |
| `Vec<Option<String>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:1866 |
| `(A,)` | `<A>` | concrete | 5 | miniextendr-api/src/into_r.rs:1924 |
| `(A, B)` | `<A, B>` | concrete | 5 | miniextendr-api/src/into_r.rs:1925 |
| `(A, B, C)` | `<A, B, C>` | concrete | 5 | miniextendr-api/src/into_r.rs:1926 |
| `(A, B, C, D)` | `<A, B, C, D>` | concrete | 5 | miniextendr-api/src/into_r.rs:1927 |
| `(A, B, C, D, E)` | `<A, B, C, D, E>` | concrete | 5 | miniextendr-api/src/into_r.rs:1928 |
| `(A, B, C, D, E, F)` | `<A, B, C, D, E, F>` | concrete | 5 | miniextendr-api/src/into_r.rs:1929 |
| `(A, B, C, D, E, F, G)` | `<A, B, C, D, E, F, G>` | concrete | 5 | miniextendr-api/src/into_r.rs:1930 |
| `(A, B, C, D, E, F, G, H)` | `<A, B, C, D, E, F, G, H>` | concrete | 5 | miniextendr-api/src/into_r.rs:1931 |
| `()` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:196 |
| `Vec<Box<[T]>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2034 |
| `Vec<Box<[String]>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2051 |
| `Vec<[T; N]>` | `<T, N> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2057 |
| `Vec<Option<Vec<T>>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2092 |
| `Vec<Option<Vec<String>>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2109 |
| `Vec<Option<std::collections::HashSet<T>>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2114 |
| `std::convert::Infallible` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:212 |
| `Vec<Option<std::collections::HashSet<String>>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2131 |
| `Vec<Option<std::collections::BTreeSet<T>>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2136 |
| `Vec<Option<std::collections::BTreeSet<String>>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2153 |
| `Vec<Option<std::collections::HashMap<String, V>>>` | `<V>` | concrete | 4 | miniextendr-api/src/into_r.rs:2158 |
| `Vec<Option<std::collections::BTreeMap<String, V>>>` | `<V>` | concrete | 4 | miniextendr-api/src/into_r.rs:2172 |
| `Vec<Option<&[T]>>` | `<T>` | concrete | 4 | miniextendr-api/src/into_r.rs:2188 |
| `Vec<Option<&[String]>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2203 |
| `Vec<std::collections::HashSet<T>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2211 |
| `Vec<std::collections::BTreeSet<T>>` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/into_r.rs:2230 |
| `Vec<std::collections::HashSet<String>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2248 |
| `Vec<std::collections::BTreeSet<String>>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:2254 |
| `Vec<std::collections::HashMap<String, V>>` | `<V>` | concrete | 4 | miniextendr-api/src/into_r.rs:2277 |
| `Vec<std::collections::BTreeMap<String, V>>` | `<V>` | concrete | 4 | miniextendr-api/src/into_r.rs:2281 |
| `crate::connection::RStdin` | `` | concrete | 3 | miniextendr-api/src/into_r.rs:2319 |
| `crate::connection::RStdout` | `` | concrete | 3 | miniextendr-api/src/into_r.rs:2331 |
| `crate::connection::RStderr` | `` | concrete | 3 | miniextendr-api/src/into_r.rs:2343 |
| `crate::connection::RNullConnection` | `` | concrete | 3 | miniextendr-api/src/into_r.rs:2356 |
| `crate::txt_progress_bar::RTxtProgressBar` | `` | concrete | 3 | miniextendr-api/src/into_r.rs:2388 |
| `i32` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:253 |
| `f64` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:254 |
| `u8` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:255 |
| `crate::Rcomplex` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:256 |
| `Option<crate::Rcomplex>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:299 |
| `i8` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:359 |
| `i16` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:360 |
| `u16` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:361 |
| `f32` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:364 |
| `u32` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:365 |
| `Vec<i32>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:407 |
| `Vec<f64>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:408 |
| `Vec<u8>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:409 |
| `Vec<crate::RLogical>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:410 |
| `Vec<crate::Rcomplex>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:411 |
| `&[T]` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:413 |
| `Box<[T]>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:443 |
| `Vec<i8>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:604 |
| `&[i8]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:604 |
| `Vec<i16>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:605 |
| `&[i16]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:605 |
| `Vec<u16>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:606 |
| `&[u16]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:606 |
| `Vec<f32>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:609 |
| `&[f32]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:609 |
| `Vec<i64>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:667 |
| `Vec<u64>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:668 |
| `Vec<isize>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:669 |
| `Vec<usize>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:671 |
| `Vec<u32>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:674 |
| `[T; N]` | `<T, N>` | concrete | 5 | miniextendr-api/src/into_r.rs:690 |
| `std::collections::VecDeque<T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:716 |
| `std::collections::BinaryHeap<T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:746 |
| `std::borrow::Cow<'_, [T]>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r.rs:782 |
| `std::borrow::Cow<'_, str>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:802 |
| `Option<std::path::PathBuf>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:969 |
| `Vec<std::path::PathBuf>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:969 |
| `Vec<Option<std::path::PathBuf>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:969 |
| `std::path::PathBuf` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:969 |
| `&std::path::Path` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:969 |
| `Option<std::ffi::OsString>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:985 |
| `Vec<std::ffi::OsString>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:985 |
| `Vec<Option<std::ffi::OsString>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:985 |
| `std::ffi::OsString` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:985 |
| `&std::ffi::OsStr` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:985 |
| `Altrep<T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r/altrep.rs:152 |
| `crate::altrep_sexp::AltrepSexp` | `` | concrete | 4 | miniextendr-api/src/into_r/altrep.rs:192 |
| `std::collections::HashSet<T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r/collections.rs:118 |
| `std::collections::BTreeSet<T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r/collections.rs:140 |
| `std::collections::HashSet<String>` | `` | concrete | 5 | miniextendr-api/src/into_r/collections.rs:184 |
| `std::collections::BTreeSet<String>` | `` | concrete | 5 | miniextendr-api/src/into_r/collections.rs:188 |
| `std::collections::HashMap<String, V>` | `<V>` | concrete | 5 | miniextendr-api/src/into_r/collections.rs:44 |
| `std::collections::BTreeMap<String, V>` | `<V>` | concrete | 5 | miniextendr-api/src/into_r/collections.rs:48 |
| `isize` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:117 |
| `usize` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:141 |
| `bool` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:186 |
| `crate::Rboolean` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:187 |
| `crate::RLogical` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:188 |
| `Option<i32>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:190 |
| `Option<f64>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:216 |
| `Option<crate::Rboolean>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:242 |
| `Option<crate::RLogical>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:269 |
| `Option<bool>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:295 |
| `Option<i64>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:348 |
| `Option<u64>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:349 |
| `Option<isize>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:350 |
| `Option<usize>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:352 |
| `Option<i8>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:375 |
| `Option<i16>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:376 |
| `Option<u16>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:377 |
| `Option<u32>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:378 |
| `Option<f32>` | `` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:379 |
| `crate::externalptr::ExternalPtr<T>` | `<T>` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:381 |
| `Vec<crate::externalptr::ExternalPtr<T>>` | `<T>` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:404 |
| `Vec<Option<crate::externalptr::ExternalPtr<T>>>` | `<T>` | concrete | 4 | miniextendr-api/src/into_r/large_integers.rs:419 |
| `T` | `<T>` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:493 |
| `i64` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:50 |
| `String` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:538 |
| `Box<str>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:558 |
| `char` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:580 |
| `&str` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:605 |
| `Option<&str>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:639 |
| `Option<String>` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:676 |
| `Option<&T>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:707 |
| `u64` | `` | concrete | 5 | miniextendr-api/src/into_r/large_integers.rs:85 |
| `Result<T, NullOnErr>` | `<T>` | concrete | 4 | miniextendr-api/src/into_r/result.rs:142 |
| `Result<T, E>` | `<T, E> +2wc` | concrete | 4 | miniextendr-api/src/into_r/result.rs:81 |
| `List` | `` | concrete | 4 | miniextendr-api/src/list.rs:1113 |
| `ListMut` | `` | concrete | 4 | miniextendr-api/src/list.rs:1127 |
| `Vec<List>` | `` | concrete | 4 | miniextendr-api/src/list.rs:1146 |
| `Vec<Option<List>>` | `` | concrete | 4 | miniextendr-api/src/list.rs:1177 |
| `NamedList` | `` | concrete | 4 | miniextendr-api/src/list/named.rs:151 |
| `NamedVector<std::collections::HashMap<String, V>>` | `<V>` | concrete | 4 | miniextendr-api/src/named_vector.rs:287 |
| `NamedVector<std::collections::BTreeMap<String, V>>` | `<V>` | concrete | 4 | miniextendr-api/src/named_vector.rs:306 |
| `Vec<Option<T>>` | `<T> +1wc` | concrete | 5 | miniextendr-api/src/newtype.rs:203 |
| `Vec<T>` | `<T>` | concrete | 4 | miniextendr-api/src/newtype.rs:97 |
| `Float64Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1155 |
| `Int32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1191 |
| `UInt8Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1224 |
| `BooleanArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1253 |
| `StringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1277 |
| `RPrimitive<Float64Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1345 |
| `RPrimitive<Int32Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1346 |
| `RPrimitive<UInt8Type>` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1347 |
| `RStringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1392 |
| `RecordBatch` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1486 |
| `ArrayRef` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1537 |
| `StringDictionaryArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:855 |
| `Date32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:902 |
| `TimestampSecondArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:934 |
| `RFlags<T>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:356 |
| `Option<RFlags<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:375 |
| `Vec<RFlags<T>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:390 |
| `Vec<Option<RFlags<T>>>` | `<T> +2wc` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:406 |
| `RBitVec` | `` | concrete | 4 | miniextendr-api/src/optionals/bitvec_impl.rs:100 |
| `Option<RBitVec>` | `` | concrete | 4 | miniextendr-api/src/optionals/bitvec_impl.rs:123 |
| `BitVec<u8, Msb0>` | `` | concrete | 4 | miniextendr-api/src/optionals/bitvec_impl.rs:174 |
| `Option<BitVec<u8, Msb0>>` | `` | concrete | 4 | miniextendr-api/src/optionals/bitvec_impl.rs:197 |
| `Borsh<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/borsh_impl.rs:38 |
| `Bytes` | `` | concrete | 5 | miniextendr-api/src/optionals/bytes_impl.rs:362 |
| `BytesMut` | `` | concrete | 5 | miniextendr-api/src/optionals/bytes_impl.rs:396 |
| `Option<Bytes>` | `` | concrete | 5 | miniextendr-api/src/optionals/bytes_impl.rs:472 |
| `Option<BytesMut>` | `` | concrete | 5 | miniextendr-api/src/optionals/bytes_impl.rs:496 |
| `Either<L, R>` | `<L, R> +2wc` | concrete | 3 | miniextendr-api/src/optionals/either_impl.rs:145 |
| `IndexMap<String, T>` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/optionals/indexmap_impl.rs:117 |
| `Zoned` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:178 |
| `Option<Zoned>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:235 |
| `Vec<Zoned>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:310 |
| `Vec<Option<Zoned>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:413 |
| `SignedDuration` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `Vec<Option<SignedDuration>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `Option<SignedDuration>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `Vec<SignedDuration>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `Vec<Option<Timestamp>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Option<Timestamp>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Vec<Timestamp>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Timestamp` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `JiffTimestampVec` | `` | concrete | 5 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffZonedVec` | `` | concrete | 5 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `Vec<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Date` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Vec<Option<Date>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Option<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `log::LevelFilter` | `` | concrete | 4 | miniextendr-api/src/optionals/log_impl.rs:332 |
| `RDVector<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1334 |
| `RDMatrix<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1347 |
| `DVector<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:137 |
| `DMatrix<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:157 |
| `SMatrix<T, R, C>` | `<T, R, C>` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:286 |
| `Option<SMatrix<T, R, C>>` | `<T, R, C>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:335 |
| `DVector<crate::coerce::Coerced<T, R>>` | `<T, R> +3wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:924 |
| `DMatrix<crate::coerce::Coerced<T, R>>` | `<T, R> +3wc` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:946 |
| `Array5<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1010 |
| `Array6<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1054 |
| `ArrayD<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1097 |
| `Array0<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1264 |
| `Array0<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1264 |
| `Array1<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1265 |
| `Array1<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1265 |
| `Array2<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1266 |
| `Array2<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1266 |
| `Array3<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1267 |
| `Array3<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1267 |
| `Array4<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1268 |
| `Array4<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1268 |
| `Array5<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1269 |
| `Array5<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1269 |
| `Array6<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1270 |
| `Array6<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1270 |
| `ArrayD<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1271 |
| `ArrayD<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1271 |
| `ArcArray1<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1277 |
| `ArcArray2<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1294 |
| `ArrayView1<'a, T>` | `<'a, T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1315 |
| `ArrayView2<'a, T>` | `<'a, T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1333 |
| `ArrayView3<'a, T>` | `<'a, T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1364 |
| `ArrayViewD<'a, T>` | `<'a, T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1407 |
| `RndVec<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3217 |
| `Array0<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:328 |
| `RndMat<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3362 |
| `Array1<T>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:345 |
| `Array2<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:876 |
| `Array3<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:918 |
| `Array4<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:965 |
| `BigInt` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:196 |
| `BigUint` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:197 |
| `Option<BigInt>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:198 |
| `Option<BigUint>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:201 |
| `Vec<BigInt>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:204 |
| `Vec<BigUint>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:209 |
| `Vec<Option<BigInt>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:214 |
| `Vec<Option<BigUint>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_bigint_impl.rs:219 |
| `Complex<f64>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_complex_impl.rs:124 |
| `Option<Complex<f64>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_complex_impl.rs:163 |
| `Vec<Complex<f64>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_complex_impl.rs:201 |
| `Vec<Option<Complex<f64>>>` | `` | concrete | 4 | miniextendr-api/src/optionals/num_complex_impl.rs:247 |
| `OrderedFloat<f64>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:227 |
| `OrderedFloat<f32>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:228 |
| `Option<OrderedFloat<f64>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:229 |
| `Option<OrderedFloat<f32>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:232 |
| `Vec<OrderedFloat<f64>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:235 |
| `Vec<OrderedFloat<f32>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:240 |
| `Vec<Option<OrderedFloat<f64>>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:245 |
| `Vec<Option<OrderedFloat<f32>>>` | `` | concrete | 4 | miniextendr-api/src/optionals/ordered_float_impl.rs:250 |
| `Decimal` | `` | concrete | 4 | miniextendr-api/src/optionals/rust_decimal_impl.rs:287 |
| `Option<Decimal>` | `` | concrete | 4 | miniextendr-api/src/optionals/rust_decimal_impl.rs:288 |
| `Vec<Decimal>` | `` | concrete | 4 | miniextendr-api/src/optionals/rust_decimal_impl.rs:291 |
| `Vec<Option<Decimal>>` | `` | concrete | 4 | miniextendr-api/src/optionals/rust_decimal_impl.rs:296 |
| `JsonValue` | `` | concrete | 4 | miniextendr-api/src/optionals/serde_impl.rs:597 |
| `Option<JsonValue>` | `` | concrete | 4 | miniextendr-api/src/optionals/serde_impl.rs:610 |
| `Vec<JsonValue>` | `` | concrete | 4 | miniextendr-api/src/optionals/serde_impl.rs:626 |
| `Vec<Option<JsonValue>>` | `` | concrete | 4 | miniextendr-api/src/optionals/serde_impl.rs:653 |
| `Table` | `` | concrete | 4 | miniextendr-api/src/optionals/tabled_impl.rs:197 |
| `OffsetDateTime` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Vec<Option<OffsetDateTime>>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Option<OffsetDateTime>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Vec<OffsetDateTime>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Vec<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Date` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Vec<Option<Date>>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Option<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `TinyVec<[T; N]>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:140 |
| `ArrayVec<[T; N]>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:159 |
| `Option<TinyVec<[T; N]>>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:202 |
| `Option<ArrayVec<[T; N]>>` | `<T, N> +2wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:251 |
| `TinyVec<[crate::coerce::Coerced<T, R>; N]>` | `<T, R, N> +3wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:391 |
| `ArrayVec<[crate::coerce::Coerced<T, R>; N]>` | `<T, R, N> +3wc` | concrete | 3 | miniextendr-api/src/optionals/tinyvec_impl.rs:413 |
| `TomlValue` | `` | concrete | 4 | miniextendr-api/src/optionals/toml_impl.rs:214 |
| `Option<TomlValue>` | `` | concrete | 4 | miniextendr-api/src/optionals/toml_impl.rs:236 |
| `Vec<TomlValue>` | `` | concrete | 4 | miniextendr-api/src/optionals/toml_impl.rs:252 |
| `Vec<Option<TomlValue>>` | `` | concrete | 4 | miniextendr-api/src/optionals/toml_impl.rs:279 |
| `Url` | `` | concrete | 4 | miniextendr-api/src/optionals/url_impl.rs:50 |
| `Option<Url>` | `` | concrete | 4 | miniextendr-api/src/optionals/url_impl.rs:52 |
| `Vec<Url>` | `` | concrete | 4 | miniextendr-api/src/optionals/url_impl.rs:56 |
| `Vec<Option<Url>>` | `` | concrete | 4 | miniextendr-api/src/optionals/url_impl.rs:62 |
| `Uuid` | `` | concrete | 4 | miniextendr-api/src/optionals/uuid_impl.rs:45 |
| `Option<Uuid>` | `` | concrete | 4 | miniextendr-api/src/optionals/uuid_impl.rs:46 |
| `Vec<Uuid>` | `` | concrete | 4 | miniextendr-api/src/optionals/uuid_impl.rs:47 |
| `Vec<Option<Uuid>>` | `` | concrete | 4 | miniextendr-api/src/optionals/uuid_impl.rs:52 |
| `RArray<T, NDIM>` | `<T, NDIM>` | concrete | 5 | miniextendr-api/src/rarray.rs:796 |
| `Raw<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:408 |
| `RawSlice<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:422 |
| `RawTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:436 |
| `RawSliceTagged<T>` | `<T>` | concrete | 2 | miniextendr-api/src/raw_conversions.rs:465 |
| `RCow<'_, T>` | `<T>` | concrete | 5 | miniextendr-api/src/rcow.rs:193 |
| `RValue` | `` | concrete | 4 | miniextendr-api/src/rvalue.rs:50 |
| `DataFrameShape` | `` | concrete | 3 | miniextendr-api/src/serde/columnar.rs:3249 |
| `AsJsonVec<T>` | `<T>` | concrete | 3 | miniextendr-api/src/serde/json_string.rs:131 |
| `AsJson<T>` | `<T>` | concrete | 3 | miniextendr-api/src/serde/json_string.rs:34 |
| `AsJsonPretty<T>` | `<T>` | concrete | 3 | miniextendr-api/src/serde/json_string.rs:58 |
| `AsSerialize<T>` | `<T>` | concrete | 2 | miniextendr-api/src/serde/traits.rs:262 |
| `StrVec<'_>` | `` | concrete | 4 | miniextendr-api/src/strvec.rs:485 |
| `ProtectedStrVec` | `` | concrete | 4 | miniextendr-api/src/strvec.rs:727 |

### `IntoR` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/into_r.rs:985** (5 impls): `Option<std::ffi::OsString>`, `Vec<std::ffi::OsString>`, `Vec<Option<std::ffi::OsString>>`, `std::ffi::OsString`, `&std::ffi::OsStr`
- **miniextendr-api/src/into_r.rs:969** (5 impls): `Option<std::path::PathBuf>`, `Vec<std::path::PathBuf>`, `Vec<Option<std::path::PathBuf>>`, `std::path::PathBuf`, `&std::path::Path`
- **miniextendr-api/src/optionals/jiff_impl.rs:81** (4 impls): `Vec<Option<Timestamp>>`, `Option<Timestamp>`, `Vec<Timestamp>`, `Timestamp`
- **miniextendr-api/src/optionals/jiff_impl.rs:98** (4 impls): `Vec<Date>`, `Date`, `Vec<Option<Date>>`, `Option<Date>`
- **miniextendr-api/src/optionals/time_impl.rs:69** (4 impls): `OffsetDateTime`, `Vec<Option<OffsetDateTime>>`, `Option<OffsetDateTime>`, `Vec<OffsetDateTime>`
- **miniextendr-api/src/optionals/jiff_impl.rs:476** (4 impls): `SignedDuration`, `Vec<Option<SignedDuration>>`, `Option<SignedDuration>`, `Vec<SignedDuration>`
- **miniextendr-api/src/optionals/time_impl.rs:97** (4 impls): `Vec<Date>`, `Date`, `Vec<Option<Date>>`, `Option<Date>`
- **miniextendr-api/src/into_r.rs:1038** (2 impls): `BTreeSet<i8>`, `HashSet<i8>`
- **miniextendr-api/src/into_r.rs:1040** (2 impls): `HashSet<u16>`, `BTreeSet<u16>`
- **miniextendr-api/src/into_r.rs:604** (2 impls): `Vec<i8>`, `&[i8]`
- **miniextendr-api/src/into_r.rs:1039** (2 impls): `HashSet<i16>`, `BTreeSet<i16>`
- **miniextendr-api/src/into_r.rs:605** (2 impls): `Vec<i16>`, `&[i16]`
- **miniextendr-api/src/into_r.rs:606** (2 impls): `Vec<u16>`, `&[u16]`
- **miniextendr-api/src/into_r.rs:609** (2 impls): `Vec<f32>`, `&[f32]`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1264** (2 impls): `Array0<String>`, `Array0<Option<String>>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1265** (2 impls): `Array1<String>`, `Array1<Option<String>>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1266** (2 impls): `Array2<String>`, `Array2<Option<String>>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1267** (2 impls): `Array3<String>`, `Array3<Option<String>>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1268** (2 impls): `Array4<String>`, `Array4<Option<String>>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1269** (2 impls): `Array5<String>`, `Array5<Option<String>>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1270** (2 impls): `Array6<String>`, `Array6<Option<String>>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1271** (2 impls): `ArrayD<String>`, `ArrayD<Option<String>>`

## `TryFrom` — 11 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `i32` | `` | concrete | 2 | miniextendr-api/src/rvalue.rs:435 |
| `bool` | `` | concrete | 2 | miniextendr-api/src/rvalue.rs:436 |
| `String` | `` | concrete | 2 | miniextendr-api/src/rvalue.rs:437 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/rvalue.rs:440 |
| `Vec<Option<bool>>` | `` | concrete | 2 | miniextendr-api/src/rvalue.rs:479 |
| `Vec<Option<i32>>` | `` | concrete | 2 | miniextendr-api/src/rvalue.rs:480 |
| `Vec<f64>` | `` | concrete | 2 | miniextendr-api/src/rvalue.rs:481 |
| `Vec<crate::sexp_types::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/rvalue.rs:482 |
| `Vec<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/rvalue.rs:483 |
| `Vec<u8>` | `` | concrete | 2 | miniextendr-api/src/rvalue.rs:484 |
| `Vec<(Option<String>, RValue)>` | `` | concrete | 2 | miniextendr-api/src/rvalue.rs:485 |

## `Borrow` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1651 |

## `BorrowMut` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1658 |

## `Send` — 10 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ColPtr` | `` | concrete | 0 | miniextendr-api/src/dataframe_builder.rs:46 |
| `ExternalPtr<T>` | `<T>` | concrete | 0 | miniextendr-api/src/externalptr.rs:507 |
| `WorkerUnprotectGuard` | `` | concrete | 0 | miniextendr-api/src/gc_protect.rs:1609 |
| `RPreservedSexp` | `` | concrete | 0 | miniextendr-api/src/optionals/arrow_impl.rs:285 |
| `RTerm` | `` | concrete | 0 | miniextendr-api/src/progress.rs:129 |
| `TraitDispatchEntry` | `` | concrete | 0 | miniextendr-api/src/registry.rs:372 |
| `AltrepRegistration` | `` | concrete | 0 | miniextendr-api/src/registry.rs:389 |
| `SEXP` | `` | concrete | 0 | miniextendr-api/src/sexp.rs:71 |
| `R_CallMethodDef` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1157 |
| `R_altrep_class_t` | `` | concrete | 0 | miniextendr-api/src/sys/altrep.rs:197 |

## `RefUnwindSafe` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RPreservedSexp` | `` | concrete | 0 | miniextendr-api/src/optionals/arrow_impl.rs:287 |

## `Sync` — 13 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ColPtr` | `` | concrete | 0 | miniextendr-api/src/dataframe_builder.rs:47 |
| `RPreservedSexp` | `` | concrete | 0 | miniextendr-api/src/optionals/arrow_impl.rs:286 |
| `RTerm` | `` | concrete | 0 | miniextendr-api/src/progress.rs:130 |
| `RWrapperEntry` | `` | concrete | 0 | miniextendr-api/src/registry.rs:236 |
| `MatchArgChoicesEntry` | `` | concrete | 0 | miniextendr-api/src/registry.rs:253 |
| `MatchArgParamDocEntry` | `` | concrete | 0 | miniextendr-api/src/registry.rs:270 |
| `ClassNameEntry` | `` | concrete | 0 | miniextendr-api/src/registry.rs:288 |
| `SidecarPropEntry` | `` | concrete | 0 | miniextendr-api/src/registry.rs:351 |
| `TraitDispatchEntry` | `` | concrete | 0 | miniextendr-api/src/registry.rs:371 |
| `AltrepRegistration` | `` | concrete | 0 | miniextendr-api/src/registry.rs:388 |
| `SEXP` | `` | concrete | 0 | miniextendr-api/src/sexp.rs:72 |
| `R_CallMethodDef` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1156 |
| `R_altrep_class_t` | `` | concrete | 0 | miniextendr-api/src/sys/altrep.rs:198 |

## `TypedExternal` — 181 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `()` | `` | concrete | 3 | miniextendr-api/src/externalptr.rs:330 |
| `std::borrow::Cow<'static, [T]>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:101 |
| `std::rc::Rc<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:107 |
| `std::sync::Arc<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:108 |
| `std::cell::Cell<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:109 |
| `std::cell::RefCell<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:110 |
| `std::cell::UnsafeCell<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:111 |
| `std::sync::Mutex<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:112 |
| `std::sync::RwLock<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:113 |
| `std::sync::OnceLock<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:114 |
| `std::pin::Pin<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:115 |
| `std::mem::ManuallyDrop<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:119 |
| `std::mem::MaybeUninit<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:124 |
| `std::marker::PhantomData<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:125 |
| `Option<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:130 |
| `Result<T, E>` | `<T, E>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:131 |
| `std::ops::Range<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:136 |
| `std::ops::RangeInclusive<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:137 |
| `std::ops::RangeFrom<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:138 |
| `std::ops::RangeTo<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:139 |
| `std::ops::RangeToInclusive<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:140 |
| `std::ops::RangeFull` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:141 |
| `std::fs::File` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:146 |
| `std::io::BufReader<R>` | `<R>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:147 |
| `std::io::BufWriter<W>` | `<W>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:148 |
| `std::io::Cursor<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:149 |
| `std::time::Duration` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:154 |
| `std::time::Instant` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:155 |
| `std::time::SystemTime` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:156 |
| `std::net::TcpStream` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:161 |
| `std::net::TcpListener` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:162 |
| `std::net::UdpSocket` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:163 |
| `std::net::IpAddr` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:164 |
| `std::net::Ipv4Addr` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:165 |
| `std::net::Ipv6Addr` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:166 |
| `std::net::SocketAddr` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:167 |
| `std::net::SocketAddrV4` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:168 |
| `std::net::SocketAddrV6` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:169 |
| `std::thread::Thread` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:174 |
| `std::thread::JoinHandle<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:175 |
| `std::sync::mpsc::Sender<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:176 |
| `std::sync::mpsc::SyncSender<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:177 |
| `std::sync::mpsc::Receiver<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:178 |
| `std::sync::Barrier` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:179 |
| `std::sync::BarrierWaitResult` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:180 |
| `std::sync::atomic::AtomicBool` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:185 |
| `std::sync::atomic::AtomicI8` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:186 |
| `std::sync::atomic::AtomicI16` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:187 |
| `std::sync::atomic::AtomicI32` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:188 |
| `std::sync::atomic::AtomicI64` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:189 |
| `std::sync::atomic::AtomicIsize` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:190 |
| `std::sync::atomic::AtomicU8` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:191 |
| `std::sync::atomic::AtomicU16` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:192 |
| `std::sync::atomic::AtomicU32` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:193 |
| `std::sync::atomic::AtomicU64` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:194 |
| `std::sync::atomic::AtomicUsize` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:195 |
| `std::num::NonZeroI8` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:201 |
| `std::num::NonZeroI16` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:202 |
| `std::num::NonZeroI32` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:203 |
| `std::num::NonZeroI64` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:204 |
| `std::num::NonZeroI128` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:205 |
| `std::num::NonZeroIsize` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:206 |
| `std::num::NonZeroU8` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:207 |
| `std::num::NonZeroU16` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:208 |
| `std::num::NonZeroU32` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:209 |
| `std::num::NonZeroU64` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:210 |
| `std::num::NonZeroU128` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:211 |
| `std::num::NonZeroUsize` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:212 |
| `std::num::Wrapping<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:213 |
| `std::num::Saturating<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:214 |
| `(A,)` | `<A>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:219 |
| `(A, B)` | `<A, B>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:220 |
| `(A, B, C)` | `<A, B, C>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:221 |
| `(A, B, C, D)` | `<A, B, C, D>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:222 |
| `(A, B, C, D, E)` | `<A, B, C, D, E>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:223 |
| `(A, B, C, D, E, F)` | `<A, B, C, D, E, F>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:224 |
| `(A, B, C, D, E, F, G)` | `<A, B, C, D, E, F, G>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:225 |
| `(A, B, C, D, E, F, G, H)` | `<A, B, C, D, E, F, G, H>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:226 |
| `(A, B, C, D, E, F, G, H, I)` | `<A, B, C, D, E, F, G, H, I>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:227 |
| `(A, B, C, D, E, F, G, H, I, J)` | `<A, B, C, D, E, F, G, H, I, J>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:228 |
| `(A, B, C, D, E, F, G, H, I, J, K)` | `<A, B, C, D, E, F, G, H, I, J, K>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:229 |
| `(A, B, C, D, E, F, G, H, I, J, K, L)` | `<A, B, C, D, E, F, G, H, I, J, K, L>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:230 |
| `[T; N]` | `<T, N>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:235 |
| `&'static [T]` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:255 |
| `&'static mut [T]` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:261 |
| `bool` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:48 |
| `char` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:49 |
| `i8` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:50 |
| `i16` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:51 |
| `i32` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:52 |
| `i64` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:53 |
| `i128` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:54 |
| `isize` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:55 |
| `u8` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:56 |
| `u16` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:57 |
| `u32` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:58 |
| `u64` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:59 |
| `u128` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:60 |
| `usize` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:61 |
| `f32` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:62 |
| `f64` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:63 |
| `String` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:68 |
| `std::ffi::CString` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:69 |
| `std::ffi::OsString` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:70 |
| `std::path::PathBuf` | `` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:71 |
| `Vec<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:76 |
| `std::collections::VecDeque<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:77 |
| `std::collections::LinkedList<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:78 |
| `std::collections::BinaryHeap<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:79 |
| `std::collections::HashMap<K, V>` | `<K, V>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:80 |
| `std::collections::BTreeMap<K, V>` | `<K, V>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:81 |
| `std::collections::HashSet<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:82 |
| `std::collections::BTreeSet<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:83 |
| `Box<T>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:88 |
| `Box<[T]>` | `<T>` | concrete | 3 | miniextendr-api/src/externalptr_std.rs:94 |
| `Float64Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1565 |
| `Int32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1571 |
| `UInt8Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1577 |
| `BooleanArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1583 |
| `StringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1589 |
| `RSessionContext` | `` | concrete | 3 | miniextendr-api/src/optionals/datafusion_impl.rs:220 |
| `RDataFrame` | `` | concrete | 3 | miniextendr-api/src/optionals/datafusion_impl.rs:395 |
| `JiffTimestampVec` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffZonedVec` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `DVector<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:385 |
| `DVector<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:386 |
| `DVector<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:387 |
| `DMatrix<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:390 |
| `DMatrix<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:391 |
| `DMatrix<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:392 |
| `SVector<f64, 2>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:396 |
| `SVector<f64, 3>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:397 |
| `SVector<f64, 4>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:398 |
| `SVector<i32, 2>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:399 |
| `SVector<i32, 3>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:400 |
| `SVector<i32, 4>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:401 |
| `SMatrix<f64, 2, 2>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:405 |
| `SMatrix<f64, 3, 3>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:406 |
| `SMatrix<f64, 4, 4>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:407 |
| `SMatrix<i32, 2, 2>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:408 |
| `SMatrix<i32, 3, 3>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:409 |
| `SMatrix<i32, 4, 4>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:410 |
| `Array1<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1895 |
| `Array1<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1896 |
| `Array1<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1897 |
| `Array1<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1898 |
| `Array1<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1899 |
| `Array2<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1902 |
| `Array2<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1903 |
| `Array2<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1904 |
| `Array2<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1905 |
| `Array2<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1906 |
| `Array3<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1909 |
| `Array3<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1910 |
| `Array3<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1911 |
| `Array3<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1912 |
| `Array3<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1913 |
| `Array4<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1916 |
| `Array4<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1917 |
| `Array4<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1918 |
| `Array4<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1919 |
| `Array4<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1920 |
| `Array5<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1923 |
| `Array5<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1924 |
| `Array5<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1925 |
| `Array5<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1926 |
| `Array5<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1927 |
| `Array6<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1930 |
| `Array6<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1931 |
| `Array6<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1932 |
| `Array6<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1933 |
| `Array6<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1934 |
| `ArrayD<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1937 |
| `ArrayD<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1938 |
| `ArrayD<u8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1939 |
| `ArrayD<crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1940 |
| `ArrayD<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1941 |
| `ArcArray1<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1944 |
| `ArcArray1<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1945 |
| `ArcArray2<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1946 |
| `ArcArray2<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:1947 |

## `IntoRAs` — 135 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `&[String]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:1004 |
| `Vec<&str>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:1011 |
| `&[&str]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:1018 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:1026 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:1048 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:1055 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:1062 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:356 |
| `i8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:364 |
| `i16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:365 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:366 |
| `u16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:367 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:370 |
| `isize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:371 |
| `u32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:372 |
| `u64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:373 |
| `usize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:374 |
| `f32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:377 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:378 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:381 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:392 |
| `f32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:406 |
| `i8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:420 |
| `i16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:427 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:434 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:450 |
| `u16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:457 |
| `u32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:464 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:472 |
| `u64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:473 |
| `isize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:474 |
| `usize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:475 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:478 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:489 |
| `i8` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:497 |
| `i16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:498 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:499 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:500 |
| `isize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:501 |
| `u16` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:502 |
| `u32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:503 |
| `u64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:504 |
| `usize` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:505 |
| `f32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:506 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:507 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:512 |
| `crate::RLogical` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:519 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:527 |
| `String` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:546 |
| `&str` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:553 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:561 |
| `f32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:579 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:586 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:593 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:600 |
| `crate::RLogical` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:607 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:681 |
| `&[i32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:688 |
| `Vec<i8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:695 |
| `&[i8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:695 |
| `&[i16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:696 |
| `Vec<i16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:696 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:697 |
| `&[u8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:697 |
| `Vec<u16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:698 |
| `&[u16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:698 |
| `&[i64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:699 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:699 |
| `Vec<isize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:700 |
| `&[isize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:700 |
| `Vec<u32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:701 |
| `&[u32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:701 |
| `&[u64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:702 |
| `Vec<u64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:702 |
| `Vec<usize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:703 |
| `&[usize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:703 |
| `Vec<f32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:704 |
| `&[f32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:704 |
| `&[f64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:705 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:705 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:708 |
| `&[bool]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:715 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:726 |
| `&[f64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:752 |
| `Vec<f32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:779 |
| `&[f32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:808 |
| `&[i8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:856 |
| `Vec<i8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:856 |
| `Vec<i16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:857 |
| `&[i16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:857 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:858 |
| `&[u8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:858 |
| `&[u16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:859 |
| `Vec<u16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:859 |
| `Vec<u32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:860 |
| `&[u32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:860 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:865 |
| `&[i32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:894 |
| `&[i64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:924 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:924 |
| `Vec<u64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:925 |
| `&[u64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:925 |
| `Vec<isize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:926 |
| `&[isize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:926 |
| `&[usize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:927 |
| `Vec<usize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:927 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:930 |
| `&[bool]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:940 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:951 |
| `&[u8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:958 |
| `&[i8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:965 |
| `Vec<i8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:965 |
| `Vec<i16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:966 |
| `&[i16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:966 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:967 |
| `&[i32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:967 |
| `&[i64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:968 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:968 |
| `Vec<isize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:969 |
| `&[isize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:969 |
| `Vec<u16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:970 |
| `&[u16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:970 |
| `Vec<u32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:971 |
| `&[u32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:971 |
| `&[u64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:972 |
| `Vec<u64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:972 |
| `Vec<usize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:973 |
| `&[usize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:973 |
| `Vec<f32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:974 |
| `&[f32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:974 |
| `&[f64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:975 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:975 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:980 |
| `&[bool]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:987 |
| `Vec<String>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:997 |

### `IntoRAs` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/into_r_as.rs:971** (2 impls): `Vec<u32>`, `&[u32]`
- **miniextendr-api/src/into_r_as.rs:856** (2 impls): `&[i8]`, `Vec<i8>`
- **miniextendr-api/src/into_r_as.rs:972** (2 impls): `&[u64]`, `Vec<u64>`
- **miniextendr-api/src/into_r_as.rs:858** (2 impls): `Vec<u8>`, `&[u8]`
- **miniextendr-api/src/into_r_as.rs:974** (2 impls): `Vec<f32>`, `&[f32]`
- **miniextendr-api/src/into_r_as.rs:859** (2 impls): `&[u16]`, `Vec<u16>`
- **miniextendr-api/src/into_r_as.rs:695** (2 impls): `Vec<i8>`, `&[i8]`
- **miniextendr-api/src/into_r_as.rs:975** (2 impls): `&[f64]`, `Vec<f64>`
- **miniextendr-api/src/into_r_as.rs:696** (2 impls): `&[i16]`, `Vec<i16>`
- **miniextendr-api/src/into_r_as.rs:924** (2 impls): `&[i64]`, `Vec<i64>`
- **miniextendr-api/src/into_r_as.rs:698** (2 impls): `Vec<u16>`, `&[u16]`
- **miniextendr-api/src/into_r_as.rs:926** (2 impls): `Vec<isize>`, `&[isize]`
- **miniextendr-api/src/into_r_as.rs:699** (2 impls): `&[i64]`, `Vec<i64>`
- **miniextendr-api/src/into_r_as.rs:927** (2 impls): `&[usize]`, `Vec<usize>`
- **miniextendr-api/src/into_r_as.rs:701** (2 impls): `Vec<u32>`, `&[u32]`
- **miniextendr-api/src/into_r_as.rs:702** (2 impls): `&[u64]`, `Vec<u64>`
- **miniextendr-api/src/into_r_as.rs:965** (2 impls): `&[i8]`, `Vec<i8>`
- **miniextendr-api/src/into_r_as.rs:704** (2 impls): `Vec<f32>`, `&[f32]`
- **miniextendr-api/src/into_r_as.rs:967** (2 impls): `Vec<i32>`, `&[i32]`
- **miniextendr-api/src/into_r_as.rs:705** (2 impls): `&[f64]`, `Vec<f64>`
- **miniextendr-api/src/into_r_as.rs:968** (2 impls): `&[i64]`, `Vec<i64>`
- **miniextendr-api/src/into_r_as.rs:970** (2 impls): `Vec<u16>`, `&[u16]`
- **miniextendr-api/src/into_r_as.rs:857** (2 impls): `Vec<i16>`, `&[i16]`
- **miniextendr-api/src/into_r_as.rs:973** (2 impls): `Vec<usize>`, `&[usize]`
- **miniextendr-api/src/into_r_as.rs:860** (2 impls): `Vec<u32>`, `&[u32]`
- **miniextendr-api/src/into_r_as.rs:697** (2 impls): `Vec<u8>`, `&[u8]`
- **miniextendr-api/src/into_r_as.rs:925** (2 impls): `Vec<u64>`, `&[u64]`
- **miniextendr-api/src/into_r_as.rs:700** (2 impls): `Vec<isize>`, `&[isize]`
- **miniextendr-api/src/into_r_as.rs:703** (2 impls): `Vec<usize>`, `&[usize]`
- **miniextendr-api/src/into_r_as.rs:966** (2 impls): `Vec<i16>`, `&[i16]`
- **miniextendr-api/src/into_r_as.rs:969** (2 impls): `Vec<isize>`, `&[isize]`

## `RDebug` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:63 |

## `Debug` — 108 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `mx_tag` | `` | concrete | 1 | miniextendr-api/src/abi.rs:82 |
| `RAllocator` | `` | concrete | 1 | miniextendr-api/src/allocator.rs:143 |
| `RBase` | `` | concrete | 1 | miniextendr-api/src/altrep.rs:51 |
| `Sortedness` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:131 |
| `Logical` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:54 |
| `AltrepSexp` | `` | concrete | 1 | miniextendr-api/src/altrep_sexp.rs:300 |
| `AltrepGuard` | `` | concrete | 1 | miniextendr-api/src/altrep_traits.rs:60 |
| `LogicalCoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:481 |
| `CoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:85 |
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:919 |
| `AsRError<E>` | `<E>` | concrete | 1 | miniextendr-api/src/condition.rs:895 |
| `RStdin` | `` | concrete | 1 | miniextendr-api/src/connection.rs:1197 |
| `RStdout` | `` | concrete | 1 | miniextendr-api/src/connection.rs:1215 |
| `RStderr` | `` | concrete | 1 | miniextendr-api/src/connection.rs:1233 |
| `RNullConnection` | `` | concrete | 1 | miniextendr-api/src/connection.rs:1361 |
| `ConnectionCapabilities` | `` | concrete | 1 | miniextendr-api/src/connection.rs:192 |
| `AsList<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:100 |
| `AsFromStr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:1022 |
| `AsFromStrVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:1064 |
| `AsExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:362 |
| `AsRNative<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:414 |
| `AsDataFrame<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:484 |
| `AsVctrs<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:530 |
| `AsNamedList<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:603 |
| `AsNamedVector<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:716 |
| `AsDisplay<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:961 |
| `AsDisplayVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:988 |
| `DataFrame` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:1546 |
| `DataFrameError` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:53 |
| `BuiltDataFrame` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:941 |
| `GroupKey` | `` | concrete | 1 | miniextendr-api/src/dataframe/group.rs:81 |
| `Dots` | `` | concrete | 1 | miniextendr-api/src/dots.rs:42 |
| `REncodingInfo` | `` | concrete | 1 | miniextendr-api/src/encoding.rs:23 |
| `TypeMismatchError` | `` | concrete | 1 | miniextendr-api/src/externalptr.rs:1534 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1689 |
| `RSidecar` | `` | concrete | 1 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `FactorVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:467 |
| `FactorOptionVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:525 |
| `GuardMode` | `` | concrete | 1 | miniextendr-api/src/ffi_guard.rs:48 |
| `SexpTypeError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:182 |
| `SexpLengthError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:203 |
| `SexpNaError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:224 |
| `SexpError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:239 |
| `Protected<'a, T>` | `<'a, T>` | concrete | 1 | miniextendr-api/src/gc_protect.rs:1274 |
| `Altrep<T>` | `<T>` | concrete | 1 | miniextendr-api/src/into_r/altrep.rs:88 |
| `NullOnErr` | `` | concrete | 1 | miniextendr-api/src/into_r/result.rs:133 |
| `CoerceErrorKind` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:298 |
| `StorageCoerceError` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:59 |
| `IntoRError` | `` | concrete | 1 | miniextendr-api/src/into_r_error.rs:14 |
| `DuplicateNameError` | `` | concrete | 1 | miniextendr-api/src/list.rs:1206 |
| `ListFromSexpError` | `` | concrete | 1 | miniextendr-api/src/list.rs:1221 |
| `List` | `` | concrete | 1 | miniextendr-api/src/list.rs:40 |
| `ListMut` | `` | concrete | 1 | miniextendr-api/src/list.rs:47 |
| `MatchArgError` | `` | concrete | 1 | miniextendr-api/src/match_arg.rs:56 |
| `Missing<T>` | `<T>` | concrete | 1 | miniextendr-api/src/missing.rs:98 |
| `NamedVector<M>` | `<M>` | concrete | 1 | miniextendr-api/src/named_vector.rs:191 |
| `RPrimitive<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:172 |
| `RStringArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:256 |
| `RFlags<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/bitflags_impl.rs:132 |
| `GlobOptions` | `` | concrete | 1 | miniextendr-api/src/optionals/globset_impl.rs:49 |
| `RRng` | `` | concrete | 1 | miniextendr-api/src/optionals/rand_impl.rs:109 |
| `CaptureGroups` | `` | concrete | 1 | miniextendr-api/src/optionals/regex_impl.rs:189 |
| `FactorHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:102 |
| `JsonOptions` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:125 |
| `NaHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:78 |
| `SpecialFloatHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:90 |
| `PanicSource` | `` | concrete | 1 | miniextendr-api/src/panic_telemetry.rs:48 |
| `RTerm` | `` | concrete | 1 | miniextendr-api/src/progress.rs:213 |
| `TermKind` | `` | concrete | 1 | miniextendr-api/src/progress.rs:67 |
| `ProtectKey` | `` | concrete | 1 | miniextendr-api/src/protect_pool.rs:61 |
| `RCoerceError` | `` | concrete | 1 | miniextendr-api/src/r_coerce.rs:101 |
| `RArray<T, NDIM>` | `<T, NDIM>` | concrete | 1 | miniextendr-api/src/rarray.rs:879 |
| `Raw<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:197 |
| `RawSlice<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:216 |
| `RawTagged<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:245 |
| `RawSliceTagged<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:265 |
| `RawError` | `` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:69 |
| `Entry` | `` | concrete | 1 | miniextendr-api/src/refcount_protect.rs:50 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:28 |
| `ColumnType` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:1296 |
| `SchemaMode` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:1362 |
| `TypeSpec` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:818 |
| `DispatchNames` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:937 |
| `RSerdeError` | `` | concrete | 1 | miniextendr-api/src/serde/error.rs:12 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:224 |
| `SEXPREC` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:25 |
| `SEXP` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:65 |
| `RLogical` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:162 |
| `Rcomplex` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:24 |
| `Rboolean` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:321 |
| `SEXPTYPE` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:35 |
| `StrVec<'a>` | `<'a>` | concrete | 1 | miniextendr-api/src/strvec.rs:25 |
| `ProtectedStrVec` | `` | concrete | 1 | miniextendr-api/src/strvec.rs:759 |
| `DllInfo` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1089 |
| `R_CMethodDef` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1117 |
| `R_CallMethodDef` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1142 |
| `RNGtype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1493 |
| `N01type` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1517 |
| `Sampletype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1537 |
| `ParseStatus` | `` | concrete | 1 | miniextendr-api/src/sys.rs:971 |
| `RTxtProgressBar` | `` | concrete | 1 | miniextendr-api/src/txt_progress_bar.rs:56 |
| `TypeSpec` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:119 |
| `TypedListError` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:191 |
| `TypedList` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:277 |
| `TypedListSpec` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:47 |
| `TypedEntry` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:75 |
| `VctrsBuildError` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:32 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:525 |

## `RClone` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:386 |

## `Clone` — 100 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `mx_tag` | `` | concrete | 1 | miniextendr-api/src/abi.rs:82 |
| `Header` | `` | concrete | 1 | miniextendr-api/src/allocator.rs:115 |
| `RBase` | `` | concrete | 1 | miniextendr-api/src/altrep.rs:51 |
| `Sortedness` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:131 |
| `Logical` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:54 |
| `AltrepGuard` | `` | concrete | 1 | miniextendr-api/src/altrep_traits.rs:60 |
| `LogicalCoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:481 |
| `CoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:85 |
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:919 |
| `RStdin` | `` | concrete | 1 | miniextendr-api/src/connection.rs:1197 |
| `RStdout` | `` | concrete | 1 | miniextendr-api/src/connection.rs:1215 |
| `RStderr` | `` | concrete | 1 | miniextendr-api/src/connection.rs:1233 |
| `ConnectionCapabilities` | `` | concrete | 1 | miniextendr-api/src/connection.rs:192 |
| `AsList<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:100 |
| `AsFromStr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:1022 |
| `AsFromStrVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:1064 |
| `AsExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:362 |
| `AsRNative<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:414 |
| `AsDataFrame<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:484 |
| `AsVctrs<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:530 |
| `AsNamedList<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:603 |
| `AsNamedVector<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:716 |
| `AsDisplay<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:961 |
| `AsDisplayVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/convert.rs:988 |
| `DataFrame` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:208 |
| `DataFrameError` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:53 |
| `GroupKey` | `` | concrete | 1 | miniextendr-api/src/dataframe/group.rs:81 |
| `ColPtr` | `` | concrete | 1 | miniextendr-api/src/dataframe_builder.rs:43 |
| `REncodingInfo` | `` | concrete | 1 | miniextendr-api/src/encoding.rs:23 |
| `TypeMismatchError` | `` | concrete | 1 | miniextendr-api/src/externalptr.rs:1534 |
| `ExternalPtr<T>` | `<T>` | concrete | 2 | miniextendr-api/src/externalptr.rs:1665 |
| `RSidecar` | `` | concrete | 1 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `FactorVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:467 |
| `FactorOptionVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:525 |
| `GuardMode` | `` | concrete | 1 | miniextendr-api/src/ffi_guard.rs:48 |
| `SexpTypeError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:182 |
| `SexpLengthError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:203 |
| `SexpNaError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:224 |
| `SexpError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:239 |
| `Root<'a>` | `<'a>` | concrete | 1 | miniextendr-api/src/gc_protect.rs:1041 |
| `TlsRoot` | `` | concrete | 1 | miniextendr-api/src/gc_protect/tls.rs:202 |
| `Altrep<T>` | `<T>` | concrete | 1 | miniextendr-api/src/into_r/altrep.rs:88 |
| `NullOnErr` | `` | concrete | 1 | miniextendr-api/src/into_r/result.rs:133 |
| `StorageCoerceError` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:59 |
| `IntoRError` | `` | concrete | 1 | miniextendr-api/src/into_r_error.rs:14 |
| `DuplicateNameError` | `` | concrete | 1 | miniextendr-api/src/list.rs:1206 |
| `ListFromSexpError` | `` | concrete | 1 | miniextendr-api/src/list.rs:1221 |
| `List` | `` | concrete | 1 | miniextendr-api/src/list.rs:40 |
| `MatchArgError` | `` | concrete | 1 | miniextendr-api/src/match_arg.rs:56 |
| `Missing<T>` | `<T>` | concrete | 1 | miniextendr-api/src/missing.rs:98 |
| `NamedVector<M>` | `<M>` | concrete | 1 | miniextendr-api/src/named_vector.rs:191 |
| `RFlags<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/bitflags_impl.rs:99 |
| `GlobOptions` | `` | concrete | 1 | miniextendr-api/src/optionals/globset_impl.rs:49 |
| `CaptureGroups` | `` | concrete | 1 | miniextendr-api/src/optionals/regex_impl.rs:189 |
| `FactorHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:102 |
| `JsonOptions` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:125 |
| `NaHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:78 |
| `SpecialFloatHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:90 |
| `PanicSource` | `` | concrete | 1 | miniextendr-api/src/panic_telemetry.rs:48 |
| `TermKind` | `` | concrete | 1 | miniextendr-api/src/progress.rs:67 |
| `ProtectKey` | `` | concrete | 1 | miniextendr-api/src/protect_pool.rs:61 |
| `RCoerceError` | `` | concrete | 1 | miniextendr-api/src/r_coerce.rs:101 |
| `RArray<T, NDIM>` | `<T, NDIM>` | concrete | 1 | miniextendr-api/src/rarray.rs:120 |
| `RawHeader` | `` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:115 |
| `Raw<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:197 |
| `RawSlice<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:216 |
| `RawTagged<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:245 |
| `RawSliceTagged<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:265 |
| `RawError` | `` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:69 |
| `Entry` | `` | concrete | 1 | miniextendr-api/src/refcount_protect.rs:50 |
| `RWrapperPriority` | `` | concrete | 1 | miniextendr-api/src/registry.rs:209 |
| `RValue` | `` | concrete | 1 | miniextendr-api/src/rvalue.rs:28 |
| `ColumnType` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:1296 |
| `SchemaMode` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:1362 |
| `TypeSpec` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:818 |
| `DispatchNames` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:937 |
| `EnumReadConfig` | `` | concrete | 1 | miniextendr-api/src/serde/dataframe_de.rs:498 |
| `RSerdeError` | `` | concrete | 1 | miniextendr-api/src/serde/error.rs:12 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:224 |
| `SEXP` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:65 |
| `RLogical` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:162 |
| `Rcomplex` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:24 |
| `Rboolean` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:321 |
| `SEXPTYPE` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:35 |
| `cetype_t` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:353 |
| `StrVec<'a>` | `<'a>` | concrete | 1 | miniextendr-api/src/strvec.rs:25 |
| `R_CMethodDef` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1117 |
| `R_CallMethodDef` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1142 |
| `RNGtype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1493 |
| `N01type` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1517 |
| `Sampletype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1537 |
| `ParseStatus` | `` | concrete | 1 | miniextendr-api/src/sys.rs:971 |
| `R_altrep_class_t` | `` | concrete | 1 | miniextendr-api/src/sys/altrep.rs:187 |
| `TypeSpec` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:119 |
| `TypedListError` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:191 |
| `TypedList` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:277 |
| `TypedListSpec` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:47 |
| `TypedEntry` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:75 |
| `VctrsBuildError` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:32 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:525 |

## `TryCoerce` — 93 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T, R> +1wc` | concrete | 2 | miniextendr-api/src/coerce.rs:112 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:318 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:328 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:338 |
| `i8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:369 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:370 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:371 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:372 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:373 |
| `u8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:374 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:375 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:376 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:377 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:378 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:381 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:391 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:401 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:412 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:423 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:434 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:445 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:456 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:467 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:500 |
| `crate::Rboolean` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:515 |
| `crate::RLogical` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:524 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `i8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `i8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `u8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:566 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:587 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:608 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:632 |
| `f32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:653 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:673 |
| `f32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:694 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:706 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:746 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:768 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:789 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:811 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:852 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:869 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:882 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:890 |
| `BigInt` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:118 |
| `BigInt` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:129 |
| `BigUint` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:140 |
| `BigUint` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:151 |
| `BigInt` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:162 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:56 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/optionals/num_bigint_impl.rs:86 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:40 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/optionals/ordered_float_impl.rs:81 |
| `Decimal` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:104 |
| `Decimal` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:119 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:74 |
| `Decimal` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:91 |

### `TryCoerce` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/coerce.rs:541** (9 impls): `i8`, `i16`, `i32`, `i64`, `u16`, `u32`, `u64`, `usize`, `isize`
- **miniextendr-api/src/coerce.rs:561** (9 impls): `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `usize`, `isize`
- **miniextendr-api/src/coerce.rs:551** (8 impls): `i8`, `i16`, `i32`, `i64`, `u32`, `u64`, `usize`, `isize`
- **miniextendr-api/src/coerce.rs:556** (7 impls): `i32`, `i64`, `u16`, `u32`, `u64`, `usize`, `isize`
- **miniextendr-api/src/coerce.rs:536** (5 impls): `u32`, `u64`, `usize`, `i64`, `isize`

## `AltrepLen` — 64 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[i32; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1001 |
| `[f64; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1026 |
| `[bool; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1051 |
| `[u8; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1063 |
| `[String; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1084 |
| `Vec<crate::Rcomplex>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1095 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1102 |
| `[crate::Rcomplex; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1109 |
| `Cow<'static, [i32]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1186 |
| `Cow<'static, [f64]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1191 |
| `Cow<'static, [u8]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1196 |
| `Cow<'static, [crate::Rcomplex]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1201 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:495 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:499 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:503 |
| `Vec<String>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:507 |
| `Vec<Option<String>>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:519 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:531 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:547 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:563 |
| `Box<[i32]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:580 |
| `Box<[f64]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:584 |
| `Box<[u8]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:588 |
| `Box<[bool]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:592 |
| `Box<[String]>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:595 |
| `std::ops::Range<i32>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:663 |
| `std::ops::Range<i64>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:760 |
| `std::ops::Range<f64>` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:901 |
| `&[i32]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:955 |
| `&[f64]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:958 |
| `&[u8]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:961 |
| `&[bool]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:964 |
| `&[String]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:967 |
| `&[&str]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:975 |
| `IterRealCoerceData<I, T>` | `<I, T> +2wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:151 |
| `IterIntFromBoolData<I>` | `<I> +1wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:218 |
| `IterStringData<I>` | `<I> +1wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:311 |
| `IterListData<I>` | `<I> +1wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:392 |
| `IterComplexData<I>` | `<I> +1wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:458 |
| `IterIntCoerceData<I, T>` | `<I, T> +2wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:70 |
| `SparseIterIntData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:212 |
| `SparseIterRealData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:261 |
| `SparseIterLogicalData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:304 |
| `SparseIterRawData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:346 |
| `SparseIterComplexData<I>` | `<I> +1wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/sparse.rs:398 |
| `IterIntData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/state.rs:260 |
| `IterRealData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/state.rs:310 |
| `IterLogicalData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/state.rs:358 |
| `IterRawData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/state.rs:405 |
| `WindowedIterIntData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/windowed.rs:259 |
| `WindowedIterRealData<I>` | `<I>` | concrete | 1 | miniextendr-api/src/altrep_data/iter/windowed.rs:308 |
| `StreamingIntData<F>` | `<F>` | concrete | 1 | miniextendr-api/src/altrep_data/stream.rs:181 |
| `StreamingRealData<F>` | `<F>` | concrete | 1 | miniextendr-api/src/altrep_data/stream.rs:84 |
| `Float64Array` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1599 |
| `Int32Array` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1605 |
| `UInt8Array` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1611 |
| `BooleanArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1617 |
| `StringArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1949 |
| `JiffTimestampVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:890 |
| `JiffZonedVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:953 |
| `DVector<f64>` | `` | concrete | 1 | miniextendr-api/src/optionals/nalgebra_impl.rs:1613 |
| `DVector<i32>` | `` | concrete | 1 | miniextendr-api/src/optionals/nalgebra_impl.rs:1619 |
| `Array1<f64>` | `` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:3772 |
| `Array1<i32>` | `` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:3778 |

## `RCopy` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:480 |

## `Copy` — 58 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `mx_tag` | `` | concrete | 0 | miniextendr-api/src/abi.rs:82 |
| `Header` | `` | concrete | 0 | miniextendr-api/src/allocator.rs:115 |
| `RBase` | `` | concrete | 0 | miniextendr-api/src/altrep.rs:51 |
| `Sortedness` | `` | concrete | 0 | miniextendr-api/src/altrep_data/core.rs:131 |
| `Logical` | `` | concrete | 0 | miniextendr-api/src/altrep_data/core.rs:54 |
| `AltrepGuard` | `` | concrete | 0 | miniextendr-api/src/altrep_traits.rs:60 |
| `LogicalCoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:481 |
| `CoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:85 |
| `Coerced<T, R>` | `<T, R>` | concrete | 0 | miniextendr-api/src/coerce.rs:919 |
| `RStdin` | `` | concrete | 0 | miniextendr-api/src/connection.rs:1197 |
| `RStdout` | `` | concrete | 0 | miniextendr-api/src/connection.rs:1215 |
| `RStderr` | `` | concrete | 0 | miniextendr-api/src/connection.rs:1233 |
| `AsList<T>` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:100 |
| `AsExternalPtr<T>` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:362 |
| `AsRNative<T>` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:414 |
| `AsDisplay<T>` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:961 |
| `DataFrame` | `` | concrete | 0 | miniextendr-api/src/dataframe.rs:208 |
| `ColPtr` | `` | concrete | 0 | miniextendr-api/src/dataframe_builder.rs:43 |
| `RSidecar` | `` | concrete | 0 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `GuardMode` | `` | concrete | 0 | miniextendr-api/src/ffi_guard.rs:48 |
| `SexpTypeError` | `` | concrete | 0 | miniextendr-api/src/from_r.rs:182 |
| `SexpLengthError` | `` | concrete | 0 | miniextendr-api/src/from_r.rs:203 |
| `SexpNaError` | `` | concrete | 0 | miniextendr-api/src/from_r.rs:224 |
| `Root<'a>` | `<'a>` | concrete | 0 | miniextendr-api/src/gc_protect.rs:1041 |
| `TlsRoot` | `` | concrete | 0 | miniextendr-api/src/gc_protect/tls.rs:202 |
| `Altrep<T>` | `<T>` | concrete | 0 | miniextendr-api/src/into_r/altrep.rs:88 |
| `NullOnErr` | `` | concrete | 0 | miniextendr-api/src/into_r/result.rs:133 |
| `List` | `` | concrete | 0 | miniextendr-api/src/list.rs:40 |
| `Missing<T>` | `<T>` | concrete | 0 | miniextendr-api/src/missing.rs:98 |
| `RFlags<T>` | `<T>` | concrete | 0 | miniextendr-api/src/optionals/bitflags_impl.rs:99 |
| `GlobOptions` | `` | concrete | 0 | miniextendr-api/src/optionals/globset_impl.rs:49 |
| `PanicSource` | `` | concrete | 0 | miniextendr-api/src/panic_telemetry.rs:48 |
| `TermKind` | `` | concrete | 0 | miniextendr-api/src/progress.rs:67 |
| `ProtectKey` | `` | concrete | 0 | miniextendr-api/src/protect_pool.rs:61 |
| `RArray<T, NDIM>` | `<T, NDIM>` | concrete | 0 | miniextendr-api/src/rarray.rs:120 |
| `RawHeader` | `` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:115 |
| `Raw<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:197 |
| `RawTagged<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:245 |
| `Entry` | `` | concrete | 0 | miniextendr-api/src/refcount_protect.rs:50 |
| `RWrapperPriority` | `` | concrete | 0 | miniextendr-api/src/registry.rs:209 |
| `ColumnType` | `` | concrete | 0 | miniextendr-api/src/serde/columnar.rs:1296 |
| `SchemaMode` | `` | concrete | 0 | miniextendr-api/src/serde/columnar.rs:1362 |
| `AsSerialize<T>` | `<T>` | concrete | 0 | miniextendr-api/src/serde/traits.rs:224 |
| `SEXP` | `` | concrete | 0 | miniextendr-api/src/sexp.rs:65 |
| `RLogical` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:162 |
| `Rcomplex` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:24 |
| `Rboolean` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:321 |
| `SEXPTYPE` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:35 |
| `cetype_t` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:353 |
| `StrVec<'a>` | `<'a>` | concrete | 0 | miniextendr-api/src/strvec.rs:25 |
| `R_CMethodDef` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1117 |
| `R_CallMethodDef` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1142 |
| `RNGtype` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1493 |
| `N01type` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1517 |
| `Sampletype` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1537 |
| `ParseStatus` | `` | concrete | 0 | miniextendr-api/src/sys.rs:971 |
| `R_altrep_class_t` | `` | concrete | 0 | miniextendr-api/src/sys/altrep.rs:187 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-api/src/vctrs.rs:525 |

## `Coerce` — 53 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `tinyvec::TinyVec<[T; N]>` | `<T, R, N> +3wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1016 |
| `tinyvec::ArrayVec<[T; N]>` | `<T, R, N> +3wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1031 |
| `(A, B)` | `<A, B, RA, RB> +2wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1061 |
| `(A, B, C)` | `<A, B, C, RA, RB, RC> +3wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1062 |
| `(A, B, C, D)` | `<A, B, C, D, RA, RB, RC, RD> +4wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1063 |
| `(A, B, C, D, E)` | `<A, B, C, D, E, RA, RB, RC, RD, RE> +5wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1064 |
| `(A, B, C, D, E, F)` | `<A, B, C, D, E, F, RA, RB, RC, RD, RE, RF> +6wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1065 |
| `(A, B, C, D, E, F, G)` | `<A, B, C, D, E, F, G, RA, RB, RC, RD, RE, RF, RG> +7wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1070 |
| `(A, B, C, D, E, F, G, H)` | `<A, B, C, D, E, F, G, H, RA, RB, RC, RD, RE, RF, RG, RH> +8wc` | concrete | 1 | miniextendr-api/src/coerce.rs:1075 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:142 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:143 |
| `crate::Rboolean` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:144 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:145 |
| `crate::Rcomplex` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:146 |
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/coerce.rs:191 |
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/coerce.rs:202 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:212 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:212 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:212 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:212 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:212 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:214 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:224 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:235 |
| `bool` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:242 |
| `crate::Rboolean` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:249 |
| `Option<f64>` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:260 |
| `Option<i32>` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:268 |
| `Option<bool>` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:276 |
| `Option<crate::Rboolean>` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:288 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:302 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:310 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
| `i8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
| `u16` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:663 |
| `&[T]` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:974 |
| `Vec<T>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:982 |
| `Box<[T]>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:990 |
| `std::collections::VecDeque<T>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:998 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/optionals/num_bigint_impl.rs:24 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/optionals/num_bigint_impl.rs:32 |
| `u32` | `` | concrete | 1 | miniextendr-api/src/optionals/num_bigint_impl.rs:40 |
| `u64` | `` | concrete | 1 | miniextendr-api/src/optionals/num_bigint_impl.rs:48 |
| `OrderedFloat<f64>` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:100 |
| `OrderedFloat<f32>` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:108 |
| `f64` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:24 |
| `f32` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:32 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/optionals/ordered_float_impl.rs:73 |
| `i32` | `` | concrete | 1 | miniextendr-api/src/optionals/rust_decimal_impl.rs:58 |
| `i64` | `` | concrete | 1 | miniextendr-api/src/optionals/rust_decimal_impl.rs:66 |

### `Coerce` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/coerce.rs:212** (5 impls): `u8`, `u8`, `u8`, `u8`, `u8`
- **miniextendr-api/src/coerce.rs:546** (5 impls): `u8`, `u8`, `u8`, `i8`, `u16`

## `InferBase` — 45 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[i32; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_impl/arrays.rs:116 |
| `[f64; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_impl/arrays.rs:125 |
| `[u8; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_impl/arrays.rs:134 |
| `[crate::Rcomplex; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_impl/arrays.rs:142 |
| `[bool; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_impl/arrays.rs:182 |
| `[String; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_impl/arrays.rs:231 |
| `Vec<i32>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:339 |
| `Vec<f64>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:346 |
| `Vec<bool>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:353 |
| `Vec<u8>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:360 |
| `Vec<String>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:367 |
| `Vec<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:374 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:384 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:391 |
| `Vec<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:398 |
| `std::ops::Range<i32>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:407 |
| `std::ops::Range<i64>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:414 |
| `std::ops::Range<f64>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:421 |
| `Box<[i32]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:436 |
| `Box<[f64]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:443 |
| `Box<[bool]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:450 |
| `Box<[u8]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:457 |
| `Box<[String]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:464 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:471 |
| `std::borrow::Cow<'static, [i32]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:481 |
| `std::borrow::Cow<'static, [f64]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:488 |
| `std::borrow::Cow<'static, [u8]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:495 |
| `std::borrow::Cow<'static, [crate::Rcomplex]>` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/builtins.rs:502 |
| `&'static [i32]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:129 |
| `&'static [f64]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:231 |
| `&'static [bool]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:285 |
| `&'static [u8]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:346 |
| `&'static [String]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:386 |
| `&'static [&'static str]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:426 |
| `Float64Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1864 |
| `Int32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1865 |
| `UInt8Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1866 |
| `BooleanArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1867 |
| `StringArray` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1969 |
| `JiffTimestampVec` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffZonedVec` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `DVector<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1673 |
| `DVector<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1674 |
| `Array1<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3857 |
| `Array1<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3858 |

## `Altrep` — 45 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[i32; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_impl/arrays.rs:116 |
| `[f64; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_impl/arrays.rs:125 |
| `[u8; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_impl/arrays.rs:134 |
| `[crate::Rcomplex; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_impl/arrays.rs:142 |
| `[bool; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_impl/arrays.rs:152 |
| `[String; N]` | `<N>` | concrete | 2 | miniextendr-api/src/altrep_impl/arrays.rs:208 |
| `Vec<i32>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:339 |
| `Vec<f64>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:346 |
| `Vec<bool>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:353 |
| `Vec<u8>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:360 |
| `Vec<String>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:367 |
| `Vec<Option<String>>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:374 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:384 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:391 |
| `Vec<crate::Rcomplex>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:398 |
| `std::ops::Range<i32>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:407 |
| `std::ops::Range<i64>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:414 |
| `std::ops::Range<f64>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:421 |
| `Box<[i32]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:436 |
| `Box<[f64]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:443 |
| `Box<[bool]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:450 |
| `Box<[u8]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:457 |
| `Box<[String]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:464 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:471 |
| `std::borrow::Cow<'static, [i32]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:481 |
| `std::borrow::Cow<'static, [f64]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:488 |
| `std::borrow::Cow<'static, [u8]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:495 |
| `std::borrow::Cow<'static, [crate::Rcomplex]>` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/builtins.rs:502 |
| `&'static [f64]` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/static_slices.rs:132 |
| `&'static [i32]` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/static_slices.rs:23 |
| `&'static [bool]` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/static_slices.rs:234 |
| `&'static [u8]` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/static_slices.rs:288 |
| `&'static [String]` | `` | concrete | 2 | miniextendr-api/src/altrep_impl/static_slices.rs:349 |
| `&'static [&'static str]` | `` | concrete | 2 | miniextendr-api/src/altrep_impl/static_slices.rs:389 |
| `Float64Array` | `` | concrete | 6 | miniextendr-api/src/optionals/arrow_impl.rs:1864 |
| `Int32Array` | `` | concrete | 6 | miniextendr-api/src/optionals/arrow_impl.rs:1865 |
| `UInt8Array` | `` | concrete | 6 | miniextendr-api/src/optionals/arrow_impl.rs:1866 |
| `BooleanArray` | `` | concrete | 6 | miniextendr-api/src/optionals/arrow_impl.rs:1867 |
| `StringArray` | `` | concrete | 6 | miniextendr-api/src/optionals/arrow_impl.rs:1969 |
| `JiffTimestampVec` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffZonedVec` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `DVector<f64>` | `` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:1673 |
| `DVector<i32>` | `` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:1674 |
| `Array1<f64>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:3857 |
| `Array1<i32>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:3858 |

## `AltVec` — 45 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[i32; N]` | `<N>` | concrete | 2 | miniextendr-api/src/altrep_impl/arrays.rs:116 |
| `[f64; N]` | `<N>` | concrete | 2 | miniextendr-api/src/altrep_impl/arrays.rs:125 |
| `[u8; N]` | `<N>` | concrete | 2 | miniextendr-api/src/altrep_impl/arrays.rs:134 |
| `[crate::Rcomplex; N]` | `<N>` | concrete | 2 | miniextendr-api/src/altrep_impl/arrays.rs:142 |
| `[bool; N]` | `<N>` | concrete | 0 | miniextendr-api/src/altrep_impl/arrays.rs:160 |
| `[String; N]` | `<N>` | concrete | 0 | miniextendr-api/src/altrep_impl/arrays.rs:218 |
| `Vec<i32>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:339 |
| `Vec<f64>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:346 |
| `Vec<bool>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:353 |
| `Vec<u8>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:360 |
| `Vec<String>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:367 |
| `Vec<Option<String>>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:374 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:384 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:391 |
| `Vec<crate::Rcomplex>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:398 |
| `std::ops::Range<i32>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:407 |
| `std::ops::Range<i64>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:414 |
| `std::ops::Range<f64>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:421 |
| `Box<[i32]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:436 |
| `Box<[f64]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:443 |
| `Box<[bool]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:450 |
| `Box<[u8]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:457 |
| `Box<[String]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:464 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:471 |
| `std::borrow::Cow<'static, [i32]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:481 |
| `std::borrow::Cow<'static, [f64]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:488 |
| `std::borrow::Cow<'static, [u8]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:495 |
| `std::borrow::Cow<'static, [crate::Rcomplex]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:502 |
| `&'static [f64]` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/static_slices.rs:140 |
| `&'static [bool]` | `` | concrete | 0 | miniextendr-api/src/altrep_impl/static_slices.rs:243 |
| `&'static [u8]` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/static_slices.rs:296 |
| `&'static [i32]` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/static_slices.rs:31 |
| `&'static [String]` | `` | concrete | 0 | miniextendr-api/src/altrep_impl/static_slices.rs:361 |
| `&'static [&'static str]` | `` | concrete | 0 | miniextendr-api/src/altrep_impl/static_slices.rs:401 |
| `Float64Array` | `` | concrete | 4 | miniextendr-api/src/optionals/arrow_impl.rs:1864 |
| `Int32Array` | `` | concrete | 4 | miniextendr-api/src/optionals/arrow_impl.rs:1865 |
| `UInt8Array` | `` | concrete | 4 | miniextendr-api/src/optionals/arrow_impl.rs:1866 |
| `BooleanArray` | `` | concrete | 4 | miniextendr-api/src/optionals/arrow_impl.rs:1867 |
| `StringArray` | `` | concrete | 4 | miniextendr-api/src/optionals/arrow_impl.rs:1969 |
| `JiffTimestampVec` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffZonedVec` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `DVector<f64>` | `` | concrete | 4 | miniextendr-api/src/optionals/nalgebra_impl.rs:1673 |
| `DVector<i32>` | `` | concrete | 4 | miniextendr-api/src/optionals/nalgebra_impl.rs:1674 |
| `Array1<f64>` | `` | concrete | 4 | miniextendr-api/src/optionals/ndarray_impl.rs:3857 |
| `Array1<i32>` | `` | concrete | 4 | miniextendr-api/src/optionals/ndarray_impl.rs:3858 |

## `PartialEq` — 44 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `mx_tag` | `` | concrete | 1 | miniextendr-api/src/abi.rs:82 |
| `RBase` | `` | concrete | 1 | miniextendr-api/src/altrep.rs:51 |
| `Sortedness` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:131 |
| `Logical` | `` | concrete | 1 | miniextendr-api/src/altrep_data/core.rs:54 |
| `AltrepGuard` | `` | concrete | 1 | miniextendr-api/src/altrep_traits.rs:60 |
| `LogicalCoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:481 |
| `CoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:85 |
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:919 |
| `GroupKey` | `` | concrete | 1 | miniextendr-api/src/dataframe/group.rs:81 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1707 |
| `RSidecar` | `` | concrete | 1 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `GuardMode` | `` | concrete | 1 | miniextendr-api/src/ffi_guard.rs:48 |
| `Altrep<T>` | `<T>` | concrete | 1 | miniextendr-api/src/into_r/altrep.rs:88 |
| `NullOnErr` | `` | concrete | 1 | miniextendr-api/src/into_r/result.rs:133 |
| `StorageCoerceError` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:59 |
| `Missing<T>` | `<T>` | concrete | 1 | miniextendr-api/src/missing.rs:98 |
| `NamedVector<M>` | `<M>` | concrete | 1 | miniextendr-api/src/named_vector.rs:191 |
| `RFlags<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/bitflags_impl.rs:99 |
| `GlobOptions` | `` | concrete | 1 | miniextendr-api/src/optionals/globset_impl.rs:49 |
| `PanicSource` | `` | concrete | 1 | miniextendr-api/src/panic_telemetry.rs:48 |
| `TermKind` | `` | concrete | 1 | miniextendr-api/src/progress.rs:67 |
| `ProtectKey` | `` | concrete | 1 | miniextendr-api/src/protect_pool.rs:61 |
| `Raw<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:197 |
| `RawSlice<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:216 |
| `RawTagged<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:245 |
| `RawSliceTagged<T>` | `<T>` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:265 |
| `RawError` | `` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:69 |
| `RWrapperPriority` | `` | concrete | 1 | miniextendr-api/src/registry.rs:209 |
| `ColumnType` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:1296 |
| `SchemaMode` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:1362 |
| `TypeSpec` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:818 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:224 |
| `SEXP` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:65 |
| `RLogical` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:162 |
| `Rcomplex` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:24 |
| `Rboolean` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:321 |
| `SEXPTYPE` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:35 |
| `RNGtype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1493 |
| `N01type` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1517 |
| `Sampletype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1537 |
| `ParseStatus` | `` | concrete | 1 | miniextendr-api/src/sys.rs:971 |
| `TypeSpec` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:119 |
| `VctrsBuildError` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:32 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:525 |

## `StructuralPartialEq` — 43 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `mx_tag` | `` | concrete | 0 | miniextendr-api/src/abi.rs:82 |
| `RBase` | `` | concrete | 0 | miniextendr-api/src/altrep.rs:51 |
| `Sortedness` | `` | concrete | 0 | miniextendr-api/src/altrep_data/core.rs:131 |
| `Logical` | `` | concrete | 0 | miniextendr-api/src/altrep_data/core.rs:54 |
| `AltrepGuard` | `` | concrete | 0 | miniextendr-api/src/altrep_traits.rs:60 |
| `LogicalCoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:481 |
| `CoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:85 |
| `Coerced<T, R>` | `<T, R>` | concrete | 0 | miniextendr-api/src/coerce.rs:919 |
| `GroupKey` | `` | concrete | 0 | miniextendr-api/src/dataframe/group.rs:81 |
| `RSidecar` | `` | concrete | 0 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `GuardMode` | `` | concrete | 0 | miniextendr-api/src/ffi_guard.rs:48 |
| `Altrep<T>` | `<T>` | concrete | 0 | miniextendr-api/src/into_r/altrep.rs:88 |
| `NullOnErr` | `` | concrete | 0 | miniextendr-api/src/into_r/result.rs:133 |
| `StorageCoerceError` | `` | concrete | 0 | miniextendr-api/src/into_r_as.rs:59 |
| `Missing<T>` | `<T>` | concrete | 0 | miniextendr-api/src/missing.rs:98 |
| `NamedVector<M>` | `<M>` | concrete | 0 | miniextendr-api/src/named_vector.rs:191 |
| `RFlags<T>` | `<T>` | concrete | 0 | miniextendr-api/src/optionals/bitflags_impl.rs:99 |
| `GlobOptions` | `` | concrete | 0 | miniextendr-api/src/optionals/globset_impl.rs:49 |
| `PanicSource` | `` | concrete | 0 | miniextendr-api/src/panic_telemetry.rs:48 |
| `TermKind` | `` | concrete | 0 | miniextendr-api/src/progress.rs:67 |
| `ProtectKey` | `` | concrete | 0 | miniextendr-api/src/protect_pool.rs:61 |
| `Raw<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:197 |
| `RawSlice<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:216 |
| `RawTagged<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:245 |
| `RawSliceTagged<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:265 |
| `RawError` | `` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:69 |
| `RWrapperPriority` | `` | concrete | 0 | miniextendr-api/src/registry.rs:209 |
| `ColumnType` | `` | concrete | 0 | miniextendr-api/src/serde/columnar.rs:1296 |
| `SchemaMode` | `` | concrete | 0 | miniextendr-api/src/serde/columnar.rs:1362 |
| `TypeSpec` | `` | concrete | 0 | miniextendr-api/src/serde/columnar.rs:818 |
| `AsSerialize<T>` | `<T>` | concrete | 0 | miniextendr-api/src/serde/traits.rs:224 |
| `SEXP` | `` | concrete | 0 | miniextendr-api/src/sexp.rs:65 |
| `RLogical` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:162 |
| `Rcomplex` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:24 |
| `Rboolean` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:321 |
| `SEXPTYPE` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:35 |
| `RNGtype` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1493 |
| `N01type` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1517 |
| `Sampletype` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1537 |
| `ParseStatus` | `` | concrete | 0 | miniextendr-api/src/sys.rs:971 |
| `TypeSpec` | `` | concrete | 0 | miniextendr-api/src/typed_list.rs:119 |
| `VctrsBuildError` | `` | concrete | 0 | miniextendr-api/src/vctrs.rs:32 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-api/src/vctrs.rs:525 |

## `Eq` — 40 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `mx_tag` | `` | concrete | 0 | miniextendr-api/src/abi.rs:82 |
| `RBase` | `` | concrete | 0 | miniextendr-api/src/altrep.rs:51 |
| `Sortedness` | `` | concrete | 0 | miniextendr-api/src/altrep_data/core.rs:131 |
| `Logical` | `` | concrete | 0 | miniextendr-api/src/altrep_data/core.rs:54 |
| `AltrepGuard` | `` | concrete | 0 | miniextendr-api/src/altrep_traits.rs:60 |
| `LogicalCoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:481 |
| `CoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:85 |
| `Coerced<T, R>` | `<T, R>` | concrete | 0 | miniextendr-api/src/coerce.rs:919 |
| `GroupKey` | `` | concrete | 0 | miniextendr-api/src/dataframe/group.rs:81 |
| `ExternalPtr<T>` | `<T>` | concrete | 0 | miniextendr-api/src/externalptr.rs:1714 |
| `RSidecar` | `` | concrete | 0 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `GuardMode` | `` | concrete | 0 | miniextendr-api/src/ffi_guard.rs:48 |
| `Altrep<T>` | `<T>` | concrete | 0 | miniextendr-api/src/into_r/altrep.rs:88 |
| `NullOnErr` | `` | concrete | 0 | miniextendr-api/src/into_r/result.rs:133 |
| `StorageCoerceError` | `` | concrete | 0 | miniextendr-api/src/into_r_as.rs:59 |
| `Missing<T>` | `<T>` | concrete | 0 | miniextendr-api/src/missing.rs:98 |
| `NamedVector<M>` | `<M>` | concrete | 0 | miniextendr-api/src/named_vector.rs:191 |
| `RFlags<T>` | `<T>` | concrete | 0 | miniextendr-api/src/optionals/bitflags_impl.rs:99 |
| `GlobOptions` | `` | concrete | 0 | miniextendr-api/src/optionals/globset_impl.rs:49 |
| `PanicSource` | `` | concrete | 0 | miniextendr-api/src/panic_telemetry.rs:48 |
| `TermKind` | `` | concrete | 0 | miniextendr-api/src/progress.rs:67 |
| `ProtectKey` | `` | concrete | 0 | miniextendr-api/src/protect_pool.rs:61 |
| `Raw<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:197 |
| `RawSlice<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:216 |
| `RawTagged<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:245 |
| `RawSliceTagged<T>` | `<T>` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:265 |
| `RawError` | `` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:69 |
| `RWrapperPriority` | `` | concrete | 0 | miniextendr-api/src/registry.rs:209 |
| `TypeSpec` | `` | concrete | 0 | miniextendr-api/src/serde/columnar.rs:818 |
| `AsSerialize<T>` | `<T>` | concrete | 0 | miniextendr-api/src/serde/traits.rs:224 |
| `SEXP` | `` | concrete | 0 | miniextendr-api/src/sexp.rs:65 |
| `RLogical` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:162 |
| `Rboolean` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:321 |
| `SEXPTYPE` | `` | concrete | 0 | miniextendr-api/src/sexp_types.rs:35 |
| `RNGtype` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1493 |
| `N01type` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1517 |
| `Sampletype` | `` | concrete | 0 | miniextendr-api/src/sys.rs:1537 |
| `ParseStatus` | `` | concrete | 0 | miniextendr-api/src/sys.rs:971 |
| `VctrsBuildError` | `` | concrete | 0 | miniextendr-api/src/vctrs.rs:32 |
| `VctrsKind` | `` | concrete | 0 | miniextendr-api/src/vctrs.rs:525 |

## `RegisterAltrep` — 33 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:554 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:555 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:556 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:557 |
| `Vec<String>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:558 |
| `Vec<Option<String>>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:559 |
| `Vec<crate::Rcomplex>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:560 |
| `std::ops::Range<i32>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:563 |
| `std::ops::Range<i64>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:564 |
| `std::ops::Range<f64>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:565 |
| `Box<[i32]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:568 |
| `Box<[f64]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:569 |
| `Box<[bool]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:570 |
| `Box<[u8]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:571 |
| `Box<[String]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:572 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:573 |
| `std::borrow::Cow<'static, [i32]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:576 |
| `std::borrow::Cow<'static, [f64]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:577 |
| `std::borrow::Cow<'static, [u8]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:578 |
| `std::borrow::Cow<'static, [crate::Rcomplex]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:579 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:582 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:583 |
| `Float64Array` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1871 |
| `Int32Array` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1890 |
| `UInt8Array` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1909 |
| `BooleanArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1928 |
| `StringArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1971 |
| `JiffTimestampVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffZonedVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `DVector<f64>` | `` | concrete | 1 | miniextendr-api/src/optionals/nalgebra_impl.rs:1678 |
| `DVector<i32>` | `` | concrete | 1 | miniextendr-api/src/optionals/nalgebra_impl.rs:1695 |
| `Array1<f64>` | `` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:3862 |
| `Array1<i32>` | `` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:3879 |

## `AltrepDataptr` — 27 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1097 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1104 |
| `Cow<'static, [i32]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1188 |
| `Cow<'static, [f64]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1193 |
| `Cow<'static, [u8]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1198 |
| `Cow<'static, [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1203 |
| `Vec<i32>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:497 |
| `Vec<f64>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:501 |
| `Vec<u8>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:505 |
| `Box<[i32]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:582 |
| `Box<[f64]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:586 |
| `Box<[u8]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:590 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:353 |
| `std::ops::Range<i32>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:407 |
| `std::ops::Range<i64>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:414 |
| `std::ops::Range<f64>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:421 |
| `Box<[bool]>` | `` | concrete | 1 | miniextendr-api/src/altrep_impl/builtins.rs:450 |
| `Float64Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1708 |
| `Int32Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1728 |
| `UInt8Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1746 |
| `BooleanArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:1867 |
| `JiffTimestampVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffZonedVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `DVector<f64>` | `` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:1653 |
| `DVector<i32>` | `` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:1663 |
| `Array1<f64>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:3821 |
| `Array1<i32>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:3839 |

## `AltrepSerialize` — 27 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cow<'static, [i32]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1189 |
| `Cow<'static, [f64]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1194 |
| `Cow<'static, [u8]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1199 |
| `Cow<'static, [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1204 |
| `Vec<i32>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:390 |
| `Vec<f64>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:391 |
| `Vec<u8>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:392 |
| `Vec<bool>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:393 |
| `Vec<String>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:394 |
| `Vec<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:395 |
| `Vec<crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:396 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:399 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:400 |
| `Box<[i32]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:408 |
| `Box<[f64]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:422 |
| `Box<[u8]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:436 |
| `Box<[bool]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:450 |
| `Box<[String]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:464 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:478 |
| `std::ops::Range<i32>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:611 |
| `std::ops::Range<i64>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:624 |
| `std::ops::Range<f64>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:647 |
| `Float64Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1781 |
| `Int32Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1804 |
| `UInt8Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1827 |
| `BooleanArray` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1846 |
| `StringArray` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1855 |

## `RDefault` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:425 |

## `Default` — 24 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:919 |
| `RNullConnection` | `` | concrete | 1 | miniextendr-api/src/connection.rs:1454 |
| `RCustomConnection` | `` | concrete | 1 | miniextendr-api/src/connection.rs:843 |
| `NamedDataFrameListBuilder` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:1537 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1681 |
| `RSidecar` | `` | concrete | 1 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `Missing<T>` | `<T>` | concrete | 1 | miniextendr-api/src/missing.rs:213 |
| `RSessionContext` | `` | concrete | 1 | miniextendr-api/src/optionals/datafusion_impl.rs:188 |
| `GlobOptions` | `` | concrete | 1 | miniextendr-api/src/optionals/globset_impl.rs:61 |
| `RRng` | `` | concrete | 1 | miniextendr-api/src/optionals/rand_impl.rs:109 |
| `FactorHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:102 |
| `JsonOptions` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:125 |
| `NaHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:78 |
| `SpecialFloatHandling` | `` | concrete | 1 | miniextendr-api/src/optionals/serde_impl.rs:90 |
| `WorkerPump<T>` | `<T>` | concrete | 1 | miniextendr-api/src/pump.rs:181 |
| `ValueExtractor` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:2065 |
| `ExtractedValue` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:2070 |
| `VariantNameExtractor` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:2634 |
| `DispatchNames` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:943 |
| `EnumReadConfig` | `` | concrete | 1 | miniextendr-api/src/serde/dataframe_de.rs:498 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:224 |
| `SEXP` | `` | concrete | 1 | miniextendr-api/src/sexp.rs:475 |
| `RThreadBuilder` | `` | concrete | 1 | miniextendr-api/src/thread.rs:319 |
| `VctrsKind` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:525 |

## `Drop` — 23 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RNullConnection` | `` | concrete | 1 | miniextendr-api/src/connection.rs:1480 |
| `BuiltDataFrame` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:919 |
| `GroupedDataFrame` | `` | concrete | 1 | miniextendr-api/src/dataframe/group.rs:169 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1798 |
| `AbortIfUnwinding` | `` | concrete | 1 | miniextendr-api/src/externalptr.rs:1823 |
| `ExternalSlice<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1967 |
| `ProtectScope` | `` | concrete | 1 | miniextendr-api/src/gc_protect.rs:1015 |
| `OwnedProtect` | `` | concrete | 1 | miniextendr-api/src/gc_protect.rs:1142 |
| `WorkerUnprotectGuard` | `` | concrete | 1 | miniextendr-api/src/gc_protect.rs:1598 |
| `TlsScopeGuard` | `` | concrete | 1 | miniextendr-api/src/gc_protect/tls.rs:64 |
| `RPreservedSexp` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:292 |
| `RVecStorage<T, R, C>` | `<T, R, C>` | concrete | 1 | miniextendr-api/src/optionals/nalgebra_impl.rs:1183 |
| `RndVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:3199 |
| `RndMat<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/ndarray_impl.rs:3344 |
| `RTerm` | `` | concrete | 1 | miniextendr-api/src/progress.rs:235 |
| `ProtectPool` | `` | concrete | 1 | miniextendr-api/src/protect_pool.rs:301 |
| `RefCountedArena` | `` | concrete | 1 | miniextendr-api/src/refcount_protect.rs:530 |
| `ArenaGuard<'_>` | `` | concrete | 1 | miniextendr-api/src/refcount_protect.rs:569 |
| `ThreadLocalState` | `` | concrete | 1 | miniextendr-api/src/refcount_protect.rs:632 |
| `RngGuard` | `` | concrete | 1 | miniextendr-api/src/rng.rs:174 |
| `RootedSentinel` | `` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:3240 |
| `StackCheckGuard` | `` | concrete | 1 | miniextendr-api/src/thread.rs:155 |
| `RTxtProgressBar` | `` | concrete | 1 | miniextendr-api/src/txt_progress_bar.rs:157 |

## `RDisplay` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:102 |

## `Display` — 22 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LogicalCoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:489 |
| `CoerceError` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:97 |
| `AsRError<E>` | `<E>` | concrete | 1 | miniextendr-api/src/condition.rs:879 |
| `DataFrameError` | `` | concrete | 1 | miniextendr-api/src/dataframe.rs:115 |
| `GroupKey` | `` | concrete | 1 | miniextendr-api/src/dataframe/group.rs:124 |
| `TypeMismatchError` | `` | concrete | 1 | miniextendr-api/src/externalptr.rs:1549 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1695 |
| `SexpTypeError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:191 |
| `SexpLengthError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:212 |
| `SexpNaError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:231 |
| `SexpError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:266 |
| `StorageCoerceError` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:123 |
| `IntoRError` | `` | concrete | 1 | miniextendr-api/src/into_r_error.rs:30 |
| `DuplicateNameError` | `` | concrete | 1 | miniextendr-api/src/list.rs:1212 |
| `ListFromSexpError` | `` | concrete | 1 | miniextendr-api/src/list.rs:1229 |
| `MatchArgError` | `` | concrete | 1 | miniextendr-api/src/match_arg.rs:73 |
| `RFlags<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/bitflags_impl.rs:138 |
| `RCoerceError` | `` | concrete | 1 | miniextendr-api/src/r_coerce.rs:138 |
| `RawError` | `` | concrete | 1 | miniextendr-api/src/raw_conversions.rs:84 |
| `RSerdeError` | `` | concrete | 1 | miniextendr-api/src/serde/error.rs:80 |
| `TypedListError` | `` | concrete | 1 | miniextendr-api/src/typed_list.rs:230 |
| `VctrsBuildError` | `` | concrete | 1 | miniextendr-api/src/vctrs.rs:78 |

## `Deref` — 21 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Coerced<T, R>` | `<T, R>` | concrete | 2 | miniextendr-api/src/coerce.rs:954 |
| `BuiltDataFrame` | `` | concrete | 2 | miniextendr-api/src/dataframe.rs:911 |
| `ExternalPtr<T>` | `<T>` | concrete | 2 | miniextendr-api/src/externalptr.rs:1621 |
| `Factor<'_>` | `` | concrete | 2 | miniextendr-api/src/factor.rs:213 |
| `FactorMut<'_>` | `` | concrete | 2 | miniextendr-api/src/factor.rs:309 |
| `FactorVec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/factor.rs:488 |
| `FactorOptionVec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/factor.rs:546 |
| `Root<'a>` | `<'a>` | concrete | 2 | miniextendr-api/src/gc_protect.rs:1063 |
| `OwnedProtect` | `` | concrete | 2 | miniextendr-api/src/gc_protect.rs:1151 |
| `Protected<'a, T>` | `<'a, T>` | concrete | 2 | miniextendr-api/src/gc_protect.rs:1265 |
| `TlsRoot` | `` | concrete | 2 | miniextendr-api/src/gc_protect/tls.rs:221 |
| `Altrep<T>` | `<T>` | concrete | 2 | miniextendr-api/src/into_r/altrep.rs:132 |
| `RPrimitive<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:150 |
| `RStringArray` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:234 |
| `RFlags<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:117 |
| `JiffTimestampVecMut` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffTimestampVecRef` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffZonedVecRef` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `JiffZonedVecMut` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `RCow<'_, T>` | `<T>` | concrete | 2 | miniextendr-api/src/rcow.rs:148 |
| `ArenaGuard<'_>` | `` | concrete | 2 | miniextendr-api/src/refcount_protect.rs:575 |

### `Deref` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/optionals/jiff_impl.rs:915** (2 impls): `JiffZonedVecRef`, `JiffZonedVecMut`
- **miniextendr-api/src/optionals/jiff_impl.rs:874** (2 impls): `JiffTimestampVecMut`, `JiffTimestampVecRef`

## `Error` — 20 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:108 |
| `LogicalCoerceError` | `` | concrete | 0 | miniextendr-api/src/coerce.rs:498 |
| `DataFrameError` | `` | concrete | 0 | miniextendr-api/src/dataframe.rs:172 |
| `TypeMismatchError` | `` | concrete | 0 | miniextendr-api/src/externalptr.rs:1565 |
| `SexpTypeError` | `` | concrete | 0 | miniextendr-api/src/from_r.rs:201 |
| `SexpLengthError` | `` | concrete | 0 | miniextendr-api/src/from_r.rs:222 |
| `SexpNaError` | `` | concrete | 0 | miniextendr-api/src/from_r.rs:237 |
| `SexpError` | `` | concrete | 1 | miniextendr-api/src/from_r.rs:288 |
| `StorageCoerceError` | `` | concrete | 0 | miniextendr-api/src/into_r_as.rs:204 |
| `IntoRError` | `` | concrete | 0 | miniextendr-api/src/into_r_error.rs:49 |
| `DuplicateNameError` | `` | concrete | 0 | miniextendr-api/src/list.rs:1218 |
| `ListFromSexpError` | `` | concrete | 0 | miniextendr-api/src/list.rs:1238 |
| `MatchArgError` | `` | concrete | 0 | miniextendr-api/src/match_arg.rs:99 |
| `RCoerceError` | `` | concrete | 0 | miniextendr-api/src/r_coerce.rs:155 |
| `RawError` | `` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:107 |
| `RSerdeError` | `` | concrete | 0 | miniextendr-api/src/serde/error.rs:120 |
| `RSerdeError` | `` | concrete | 1 | miniextendr-api/src/serde/error.rs:68 |
| `RSerdeError` | `` | concrete | 1 | miniextendr-api/src/serde/error.rs:74 |
| `TypedListError` | `` | concrete | 0 | miniextendr-api/src/typed_list.rs:263 |
| `VctrsBuildError` | `` | concrete | 0 | miniextendr-api/src/vctrs.rs:123 |

## `RError` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 3 | miniextendr-api/src/adapter_traits.rs:288 |

## `RHash` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:148 |

## `AltIntegerData` — 16 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[i32; N]` | `<N>` | concrete | 4 | miniextendr-api/src/altrep_data/builtins.rs:1003 |
| `std::borrow::Cow<'static, [i32]>` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:1187 |
| `Vec<i32>` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:496 |
| `Box<[i32]>` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:581 |
| `std::ops::Range<i32>` | `` | concrete | 6 | miniextendr-api/src/altrep_data/builtins.rs:673 |
| `std::ops::Range<i64>` | `` | concrete | 6 | miniextendr-api/src/altrep_data/builtins.rs:770 |
| `&[i32]` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:956 |
| `IterIntFromBoolData<I>` | `<I> +1wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:227 |
| `IterIntCoerceData<I, T>` | `<I, T> +2wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:80 |
| `SparseIterIntData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:218 |
| `IterIntData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:266 |
| `WindowedIterIntData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/windowed.rs:265 |
| `StreamingIntData<F>` | `<F>` | concrete | 2 | miniextendr-api/src/altrep_data/stream.rs:187 |
| `Int32Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1652 |
| `DVector<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1639 |
| `Array1<i32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3803 |

## `AltRealData` — 16 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[f64; N]` | `<N>` | concrete | 4 | miniextendr-api/src/altrep_data/builtins.rs:1028 |
| `std::borrow::Cow<'static, [f64]>` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:1192 |
| `Vec<f64>` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:500 |
| `Box<[f64]>` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:585 |
| `std::ops::Range<f64>` | `` | concrete | 6 | miniextendr-api/src/altrep_data/builtins.rs:912 |
| `&[f64]` | `` | concrete | 7 | miniextendr-api/src/altrep_data/builtins.rs:959 |
| `IterRealCoerceData<I, T>` | `<I, T> +2wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:161 |
| `SparseIterRealData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:267 |
| `IterRealData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:316 |
| `WindowedIterRealData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/windowed.rs:314 |
| `StreamingRealData<F>` | `<F>` | concrete | 2 | miniextendr-api/src/altrep_data/stream.rs:90 |
| `Float64Array` | `` | concrete | 3 | miniextendr-api/src/optionals/arrow_impl.rs:1627 |
| `JiffTimestampVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:896 |
| `JiffZonedVec` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:959 |
| `DVector<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1625 |
| `Array1<f64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:3784 |

## `Hash` — 15 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `mx_tag` | `` | concrete | 1 | miniextendr-api/src/abi.rs:82 |
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:919 |
| `GroupKey` | `` | concrete | 1 | miniextendr-api/src/dataframe/group.rs:81 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1730 |
| `RSidecar` | `` | concrete | 1 | miniextendr-api/src/externalptr/altrep_helpers.rs:173 |
| `Altrep<T>` | `<T>` | concrete | 1 | miniextendr-api/src/into_r/altrep.rs:88 |
| `RFlags<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/bitflags_impl.rs:99 |
| `ProtectKey` | `` | concrete | 1 | miniextendr-api/src/protect_pool.rs:61 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:224 |
| `RLogical` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:162 |
| `Rboolean` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:321 |
| `SEXPTYPE` | `` | concrete | 1 | miniextendr-api/src/sexp_types.rs:35 |
| `RNGtype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1493 |
| `N01type` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1517 |
| `Sampletype` | `` | concrete | 1 | miniextendr-api/src/sys.rs:1537 |

## `TraitView` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RHashView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:138 |
| `ROrdView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:176 |
| `RPartialOrdView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:216 |
| `RErrorView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:276 |
| `RFromStrView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:340 |
| `RCloneView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:380 |
| `RDefaultView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:419 |
| `RCopyView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:466 |
| `RDebugView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:54 |
| `RIteratorView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:552 |
| `RDisplayView` | `` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:96 |
| `RRngOpsView` | `` | concrete | 2 | miniextendr-api/src/optionals/rand_impl.rs:321 |

## `AltReal` — 11 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[f64; N]` | `<N>` | concrete | 6 | miniextendr-api/src/altrep_impl/arrays.rs:125 |
| `Vec<f64>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:346 |
| `std::ops::Range<f64>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:421 |
| `Box<[f64]>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:443 |
| `std::borrow::Cow<'static, [f64]>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:488 |
| `&'static [f64]` | `` | concrete | 12 | miniextendr-api/src/altrep_impl/static_slices.rs:162 |
| `Float64Array` | `` | concrete | 14 | miniextendr-api/src/optionals/arrow_impl.rs:1864 |
| `JiffTimestampVec` | `` | concrete | 14 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffZonedVec` | `` | concrete | 14 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `DVector<f64>` | `` | concrete | 14 | miniextendr-api/src/optionals/nalgebra_impl.rs:1673 |
| `Array1<f64>` | `` | concrete | 14 | miniextendr-api/src/optionals/ndarray_impl.rs:3857 |

## `AltInteger` — 10 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[i32; N]` | `<N>` | concrete | 6 | miniextendr-api/src/altrep_impl/arrays.rs:116 |
| `Vec<i32>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:339 |
| `std::ops::Range<i32>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:407 |
| `std::ops::Range<i64>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:414 |
| `Box<[i32]>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:436 |
| `std::borrow::Cow<'static, [i32]>` | `` | concrete | 14 | miniextendr-api/src/altrep_impl/builtins.rs:481 |
| `&'static [i32]` | `` | concrete | 12 | miniextendr-api/src/altrep_impl/static_slices.rs:54 |
| `Int32Array` | `` | concrete | 14 | miniextendr-api/src/optionals/arrow_impl.rs:1865 |
| `DVector<i32>` | `` | concrete | 14 | miniextendr-api/src/optionals/nalgebra_impl.rs:1674 |
| `Array1<i32>` | `` | concrete | 14 | miniextendr-api/src/optionals/ndarray_impl.rs:3858 |

## `AltStringData` — 10 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[String; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:1086 |
| `Vec<String>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:509 |
| `Vec<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:521 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:537 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:553 |
| `Box<[String]>` | `` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:597 |
| `&[String]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:969 |
| `&[&str]` | `` | concrete | 1 | miniextendr-api/src/altrep_data/builtins.rs:977 |
| `IterStringData<I>` | `<I> +1wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:320 |
| `StringArray` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1955 |

## `Serializer` — 10 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `&'a mut SchemaDiscoverer` | `<'a>` | concrete | 37 | miniextendr-api/src/serde/columnar.rs:1647 |
| `&mut TypeProbe` | `` | concrete | 37 | miniextendr-api/src/serde/columnar.rs:1734 |
| `&mut ValueExtractor` | `` | concrete | 37 | miniextendr-api/src/serde/columnar.rs:2080 |
| `ColumnFiller<'a>` | `<'a>` | concrete | 37 | miniextendr-api/src/serde/columnar.rs:2340 |
| `&'a mut VariantNameExtractor` | `<'a>` | concrete | 37 | miniextendr-api/src/serde/columnar.rs:2748 |
| `VariantStrippingSerializer<S>` | `<S>` | concrete | 37 | miniextendr-api/src/serde/columnar.rs:2940 |
| `VariantStrippingMapForwarder<'m, M>` | `<'m, M>` | concrete | 37 | miniextendr-api/src/serde/columnar.rs:3596 |
| `FieldSelectingForwarder<'m, M>` | `<'m, M>` | concrete | 37 | miniextendr-api/src/serde/columnar.rs:4035 |
| `ParColumnFiller<'a>` | `<'a>` | concrete | 37 | miniextendr-api/src/serde/columnar.rs:723 |
| `RSerializer` | `` | concrete | 37 | miniextendr-api/src/serde/ser.rs:51 |

## `AtomicElement` — 9 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `bool` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:106 |
| `String` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:118 |
| `Option<i32>` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:130 |
| `Option<f64>` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:140 |
| `Option<bool>` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:150 |
| `Option<String>` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:160 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:47 |
| `f64` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:66 |
| `u8` | `` | concrete | 2 | miniextendr-api/src/named_vector.rs:85 |

## `AltString` — 9 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[String; N]` | `<N>` | concrete | 1 | miniextendr-api/src/altrep_impl/arrays.rs:220 |
| `Vec<String>` | `` | concrete | 5 | miniextendr-api/src/altrep_impl/builtins.rs:367 |
| `Vec<Option<String>>` | `` | concrete | 5 | miniextendr-api/src/altrep_impl/builtins.rs:374 |
| `Vec<std::borrow::Cow<'static, str>>` | `` | concrete | 5 | miniextendr-api/src/altrep_impl/builtins.rs:384 |
| `Vec<Option<std::borrow::Cow<'static, str>>>` | `` | concrete | 5 | miniextendr-api/src/altrep_impl/builtins.rs:391 |
| `Box<[String]>` | `` | concrete | 5 | miniextendr-api/src/altrep_impl/builtins.rs:464 |
| `&'static [String]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:363 |
| `&'static [&'static str]` | `` | concrete | 3 | miniextendr-api/src/altrep_impl/static_slices.rs:403 |
| `StringArray` | `` | concrete | 5 | miniextendr-api/src/optionals/arrow_impl.rs:1969 |

## `AltRawData` — 8 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[u8; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1065 |
| `std::borrow::Cow<'static, [u8]>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1197 |
| `Vec<u8>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:504 |
| `Box<[u8]>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:589 |
| `&[u8]` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:962 |
| `SparseIterRawData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:352 |
| `IterRawData<I>` | `<I>` | concrete | 3 | miniextendr-api/src/altrep_data/iter/state.rs:411 |
| `UInt8Array` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1674 |

## `DerefMut` — 8 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:963 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1630 |
| `FactorMut<'_>` | `` | concrete | 1 | miniextendr-api/src/factor.rs:318 |
| `FactorVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:495 |
| `FactorOptionVec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/factor.rs:553 |
| `Altrep<T>` | `<T>` | concrete | 1 | miniextendr-api/src/into_r/altrep.rs:141 |
| `JiffTimestampVecMut` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffZonedVecMut` | `` | concrete | 1 | miniextendr-api/src/optionals/jiff_impl.rs:915 |

## `SerializeStruct` — 7 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SchemaStructDiscoverer<'_>` | `` | concrete | 4 | miniextendr-api/src/serde/columnar.rs:1680 |
| `ColumnFiller<'_>` | `` | concrete | 4 | miniextendr-api/src/serde/columnar.rs:2383 |
| `TagStructCapture<'_>` | `` | concrete | 4 | miniextendr-api/src/serde/columnar.rs:2678 |
| `ForwardingMapEmitter<'_, M>` | `<M>` | concrete | 4 | miniextendr-api/src/serde/columnar.rs:3767 |
| `SelectingMapEmitter<'_, M>` | `<M>` | concrete | 4 | miniextendr-api/src/serde/columnar.rs:4207 |
| `ParColumnFiller<'_>` | `` | concrete | 4 | miniextendr-api/src/serde/columnar.rs:766 |
| `StructSerializer` | `` | concrete | 4 | miniextendr-api/src/serde/ser.rs:401 |

## `WidensToF64` — 7 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `f32` | `` | concrete | 0 | miniextendr-api/src/markers.rs:219 |
| `i8` | `` | concrete | 0 | miniextendr-api/src/markers.rs:220 |
| `i16` | `` | concrete | 0 | miniextendr-api/src/markers.rs:221 |
| `i32` | `` | concrete | 0 | miniextendr-api/src/markers.rs:222 |
| `u8` | `` | concrete | 0 | miniextendr-api/src/markers.rs:223 |
| `u16` | `` | concrete | 0 | miniextendr-api/src/markers.rs:224 |
| `u32` | `` | concrete | 0 | miniextendr-api/src/markers.rs:225 |

## `IoCaps` — 7 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IoRead<T>` | `<T>` | concrete | 1 | miniextendr-api/src/connection/io_adapters.rs:102 |
| `IoWrite<T>` | `<T>` | concrete | 1 | miniextendr-api/src/connection/io_adapters.rs:130 |
| `IoReadWrite<T>` | `<T>` | concrete | 2 | miniextendr-api/src/connection/io_adapters.rs:162 |
| `IoReadSeek<T>` | `<T>` | concrete | 2 | miniextendr-api/src/connection/io_adapters.rs:184 |
| `IoWriteSeek<T>` | `<T>` | concrete | 2 | miniextendr-api/src/connection/io_adapters.rs:200 |
| `IoReadWriteSeek<T>` | `<T>` | concrete | 3 | miniextendr-api/src/connection/io_adapters.rs:220 |
| `IoBufRead<T>` | `<T>` | concrete | 2 | miniextendr-api/src/connection/io_adapters.rs:260 |

## `RConnectionImpl` — 7 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IoRead<T>` | `<T>` | concrete | 1 | miniextendr-api/src/connection/io_adapters.rs:102 |
| `IoWrite<T>` | `<T>` | concrete | 2 | miniextendr-api/src/connection/io_adapters.rs:130 |
| `IoReadWrite<T>` | `<T>` | concrete | 3 | miniextendr-api/src/connection/io_adapters.rs:162 |
| `IoReadSeek<T>` | `<T>` | concrete | 2 | miniextendr-api/src/connection/io_adapters.rs:184 |
| `IoWriteSeek<T>` | `<T>` | concrete | 3 | miniextendr-api/src/connection/io_adapters.rs:200 |
| `IoReadWriteSeek<T>` | `<T>` | concrete | 4 | miniextendr-api/src/connection/io_adapters.rs:220 |
| `IoBufRead<T>` | `<T>` | concrete | 2 | miniextendr-api/src/connection/io_adapters.rs:260 |

## `IntoIterator` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `StrVec<'a>` | `<'a>` | concrete | 3 | miniextendr-api/src/strvec.rs:347 |
| `&'a ProtectedStrVec` | `<'a>` | concrete | 3 | miniextendr-api/src/strvec.rs:717 |

## `SerializeMap` — 7 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SchemaMapDiscoverer<'_>` | `` | concrete | 5 | miniextendr-api/src/serde/columnar.rs:1703 |
| `ColumnFiller<'_>` | `` | concrete | 5 | miniextendr-api/src/serde/columnar.rs:2401 |
| `TagMapCapture<'_>` | `` | concrete | 5 | miniextendr-api/src/serde/columnar.rs:2714 |
| `ForwardingMapEmitter<'_, M>` | `<M>` | concrete | 6 | miniextendr-api/src/serde/columnar.rs:3801 |
| `SelectingMapEmitter<'_, M>` | `<M>` | concrete | 6 | miniextendr-api/src/serde/columnar.rs:4224 |
| `ParColumnFiller<'_>` | `` | concrete | 5 | miniextendr-api/src/serde/columnar.rs:785 |
| `MapSerializer` | `` | concrete | 5 | miniextendr-api/src/serde/ser.rs:359 |

## `AltLogicalData` — 7 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[bool; N]` | `<N>` | concrete | 2 | miniextendr-api/src/altrep_data/builtins.rs:1053 |
| `Vec<bool>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:564 |
| `Box<[bool]>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:593 |
| `&[bool]` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:965 |
| `SparseIterLogicalData<I>` | `<I>` | concrete | 2 | miniextendr-api/src/altrep_data/iter/sparse.rs:310 |
| `IterLogicalData<I>` | `<I>` | concrete | 2 | miniextendr-api/src/altrep_data/iter/state.rs:364 |
| `BooleanArray` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:1688 |

## `RSerializeNative` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:73 |

## `AsRef` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1637 |
| `RPrimitive<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:158 |
| `RPrimitive<T>` | `<T>` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:165 |
| `RStringArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:242 |
| `RStringArray` | `` | concrete | 1 | miniextendr-api/src/optionals/arrow_impl.rs:249 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:241 |

## `RSerialize` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/serde_impl.rs:228 |

## `Deserializer` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CellDeserializer<'de, '_>` | `<'de>` | concrete | 30 | miniextendr-api/src/serde/dataframe_de.rs:1086 |
| `RowDeserializer<'de>` | `<'de>` | concrete | 30 | miniextendr-api/src/serde/dataframe_de.rs:583 |
| `MaybeNestedDeserializer<'de>` | `<'de>` | concrete | 30 | miniextendr-api/src/serde/dataframe_de.rs:829 |
| `StrDeserializer<'a>` | `<'de, 'a>` | concrete | 30 | miniextendr-api/src/serde/de.rs:1046 |
| `VectorElementDeserializer` | `<'de>` | concrete | 30 | miniextendr-api/src/serde/de.rs:770 |
| `RDeserializer` | `<'de>` | concrete | 30 | miniextendr-api/src/serde/de.rs:89 |

## `AltComplexData` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1096 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1103 |
| `[crate::Rcomplex; N]` | `<N>` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1111 |
| `std::borrow::Cow<'static, [crate::Rcomplex]>` | `` | concrete | 3 | miniextendr-api/src/altrep_data/builtins.rs:1202 |
| `IterComplexData<I>` | `<I> +1wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/coerce.rs:467 |
| `SparseIterComplexData<I>` | `<I> +1wc` | concrete | 3 | miniextendr-api/src/altrep_data/iter/sparse.rs:407 |

## `AltRaw` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[u8; N]` | `<N>` | concrete | 4 | miniextendr-api/src/altrep_impl/arrays.rs:134 |
| `Vec<u8>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:360 |
| `Box<[u8]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:457 |
| `std::borrow::Cow<'static, [u8]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:495 |
| `&'static [u8]` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/static_slices.rs:318 |
| `UInt8Array` | `` | concrete | 4 | miniextendr-api/src/optionals/arrow_impl.rs:1866 |

## `RNdArrayOps` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Array1<f64>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:2013 |
| `Array2<f64>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:2071 |
| `ArrayD<f64>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:2129 |
| `Array1<i32>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:2189 |
| `Array2<i32>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:2254 |
| `ArrayD<i32>` | `` | concrete | 11 | miniextendr-api/src/optionals/ndarray_impl.rs:2319 |

## `AltLogical` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[bool; N]` | `<N>` | concrete | 4 | miniextendr-api/src/altrep_impl/arrays.rs:162 |
| `Vec<bool>` | `` | concrete | 10 | miniextendr-api/src/altrep_impl/builtins.rs:353 |
| `Box<[bool]>` | `` | concrete | 10 | miniextendr-api/src/altrep_impl/builtins.rs:450 |
| `&'static [bool]` | `` | concrete | 6 | miniextendr-api/src/altrep_impl/static_slices.rs:245 |
| `BooleanArray` | `` | concrete | 10 | miniextendr-api/src/optionals/arrow_impl.rs:1867 |

## `AltrepExtract` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/altrep_data/core.rs:336 |

## `RPartialOrd` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:228 |

## `ExactSizeIterator` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1765 |
| `StrVecIter<'_>` | `` | concrete | 0 | miniextendr-api/src/strvec.rs:310 |
| `StrVecCowIter<'_>` | `` | concrete | 0 | miniextendr-api/src/strvec.rs:345 |
| `ProtectedStrVecIter<'_>` | `` | concrete | 0 | miniextendr-api/src/strvec.rs:686 |
| `ProtectedStrVecCowIter<'_>` | `` | concrete | 0 | miniextendr-api/src/strvec.rs:715 |

## `Serialize` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `VariantPayload<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:2900 |
| `TaggedVariantRow<'_, T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:3519 |
| `FlattenEnumFieldsRow<'_, T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:4002 |
| `MapEntry<'_, K, V>` | `<K, V>` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:4426 |
| `CollatedResultRow<'_, T, E>` | `<T, E>` | concrete | 1 | miniextendr-api/src/serde/columnar.rs:4613 |

## `IntoList` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/list.rs:713 |
| `std::collections::HashMap<K, V>` | `<K, V> +2wc` | concrete | 1 | miniextendr-api/src/list.rs:764 |
| `std::collections::BTreeMap<K, V>` | `<K, V> +2wc` | concrete | 1 | miniextendr-api/src/list.rs:821 |
| `std::collections::HashSet<T>` | `<T> +1wc` | concrete | 1 | miniextendr-api/src/list.rs:878 |
| `std::collections::BTreeSet<T>` | `<T> +1wc` | concrete | 1 | miniextendr-api/src/list.rs:903 |

## `ROrd` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:187 |

## `Iterator` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 4 | miniextendr-api/src/externalptr.rs:1737 |
| `StrVecIter<'a>` | `<'a>` | concrete | 3 | miniextendr-api/src/strvec.rs:286 |
| `StrVecCowIter<'a>` | `<'a>` | concrete | 3 | miniextendr-api/src/strvec.rs:321 |
| `ProtectedStrVecIter<'a>` | `<'a>` | concrete | 3 | miniextendr-api/src/strvec.rs:664 |
| `ProtectedStrVecCowIter<'a>` | `<'a>` | concrete | 3 | miniextendr-api/src/strvec.rs:695 |

## `RNativeType` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `i32` | `` | concrete | 4 | miniextendr-api/src/sexp_types.rs:209 |
| `f64` | `` | concrete | 4 | miniextendr-api/src/sexp_types.rs:230 |
| `u8` | `` | concrete | 4 | miniextendr-api/src/sexp_types.rs:251 |
| `RLogical` | `` | concrete | 4 | miniextendr-api/src/sexp_types.rs:273 |
| `Rcomplex` | `` | concrete | 4 | miniextendr-api/src/sexp_types.rs:295 |

## `SeqAccess` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `EnumTupleSeqAccess<'de>` | `<'de>` | concrete | 3 | miniextendr-api/src/serde/dataframe_de.rs:1605 |
| `EmptySeqAccess` | `<'de>` | concrete | 2 | miniextendr-api/src/serde/de.rs:675 |
| `VectorSeqAccess` | `<'de>` | concrete | 2 | miniextendr-api/src/serde/de.rs:705 |
| `VectorElementSeqAccess` | `<'de>` | concrete | 3 | miniextendr-api/src/serde/de.rs:890 |
| `ListSeqAccess` | `<'de>` | concrete | 2 | miniextendr-api/src/serde/de.rs:960 |

## `TryFromList` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<T>` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/list.rs:734 |
| `std::collections::HashMap<String, V>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/list.rs:775 |
| `std::collections::BTreeMap<String, V>` | `<V> +1wc` | concrete | 2 | miniextendr-api/src/list.rs:832 |
| `std::collections::HashSet<T>` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/list.rs:888 |
| `std::collections::BTreeSet<T>` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/list.rs:913 |

## `SerializeTupleVariant` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `NoopTupleVariant` | `` | concrete | 4 | miniextendr-api/src/serde/columnar.rs:2662 |
| `VariantAsTupleStruct<S>` | `<S>` | concrete | 4 | miniextendr-api/src/serde/columnar.rs:2929 |
| `ForwardingMapEmitter<'_, M>` | `<M>` | concrete | 4 | miniextendr-api/src/serde/columnar.rs:3842 |
| `TupleVariantSerializer` | `` | concrete | 4 | miniextendr-api/src/serde/ser.rs:322 |

## `AltComplex` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `[crate::Rcomplex; N]` | `<N>` | concrete | 4 | miniextendr-api/src/altrep_impl/arrays.rs:142 |
| `Vec<crate::Rcomplex>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:398 |
| `Box<[crate::Rcomplex]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:471 |
| `std::borrow::Cow<'static, [crate::Rcomplex]>` | `` | concrete | 4 | miniextendr-api/src/altrep_impl/builtins.rs:502 |

## `TryRng` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RRng` | `` | concrete | 4 | miniextendr-api/src/optionals/rand_impl.rs:129 |

## `PartialOrd` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:919 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1716 |
| `RWrapperPriority` | `` | concrete | 1 | miniextendr-api/src/registry.rs:209 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:224 |

## `SerializeStructVariant` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `NoopStructVariant` | `` | concrete | 4 | miniextendr-api/src/serde/columnar.rs:2646 |
| `VariantAsStruct<S>` | `<S>` | concrete | 4 | miniextendr-api/src/serde/columnar.rs:2912 |
| `ForwardingMapEmitter<'_, M>` | `<M>` | concrete | 4 | miniextendr-api/src/serde/columnar.rs:3784 |
| `StructVariantSerializer` | `` | concrete | 4 | miniextendr-api/src/serde/ser.rs:429 |

## `Ord` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Coerced<T, R>` | `<T, R>` | concrete | 1 | miniextendr-api/src/coerce.rs:919 |
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1723 |
| `RWrapperPriority` | `` | concrete | 1 | miniextendr-api/src/registry.rs:209 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:224 |

## `WidensToI32` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `i8` | `` | concrete | 0 | miniextendr-api/src/markers.rs:214 |
| `i16` | `` | concrete | 0 | miniextendr-api/src/markers.rs:215 |
| `u8` | `` | concrete | 0 | miniextendr-api/src/markers.rs:216 |
| `u16` | `` | concrete | 0 | miniextendr-api/src/markers.rs:217 |

## `Write` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RStdout` | `` | concrete | 2 | miniextendr-api/src/connection.rs:1309 |
| `RStderr` | `` | concrete | 2 | miniextendr-api/src/connection.rs:1319 |
| `RNullConnection` | `` | concrete | 2 | miniextendr-api/src/connection.rs:1460 |

## `IntoRAltrep` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/into_r.rs:2016 |

## `VariantAccess` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `DfVariantAccess<'de>` | `<'de>` | concrete | 5 | miniextendr-api/src/serde/dataframe_de.rs:1534 |
| `UnitVariantDeserializer` | `<'de>` | concrete | 5 | miniextendr-api/src/serde/de.rs:1083 |
| `DataVariantDeserializer` | `<'de>` | concrete | 5 | miniextendr-api/src/serde/de.rs:1143 |

## `AsNamedListExt` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<(K, V)>` | `<K, V>` | concrete | 0 | miniextendr-api/src/convert.rs:919 |
| `[(K, V); N]` | `<K, V, N>` | concrete | 0 | miniextendr-api/src/convert.rs:920 |
| `&[(K, V)]` | `<K, V>` | concrete | 0 | miniextendr-api/src/convert.rs:921 |

## `AsNamedVectorExt` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<(K, V)>` | `<K, V>` | concrete | 0 | miniextendr-api/src/convert.rs:938 |
| `[(K, V); N]` | `<K, V, N>` | concrete | 0 | miniextendr-api/src/convert.rs:939 |
| `&[(K, V)]` | `<K, V>` | concrete | 0 | miniextendr-api/src/convert.rs:940 |

## `AsRNativeExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:875 |

## `EnumAccess` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `DfEnumAccess<'de>` | `<'de>` | concrete | 3 | miniextendr-api/src/serde/dataframe_de.rs:1481 |
| `UnitVariantAccess` | `<'de>` | concrete | 3 | miniextendr-api/src/serde/de.rs:1068 |
| `DataVariantAccess` | `<'de>` | concrete | 3 | miniextendr-api/src/serde/de.rs:1126 |

## `RNdSlice2D` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Array2<f64>` | `` | concrete | 7 | miniextendr-api/src/optionals/ndarray_impl.rs:2558 |
| `Array2<i32>` | `` | concrete | 7 | miniextendr-api/src/optionals/ndarray_impl.rs:2606 |

## `AsMut` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1644 |
| `AsSerialize<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:248 |

## `Protector` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ProtectScope` | `` | concrete | 1 | miniextendr-api/src/gc_protect.rs:219 |
| `crate::protect_pool::ProtectPool` | `` | concrete | 1 | miniextendr-api/src/gc_protect.rs:226 |

## `IntoDataFrame` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/dataframe.rs:1234 |
| `SerdeRows<T>` | `<T>` | concrete | 1 | miniextendr-api/src/serde/dataframe_de.rs:429 |

## `FromDataFrame` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/dataframe.rs:1250 |
| `SerdeRows<T>` | `<T> +1wc` | concrete | 1 | miniextendr-api/src/serde/dataframe_de.rs:444 |

## `RSourced` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RPrimitive<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:141 |
| `RStringArray` | `` | concrete | 2 | miniextendr-api/src/optionals/arrow_impl.rs:223 |

## `RNdIndex` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ArrayD<f64>` | `` | concrete | 10 | miniextendr-api/src/optionals/ndarray_impl.rs:2732 |
| `ArrayD<i32>` | `` | concrete | 10 | miniextendr-api/src/optionals/ndarray_impl.rs:2886 |

## `RNdSlice` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Array1<f64>` | `` | concrete | 5 | miniextendr-api/src/optionals/ndarray_impl.rs:2445 |
| `Array1<i32>` | `` | concrete | 5 | miniextendr-api/src/optionals/ndarray_impl.rs:2478 |

## `MapAccess` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RowMapAccess<'de>` | `<'de>` | concrete | 3 | miniextendr-api/src/serde/dataframe_de.rs:747 |
| `NamedListMapAccess` | `<'de>` | concrete | 3 | miniextendr-api/src/serde/de.rs:1009 |

## `AltrepClass` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `JiffTimestampVec` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffZonedVec` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:915 |

## `Storage` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RVecStorage<T, nalgebra::base::dimension::Dyn, nalgebra::base::dimension::Dyn>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1251 |
| `RVecStorage<T, nalgebra::base::dimension::Dyn, nalgebra::base::dimension::U1>` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/nalgebra_impl.rs:1276 |

## `AsDataFrameExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:888 |

## `RDateTimeFormat` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `OffsetDateTime` | `` | concrete | 2 | miniextendr-api/src/optionals/time_impl.rs:287 |
| `Date` | `` | concrete | 2 | miniextendr-api/src/optionals/time_impl.rs:299 |

## `UnitEnumFactor` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 3 | miniextendr-api/src/factor.rs:560 |

## `ROrderedFloatOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `OrderedFloat<T>` | `<T> +1wc` | concrete | 16 | miniextendr-api/src/optionals/ordered_float_impl.rs:344 |

## `RawStorageMut` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RVecStorage<T, nalgebra::base::dimension::Dyn, C>` | `<T, C>` | concrete | 2 | miniextendr-api/src/optionals/nalgebra_impl.rs:1230 |

## `RDate` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Date` | `` | concrete | 10 | miniextendr-api/src/optionals/jiff_impl.rs:828 |

## `RTomlOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `TomlValue` | `` | concrete | 16 | miniextendr-api/src/optionals/toml_impl.rs:624 |

## `IsContiguous` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RVecStorage<T, nalgebra::base::dimension::Dyn, C>` | `<T, C>` | concrete | 0 | miniextendr-api/src/optionals/nalgebra_impl.rs:1301 |

## `RDeserialize` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/serde_impl.rs:275 |

## `Log` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RLogger` | `` | concrete | 3 | miniextendr-api/src/optionals/log_impl.rs:162 |

## `SerializeTupleStruct` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SeqSerializer` | `` | concrete | 4 | miniextendr-api/src/serde/ser.rs:303 |

## `RVectorOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `DVector<f64>` | `` | concrete | 18 | miniextendr-api/src/optionals/nalgebra_impl.rs:495 |

## `SexpExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `crate::SEXP` | `` | concrete | 85 | miniextendr-api/src/sexp_ext.rs:523 |

## `BitOr` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RFlags<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:152 |

## `RRegexOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Regex` | `` | concrete | 7 | miniextendr-api/src/optionals/regex_impl.rs:150 |

## `RToVec` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `C` | `<C, T> +3wc` | concrete | 2 | miniextendr-api/src/adapter_traits.rs:822 |

## `Pod` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RawHeader` | `` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:115 |

## `RUrlOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Url` | `` | concrete | 12 | miniextendr-api/src/optionals/url_impl.rs:138 |

## `PairListExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `crate::SEXP` | `` | concrete | 14 | miniextendr-api/src/sexp_ext.rs:1122 |

## `RZoned` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Zoned` | `` | concrete | 10 | miniextendr-api/src/optionals/jiff_impl.rs:774 |

## `RSigned` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/optionals/num_traits_impl.rs:139 |

## `GlobalAlloc` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RAllocator` | `` | concrete | 4 | miniextendr-api/src/allocator.rs:146 |

## `IntoDataFrameSplit` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Vec<T>` | `<T>` | concrete | 1 | miniextendr-api/src/dataframe.rs:1282 |

## `RFloat` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T> +1wc` | concrete | 41 | miniextendr-api/src/optionals/num_traits_impl.rs:354 |

## `RawStorage` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RVecStorage<T, nalgebra::base::dimension::Dyn, C>` | `<T, C>` | concrete | 7 | miniextendr-api/src/optionals/nalgebra_impl.rs:1191 |

## `RAhoCorasickOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `AhoCorasick` | `` | concrete | 6 | miniextendr-api/src/optionals/aho_corasick_impl.rs:305 |

## `RDateTime` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `DateTime` | `` | concrete | 10 | miniextendr-api/src/optionals/jiff_impl.rs:655 |

## `RCaptureGroups` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CaptureGroups` | `` | concrete | 5 | miniextendr-api/src/optionals/regex_impl.rs:250 |

## `RIndexMapOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IndexMap<String, T>` | `<T>` | concrete | 9 | miniextendr-api/src/optionals/indexmap_impl.rs:208 |

## `RDistributions` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RRng` | `` | concrete | 4 | miniextendr-api/src/optionals/rand_impl.rs:232 |

## `RTime` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Time` | `` | concrete | 5 | miniextendr-api/src/optionals/jiff_impl.rs:703 |

## `Read` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RStdin` | `` | concrete | 1 | miniextendr-api/src/connection.rs:1252 |

## `TermLike` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RTerm` | `` | concrete | 9 | miniextendr-api/src/progress.rs:246 |

## `RSignedDuration` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SignedDuration` | `` | concrete | 10 | miniextendr-api/src/optionals/jiff_impl.rs:523 |

## `RBorshOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 3 | miniextendr-api/src/optionals/borsh_impl.rs:91 |

## `AltListData` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `IterListData<I>` | `<I> +1wc` | concrete | 1 | miniextendr-api/src/altrep_data/iter/coerce.rs:401 |

## `RTimestamp` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Timestamp` | `` | concrete | 5 | miniextendr-api/src/optionals/jiff_impl.rs:736 |

## `RDecimalOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Decimal` | `` | concrete | 22 | miniextendr-api/src/optionals/rust_decimal_impl.rs:400 |

## `BitAnd` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RFlags<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:145 |

## `AsListExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:830 |

## `MatchArg` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `log::LevelFilter` | `` | concrete | 3 | miniextendr-api/src/optionals/log_impl.rs:283 |

## `AsExternalPtrExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:854 |

## `Not` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RFlags<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:166 |

## `RDuration` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Duration` | `` | concrete | 10 | miniextendr-api/src/optionals/time_impl.rs:189 |

## `RBigUintBitOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `BigUint` | `` | concrete | 10 | miniextendr-api/src/optionals/num_bigint_impl.rs:691 |

## `RSpan` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Span` | `` | concrete | 14 | miniextendr-api/src/optionals/jiff_impl.rs:587 |

## `FusedIterator` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 0 | miniextendr-api/src/externalptr.rs:1773 |

## `RJsonValueOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `JsonValue` | `` | concrete | 15 | miniextendr-api/src/optionals/serde_impl.rs:937 |

## `RFromStr` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:349 |

## `RJsonBridge` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/serde_impl.rs:1049 |

## `RBigUintOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `BigUint` | `` | concrete | 13 | miniextendr-api/src/optionals/num_bigint_impl.rs:449 |

## `RMatrixOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `DMatrix<f64>` | `` | concrete | 24 | miniextendr-api/src/optionals/nalgebra_impl.rs:687 |

## `AltrepSexpExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `crate::SEXP` | `` | concrete | 4 | miniextendr-api/src/altrep_ext.rs:54 |

## `SerializeSeq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SeqSerializer` | `` | concrete | 4 | miniextendr-api/src/serde/ser.rs:256 |

## `RUuidOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Uuid` | `` | concrete | 8 | miniextendr-api/src/optionals/uuid_impl.rs:113 |

## `SerializeTuple` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `SeqSerializer` | `` | concrete | 4 | miniextendr-api/src/serde/ser.rs:289 |

## `RDeserializeNative` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:146 |

## `ParCollectR` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 0 | miniextendr-api/src/optionals/rayon_bridge.rs:1378 |

## `RComplexOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Complex<f64>` | `` | concrete | 11 | miniextendr-api/src/optionals/num_complex_impl.rs:329 |

## `RBigIntBitOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `BigInt` | `` | concrete | 11 | miniextendr-api/src/optionals/num_bigint_impl.rs:585 |

## `RFromIter` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `C` | `<C, T> +1wc` | concrete | 1 | miniextendr-api/src/adapter_traits.rs:751 |

## `Pointer` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 1 | miniextendr-api/src/externalptr.rs:1701 |

## `BitXor` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RFlags<T>` | `<T>` | concrete | 2 | miniextendr-api/src/optionals/bitflags_impl.rs:159 |

## `RNum` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T> +1wc` | concrete | 4 | miniextendr-api/src/optionals/num_traits_impl.rs:78 |

## `Zeroable` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `RawHeader` | `` | concrete | 0 | miniextendr-api/src/raw_conversions.rs:115 |

## `AsVctrsExt` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 0 | miniextendr-api/src/convert.rs:902 |

## `RBigIntOps` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `BigInt` | `` | concrete | 17 | miniextendr-api/src/optionals/num_bigint_impl.rs:309 |

## `IntoRVecElement` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/match_arg.rs:281 |

## `DoubleEndedIterator` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ExternalPtr<T>` | `<T>` | concrete | 2 | miniextendr-api/src/externalptr.rs:1753 |
