//! `Option<scalar>` fields in `#[derive(DataFrameRow)]` structs (#1437).
//!
//! `Option<T>` for a scalar `T` is the NA contract: `None` is written as the
//! typed `NA` (`NA_real_`, `NA_character_`, `NA`, `NA_integer_`) and read back
//! as `None`. The derive used to reject every `Option<…>` field (the #484 guard
//! against `Option<Map>` / `Option<Struct>` silently becoming a list-column);
//! that guard now applies only to non-scalar payloads.
//!
//! The pair `df_option_scalar_rows()` / `df_option_scalar_roundtrip(df)` proves
//! writer and reader agree; `df_option_scalar_none_count(df)` looks at the
//! `None`s from the Rust side so the reader's NA handling is pinned directly.

use miniextendr_api::dataframe::{BuiltDataFrame, DataFrame, FromDataFrame, IntoDataFrame};
use miniextendr_api::{DataFrameRow, IntoList, miniextendr};

/// One observation with an NA-able column per scalar kind.
#[derive(Clone, Debug, PartialEq, IntoList, DataFrameRow)]
pub struct OptObs {
    pub id: i32,
    pub weight: Option<f64>,
    pub label: Option<String>,
    pub flag: Option<bool>,
    pub count: Option<i32>,
}

fn sample_rows() -> Vec<OptObs> {
    vec![
        OptObs {
            id: 1,
            weight: Some(1.5),
            label: Some("a".into()),
            flag: Some(true),
            count: Some(10),
        },
        OptObs {
            id: 2,
            weight: None,
            label: None,
            flag: None,
            count: None,
        },
        OptObs {
            id: 3,
            weight: Some(-2.25),
            label: Some("".into()),
            flag: Some(false),
            count: Some(-7),
        },
    ]
}

/// Three rows; the middle one is `None` in every optional column.
#[miniextendr]
pub fn df_option_scalar_rows() -> BuiltDataFrame {
    sample_rows().into_dataframe().unwrap()
}

/// Read `df` into `Vec<OptObs>` with the generated reader and write it back.
///
/// @param df A data.frame with the `id`, `weight`, `label`, `flag`, `count` columns.
#[miniextendr]
pub fn df_option_scalar_roundtrip(df: DataFrame) -> BuiltDataFrame {
    Vec::<OptObs>::from_dataframe(&df)
        .unwrap()
        .into_dataframe()
        .unwrap()
}

/// Number of `None` cells across the four optional columns, counted in Rust.
///
/// @param df A data.frame with the `id`, `weight`, `label`, `flag`, `count` columns.
#[miniextendr]
pub fn df_option_scalar_none_count(df: DataFrame) -> i32 {
    Vec::<OptObs>::from_dataframe(&df)
        .unwrap()
        .iter()
        .map(|r| {
            i32::from(r.weight.is_none())
                + i32::from(r.label.is_none())
                + i32::from(r.flag.is_none())
                + i32::from(r.count.is_none())
        })
        .sum()
}
