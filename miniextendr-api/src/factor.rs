//! Factor support for enum ↔ R factor conversions.
//!
//! R factors are integer vectors with a `levels` attribute (character vector)
//! and a `class` attribute set to `"factor"`. The integer payload uses 1-based
//! indexing into the levels, with `NA_INTEGER` for missing values.
//!
//! # Usage
//!
//! ```ignore
//! use miniextendr_api::RFactor;
//!
//! #[derive(Copy, Clone, RFactor)]
//! enum Color { Red, Green, Blue }
//!
//! // Enum values convert to/from R factors automatically
//! #[miniextendr]
//! fn describe(c: Color) -> &'static str {
//!     match c {
//!         Color::Red => "red",
//!         Color::Green => "green",
//!         Color::Blue => "blue",
//!     }
//! }
//! ```

use std::ffi::CString;
use std::ops::Deref;
use std::sync::OnceLock;

use crate::altrep_traits::NA_INTEGER;
use crate::from_r::{SexpError, TryFromSexp, charsxp_to_str};
use crate::gc_protect::OwnedProtect;
use crate::into_r::IntoR;
use crate::sys::{Rf_allocVector, Rf_install};
use crate::{SEXP, SEXPTYPE, SexpExt};

// region: Cached "factor" class STRSXP

static FACTOR_CLASS: OnceLock<SEXP> = OnceLock::new();

pub(crate) fn factor_class_sexp() -> SEXP {
    *FACTOR_CLASS.get_or_init(|| unsafe {
        let class_sexp = Rf_allocVector(SEXPTYPE::STRSXP, 1);
        crate::sys::R_PreserveObject(class_sexp);
        // Use symbol PRINTNAME for permanent CHARSXP
        let sym = Rf_install(c"factor".as_ptr());
        class_sexp.set_string_elt(0, sym.printname());
        class_sexp
    })
}
// endregion

// region: RFactor trait

/// Trait for mapping Rust enums to R factors.
///
/// Typically implemented via `#[derive(RFactor)]` for C-style enums.
/// The derive macro also generates `IntoR` and `TryFromSexp` implementations.
pub trait RFactor: crate::match_arg::MatchArg + Copy + 'static {
    /// Convert variant to 1-based level index.
    fn to_level_index(self) -> i32;

    /// Convert 1-based level index to variant, or `None` if out of range.
    fn from_level_index(idx: i32) -> Option<Self>;
}
// endregion

// region: Core building functions

/// Build a levels STRSXP using symbol PRINTNAMEs for permanent CHARSXP protection.
///
/// The returned STRSXP is NOT protected - caller must protect or preserve it.
pub fn build_levels_sexp(levels: &[&str]) -> SEXP {
    unsafe {
        let sexp = Rf_allocVector(SEXPTYPE::STRSXP, levels.len() as isize);
        for (i, level) in levels.iter().enumerate() {
            // Install as symbol - symbols and their PRINTNAMEs are never GC'd
            let c_str = CString::new(*level).expect("level name contains null byte");
            let sym = Rf_install(c_str.as_ptr());
            sexp.set_string_elt(i as isize, sym.printname());
        }
        sexp
    }
}

/// Build a levels STRSXP and preserve it permanently (for caching).
pub fn build_levels_sexp_cached(levels: &[&str]) -> SEXP {
    unsafe {
        let sexp = build_levels_sexp(levels);
        crate::sys::R_PreserveObject(sexp);
        sexp
    }
}

/// Build a factor SEXP from indices and a levels STRSXP.
pub fn build_factor(indices: &[i32], levels: SEXP) -> SEXP {
    unsafe {
        let (sexp, dst) = crate::into_r::alloc_r_vector::<i32>(indices.len());
        dst.copy_from_slice(indices);
        sexp.set_levels(levels);
        sexp.set_class(factor_class_sexp());
        sexp
    }
}

/// Build a factor SEXP from indices and level names in a single call.
///
/// Builds the levels STRSXP via [`build_levels_sexp`] and protects it
/// across the [`build_factor`] allocation, so callers don't need to manage
/// the levels protection themselves. The returned factor SEXP is **not**
/// protected — caller must protect or return it.
///
/// This is the recommended path for one-shot factor construction; for
/// repeated calls with the same levels prefer caching via
/// [`build_levels_sexp_cached`] (no protection needed because the cached
/// SEXP is on R's precious list).
///
/// See CLAUDE.md "PROTECT discipline against R-devel GC" for why this
/// matters even though `build_levels_sexp` uses symbol PRINTNAMEs for the
/// per-element CHARSXPs — the container STRSXP itself is freshly allocated
/// and unprotected.
pub fn build_factor_with_levels(indices: &[i32], level_names: &[&str]) -> SEXP {
    unsafe {
        let levels = OwnedProtect::new(build_levels_sexp(level_names));
        build_factor(indices, levels.get())
    }
}
// endregion

// region: Validation helper

/// Validate that a factor has the expected levels.
pub(crate) fn validate_factor_levels(sexp: SEXP, expected: &[&str]) -> Result<(), SexpError> {
    if !sexp.is_factor() {
        return Err(SexpError::InvalidValue("expected a factor".into()));
    }

    let levels = sexp.get_levels();
    if levels.type_of() != SEXPTYPE::STRSXP {
        return Err(SexpError::InvalidValue("levels is not STRSXP".into()));
    }

    let n = levels.len();
    if n != expected.len() {
        return Err(SexpError::InvalidValue(format!(
            "expected {} levels, got {}",
            expected.len(),
            n
        )));
    }

    for (i, exp) in expected.iter().enumerate() {
        let charsxp = levels.string_elt(i as isize);
        let actual = unsafe { charsxp_to_str(charsxp) };
        if actual != *exp {
            return Err(SexpError::InvalidValue(format!(
                "level {}: expected '{}', got '{}'",
                i + 1,
                exp,
                actual
            )));
        }
    }

    Ok(())
}
// endregion

// region: Conversion helpers (used by derive macro)

/// Convert an R factor SEXP to a single enum value.
#[inline]
pub fn factor_from_sexp<T: RFactor>(sexp: SEXP) -> Result<T, SexpError> {
    validate_factor_levels(sexp, T::CHOICES)?;

    let len = sexp.xlength();
    if len != 1 {
        return Err(SexpError::InvalidValue(format!(
            "expected length 1, got {}",
            len
        )));
    }

    let idx = sexp.integer_elt(0);
    if idx == NA_INTEGER {
        return Err(SexpError::InvalidValue("unexpected NA".into()));
    }

    T::from_level_index(idx).ok_or_else(|| SexpError::InvalidValue("index out of range".into()))
}

/// Convert an R factor SEXP to a Vec of enum values.
#[inline]
pub(crate) fn factor_vec_from_sexp<T: RFactor>(sexp: SEXP) -> Result<Vec<T>, SexpError> {
    validate_factor_levels(sexp, T::CHOICES)?;

    let len = sexp.len();
    let mut result = Vec::with_capacity(len);

    for i in 0..len {
        let idx = sexp.integer_elt(i as isize);
        if idx == NA_INTEGER {
            return Err(SexpError::InvalidValue(format!("NA at index {}", i)));
        }
        result.push(
            T::from_level_index(idx)
                .ok_or_else(|| SexpError::InvalidValue("index out of range".into()))?,
        );
    }

    Ok(result)
}

/// Convert an R factor SEXP to a Vec of Option enum values (NA → None).
#[inline]
pub(crate) fn factor_option_vec_from_sexp<T: RFactor>(
    sexp: SEXP,
) -> Result<Vec<Option<T>>, SexpError> {
    validate_factor_levels(sexp, T::CHOICES)?;

    let len = sexp.len();
    let mut result = Vec::with_capacity(len);

    for i in 0..len {
        let idx = sexp.integer_elt(i as isize);
        if idx == NA_INTEGER {
            result.push(None);
        } else {
            result.push(Some(T::from_level_index(idx).ok_or_else(|| {
                SexpError::InvalidValue("index out of range".into())
            })?));
        }
    }

    Ok(result)
}

/// Convert an R factor SEXP to a `Vec<Option<T>>` using [`UnitEnumFactor`] (NA → `None`).
///
/// Used by the enum DataFrame reader to reconstruct `as_factor` columns.
/// Unlike `factor_option_vec_from_sexp` (which requires `RFactor + MatchArg`),
/// this accepts any `UnitEnumFactor` — including `#[derive(DataFrameRow)]`
/// unit-only enums that do not implement `RFactor`.
#[inline]
pub fn unit_factor_option_vec_from_sexp<T: UnitEnumFactor>(
    sexp: SEXP,
) -> Result<Vec<Option<T>>, SexpError> {
    validate_factor_levels(sexp, T::FACTOR_LEVELS)?;

    let len = sexp.len();
    let mut result = Vec::with_capacity(len);

    for i in 0..len {
        let idx = sexp.integer_elt(i as isize);
        if idx == NA_INTEGER {
            result.push(None);
        } else {
            result.push(Some(T::from_factor_index(idx).ok_or_else(|| {
                SexpError::InvalidValue("factor index out of range".into())
            })?));
        }
    }

    Ok(result)
}
// endregion

// region: Newtype wrappers (for orphan rule workaround)

/// Wrapper for `Vec<T: RFactor>` enabling `IntoR`/`TryFromSexp`.
#[derive(Debug, Clone)]
pub struct FactorVec<T>(pub Vec<T>);

impl<T> FactorVec<T> {
    /// Wrap a `Vec<T>` so it can be converted to and from R factors.
    pub fn new(vec: Vec<T>) -> Self {
        Self(vec)
    }

    /// Extract the inner vector.
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T> From<Vec<T>> for FactorVec<T> {
    fn from(vec: Vec<T>) -> Self {
        Self(vec)
    }
}

impl<T> Deref for FactorVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for FactorVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: RFactor> IntoR for FactorVec<T> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        let indices: Vec<i32> = self.0.iter().map(|v| v.to_level_index()).collect();
        // build_factor_with_levels protects the levels STRSXP across the
        // build_factor allocation — see "PROTECT discipline against R-devel GC".
        build_factor_with_levels(&indices, T::CHOICES)
    }
}

impl<T: RFactor> TryFromSexp for FactorVec<T> {
    type Error = SexpError;
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        factor_vec_from_sexp(sexp).map(FactorVec)
    }
}

/// Wrapper for `Vec<Option<T: RFactor>>` with NA support.
#[derive(Debug, Clone)]
pub struct FactorOptionVec<T>(pub Vec<Option<T>>);

impl<T> FactorOptionVec<T> {
    /// Wrap a `Vec<Option<T>>` so it can be converted to and from R factors with NA support.
    pub fn new(vec: Vec<Option<T>>) -> Self {
        Self(vec)
    }

    /// Extract the inner vector.
    pub fn into_inner(self) -> Vec<Option<T>> {
        self.0
    }
}

impl<T> From<Vec<Option<T>>> for FactorOptionVec<T> {
    fn from(vec: Vec<Option<T>>) -> Self {
        Self(vec)
    }
}

impl<T> Deref for FactorOptionVec<T> {
    type Target = Vec<Option<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for FactorOptionVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Blanket: every RFactor type is also a UnitEnumFactor (provides IntoR for FactorOptionVec<T>).
impl<T: RFactor + crate::match_arg::MatchArg> UnitEnumFactor for T {
    const FACTOR_LEVELS: &'static [&'static str] = T::CHOICES;
    fn to_factor_index(self) -> i32 {
        self.to_level_index()
    }
    fn from_factor_index(idx: i32) -> Option<Self> {
        T::from_level_index(idx)
    }
}

impl<T: RFactor> TryFromSexp for FactorOptionVec<T> {
    type Error = SexpError;
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        factor_option_vec_from_sexp(sexp).map(FactorOptionVec)
    }
}

// region: UnitEnumFactor — factor trait for DataFrameRow unit-only enums

/// Trait implemented by unit-only enums derived via `#[derive(DataFrameRow)]`.
///
/// Provides the level names and 1-based index needed to convert enum values
/// into R factor SEXPs. Unlike `RFactor`, this trait does **not** require
/// `Copy` or `MatchArg`, making it usable with `DataFrameRow`-derived types
/// that only need to participate as factor columns in data frames.
///
/// Implemented automatically by `#[derive(DataFrameRow)]` on unit-only enums.
/// The blanket `impl<T: UnitEnumFactor> IntoR for FactorOptionVec<T>` in
/// `miniextendr-api` provides the actual SEXP conversion used by the
/// companion struct's `into_data_frame` method.
///
/// # Safety contract
///
/// `to_factor_index` must return a value in `1..=FACTOR_LEVELS.len() as i32`
/// (or `NA_INTEGER` for missing) to produce a valid R factor SEXP.
pub trait UnitEnumFactor: Sized {
    /// Ordered level names (in the same order as the enum variants).
    const FACTOR_LEVELS: &'static [&'static str];

    /// Convert `self` to a 1-based R factor level index.
    fn to_factor_index(self) -> i32;

    /// Inverse: 1-based level index → variant, or `None` if out of range.
    ///
    /// Used by the enum DataFrame reader to reconstruct unit-only enum values
    /// from an R factor column. The default blanket impl delegates to the
    /// underlying `RFactor::from_level_index`.
    fn from_factor_index(idx: i32) -> Option<Self>;
}

impl<T: UnitEnumFactor> IntoR for FactorOptionVec<T> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        // Generic statics aren't allowed in Rust, so levels are built per call.
        // build_factor_with_levels protects the levels STRSXP across the
        // build_factor allocation — see "PROTECT discipline against R-devel GC".
        let indices: Vec<i32> = self
            .0
            .into_iter()
            .map(|opt| match opt {
                None => NA_INTEGER,
                Some(v) => v.to_factor_index(),
            })
            .collect();
        build_factor_with_levels(&indices, T::FACTOR_LEVELS)
    }
}
// endregion: UnitEnumFactor

// region: Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::match_arg::MatchArg;

    #[derive(Copy, Clone, Debug, PartialEq)]
    enum TestColor {
        Red,
        Green,
        Blue,
    }

    impl MatchArg for TestColor {
        const CHOICES: &'static [&'static str] = &["Red", "Green", "Blue"];

        fn from_choice(choice: &str) -> Option<Self> {
            match choice {
                "Red" => Some(TestColor::Red),
                "Green" => Some(TestColor::Green),
                "Blue" => Some(TestColor::Blue),
                _ => None,
            }
        }

        fn to_choice(self) -> &'static str {
            match self {
                TestColor::Red => "Red",
                TestColor::Green => "Green",
                TestColor::Blue => "Blue",
            }
        }
    }

    impl RFactor for TestColor {
        fn to_level_index(self) -> i32 {
            match self {
                TestColor::Red => 1,
                TestColor::Green => 2,
                TestColor::Blue => 3,
            }
        }

        fn from_level_index(idx: i32) -> Option<Self> {
            match idx {
                1 => Some(TestColor::Red),
                2 => Some(TestColor::Green),
                3 => Some(TestColor::Blue),
                _ => None,
            }
        }
    }

    #[test]
    fn test_level_index_roundtrip() {
        assert_eq!(
            TestColor::from_level_index(TestColor::Red.to_level_index()),
            Some(TestColor::Red)
        );
        assert_eq!(
            TestColor::from_level_index(TestColor::Green.to_level_index()),
            Some(TestColor::Green)
        );
        assert_eq!(
            TestColor::from_level_index(TestColor::Blue.to_level_index()),
            Some(TestColor::Blue)
        );
    }

    #[test]
    fn test_invalid_index() {
        assert_eq!(TestColor::from_level_index(0), None);
        assert_eq!(TestColor::from_level_index(4), None);
        assert_eq!(TestColor::from_level_index(-1), None);
    }

    #[test]
    fn test_levels_array() {
        assert_eq!(TestColor::CHOICES, &["Red", "Green", "Blue"]);
    }

    // Test interaction factor (manual impl to verify logic)
    #[derive(Copy, Clone, Debug, PartialEq)]
    enum Size {
        Small,
        Large,
    }

    impl MatchArg for Size {
        const CHOICES: &'static [&'static str] = &["Small", "Large"];

        fn from_choice(choice: &str) -> Option<Self> {
            match choice {
                "Small" => Some(Size::Small),
                "Large" => Some(Size::Large),
                _ => None,
            }
        }

        fn to_choice(self) -> &'static str {
            match self {
                Size::Small => "Small",
                Size::Large => "Large",
            }
        }
    }

    impl RFactor for Size {
        fn to_level_index(self) -> i32 {
            match self {
                Size::Small => 1,
                Size::Large => 2,
            }
        }

        fn from_level_index(idx: i32) -> Option<Self> {
            match idx {
                1 => Some(Size::Small),
                2 => Some(Size::Large),
                _ => None,
            }
        }
    }

    // Manual interaction factor impl (what derive should generate)
    #[derive(Copy, Clone, Debug, PartialEq)]
    enum ColorSize {
        Red(Size),
        Green(Size),
        Blue(Size),
    }

    impl MatchArg for ColorSize {
        const CHOICES: &'static [&'static str] = &[
            "Red.Small",
            "Red.Large",
            "Green.Small",
            "Green.Large",
            "Blue.Small",
            "Blue.Large",
        ];

        fn from_choice(choice: &str) -> Option<Self> {
            let idx_1 = Self::CHOICES
                .iter()
                .position(|&l| l == choice)
                .map(|i| i as i32 + 1)?;
            Self::from_level_index(idx_1)
        }

        fn to_choice(self) -> &'static str {
            Self::CHOICES[(self.to_level_index() - 1) as usize]
        }
    }

    impl RFactor for ColorSize {
        fn to_level_index(self) -> i32 {
            match self {
                Self::Red(inner) => {
                    let inner_idx_0 = inner.to_level_index() - 1;
                    inner_idx_0 + 1
                }
                Self::Green(inner) => {
                    let inner_idx_0 = inner.to_level_index() - 1;
                    2 + inner_idx_0 + 1
                }
                Self::Blue(inner) => {
                    let inner_idx_0 = inner.to_level_index() - 1;
                    2 * 2 + inner_idx_0 + 1
                }
            }
        }

        fn from_level_index(idx: i32) -> Option<Self> {
            match idx {
                1..=2 => {
                    let inner_idx_1 = (idx - 1) % 2 + 1;
                    Size::from_level_index(inner_idx_1).map(Self::Red)
                }
                3..=4 => {
                    let inner_idx_1 = (idx - 1) % 2 + 1;
                    Size::from_level_index(inner_idx_1).map(Self::Green)
                }
                5..=6 => {
                    let inner_idx_1 = (idx - 1) % 2 + 1;
                    Size::from_level_index(inner_idx_1).map(Self::Blue)
                }
                _ => None,
            }
        }
    }

    #[test]
    fn test_interaction_levels() {
        assert_eq!(
            ColorSize::CHOICES,
            &[
                "Red.Small",
                "Red.Large",
                "Green.Small",
                "Green.Large",
                "Blue.Small",
                "Blue.Large"
            ]
        );
    }

    #[test]
    fn test_interaction_to_index() {
        assert_eq!(ColorSize::Red(Size::Small).to_level_index(), 1);
        assert_eq!(ColorSize::Red(Size::Large).to_level_index(), 2);
        assert_eq!(ColorSize::Green(Size::Small).to_level_index(), 3);
        assert_eq!(ColorSize::Green(Size::Large).to_level_index(), 4);
        assert_eq!(ColorSize::Blue(Size::Small).to_level_index(), 5);
        assert_eq!(ColorSize::Blue(Size::Large).to_level_index(), 6);
    }

    #[test]
    fn test_interaction_from_index() {
        assert_eq!(
            ColorSize::from_level_index(1),
            Some(ColorSize::Red(Size::Small))
        );
        assert_eq!(
            ColorSize::from_level_index(2),
            Some(ColorSize::Red(Size::Large))
        );
        assert_eq!(
            ColorSize::from_level_index(3),
            Some(ColorSize::Green(Size::Small))
        );
        assert_eq!(
            ColorSize::from_level_index(4),
            Some(ColorSize::Green(Size::Large))
        );
        assert_eq!(
            ColorSize::from_level_index(5),
            Some(ColorSize::Blue(Size::Small))
        );
        assert_eq!(
            ColorSize::from_level_index(6),
            Some(ColorSize::Blue(Size::Large))
        );
        assert_eq!(ColorSize::from_level_index(0), None);
        assert_eq!(ColorSize::from_level_index(7), None);
    }

    #[test]
    fn test_interaction_roundtrip() {
        for i in 1..=6 {
            let color_size = ColorSize::from_level_index(i).unwrap();
            assert_eq!(color_size.to_level_index(), i);
        }
    }
}
// endregion
