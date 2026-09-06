//! `#[miniextendr(serde_error)]`: classed `Result` errors derived from the
//! error type's serde output (#1449).
//!
//! No `RConditionError` impl: the enum variant becomes the member class
//! `<prefix>_<variant>`, the variant's fields become `e$<name>`, and the
//! message comes from `Display`. Internally tagged enums
//! (`#[serde(tag = "kind")]`) have the tag consumed as the variant; externally
//! tagged enums (serde's default) report the variant name verbatim. The default
//! prefix is `<crate>_error` (`miniextendr_error` here); `serde_error(prefix =
//! "engine")` overrides it.
//!
//! Payload-field control (#1457): `serde_error(skip("message"))` drops a field,
//! `serde_error(rename(message = "detail"))` splices it under another name, and
//! a `message` field whose text equals the `Display` output is dropped with no
//! option at all. Every other collision with `message` / `call` / `kind` still
//! raises the reserved-name error, and a rename onto a name the variant already
//! carries raises the duplicate-name error (#1459).

use crate::serde::Serialize;
use miniextendr_api::miniextendr;

// region: Internally tagged error enum

/// Stand-in for a downstream crate's error enum, already serde-tagged for
/// other consumers (logging, JSON).
#[derive(Debug, Serialize)]
#[serde(crate = "crate::serde", tag = "kind", rename_all = "snake_case")]
pub enum EngineError {
    MissingField { field: String },
    OutOfRange { value: f64, max: f64, route: Route },
    Io,
}

/// A nested unit enum: serializes as its (renamed) variant name.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(crate = "crate::serde", rename_all = "snake_case")]
pub enum Route {
    Oral,
    Iv,
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::MissingField { field } => write!(f, "field `{field}` is missing"),
            EngineError::OutOfRange { value, max, .. } => {
                write!(f, "{value} exceeds the maximum {max}")
            }
            EngineError::Io => write!(f, "I/O failure"),
        }
    }
}

impl std::error::Error for EngineError {}

/// Struct variant with one string field.
/// @param field Field name to report missing.
#[miniextendr(serde_error)]
pub fn serde_err_missing(field: String) -> Result<i32, EngineError> {
    Err(EngineError::MissingField { field })
}

/// Struct variant with numeric fields and a nested unit enum.
/// @param value Value to check against the bound 100.
#[miniextendr(serde_error)]
pub fn serde_err_range(value: f64) -> Result<f64, EngineError> {
    if value > 100.0 {
        return Err(EngineError::OutOfRange {
            value,
            max: 100.0,
            route: Route::Oral,
        });
    }
    Ok(value)
}

/// Unit variant: classes only, no data.
#[miniextendr(serde_error)]
pub fn serde_err_unit_variant() -> Result<(), EngineError> {
    Err(EngineError::Io)
}

/// `prefix =` override: classes `engine_out_of_range` / `engine`.
/// @param value Value to check against the bound 100.
#[miniextendr(serde_error(prefix = "engine"))]
pub fn serde_err_prefixed(value: f64) -> Result<f64, EngineError> {
    serde_err_range(value)
}

/// The `Ok` arm is untouched by `serde_error`.
/// @param value Returned as-is.
#[miniextendr(serde_error)]
pub fn serde_err_ok(value: f64) -> Result<f64, EngineError> {
    Ok(value)
}

// endregion

// region: Externally tagged error enum

/// serde's default (external) tagging: struct, newtype and unit variants.
#[derive(Debug, Serialize)]
#[serde(crate = "crate::serde")]
pub enum ExtError {
    Bad { code: i32 },
    Plain(String),
    Unit,
}

impl std::fmt::Display for ExtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtError::Bad { code } => write!(f, "bad thing {code}"),
            ExtError::Plain(msg) => write!(f, "{msg}"),
            ExtError::Unit => write!(f, "unit failure"),
        }
    }
}

/// Raise one of the externally tagged variants.
/// @param which One of `"bad"`, `"plain"`, `"unit"`.
#[miniextendr(serde_error)]
pub fn serde_err_external(which: String) -> Result<(), ExtError> {
    match which.as_str() {
        "bad" => Err(ExtError::Bad { code: 7 }),
        "plain" => Err(ExtError::Plain("boom".to_string())),
        _ => Err(ExtError::Unit),
    }
}

// endregion

// region: Reserved payload names: skip, rename, Display-equal drop (#1457)

/// Stand-in for a wrapped parser error: the ordinary `Variant { message }`
/// shape, plus a variant whose reserved field is not `message`.
#[derive(Debug, Serialize)]
#[serde(crate = "crate::serde", tag = "kind", rename_all = "snake_case")]
pub enum ParserError {
    /// `Display` adds the line number, so `message` is not redundant.
    Parse { message: String, line: u32 },
    /// `Display` is the wrapped message verbatim.
    Wrapped { message: String },
    /// A reserved name no option here addresses.
    Bad { call: String },
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::Parse { message, line } => write!(f, "line {line}: {message}"),
            ParserError::Wrapped { message } => f.write_str(message),
            ParserError::Bad { .. } => f.write_str("bad call"),
        }
    }
}

impl std::error::Error for ParserError {}

fn parser_error(which: &str, message: String) -> ParserError {
    match which {
        "parse" => ParserError::Parse { message, line: 3 },
        "wrapped" => ParserError::Wrapped { message },
        _ => ParserError::Bad { call: message },
    }
}

/// No field option: `wrapped` (text equal to `Display`) drops the field,
/// `parse` (text differs) and `call` hit the reserved-name error.
/// @param which One of `"parse"`, `"wrapped"`, `"call"`.
/// @param message Payload text.
#[miniextendr(serde_error)]
pub fn serde_err_message_default(which: String, message: String) -> Result<(), ParserError> {
    Err(parser_error(&which, message))
}

/// `skip("message")`: the field is dropped for every variant that has it;
/// `call` still hits the reserved-name error.
/// @param which One of `"parse"`, `"wrapped"`, `"call"`.
/// @param message Payload text.
#[miniextendr(serde_error(skip("message")))]
pub fn serde_err_message_skipped(which: String, message: String) -> Result<(), ParserError> {
    Err(parser_error(&which, message))
}

/// `rename(message = "detail")`: the field reaches R as `e$detail`, also when
/// its text equals `Display`; `call` still hits the reserved-name error.
/// @param which One of `"parse"`, `"wrapped"`, `"call"`.
/// @param message Payload text.
#[miniextendr(serde_error(rename(message = "detail")))]
pub fn serde_err_message_renamed(which: String, message: String) -> Result<(), ParserError> {
    Err(parser_error(&which, message))
}

/// `rename(message = "line")`: `parse` already carries `line`, so the
/// condition would hold two `line` fields and `e$line` would read only the
/// first; that is the duplicate-name error (#1459). `wrapped` has no `line`
/// and reaches R with `e$line` holding the text; `call` still hits the
/// reserved-name error.
/// @param which One of `"parse"`, `"wrapped"`, `"call"`.
/// @param message Payload text.
#[miniextendr(serde_error(rename(message = "line")))]
pub fn serde_err_message_renamed_onto_line(
    which: String,
    message: String,
) -> Result<(), ParserError> {
    Err(parser_error(&which, message))
}

// endregion

// region: S3 method (impl-block codegen arm)

/// The same error type raised from an S3 method.
#[derive(miniextendr_api::ExternalPtr)]
pub struct SerdeChecker {
    max: f64,
}

#[miniextendr(s3)]
impl SerdeChecker {
    /// A checker with an upper bound.
    /// @param max Upper bound.
    pub fn new(max: f64) -> Self {
        SerdeChecker { max }
    }

    /// Check `value` against the bound; raises `miniextendr_error_out_of_range`.
    /// @param value Value to check.
    #[miniextendr(serde_error)]
    pub fn check_value(&self, value: f64) -> Result<f64, EngineError> {
        if value > self.max {
            return Err(EngineError::OutOfRange {
                value,
                max: self.max,
                route: Route::Iv,
            });
        }
        Ok(value)
    }

    /// Parse `text` as a number; the parser error's `message` field is skipped
    /// (method-side `serde_error(skip(...))`).
    /// @param text Text to parse.
    #[miniextendr(serde_error(skip("message")))]
    pub fn parse_value(&self, text: String) -> Result<f64, ParserError> {
        text.trim().parse::<f64>().map_err(|e| ParserError::Parse {
            message: e.to_string(),
            line: 1,
        })
    }
}

// endregion
