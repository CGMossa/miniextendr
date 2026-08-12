# Expression Evaluation Helpers

Safe wrappers for building and evaluating R function calls from Rust.

## Types

| Type | Purpose |
|------|---------|
| `RSymbol` | Interned R symbol (SYMSXP) -- never GC'd |
| `RCall` | Builder for R function calls (LANGSXP) |
| `REnv` | Well-known R environments (Global, Base, Empty) |

Plus free functions: `r_eval_str` / `r_eval_str_global` (parse + evaluate a
string of R source) and `dollar_extract` (the R `$` operator).

## Quick Example

```rust
use miniextendr_api::expression::{RCall, REnv};
use miniextendr_api::sys::Rf_mkString;

unsafe {
    // Call paste0("hello", " world") in base
    let result = RCall::new("paste0")
        .arg(Rf_mkString(c"hello".as_ptr()))
        .arg(Rf_mkString(c" world".as_ptr()))
        .eval(REnv::base().as_sexp())?;
}
```

## RSymbol

Wraps R's `Rf_install()` for interned symbols. Symbols are never garbage collected, so `RSymbol` needs no GC protection.

```rust
use miniextendr_api::expression::RSymbol;

// From a Rust string (allocates a CString)
let sym = unsafe { RSymbol::new("my_var") };

// From a C string literal (zero allocation)
let sym = unsafe { RSymbol::from_cstr(c"my_var") };

// Use as SEXP
let sexp = sym.as_sexp();
```

## RCall

Builds R function calls with positional and named arguments.

```rust
use miniextendr_api::expression::RCall;

unsafe {
    // Positional arguments
    let result = RCall::new("sum")
        .arg(my_vector_sexp)
        .eval(env)?;

    // Named arguments
    let result = RCall::new("paste")
        .arg(x_sexp)
        .arg(y_sexp)
        .named_arg("sep", sep_sexp)
        .eval(env)?;
}
```

### Error Handling

`eval()` uses `R_tryEvalSilent` and returns `Result<SEXP, String>`. On failure, the error message comes from R's `geterrmessage()`.

```rust
match RCall::new("stop").arg(msg_sexp).eval(env) {
    Ok(result) => { /* success */ },
    Err(msg) => { /* msg contains R's error message */ },
}
```

### GC Protection

`RCall` roots the callable and every positional or named argument for the
builder's lifetime, including freshly allocated inline arguments. It also
protects the call object and intermediate pairlist during construction. The
**returned SEXP is unprotected** -- caller must protect it if it will survive
across R API calls.

## REnv

Provides handles to R's well-known environments:

```rust
use miniextendr_api::expression::REnv;

unsafe {
    let global = REnv::global();                     // R_GlobalEnv
    let base = REnv::base();                         // R_BaseEnv
    let empty = REnv::empty();                       // R_EmptyEnv
    let base_ns = REnv::base_namespace();            // R_BaseNamespace (for .Internal etc.)
    let methods = REnv::package_namespace("methods")?; // getNamespace("methods")
    let caller = REnv::caller();                     // calling environment (R_GetCurrentEnv)

    // Use as SEXP
    let sexp = base.as_sexp();
}
```

Prefer `package_namespace(pkg)` over chasing symbols through
`R_GlobalEnv`. The former mirrors `getNamespace(pkg)` and resolves
against the package's own namespace regardless of what the user has
attached on the search path. `eval_global()` has been removed; evaluate
in `base()`, `base_namespace()`, or the caller's env instead.

## r_eval_str

Parse a string of R source and evaluate it — the runtime workhorse behind the
`r_str!` / `r!` macros. Every top-level expression is evaluated in order
(so side effects take effect); the value of the **last** one is returned,
matching `eval(parse(text = ...))`. Empty / whitespace-only input yields
`R_NilValue`.

```rust
use miniextendr_api::expression::{r_eval_str, r_eval_str_global};

unsafe {
    // In a specific environment
    let three = r_eval_str("1L + 2L", env)?;

    // Convenience wrapper for R_GlobalEnv
    let six = r_eval_str_global("local({ x <- 2; x * 3 })")?;
}
```

Parse failures (syntax error, incomplete input) and R evaluation errors both
come back as `Err(String)` — evaluation uses `R_tryEvalSilent`, so R errors
never longjmp through Rust frames. The returned SEXP is unprotected.

## dollar_extract

Convenience wrapper for the R `$` extraction operator, replacing hand-rolled
`Rf_install("$")` + `Rf_lang3` + `R_tryEvalSilent` ladders:

```rust
use miniextendr_api::expression::dollar_extract;

unsafe {
    let value = dollar_extract(list_sexp, "field_name")?;
}
```

## Safety Requirements

All functions in this module require:

- Being called from the **R main thread** (they use R API calls)
- `unsafe` blocks (they call into C)

Standalone `#[miniextendr]` functions already run on the main thread (they are the default), so these calls are safe there; they are also safe within ALTREP callbacks, which run on the main thread too.

## Use Cases

- **S4 slot access**: The `s4_helpers` module uses `RCall` internally
- **Calling R functions from ALTREP callbacks**: When `elt()` needs to call R
- **Dynamic dispatch**: Building R function calls based on runtime data
- **Package interop**: Calling functions from other R packages

## See Also

- [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md#s4-helpers-module) -- S4 helpers built on RCall
- [THREADS.md](THREADS.md) -- Main thread requirements
- [GC_PROTECT.md](GC_PROTECT.md) -- Protecting returned SEXPs
