# S3 Methods Guide

How to implement S3 generics (`print`, `format`, etc.) with `#[miniextendr(s3)]`.

## Quick Example

```rust
use miniextendr_api::{miniextendr, ExternalPtr};

#[derive(ExternalPtr)]
pub struct Person {
    name: String,
    age: i32,
}

#[miniextendr(s3)]
impl Person {
    /// @param name Name of the person.
    /// @param age Age of the person.
    pub fn new(name: String, age: i32) -> Self {
        Person { name, age }
    }

    /// Implements format.Person - returns a formatted string.
    #[miniextendr(generic = "format")]
    pub fn fmt(&self) -> String {
        format!("{} (age {})", self.name, self.age)
    }

    /// Implements print.Person - prints and returns self invisibly.
    #[miniextendr(generic = "print")]
    pub fn show(&mut self) {
        println!("Person: {}, age {}", self.name, self.age);
    }

    /// Custom method: greet.Person
    pub fn greet(&self) -> String {
        format!("Hello, I'm {}!", self.name)
    }
}

// Registration is automatic via #[miniextendr].
```

Generated R code (`.Call()` symbols are prefixed with your crate's name —
`mypkg` here — for webR cross-package uniqueness, see `docs/WEBR.md`):

```r
# Constructor
new_person <- function(name, age) {
  structure(.Call(C_mypkg_Person__new, name, age), class = "Person")
}

# format.Person - returns the string directly
format.Person <- function(x, ...) {
  .Call(C_mypkg_Person__fmt, x)
}

# print.Person - calls Rust, then returns invisible(x)
print.Person <- function(x, ...) {
  .Call(C_mypkg_Person__show, x)
  invisible(x)
}

# greet generic + method
greet <- function(x, ...) UseMethod("greet")
greet.Person <- function(x, ...) {
  .Call(C_mypkg_Person__greet, x)
}
```

## Key Concepts

### Generic Override with `#[miniextendr(generic = "...")]`

By default, the Rust method name becomes the S3 generic name. Use `generic = "..."` to
override this, mapping to a different R generic:

```rust
// Rust method is `fmt`, but R generic is `format`
#[miniextendr(generic = "format")]
pub fn fmt(&self) -> String { ... }

// Rust method is `show`, but R generic is `print`
#[miniextendr(generic = "print")]
pub fn show(&mut self) { ... }
```

This is essential for base R generics like `print` and `format` where you want your Rust
method to have a more descriptive name.

### Dots (`...`)

All S3 method signatures include `...` automatically. You don't need to declare them
in your Rust signature. The generated R wrapper adds `...` to the parameter list:

```r
# Generated: always has (x, ...) or (x, other_params, ...)
format.Person <- function(x, ...) { ... }
greet.Person <- function(x, ...) { ... }
```

If your Rust method takes extra parameters, they appear between `x` and `...`:

```rust
pub fn describe(&self, verbose: bool) -> String { ... }
// → describe.Person <- function(x, verbose, ...) { ... }
```

### Constructor

The constructor is always the method returning `Self`. It generates a function named
`new_<classname_lowercase>()` that wraps the result with `structure(..., class = "ClassName")`:

```rust
pub fn new(name: String, age: i32) -> Self { ... }
// → new_person <- function(name, age) {
//     structure(.Call(C_mypkg_Person__new, name, age), class = "Person")
//   }
```

### Static Methods

Methods without `self` become standalone functions prefixed with the lowercase class name:

```rust
pub fn species() -> String { "Homo sapiens".into() }
// → person_species <- function() { .Call(C_mypkg_Person__species) }
```

## Functional builders (native pipe `|>`)

An idiomatic Rust builder mutates the receiver in place and returns `&mut Self`
so steps chain: `b.set_a(1).set_b(2)`. On the S3 path this maps to a
**pipe-friendly free function**: the generated generic takes the object as its
first argument and returns the (same, mutated) object, so it composes under R's
native pipe operator `|>` (R 4.1+).

```rust
use miniextendr_api::{miniextendr, ExternalPtr};

#[derive(ExternalPtr)]
pub struct GreetingBuilder {
    name: String,
    punctuation: String,
    loud: bool,
}

#[miniextendr(s3)]
impl GreetingBuilder {
    pub fn new() -> Self { /* empty defaults */ }

    // Builder steps: `&mut self -> &mut Self`.
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }
    pub fn set_loud(&mut self, loud: bool) -> &mut Self {
        self.loud = loud;
        self
    }

    // Terminal step: `&self -> T` (a different type), converted to R via `IntoR`.
    pub fn build(&self) -> String { /* render the greeting */ }
}
```

In R, every step is a free function whose first argument is the object, so the
whole pipeline reads naturally:

```r
new_greetingbuilder() |>
  set_name("World") |>
  set_loud(TRUE) |>
  build()
#> [1] "HELLO, WORLD."
```

**Value semantics.** A `&mut self -> &mut Self` step mutates the underlying Rust
value *in place* and the wrapper returns the **same** `ExternalPtr` handle —
there is no clone and no re-wrap, so object identity is preserved
(`identical(x, set_name(x, "a"))` is `TRUE`). This is the same reference
semantics as `&mut self -> ()` mutators (which return the object via the
`Chainable Mutation` strategy); the only difference is that returning
`&mut Self` lets the same method *also* chain in Rust.

The terminal `build()` takes `&self` so the R object stays valid afterwards,
and returns a different type (`String` here) through the usual `IntoR`
conversion.

**Consuming receivers.** Builders written as `fn step(mut self, ..) -> Self`
work too: the wrapper moves the value out of the R handle, calls the step, and
writes the result back into the *same* handle, so identity is preserved
exactly as for `&mut Self`. `fn step(mut self, ..) -> Result<Self, E>` /
`Option<Self>` run on a clone (the type must be `Clone`) and overwrite the
handle only on success, so a failed step leaves the R object as it was. A
consuming method with any other return type (`fn finish(self) -> T`) is a
terminal consume: the value is moved out and converted, and later use of the
handle errors with "was consumed". See
[CLASS_SYSTEMS.md](CLASS_SYSTEMS.md#consuming-receivers-self).

**Fallible in-place steps.** `&mut self -> Result<&mut Self, E>` and
`-> Option<&mut Self>` are recognised as in-place builders as well: `Ok` /
`Some` hands back the same handle, `Err` / `None` raises through the normal
`Result` / `Option` error paths.

> The generated generic is named after the Rust method (`set_name`,
> `set_loud`, `build`), so pick method names that don't collide with existing
> generics or base-R functions in the importing package.

All of the above works on every impl-block class system (S3, Env, R6, S4,
S7): R6 and Env chain through `invisible(self)`, S4 and S7 return the
receiver from the generated generic. `rpkg/src/rust/pipe_builder_tests.rs`
is the cross-system fixture.

## Implementing `print`

R convention: `print()` displays output and returns `invisible(x)` so the object
can be used in pipelines without double-printing.

**Recommended pattern: use `&mut self` returning `()`:**

```rust
#[miniextendr(generic = "print")]
pub fn show(&mut self) {
    println!("Person: {}, age {}", self.name, self.age);
}
```

This generates the `ChainableMutation` return strategy:

```r
print.Person <- function(x, ...) {
  .Call(C_mypkg_Person__show, x)
  invisible(x)
}
```

The `&mut self` + void return triggers `invisible(x)` after the `.Call()`. This
matches the R convention where `print()` returns the object invisibly.

**Why `&mut self`?** The `invisible(x)` pattern is only generated for `&mut self`
methods returning `()`. With `&self` returning `()`, the generated code would be
a bare `.Call(...)` returning `NULL`, which is functional but doesn't follow R convention.
If your print method doesn't actually mutate, using `&mut self` is a pragmatic
choice to get correct R behavior.

**Alternative: `&self` returning `()`:**

```rust
#[miniextendr(generic = "print")]
pub fn show(&self) {
    println!("Person: {}, age {}", self.name, self.age);
}
```

Generates:

```r
print.Person <- function(x, ...) {
  .Call(C_mypkg_Person__show, x)
}
```

This works (the `println!` output appears), but the function returns `NULL` instead
of `invisible(x)`. For most interactive use this is fine, but `y <- print(x)` gives
`NULL` rather than `x`.

## Implementing `format`

R convention: `format()` returns a character vector representation. Many R functions
use `format()` internally (e.g., `paste()`, `cat(format(x))`).

**Recommended pattern: `&self` returning `String`:**

```rust
#[miniextendr(generic = "format")]
pub fn fmt(&self) -> String {
    format!("{} (age {})", self.name, self.age)
}
```

Generates:

```r
format.Person <- function(x, ...) {
  .Call(C_mypkg_Person__fmt, x)
}
```

The string is returned directly (visible), which is correct for `format()`.

**Tip:** Implement `format` even if you also implement `print`. Other R code may call
`format()` on your object (e.g., when building error messages or tibble displays).

## Implementing Both `print` and `format`

A complete type should implement both. The standard R pattern is for `print` to call
`format` internally, but with miniextendr each method dispatches to separate Rust code:

```rust
#[derive(ExternalPtr)]
pub struct Temperature {
    celsius: f64,
}

#[miniextendr(s3)]
impl Temperature {
    pub fn new(celsius: f64) -> Self {
        Temperature { celsius }
    }

    #[miniextendr(generic = "format")]
    pub fn fmt(&self) -> String {
        format!("{:.1}°C", self.celsius)
    }

    #[miniextendr(generic = "print")]
    pub fn show(&mut self) {
        println!("{:.1}°C", self.celsius);
    }
}
```

Usage in R:

```r
t <- new_temperature(36.6)
print(t)    # 36.6°C  (returns invisible(t))
format(t)   # "36.6°C" (returns the string)
cat(format(t), "\n")  # 36.6°C
```

## Standalone S3 methods (class defined elsewhere)

The `#[miniextendr(s3)]`-on-impl pattern above owns both the constructor and the methods: the Rust type `Person` *is* the S3 class. That is the right tool when Rust holds the canonical data. It is the wrong tool when:

- The class is defined in R (an existing `structure(list(), class = "my_thing")` or a package you don't control).
- The class is a vctrs type where the "object" is an R vector with a class attribute, not a Rust struct.
- You want to provide a method for a base R class (`print.data.frame`, `format.POSIXct`) from a Rust extension.

For those cases, drop the impl block and write a plain function:

```rust
use miniextendr_api::{miniextendr, SEXP};

/// Implements `format.percent` in Rust.
#[miniextendr(s3(generic = "format", class = "percent"))]
pub fn format_percent(x: SEXP, _dots: ...) -> Vec<String> {
    let data: &[f64] = unsafe { x.as_slice::<f64>() };
    data.iter().map(|v| format!("{:.1}%", v * 100.0)).collect()
}
```

What the macro does:

1. Emits an R wrapper named `format.percent` (the `<generic>.<class>` convention).
2. Adds `#' @method format percent` to the roxygen block. `devtools::document()` picks that up and writes `S3method(format, percent)` into `NAMESPACE`. You do not call `.S3method()` yourself.
3. Registers the underlying C entry point via `distributed_slice`, same as every other `#[miniextendr]` function.

Your Rust function receives exactly the arguments R dispatches with. For most S3 generics that is `(x, ...)`, so the first param is typed (`SEXP`, or a concrete type with `TryFromSexp`) and the last is `_dots: ...` to absorb the R-level `...`. If the generic takes more positional arguments (`vec_cast(x, to, ...)`), list them in order.

### Double dispatch (vctrs)

A few generics dispatch on two arguments. vctrs calls `vec_ptype2(x, y, ...)` and resolves the method as `vec_ptype2.<class_of_x>.<class_of_y>`. Encode that as a dotted class string:

```rust
#[miniextendr(s3(generic = "vec_ptype2", class = "percent.percent"))]
pub fn vec_ptype2_percent_percent(_x: SEXP, _y: SEXP, _dots: ...) -> SEXP { ... }

#[miniextendr(s3(generic = "vec_cast", class = "percent.double"))]
pub fn vec_cast_percent_double(x: SEXP, _to: SEXP, _dots: ...) -> SEXP { ... }
```

The wrapper name becomes `vec_ptype2.percent.percent`, and the roxygen becomes `#' @method vec_ptype2 percent.percent`. That is what vctrs expects.

### vctrs `@importFrom`

The macro recognizes the vctrs generic names (`vec_ptype_abbr`, `vec_proxy`, `vec_restore`, `vec_ptype2`, `vec_cast`, etc.) and auto-injects `#' @importFrom vctrs <generic>` into the roxygen block. This forces vctrs to load before your method is registered, which is necessary for `R_GetCCallable` lookups (vctrs registers its native generics via ccallable).

For non-vctrs generics from other packages (e.g., `tibble::tbl_sum`), add the import manually:

```rust
/// @importFrom tibble tbl_sum
#[miniextendr(s3(generic = "tbl_sum", class = "my_tbl"))]
pub fn tbl_sum_my_tbl(x: SEXP, _dots: ...) -> Vec<String> { ... }
```

### Constraints

- `s3(...)` requires `class`. `generic` defaults to the Rust function name if omitted, but supplying it explicitly is the convention.
- Cannot be combined with `r_name = "..."`. The S3 naming (`<generic>.<class>`) already fixes the R wrapper name.
- The constructor for the class is out of scope for the method function. Either write it in R, or write a separate `#[miniextendr]` function that returns `structure(x, class = "my_class")` via `new_vctr(...)` or equivalent helpers in `miniextendr_api::vctrs`.

### When to pick which

| You have... | Use |
|---|---|
| A Rust struct that owns the data, methods hang off it | `#[miniextendr(s3)] impl MyType { ... }` |
| An R-side class (vctrs, base R, another package), methods are Rust logic | standalone `#[miniextendr(s3(generic = ..., class = ...))]` functions |
| A vctrs class authored here | `#[miniextendr(vctrs)]` on impl for the constructor + static methods, standalone `s3(...)` functions for the dispatch-on-class-attribute generics |

## Summary Table

| Method | Receiver | Return | Generated R | R Convention |
|--------|----------|--------|-------------|--------------|
| `format` | `&self` | `String` | `.Call(...)` | Returns formatted string (visible) |
| `print` | `&mut self` | `()` | `.Call(...); invisible(x)` | Returns self invisibly |
| `print` | `&self` | `()` | `.Call(...)` | Returns NULL (works but unconventional) |
| custom | `&self` | any | `.Call(...)` | Returns value directly |
| custom | `&mut self` | `()` | `.Call(...); x` | Chainable mutation |
| builder | `&mut self` | `&mut Self` | `.Call(...); x` | In-place builder; returns same handle, pipes under `\|>` |
| builder | `&self` | `&Self` | `.Call(...); x` | Read-only builder step; returns same handle |

## See Also

- [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md): overview of all 6 class systems (env, R6, S3, S4, S7, vctrs)
- [DOTS_TYPED_LIST.md](DOTS_TYPED_LIST.md): using dots and typed_list validation
- [VCTRS.md](VCTRS.md): vctrs-compatible S3 classes with `format` methods
