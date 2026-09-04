# Enums and Factors Guide

How to map Rust enums to R factors and character strings.

miniextendr provides two complementary systems for enum-like types:

| System | R Representation | Partial Match | Default | Use Case |
|--------|-----------------|---------------|---------|----------|
| `RFactor` | Factor (integer + levels) | No | - | Categorical data for `table()`, `lm()`, etc. |
| `MatchArg` | Character scalar | Yes | First choice | Parameter validation (`match.arg()` style) |

Both systems pass the enum **by value** (as a factor/string). If instead you want
the enum to be a methods-bearing **instance** — an `ExternalPtr` wrapped with a
class-system `impl` block — see
[CLASS_SYSTEMS.md § Enums: value vs instance](CLASS_SYSTEMS.md#enums-value-vs-instance).
Note the two are **mutually exclusive**: deriving `ExternalPtr` alongside
`MatchArg`/`RFactor` is a compile error (`E0119`, conflicting `IntoR`).

## RFactor: enum as R Factor

Maps a Rust enum to an R factor with levels. Each variant becomes a level.

```rust
#[derive(Copy, Clone, RFactor)]
pub enum Color {
    Red,    // level index 1
    Green,  // level index 2
    Blue,   // level index 3
}
```

Use in functions:

```rust
#[miniextendr]
pub fn describe(color: Color) -> &'static str {
    match color {
        Color::Red => "warm",
        Color::Green => "cool",
        Color::Blue => "cool",
    }
}

#[miniextendr]
pub fn favorite() -> Color {
    Color::Blue
}
```

From R:

```r
describe(factor("Red", levels = c("Red", "Green", "Blue")))
# [1] "warm"

favorite()
# [1] Blue
# Levels: Red Green Blue
```

### Rename Variants

```rust
#[derive(Copy, Clone, RFactor)]
#[r_factor(rename_all = "snake_case")]
pub enum Status {
    InProgress,   // level: "in_progress"
    Completed,    // level: "completed"
    NotStarted,   // level: "not_started"
}

#[derive(Copy, Clone, RFactor)]
pub enum Priority {
    #[r_factor(rename = "lo")]
    Low,
    #[r_factor(rename = "med")]
    Medium,
    #[r_factor(rename = "hi")]
    High,
}
```

Supported `rename_all` values: `snake_case`, `kebab-case`, `lower`, `upper`.

### Factor Vectors

Use `FactorVec<T>` for vectors and `FactorOptionVec<T>` for vectors with NA:

```rust
use miniextendr_api::{FactorVec, FactorOptionVec};

#[miniextendr]
pub fn all_colors() -> FactorVec<Color> {
    FactorVec(vec![Color::Red, Color::Green, Color::Blue])
}

#[miniextendr]
pub fn parse_colors(input: FactorOptionVec<Color>) -> Vec<&'static str> {
    input.0.iter().map(|c| match c {
        Some(Color::Red) => "red",
        Some(Color::Green) => "green",
        Some(Color::Blue) => "blue",
        None => "NA",
    }).collect()
}
```

From R:

```r
all_colors()
# [1] Red   Green Blue
# Levels: Red Green Blue

x <- factor(c("Red", NA, "Blue"), levels = c("Red", "Green", "Blue"))
parse_colors(x)
# [1] "red" "NA"  "blue"
```

### Caching

The `#[derive(RFactor)]` macro generates a `OnceLock`-cached levels STRSXP. The
levels string vector is allocated once and reused for all subsequent conversions,
giving ~4x speedup for single-value conversions.

### Via `#[miniextendr]`

Instead of `#[derive(RFactor)]`, you can use the attribute macro:

```rust
#[miniextendr]
#[derive(Copy, Clone)]
pub enum Color { Red, Green, Blue }
```

These are equivalent. `#[miniextendr]` on a fieldless enum dispatches to the same
RFactor derive internally.

---

## MatchArg: enum as string parameter

Maps a Rust enum to R character strings with `match.arg()` validation. Supports
partial matching and defaults to the first variant when `NULL` is passed.

```rust
#[derive(Copy, Clone, MatchArg)]
pub enum Mode {
    Fast,    // choice: "Fast"
    Safe,    // choice: "Safe"
    Debug,   // choice: "Debug"
}
```

Use in functions:

```rust
#[miniextendr]
pub fn run(mode: Mode) -> String {
    match mode {
        Mode::Fast => "running fast".into(),
        Mode::Safe => "running safe".into(),
        Mode::Debug => "running debug".into(),
    }
}
```

The generated R wrapper shows the choice list directly as the formal default:

```r
run <- function(mode = c("Fast", "Safe", "Debug")) {
  mode <- if (is.factor(mode)) as.character(mode) else mode
  mode <- base::match.arg(mode)
  .Call(C_mypkg_run, mode)
}
```

The enum's `CHOICES` are spliced in at wrapper-gen time (not stored in an R
variable), so `?run` and tab-completion both show the real options. If you set
an explicit `default = "\"Safe\""`, the splice rotates that value to position 1
of the vector — `match.arg(arg)` returns `arg[1]` when `arg` matches the formal
default, so the rotated value becomes the effective default while the rest of
the choices remain visible: `function(mode = c("Safe", "Fast", "Debug"))`. The
default value must be one of the enum's choices; otherwise wrapper generation panics
at write time.

From R:

```r
run("Fast")       # exact match
run("F")          # partial match → "Fast"
run()             # NULL → default (first choice: "Fast")
run("Saf")        # partial match → "Safe"
run("X")          # Error: 'arg' should be one of "Fast", "Safe", "Debug"
```

### Rename Variants

Same syntax as RFactor but with `#[match_arg(...)]`:

```rust
#[derive(Copy, Clone, MatchArg)]
#[match_arg(rename_all = "snake_case")]
pub enum BuildStatus {
    InProgress,    // choice: "in_progress"
    Completed,     // choice: "completed"
}

#[derive(Copy, Clone, MatchArg)]
pub enum Priority {
    #[match_arg(rename = "lo")]  Low,
    #[match_arg(rename = "med")] Medium,
    #[match_arg(rename = "hi")]  High,
}
```

### Via `#[miniextendr]`

```rust
#[miniextendr(match_arg)]
#[derive(Copy, Clone)]
pub enum Mode { Fast, Safe, Debug }
```

### For an Enum You Don't Own (Newtype)

`#[derive(MatchArg)]` has to sit on the enum's declaration, so it cannot be
attached to an enum from a crate that does not depend on miniextendr (the
usual shape when an R package wraps an existing Rust library). Wrap the
foreign enum in a newtype and implement the three traits the derive would
have emitted. `MatchArg` (`miniextendr_api::match_arg::MatchArg`) is
public, and so are the helpers the derive leans on:

```rust
use miniextendr_api::match_arg::MatchArg;
use miniextendr_api::{IntoR, SEXP, SexpError, TryFromSexp};

// Owned by the wrapped library; no miniextendr types anywhere near it.
use wrapped::Interp;

#[derive(Copy, Clone)]
pub struct InterpChoice(pub Interp);

impl MatchArg for InterpChoice {
    const CHOICES: &'static [&'static str] = &["linear", "cubic", "nearest"];

    fn from_choice(choice: &str) -> Option<Self> {
        Some(InterpChoice(match choice {
            "linear" => Interp::Linear,
            "cubic" => Interp::Cubic,
            "nearest" => Interp::Nearest,
            _ => return None,
        }))
    }

    fn to_choice(self) -> &'static str {
        match self.0 {
            Interp::Linear => "linear",
            Interp::Cubic => "cubic",
            Interp::Nearest => "nearest",
        }
    }
}

impl TryFromSexp for InterpChoice {
    type Error = SexpError;
    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        miniextendr_api::match_arg_from_sexp(sexp).map_err(Into::into)
    }
}

impl IntoR for InterpChoice {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> { Ok(self.into_sexp()) }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> { self.try_into_sexp() }
    fn into_sexp(self) -> SEXP { self.to_choice().into_sexp() }
}

#[miniextendr]
pub fn interpolate(#[miniextendr(match_arg)] method: InterpChoice) -> f64 {
    wrapped::run(method.0)
}
```

Everything downstream of the trait is shared with derived enums: the
`match.arg()` prelude, the choices spliced into the formal default
(`function(method = c("linear", "cubic", "nearest"))`), partial matching,
factor input, `several_ok` (`Vec<InterpChoice>`), the `Vec<T>` return path
(via the blanket `IntoRVecElement` bridge), impl-block `match_arg(param)`
attributes, and the auto-injected `@param` choices text. `to_choice` is an
exhaustive `match`, so adding a variant to the wrapped enum without updating
the newtype is a compile error rather than silent drift.
`rpkg/src/rust/match_arg_foreign_tests.rs` is the reference fixture.

### Inline String Choices

For simple cases where you don't need an enum, use `choices(...)` on a `&str` parameter:

```rust
#[miniextendr]
pub fn correlate(
    x: f64, y: f64,
    #[miniextendr(choices("pearson", "kendall", "spearman"))] method: &str,
) -> String {
    format!("method={}, cor={}", method, x * y)
}
```

### Multiple Choices with `several_ok`

R's `match.arg(..., several.ok = TRUE)` accepts multiple values from the choice
list and returns a character vector. miniextendr exposes this with
`several_ok`, which is valid on both `match_arg` and `choices`:

```rust
// Enum: Vec<Mode> - each element validated against MatchArg::CHOICES
#[miniextendr]
pub fn pick_modes(#[miniextendr(match_arg, several_ok)] modes: Vec<Mode>) -> String { ... }

// Inline: Vec<String> - each element validated against the inline list
#[miniextendr]
pub fn pick_metrics(
    #[miniextendr(choices("mean", "median", "sd", "var"), several_ok)] metrics: Vec<String>,
) -> String { ... }
```

Accepted container shapes: `Vec<T>`, `Box<[T]>`, `&[T]`, `[T; N]`, and
`Missing<Vec<T>>` (optional). `several_ok` without `match_arg` or `choices`
is a compile error (no choice list to validate against). `several_ok` on a
scalar type (e.g. `Mode` without a `Vec`) is also a compile error.

Default behavior when the R caller omits the argument: the full choice list,
matching `base::match.arg`. Pass a single string to get partial matching,
pass a character vector to get multiple exact/partial matches.

### On Impl-Block Methods

Rust rejects attribute macros on function parameters inside impl items, so
`match_arg` / `choices` / `several_ok` on impl methods use **method-level**
attributes that name the parameter. This works for all class systems
(`r6`, `env`, `s3`, `s4`, `s7`, `vctrs`) on both constructors and instance
methods:

```rust
#[miniextendr(r6)]
impl Counter {
    // Constructor: first choice becomes the R default.
    #[miniextendr(match_arg(mode))]
    pub fn new(mode: Mode) -> Self { ... }

    // several_ok variant - note the distinct attribute name.
    #[miniextendr(match_arg_several_ok(modes))]
    pub fn reset(&mut self, modes: Vec<Mode>) -> i32 { ... }

    // Inline string choices - pass the list as a comma-separated string.
    #[miniextendr(choices(level = "low, medium, high"))]
    pub fn describe(level: String) -> String { ... }
}
```

Each form generates the same R `match.arg` prelude you get on standalone
functions, including the choices vector as the formal default.

The vctrs class system accepts `match_arg` on its `fn new()` constructor
even though vctrs constructors return a data vector (`Vec<T>`) rather than
`Self`. The vctrs generator recognizes a receiverless `new` as the
constructor regardless of return type.

### Auto-Injected `@param` Docs

When you leave a `match_arg` parameter undocumented, miniextendr fills in
the roxygen `@param` line at write time using the enum's `CHOICES`:

```r
#' @param mode One of "Fast", "Safe", "Debug".
```

This runs for both standalone functions and impl-block methods across every
class system. Explicit `@param` lines you write yourself are preserved
verbatim; only missing entries are auto-generated.

---

### Returning `Vec<Enum>`

Functions can return `Vec<T>` for any `MatchArg` enum. Each variant round-trips
to its choice string:

```rust
#[miniextendr]
pub fn all_modes() -> Vec<Mode> {
    vec![Mode::Fast, Mode::Safe, Mode::Debug]
}
```

```r
all_modes()
# [1] "Fast"  "Safe"  "Debug"
```

This is provided by a blanket `impl<T: MatchArg> IntoR for Vec<T>` in
`miniextendr-api`. No extra derive is required.

---

## MatchArg as Base Trait

`MatchArg` is the base trait for all enum-like types. `RFactor` requires `MatchArg`
as a supertrait, so any `RFactor` type also has `MatchArg::CHOICES`, `from_choice()`,
and `to_choice()`. It can also be implemented by hand on a newtype (see
[For an Enum You Don't Own](#for-an-enum-you-dont-own-newtype)). Use `MatchArg`
as a bound for generic code over both systems:

```rust
use miniextendr_api::MatchArg;

fn describe_choices<T: MatchArg>() -> String {
    T::CHOICES.join(", ")
}

fn lookup<T: MatchArg>(choice: &str) -> Option<T> {
    T::from_choice(choice)
}
```

---

## Comparison Table

| Feature | RFactor | MatchArg |
|---------|---------|----------|
| R storage | `factor(1, levels=c(...))` | `"Fast"` (character) |
| Validation | Type check (is factor with correct levels) | `match.arg()` with partial matching |
| Default on NULL | Error | First choice |
| Vec support | `FactorVec<T>`, `FactorOptionVec<T>` | `Vec<T>` return + `several_ok` inputs |
| Partial matching | No | Yes (`"F"` → `"Fast"`) |
| Factor input | Native | Converted to character first |
| Use case | Categorical data | Parameter selection |

## When to Use Which

**RFactor** when:
- Data is categorical (colors, species, status codes)
- Working with R functions expecting factors (`table()`, `lm()`, `ggplot2`)
- Need vector support with NA handling
- Factor level ordering matters

**MatchArg** when:
- Building an API with string-based options
- Want R's `match.arg()` partial matching and error messages
- Want a default value when the argument is omitted
- Validating user input parameters

## See Also

- [MINIEXTENDR_ATTRIBUTE.md](MINIEXTENDR_ATTRIBUTE.md): `#[miniextendr]` on enums
- [TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md): full type conversion reference
