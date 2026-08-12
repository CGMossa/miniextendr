# Trait impl inventory

Source: `target/doc/miniextendr_api.json`

Traits with impls: 9

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `TryFromSexp` | 468 | 468 |
| `IntoR` | 353 | 353 |
| `IntoRAs` | 135 | 135 |
| `TryCoerce` | 95 | 93 |
| `Coerce` | 53 | 53 |
| `AltrepSerialize` | 27 | 27 |
| `RSerializeNative` | 6 | 1 |
| `IntoRAltrep` | 3 | 1 |
| `RDeserializeNative` | 1 | 1 |

## `TryFromSexp` — 468 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `AltrepSexp` | `` | concrete | 3 | miniextendr-api/src/altrep_sexp.rs:282 |
| `AsFromStr<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/convert.rs:1025 |
| `AsFromStrVec<T>` | `<T> +1wc` | concrete | 3 | miniextendr-api/src/convert.rs:1067 |
| `DataFrame` | `` | concrete | 2 | miniextendr-api/src/dataframe.rs:711 |
| `FactorVec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/factor.rs:321 |
| `FactorOptionVec<T>` | `<T>` | concrete | 2 | miniextendr-api/src/factor.rs:374 |
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
| `Option<std::path::PathBuf>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:289 |
| `Vec<Option<std::path::PathBuf>>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:289 |
| `Vec<std::path::PathBuf>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:289 |
| `std::path::PathBuf` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:289 |
| `Option<std::ffi::OsString>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:312 |
| `Vec<Option<std::ffi::OsString>>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:312 |
| `Vec<std::ffi::OsString>` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:312 |
| `std::ffi::OsString` | `` | concrete | 3 | miniextendr-api/src/from_r/cow_and_paths.rs:312 |
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
| `&'static i32` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:321 |
| `&'static mut i32` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:321 |
| `Option<&'static i32>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:321 |
| `Option<&'static mut i32>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<&'static [i32]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<&'static i32>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<&'static mut [i32]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<&'static mut i32>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<Option<&'static [i32]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<Option<&'static i32>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<Option<&'static mut [i32]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `Vec<Option<&'static mut i32>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:321 |
| `&'static f64` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:322 |
| `&'static mut f64` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:322 |
| `Option<&'static f64>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:322 |
| `Option<&'static mut f64>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<&'static [f64]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<&'static f64>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<&'static mut [f64]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<&'static mut f64>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<Option<&'static [f64]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<Option<&'static f64>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<Option<&'static mut [f64]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `Vec<Option<&'static mut f64>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:322 |
| `&'static mut u8` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:323 |
| `&'static u8` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:323 |
| `Option<&'static mut u8>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:323 |
| `Option<&'static u8>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<&'static [u8]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<&'static mut [u8]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<&'static mut u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<&'static u8>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<Option<&'static [u8]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<Option<&'static mut [u8]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<Option<&'static mut u8>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `Vec<Option<&'static u8>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:323 |
| `&'static crate::RLogical` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:324 |
| `&'static mut crate::RLogical` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:324 |
| `Option<&'static crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:324 |
| `Option<&'static mut crate::RLogical>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<&'static [crate::RLogical]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<&'static crate::RLogical>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<&'static mut [crate::RLogical]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<&'static mut crate::RLogical>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<Option<&'static [crate::RLogical]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<Option<&'static crate::RLogical>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<Option<&'static mut [crate::RLogical]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `Vec<Option<&'static mut crate::RLogical>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:324 |
| `&'static crate::Rcomplex` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:325 |
| `&'static mut crate::Rcomplex` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:325 |
| `Option<&'static crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:325 |
| `Option<&'static mut crate::Rcomplex>` | `` | concrete | 3 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<&'static [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<&'static crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<&'static mut [crate::Rcomplex]>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<&'static mut crate::Rcomplex>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<Option<&'static [crate::Rcomplex]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<Option<&'static crate::Rcomplex>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<Option<&'static mut [crate::Rcomplex]>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
| `Vec<Option<&'static mut crate::Rcomplex>>` | `` | concrete | 2 | miniextendr-api/src/from_r/references.rs:325 |
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
| `Option<SignedDuration>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `SignedDuration` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `Vec<Option<SignedDuration>>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `Vec<SignedDuration>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `Option<Timestamp>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Timestamp` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Vec<Option<Timestamp>>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Vec<Timestamp>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `JiffTimestampVecMut` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffTimestampVecRef` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffZonedVecMut` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `JiffZonedVecRef` | `` | concrete | 2 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `Date` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Option<Date>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Vec<Date>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Vec<Option<Date>>` | `` | concrete | 3 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
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
| `Array2<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array3<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array4<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array5<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array6<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `ArrayD<i8>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:668 |
| `Array1<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array2<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array3<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array4<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array5<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array6<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `ArrayD<i16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:669 |
| `Array1<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array2<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array3<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array4<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array5<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array6<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `ArrayD<i64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:670 |
| `Array1<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array2<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array3<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array4<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array5<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array6<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `ArrayD<isize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:671 |
| `Array1<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array2<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array3<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array4<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array5<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array6<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `ArrayD<u16>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:672 |
| `Array1<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:673 |
| `Array2<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:673 |
| `Array3<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:673 |
| `Array4<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:673 |
| `Array5<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:673 |
| `Array6<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:673 |
| `ArrayD<u32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:673 |
| `Array1<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:674 |
| `Array2<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:674 |
| `Array3<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:674 |
| `Array4<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:674 |
| `Array5<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:674 |
| `Array6<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:674 |
| `ArrayD<u64>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:674 |
| `Array1<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:675 |
| `Array2<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:675 |
| `Array3<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:675 |
| `Array4<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:675 |
| `Array5<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:675 |
| `Array6<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:675 |
| `ArrayD<usize>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:675 |
| `Array1<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:679 |
| `Array2<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:679 |
| `Array3<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:679 |
| `Array4<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:679 |
| `Array5<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:679 |
| `Array6<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:679 |
| `ArrayD<f32>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:679 |
| `Array1<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:683 |
| `Array2<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:683 |
| `Array3<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:683 |
| `Array4<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:683 |
| `Array5<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:683 |
| `Array6<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:683 |
| `ArrayD<bool>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:683 |
| `Array0<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `Array1<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `Array2<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `Array3<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `Array4<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `Array5<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `Array6<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `ArrayD<String>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:866 |
| `Array0<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `Array1<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `Array2<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `Array3<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `Array4<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `Array5<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `Array6<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
| `ArrayD<Option<String>>` | `` | concrete | 3 | miniextendr-api/src/optionals/ndarray_impl.rs:867 |
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
| `Regex` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:67 |
| `Vec<Option<Regex>>` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:67 |
| `Vec<Regex>` | `` | concrete | 2 | miniextendr-api/src/optionals/regex_impl.rs:67 |
| `Decimal` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:146 |
| `Option<Decimal>` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:179 |
| `Vec<Decimal>` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:206 |
| `Vec<Option<Decimal>>` | `` | concrete | 2 | miniextendr-api/src/optionals/rust_decimal_impl.rs:251 |
| `JsonValue` | `` | concrete | 2 | miniextendr-api/src/optionals/serde_impl.rs:578 |
| `Option<JsonValue>` | `` | concrete | 3 | miniextendr-api/src/optionals/serde_impl.rs:590 |
| `Vec<JsonValue>` | `` | concrete | 3 | miniextendr-api/src/optionals/serde_impl.rs:591 |
| `Vec<Option<JsonValue>>` | `` | concrete | 3 | miniextendr-api/src/optionals/serde_impl.rs:592 |
| `OffsetDateTime` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Option<OffsetDateTime>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Vec<OffsetDateTime>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Vec<Option<OffsetDateTime>>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Date` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Option<Date>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Vec<Date>` | `` | concrete | 3 | miniextendr-api/src/optionals/time_impl.rs:97 |
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
| `Url` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:48 |
| `Vec<Option<Url>>` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:48 |
| `Vec<Url>` | `` | concrete | 2 | miniextendr-api/src/optionals/url_impl.rs:48 |
| `Option<Uuid>` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:43 |
| `Uuid` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:43 |
| `Vec<Option<Uuid>>` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:43 |
| `Vec<Uuid>` | `` | concrete | 2 | miniextendr-api/src/optionals/uuid_impl.rs:43 |
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

- **miniextendr-api/src/from_r/references.rs:321** (12 impls): `&'static i32`, `&'static mut i32`, `Option<&'static i32>`, `Option<&'static mut i32>`, `Vec<&'static [i32]>`, `Vec<&'static i32>`, `Vec<&'static mut [i32]>`, `Vec<&'static mut i32>`, `Vec<Option<&'static [i32]>>`, `Vec<Option<&'static i32>>`, `Vec<Option<&'static mut [i32]>>`, `Vec<Option<&'static mut i32>>`
- **miniextendr-api/src/from_r/references.rs:322** (12 impls): `&'static f64`, `&'static mut f64`, `Option<&'static f64>`, `Option<&'static mut f64>`, `Vec<&'static [f64]>`, `Vec<&'static f64>`, `Vec<&'static mut [f64]>`, `Vec<&'static mut f64>`, `Vec<Option<&'static [f64]>>`, `Vec<Option<&'static f64>>`, `Vec<Option<&'static mut [f64]>>`, `Vec<Option<&'static mut f64>>`
- **miniextendr-api/src/from_r/references.rs:323** (12 impls): `&'static mut u8`, `&'static u8`, `Option<&'static mut u8>`, `Option<&'static u8>`, `Vec<&'static [u8]>`, `Vec<&'static mut [u8]>`, `Vec<&'static mut u8>`, `Vec<&'static u8>`, `Vec<Option<&'static [u8]>>`, `Vec<Option<&'static mut [u8]>>`, `Vec<Option<&'static mut u8>>`, `Vec<Option<&'static u8>>`
- **miniextendr-api/src/from_r/references.rs:324** (12 impls): `&'static crate::RLogical`, `&'static mut crate::RLogical`, `Option<&'static crate::RLogical>`, `Option<&'static mut crate::RLogical>`, `Vec<&'static [crate::RLogical]>`, `Vec<&'static crate::RLogical>`, `Vec<&'static mut [crate::RLogical]>`, `Vec<&'static mut crate::RLogical>`, `Vec<Option<&'static [crate::RLogical]>>`, `Vec<Option<&'static crate::RLogical>>`, `Vec<Option<&'static mut [crate::RLogical]>>`, `Vec<Option<&'static mut crate::RLogical>>`
- **miniextendr-api/src/from_r/references.rs:325** (12 impls): `&'static crate::Rcomplex`, `&'static mut crate::Rcomplex`, `Option<&'static crate::Rcomplex>`, `Option<&'static mut crate::Rcomplex>`, `Vec<&'static [crate::Rcomplex]>`, `Vec<&'static crate::Rcomplex>`, `Vec<&'static mut [crate::Rcomplex]>`, `Vec<&'static mut crate::Rcomplex>`, `Vec<Option<&'static [crate::Rcomplex]>>`, `Vec<Option<&'static crate::Rcomplex>>`, `Vec<Option<&'static mut [crate::Rcomplex]>>`, `Vec<Option<&'static mut crate::Rcomplex>>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:866** (8 impls): `Array0<String>`, `Array1<String>`, `Array2<String>`, `Array3<String>`, `Array4<String>`, `Array5<String>`, `Array6<String>`, `ArrayD<String>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:867** (8 impls): `Array0<Option<String>>`, `Array1<Option<String>>`, `Array2<Option<String>>`, `Array3<Option<String>>`, `Array4<Option<String>>`, `Array5<Option<String>>`, `Array6<Option<String>>`, `ArrayD<Option<String>>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:668** (7 impls): `Array1<i8>`, `Array2<i8>`, `Array3<i8>`, `Array4<i8>`, `Array5<i8>`, `Array6<i8>`, `ArrayD<i8>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:669** (7 impls): `Array1<i16>`, `Array2<i16>`, `Array3<i16>`, `Array4<i16>`, `Array5<i16>`, `Array6<i16>`, `ArrayD<i16>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:670** (7 impls): `Array1<i64>`, `Array2<i64>`, `Array3<i64>`, `Array4<i64>`, `Array5<i64>`, `Array6<i64>`, `ArrayD<i64>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:671** (7 impls): `Array1<isize>`, `Array2<isize>`, `Array3<isize>`, `Array4<isize>`, `Array5<isize>`, `Array6<isize>`, `ArrayD<isize>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:672** (7 impls): `Array1<u16>`, `Array2<u16>`, `Array3<u16>`, `Array4<u16>`, `Array5<u16>`, `Array6<u16>`, `ArrayD<u16>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:673** (7 impls): `Array1<u32>`, `Array2<u32>`, `Array3<u32>`, `Array4<u32>`, `Array5<u32>`, `Array6<u32>`, `ArrayD<u32>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:674** (7 impls): `Array1<u64>`, `Array2<u64>`, `Array3<u64>`, `Array4<u64>`, `Array5<u64>`, `Array6<u64>`, `ArrayD<u64>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:675** (7 impls): `Array1<usize>`, `Array2<usize>`, `Array3<usize>`, `Array4<usize>`, `Array5<usize>`, `Array6<usize>`, `ArrayD<usize>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:679** (7 impls): `Array1<f32>`, `Array2<f32>`, `Array3<f32>`, `Array4<f32>`, `Array5<f32>`, `Array6<f32>`, `ArrayD<f32>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:683** (7 impls): `Array1<bool>`, `Array2<bool>`, `Array3<bool>`, `Array4<bool>`, `Array5<bool>`, `Array6<bool>`, `ArrayD<bool>`
- **miniextendr-api/src/from_r/cow_and_paths.rs:289** (4 impls): `Option<std::path::PathBuf>`, `Vec<Option<std::path::PathBuf>>`, `Vec<std::path::PathBuf>`, `std::path::PathBuf`
- **miniextendr-api/src/from_r/cow_and_paths.rs:312** (4 impls): `Option<std::ffi::OsString>`, `Vec<Option<std::ffi::OsString>>`, `Vec<std::ffi::OsString>`, `std::ffi::OsString`
- **miniextendr-api/src/optionals/jiff_impl.rs:476** (4 impls): `Option<SignedDuration>`, `SignedDuration`, `Vec<Option<SignedDuration>>`, `Vec<SignedDuration>`
- **miniextendr-api/src/optionals/jiff_impl.rs:81** (4 impls): `Option<Timestamp>`, `Timestamp`, `Vec<Option<Timestamp>>`, `Vec<Timestamp>`
- **miniextendr-api/src/optionals/jiff_impl.rs:98** (4 impls): `Date`, `Option<Date>`, `Vec<Date>`, `Vec<Option<Date>>`
- **miniextendr-api/src/optionals/num_bigint_impl.rs:193** (4 impls): `BigInt`, `Option<BigInt>`, `Vec<BigInt>`, `Vec<Option<BigInt>>`
- **miniextendr-api/src/optionals/num_bigint_impl.rs:194** (4 impls): `BigUint`, `Option<BigUint>`, `Vec<BigUint>`, `Vec<Option<BigUint>>`
- **miniextendr-api/src/optionals/regex_impl.rs:67** (4 impls): `Option<Regex>`, `Regex`, `Vec<Option<Regex>>`, `Vec<Regex>`
- **miniextendr-api/src/optionals/time_impl.rs:69** (4 impls): `OffsetDateTime`, `Option<OffsetDateTime>`, `Vec<OffsetDateTime>`, `Vec<Option<OffsetDateTime>>`
- **miniextendr-api/src/optionals/time_impl.rs:97** (4 impls): `Date`, `Option<Date>`, `Vec<Date>`, `Vec<Option<Date>>`
- **miniextendr-api/src/optionals/url_impl.rs:48** (4 impls): `Option<Url>`, `Url`, `Vec<Option<Url>>`, `Vec<Url>`
- **miniextendr-api/src/optionals/uuid_impl.rs:43** (4 impls): `Option<Uuid>`, `Uuid`, `Vec<Option<Uuid>>`, `Vec<Uuid>`
- **miniextendr-api/src/optionals/jiff_impl.rs:874** (2 impls): `JiffTimestampVecMut`, `JiffTimestampVecRef`
- **miniextendr-api/src/optionals/jiff_impl.rs:915** (2 impls): `JiffZonedVecMut`, `JiffZonedVecRef`

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
| `FactorVec<T>` | `<T>` | concrete | 4 | miniextendr-api/src/factor.rs:305 |
| `FactorOptionVec<T>` | `<T>` | concrete | 4 | miniextendr-api/src/factor.rs:414 |
| `BTreeSet<i8>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1038 |
| `HashSet<i8>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1038 |
| `BTreeSet<i16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1039 |
| `HashSet<i16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1039 |
| `BTreeSet<u16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1040 |
| `HashSet<u16>` | `` | concrete | 4 | miniextendr-api/src/into_r.rs:1040 |
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
| `&[i8]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:604 |
| `Vec<i8>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:604 |
| `&[i16]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:605 |
| `Vec<i16>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:605 |
| `&[u16]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:606 |
| `Vec<u16>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:606 |
| `&[f32]` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:609 |
| `Vec<f32>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:609 |
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
| `&std::path::Path` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:969 |
| `Option<std::path::PathBuf>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:969 |
| `Vec<Option<std::path::PathBuf>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:969 |
| `Vec<std::path::PathBuf>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:969 |
| `std::path::PathBuf` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:969 |
| `&std::ffi::OsStr` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:985 |
| `Option<std::ffi::OsString>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:985 |
| `Vec<Option<std::ffi::OsString>>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:985 |
| `Vec<std::ffi::OsString>` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:985 |
| `std::ffi::OsString` | `` | concrete | 5 | miniextendr-api/src/into_r.rs:985 |
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
| `Option<SignedDuration>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `SignedDuration` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `Vec<Option<SignedDuration>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `Vec<SignedDuration>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:476 |
| `Option<Timestamp>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Timestamp` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Vec<Option<Timestamp>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `Vec<Timestamp>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:81 |
| `JiffTimestampVec` | `` | concrete | 5 | miniextendr-api/src/optionals/jiff_impl.rs:874 |
| `JiffZonedVec` | `` | concrete | 5 | miniextendr-api/src/optionals/jiff_impl.rs:915 |
| `Date` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Option<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Vec<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
| `Vec<Option<Date>>` | `` | concrete | 4 | miniextendr-api/src/optionals/jiff_impl.rs:98 |
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
| `Array0<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1264 |
| `Array0<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1264 |
| `Array1<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1265 |
| `Array1<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1265 |
| `Array2<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1266 |
| `Array2<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1266 |
| `Array3<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1267 |
| `Array3<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1267 |
| `Array4<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1268 |
| `Array4<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1268 |
| `Array5<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1269 |
| `Array5<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1269 |
| `Array6<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1270 |
| `Array6<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1270 |
| `ArrayD<Option<String>>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1271 |
| `ArrayD<String>` | `` | concrete | 2 | miniextendr-api/src/optionals/ndarray_impl.rs:1271 |
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
| `Option<OffsetDateTime>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Vec<OffsetDateTime>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Vec<Option<OffsetDateTime>>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:69 |
| `Date` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Option<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Vec<Date>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:97 |
| `Vec<Option<Date>>` | `` | concrete | 4 | miniextendr-api/src/optionals/time_impl.rs:97 |
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

- **miniextendr-api/src/into_r.rs:969** (5 impls): `&std::path::Path`, `Option<std::path::PathBuf>`, `Vec<Option<std::path::PathBuf>>`, `Vec<std::path::PathBuf>`, `std::path::PathBuf`
- **miniextendr-api/src/into_r.rs:985** (5 impls): `&std::ffi::OsStr`, `Option<std::ffi::OsString>`, `Vec<Option<std::ffi::OsString>>`, `Vec<std::ffi::OsString>`, `std::ffi::OsString`
- **miniextendr-api/src/optionals/jiff_impl.rs:476** (4 impls): `Option<SignedDuration>`, `SignedDuration`, `Vec<Option<SignedDuration>>`, `Vec<SignedDuration>`
- **miniextendr-api/src/optionals/jiff_impl.rs:81** (4 impls): `Option<Timestamp>`, `Timestamp`, `Vec<Option<Timestamp>>`, `Vec<Timestamp>`
- **miniextendr-api/src/optionals/jiff_impl.rs:98** (4 impls): `Date`, `Option<Date>`, `Vec<Date>`, `Vec<Option<Date>>`
- **miniextendr-api/src/optionals/time_impl.rs:69** (4 impls): `OffsetDateTime`, `Option<OffsetDateTime>`, `Vec<OffsetDateTime>`, `Vec<Option<OffsetDateTime>>`
- **miniextendr-api/src/optionals/time_impl.rs:97** (4 impls): `Date`, `Option<Date>`, `Vec<Date>`, `Vec<Option<Date>>`
- **miniextendr-api/src/into_r.rs:1038** (2 impls): `BTreeSet<i8>`, `HashSet<i8>`
- **miniextendr-api/src/into_r.rs:1039** (2 impls): `BTreeSet<i16>`, `HashSet<i16>`
- **miniextendr-api/src/into_r.rs:1040** (2 impls): `BTreeSet<u16>`, `HashSet<u16>`
- **miniextendr-api/src/into_r.rs:604** (2 impls): `&[i8]`, `Vec<i8>`
- **miniextendr-api/src/into_r.rs:605** (2 impls): `&[i16]`, `Vec<i16>`
- **miniextendr-api/src/into_r.rs:606** (2 impls): `&[u16]`, `Vec<u16>`
- **miniextendr-api/src/into_r.rs:609** (2 impls): `&[f32]`, `Vec<f32>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1264** (2 impls): `Array0<Option<String>>`, `Array0<String>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1265** (2 impls): `Array1<Option<String>>`, `Array1<String>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1266** (2 impls): `Array2<Option<String>>`, `Array2<String>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1267** (2 impls): `Array3<Option<String>>`, `Array3<String>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1268** (2 impls): `Array4<Option<String>>`, `Array4<String>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1269** (2 impls): `Array5<Option<String>>`, `Array5<String>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1270** (2 impls): `Array6<Option<String>>`, `Array6<String>`
- **miniextendr-api/src/optionals/ndarray_impl.rs:1271** (2 impls): `ArrayD<Option<String>>`, `ArrayD<String>`

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
| `&[i8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:695 |
| `Vec<i8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:695 |
| `&[i16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:696 |
| `Vec<i16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:696 |
| `&[u8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:697 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:697 |
| `&[u16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:698 |
| `Vec<u16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:698 |
| `&[i64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:699 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:699 |
| `&[isize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:700 |
| `Vec<isize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:700 |
| `&[u32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:701 |
| `Vec<u32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:701 |
| `&[u64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:702 |
| `Vec<u64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:702 |
| `&[usize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:703 |
| `Vec<usize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:703 |
| `&[f32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:704 |
| `Vec<f32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:704 |
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
| `&[i16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:857 |
| `Vec<i16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:857 |
| `&[u8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:858 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:858 |
| `&[u16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:859 |
| `Vec<u16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:859 |
| `&[u32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:860 |
| `Vec<u32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:860 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:865 |
| `&[i32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:894 |
| `&[i64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:924 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:924 |
| `&[u64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:925 |
| `Vec<u64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:925 |
| `&[isize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:926 |
| `Vec<isize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:926 |
| `&[usize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:927 |
| `Vec<usize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:927 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:930 |
| `&[bool]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:940 |
| `Vec<u8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:951 |
| `&[u8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:958 |
| `&[i8]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:965 |
| `Vec<i8>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:965 |
| `&[i16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:966 |
| `Vec<i16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:966 |
| `&[i32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:967 |
| `Vec<i32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:967 |
| `&[i64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:968 |
| `Vec<i64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:968 |
| `&[isize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:969 |
| `Vec<isize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:969 |
| `&[u16]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:970 |
| `Vec<u16>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:970 |
| `&[u32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:971 |
| `Vec<u32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:971 |
| `&[u64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:972 |
| `Vec<u64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:972 |
| `&[usize]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:973 |
| `Vec<usize>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:973 |
| `&[f32]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:974 |
| `Vec<f32>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:974 |
| `&[f64]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:975 |
| `Vec<f64>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:975 |
| `Vec<bool>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:980 |
| `&[bool]` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:987 |
| `Vec<String>` | `` | concrete | 1 | miniextendr-api/src/into_r_as.rs:997 |

### `IntoRAs` — for-types sharing a source span (likely macro-expanded / co-located)

- **miniextendr-api/src/into_r_as.rs:695** (2 impls): `&[i8]`, `Vec<i8>`
- **miniextendr-api/src/into_r_as.rs:696** (2 impls): `&[i16]`, `Vec<i16>`
- **miniextendr-api/src/into_r_as.rs:697** (2 impls): `&[u8]`, `Vec<u8>`
- **miniextendr-api/src/into_r_as.rs:698** (2 impls): `&[u16]`, `Vec<u16>`
- **miniextendr-api/src/into_r_as.rs:699** (2 impls): `&[i64]`, `Vec<i64>`
- **miniextendr-api/src/into_r_as.rs:700** (2 impls): `&[isize]`, `Vec<isize>`
- **miniextendr-api/src/into_r_as.rs:701** (2 impls): `&[u32]`, `Vec<u32>`
- **miniextendr-api/src/into_r_as.rs:702** (2 impls): `&[u64]`, `Vec<u64>`
- **miniextendr-api/src/into_r_as.rs:703** (2 impls): `&[usize]`, `Vec<usize>`
- **miniextendr-api/src/into_r_as.rs:704** (2 impls): `&[f32]`, `Vec<f32>`
- **miniextendr-api/src/into_r_as.rs:705** (2 impls): `&[f64]`, `Vec<f64>`
- **miniextendr-api/src/into_r_as.rs:856** (2 impls): `&[i8]`, `Vec<i8>`
- **miniextendr-api/src/into_r_as.rs:857** (2 impls): `&[i16]`, `Vec<i16>`
- **miniextendr-api/src/into_r_as.rs:858** (2 impls): `&[u8]`, `Vec<u8>`
- **miniextendr-api/src/into_r_as.rs:859** (2 impls): `&[u16]`, `Vec<u16>`
- **miniextendr-api/src/into_r_as.rs:860** (2 impls): `&[u32]`, `Vec<u32>`
- **miniextendr-api/src/into_r_as.rs:924** (2 impls): `&[i64]`, `Vec<i64>`
- **miniextendr-api/src/into_r_as.rs:925** (2 impls): `&[u64]`, `Vec<u64>`
- **miniextendr-api/src/into_r_as.rs:926** (2 impls): `&[isize]`, `Vec<isize>`
- **miniextendr-api/src/into_r_as.rs:927** (2 impls): `&[usize]`, `Vec<usize>`
- **miniextendr-api/src/into_r_as.rs:965** (2 impls): `&[i8]`, `Vec<i8>`
- **miniextendr-api/src/into_r_as.rs:966** (2 impls): `&[i16]`, `Vec<i16>`
- **miniextendr-api/src/into_r_as.rs:967** (2 impls): `&[i32]`, `Vec<i32>`
- **miniextendr-api/src/into_r_as.rs:968** (2 impls): `&[i64]`, `Vec<i64>`
- **miniextendr-api/src/into_r_as.rs:969** (2 impls): `&[isize]`, `Vec<isize>`
- **miniextendr-api/src/into_r_as.rs:970** (2 impls): `&[u16]`, `Vec<u16>`
- **miniextendr-api/src/into_r_as.rs:971** (2 impls): `&[u32]`, `Vec<u32>`
- **miniextendr-api/src/into_r_as.rs:972** (2 impls): `&[u64]`, `Vec<u64>`
- **miniextendr-api/src/into_r_as.rs:973** (2 impls): `&[usize]`, `Vec<usize>`
- **miniextendr-api/src/into_r_as.rs:974** (2 impls): `&[f32]`, `Vec<f32>`
- **miniextendr-api/src/into_r_as.rs:975** (2 impls): `&[f64]`, `Vec<f64>`

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
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:536 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `i8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:541 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `i8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:551 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:556 |
| `i16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `i32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `i64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `isize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `u16` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `u32` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `u64` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `u8` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
| `usize` | `` | concrete | 2 | miniextendr-api/src/coerce.rs:561 |
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

- **miniextendr-api/src/coerce.rs:541** (9 impls): `i16`, `i32`, `i64`, `i8`, `isize`, `u16`, `u32`, `u64`, `usize`
- **miniextendr-api/src/coerce.rs:561** (9 impls): `i16`, `i32`, `i64`, `isize`, `u16`, `u32`, `u64`, `u8`, `usize`
- **miniextendr-api/src/coerce.rs:551** (8 impls): `i16`, `i32`, `i64`, `i8`, `isize`, `u32`, `u64`, `usize`
- **miniextendr-api/src/coerce.rs:556** (7 impls): `i32`, `i64`, `isize`, `u16`, `u32`, `u64`, `usize`
- **miniextendr-api/src/coerce.rs:536** (5 impls): `i64`, `isize`, `u32`, `u64`, `usize`

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
| `i8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
| `u16` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
| `u8` | `` | concrete | 1 | miniextendr-api/src/coerce.rs:546 |
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
- **miniextendr-api/src/coerce.rs:546** (5 impls): `i8`, `u16`, `u8`, `u8`, `u8`

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

## `RSerializeNative` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:73 |

## `IntoRAltrep` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T> +1wc` | concrete | 2 | miniextendr-api/src/into_r.rs:2016 |

## `RDeserializeNative` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `T` | `<T>` | concrete | 1 | miniextendr-api/src/serde/traits.rs:146 |
