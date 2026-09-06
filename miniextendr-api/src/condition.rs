//! Condition macros and signal enum for the Rust→R condition pipeline.
//!
//! This module provides two things:
//!
//! 1. **[`RCondition`] enum** — the internal panic payload used by `error!()`,
//!    `warning!()`, `message!()`, and `condition!()` macros. Caught by
//!    [`crate::unwind_protect::with_r_unwind_protect`] before the generic
//!    panic→error path, then forwarded to R as a structured condition with
//!    `rust_*` class layering via
//!    [`crate::error_value::make_rust_condition_value`].
//!
//! 2. **[`AsRError`] struct** — wraps any `E: std::error::Error` and
//!    preserves the full error chain (cause/source) when converting to an R
//!    error message. Use as the `Err` type in `Result` returns.
//!
//! # When to reach for what
//!
//! There are three Rust→R error-emission paths and they are not
//! interchangeable. The crate-level rationale (why tagged-SEXP at all, what
//! `error_in_r` defaults imply, and the `with_r_unwind_protect` leak) lives
//! on [`crate::error_value`]; the practical picking-one summary:
//!
//! - **`panic!`** — escape hatch. Becomes class `c("rust_error",
//!   "simpleError", "error", "condition")` with `kind = "panic"`. Use for
//!   genuine bugs or impossible states. Cheapest in source; coarsest in R
//!   (callers can only match `rust_error` / `error`, not a specific class).
//! - **`error!` / `warning!` / `message!` / `condition!`** (this module) —
//!   typed conditions. Same transport, but allow an optional `class =
//!   "name"` so R-side `tryCatch` can route by class. `warning!` /
//!   `message!` / `condition!` are the only way to emit non-error
//!   conditions; `panic!` is always an error.
//! - **`Result<T, E>` with [`AsRError<E>`]** — value-style propagation
//!   through Rust code. Converts at the boundary; `kind = "result_err"`.
//!   Best when the failure path is real-and-recoverable in Rust and the
//!   error chain (`std::error::Error::source`) is worth preserving.
//!
//! `Rf_error` is *not* on this list. Direct `Rf_error` skips Rust
//! destructors unconditionally and is forbidden by lint MXL300; see
//! [`crate::error_value`] for the full reasoning.
//!
//! # Macro-vs-module name collision
//!
//! `#[macro_export]` puts each macro at the *crate root*, where `error!` and
//! `condition!` collide with the same-named modules `pub mod error` and `pub
//! mod condition`. The practical implication: `use miniextendr_api::{error,
//! condition}` imports the *modules*, not the macros, and a subsequent
//! `error!(...)` call fails to resolve.
//!
//! Workarounds, in rough order of ergonomics:
//!
//! 1. Invoke via fully-qualified path: `miniextendr_api::error!("...")`.
//! 2. `use miniextendr_api as mx;` then `mx::error!("...")`.
//! 3. `warning!` and `message!` have no module conflict — `use
//!    miniextendr_api::{warning, message};` works directly.
//!
//! See the individual macro docs for the per-macro reminder.
//!
//! # Condition macros
//!
//! The four macros are the user-facing API for raising non-panic conditions from
//! Rust. They ride the tagged-condition transport that every `#[miniextendr]`
//! function uses:
//!
//! ```ignore
//! use miniextendr_api::{error, warning, message, condition};
//!
//! #[miniextendr]
//! fn demo_error() {
//!     error!("something went wrong: {}", 42);
//! }
//!
//! #[miniextendr]
//! fn demo_warning() {
//!     warning!("something looks suspicious");
//! }
//!
//! #[miniextendr]
//! fn demo_message() {
//!     message!("progress: {} of {}", 1, 10);
//! }
//!
//! #[miniextendr]
//! fn demo_condition() {
//!     condition!("a signallable condition");
//! }
//! ```
//!
//! Optional `class =` extension for programmatic catching:
//!
//! ```ignore
//! #[miniextendr]
//! fn typed_error(name: &str) {
//!     error!(class = "my_error", "missing field: {name}");
//! }
//! ```
//!
//! ```r
//! tryCatch(typed_error("x"), my_error = function(e) "caught!")
//! # [1] "caught!"
//! ```
//!
//! Optional `data =` extension attaches structured fields readable as
//! `e$<name>` in handlers (rlang-`abort()`-style):
//!
//! ```ignore
//! #[miniextendr]
//! fn validate(value: i32) {
//!     if !(0..=100).contains(&value) {
//!         miniextendr_api::error!(
//!             class = "validation_error",
//!             data = [("value", value), ("min", 0), ("max", 100)],
//!             "value {value} out of range"
//!         );
//!     }
//! }
//! ```
//!
//! ```r
//! tryCatch(validate(150L), validation_error = function(e) c(e$value, e$min, e$max))
//! # [1] 150   0 100
//! ```
//!
//! Supported `data` value types (anything with `RValue: From<_>`): scalars and
//! `Vec`s of `i32`, `f64`, `bool`, `String` / `&str`; their NA-aware `Option` /
//! `Vec<Option<_>>` forms (`None` → R `NA`); the wide-integer ladder (`i64` /
//! `u32`, narrowed to `integer(1)` when it fits, `double(1)` otherwise); and the
//! [`RValue::debug`](crate::RValue::debug) escape hatch, which stringifies any
//! `T: Debug`. For nested lists or complex/raw/NA-bearing values build an
//! [`RValue`](crate::RValue) directly. The payload is built as a Send-safe owned
//! value at the call site and materialised as R objects on the main thread — so
//! `data =` works from worker-thread code too.
//!
//! Three `data =` grammars are accepted (see [`crate::error!`]):
//! - single pair: `data = ("name", value)`
//! - bracketed list: `data = [("a", v1), ("b", v2)]`
//! - keyed builder sugar: `data = { value = 42, code = 7 }` (bare-ident keys)
//!
//! # `AsRError`
//!
//! ```ignore
//! use miniextendr_api::condition::AsRError;
//!
//! #[miniextendr]
//! fn parse_config(path: &str) -> Result<i32, AsRError<std::io::Error>> {
//!     let content = std::fs::read_to_string(path).map_err(AsRError)?;
//!     Ok(content.len() as i32)
//! }
//! ```

// region: ConditionData — Send-safe owned condition-data payload

/// Named condition-data payload: an ordered list of `(name, value)` pairs.
///
/// Produced by the macros' `data = ...` form and consumed by
/// [`crate::error_value::make_rust_condition_value`]. Each value is an
/// [`RValue`](crate::RValue) — an owned, `Send`, R-native value tree. Send-safe
/// by construction (no live `SEXP`), so the payload can travel through
/// `panic_any` and cross the worker→main thread boundary; the R objects are
/// materialised on the main thread at the unwind boundary.
///
/// The macros accept any value with `RValue: From<_>` (scalars and `Vec`s of
/// `i32` / `f64` / `bool` / `String` / `&str`; their NA-aware `Option` /
/// `Vec<Option<_>>` forms; the `i64` / `u32` wide-integer ladder); a scalar
/// `7i32` becomes `integer(1)` and a `Vec<i32>` becomes `integer(n)`. Any
/// `T: Debug` rides along via [`RValue::debug`](crate::RValue::debug). For
/// nested lists or complex/raw values build an [`RValue`](crate::RValue)
/// directly.
pub type ConditionData = Vec<(String, crate::RValue)>;

// endregion

// region: RCondition enum — internal panic payload

/// Internal panic payload for structured R conditions.
///
/// Raised by the `error!()`, `warning!()`, `message!()`, and `condition!()`
/// macros via `std::panic::panic_any`. Caught by `with_r_unwind_protect`
/// before the generic panic→string path and forwarded to R as a tagged SEXP
/// with `rust_*` class layering.
///
/// This type is `#[doc(hidden)]` because users interact with the macros,
/// not the enum directly.
#[doc(hidden)]
#[derive(Debug)]
pub enum RCondition {
    /// Raised by `error!(...)` / `error!(class = "...", ...)`, and by the
    /// `Result<T, E>` Err arm once reconstructed across a package boundary.
    /// `class` is the user-supplied class vector (empty = none), prepended
    /// to the `rust_error` layering on the R side.
    Error {
        message: String,
        class: Vec<String>,
        data: Option<ConditionData>,
    },
    /// Raised by `warning!(...)` / `warning!(class = "...", ...)`.
    Warning {
        message: String,
        class: Vec<String>,
        data: Option<ConditionData>,
    },
    /// Raised by `message!(...)`.
    Message {
        message: String,
        data: Option<ConditionData>,
    },
    /// Raised by `condition!(...)` / `condition!(class = "...", ...)`.
    Condition {
        message: String,
        class: Vec<String>,
        data: Option<ConditionData>,
    },
}

// endregion

// region: Macros

/// Internal: one condition-data field name, validated against the reserved
/// slots. Not part of the public API.
///
/// - `lit "name"` / `ident name`: the name is a literal, so the reserved-name
///   check runs at compile time (`const` assertion) and the field costs one
///   `to_string()` at runtime.
/// - `expr <e>`: a computed name; checked at runtime by
///   [`crate::condition::check_condition_field_name`].
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_condition_field_name {
    (lit $name:literal) => {{
        const _: () = ::core::assert!(
            !$crate::condition::is_reserved_condition_field($name),
            "condition data field name is reserved: `message`, `call` and `kind` are the condition's own slots; rename the field"
        );
        ($name).to_string()
    }};
    (ident $name:ident) => {{
        const _: () = ::core::assert!(
            !$crate::condition::is_reserved_condition_field(::core::stringify!($name)),
            "condition data field name is reserved: `message`, `call` and `kind` are the condition's own slots; rename the field"
        );
        ::core::stringify!($name).to_string()
    }};
    (expr $name:expr) => {
        $crate::condition::check_condition_field_name(($name).to_string())
    };
}

/// Internal: one `(name, value)` pair of a bracketed `data = [...]` list; the
/// name goes through the reserved-slot check (compile time for literals).
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_condition_pair {
    (($name:literal, $value:expr $(,)?)) => {
        ($crate::__mx_condition_field_name!(lit $name), $crate::RValue::from($value))
    };
    (($name:expr, $value:expr $(,)?)) => {
        ($crate::__mx_condition_field_name!(expr $name), $crate::RValue::from($value))
    };
}

/// Internal: normalise a macro `data = ...` argument into
/// `Option<ConditionData>`. Not part of the public API.
///
/// Three forms are accepted:
/// - a single pair: `("name", value)`
/// - a bracketed list of pairs: `[("a", v1), ("b", v2)]`
/// - keyed builder sugar: `{ name = value, other = value }` — the field name is
///   a bare identifier (stringified by the macro), so `{ value = 42, code = 7 }`
///   is shorthand for `[("value", 42), ("code", 7)]`.
///
/// Each `value` is converted via `RValue::from`, so any type with an `RValue`
/// `From` impl (the scalar/vector/`Option`/wide-integer set) works without
/// ceremony.
///
/// Field names are checked against the condition's own slots (`message` /
/// `call` / `kind`): at compile time when the name is a literal or bare
/// identifier, at runtime otherwise.
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_condition_data {
    (($name:literal, $value:expr $(,)?)) => {
        ::std::option::Option::Some(::std::vec![(
            $crate::__mx_condition_field_name!(lit $name),
            $crate::RValue::from($value),
        )])
    };
    (($name:expr, $value:expr $(,)?)) => {
        ::std::option::Option::Some(::std::vec![(
            $crate::__mx_condition_field_name!(expr $name),
            $crate::RValue::from($value),
        )])
    };
    ([ $($pair:tt),* $(,)? ]) => {
        ::std::option::Option::Some(::std::vec![
            $( $crate::__mx_condition_pair!($pair), )*
        ])
    };
    ({ $($name:ident = $value:expr),* $(,)? }) => {
        ::std::option::Option::Some(::std::vec![
            $(
                (
                    $crate::__mx_condition_field_name!(ident $name),
                    $crate::RValue::from($value),
                ),
            )*
        ])
    };
}

/// Internal: parse the shared option grid of `error!` / `warning!` /
/// `condition!` into `(class, data, message)`. Not part of the public API.
///
/// Accepted orders (every part optional except the message):
/// `class = ..`, then `data = ..`, then the `format!` arguments. `class` takes
/// anything implementing [`crate::condition::ConditionClass`] (one string or
/// several).
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_condition_parts {
    (class = $class:expr, data = $data:tt, $($arg:tt)*) => {
        (
            $crate::condition::ConditionClass::into_condition_class($class),
            $crate::__mx_condition_data!($data),
            ::std::format!($($arg)*),
        )
    };
    (data = $data:tt, $($arg:tt)*) => {
        (
            ::std::vec::Vec::<::std::string::String>::new(),
            $crate::__mx_condition_data!($data),
            ::std::format!($($arg)*),
        )
    };
    (class = $class:expr, $($arg:tt)*) => {
        (
            $crate::condition::ConditionClass::into_condition_class($class),
            ::std::option::Option::<$crate::condition::ConditionData>::None,
            ::std::format!($($arg)*),
        )
    };
    ($($arg:tt)*) => {
        (
            ::std::vec::Vec::<::std::string::String>::new(),
            ::std::option::Option::<$crate::condition::ConditionData>::None,
            ::std::format!($($arg)*),
        )
    };
}

/// Raise an R error from Rust with `rust_error` class layering.
///
/// Rides the tagged-condition transport that every `#[miniextendr]` function uses.
/// The raised condition has class `c("rust_error", "simpleError", "error", "condition")`.
///
/// An optional `class = "name"` form prepends a custom class for programmatic catching:
/// `c("name", "rust_error", "simpleError", "error", "condition")`. `class` also
/// takes a vector (`class = ["pkg_error_missing_field", "pkg_error"]`, or any
/// [`ConditionClass`] value) so handlers can catch the family or the member.
///
/// # Structured `data = ...` payloads
///
/// An optional `data = ...` form (after `class`, before the message) attaches
/// named fields to the condition object, rlang-`abort()`-style. Handlers read
/// them as `e$<name>` instead of parsing the message string:
///
/// ```ignore
/// // Single field:
/// mx::error!(class = "range_error", data = ("value", value), "value {value} out of range");
///
/// // Multiple fields (bracketed list of pairs):
/// mx::error!(
///     class = "validation_error",
///     data = [("value", value), ("min", 0), ("max", 100)],
///     "value {value} out of range"
/// );
///
/// // Keyed builder sugar (bare-ident keys, stringified by the macro):
/// mx::error!(
///     class = "validation_error",
///     data = { value = value, min = 0, max = 100 },
///     "value {value} out of range"
/// );
/// ```
///
/// ```r
/// tryCatch(validate(150L), validation_error = function(e) c(e$value, e$min, e$max))
/// # [1] 150   0 100
/// ```
///
/// Argument order is fixed: `class = ...` (optional), then `data = ...`
/// (optional), then the format message.
///
/// Field names `message`, `call` and `kind` are the condition's own slots and
/// are rejected (at compile time for literal / bare-identifier names, at
/// runtime otherwise); give the field another name (`rule` instead of `kind`,
/// say). A downstream error type with such a field renames it where the
/// payload is built (`#[serde(rename)]`, a different key).
///
/// **Supported value types**: scalars and `Vec`s of `i32`, `f64`, `bool`, and
/// `String` (plus `&str` / `Vec<&str>`, converted to owned); their NA-aware
/// `Option` / `Vec<Option<_>>` forms (→ R `NA`); the wide-integer ladder (`i64`
/// / `u32`); and the [`RValue::debug`](crate::RValue::debug) escape hatch for
/// any `T: Debug`. The payload must be `Send` — it travels through `panic_any`
/// and may cross the worker→main thread boundary, so live `SEXP`s cannot ride
/// along; the R objects are materialised on the main thread at the unwind
/// boundary. For nested lists or complex/raw values build an
/// [`RValue`](crate::RValue) directly.
///
/// # See also
///
/// - [`crate::warning!`] / [`crate::message!`] / [`crate::condition!`] — the
///   non-error sibling kinds (warning continues execution; message is muffled
///   by `suppressMessages`; condition is silent without a handler).
/// - [`std::panic!`] — escape hatch with the same `rust_error` class layering
///   but no custom-class slot. Use for true bugs / impossible states; reach for
///   `error!` when callers might want to route by class.
/// - [`AsRError`] — wraps `Result<_, E: std::error::Error>` for value-style
///   propagation through Rust code; converts at the boundary.
/// - [`crate::error_value`] — module-level rationale for the tagged-SEXP
///   transport and the `error_in_r` default.
///
/// **Name-collision note.** Because `pub mod error` exists at the crate root,
/// `use miniextendr_api::error` imports the module rather than this macro, and
/// glob imports (`use miniextendr_api::*;`) hit the same shadow. Prefer the
/// collision-free alias [`crate::rust_error!`], which has the identical
/// expansion; otherwise invoke via `miniextendr_api::error!(...)` (fully
/// qualified) or `mx::error!(...)` after `use miniextendr_api as mx;`.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api as mx;
///
/// #[miniextendr]
/// fn fail() {
///     mx::error!("something went wrong: {}", 42);
/// }
///
/// // With a custom class for tryCatch:
/// #[miniextendr]
/// fn typed_fail(name: &str) {
///     mx::error!(class = "my_error", "missing field: {name}");
/// }
/// ```
///
/// ```r
/// tryCatch(fail(), rust_error = function(e) conditionMessage(e))
/// # [1] "something went wrong: 42"
///
/// tryCatch(typed_fail("x"), my_error = function(e) "caught!")
/// # [1] "caught!"
/// ```
#[macro_export]
macro_rules! error {
    ($($t:tt)*) => {{
        let (__mx_class, __mx_data, __mx_message) = $crate::__mx_condition_parts!($($t)*);
        ::std::panic::panic_any($crate::condition::RCondition::Error {
            message: __mx_message,
            class: __mx_class,
            data: __mx_data,
        })
    }};
}

/// Collision-free alias for [`crate::error!`].
///
/// Identical expansion and grammar (`class = …`, `data = …`, and the plain
/// `format!` forms) — it exists so that `use miniextendr_api::*;` or
/// `use miniextendr_api::rust_error;` gives you a usable macro name. The bare
/// `error!` name is shadowed at the crate root by `pub mod error` (see the
/// name-collision note on [`crate::error!`]), so a glob or direct import
/// resolves to the module, not the macro. `rust_error!` has no such clash.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rust_error;
///
/// #[miniextendr]
/// fn typed_fail(name: &str) {
///     rust_error!(class = "my_error", "missing field: {name}");
/// }
/// ```
#[macro_export]
macro_rules! rust_error {
    ($($t:tt)*) => { $crate::error!($($t)*) };
}

/// Raise an R warning from Rust with `rust_warning` class layering.
///
/// Rides the tagged-condition transport that every `#[miniextendr]` function uses.
/// Unlike `panic!`, execution continues after `warning!` is caught by a handler.
/// The raised condition has class `c("rust_warning", "simpleWarning", "warning", "condition")`.
///
/// An optional `class = "name"` form prepends a custom class. An optional
/// `data = ...` form (after `class`, before the message) attaches named fields
/// readable as `w$<name>` in handlers — same grammar and supported value types
/// as [`crate::error!`] (see there for details):
///
/// ```ignore
/// warning!(class = "truncation", data = ("dropped", n), "dropped {n} rows");
/// ```
///
/// # See also
///
/// - [`crate::error!`] — fatal sibling; aborts the call instead of continuing.
/// - [`crate::message!`] / [`crate::condition!`] — softer signal kinds (muffled
///   by `suppressMessages` / silent without handler, respectively).
/// - [`std::panic!`] — escape hatch when "continue after this" is not a sensible
///   semantic.
/// - [`crate::error_value`] — tagged-SEXP transport rationale.
///
/// No name-collision caveat: there is no `pub mod warning`, so
/// `use miniextendr_api::warning;` then `warning!(...)` works directly.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::warning;
///
/// #[miniextendr]
/// fn maybe_warn(x: i32) -> i32 {
///     if x > 100 {
///         warning!("x is large: {x}");
///     }
///     x * 2
/// }
/// ```
///
/// ```r
/// withCallingHandlers(
///   maybe_warn(200L),
///   warning = function(w) { cat("saw:", conditionMessage(w)); invokeRestart("muffleWarning") }
/// )
/// # saw: x is large: 200
/// # [1] 400
/// ```
#[macro_export]
macro_rules! warning {
    ($($t:tt)*) => {{
        let (__mx_class, __mx_data, __mx_message) = $crate::__mx_condition_parts!($($t)*);
        ::std::panic::panic_any($crate::condition::RCondition::Warning {
            message: __mx_message,
            class: __mx_class,
            data: __mx_data,
        })
    }};
}

/// Emit an R message from Rust with `rust_message` class layering.
///
/// Rides the tagged-condition transport that every `#[miniextendr]` function uses.
/// The raised condition has class `c("rust_message", "simpleMessage", "message", "condition")`.
/// Muffled by `suppressMessages()` automatically (standard R restart mechanism).
///
/// An optional `data = ...` form (before the message) attaches named fields
/// readable as `m$<name>` in `withCallingHandlers` — same grammar and
/// supported value types as [`crate::error!`] (see there for details). There
/// is no `class =` form for `message!`.
///
/// # See also
///
/// - [`crate::warning!`] / [`crate::condition!`] — louder/quieter sibling kinds.
/// - [`crate::error!`] — for fatal failures.
/// - [`std::panic!`] — escape hatch.
/// - [`crate::error_value`] — tagged-SEXP transport rationale.
///
/// No name-collision caveat: there is no `pub mod message`, so
/// `use miniextendr_api::message;` then `message!(...)` works directly.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::message;
///
/// #[miniextendr]
/// fn log_step(step: i32) {
///     message!("step {} complete", step);
/// }
/// ```
///
/// ```r
/// log_step(3L)
/// # step 3 complete
///
/// suppressMessages(log_step(3L))  # no output
/// ```
#[macro_export]
macro_rules! message {
    (data = $data:tt, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Message {
            message: ::std::format!($($arg)*),
            data: $crate::__mx_condition_data!($data),
        })
    };
    ($($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Message {
            message: ::std::format!($($arg)*),
            data: ::std::option::Option::None,
        })
    };
}

/// Signal a generic R condition from Rust with `rust_condition` class layering.
///
/// Rides the tagged-condition transport that every `#[miniextendr]` function uses.
/// Unlike `error!`, a bare condition is a silent no-op if there is no handler.
/// The raised condition has class `c("rust_condition", "simpleCondition", "condition")`.
///
/// An optional `class = "name"` form prepends a custom class. An optional
/// `data = ...` form (after `class`, before the message) attaches named fields
/// readable as `c$<name>` in handlers — same grammar and supported value types
/// as [`crate::error!`] (see there for details).
///
/// # See also
///
/// - [`crate::error!`] / [`crate::warning!`] / [`crate::message!`] — louder
///   condition kinds. Pick `condition!` when "no handler = silent" is the
///   right default (progress events, structured logging hooks).
/// - [`std::panic!`] — escape hatch when the failure cannot be ignored.
/// - [`crate::error_value`] — tagged-SEXP transport rationale.
///
/// **Name-collision note.** Because `pub mod condition` exists at the crate
/// root, `use miniextendr_api::condition` imports the module rather than this
/// macro, and glob imports (`use miniextendr_api::*;`) hit the same shadow.
/// Prefer the collision-free alias [`crate::rust_condition!`], which has the
/// identical expansion; otherwise invoke via `miniextendr_api::condition!(...)`
/// (fully qualified) or `mx::condition!(...)` after `use miniextendr_api as mx;`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::condition;
///
/// #[miniextendr]
/// fn signal_progress(n: i32) {
///     condition!(class = "my_progress", "processed {n} items");
/// }
/// ```
///
/// ```r
/// withCallingHandlers(
///   signal_progress(42L),
///   my_progress = function(c) cat("progress:", conditionMessage(c), "\n")
/// )
/// # progress: processed 42 items
/// ```
#[macro_export]
macro_rules! condition {
    ($($t:tt)*) => {{
        let (__mx_class, __mx_data, __mx_message) = $crate::__mx_condition_parts!($($t)*);
        ::std::panic::panic_any($crate::condition::RCondition::Condition {
            message: __mx_message,
            class: __mx_class,
            data: __mx_data,
        })
    }};
}

/// Collision-free alias for [`crate::condition!`].
///
/// Identical expansion and grammar (`class = …`, `data = …`, and the plain
/// `format!` forms) — it exists so that `use miniextendr_api::*;` or
/// `use miniextendr_api::rust_condition;` gives you a usable macro name. The
/// bare `condition!` name is shadowed at the crate root by `pub mod condition`
/// (see the name-collision note on [`crate::condition!`]), so a glob or direct
/// import resolves to the module, not the macro. `rust_condition!` has no such
/// clash.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rust_condition;
///
/// #[miniextendr]
/// fn signal_progress(n: i32) {
///     rust_condition!(class = "my_progress", "processed {n} items");
/// }
/// ```
#[macro_export]
macro_rules! rust_condition {
    ($($t:tt)*) => { $crate::condition!($($t)*) };
}

// endregion

// region: Class vectors and reserved field names

/// What the condition macros' `class = …` (and [`RError::class`]) accept: one
/// class or several, as `&str` / `String` / arrays / `Vec` / slices.
///
/// The classes are prepended in order to the `rust_*` layering, so put the most
/// specific first: `class = ["pkg_error_missing_field", "pkg_error"]` renders
/// as `c("pkg_error_missing_field", "pkg_error", "rust_error", …)` and both
/// `tryCatch(pkg_error_missing_field = …)` and `tryCatch(pkg_error = …)` match.
pub trait ConditionClass {
    /// The class vector, most specific first.
    fn into_condition_class(self) -> Vec<String>;
}

impl ConditionClass for &str {
    fn into_condition_class(self) -> Vec<String> {
        vec![self.to_string()]
    }
}
impl ConditionClass for String {
    fn into_condition_class(self) -> Vec<String> {
        vec![self]
    }
}
impl ConditionClass for &String {
    fn into_condition_class(self) -> Vec<String> {
        vec![self.clone()]
    }
}
impl<const N: usize> ConditionClass for [&str; N] {
    fn into_condition_class(self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}
impl<const N: usize> ConditionClass for [String; N] {
    fn into_condition_class(self) -> Vec<String> {
        self.into_iter().collect()
    }
}
impl ConditionClass for Vec<&str> {
    fn into_condition_class(self) -> Vec<String> {
        self.into_iter().map(str::to_string).collect()
    }
}
impl ConditionClass for Vec<String> {
    fn into_condition_class(self) -> Vec<String> {
        self
    }
}
impl ConditionClass for &[&str] {
    fn into_condition_class(self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}
impl ConditionClass for &[String] {
    fn into_condition_class(self) -> Vec<String> {
        self.to_vec()
    }
}

/// Condition-data field names that would overwrite the condition object's own
/// slots when the R helper splices `data` in (`utils::modifyList` over
/// `list(message, call, kind)`).
pub const RESERVED_CONDITION_FIELDS: &[&str] = &["message", "call", "kind"];

/// `true` for [`RESERVED_CONDITION_FIELDS`]. `const` so the macros can reject a
/// literal field name at compile time:
///
/// ```compile_fail
/// # use miniextendr_api as mx;
/// # fn f() {
/// mx::rust_error!(data = ("kind", 1), "boom");
/// # }
/// ```
///
/// Any other name is fine, so the fix for a clash is a rename at the source:
///
/// ```
/// # use miniextendr_api as mx;
/// # fn f() {
/// mx::rust_error!(data = ("rule", 1), "boom");
/// # }
/// ```
pub const fn is_reserved_condition_field(name: &str) -> bool {
    const fn eq(a: &str, b: &str) -> bool {
        let (a, b) = (a.as_bytes(), b.as_bytes());
        if a.len() != b.len() {
            return false;
        }
        let mut i = 0;
        while i < a.len() {
            if a[i] != b[i] {
                return false;
            }
            i += 1;
        }
        true
    }
    eq(name, "message") || eq(name, "call") || eq(name, "kind")
}

/// Runtime half of the reserved-name check, for computed field names (the
/// macros' `expr` path and [`RConditionError::data`] on user types). Panics
/// when `name` is reserved; the panic becomes a plain `rust_error` in R,
/// replacing the former silent overwrite.
#[track_caller]
pub fn check_condition_field_name(name: String) -> String {
    if is_reserved_condition_field(&name) {
        panic!(
            "condition data field `{name}` is reserved: `message`, `call` and `kind` are the \
             condition's own slots; rename the field."
        );
    }
    name
}

/// Apply [`check_condition_field_name`] to every field of a payload.
#[track_caller]
pub fn check_condition_data(data: Option<ConditionData>) -> Option<ConditionData> {
    data.map(|fields| {
        fields
            .into_iter()
            .map(|(name, value)| (check_condition_field_name(name), value))
            .collect()
    })
}

// endregion

// region: Classed `Result` errors — RConditionError + RError

/// Give a `Result<T, E>` error type an R class vector and structured fields.
///
/// Every `#[miniextendr]` function or method returning `Result<T, E>` raises
/// `Err(e)` as an R error. By default the error is a bare `rust_error` whose
/// message is `format!("{e:?}")`. Implement this trait for `E` (or return
/// [`RError`], which implements it) and the `Err` arm instead raises
/// `c(<class()…>, "rust_error", "simpleError", "error", "condition")` with
/// every `data()` field readable as `e$<name>`, so a thiserror-style error enum
/// can keep `?` composition *and* give R handlers something to dispatch on:
///
/// ```ignore
/// use miniextendr_api::condition::{ConditionData, RConditionError};
///
/// #[derive(Debug, thiserror::Error)]
/// pub enum PkgError {
///     #[error("column `{column}` is missing")]
///     MissingColumn { column: String },
///     #[error("{value} exceeds {max}")]
///     TooLarge { value: f64, max: f64 },
/// }
///
/// impl RConditionError for PkgError {
///     fn message(&self) -> String { self.to_string() }
///     fn class(&self) -> Vec<String> {
///         let member = match self {
///             PkgError::MissingColumn { .. } => "pkg_error_missing_column",
///             PkgError::TooLarge { .. } => "pkg_error_too_large",
///         };
///         vec![member.into(), "pkg_error".into()]
///     }
///     fn data(&self) -> Option<ConditionData> {
///         Some(match self {
///             PkgError::MissingColumn { column } => vec![("column".into(), column.clone().into())],
///             PkgError::TooLarge { value, max } => vec![("value".into(), (*value).into()), ("max".into(), (*max).into())],
///         })
///     }
/// }
///
/// #[miniextendr]
/// pub fn check(value: f64) -> Result<f64, PkgError> { /* uses `?` freely */ }
/// ```
///
/// ```r
/// tryCatch(check(1e9), pkg_error_too_large = function(e) e$max)   # member
/// tryCatch(check(1e9), pkg_error = function(e) conditionMessage(e)) # family
/// ```
///
/// Detection is by trait, not by attribute: the generated `Err` arm probes
/// `E: RConditionError` first and falls back to the `Debug` rendering
/// otherwise, so existing `Result<T, String>` / `Result<T, MyDebugError>`
/// functions are unchanged. The condition's `kind` stays `"result_err"`.
///
/// `data()` field names must not be `message`, `call` or `kind` (see
/// [`RESERVED_CONDITION_FIELDS`]); a reserved name raises a plain
/// `rust_error` explaining the clash, so rename such a field at the source.
pub trait RConditionError {
    /// The condition message (`conditionMessage(e)`).
    fn message(&self) -> String;
    /// User classes, most specific first; empty for none.
    fn class(&self) -> Vec<String> {
        Vec::new()
    }
    /// Structured fields spliced into the condition object (`e$<name>`).
    fn data(&self) -> Option<ConditionData> {
        None
    }
}

/// A ready-made classed error value for `Result<T, RError>` returns.
///
/// Carries a message, an optional class vector, structured fields and an
/// optional field-name prefix, and implements [`RConditionError`]. Any
/// `std::error::Error` converts into it with `?` / [`From`] (the message keeps
/// the `caused by:` chain, like [`AsRError`]), after which the builder methods
/// add the R-facing parts:
///
/// ```ignore
/// use miniextendr_api::condition::RError;
///
/// #[miniextendr]
/// pub fn parse_port(s: &str) -> Result<i32, RError> {
///     let port: i32 = s
///         .parse()
///         .map_err(|e| RError::from(e).class(["pkg_bad_port", "pkg_error"]).data("input", s))?;
///     Ok(port)
/// }
/// ```
///
/// ```r
/// e <- tryCatch(parse_port("x"), pkg_bad_port = function(e) e)
/// e$input          # "x"
/// class(e)[1:2]    # "pkg_bad_port" "pkg_error"
/// ```
///
/// `RError` deliberately does **not** implement `std::error::Error`: that
/// keeps the blanket `From<E: Error>` coherent. It does implement `Display`
/// (the message), so it also works with `#[miniextendr(unwrap_in_r)]`.
#[derive(Debug, Clone)]
pub struct RError {
    message: String,
    class: Vec<String>,
    data: ConditionData,
}

impl RError {
    /// A classless error with `message`.
    pub fn new(message: impl Into<String>) -> Self {
        RError {
            message: message.into(),
            class: Vec::new(),
            data: Vec::new(),
        }
    }

    /// Append user classes (most specific first). Accepts one string or a
    /// vector, see [`ConditionClass`].
    pub fn class(mut self, class: impl ConditionClass) -> Self {
        self.class.extend(class.into_condition_class());
        self
    }

    /// Attach a structured field readable as `e$<name>`. Any value with an
    /// [`RValue`](crate::RValue) `From` impl works. The name must not be
    /// `message`, `call` or `kind`; see [`RESERVED_CONDITION_FIELDS`].
    pub fn data(mut self, name: impl Into<String>, value: impl Into<crate::RValue>) -> Self {
        self.data.push((name.into(), value.into()));
        self
    }

    /// The message.
    pub fn message_str(&self) -> &str {
        &self.message
    }

    /// The user classes, most specific first.
    pub fn classes(&self) -> &[String] {
        &self.class
    }

    /// The structured fields as attached (unprefixed).
    pub fn fields(&self) -> &ConditionData {
        &self.data
    }
}

impl<E: std::error::Error> From<E> for RError {
    /// Message = the error's `Display` plus its `source()` chain, one
    /// `caused by:` line per link (same rendering as [`AsRError`]).
    fn from(err: E) -> Self {
        let mut message = err.to_string();
        let mut current: &dyn std::error::Error = &err;
        while let Some(source) = current.source() {
            message.push_str("\n  caused by: ");
            message.push_str(&source.to_string());
            current = source;
        }
        RError::new(message)
    }
}

impl std::fmt::Display for RError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl RConditionError for RError {
    fn message(&self) -> String {
        self.message.clone()
    }
    fn class(&self) -> Vec<String> {
        self.class.clone()
    }
    fn data(&self) -> Option<ConditionData> {
        (!self.data.is_empty()).then(|| self.data.clone())
    }
}

/// The three parts the generated `Err` arm hands to
/// [`crate::error_value::result_err_condition_value`].
#[doc(hidden)]
pub struct ErrParts {
    pub message: String,
    pub class: Vec<String>,
    pub data: Option<ConditionData>,
}

/// Autoref-specialisation probe, preferred arm: `E: RConditionError`.
///
/// The generated code calls `(&e).__mx_err_parts()` with both probe traits in
/// scope. Method resolution tries the receiver type `&E` by value first, which
/// only this impl (on `E`, taking `&self`) satisfies; the `Debug` fallback on
/// `&E` needs one more auto-ref and is only reached when `E` does not
/// implement [`RConditionError`].
#[doc(hidden)]
pub trait ErrPartsClassed {
    fn __mx_err_parts(&self) -> ErrParts;
}

impl<E: RConditionError> ErrPartsClassed for E {
    #[track_caller]
    fn __mx_err_parts(&self) -> ErrParts {
        ErrParts {
            message: self.message(),
            class: self.class(),
            data: check_condition_data(self.data()),
        }
    }
}

/// Autoref-specialisation probe, fallback arm: any `E: Debug` (the historical
/// `format!("{e:?}")` rendering, no class, no data).
#[doc(hidden)]
pub trait ErrPartsDebug {
    fn __mx_err_parts(&self) -> ErrParts;
}

impl<E: std::fmt::Debug> ErrPartsDebug for &E {
    fn __mx_err_parts(&self) -> ErrParts {
        ErrParts {
            message: format!("{self:?}"),
            class: Vec::new(),
            data: None,
        }
    }
}

/// Internal: the `Err(e)` probe used by generated wrappers. Not public API.
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_result_err_parts {
    ($e:expr) => {{
        #[allow(unused_imports)]
        use $crate::condition::{ErrPartsClassed as _, ErrPartsDebug as _};
        (&$e).__mx_err_parts()
    }};
}

// endregion

// region: Serde-tagged Result errors (#[miniextendr(serde_error)])

/// Build the `Err`-arm parts for `#[miniextendr(serde_error)]`.
///
/// Message from `Display`; classes `[<prefix>_<variant>, <prefix>]` when the
/// serialized value carries a variant, `[<prefix>]` otherwise; data from the
/// serialized fields (checked against the reserved names). `tag` names the
/// internally-tagged discriminator field (`#[serde(tag = "kind")]`), which is
/// consumed as the member class rather than spliced as data; externally tagged
/// enums (serde's default) yield the variant name directly. See
/// `docs/CONDITIONS.md`, "Classed `Result` errors".
///
/// Field control, applied to the payload fields before the reserved-name
/// check (`serde_error(skip(..), rename(..))`, #1457):
///
/// - `skip`: fields with these names are dropped.
/// - `rename`: `(from, to)` pairs; a field named `from` is spliced as `to`.
/// - A remaining field named `message` whose value is the single string equal
///   to the `Display` output is dropped: it duplicates the condition's own
///   `message` slot, so a wrapped parser error of the shape
///   `Variant { message: String }` reaches R without any option.
///
/// Names that a given variant does not carry are a no-op for that variant.
/// Any other collision with `message`, `call` or `kind` still panics via
/// [`check_condition_data`].
///
/// After `skip` / `rename` the field names must be distinct: a `rename` whose
/// target is a name the variant already carries (or two payload fields that
/// serialize under one name) would give the condition two entries with the
/// same name, of which R's `e$name` reads only the first. That panics too,
/// naming the field and the rename that produced it (#1459).
///
/// Serialization failing (a map with non-string keys, a custom `Serialize`
/// erroring) panics with both failures in the message: the original error
/// text must not be lost behind the reporting problem.
#[cfg(feature = "serde")]
#[doc(hidden)]
#[track_caller]
pub fn serde_err_parts<E: ?Sized + ::serde::Serialize + std::fmt::Display>(
    e: &E,
    tag: &str,
    prefix: &str,
    skip: &[&str],
    rename: &[(&str, &str)],
) -> ErrParts {
    let message = e.to_string();
    let (member, fields) = match crate::serde::rvalue_ser::tagged_parts(e, tag) {
        Ok(parts) => parts,
        Err(err) => panic!(
            "#[miniextendr(serde_error)]: serializing `{}` for its condition data failed ({err}); \
             the original error was: {message}",
            std::any::type_name::<E>()
        ),
    };
    let mut class = Vec::with_capacity(2);
    if let Some(member) = member {
        class.push(format!("{prefix}_{member}"));
    }
    class.push(prefix.to_string());
    let fields: ConditionData = fields
        .into_iter()
        .filter_map(|(name, value)| {
            if skip.contains(&name.as_str()) {
                return None;
            }
            if let Some((_, to)) = rename.iter().find(|(from, _)| *from == name) {
                return Some(((*to).to_string(), value));
            }
            let echoes_display = name == "message"
                && matches!(&value, crate::RValue::Character(v)
                    if v.len() == 1 && v[0].as_deref() == Some(message.as_str()));
            (!echoes_display).then_some((name, value))
        })
        .collect();
    check_distinct_field_names(&fields, rename);
    ErrParts {
        message,
        class,
        data: check_condition_data((!fields.is_empty()).then_some(fields)),
    }
}

/// Panic when two payload fields share a name after `skip` / `rename`.
///
/// R's `e$<name>` returns the first entry with that name and the second is
/// unreachable, which is the silent-overwrite failure the reserved-name rule
/// exists to remove (#1440). Neither first-wins nor last-wins would report it.
#[cfg(feature = "serde")]
#[track_caller]
fn check_distinct_field_names(fields: &[(String, crate::RValue)], rename: &[(&str, &str)]) {
    let mut seen = std::collections::HashSet::with_capacity(fields.len());
    for (name, _) in fields {
        if seen.insert(name.as_str()) {
            continue;
        }
        let renames: Vec<String> = rename
            .iter()
            .filter(|(_, to)| *to == name)
            .map(|(from, to)| format!("`rename({from} = \"{to}\")`"))
            .collect();
        if renames.is_empty() {
            panic!(
                "#[miniextendr(serde_error)]: condition data field `{name}` appears twice in the \
                 serialized payload; R's `e${name}` would read only the first. Give the fields \
                 distinct names or `skip` one of them."
            );
        }
        panic!(
            "#[miniextendr(serde_error)]: condition data field `{name}` appears twice: {} targets \
             a name this variant already carries; R's `e${name}` would read only the first. Pick \
             another target or `skip` the existing field.",
            renames.join(" and ")
        );
    }
}

// endregion

// region: from_tagged_sexp + repanic_if_rust_error — shim re-panic helpers

impl RCondition {
    /// Reconstruct an [`RCondition::Error`] from a tagged SEXP produced by
    /// [`crate::error_value::make_rust_condition_value`].
    ///
    /// Returns `Some(RCondition)` when `sexp` has class `"rust_condition_value"` AND
    /// the `"__rust_condition__"` attribute is `TRUE`. Returns `None` for all other
    /// SEXPs (normal return values, `R_NilValue`, etc.).
    ///
    /// Reconstructs the matching variant for each kind: `"error"`/`"panic"`/
    /// `"result_err"`/`"none_err"`/`"other_rust_error"` → [`RCondition::Error`];
    /// `"warning"` → [`RCondition::Warning`]; `"message"` → [`RCondition::Message`];
    /// `"condition"` → [`RCondition::Condition`]. Unknown kinds degrade to
    /// [`RCondition::Error`] with the kind string prefixed to the message.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread.
    pub unsafe fn from_tagged_sexp(sexp: crate::SEXP) -> Option<Self> {
        use crate::SexpExt;
        use crate::from_r::TryFromSexp;

        // Use SexpExt::inherits_class — wraps Rf_inherits, already main-thread.
        if !sexp.inherits_class(c"rust_condition_value") {
            return None;
        }

        // Belt-and-suspenders PROTECT across the full inspection window. The reads
        // below are nominally non-allocating, but R-devel's GC is aggressive enough
        // (see MEMORY.md "Common gotchas") that a defensive guard is cheap and
        // closes the door on subtle regressions if the read path ever changes.
        let _guard = unsafe { crate::gc_protect::OwnedProtect::new(sexp) };

        // Verify the __rust_condition__ marker attribute is TRUE (a length-1 LGLSXP
        // with value 1). This guards against coincidental class attribute collisions.
        let attr_sym = crate::cached_class::rust_condition_attr_symbol();
        let marker = sexp.get_attr(attr_sym);
        // marker should be a scalar logical TRUE: is_logical() and logical_elt(0) == 1
        if !marker.is_logical() || marker.logical_elt(0) != 1 {
            return None;
        }

        // It's a tagged SEXP. Read the elements.
        // Both 3-element (legacy) and 4-element (condition) forms have:
        //   [0] = error message (STRSXP)
        //   [1] = kind string (STRSXP)
        //   [2] = class name or NULL (only in 4-element form; absent in legacy)

        let len = sexp.len();

        // Defense-in-depth: a tagged SEXP must have at least the message and kind
        // slots. inherits_class + __rust_condition__ marker should already imply this,
        // but a corrupted/spoofed SEXP that satisfies both checks shouldn't OOB
        // the vector_elt reads below.
        if len < 2 {
            return None;
        }

        let msg_sexp = sexp.vector_elt(0);
        let msg: String = msg_sexp
            .string_elt_str(0)
            .unwrap_or("<invalid error message>")
            .to_string();

        let kind_sexp = sexp.vector_elt(1);
        let kind: &str = kind_sexp
            .string_elt_str(0)
            .unwrap_or(crate::error_value::kind::PANIC);

        // Class slot is element [2] in the 4-element form (NULL in legacy form):
        // a STRSXP of any length (the user class vector, family last).
        let class: Vec<String> = if len >= 4 {
            let class_sexp = sexp.vector_elt(2);
            if class_sexp.is_nil() || !class_sexp.is_character() {
                Vec::new()
            } else {
                (0..class_sexp.len() as isize)
                    .filter_map(|i| class_sexp.string_elt_str(i).map(str::to_string))
                    .collect()
            }
        } else {
            Vec::new()
        };

        use crate::error_value::kind as kind_const;

        // Slot [4] is the optional named-list condition data, present when `len >= 5`.
        //
        // Each field value is decoded through the single SEXP→owned-tree walker,
        // [`RValue::try_from_sexp`], so structured fields survive the cross-package
        // trait-ABI re-panic path (`repanic_if_rust_error`): the consumer's outer
        // `with_r_unwind_protect` guard rebuilds the tagged SEXP from the
        // reconstructed `RCondition`, which now carries the data — so `e$field_name`
        // is accessible in R handlers even when the error crossed a package boundary.
        //
        // `RValue` is NA-aware (logical/integer/character carry `None`; double
        // carries the `NA_REAL` bit), so NA-bearing fields now round-trip faithfully
        // rather than being dropped. Fields whose name is missing/empty, or whose
        // value is not R data (closures, environments, …, which `try_from_sexp`
        // rejects) are dropped — safe degradation that preserves message/class/kind.
        //
        // All reads here are non-allocating copies into owned Rust values, so no new
        // SEXPs are created and the existing `_guard` OwnedProtect suffices.
        let data: Option<ConditionData> = if len >= 5 {
            let data_sexp = sexp.vector_elt(4);
            if data_sexp.is_nil() || !data_sexp.is_list() {
                None
            } else {
                let data_len = data_sexp.len();
                let names_sexp = data_sexp.get_names();
                let mut fields: ConditionData = Vec::with_capacity(data_len);
                for i in 0..data_len as isize {
                    // Read the field name from the names attribute. If missing/empty, skip.
                    let name: String = if names_sexp.is_nil() || !names_sexp.is_character() {
                        continue;
                    } else {
                        match names_sexp.string_elt_str(i) {
                            Some(s) if !s.is_empty() => s.to_string(),
                            _ => continue,
                        }
                    };
                    if let Ok(value) = crate::RValue::try_from_sexp(data_sexp.vector_elt(i)) {
                        fields.push((name, value));
                    }
                }
                if fields.is_empty() {
                    None
                } else {
                    Some(fields)
                }
            }
        } else {
            None
        };

        let cond = match kind {
            kind_const::ERROR
            | kind_const::PANIC
            | kind_const::RESULT_ERR
            | kind_const::NONE_ERR
            | kind_const::OTHER_RUST_ERROR => RCondition::Error {
                message: msg,
                class,
                data,
            },
            kind_const::WARNING => RCondition::Warning {
                message: msg,
                class,
                data,
            },
            kind_const::MESSAGE => RCondition::Message { message: msg, data },
            kind_const::CONDITION => RCondition::Condition {
                message: msg,
                class,
                data,
            },
            other => {
                // Unknown kind — degrade to error
                RCondition::Error {
                    message: format!("[{other}] {msg}"),
                    class,
                    data,
                }
            }
        };
        Some(cond)
    }
}

/// Inspect a SEXP returned by a trait-ABI vtable shim and, if it is a tagged
/// error value, re-panic with the reconstructed [`RCondition`].
///
/// This is the "re-panic at the View boundary" step of Approach 1 from the
/// issue-345 plan. The caller (a generated View method wrapper) does:
///
/// ```ignore
/// let result = { vtable_call };
/// ::miniextendr_api::trait_abi::repanic_if_rust_error(result);
/// // ... convert result normally if we reach here
/// ```
///
/// When `sexp` is a tagged error value:
/// - `RCondition::Error` / `RCondition::Warning` / etc. → `panic_any!(cond)`.
///   The outer `with_r_unwind_protect` in the consumer's C entry point will
///   catch this and produce a tagged SEXP for the consumer's R wrapper.
///
/// When `sexp` is a normal value: this is a no-op.
///
/// # Safety
///
/// Must be called from R's main thread. `sexp` must be a valid (possibly
/// tagged) SEXP.
pub unsafe fn repanic_if_rust_error(sexp: crate::SEXP) {
    if let Some(cond) = unsafe { RCondition::from_tagged_sexp(sexp) } {
        std::panic::panic_any(cond);
    }
}

// endregion

// region: AsRError struct — wraps std::error::Error for Result returns

/// Structured error wrapper that preserves the `std::error::Error` cause chain.
///
/// When displayed, formats the error message with its full source chain:
/// ```text
/// top-level message
///   caused by: middle error
///   caused by: root cause
/// ```
///
/// Implements `From<E>` so it works with `?` and `.map_err(AsRError)`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::condition::AsRError;
/// use std::num::ParseIntError;
///
/// #[miniextendr]
/// fn parse_number(s: &str) -> Result<i32, AsRError<ParseIntError>> {
///     s.parse::<i32>().map_err(AsRError)
/// }
/// ```
pub struct AsRError<E: std::error::Error>(pub E);

impl<E: std::error::Error> From<E> for AsRError<E> {
    #[inline]
    fn from(err: E) -> Self {
        AsRError(err)
    }
}

impl<E: std::error::Error> std::fmt::Display for AsRError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Write the top-level message
        write!(f, "{}", self.0)?;

        // Walk the cause chain
        let mut current: &dyn std::error::Error = &self.0;
        while let Some(source) = current.source() {
            write!(f, "\n  caused by: {source}")?;
            current = source;
        }

        Ok(())
    }
}

impl<E: std::error::Error> std::fmt::Debug for AsRError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AsRError<{}>({})", std::any::type_name::<E>(), self)
    }
}

impl<E: std::error::Error> AsRError<E> {
    /// Get the inner error.
    #[inline]
    pub fn into_inner(self) -> E {
        self.0
    }

    /// Get the Rust type name of the wrapped error (for programmatic matching).
    #[inline]
    pub fn rust_type_name(&self) -> &'static str {
        std::any::type_name::<E>()
    }

    /// Collect the full cause chain as a `Vec<String>`.
    pub fn cause_chain(&self) -> Vec<String> {
        let mut chain = vec![self.0.to_string()];
        let mut current: &dyn std::error::Error = &self.0;
        while let Some(source) = current.source() {
            chain.push(source.to_string());
            current = source;
        }
        chain
    }
}

// endregion

// region: Tests — macro grammar + payload contents (no R runtime needed)

#[cfg(test)]
mod condition_macro_tests {
    use super::{ConditionData, RCondition};
    use crate::RValue;

    /// Catch the `panic_any(RCondition)` raised by a macro invocation and
    /// return the payload. No R runtime needed — the macros panic before any
    /// R API call.
    fn catch(f: impl FnOnce() + std::panic::UnwindSafe) -> RCondition {
        let payload = std::panic::catch_unwind(f).expect_err("macro must panic");
        *payload
            .downcast::<RCondition>()
            .expect("payload must be RCondition")
    }

    fn assert_data(data: &Option<ConditionData>, expected: &[(&str, RValue)]) {
        let data = data.as_ref().expect("data must be Some");
        assert_eq!(data.len(), expected.len());
        for ((name, value), (exp_name, exp_value)) in data.iter().zip(expected) {
            assert_eq!(name, exp_name);
            // RValue has no PartialEq (f64); compare via Debug.
            assert_eq!(format!("{value:?}"), format!("{exp_value:?}"));
        }
    }

    #[test]
    fn error_class_vector_and_keyed_data() {
        let cond = catch(|| {
            crate::error!(
                class = ["member", "family"],
                data = { rule = 1, detail = "inner" },
                "layered"
            )
        });
        match cond {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "layered");
                assert_eq!(class, vec!["member", "family"]);
                assert_data(
                    &data,
                    &[
                        ("rule", RValue::Integer(vec![Some(1)])),
                        ("detail", RValue::Character(vec![Some("inner".to_string())])),
                    ],
                );
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn error_class_vector_from_vec_string() {
        let classes = vec!["a".to_string(), "b".to_string()];
        let cond = catch(move || crate::warning!(class = classes, "w"));
        match cond {
            RCondition::Warning { class, .. } => assert_eq!(class, vec!["a", "b"]),
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn reserved_field_name_runtime_check_panics() {
        let name = String::from("kind");
        let payload = std::panic::catch_unwind(move || crate::error!(data = (name, 1), "x"))
            .expect_err("must panic");
        let msg = payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .expect("plain panic message");
        assert!(msg.contains("reserved"), "got: {msg}");
        assert!(msg.contains("rename"), "got: {msg}");
    }

    #[cfg(feature = "serde")]
    mod serde_err_parts {
        use super::super::serde_err_parts;
        use crate::RValue;
        use ::serde::Serialize;

        #[derive(Serialize)]
        #[serde(tag = "kind", rename_all = "snake_case")]
        enum Tagged {
            MissingField { field: String },
            Io,
        }

        impl std::fmt::Display for Tagged {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Tagged::MissingField { field } => write!(f, "field `{field}` is missing"),
                    Tagged::Io => f.write_str("io"),
                }
            }
        }

        #[derive(Serialize)]
        struct Plain {
            code: i32,
        }

        impl std::fmt::Display for Plain {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "plain {}", self.code)
            }
        }

        #[derive(Serialize)]
        enum Clash {
            Bad { kind: String },
        }

        impl std::fmt::Display for Clash {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("clash")
            }
        }

        #[test]
        fn internally_tagged_variant_becomes_member_class_and_fields_become_data() {
            let parts = serde_err_parts(
                &Tagged::MissingField { field: "id".into() },
                "kind",
                "engine",
                &[],
                &[],
            );
            assert_eq!(parts.message, "field `id` is missing");
            assert_eq!(parts.class, ["engine_missing_field", "engine"]);
            let data = parts.data.expect("fields become data");
            assert_eq!(data.len(), 1);
            assert_eq!(data[0].0, "field");
            assert!(
                matches!(&data[0].1, RValue::Character(v) if v == &vec![Some("id".to_string())])
            );
        }

        #[test]
        fn unit_variant_has_classes_but_no_data() {
            let parts = serde_err_parts(&Tagged::Io, "kind", "engine", &[], &[]);
            assert_eq!(parts.class, ["engine_io", "engine"]);
            assert!(parts.data.is_none());
        }

        #[test]
        fn value_without_variant_gets_only_the_family_class() {
            let parts = serde_err_parts(&Plain { code: 3 }, "kind", "engine", &[], &[]);
            assert_eq!(parts.message, "plain 3");
            assert_eq!(parts.class, ["engine"]);
            let data = parts.data.expect("struct fields become data");
            assert_eq!(data[0].0, "code");
        }

        #[test]
        fn reserved_payload_field_is_rejected() {
            let err = std::panic::AssertUnwindSafe(Clash::Bad { kind: "x".into() });
            let payload =
                std::panic::catch_unwind(move || serde_err_parts(&err.0, "type", "p", &[], &[]))
                    .err()
                    .expect("must panic");
            let msg = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default();
            assert!(msg.contains("reserved"), "{msg}");
            assert!(msg.contains("`kind`"), "{msg}");
        }

        /// The wrapped-parser-error shape (#1457): a `message` payload field.
        #[derive(Serialize)]
        #[serde(tag = "kind", rename_all = "snake_case")]
        enum Wrapped {
            /// `Display` adds context, so `message` is not redundant.
            Parse {
                message: String,
                line: u32,
            },
            /// `Display` is the field verbatim.
            Echo {
                message: String,
            },
            Bad {
                call: String,
            },
        }

        impl std::fmt::Display for Wrapped {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Wrapped::Parse { message, line } => write!(f, "line {line}: {message}"),
                    Wrapped::Echo { message } => f.write_str(message),
                    Wrapped::Bad { .. } => f.write_str("bad"),
                }
            }
        }

        fn panic_message(f: impl FnOnce() + std::panic::UnwindSafe) -> String {
            let payload = std::panic::catch_unwind(f).expect_err("must panic");
            payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default()
        }

        fn names(parts: &super::super::ErrParts) -> Vec<&str> {
            parts
                .data
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|(n, _)| n.as_str())
                .collect()
        }

        #[test]
        fn message_field_equal_to_display_is_dropped_without_options() {
            let parts = serde_err_parts(
                &Wrapped::Echo {
                    message: "boom".into(),
                },
                "kind",
                "p",
                &[],
                &[],
            );
            assert_eq!(parts.message, "boom");
            assert_eq!(parts.class, ["p_echo", "p"]);
            assert!(parts.data.is_none(), "{:?}", parts.data);
        }

        #[test]
        fn message_field_differing_from_display_is_still_rejected() {
            let err = std::panic::AssertUnwindSafe(Wrapped::Parse {
                message: "unexpected token".into(),
                line: 3,
            });
            let msg = panic_message(move || {
                serde_err_parts(&err.0, "kind", "p", &[], &[]);
            });
            assert!(msg.contains("reserved"), "{msg}");
            assert!(msg.contains("`message`"), "{msg}");
        }

        #[test]
        fn skip_drops_the_named_field_and_is_a_no_op_elsewhere() {
            let parts = serde_err_parts(
                &Wrapped::Parse {
                    message: "unexpected token".into(),
                    line: 3,
                },
                "kind",
                "p",
                &["message", "absent"],
                &[],
            );
            assert_eq!(parts.message, "line 3: unexpected token");
            assert_eq!(parts.class, ["p_parse", "p"]);
            assert_eq!(names(&parts), ["line"]);

            // `skip` runs before the Display-equality rule; same outcome.
            let parts = serde_err_parts(
                &Wrapped::Echo {
                    message: "boom".into(),
                },
                "kind",
                "p",
                &["message"],
                &[],
            );
            assert!(parts.data.is_none());

            // A variant without the skipped field but with another reserved
            // name still fails the reserved-name check.
            let err = std::panic::AssertUnwindSafe(Wrapped::Bad { call: "f()".into() });
            let msg = panic_message(move || {
                serde_err_parts(&err.0, "kind", "p", &["message"], &[]);
            });
            assert!(msg.contains("`call`"), "{msg}");
        }

        #[test]
        fn rename_splices_the_field_under_the_new_name() {
            let parts = serde_err_parts(
                &Wrapped::Parse {
                    message: "unexpected token".into(),
                    line: 3,
                },
                "kind",
                "p",
                &[],
                &[("message", "detail"), ("absent", "other")],
            );
            assert_eq!(names(&parts), ["detail", "line"]);
            let data = parts.data.expect("data");
            assert!(matches!(
                &data[0].1,
                RValue::Character(v) if v == &vec![Some("unexpected token".to_string())]
            ));

            // `rename` wins over the Display-equality drop: the caller asked
            // for the field explicitly.
            let parts = serde_err_parts(
                &Wrapped::Echo {
                    message: "boom".into(),
                },
                "kind",
                "p",
                &[],
                &[("message", "detail")],
            );
            assert_eq!(names(&parts), ["detail"]);
        }

        /// #1459: a `rename` target the variant already carries would give the
        /// condition two `line` entries; R reads only the first.
        #[test]
        fn rename_onto_a_name_the_variant_carries_panics() {
            let err = std::panic::AssertUnwindSafe(Wrapped::Parse {
                message: "unexpected token".into(),
                line: 3,
            });
            let msg = panic_message(move || {
                serde_err_parts(&err.0, "kind", "p", &[], &[("message", "line")]);
            });
            assert!(msg.contains("field `line` appears twice"), "{msg}");
            assert!(msg.contains("`rename(message = \"line\")`"), "{msg}");

            // The same attribute is fine on a variant without `line`.
            let parts = serde_err_parts(
                &Wrapped::Echo {
                    message: "boom".into(),
                },
                "kind",
                "p",
                &[],
                &[("message", "line")],
            );
            assert_eq!(names(&parts), ["line"]);
        }

        /// A target that only *another* variant carries is no collision, and
        /// `skip` frees a name for `rename` because both run before the check.
        #[test]
        fn rename_target_is_checked_per_variant_after_skip() {
            let parts = serde_err_parts(
                &Wrapped::Bad { call: "f()".into() },
                "kind",
                "p",
                &[],
                &[("call", "line")],
            );
            assert_eq!(names(&parts), ["line"]);

            let parts = serde_err_parts(
                &Wrapped::Parse {
                    message: "unexpected token".into(),
                    line: 3,
                },
                "kind",
                "p",
                &["line"],
                &[("message", "line")],
            );
            assert_eq!(names(&parts), ["line"]);
            let data = parts.data.expect("data");
            assert!(matches!(
                &data[0].1,
                RValue::Character(v) if v == &vec![Some("unexpected token".to_string())]
            ));
        }
    }

    #[test]
    fn rerror_reserved_field_rejected_at_err_arm() {
        use super::RError;
        let err = std::panic::AssertUnwindSafe(RError::new("m").data("kind", 2));
        let payload = std::panic::catch_unwind(move || crate::__mx_result_err_parts!(err.0))
            .err()
            .expect("must panic");
        let msg = payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .expect("plain panic message");
        assert!(msg.contains("reserved"), "got: {msg}");
        assert!(msg.contains("`kind`"), "got: {msg}");
    }

    #[test]
    fn rerror_builder_and_from_error() {
        use super::{RConditionError, RError};
        let e: RError = "x".parse::<i32>().unwrap_err().into();
        let e = e.class(["member", "family"]).data("input", "x");
        assert!(e.message_str().contains("invalid digit"));
        assert_eq!(RConditionError::class(&e), vec!["member", "family"]);
        let data = RConditionError::data(&e).expect("data");
        assert_eq!(data[0].0, "input");

        assert!(RConditionError::data(&RError::new("m")).is_none());
        assert!(super::is_reserved_condition_field("call"));
        assert!(!super::is_reserved_condition_field("calls"));
    }

    #[test]
    fn error_message_only_backcompat() {
        let cond = catch(|| crate::error!("plain {}", 42));
        match cond {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "plain 42");
                assert!(class.is_empty());
                assert!(data.is_none());
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn error_class_only_backcompat() {
        let cond = catch(|| crate::error!(class = "my_error", "missing field: {}", "x"));
        match cond {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "missing field: x");
                assert_eq!(class, vec!["my_error"]);
                assert!(data.is_none());
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn error_single_data_pair() {
        let value = 41_i32;
        let cond = catch(move || crate::error!(data = ("value", value), "v = {value}"));
        match cond {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "v = 41");
                assert!(class.is_empty());
                assert_data(&data, &[("value", RValue::Integer(vec![Some(41)]))]);
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn error_class_and_data_list_all_value_types() {
        let cond = catch(|| {
            crate::error!(
                class = "validation_error",
                data = [
                    ("value", 1.5),
                    ("code", 7),
                    ("label", "lhs"),
                    ("fatal", false),
                    ("ints", vec![1, 2]),
                    ("reals", vec![0.5_f64]),
                    ("flags", vec![true]),
                    ("labels", vec!["a".to_string()])
                ],
                "out of range"
            )
        });
        match cond {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "out of range");
                assert_eq!(class, vec!["validation_error"]);
                assert_data(
                    &data,
                    &[
                        ("value", RValue::Double(vec![1.5])),
                        ("code", RValue::Integer(vec![Some(7)])),
                        ("label", RValue::Character(vec![Some("lhs".into())])),
                        ("fatal", RValue::Logical(vec![Some(false)])),
                        ("ints", RValue::Integer(vec![Some(1), Some(2)])),
                        ("reals", RValue::Double(vec![0.5])),
                        ("flags", RValue::Logical(vec![Some(true)])),
                        ("labels", RValue::Character(vec![Some("a".into())])),
                    ],
                );
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn warning_with_class_and_data() {
        let cond = catch(|| crate::warning!(class = "trunc", data = ("dropped", 3), "dropped"));
        match cond {
            RCondition::Warning {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "dropped");
                assert_eq!(class, vec!["trunc"]);
                assert_data(&data, &[("dropped", RValue::Integer(vec![Some(3)]))]);
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn message_with_data() {
        let cond = catch(|| crate::message!(data = ("step", 2), "step {}", 2));
        match cond {
            RCondition::Message { message, data } => {
                assert_eq!(message, "step 2");
                assert_data(&data, &[("step", RValue::Integer(vec![Some(2)]))]);
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn condition_with_class_and_data() {
        let cond =
            catch(|| crate::condition!(class = "progress", data = [("n", 10)], "processed {}", 10));
        match cond {
            RCondition::Condition {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "processed 10");
                assert_eq!(class, vec!["progress"]);
                assert_data(&data, &[("n", RValue::Integer(vec![Some(10)]))]);
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn data_list_trailing_comma() {
        let cond = catch(|| crate::error!(data = [("a", 1), ("b", 2),], "msg"));
        match cond {
            RCondition::Error { data, .. } => {
                assert_data(
                    &data,
                    &[
                        ("a", RValue::Integer(vec![Some(1)])),
                        ("b", RValue::Integer(vec![Some(2)])),
                    ],
                );
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    // region: keyed builder sugar (ported from #1044/#995)

    #[test]
    fn keyed_builder_arm_stringifies_idents() {
        let cond = catch(|| crate::error!(data = { value = 42, code = 7 }, "boom"));
        match cond {
            RCondition::Error { data, .. } => {
                assert_data(
                    &data,
                    &[
                        ("value", RValue::Integer(vec![Some(42)])),
                        ("code", RValue::Integer(vec![Some(7)])),
                    ],
                );
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn keyed_builder_arm_trailing_comma_and_mixed_types() {
        let cond = catch(|| {
            crate::warning!(
                class = "trunc",
                data = { dropped = 3, ratio = 0.5_f64, tag = "rows", },
                "dropped some"
            )
        });
        match cond {
            RCondition::Warning { data, class, .. } => {
                assert_eq!(class, vec!["trunc"]);
                assert_data(
                    &data,
                    &[
                        ("dropped", RValue::Integer(vec![Some(3)])),
                        ("ratio", RValue::Double(vec![0.5])),
                        ("tag", RValue::Character(vec![Some("rows".into())])),
                    ],
                );
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    // endregion

    // region: NA-aware + wide-int + debug value types via the macro (#995)

    #[test]
    fn option_scalar_fields_carry_na() {
        let cond = catch(|| {
            crate::error!(
                data = [("present", Some(9_i32)), ("missing", None::<i32>)],
                "opts"
            )
        });
        match cond {
            RCondition::Error { data, .. } => {
                assert_data(
                    &data,
                    &[
                        ("present", RValue::Integer(vec![Some(9)])),
                        ("missing", RValue::Integer(vec![None])),
                    ],
                );
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn vec_option_field_carries_embedded_na() {
        let cond =
            catch(|| crate::error!(data = ("codes", vec![Some(1_i32), None, Some(3)]), "vec"));
        match cond {
            RCondition::Error { data, .. } => {
                assert_data(
                    &data,
                    &[("codes", RValue::Integer(vec![Some(1), None, Some(3)]))],
                );
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn wide_integer_ladder_via_macro() {
        // Fits in i32 → integer; beyond → double.
        let cond = catch(|| {
            crate::error!(
                data = [("small", 42_i64), ("big", 5_000_000_000_i64)],
                "wide"
            )
        });
        match cond {
            RCondition::Error { data, .. } => {
                assert_data(
                    &data,
                    &[
                        ("small", RValue::Integer(vec![Some(42)])),
                        ("big", RValue::Double(vec![5_000_000_000.0])),
                    ],
                );
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn debug_fallback_via_macro() {
        let cond = catch(|| crate::error!(data = ("range", RValue::debug(0..=100)), "dbg"));
        match cond {
            RCondition::Error { data, .. } => {
                assert_data(
                    &data,
                    &[("range", RValue::Character(vec![Some("0..=100".into())]))],
                );
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    // endregion
}

// endregion
