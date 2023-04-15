// include!(concat!(env!("OUT_DIR"), "/r-bindings.rs"));
// #![allow(non_upper_case_globals)]
// #![allow(non_camel_case_types)]
// #![allow(non_snake_case)]
// #![allow(improper_ctypes)]

include!("../r-bindings.rs");

pub const M_E: f64 = std::f64::consts::E;
pub const M_LOG2E: f64 = std::f64::consts::LOG2_E;
pub const M_LOG10E: f64 = std::f64::consts::LOG10_E;
pub const M_LN2: f64 = std::f64::consts::LN_2;
pub const M_LN10: f64 = std::f64::consts::LN_10;
pub const M_PI: f64 = std::f64::consts::PI;
pub const M_PI_2: f64 = std::f64::consts::FRAC_PI_2;
pub const M_PI_4: f64 = std::f64::consts::FRAC_PI_4;
pub const M_1_PI: f64 = std::f64::consts::FRAC_1_PI;
pub const M_2_PI: f64 = std::f64::consts::FRAC_2_PI;
pub const M_2_SQRTPI: f64 = std::f64::consts::FRAC_2_SQRT_PI;
pub const M_SQRT2: f64 = std::f64::consts::SQRT_2;
pub const M_SQRT1_2: f64 = std::f64::consts::FRAC_1_SQRT_2;
