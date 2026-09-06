---
name: miniextendr-dots
description: "Use when the user asks about handling R's ... (dots/variadic) arguments in Rust, the Dots type, the typed_list! macro, #[miniextendr(dots = typed_list!(...))] attribute sugar, custom dots binding names with name @ ..., optional vs required fields in typed lists, or TypedList accessors."
---

# miniextendr Dots and `typed_list!`

R's `...` (dots) passes an untyped sequence of named or unnamed arguments through a call stack. miniextendr maps `...` to a `&Dots` parameter in Rust and provides the `typed_list!` macro for compile-time-specified runtime validation.

## When to use this skill

- "How do I accept `...` in my Rust function?"
- "What is the `Dots` type?"
- "How do I use `typed_list!`?"
- "What is `name @ ...` syntax?"
- "How do I make a dots field optional?"
- "What does `#[miniextendr(dots = typed_list!(...))]` do?"
- "What error does a mismatched typed list produce?"

## Key concepts

### The `Dots` type

When a `#[miniextendr]` function has `...` in its Rust signature, the macro transforms that position into a `_dots: &Dots` parameter. `_dots` is the default name; the underscore prefix suppresses unused-variable warnings if you do not use the dots in the function body.

`Dots` provides three accessors:

| Method | Returns | Description |
|--------|---------|-------------|
| `as_list()` | `List` | Fast unchecked conversion to a `List` |
| `try_list()` | `Result<List, …>` | Validated conversion |
| `typed(spec)` | `Result<TypedList, TypedListError>` | Validate against a `TypedListSpec` |

### Custom binding name with `name @ ...`

To give the dots parameter a descriptive name instead of `_dots`, use the `name @ ...` syntax:

```rust
#[miniextendr]
pub fn my_func(args @ ...) -> i32 {
    args.as_list().len() as i32
}
```

The `@` annotation is parsed by the `#[miniextendr]` macro. The generated R wrapper still uses `...`; the Rust binding name is local only.

### `typed_list!` macro

`typed_list!` creates a `TypedListSpec` describing the structure expected in dots. Validation happens at R call time.

Syntax:

```
typed_list!(
    field_name => type_spec,
    optional_field? => type_spec,   // ? marks optional
)
```

Type specifiers:

| Syntax | Matches |
|--------|---------|
| `numeric()` | Real/double vector, any length |
| `numeric(4)` | Real/double vector, exactly 4 elements |
| `integer()` | Integer vector |
| `logical()` | Logical vector |
| `character()` | Character vector |
| `raw()` | Raw vector |
| `complex()` | Complex vector |
| `list()` | List (VECSXP or pairlist) |
| `"data.frame"` | Object with that class |
| `"my_class"` | Any class name as a string literal |

By default, extra fields in dots are allowed. Use `@exact;` at the start of the spec for strict mode:

```rust
typed_list!(@exact;
    x => numeric(),
    y => numeric()
)
```

### Attribute sugar

The most ergonomic pattern is `#[miniextendr(dots = typed_list!(...))]`. The macro injects validation at the top of the function body and binds the result to `dots_typed`:

```rust
#[miniextendr(dots = typed_list!(x => numeric(), y => numeric()))]
pub fn compute(...) -> f64 {
    let x: Vec<f64> = dots_typed.get("x").expect("x");
    let y: Vec<f64> = dots_typed.get("y").expect("y");
    x.iter().zip(y.iter()).map(|(a, b)| a + b).sum()
}
```

The macro expands this to:

```rust
let dots_typed = _dots.typed(typed_list!(x => numeric(), y => numeric()))
    .expect("dots validation failed");
```

### `TypedList` accessors

After validation, `dots_typed` (a `TypedList`) provides:

| Method | Returns |
|--------|---------|
| `get::<T>(name)` | `Result<T, TypedListError>` — required field |
| `get_opt::<T>(name)` | `Result<Option<T>, TypedListError>` — optional field |
| `get_raw(name)` | `Result<SEXP, TypedListError>` — raw SEXP |
| `as_list()` | `List` — underlying list |

`get::<T>()` / `get_opt::<T>()` accept **both** scalar and vector target types:
`T` may be a scalar (`f64`, `i32`, `String`, `bool`) **or** a vector/collection
(`Vec<f64>`, `Vec<i32>`, `Vec<String>`, …) — any `TryFromSexp` with a `Display`
error type. So `let x: Vec<f64> = dots_typed.get("x")?` (above) compiles for a
`numeric()` field, and `dots_typed.get::<Vec<i32>>("survive")` for an
`integer()` field.

### Error types

`TypedListError` variants emitted on validation failure:

| Variant | Cause |
|---------|-------|
| `NotList` | Input was not a list |
| `Missing { name }` | Required field absent |
| `WrongType { name, expected, actual }` | Field type mismatch |
| `WrongLen { name, expected, actual }` | Field length mismatch |
| `ExtraFields { names }` | Extra fields in strict (`@exact`) mode |
| `DuplicateNames { name }` | Duplicate field names |

## How it works

### R wrapper generation

When `#[miniextendr]` sees `...` in the Rust signature, the generated R wrapper function includes `...` in its formals. The `.Call` invocation collects dots into a named list and passes it as the dots argument to the C wrapper. `TryFromSexp` converts the incoming pairlist to the `Dots` type.

### Manual validation

Call `.typed()` directly in the function body instead of using the attribute sugar:

```rust
use miniextendr_api::typed_list;

#[miniextendr]
pub fn configure_model(...) -> String {
    let spec = typed_list!(
        learning_rate => numeric(),
        epochs => integer(),
        verbose? => logical()
    );
    let args = match _dots.typed(spec) {
        Ok(a) => a,
        Err(e) => panic!("{e}"),
    };
    let lr: f64 = args.get("learning_rate").expect("learning_rate");
    format!("lr={lr}")
}
```

## Decision trees

### Do I need positional dots or named/typed dots?

- I just want to forward dots to another R function or count how many arguments were passed:
  - Use `_dots.as_list()` (unchecked) or `_dots.try_list()` (validated).
- I know the exact structure: specific named fields each with known types:
  - Use `typed_list!` either as attribute sugar or manually.
- I want optional fields mixed with required ones:
  - Mark optional fields with `?`: `field? => type_spec`.
  - Use `get_opt` in the function body.

### Custom name or default `_dots`?

- Default `_dots` is fine for most functions. The underscore suppresses unused warnings.
- Use `name @ ...` when the name has semantic meaning in the function body (e.g., `options @ ...`) or you are forwarding to a helper that expects a specific variable name.

## Key files

- `docs/DOTS_TYPED_LIST.md` — full documentation with examples.
- `miniextendr-api/src/dots.rs` — `Dots` type, `TypedList`, `TypedListSpec`, `TypedListError`.
- `miniextendr-macros/src/typed_list.rs` — `typed_list!` macro implementation.

## Common pitfalls

- **`match.arg` returning the full vector**: `match.arg(arg = default_choices_vector, several.ok = TRUE)` returns the entire choices vector when called with a default multi-element argument. If you use `match.arg` in an R wrapper alongside dots, check that the defaults are not inadvertently the full choices list. This is a broader `match.arg` gotcha documented in `miniextendr-macros`.

- **Using `as_list()` when you need validation**: `as_list()` is unchecked. If R calls the function with the wrong types, you get a runtime error deep inside your Rust code instead of a clean validation error. Prefer `typed_list!` when the schema is known at compile time.

- **Strict mode rejecting intended extras**: `@exact` causes any unnamed or extra fields to be rejected. If you want to accept caller-defined extras and only validate required fields, omit `@exact`.

- **`get` vs `get_opt` mismatch**: calling `get` for a field marked `?` (optional) panics if the field is absent. Use `get_opt` for optional fields; it returns `None` when the field is missing rather than an error.

- **Duplicate field names in the R call**: `DuplicateNames` fires if the R caller passes the same name twice in `...`. This is usually an R-side mistake; the error message includes the duplicated name.

## Related skills

- `miniextendr-macros` — broader `#[miniextendr]` attribute parsing, codegen, `match.arg` integration.
- `miniextendr-conversions` — `TryFromSexp` and `IntoR` for field types used inside `TypedList::get`.
