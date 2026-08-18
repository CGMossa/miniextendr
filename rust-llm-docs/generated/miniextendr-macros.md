# miniextendr_macros v0.1.0

## miniextendr-macros - Procedural macros for Rust <-> R interop

This crate provides the procedural macros that power miniextendr's code
generation. Most users should depend on `miniextendr-api` and use its
re-exports, but this crate can be used directly when you only need macros.

Primary macros and derives:
- `#[miniextendr]` on functions, impl blocks, trait defs, and trait impls.
- `#[r_ffi_checked]` for main-thread routing of C-ABI wrappers.
- Derives: `ExternalPtr`, `RNativeType`, ALTREP derives, `RFactor`.
- Helpers: `typed_list` for typed list builders.

R wrapper generation is driven by Rust doc comments (roxygen tags are
extracted). During package build, the wrapper-gen pass loads the installed
shared object into R and calls `miniextendr_write_wrappers`, which walks the
linkme `#[distributed_slice]` tables and writes `R/<pkg>-wrappers.R`.

### Quick start

```ignore
use miniextendr_api::miniextendr;

#[miniextendr]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Macro expansion pipeline

#### Overview

```text
┌──────────────────────────────────────────────────────────────────────────┐
│                         #[miniextendr] on fn                             │
│                                                                          │
│  1. Parse: syn::ItemFn → MiniextendrFunctionParsed                       │
│  2. Analyze return type (Result<T>, Option<T>, raw SEXP, etc.)           │
│  3. Generate:                                                            │
│     ├── C wrapper: extern "C-unwind" fn C_<crate>_<name>(...) → SEXP     │
│     ├── R wrapper: const R_WRAPPER_<NAME>: &str = "..."                  │
│     └── Registration: const call_method_def_<name>: R_CallMethodDef      │
│  4. Original function preserved (with added attributes)                  │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│                    #[miniextendr(env|r6|s3|s4|s7)] on impl               │
│                                                                          │
│  1. Parse: syn::ItemImpl → extract methods                               │
│  2. For each method:                                                     │
│     ├── Generate C wrapper (handles self parameter)                      │
│     ├── Generate R method wrapper string                                 │
│     └── Generate registration entry                                      │
│  3. Generate class definition code per class system:                     │
│     ├── env: new.env() + method assignment                               │
│     ├── r6: R6Class() definition                                         │
│     ├── s3: S3 generics + methods                                        │
│     ├── s4: setClass() + setMethod()                                     │
│     └── s7: new_class() definition                                       │
│  4. Emit const with combined R code                                      │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│                         #[miniextendr] on trait                          │
│                                                                          │
│  1. Parse: syn::ItemTrait → extract method signatures                    │
│  2. Generate:                                                            │
│     ├── Trait tag constant: const TAG_<TRAIT>: mx_tag = ...              │
│     ├── Vtable struct: struct __vtable_<Trait> { ... }                   │
│     └── CCalls table: static MX_CCALL_<TRAIT>: [...] = ...               │
│  3. Original trait preserved                                             │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│                    #[miniextendr] impl Trait for Type                    │
│                                                                          │
│  1. Parse: syn::ItemImpl (trait impl)                                    │
│  2. Generate:                                                            │
│     ├── Vtable instance: static __VTABLE_<CRATE>_<TRAIT>_FOR_<TYPE>     │
│     ├── Wrapper struct: struct __MxWrapper<Type> { erased, data }        │
│     ├── Query function: fn __mx_query_<type>(tag) → vtable ptr           │
│     └── Base vtable: static __MX_BASE_VTABLE_<TYPE>: ...                 │
│  3. Original impl preserved                                              │
└──────────────────────────────────────────────────────────────────────────┘

```

#### Key Modules

| Module | Purpose |
|--------|---------|
| `miniextendr_fn` | Function parsing and attribute handling |
| `c_wrapper_builder` | C wrapper generation (`extern "C-unwind"`) |
| `r_wrapper_builder` | R wrapper code generation |
| `rust_conversion_builder` | Rust→SEXP return value conversion |
| `miniextendr_impl` | `impl Type` block processing |
| `r_class_formatter` | Class system code generation (env/r6/s3/s4/s7) |
| `miniextendr_trait` | Trait ABI metadata generation |
| `miniextendr_impl_trait` | `impl Trait for Type` vtable generation |
| `altrep` / `altrep_derive` | ALTREP struct derivation |
| `externalptr_derive` | `#[derive(ExternalPtr)]` |
| `roxygen` | Roxygen doc comment handling |

#### Generated Symbol Naming

Every `#[no_mangle]` symbol is prefixed with the consuming crate's name
(from `CARGO_CRATE_NAME`) so two packages loaded into one webR session
can't collide on a C symbol (#1273; helpers in `naming.rs`). For a
function `my_func` in a crate `mypkg`:
- C wrapper: `C_mypkg_my_func`
- R wrapper const: `R_WRAPPER_MY_FUNC`
- Registration: `call_method_def_my_func`

For a type `MyType` with trait `Counter` in a crate `mypkg`:
- Vtable: `__VTABLE_MYPKG_COUNTER_FOR_MYTYPE`
- Wrapper: `__MxWrapperMyType`
- Query: `__mx_query_mytype`

### Return Type Handling

The `return_type_analysis` module determines how to convert Rust returns to SEXP:

| Rust Type | Strategy | R Result |
|-----------|----------|----------|
| `T: IntoR` | `.into_sexp()` | Converted value |
| `Result<T, E>` | Unwrap or R error | Value or error |
| `Option<T>` | `Some` → value, `None` → `NULL` | Value or NULL |
| `SEXP` | Pass through | Raw SEXP |
| `()` | Invisible NULL | `invisible(NULL)` |

Use `#[miniextendr(unwrap_in_r)]` to return `Result<T, E>` to R without unwrapping.

### Thread Strategy

By default, `#[miniextendr]` functions run on R's main thread. Opt into
worker-thread execution with `#[miniextendr(worker)]` (requires the
`worker-thread` feature on `miniextendr-api`). A worker opt-in is ignored
when the signature requires main-thread execution (returns/takes `SEXP`,
uses variadic dots, or sets `check_interrupt`).

**Note**: `ExternalPtr<T>` is `Send` — it can be returned from worker
thread functions. All R API operations on `ExternalPtr` are serialized
through `with_r_thread`.

### Class Systems

The `r_class_formatter` module generates R code for different class systems:

| System | Generated R Code | Self Parameter |
|--------|------------------|----------------|
| `env` | `new.env()` with methods | `self` environment |
| `r6` | `R6Class()` | `self` environment |
| `s3` | `structure()` + generics | First argument |
| `s4` | `setClass()` + `setMethod()` | First argument |
| `s7` | `new_class()` | `self` property |

---

## Structs

### `c_wrapper_builder::CWrapperContext`

```rust
pub struct CWrapperContext
```

All information needed to generate a C wrapper function for an R-exported Rust item.

This struct abstracts over the differences between standalone `#[miniextendr]` functions
and `impl` block methods (R6, S3, S4, S7, Env). It is constructed via
[`CWrapperContextBuilder`] and consumed by [`CWrapperContext::generate`], which emits
both the `extern "C-unwind"` wrapper and the corresponding `R_CallMethodDef` constant.

**Fields:**

- `fn_ident`: `syn::Ident`
  - Identifier of the original Rust function or method being wrapped.
- `c_ident`: `syn::Ident`
  - Identifier for the generated C wrapper (e.g., `C_<crate>_foo` or `C_<crate>_Type__method`).
- `r_wrapper_const`: `syn::Ident`
  - Identifier of the `R_WRAPPER_*` or `R_WRAPPERS_IMPL_*` const that holds the
- `inputs`: `syn::punctuated::Punctuated<syn::FnArg, $crate::token::Comma>`
  - Function parameters (excluding the `self` receiver for methods).
- `output`: `syn::ReturnType`
  - The original Rust return type. Used by strict-mode to inspect whether the inner
- `pre_call`: `Vec<proc_macro2::TokenStream>`
  - Statements emitted before the call expression. For instance methods, this
- `call_expr`: `proc_macro2::TokenStream`
  - The actual Rust call expression (e.g., `my_func(arg0, arg1)` or
- `thread_strategy`: `ThreadStrategy`
  - Whether to run on the main R thread or dispatch to the worker thread.
- `return_handling`: `ReturnHandling`
  - How to convert the Rust return value into a `SEXP` for R.
- `coerce_all`: `bool`
  - When `true`, all parameters use coercing conversion (`Rf_coerceVector`) instead
- `coerce_params`: `Vec<String>`
  - Names of individual parameters that use coercing conversion.
- `check_interrupt`: `bool`
  - When `true`, emits `R_CheckUserInterrupt()` before the call expression.
- `rng`: `bool`
  - When `true`, wraps the call in `GetRNGstate()`/`PutRNGstate()` for R's
- `cfg_attrs`: `Vec<syn::Attribute>`
  - `#[cfg(...)]` attributes from the original item, propagated to the C wrapper
- `type_context`: `Option<syn::Ident>`
  - For methods: the type identifier (e.g., `MyStruct`). Used in doc comments
- `has_self`: `bool`
  - Whether the original method has a `self` receiver. When `true`, the C wrapper
- `call_method_def_ident`: `Option<syn::Ident>`
  - Override for the `call_method_def` constant name. If `None`, defaults to
- `strict`: `bool`
  - When `true`, uses `checked_into_sexp_*` for lossy return types (`i64`, `u64`,
- `match_arg_several_ok_params`: `Vec<String>`
  - Parameter names with `#[miniextendr(match_arg, several_ok)]` — use
- `preserve_param_names`: `bool`
  - When `true`, preserve original parameter names from `inputs` in the C wrapper
- `vis`: `syn::Visibility`
  - Visibility of the generated `extern "C-unwind"` wrapper function.
- `generics`: `syn::Generics`
  - Generic parameters of the wrapped function, emitted on the C wrapper signature
- `skip_wrapper`: `bool`
  - When `true`, the original Rust fn is already an `extern "C-unwind"` symbol (user-written).

**Inherent associated items:**

#### `builder`

```rust
fn builder(fn_ident: syn::Ident, c_ident: syn::Ident) -> CWrapperContextBuilder
```

Creates a new [`CWrapperContextBuilder`] with the given function and C wrapper identifiers.

All other fields start at their defaults (empty/false/None). Use the builder methods
to configure the context, then call [`CWrapperContextBuilder::build`] to finalize.

#### `generate`

```rust
fn generate(self: &Self) -> TokenStream
```

Generates the complete output for this wrapper: an `extern "C-unwind"` function
and an `R_CallMethodDef` constant, both decorated with `#[cfg(...)]` attributes
if present.

When `skip_wrapper` is set (for user-written `extern "C-unwind"` fns), only the
`R_CallMethodDef` is emitted — the fn body itself is already the C symbol.

Dispatches to [`generate_main_thread_wrapper`](Self::generate_main_thread_wrapper) or
[`generate_worker_thread_wrapper`](Self::generate_worker_thread_wrapper) based on
[`thread_strategy`](Self::thread_strategy).

### `c_wrapper_builder::CWrapperContextBuilder`

```rust
pub struct CWrapperContextBuilder
```

Builder for [`CWrapperContext`].

Created via [`CWrapperContext::builder`]. All fields except `fn_ident` and `c_ident`
(provided at construction) default to empty/false/None. Required fields (`call_expr`,
`r_wrapper_const`) must be set before calling [`build`](Self::build) or it will panic.

Optional fields like `thread_strategy` and `return_handling` are auto-detected from
the function signature if not explicitly set.

**Inherent associated items:**

#### `build`

```rust
fn build(self: Self) -> CWrapperContext
```

Consumes the builder and returns a fully configured [`CWrapperContext`].

If `thread_strategy` was not set, defaults to [`ThreadStrategy::MainThread`].
If `return_handling` was not set, auto-detects from the `output` type via
[`detect_return_handling`].

##### Panics

Panics if `call_expr` or `r_wrapper_const` was not set.

#### `call_expr`

```rust
fn call_expr(self: Self, expr: TokenStream) -> Self
```

Sets the Rust call expression (e.g., `my_func(arg0)` or `self_ref.method(arg0)`).
**Required** -- [`build`](Self::build) panics if not set.

#### `call_method_def_ident`

```rust
fn call_method_def_ident(self: Self, ident: syn::Ident) -> Self
```

Set a custom call_method_def identifier.

If not set, the default naming is used:
- With type_context: `call_method_def_{type}_{method}`
- Without: `call_method_def_{method}`

#### `cfg_attrs`

```rust
fn cfg_attrs(self: Self, attrs: Vec<syn::Attribute>) -> Self
```

Sets `#[cfg(...)]` attributes to propagate to the C wrapper and `call_method_def`.

#### `check_interrupt`

```rust
fn check_interrupt(self: Self) -> Self
```

Enables `R_CheckUserInterrupt()` before the call expression.

#### `coerce_all`

```rust
fn coerce_all(self: Self) -> Self
```

Enables coercing conversion for all parameters via `Rf_coerceVector`.

#### `generics`

```rust
fn generics(self: Self, generics: syn::Generics) -> Self
```

Set the generic parameters for the C wrapper function signature.

Defaults to empty generics. Standalone fns with generic parameters
must forward them so the generated wrapper is also generic.

#### `has_self`

```rust
fn has_self(self: Self) -> Self
```

Marks this as an instance method with a `self` receiver.
Causes the C wrapper to include a `self_sexp` parameter.

#### `inputs`

```rust
fn inputs(self: Self, inputs: syn::punctuated::Punctuated<syn::FnArg, $crate::token::Comma>) -> Self
```

Sets the function parameters (excluding `self` receiver).
Each input becomes a `SEXP` argument in the C wrapper.

#### `match_arg_several_ok`

```rust
fn match_arg_several_ok(self: Self, param_name: String) -> Self
```

Record a parameter as `match_arg + several_ok`.

Passed through to `RustConversionBuilder::with_match_arg_several_ok`, which
switches that parameter's conversion from `TryFromSexp` to
`match_arg_vec_from_sexp::<Inner>` so each STRSXP element is validated against
the enum's `MatchArg::CHOICES`.

#### `output`

```rust
fn output(self: Self, output: syn::ReturnType) -> Self
```

Sets the Rust return type. Used for auto-detecting [`ReturnHandling`]
and for strict-mode type inspection.

#### `pre_call`

```rust
fn pre_call(self: Self, stmts: Vec<TokenStream>) -> Self
```

Sets pre-call statements emitted before the call expression.
Typically used for self-extraction in instance methods.

#### `preserve_param_names`

```rust
fn preserve_param_names(self: Self) -> Self
```

Preserve original parameter names in the C wrapper signature.

When `true`, `build_c_params` uses the original identifier from `inputs` instead
of renaming to `arg_N`. Enables rustdoc to show descriptive parameter names.
Used by the standalone-fn path; impl methods use the default `arg_N` form.

#### `r_wrapper_const`

```rust
fn r_wrapper_const(self: Self, ident: syn::Ident) -> Self
```

Sets the R wrapper constant identifier (e.g., `R_WRAPPER_my_func`).
**Required** -- [`build`](Self::build) panics if not set.

#### `return_handling`

```rust
fn return_handling(self: Self, handling: ReturnHandling) -> Self
```

Overrides the return handling strategy. If not called, auto-detected from `output`
via [`detect_return_handling`].

#### `rng`

```rust
fn rng(self: Self) -> Self
```

Enable RNG state management (GetRNGstate/PutRNGstate).

#### `skip_wrapper`

```rust
fn skip_wrapper(self: Self) -> Self
```

Skip generating the wrapper body and only emit the `R_CallMethodDef`.

Use this when the Rust fn is already `extern "C-unwind"` with `#[no_mangle]` or
`#[unsafe(no_mangle)]` (the user wrote the C symbol directly). The function still
needs to be registered with R via `R_CallMethodDef`.

When set, `numArgs` is computed from `inputs` directly (no synthetic
`__miniextendr_call` param).

#### `strict`

```rust
fn strict(self: Self) -> Self
```

Enables strict checked conversions for lossy return types (`i64`, `u64`, `isize`,
`usize` and their `Vec` variants).

#### `thread_strategy`

```rust
fn thread_strategy(self: Self, strategy: ThreadStrategy) -> Self
```

Overrides the thread strategy. If not called, defaults to [`ThreadStrategy::MainThread`].

#### `type_context`

```rust
fn type_context(self: Self, type_ident: syn::Ident) -> Self
```

Sets the type context for methods (e.g., `MyStruct`). Used in doc comments
and default `call_method_def` naming.

#### `vis`

```rust
fn vis(self: Self, vis: syn::Visibility) -> Self
```

Set the visibility of the generated `extern "C-unwind"` wrapper.

Defaults to [`syn::Visibility::Inherited`]. Standalone fns forward the user's
declared visibility (`pub`, `pub(crate)`, etc.).

#### `with_coerce_param`

```rust
fn with_coerce_param(self: Self, param_name: String) -> Self
```

Enables coercing conversion for a specific named parameter.

### `lifecycle::LifecycleSpec`

```rust
pub struct LifecycleSpec
```

Full lifecycle specification for a function or method.

**Fields:**

- `stage`: `LifecycleStage`
  - The lifecycle stage.
- `when`: `Option<String>`
  - Version when the lifecycle change occurred (e.g., "0.4.0").
- `what`: `Option<String>`
  - What is being deprecated (e.g., "old_fn()" or "old_fn(arg)").
- `with`: `Option<String>`
  - Replacement to suggest (e.g., "new_fn()").
- `details`: `Option<String>`
  - Additional details message.
- `id`: `Option<String>`
  - Unique ID for lifecycle deprecation tracking.

**Inherent associated items:**

#### `from_deprecated`

```rust
fn from_deprecated(since: Option<&str>, note: Option<&str>) -> Self
```

Create a lifecycle spec from a Rust `#[deprecated]` attribute.

Maps the `since` field to `when` and attempts to parse the `note` field
for a "use X instead" pattern to populate `with`. The full note is also
stored in `details`.

#### `new`

```rust
fn new(stage: LifecycleStage) -> Self
```

Create a new lifecycle spec with the given stage and no additional metadata.

#### `r_prelude`

```rust
fn r_prelude(self: &Self, fn_name: &str) -> Option<String>
```

Generate the R prelude code for lifecycle signaling.

Returns a single line of R code to insert at the start of the function body,
or `None` for `Stable` stage. The `fn_name` is used as the `what` argument
if no explicit `what` was provided.

For experimental/superseded: `lifecycle::signal_stage("stage", "fn_name()")`.
For deprecated variants: `lifecycle::deprecate_*(when, what, with, details, id)`.

### `list_macro::ListEntry`

```rust
pub struct ListEntry
```

A single entry in the list.

**Fields:**

- `name`: `Option<ListName>`
  - The name (None for unnamed entries).
- `value`: `syn::Expr`
  - The value expression.

### `list_macro::ListInput`

```rust
pub struct ListInput
```

Parsed `list!` macro input containing zero or more entries.

**Fields:**

- `entries`: `Vec<ListEntry>`
  - The list entries, which may be named, unnamed, or a mix.

### `method_return_builder::MethodReturnBuilder`

```rust
pub struct MethodReturnBuilder
```

Builder for generating R method body lines with appropriate return handling.

Produces lines of R code for a method body, combining the `.Call()` expression
with the return strategy and the tagged-condition error guard. Each class
system has specialized builder methods (`build_r6_body`, `build_s3_body`,
etc.) that produce idiomatic R code for that system.

**Inherent associated items:**

#### `build`

```rust
fn build(self: &Self) -> Vec<String>
```

Build R code lines for the method body.

Returns a vector of strings, one per line (without trailing newlines).

#### `build_r6_body`

```rust
fn build_r6_body(self: &Self) -> Vec<String>
```

Build R6-style return (uses invisible(self) for chaining).

#### `build_s3_body`

```rust
fn build_s3_body(self: &Self) -> Vec<String>
```

Build S3-style return (uses structure() for Self returns).

#### `build_s4_body`

```rust
fn build_s4_body(self: &Self) -> Vec<String>
```

Build S4-style method body lines (uses methods::new() to wrap Self returns).

Returns lines suitable for embedding inside an outer
`function(...) { ... }` block, mirroring [`build_s7_body`](Self::build_s7_body).

#### `build_s4_inline`

```rust
fn build_s4_inline(self: &Self) -> String
```

Build S4-style return (uses methods::new()).

Returns a multi-line block expression that performs the condition check
inline.

#### `build_s7_body`

```rust
fn build_s7_body(self: &Self) -> Vec<String>
```

Build S7-style method body lines (creates new S7 object with .ptr).

Returns lines suitable for embedding inside an outer `function(...) { ... }`
block — unlike [`build_s7_inline`](Self::build_s7_inline) which wraps the
body in its own `{ }` and is intended for callers that emit
`function(...) <expr>` directly (e.g., S7 `convert` definitions).

#### `build_s7_inline`

```rust
fn build_s7_inline(self: &Self) -> String
```

Build S7-style return (creates new S7 object with .ptr).

Returns a multi-line block expression that performs the condition check
inline (suitable for S7 property definitions / convert methods that
require a single expression).

#### `new`

```rust
fn new(call_expr: String) -> Self
```

Create a new builder with the given .Call expression.

#### `with_chain_var`

```rust
fn with_chain_var(self: Self, var: String) -> Self
```

Set the variable name to return for chaining (default: "self").

#### `with_class_name`

```rust
fn with_class_name(self: Self, class_name: String) -> Self
```

Set the class name (for Self returns).

#### `with_indent`

```rust
fn with_indent(self: Self, indent: usize) -> Self
```

Set indentation level (number of spaces).

#### `with_return_class`

```rust
fn with_return_class(self: Self, return_class: String) -> Self
```

Set the return class name (for cross-class ExternalPtr returns).

#### `with_return_class_from_method`

```rust
fn with_return_class_from_method(self: Self, method: &ParsedMethod) -> Self
```

Attach the method's cross-class return name when this builder uses
[`ReturnStrategy::ReturnOtherClass`] or
[`ReturnStrategy::ReturnOtherClassList`].

#### `with_strategy`

```rust
fn with_strategy(self: Self, strategy: ReturnStrategy) -> Self
```

Set the return strategy.

### `miniextendr_impl::ImplAttrs`

```rust
pub struct ImplAttrs
```

Attributes parsed from `#[miniextendr(...)]` on an impl block.

These control which R class system to use, class naming, multi-impl labeling,
and class-system-specific options (R6 inheritance, S7 parent, vctrs kind, etc.).

Parsed by the [`syn::parse::Parse`] implementation which handles all supported
attribute formats like `#[miniextendr(r6, class = "Custom", label = "ops")]`.

**Fields:**

- `class_system`: `ClassSystem`
  - Which R class system to generate wrappers for.
- `class_name`: `Option<String>`
  - Optional override for the R class name. When `None`, the Rust type name is used.
- `label`: `Option<String>`
  - Optional label for distinguishing multiple impl blocks of the same type.
- `vctrs_attrs`: `VctrsAttrs`
  - vctrs-specific attributes (only used when class_system is Vctrs)
- `r6_inherit`: `Option<String>`
  - R6 parent class for inheritance.
- `r6_portable`: `Option<bool>`
  - R6 portable flag. Default TRUE. Set to false for non-portable R6 classes.
- `r6_cloneable`: `Option<bool>`
  - R6 cloneable flag. Controls whether `$clone()` is available.
- `r6_lock_objects`: `Option<bool>`
  - R6 lock_objects flag. Controls whether fields can be added after creation.
- `r6_lock_class`: `Option<bool>`
  - R6 lock_class flag. Controls whether the class definition can be modified.
- `s7_parent`: `Option<String>`
  - S7 parent class for inheritance.
- `s7_abstract`: `bool`
  - S7 abstract class flag. Abstract classes cannot be instantiated.
- `r_data_accessors`: `bool`
  - When true, auto-include `#[r_data]` field accessors in the class definition.
- `strict`: `bool`
  - When true, methods returning lossy types (i64/u64/isize/usize + Vec variants)
- `no_preconditions`: `bool`
  - When true, drop the R-side `stopifnot(...)` precondition block from all
- `no_call_attribution`: `bool`
  - When true, emit `.call = NULL` instead of `.call = match.call()` in all
- `internal`: `bool`
  - Mark class as internal: adds `@keywords internal`, suppresses `@export`.
- `noexport`: `bool`
  - Suppress `@export` without adding `@keywords internal`.
- `blanket`: `bool`
  - When true on a trait impl (`impl Trait for Type`), the impl block is NOT

### `miniextendr_impl::MethodAttrs`

```rust
pub struct MethodAttrs
```

Per-method attributes for class system customization.

**Fields:**

- `ignore`: `bool`
  - Skip this method
- `constructor`: `bool`
  - Mark as constructor
- `r6`: `R6MethodAttrs`
  - R6-specific method markers. All R6 boolean flags live here.
- `as_coercion`: `Option<String>`
  - Generate as `as.<class>()` S3 method (e.g., "data.frame", "list", "character").
- `as_coercion_span`: `Option<proc_macro2::Span>`
  - Span of `as = "..."` for error reporting.
- `generic`: `Option<String>`
  - Override generic name for S3/S4/S7 methods.
- `class`: `Option<String>`
  - Override class suffix for S3 methods.
- `worker`: `bool`
  - Worker thread execution (default: auto-detect based on types)
- `unsafe_main_thread`: `bool`
  - Force main thread execution (unsafe)
- `check_interrupt`: `bool`
  - Enable R interrupt checking
- `coerce`: `bool`
  - Enable coercion for this method's parameters
- `rng`: `bool`
  - Enable RNG state management (GetRNGstate/PutRNGstate)
- `has_dots`: `bool`
  - Whether this method accepts dots, either from raw `...` rewritten to
- `named_dots`: `Option<syn::Ident>`
  - User-provided dots binding, if one exists.
- `dots_spec`: `Option<proc_macro2::TokenStream>`
  - `typed_list!(...)` spec from `#[miniextendr(dots = typed_list!(...))]` on
- `unwrap_in_r`: `bool`
  - Return `Result<T, E>` to R without unwrapping.
- `defaults`: `std::collections::HashMap<String, String>`
  - Parameter defaults from `#[miniextendr(defaults(param = "value", ...))]`
- `defaults_span`: `Option<proc_macro2::Span>`
  - Span of `defaults(...)` for error reporting.
- `per_param`: `std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>`
  - Per-parameter `match_arg` / `several_ok` / `choices` state for this
- `match_arg_span`: `Option<proc_macro2::Span>`
  - Span of `match_arg(...)` / `choices(...)` for error reporting.
- `s7`: `S7MethodAttrs`
  - S7-specific method markers. Only consumed by the S7 class generator;
- `lifecycle`: `Option<crate::lifecycle::LifecycleSpec>`
  - Lifecycle specification for deprecation/experimental status on methods.
- `vctrs_protocol`: `Option<String>`
  - vctrs protocol method override.
- `r_name`: `Option<String>`
  - Override R method name.
- `r_entry`: `Option<String>`
  - R code to inject at the very top of the method body (before all built-in checks).
- `r_post_checks`: `Option<String>`
  - R code to inject after all built-in checks, immediately before `.Call()`.
- `r_on_exit`: `Option<crate::miniextendr_fn::ROnExit>`
  - Register `on.exit()` cleanup code in the R method wrapper.
- `internal`: `bool`
  - Mark this method as internal: adds `@keywords internal`, suppresses export.
- `noexport`: `bool`
  - Suppress export for this method without adding `@keywords internal`.

### `miniextendr_impl::ParsedImpl`

```rust
pub struct ParsedImpl
```

Fully parsed `#[miniextendr]` impl block, ready for code generation.

Contains the type identity, chosen class system, all parsed methods, the original
impl block (with miniextendr attrs stripped for re-emission), and all class-system-specific
configuration options. Created by [`ParsedImpl::parse`] and consumed by the per-class-system
R wrapper generators and [`generate_method_c_wrapper`].

**Fields:**

- `type_ident`: `syn::Ident`
  - The Rust type name being implemented (e.g., `Counter`).
- `class_system`: `ClassSystem`
  - Which R class system to generate wrappers for.
- `class_name`: `Option<String>`
  - Optional override for the R class name. When `None`, uses `type_ident` as the class name.
- `label`: `Option<String>`
  - Optional label for distinguishing multiple impl blocks of the same type.
- `doc_tags`: `Vec<String>`
  - Roxygen tag lines extracted from `///` doc comments on the impl block.
- `methods`: `Vec<ParsedMethod>`
  - All parsed methods in this impl block, in source order.
- `original_impl`: `syn::ItemImpl`
  - The original impl block with `#[miniextendr]` and roxygen attrs stripped.
- `cfg_attrs`: `Vec<syn::Attribute>`
  - `#[cfg(...)]` attributes from the impl block, propagated to all generated items
- `vctrs_attrs`: `VctrsAttrs`
  - vctrs-specific attributes (only used when class_system is Vctrs)
- `r6_inherit`: `Option<String>`
  - R6 parent class name for inheritance (e.g., `"ParentClass"`).
- `r6_portable`: `Option<bool>`
  - R6 portable flag. When `Some(true)`, generates a portable R6 class.
- `r6_cloneable`: `Option<bool>`
  - R6 cloneable flag. Controls whether `$clone()` is available on instances.
- `r6_lock_objects`: `Option<bool>`
  - R6 lock_objects flag. When `Some(true)`, prevents adding new fields after creation.
- `r6_lock_class`: `Option<bool>`
  - R6 lock_class flag. When `Some(true)`, prevents modifying the class definition.
- `s7_parent`: `Option<String>`
  - S7 parent class name for inheritance (e.g., `"ParentClass"`).
- `s7_abstract`: `bool`
  - When true, marks this as an abstract S7 class that cannot be instantiated.
- `r_data_accessors`: `bool`
  - When true, auto-include sidecar `#[r_data]` field accessors in the class definition.
- `strict`: `bool`
  - Strict conversion mode: methods returning lossy types use checked conversions.
- `no_preconditions`: `bool`
  - Drop the R-side `stopifnot(...)` precondition block from method wrappers.
- `no_call_attribution`: `bool`
  - Emit `.call = NULL` instead of `.call = match.call()` in method wrappers.
- `internal`: `bool`
  - Mark class as internal: adds `@keywords internal`, suppresses `@export`.
- `noexport`: `bool`
  - Suppress `@export` without adding `@keywords internal`.
- `param_warnings`: `proc_macro2::TokenStream`
  - Deprecation warnings for `@param` tags found on the impl block.
- `class_param_names`: `std::collections::HashSet<String>`
  - Parameter names declared via `@param` in the impl-block doc comments.

**Inherent associated items:**

#### `active_instance_methods`

```rust
fn active_instance_methods(self: &Self) -> impl Iterator<Item = &ParsedMethod>
```

Get active binding getter methods for R6 (have env, marked active, not setter).
Active bindings provide property-like access (obj$name instead of obj$name()).

#### `active_setter_methods`

```rust
fn active_setter_methods(self: &Self) -> impl Iterator<Item = &ParsedMethod>
```

Get active binding setter methods for R6 (have env, marked as r6_setter).

#### `as_coercion_methods`

```rust
fn as_coercion_methods(self: &Self) -> impl Iterator<Item = &ParsedMethod>
```

Get methods with `#[miniextendr(as = "...")]` attribute.

These generate S3 methods for R's `as.<class>()` generics like
`as.data.frame.MyType`, `as.list.MyType`, etc.

#### `class_name`

```rust
fn class_name(self: &Self) -> String
```

Get the class name (override or type name).

#### `constructor`

```rust
fn constructor(self: &Self) -> Option<&ParsedMethod>
```

Get the constructor method (fn new() -> Self), if included.
Respects `#[...(ignore)]` and visibility filters.

#### `finalizer`

```rust
fn finalizer(self: &Self) -> Option<&ParsedMethod>
```

Get the finalizer method, if any.

#### `find_setter_for_prop`

```rust
fn find_setter_for_prop(self: &Self, prop_name: &str) -> Option<&ParsedMethod>
```

Find the setter method for a given property name.

#### `included_methods`

```rust
fn included_methods(self: &Self) -> impl Iterator<Item = &ParsedMethod>
```

Get methods that should be included.

#### `instance_methods`

```rust
fn instance_methods(self: &Self) -> impl Iterator<Item = &ParsedMethod>
```

Get instance methods (have env) - includes both public and private.

#### `label`

```rust
fn label(self: &Self) -> Option<&str>
```

Returns the label if present.

#### `parse`

```rust
fn parse(attrs: ImplAttrs, item_impl: syn::ItemImpl) -> syn::Result<Self>
```

Parse an impl block with class system attribute.

Note: Trait impls (`impl Trait for Type`) are handled by `expand_impl`
before this function is called, so we only handle inherent impls here.

#### `private_instance_methods`

```rust
fn private_instance_methods(self: &Self) -> impl Iterator<Item = &ParsedMethod>
```

Get private instance methods (have env, private visibility, not active).

#### `public_instance_methods`

```rust
fn public_instance_methods(self: &Self) -> impl Iterator<Item = &ParsedMethod>
```

Get public instance methods (have env, not private, not active).

#### `r_wrappers_const_ident`

```rust
fn r_wrappers_const_ident(self: &Self) -> syn::Ident
```

Module constant identifier for R wrapper parts.

Format: `R_WRAPPERS_IMPL_{TYPE}` or `R_WRAPPERS_IMPL_{TYPE}_{LABEL}` if labeled.

#### `static_methods`

```rust
fn static_methods(self: &Self) -> impl Iterator<Item = &ParsedMethod>
```

Get static methods (no env, not constructor, not finalizer).

### `miniextendr_impl::ParsedMethod`

```rust
pub struct ParsedMethod
```

Parsed method from an impl block.

#### Default Parameters

Default parameters are specified using method-level syntax:
- `#[miniextendr(defaults(param = "value", ...))]` on the method

Note: Parameter-level `#[miniextendr(default = "...")]` syntax is only supported
for standalone functions, not impl methods (Rust language limitation).

Defaults cannot be specified for `self` parameters (compile error).

**Fields:**

- `ident`: `syn::Ident`
  - The method's name (e.g., `new`, `get`, `set_value`).
- `env`: `ReceiverKind`
  - How this method receives `self`: `&self`, `&mut self`, by value, or not at all (static).
- `sig`: `syn::Signature`
  - Method signature with the `self` receiver stripped. Used for C wrapper generation
- `vis`: `syn::Visibility`
  - Rust visibility of the method. Non-`pub` methods become private in R6;
- `doc_tags`: `Vec<String>`
  - Roxygen tag lines extracted from Rust doc comments
- `method_attrs`: `MethodAttrs`
  - Per-method attributes for class system overrides. Also carries the
- `param_defaults`: `std::collections::HashMap<String, String>`
  - Parameter default values from `#[miniextendr(default = "...")]`
- `has_dots`: `bool`
  - Whether this method accepts dots, either from raw `...` rewritten to

**Inherent associated items:**

#### `c_wrapper_ident`

```rust
fn c_wrapper_ident(self: &Self, type_ident: &syn::Ident, label: Option<&str>) -> syn::Ident
```

C wrapper identifier for this method.

Format: `C_{crate}_{Type}__{method}` or `C_{crate}_{Type}_{label}_{method}`
if labeled — crate-prefixed for webR cross-package symbol uniqueness (#1273).

#### `from_impl_item`

```rust
fn from_impl_item(item: &mut syn::ImplItemFn, _class_system: ClassSystem) -> syn::Result<Self>
```

Parse a method from an impl item.

Regular doc comments are auto-converted to `@description` for all class systems.

#### `is_active`

```rust
fn is_active(self: &Self) -> bool
```

Returns true if this method should be an R6 active binding.
Active bindings provide property-like access (obj$name instead of obj$name()).

#### `is_constructor`

```rust
fn is_constructor(self: &Self) -> bool
```

Returns true if this is likely a constructor.
Inferred from: no env + named "new" + returns Self.

#### `is_finalizer`

```rust
fn is_finalizer(self: &Self) -> bool
```

Returns true if this is likely a finalizer.
Inferred from: consumes self (by value) + doesn't return Self.

#### `is_private`

```rust
fn is_private(self: &Self) -> bool
```

Returns true if this method should be private in R6.
Inferred from Rust visibility: anything not `pub` is private.

#### `lifecycle_prelude`

```rust
fn lifecycle_prelude(self: &Self, what: &str) -> Option<String>
```

Generate lifecycle prelude R code for this method, if lifecycle is specified.

The `what` parameter describes the method in the format appropriate for the class system:
- Env/R6: `"Type$method()"`
- S3: `"method.Type()"`
- S7: `"method()`" (dispatched generics)

#### `r_method_name`

```rust
fn r_method_name(self: &Self) -> String
```

R-facing method name.

Returns `r_name` if set, otherwise the Rust ident as a string.

#### `returns_option_self`

```rust
fn returns_option_self(self: &Self) -> bool
```

Returns true if this method returns `Option<Self>` — a lookup-shaped
fallible constructor (e.g. `try_find`). On the R side this is treated
exactly like a bare `Self` return (wrapped class object via
[`crate::ReturnStrategy::for_method`]); the C wrapper still raises on
`None` via the normal `Option` error path (see
[`crate::c_wrapper_builder::ReturnHandling::OptionExternalPtr`]).
Symmetric with [`Self::returns_result_self`].

#### `returns_other_class`

```rust
fn returns_other_class(self: &Self) -> Option<syn::Ident>
```

Returns the bare type name when this method returns a type that may be a
different registered ExternalPtr-backed class.

This is deliberately syntactic and conservative about primitives and
common containers. The write-time wrapper resolver checks the complete
class registry, so an unregistered capitalized type falls back to the
direct `.val` return.

Three shapes are recognized:
- A bare capitalized path, e.g. `-> Board`.
- `Option<T>` where `T` is a bare capitalized path. The C wrapper
  already unwraps this and raises on `None`
  (`ReturnHandling::OptionIntoRUnwrap`), so the successful `.val` is a
  bare pointer — identical to the bare-class case on the R side.
- `Result<T, E>` where `T` is a bare capitalized path and `E` is not
  `()`. `Result<T, ()>` is excluded: the unit-error sentinel maps to
  `ReturnHandling::ResultNullOnErr`, where `.val` can be `NULL` on
  `Ok`, and wrapping `NULL` in a class constructor would break.

`Option<Self>` / `Result<Self, _>` are excluded here too (the inner
ident is `Self`) — `ReturnStrategy::for_method` checks
`returns_result_self()` / `returns_option_self()` before this method,
so those already take the `ReturnSelf` path regardless.
List-shaped returns (`Vec<Class>` and its `Option`/`Result` wrappers)
are handled separately by
[`returns_other_class_list`](Self::returns_other_class_list); any other
nested container is not recognized: the inner type must be a bare path
with no path arguments of its own.

#### `returns_other_class_list`

```rust
fn returns_other_class_list(self: &Self) -> Option<syn::Ident>
```

Returns the bare element type name when this method returns a *list* of
values that may be a different registered ExternalPtr-backed class.

The list-shaped sibling of
[`returns_other_class`](Self::returns_other_class) (#1284). Like the
scalar detector it is deliberately syntactic: the write-time resolver
checks the complete class registry, and an unregistered element type
falls back to the bare list of external pointers.

Three shapes are recognized (the element type must pass the same bare
capitalized-path filter as the scalar case):
- `Vec<Class>` — the C wrapper converts via `IntoR` into a `VECSXP` of
  external pointers (the `IntoRVecElement` impl emitted by
  `#[derive(ExternalPtr)]`), so `.val` is a bare list.
- `Option<Vec<Class>>` — the C wrapper already unwraps and raises on
  `None` (`ReturnHandling::OptionIntoRUnwrap`), so the successful
  `.val` is the same bare list.
- `Result<Vec<Class>, E>` where `E` is not `()` — the C wrapper raises
  on `Err` (`ReturnHandling::ResultIntoR`); same bare list on `Ok`.
  `Result<Vec<Class>, ()>` is excluded: the unit-error sentinel maps to
  `ReturnHandling::ResultNullOnErr`, where `.val` can be `NULL`, and
  `lapply(NULL, ...)` would silently turn that `NULL` into `list()`.

Not recognized (deliberately):
- `Vec<Self>` — the receiver type ident is not visible here; spell the
  concrete type name (`-> Vec<Board>` inside `impl Board`) to get the
  wrapped list.
- `Vec<Option<Class>>` — has no `IntoR` impl (the `Vec<Option<T>>`
  blanket slot is occupied by the newtype funnel) and would need a
  NULL-tolerant per-element wrap; tracked as a follow-up.
- `Vec<ExternalPtr<Class>>` — kept unwrapped for symmetry with the
  scalar `ExternalPtr<Class>` return, which `returns_other_class` also
  leaves unwrapped.

#### `returns_result_self`

```rust
fn returns_result_self(self: &Self) -> bool
```

Returns true if this method returns `Result<Self, E>` — a fallible
constructor-shaped method (e.g. `from_r`, `try_new`). On the R side this
is treated exactly like a bare `Self` return (wrapped class object via
[`crate::ReturnStrategy::for_method`]); the C wrapper still raises on
`Err` via the normal `Result` error path (see
[`crate::c_wrapper_builder::ReturnHandling::ResultExternalPtr`]).

#### `returns_self`

```rust
fn returns_self(self: &Self) -> bool
```

Returns true if this method returns Self.

#### `returns_self_ref`

```rust
fn returns_self_ref(self: &Self) -> bool
```

Returns true if this method returns a reference to `Self` (`&Self` or
`&mut Self`) — the idiomatic Rust in-place builder signature.

These methods mutate (`&mut self`) or read (`&self`) the receiver and
return a borrow of the same value so calls can be chained in Rust
(`b.set_a(1).set_b(2)`). On the R side this maps to a pipe-friendly free
function (S3) / chainable instance method that returns the *same*
ExternalPtr handle, so the R idiom `obj |> set_a(1) |> set_b(2)` works
with in-place value semantics (no clone). See
[`crate::c_wrapper_builder::ReturnHandling::SelfHandle`].

#### `returns_unit`

```rust
fn returns_unit(self: &Self) -> bool
```

Returns true if this method has no return type (returns unit `()`).

#### `should_include`

```rust
fn should_include(self: &Self) -> bool
```

Returns true if this method should be included in the class.

### `miniextendr_impl::R6MethodAttrs`

```rust
pub struct R6MethodAttrs
```

R6-specific per-method markers, separated from [`MethodAttrs`] so the
`r6` parser branch and R6 class generator own a self-contained bag.

All R6 boolean flags live here.  Using any of these markers under a
non-R6 class system (`#[miniextendr(s3)]`, `s4`, `s7`, `env`) is a
compile-time error caught by [`ParsedMethod::validate_method_attrs`].

**Fields:**

- `active`: `bool`
  - Mark as active binding getter (`#[miniextendr(r6(active))]`).
- `active_span`: `Option<proc_macro2::Span>`
  - Span of the `r6(active)` marker — used for error reporting when the
- `setter`: `bool`
  - R6 active-binding *setter* (paired with an `active` getter by `prop`).
- `prop`: `Option<String>`
  - R6 active-binding property name (defaults to the method name).
- `private`: `bool`
  - Mark as private method (`#[miniextendr(r6(private))]`).
- `private_span`: `Option<proc_macro2::Span>`
  - Span of the `r6(private)` marker — points the validator's diagnostic
- `finalize`: `bool`
  - Mark as finalizer (`#[miniextendr(r6(finalize))]`).
- `finalize_span`: `Option<proc_macro2::Span>`
  - Span of the `r6(finalize)` marker — see `private_span`.
- `deep_clone`: `bool`
  - Mark as R6 deep-clone handler (`#[miniextendr(r6(deep_clone))]`).
- `deep_clone_span`: `Option<proc_macro2::Span>`
  - Span of the `r6(deep_clone)` marker — see `private_span`.

### `miniextendr_impl::S7MethodAttrs`

```rust
pub struct S7MethodAttrs
```

S7-specific per-method markers, separated from [`MethodAttrs`] so the S7
class generator has a self-contained bag of its own state (property
getters/setters, generic-dispatch controls, convert() wiring) and the other
class generators don't have to look past them.

#### Mapping from `s7(...)` attribute keys

| Attribute | Field |
|-----------|-------|
| `s7(getter)` | `getter: true` |
| `s7(setter)` | `setter: true` |
| `s7(prop = "name")` | `prop: Some("name")` |
| `s7(default = "expr")` | `default: Some("expr")` |
| `s7(validate)` | `validate: true` |
| `s7(required)` | `required: true` |
| `s7(frozen)` | `frozen: true` |
| `s7(deprecated = "msg")` | `deprecated: Some("msg")` |
| `s7(no_dots)` | `no_dots: true` |
| `s7(dispatch = "x,y")` | `dispatch: Some("x,y")` |
| `s7(fallback)` | `fallback: true` |
| `s7(convert_from = "T")` | `convert_from: Some("T")` |
| `s7(convert_to = "T")` | `convert_to: Some("T")` |

**Fields:**

- `getter`: `bool`
- `setter`: `bool`
- `prop`: `Option<String>`
- `default`: `Option<String>`
- `validate`: `bool`
- `required`: `bool`
- `frozen`: `bool`
- `deprecated`: `Option<String>`
- `no_dots`: `bool`
- `dispatch`: `Option<String>`
- `fallback`: `bool`
- `convert_from`: `Option<String>`
- `convert_to`: `Option<String>`
- `no_shortcut`: `bool`
  - Opt out of the per-class `<ClassName>_<method>` fast-dispatch shortcut

### `miniextendr_impl::VctrsAttrs`

```rust
pub struct VctrsAttrs
```

Attributes for vctrs class generation.

**Fields:**

- `kind`: `VctrsKind`
  - The vctrs kind (vctr, rcrd, list_of)
- `base`: `Option<String>`
  - Base type for vctr (e.g., "double", "integer", "character")
- `inherit_base_type`: `Option<bool>`
  - Whether to inherit base type in class vector
- `ptype`: `Option<String>`
  - Prototype type for list_of (R expression)
- `abbr`: `Option<String>`
  - Abbreviation for vec_ptype_abbr (for printing)

### `r_class_formatter::ClassDocBuilder`

```rust
pub struct ClassDocBuilder<'a>
```

Builder for class-level roxygen documentation header.

Generates the common roxygen tags that appear at the start of each class definition:
- `@title` (unless user provided)
- `@name` (unless user provided)
- `@rdname` (unless user provided)
- User-provided doc tags
- `@source Generated by miniextendr...`
- Class-system-specific imports
- `@export` (unless user provided, `@noRd`, or internal/noexport flags)

**Inherent associated items:**

#### `build`

```rust
fn build(self: &Self) -> Vec<String>
```

Build the roxygen `#' @tag` lines for the class header.

Returns a vector of strings, each a complete roxygen comment line (e.g., `"#' @title ..."`).
Auto-generates `@title`, `@name`, and `@rdname` if not provided by the user, and
respects `@noRd` to suppress all documentation output.

#### `new`

```rust
fn new(class_name: &'a str, type_ident: &'a syn::Ident, doc_tags: &'a [String], class_system_label: &'static str) -> Self
```

Create a new ClassDocBuilder with the given class metadata.

By default, `@export` is included unless suppressed by user tags or
the `with_export_control` method.

#### `with_export_control`

```rust
fn with_export_control(self: Self, internal: bool, noexport: bool) -> Self
```

Set attribute-level internal/noexport flags from `ParsedImpl`.

#### `with_imports`

```rust
fn with_imports(self: Self, imports: impl Into<String>) -> Self
```

Set R package imports (e.g., "@importFrom R6 R6Class").

### `r_class_formatter::MethodContext`

```rust
pub struct MethodContext<'a>
```

Pre-computed context for a method, holding all data needed for R wrapper generation.

This struct captures the common computations performed for every method across all
class systems, reducing duplicate code. It pre-formats the C wrapper name, R formal
parameters (with defaults), and R call arguments so each class generator can
focus on its specific formatting logic.

**Fields:**

- `method`: `&'a crate::miniextendr_impl::ParsedMethod`
  - Reference to the parsed method metadata.
- `c_ident`: `String`
  - The C wrapper identifier string (e.g., `"C_Counter__inc"`), used in `.Call()`.
- `params`: `String`
  - R formals string with defaults (e.g., `"value, step = 1L"`), used in
- `args`: `String`
  - R call arguments string without defaults (e.g., `"value, step"`), used
- `no_preconditions`: `bool`
  - Drop the R-side `stopifnot(...)` block from the generated wrapper.
- `no_call_attribution`: `bool`
  - Emit `.call = NULL` instead of `.call = match.call()` in non-lambda

**Inherent associated items:**

#### `class_suffix`

```rust
fn class_suffix(self: &Self) -> Option<&str>
```

Get custom class suffix if specified.

This allows double-dispatch patterns like `vec_ptype2.my_class.my_class`
by specifying `#[miniextendr(s3(generic = "vec_ptype2", class = "my_class.my_class"))]`.

#### `emit_method_prelude`

```rust
fn emit_method_prelude(self: &Self, lines: &mut Vec<String>, indent: &str, what: &str)
```

Emit the 6-step method prelude into `lines`, each line prefixed with `indent`.

The prelude is the standardised sequence that appears at the top of every
generated R method body, in order:

1. `r_entry` — user code injected before any checks
2. `r_on_exit` — `on.exit(...)` cleanup
3. `lifecycle_prelude` — deprecation/superseded banner (class-system-specific label)
4. `precondition_checks` — `stopifnot(is.*(param))` for typed params
5. `match_arg_prelude` — `base::match.arg(param)` validation
6. `r_post_checks` — user code after all checks, before `.Call()`

(`Missing<T>` forwarding is not a prelude step: it lives inline in the
`.Call()` args — see `build_call_args_vec` — because a binding of the
missing sentinel errors on lookup.)

`what` is the human-readable method label passed to `lifecycle_prelude`
(e.g., `"Type.method"` for S3/S4, `"Type$method"` for Env/R6/S7).
`indent` is the per-line prefix (e.g., `"  "` for 2-space, `"      "` for 6-space).

#### `generic_name`

```rust
fn generic_name(self: &Self) -> String
```

Get the generic name (uses override if present).

#### `has_class_override`

```rust
fn has_class_override(self: &Self) -> bool
```

Check if this method uses a custom class suffix.

#### `has_generic_override`

```rust
fn has_generic_override(self: &Self) -> bool
```

Check if this method uses a generic override (for existing generics like print).

#### `instance_call`

```rust
fn instance_call(self: &Self, self_expr: &str) -> String
```

Build the `.Call()` expression for an instance method with `self` as ptr.

The `self_expr` is typically "self", "private$.ptr", "x", "x@ptr", or "x@.ptr".

#### `instance_call_null_attr`

```rust
fn instance_call_null_attr(self: &Self, self_expr: &str) -> String
```

Like [`instance_call`](Self::instance_call) but passes `.call = NULL`.

Use for lambda dispatch sites (S7 property getter/setter) where
`match.call()` captures the S7 dispatch frame, not the user's call.

#### `instance_formals`

```rust
fn instance_formals(self: &Self, add_self_param: bool) -> String
```

Build full R formals for instance methods (prefixing x/self parameter).

For S3/S4/S7: `"x, <params>, ..."`
For Env/R6: `"<params>"` (self is implicit)

#### `instance_formals_with_dots`

```rust
fn instance_formals_with_dots(self: &Self, add_self_param: bool, include_dots: bool) -> String
```

Build full R formals for instance methods with optional dots.

When `include_dots` is false, omits `...` from the signature.
This is used for strict generics that don't accept extra args.

#### `instance_formals_with_receiver`

```rust
fn instance_formals_with_receiver(self: &Self, receiver: &str, include_dots: bool) -> String
```

Build instance formals with a custom receiver name (default is `x`).

Used by the S7 per-class fast-path shortcut (#949), whose receiver is
named `self` to mirror the property dispatch lambdas, rather than the
`x` used by the S7 generic.

#### `match_arg_doc_placeholders`

```rust
fn match_arg_doc_placeholders(self: &Self) -> std::collections::HashMap<String, String>
```

Build the R-param-name → @param placeholder map for this method's
match_arg params. Pass to `MethodDocBuilder::with_match_arg_doc_placeholders`
so the cdylib write pass rewrites the placeholders into rendered choice
descriptions (#210).

#### `match_arg_prelude`

```rust
fn match_arg_prelude(self: &Self) -> Vec<String>
```

Build R prelude lines that validate `match_arg` / `choices` / `several_ok`
parameters via `base::match.arg()` before the `.Call()`.

Returns an empty vector when the method declares none. Both `match_arg`
and `choices(...)` carry their choice list as the formal default
(`c("a", "b", ...)`), so `base::match.arg(arg)` finds the list by
itself — no second arg, no C helper lookup. `match_arg` adds a
factor → character coercion in front of `match.arg`.

Callers should include these lines in the R wrapper body after parameter
defaulting but before the `.Call()`.

#### `new`

```rust
fn new(method: &'a ParsedMethod, type_ident: &syn::Ident, label: Option<&str>) -> Self
```

Create a new MethodContext for a method.

Computes the C wrapper identifier from the method name, type name, and optional
label (for multi-impl-block disambiguation), then formats the R formals and
call arguments from the method's signature and default values. Fast-path
knobs default off; use [`MethodContext::with_fast_flags`] to inherit them
from `ImplAttrs`.

#### `precondition_checks`

```rust
fn precondition_checks(self: &Self) -> Vec<String>
```

Build R-side precondition `stopifnot()` lines for this method's parameters.

Returns static checks for known types. Custom types not in the static table
are identified as fallback params but no R-side precheck is generated for them.

Skips `self`/receiver parameters automatically (they are `FnArg::Receiver`) and
any parameter validated by `base::match.arg()` (via `match_arg` / `choices`) —
those already have a stronger runtime guarantee than `stopifnot(is.character(...))`.

#### `source_comment`

```rust
fn source_comment(self: &Self, type_ident: &syn::Ident) -> String
```

Generate a source location comment for this method.

Returns a string like `# Type::method (line:col)` using the method's span info.
The file name is already stated in the impl block header comment, so line:col
is sufficient to locate the method within that file.

#### `static_call`

```rust
fn static_call(self: &Self) -> String
```

Build the `.Call()` expression for a static/constructor call.

#### `with_fast_flags`

```rust
fn with_fast_flags(self: Self, no_preconditions: bool, no_call_attribution: bool) -> Self
```

Set the fast-path flags inherited from the surrounding `ImplAttrs`.
Returns `self` so callers can chain on top of `MethodContext::new`.

### `r_class_formatter::MethodDocBuilder`

```rust
pub struct MethodDocBuilder<'a>
```

Builder for method-level roxygen documentation.

Generates roxygen tags for individual methods within a class. Methods share
the class's `@rdname` so they appear on the same help page. The builder handles
`@name` formatting (with optional prefix like `$` for `Class$method` style)
and respects `@noRd` inheritance from the parent class.

**Inherent associated items:**

#### `build`

```rust
fn build(self: &Self) -> Vec<String>
```

Build the roxygen `#' @tag` lines for the method.

Returns a vector of strings, each a complete roxygen comment line. If the parent
class has `@noRd`, returns only `["#' @noRd"]`. Otherwise generates `@name`,
`@rdname`, `@source`, and optionally `@export` tags, plus any user-provided tags.

#### `new`

```rust
fn new(class_name: &'a str, method_name: &'a str, type_ident: &'a syn::Ident, doc_tags: &'a [String]) -> Self
```

Create a new MethodDocBuilder with default settings.

By default, `always_export` is `false` because methods accessed via `Class$method`
should not be exported directly -- only the class env and standalone S3 methods
need `@export`.

#### `with_class_no_rd`

```rust
fn with_class_no_rd(self: Self, class_has_no_rd: bool) -> Self
```

Set whether the parent class has @noRd.

When true, skips @name, @rdname, @source tags and adds @noRd instead.

#### `with_match_arg_doc_placeholders`

```rust
fn with_match_arg_doc_placeholders(self: Self, placeholders: &'a std::collections::HashMap<String, String>) -> Self
```

Supply a map from R-param-name to a write-time doc placeholder for
match_arg'd params. When the auto-generated `@param` line would otherwise
say `(undocumented)`, the placeholder is emitted instead and the cdylib
write pass rewrites it to a rendered choice description. See #210.

#### `with_name_prefix`

```rust
fn with_name_prefix(self: Self, prefix: &'a str) -> Self
```

Set a prefix for the @name tag (e.g., "$" for "Class$method").

#### `with_params_as_details`

```rust
fn with_params_as_details(self: Self) -> Self
```

Convert `@param` tags to inline `\describe{}` blocks instead of roxygen `@param`.

Used for env-class methods where roxygen can't infer `\usage` from `Class$method <- function()`.
Without this, `@param` tags create `\arguments` entries with no matching `\usage`,
causing R CMD check warnings ("Documented arguments not in \\usage").

#### `with_r_name`

```rust
fn with_r_name(self: Self, r_name: String) -> Self
```

Override the @name tag with a custom R function name.

Use this when the R function name differs from the Rust method name
(e.g., for standalone S3/S4/S7 static methods like `s3counter_default_counter`).

#### `with_r_params`

```rust
fn with_r_params(self: Self, params: &'a str) -> Self
```

Set the method's formal parameter names (comma-separated R params string).

When set, auto-generates `@param name (undocumented)` for any parameter
not already covered by a user `@param` tag. Skips `self`, `.ptr`, and
`...` parameters.

#### `with_suppress_params`

```rust
fn with_suppress_params(self: Self) -> Self
```

Suppress `@param` tags from user doc comments.

Used for S4/S7 instance methods where the method is defined via `setMethod()`
or `S7::method()` assignment, which roxygen2 doesn't parse for `\usage` entries.

### `r_preconditions::FallbackParam`

```rust
pub struct FallbackParam
```

A parameter whose Rust type is not in the static type table.

Currently, fallback params are recorded but no R-side validation is generated
for them -- the Rust-side conversion handles type errors with its own messages.

**Fields:**

- `r_name`: `String`
  - R-normalized parameter name (e.g., `_dots` becomes `.dots`).

### `r_preconditions::PreconditionOptions`

```rust
pub struct PreconditionOptions
```

Per-function knobs that influence precondition codegen.

Currently only the `coerce` knob matters: `#[miniextendr(coerce)]` (or a
per-param `#[miniextendr(coerce)]`) changes the inbound conversion for an
integer-element vector to read via the native `&[i32]` slice and then
`TryCoerce` element-wise (see `rust_conversion_builder.rs` `CoercionMapping::Vec`).
That `&[i32]` read is INTSXP-only, so a coerced integer vector that would
otherwise accept whole-number `REALSXP` ([`RTypeCheck::VectorIntegerWide`])
must instead get the strict `is.integer` gate ([`RTypeCheck::VectorIntegerStrict`])
— otherwise a `double` passes the R precondition only to fail the Rust read
with "expected INTSXP, got REALSXP" (issue #616).

`strict` is intentionally not represented: inbound conversion is identical in
strict and default mode (`TryFromSexp`), so the R-side integer gate doesn't
change. The precise strict checking happens on the Rust *outbound* side.

**Fields:**

- `coerce_all`: `bool`
  - `coerce` knob is active for all parameters (`coerce_all`).
- `coerce_params`: `std::collections::HashSet<String>`
  - R-normalized names of parameters with a per-param `coerce` attribute.

### `r_preconditions::PreconditionOutput`

```rust
pub struct PreconditionOutput
```

Output of precondition analysis for a function's parameters.

Contains both the generated R `stopifnot()` code for known types and a list
of parameters with unknown types that were not statically prechecked.

**Fields:**

- `static_checks`: `Vec<String>`
  - Lines forming a `stopifnot(...)` call for known types.
- `fallback_params`: `Vec<FallbackParam>`
  - Parameters with unknown custom types that were not prechecked.

### `r_wrapper_builder::DotCallBuilder`

```rust
pub struct DotCallBuilder
```

Builder for formatting `.Call()` invocations in R wrapper code.

Handles the common pattern of `.Call(C_ident, .call = match.call(), args...)`.

#### Example

```ignore
let call = DotCallBuilder::new("C_Counter__increment")
    .with_self("self")
    .build();
// => ".Call(C_Counter__increment, .call = match.call(), self)"

let call = DotCallBuilder::new("C_Counter__add")
    .with_self("x")
    .with_args(&["n"])
    .build();
// => ".Call(C_Counter__add, .call = match.call(), x, n)"
```

**Inherent associated items:**

#### `build`

```rust
fn build(self: &Self) -> String
```

Build the `.Call()` string.

#### `new`

```rust
fn new(c_ident: impl Into<String>) -> Self
```

Create a new builder with the C function identifier.

#### `null_call_attribution`

```rust
fn null_call_attribution(self: Self) -> Self
```

Pass `.call = NULL` instead of `.call = match.call()`.

Use for lambda dispatch sites (R6 finalizer/`deep_clone`, S7 property
getter/setter/validator) where `match.call()` captures an internal
dispatch frame instead of the user's call. With `NULL`, the
`if (is.null(.val$call)) .call_default else .val$call` fallback in `condition_check_lines` surfaces the
nearest meaningful frame instead.

#### `with_args`

```rust
fn with_args(self: Self, args: &[impl AsRef<str>]) -> Self
```

Add arguments after self (if any).

#### `with_args_str`

```rust
fn with_args_str(self: Self, args: &str) -> Self
```

Add a pre-joined argument string (e.g., `"x, y"`) as a single emit unit.

Empty strings are ignored, so callers can pass the result of
`build_r_call_args_from_sig` directly without a length check.

#### `with_self`

```rust
fn with_self(self: Self, var: impl Into<String>) -> Self
```

Add a self/x parameter (prepended to args).

### `r_wrapper_builder::RArgumentBuilder`

```rust
pub struct RArgumentBuilder<'a>
```

Builder for R function formal parameters and call arguments.

Handles:
- Underscore normalization (`_x` → `unused_x`)
- Unit type defaults (`()` → `= NULL`)
- Dots (`...`) with optional naming
- Consistent formatting across function and method wrappers

**Inherent associated items:**

#### `build_call_args`

```rust
fn build_call_args(self: &Self) -> String
```

Build R call arguments string (for `.Call()` invocation).

##### Returns
Comma-separated argument list, e.g., `"x, y, list(...)"`

#### `build_call_args_vec`

```rust
fn build_call_args_vec(self: &Self) -> Vec<String>
```

Build R call arguments as a `Vec<String>`.

Each element is a single argument expression. Dots parameters become
`"list(...)"` to capture variadic args as an R list for the `.Call()` interface.

#### `build_formals`

```rust
fn build_formals(self: &Self) -> String
```

Build R formal parameters string (for function signature).

##### Returns
Comma-separated parameter list, e.g., `"x, y = NULL, ..."`

This method handles R-style defaults (like `1L`, `c(1,2,3)`) that aren't
valid Rust syntax by outputting them directly as strings.

#### `new`

```rust
fn new(inputs: &'a syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> Self
```

Create a new builder for the given function inputs.

#### `skip_first`

```rust
fn skip_first(self: Self) -> Self
```

Skip the first parameter (for instance methods with `self`).

#### `with_defaults`

```rust
fn with_defaults(self: Self, defaults: std::collections::HashMap<String, String>) -> Self
```

Add parameter defaults from `#[miniextendr(default = "...")]` attributes.

Keys are normalized R parameter names (after underscore stripping),
values are R expression strings emitted verbatim into formals.

#### `with_dots`

```rust
fn with_dots(self: Self, named_dots: Option<String>) -> Self
```

Mark the last parameter as dots (`...`).

If `named_dots` is `Some("name")`, the dots have a Rust-side binding
(from `name: ...` syntax). The name is normalized but only affects the
Rust side -- R formals always emit plain `...`.

### `r_wrapper_builder::RoxygenBuilder`

```rust
pub struct RoxygenBuilder
```

Builder for generating roxygen2 documentation tags.

Provides a fluent API for building common roxygen tag patterns used
across all class systems.

#### Example

```ignore
let tags = RoxygenBuilder::new()
    .name("Counter$increment")
    .rdname("Counter")
    .export()
    .build();
// => vec!["#' @name Counter$increment", "#' @rdname Counter", "#' @export"]
```

**Inherent associated items:**

#### `build`

```rust
fn build(self: &Self) -> Vec<String>
```

Build the roxygen tag lines (each prefixed with `#' `).

#### `custom`

```rust
fn custom(self: Self, tag: impl Into<String>) -> Self
```

Add a custom tag line (without the `#' ` prefix).

#### `description`

```rust
fn description(self: Self, desc: impl Into<String>) -> Self
```

Set the `@description` tag.

#### `export`

```rust
fn export(self: Self) -> Self
```

Add `@export` tag.

#### `export_method`

```rust
fn export_method(self: Self, method: impl Into<String>) -> Self
```

Add `@exportMethod` tag (for S4).

#### `method`

```rust
fn method(self: Self, generic: impl Into<String>, class: impl Into<String>) -> Self
```

Add `@method` tag (for S3).

#### `name`

```rust
fn name(self: Self, name: impl Into<String>) -> Self
```

Set the `@name` tag.

#### `new`

```rust
fn new() -> Self
```

Create a new empty builder.

#### `rdname`

```rust
fn rdname(self: Self, rdname: impl Into<String>) -> Self
```

Set the `@rdname` tag (groups docs into one page).

#### `source`

```rust
fn source(self: Self, source: impl Into<String>) -> Self
```

Set the `@source` tag (typically "Generated by miniextendr...").

#### `title`

```rust
fn title(self: Self, title: impl Into<String>) -> Self
```

Set the `@title` tag.

### `rust_conversion_builder::RustConversionBuilder`

```rust
pub struct RustConversionBuilder
```

Builder for generating Rust conversion statements from R SEXP parameters.

Handles:
- Unit types `()` → identity binding
- `&Dots` → special wrapper with storage
- Slices `&[T]` → TryFromSexp
- `&str` → String + Borrow (for worker thread compatibility)
- Scalar references → DATAPTR_RO_unchecked
- Coercion → extract R native type + TryCoerce
- Default → TryFromSexp

**Inherent associated items:**

#### `build_conversion`

```rust
fn build_conversion(self: &Self, pat_type: &syn::PatType, sexp_ident: &syn::Ident) -> Vec<TokenStream>
```

Generate conversion statement for a single parameter.

This is the non-split variant: owned conversions and borrow statements are
concatenated into a single list, suitable for main-thread execution where
everything runs in the same scope.

- `pat_type`: the typed pattern from the function signature (e.g., `x: i32`).
- `sexp_ident`: the identifier of the raw SEXP variable holding the R argument.

Returns a flat list of `let` binding statements that convert `sexp_ident` into
the Rust type declared in `pat_type`.

#### `build_conversion_split`

```rust
fn build_conversion_split(self: &Self, pat_type: &syn::PatType, sexp_ident: &syn::Ident) -> (Vec<TokenStream>, Vec<TokenStream>)
```

Generate conversion statements split into two phases for worker thread execution.

For reference types like `&str`, we need to:
1. Convert SEXP to owned type (String) -- runs on the main thread before the
   worker closure, so the owned value can be moved into the closure.
2. Borrow from the owned type (`&str`) -- runs inside the worker closure.

For non-reference types (scalars, `Vec`, etc.) everything goes into the first
phase and the second vec is empty.

- `pat_type`: the typed pattern from the function signature (e.g., `s: &str`).
- `sexp_ident`: the identifier of the raw SEXP variable holding the R argument.

Returns `(owned_conversions, borrow_statements)` where each element is a list
of `let` binding token streams.

#### `build_conversions`

```rust
fn build_conversions(self: &Self, inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>, sexp_idents: &[syn::Ident]) -> Vec<TokenStream>
```

Generate conversion statements for all parameters in a function signature.

Iterates over `inputs` (the function's parameter list) paired with `sexp_idents`
(the corresponding SEXP variable names), calling [`build_conversion`](Self::build_conversion)
for each typed parameter. Receiver parameters (`self`) are silently skipped.

Returns a flat list of all conversion statements, in parameter order.

#### `new`

```rust
fn new() -> Self
```

Create a new conversion builder.

#### `with_coerce_all`

```rust
fn with_coerce_all(self: Self) -> Self
```

Enable coercion for all parameters.

#### `with_coerce_param`

```rust
fn with_coerce_param(self: Self, param_name: String) -> Self
```

Add a single parameter name that should use coercion.

`param_name` is matched against the identifier in the function signature.
Can be called multiple times to add several parameters.

#### `with_match_arg_several_ok`

```rust
fn with_match_arg_several_ok(self: Self, param_name: String) -> Self
```

Mark a parameter as `match_arg + several_ok` — uses `match_arg_vec_from_sexp`
instead of `TryFromSexp` for converting STRSXP → `Vec<EnumType>`.

#### `with_strict`

```rust
fn with_strict(self: Self) -> Self
```

Enable strict input conversion for lossy types (i64/u64/isize/usize + Vec variants).

### `typed_dataframe::TypedDataframeField`

```rust
pub struct TypedDataframeField
```

A single column declaration.

**Fields:**

- `attrs`: `Vec<syn::Attribute>`
  - Field attributes (doc comments etc.).
- `name`: `syn::Ident`
  - Column name (as used in R `names(df)`).
- `elem_ty`: `syn::Type`
  - Element type (e.g. `i32`, `f64`).
- `optional`: `bool`
  - True if the field was declared `Option<T>` (column may be absent).

### `typed_dataframe::TypedDataframeInput`

```rust
pub struct TypedDataframeInput
```

Parsed input for `typed_dataframe!`.

**Fields:**

- `attrs`: `Vec<syn::Attribute>`
  - Outer attributes (doc comments etc.) on the struct.
- `vis`: `syn::Visibility`
  - Visibility (e.g. `pub`).
- `name`: `syn::Ident`
  - Struct name.
- `fields`: `Vec<TypedDataframeField>`
  - Declared columns.
- `allow_extra`: `bool`
  - If `false`, the input data.frame may not have extra (un-declared) columns.

### `typed_list::ParsedEntry`

```rust
pub struct ParsedEntry
```

A single entry in the typed list spec.

**Fields:**

- `name`: `syn::Ident`
  - The field name.
- `optional`: `bool`
  - Whether this field is optional (marked with `?`).
- `spec`: `Option<ParsedTypeSpec>`
  - The type spec (None = Any).

### `typed_list::TypedListInput`

```rust
pub struct TypedListInput
```

Parsed typed_list! macro input.

**Fields:**

- `allow_extra`: `bool`
  - Whether @exact mode is enabled (strict, no extra fields).
- `entries`: `Vec<ParsedEntry>`
  - The list entries.

---

## Enums

### `c_wrapper_builder::ReturnHandling`

```rust
pub enum ReturnHandling
```

Strategy for converting a Rust return value into an R `SEXP`.

Determined automatically by [`detect_return_handling`] from the function's return type,
or set explicitly via [`CWrapperContextBuilder::return_handling`]. Each variant
handles a different return type pattern, controlling how the C wrapper converts
the Rust value back to R and how errors/None values are surfaced.

**Variants:**

- `Unit`
  - Returns unit type `()` -- emits `R_NilValue`.
- `RawSexp`
  - Returns raw `SEXP` -- passes the value through unchanged (no conversion).
- `ExternalPtr`
  - Returns `Self` -- wraps the value in an `ExternalPtr` via `ExternalPtr::new`.
- `SelfHandle`
  - Returns `&Self` / `&mut Self` (an in-place builder) -- evaluates the call
- `IntoR`
  - Returns an arbitrary type `T: IntoR` -- converts via `IntoR::into_sexp`.
- `OptionUnit`
  - Returns `Option<()>` -- raises an error on `None`, otherwise emits `R_NilValue`.
- `OptionSexp`
  - Returns `Option<SEXP>` -- raises an error on `None`, otherwise passes through.
- `OptionIntoR`
  - Returns `Option<T>` where `Option<T>: IntoR` -- calls `IntoR::into_sexp` on the whole
- `OptionIntoRUnwrap`
  - Returns `Option<T>` where `T: IntoR` -- unwraps the option first, then converts the
- `OptionExternalPtr`
  - Returns `Option<Self>` -- a lookup-shaped fallible constructor (e.g. `try_find`).
- `ResultUnit`
  - Returns `Result<(), E>` -- raises an error on `Err`, otherwise emits `R_NilValue`.
- `ResultSexp`
  - Returns `Result<SEXP, E>` -- raises an error on `Err`, otherwise passes through.
- `ResultIntoR`
  - Returns `Result<T, E>` -- raises an error on `Err`, otherwise converts via `IntoR::into_sexp`.
- `ResultNullOnErr`
  - Returns `Result<T, ()>` -- maps `Err(())` to `Err(NullOnErr)` then converts via `IntoR`.
- `ResultExternalPtr`
  - Returns `Result<Self, E>` -- a fallible constructor-shaped method (e.g. `from_r`,
- `AsListOf`
  - Returns `T` where `T: IntoList` -- wraps in `AsList(result)` then calls `IntoR::into_sexp`.
- `AsExternalPtrOf`
  - Returns `T` where `T: IntoExternalPtr` -- wraps in `AsExternalPtr(result)` then calls `IntoR::into_sexp`.
- `AsNativeOf`
  - Returns `T` where `T: RNativeType` -- wraps in `AsRNative(result)` then calls `IntoR::into_sexp`.

### `c_wrapper_builder::ThreadStrategy`

```rust
pub enum ThreadStrategy
```

Thread execution strategy for C wrappers.

**Variants:**

- `MainThread`
  - Execute on main R thread with `with_r_unwind_protect`. **Default.**
- `WorkerThread`
  - Execute on worker thread with panic catching. **Opt-in via `#[miniextendr(worker)]`.**

### `lifecycle::LifecycleStage`

```rust
pub enum LifecycleStage
```

Lifecycle stage for a function, method, or argument.

**Variants:**

- `Experimental`
  - Function is experimental and may change.
- `Stable`
  - Function is stable (no badge/warning needed).
- `Superseded`
  - Function has a better alternative but will be maintained.
- `SoftDeprecated`
  - Function should no longer be used (soft warning first).
- `Deprecated`
  - Function should no longer be used (warning).
- `Defunct`
  - Function no longer works (error).

**Inherent associated items:**

#### `badge`

```rust
fn badge(self: &Self) -> Option<&'static str>
```

Get the inline R roxygen expression for the lifecycle badge.

Returns an R inline code expression like `` `r lifecycle::badge("experimental")` ``
that roxygen2 evaluates to render a colored badge. Returns `None` for `Stable`
(no badge needed).

#### `from_str`

```rust
fn from_str(s: &str) -> Option<Self>
```

Parse a lifecycle stage from a string.

Accepts lowercase stage names. Both `"soft-deprecated"` and `"soft_deprecated"`
are recognized. Returns `None` for unrecognized strings.

#### `import_from_fn`

```rust
fn import_from_fn(self: &Self) -> Option<&'static str>
```

Get the bare R function name for `@importFrom lifecycle` roxygen tag.

Returns the function name without the `lifecycle::` prefix.

#### `keywords`

```rust
fn keywords(self: &Self) -> Option<&'static str>
```

Get the roxygen `@keywords` value, if this stage needs one.

Only `Experimental` adds `@keywords internal` to keep the function
off the main package index. Returns `None` for all other stages.

#### `signal_fn`

```rust
fn signal_fn(self: &Self) -> Option<&'static str>
```

Get the fully-qualified lifecycle signal function name.

Returns the R function to call at the start of the wrapper body to emit
the lifecycle signal (e.g., `"lifecycle::deprecate_warn"`). Returns `None`
for `Stable` (no signal needed).

### `list_macro::ListName`

```rust
pub enum ListName
```

Name for a list entry -- either a Rust identifier or a string literal.

Identifiers are used for simple names (e.g., `alpha = 1`), while string
literals allow names that are not valid Rust identifiers (e.g., `"my-name" = 1`).

**Variants:**

- `Ident(syn::Ident)`
  - A bare Rust identifier used as the entry name (e.g., `alpha`).
- `Str(syn::LitStr)`
  - A string literal used as the entry name (e.g., `"my-name"`).

### `method_return_builder::ReturnStrategy`

```rust
pub enum ReturnStrategy
```

Return handling strategy for class methods.

Determines how the R wrapper function processes and returns the `.Call()` result.
Each class system generator uses this to produce idiomatic R return code.

**Variants:**

- `ReturnSelf`
  - The method returns `Self`. The wrapper wraps the raw pointer result with
- `ReturnOtherClass`
  - The method returns a bare type name that may be another registered
- `ReturnOtherClassList`
  - The method returns a *list* of values (`Vec<Class>`, plus the
- `ChainableMutation`
  - The method is a `&mut self` method returning `()`. The wrapper calls the
- `Direct`
  - Default strategy: return the `.Call()` result directly without wrapping.

**Inherent associated items:**

#### `for_method`

```rust
fn for_method(method: &ParsedMethod) -> Self
```

Determine the return strategy for a parsed method.

- Methods that return `Self`, `Result<Self, E>`, or `Option<Self>` use
  `ReturnSelf`. For the latter two, the C wrapper already raised on
  `Err` / `None` (see
  [`crate::c_wrapper_builder::ReturnHandling::ResultExternalPtr`] and
  [`crate::c_wrapper_builder::ReturnHandling::OptionExternalPtr`]), so a
  successful `.val` is a bare ExternalPtr — identical in shape to the
  bare-`Self` case — and gets the same class-wrapping tail.
- In-place builders (`&mut self -> &mut Self` / `&self -> Self`) and
  `&mut self -> ()` methods use `ChainableMutation`. Both return the
  receiver object (`x` / `invisible(self)`) so the call composes under
  the native pipe (`obj |> set_a(1) |> set_b(2)`); the C wrapper hands
  back the same ExternalPtr handle (see
  [`crate::c_wrapper_builder::ReturnHandling::SelfHandle`]).
- Bare capitalized return types that are not known primitives/containers
  use `ReturnOtherClass`; write-time registry lookup wraps registered
  classes and leaves false positives unchanged.
- `Vec<Class>` (and the `Option`/`Result` wrappers of it the C wrapper
  unwraps) use `ReturnOtherClassList`; write-time registry lookup wraps
  registered element classes via `lapply` and leaves false positives
  unchanged (#1284).
- All other methods use `Direct`.

### `miniextendr_impl::ClassSystem`

```rust
pub enum ClassSystem
```

Class system flavor for wrapper generation.

**Variants:**

- `Env`
  - Environment-style with `$`/`[[` dispatch
- `R6`
  - R6::R6Class
- `S7`
  - S7::new_class
- `S3`
  - S3 structure() with class attribute
- `S4`
  - S4 setClass
- `Vctrs`
  - vctrs-compatible S3 class (vctr, rcrd, or list_of)

**Inherent associated items:**

#### `from_ident`

```rust
fn from_ident(ident: &syn::Ident) -> Option<Self>
```

Parse from an identifier (inverse of `to_ident`).

#### `to_ident`

```rust
fn to_ident(self: Self) -> syn::Ident
```

Convert to an identifier for token transport (e.g., in macro_rules! expansion).

### `miniextendr_impl::ReceiverKind`

```rust
pub enum ReceiverKind
```

Receiver kind for methods.

**Variants:**

- `None`
  - No env - static/associated function
- `Ref`
  - `&self` - immutable borrow
- `RefMut`
  - `&mut self` - mutable borrow
- `Value`
  - `self` - consuming (not supported in v1)
- `ExternalPtrRef`
  - `self: &ExternalPtr<Self>` — immutable borrow of the wrapping ExternalPtr
- `ExternalPtrRefMut`
  - `self: &mut ExternalPtr<Self>` — mutable borrow of the wrapping ExternalPtr
- `ExternalPtrValue`
  - `self: ExternalPtr<Self>` — owned ExternalPtr (not consuming the inner T)

**Inherent associated items:**

#### `is_instance`

```rust
fn is_instance(self: &Self) -> bool
```

Returns true if this is an instance method (has self).

#### `is_mut`

```rust
fn is_mut(self: &Self) -> bool
```

Returns true if this is a mutable instance receiver.

### `miniextendr_impl::VctrsKind`

```rust
pub enum VctrsKind
```

Kind of vctrs class being created.

**Variants:**

- `Vctr`
  - Simple vctr backed by a base vector (new_vctr)
- `Rcrd`
  - Record type with named fields (new_rcrd)
- `ListOf`
  - Homogeneous list with ptype (new_list_of)

### `typed_list::ParsedTypeSpec`

```rust
pub enum ParsedTypeSpec
```

Parsed type specification from a `typed_list!` entry's `=> type` clause.

**Variants:**

- `StringLit(String)`
  - A string literal type spec (e.g., `"numeric"`, `"data.frame"`, `"myclass"`).
- `TypeCall { name: String, len: Option<usize> }`
  - A call-like type spec (e.g., `numeric()`, `integer(4)`).

---

## Traits

### `r_class_formatter::ParsedImplExt`

```rust
pub trait ParsedImplExt
```

Extension trait for `ParsedImpl` to iterate over methods as [`MethodContext`].

Provides convenience methods that wrap `ParsedImpl`'s method iterators,
automatically constructing a `MethodContext` for each method. This avoids
repeating the `MethodContext::new(m, type_ident, label)` boilerplate in
every class system generator.

**Required methods:**

- `fn constructor_context(self: &Self) -> Option<MethodContext<'_>>`
  - Create a `MethodContext` for the constructor method, if one exists.
- `fn instance_method_contexts(self: &Self) -> impl Iterator<Item = MethodContext<'_>>`
  - Iterate over all instance methods (public + private + active) as `MethodContext`.
- `fn static_method_contexts(self: &Self) -> impl Iterator<Item = MethodContext<'_>>`
  - Iterate over static (non-receiver) methods as `MethodContext`.
- `fn public_instance_method_contexts(self: &Self) -> impl Iterator<Item = MethodContext<'_>>`
  - Iterate over public instance methods as `MethodContext` (for R6 `public` list).
- `fn private_instance_method_contexts(self: &Self) -> impl Iterator<Item = MethodContext<'_>>`
  - Iterate over private instance methods as `MethodContext` (for R6 `private` list).
- `fn active_instance_method_contexts(self: &Self) -> impl Iterator<Item = MethodContext<'_>>`
  - Iterate over active binding methods as `MethodContext` (for R6 `active` list).

---

## Functions

### `altrep::derive_altrep`

```rust
fn derive_altrep(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Entry point for `#[derive(Altrep)]`.

Generates ALTREP registration only (TypedExternal, AltrepClass,
RegisterAltrep, IntoR, linkme entry, Ref/Mut accessor types).

The struct must already have low-level ALTREP traits implemented.
For most use cases, prefer a family-specific derive instead:
`#[derive(AltrepInteger)]`, `#[derive(AltrepReal)]`, etc.
Those generate both the low-level traits AND registration.
Use `#[altrep(manual)]` on a family derive to skip data trait generation
when you provide your own `AltrepLen` + `Alt*Data` impls.

#### Helper attributes

```ignore
#[altrep(class = "CustomName")]  // override ALTREP class name (default: struct name)
```

### `altrep_derive::derive_altrep_complex`

```rust
fn derive_altrep_complex(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Derive macro entry point for `AltrepComplex`.

Auto-implements `AltrepLen` and `AltComplexData` for a struct with a length field.
The `elt()` method returns `self.{elt_field}` as `Rcomplex` if
`#[altrep(elt = "...")]` is specified, or `Rcomplex { r: NAN, i: NAN }` by default.

Supports `#[altrep(dataptr)]` for direct `Rcomplex` data pointer access and
`#[altrep(subset)]` for `Extract_subset`.

### `altrep_derive::derive_altrep_integer`

```rust
fn derive_altrep_integer(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Derive macro entry point for `AltrepInteger`.

Auto-implements `AltrepLen` and `AltIntegerData` for a struct with a length field.
The `elt()` method returns `self.{elt_field}` as `i32` if `#[altrep(elt = "...")]`
is specified, or `NA_INTEGER` by default.

Supports `#[altrep(dataptr)]` for direct `i32` data pointer access and
`#[altrep(subset)]` for `Extract_subset`.

### `altrep_derive::derive_altrep_list`

```rust
fn derive_altrep_list(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Derive macro entry point for `AltrepList`.

Auto-implements `AltrepLen` and `AltListData` for a struct with a length field.
The `elt()` method returns `self.{elt_field}[i]` as `SEXP` if
`#[altrep(elt = "...")]` is specified (the field should be indexable and return `SEXP`),
or `R_NilValue` by default.

List ALTREP does **not** support `#[altrep(dataptr)]` or `#[altrep(subset)]` -- both
are rejected at compile time. `#[altrep(serialize)]` is supported.

### `altrep_derive::derive_altrep_logical`

```rust
fn derive_altrep_logical(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Derive macro entry point for `AltrepLogical`.

Auto-implements `AltrepLen` and `AltLogicalData` for a struct with a length field.
The `elt()` method returns `self.{elt_field}.into()` as `Logical` if
`#[altrep(elt = "...")]` is specified, or `Logical::Na` by default.

Supports `#[altrep(dataptr)]` for direct `i32` data pointer access (logicals are
stored as `i32` in R) and `#[altrep(subset)]` for `Extract_subset`.

### `altrep_derive::derive_altrep_raw`

```rust
fn derive_altrep_raw(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Derive macro entry point for `AltrepRaw`.

Auto-implements `AltrepLen` and `AltRawData` for a struct with a length field.
The `elt()` method returns `self.{elt_field}` as `u8` if `#[altrep(elt = "...")]`
is specified, or `0u8` by default.

Supports `#[altrep(dataptr)]` for direct `u8` data pointer access and
`#[altrep(subset)]` for `Extract_subset`.

### `altrep_derive::derive_altrep_real`

```rust
fn derive_altrep_real(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Derive macro entry point for `AltrepReal`.

Auto-implements `AltrepLen` and `AltRealData` for a struct with a length field.
The `elt()` method returns `self.{elt_field}` as `f64` if `#[altrep(elt = "...")]`
is specified, or `f64::NAN` (R's `NA_real_`) by default.

Supports `#[altrep(dataptr)]` for direct `f64` data pointer access and
`#[altrep(subset)]` for `Extract_subset`.

### `altrep_derive::derive_altrep_string`

```rust
fn derive_altrep_string(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Derive macro entry point for `AltrepString`.

Auto-implements `AltrepLen` and `AltStringData` for a struct with a length field.
The `elt()` method returns `Some(self.{elt_field}.as_ref())` as `Option<&str>` if
`#[altrep(elt = "...")]` is specified, or `None` (R's `NA_character_`) by default.

String ALTREP supports `#[altrep(dataptr)]` for materialized `STRSXP` dataptr
(via `__impl_altvec_string_dataptr`) and `#[altrep(subset)]` for `Extract_subset`.
Note: String dataptr materializes the entire vector into a cached `STRSXP` in the
data2 slot.

### `c_wrapper_builder::detect_return_handling`

```rust
fn detect_return_handling(output: &syn::ReturnType) -> ReturnHandling
```

Detects the appropriate [`ReturnHandling`] strategy from a function's return type.

Inspects the `syn::ReturnType`:
- No return type annotation (`Default`) maps to [`ReturnHandling::Unit`].
- An explicit type is analyzed by [`detect_return_handling_from_type`].

### `c_wrapper_builder::detect_return_handling_standalone_fn`

```rust
fn detect_return_handling_standalone_fn(output: &syn::ReturnType) -> ReturnHandling
```

Detects [`ReturnHandling`] for the standalone-`#[miniextendr]`-fn path.

Identical to [`detect_return_handling`] except that general `Option<T>` maps to
[`ReturnHandling::OptionIntoR`] (call `into_sexp` on the whole Option, matching the
historical `analyze_return_type` behavior) rather than [`ReturnHandling::OptionIntoRUnwrap`]
(the default that preserves impl-method behavior). Use this when building a
[`CWrapperContext`] for a standalone function.

### `dataframe_derive::derive_dataframe_row`

```rust
fn derive_dataframe_row(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Derive `DataFrameRow`: generates a companion DataFrame type with collection fields.

#### Requirements

For structs: the type must implement `IntoList`.
For enums: all variants must have named fields.

#### Generated Items

For a struct `Measurement { time: f64, value: f64 }`:
- Struct `MeasurementDataFrame { time: Vec<f64>, value: Vec<f64> }`
- `impl IntoDataFrame for MeasurementDataFrame` (rows → R `data.frame`)
- `impl ColumnarFrame for MeasurementDataFrame` (rows ↔ the pure-Rust companion)
- `impl From<Vec<Measurement>> for MeasurementDataFrame`
- `impl IntoIterator for MeasurementDataFrame`

For an enum:
- Companion struct with `Vec<Option<T>>` columns (field-name union)
- Optional tag column for variant discrimination
- `impl From<Vec<Enum>> for EnumDataFrame`
- `impl IntoDataFrame for EnumDataFrame`
- `impl ColumnarFrame for EnumDataFrame`
- `impl DataFrameRowSplit for Enum` (the split representation behind
  `IntoDataFrameSplit`)

#### Public surface (which verbs to call)

- Rows → R `data.frame`: `rows.into_dataframe()?` (`/_par`) or
  `rows.wrap_data_frame()`, from `IntoDataFrame` / `AsDataFrameExt`.
- R `data.frame` → rows: `Vec::<Row>::from_dataframe(&df)?` (`/_par`) from
  `FromDataFrame`, or the one-call `Row::try_from_dataframe(sexp)` reader.
- Rows ↔ the pure-Rust columnar companion: the `ColumnarFrame` trait
  (`<Row>DataFrame::from_rows` / `from_rows_par` / `into_rows`), or the `std`
  `Vec<Row>: Into<companion>` / companion `IntoIterator`.
- Enums: `rows.into_dataframe_split()` (one `data.frame` per variant) from
  `IntoDataFrameSplit`, implemented via the hidden `DataFrameRowSplit` bridge
  on the enum type.

All the traits above are re-exported from `miniextendr_api::prelude`. The row
type also carries two `#[doc(hidden)]` helpers: `to_dataframe(rows) -> companion`,
retained only because the struct-flatten / nested-enum write paths build a
nested companion via `Inner::to_dataframe(..)` without naming its type, and
`to_dataframe_split(rows)`, the body holder behind the `DataFrameRowSplit` bridge.

#### Attributes

- `#[dataframe(name = "CustomName")]` — Custom companion type name
- `#[dataframe(align)]` — Enum alignment mode (accepted but implicit)
- `#[dataframe(tag = "col")]` — Add variant discriminator column

### `externalptr_derive::derive_external_ptr`

```rust
fn derive_external_ptr(input: syn::DeriveInput, emit_into_r_marker: bool) -> syn::Result<proc_macro2::TokenStream>
```

Main entry point for `#[derive(ExternalPtr)]`.

Orchestrates the full derive expansion:
1. Parses `#[externalptr(...)]` attributes for class system selection.
2. Analyzes struct fields for `#[r_data]` sidecar slots.
3. Generates `TypedExternal` impl (type identity for `ExternalPtr<T>`).
4. Generates `IntoExternalPtr` marker impl (enables `IntoR` blanket impl)
   and a concrete `IntoRVecElement` impl (enables the `IntoR for Vec<T>`
   blanket, so `-> Vec<MyType>` returns a `VECSXP` of external pointers —
   #1284). Both are suppressed when `emit_into_r_marker` is `false`, which
   the struct-level `#[miniextendr(prefer = "native")]` path uses so its
   concrete `AsRNative` `IntoR` does not collide with the blanket
   `IntoExternalPtr` `IntoR` (E0119, #1283).
5. Generates sidecar accessor FFI functions, registration constants, and R wrappers.

Returns the combined token stream of all generated items.

### `factor_derive::derive_r_factor`

```rust
fn derive_r_factor(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Main entry point for `#[derive(RFactor)]`.

Dispatches to either [`derive_simple_factor`] (C-style unit variants) or
[`derive_interaction_factor`] (tuple variants wrapping an inner RFactor type),
based on whether `#[r_factor(interaction = [...])]` is present.

Generates:
- `impl MatchArg` (string choices for `match.arg`)
- `impl RFactor` (1-based level index conversion)
- `impl IntoR` (Rust enum -> R factor SEXP)
- `impl TryFromSexp` (R factor SEXP -> Rust enum)

Returns `Err` for structs, unions, or invalid attribute combinations.

### `lifecycle::collect_lifecycle_imports`

```rust
fn collect_lifecycle_imports<'a>(specs: impl Iterator<Item = &'a LifecycleSpec>) -> Option<String>
```

Collect the `@importFrom lifecycle ...` roxygen tag needed for a set of lifecycle specs.

This is used by class generators (R6, env, S3, S4, S7) to aggregate lifecycle
imports from all methods and include them in the class-level roxygen block.
Returns `None` if no lifecycle imports are needed.

### `lifecycle::inject_lifecycle_badge`

```rust
fn inject_lifecycle_badge(tags: &mut Vec<String>, spec: &LifecycleSpec)
```

Inject lifecycle badge into roxygen tags if not already present.

Modifies the tags in place, prepending the badge to @description if present,
or adding a new @description tag with just the badge.

### `lifecycle::inject_lifecycle_imports`

```rust
fn inject_lifecycle_imports(tags: &mut Vec<String>, spec: &LifecycleSpec)
```

Inject `@importFrom lifecycle` roxygen tags for the signal function and badge.

Adds the necessary `@importFrom` tag so roxygen2 registers the lifecycle
dependency in NAMESPACE. This is only added if not already present.

### `lifecycle::parse_lifecycle_attr`

```rust
fn parse_lifecycle_attr(meta: &syn::Meta) -> syn::Result<Option<LifecycleSpec>>
```

Parse lifecycle spec from miniextendr attribute arguments.

Supports:
- `lifecycle = "deprecated"` (simple stage)
- `lifecycle(stage = "deprecated", when = "0.4.0", with = "new_fn()")` (full spec)

### `lifecycle::parse_rust_deprecated`

```rust
fn parse_rust_deprecated(attr: &syn::Attribute) -> Option<LifecycleSpec>
```

Extract lifecycle info from a `#[deprecated]` attribute.

Handles all three forms: `#[deprecated]`, `#[deprecated = "msg"]`,
and `#[deprecated(since = "...", note = "...")]`. Returns `None` if the
attribute is not `#[deprecated]`.

### `list_derive::derive_into_list`

```rust
fn derive_into_list(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Derive `IntoList` for structs (Rust -> R).

Generates an `impl IntoList for T` that converts the struct into an R list:
- Named structs (`struct Foo { x: i32 }`) produce a named R list: `list(x = 1L)`
- Tuple structs (`struct Foo(i32, i32)`) produce an unnamed R list: `list(1L, 2L)`
- Unit structs (`struct Foo`) produce an empty R list: `list()`

Fields marked with `#[into_list(ignore)]` are excluded from the list.
Each non-ignored field's type must implement `IntoR` (enforced via where-clause bounds).

Returns `Err` if applied to a non-struct type or if an unknown field attribute is found.

### `list_derive::derive_prefer_data_frame`

```rust
fn derive_prefer_data_frame(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Derive `PreferDataFrame`: emits an `IntoR` impl that converts to R via
`ColumnSource::into_column_list`, then `into_sexp`.

The type must implement `ColumnSource` (typically the companion struct generated
by `#[derive(DataFrameRow)]`). The generated `IntoR::Error` is `Infallible`.

### `list_derive::derive_prefer_externalptr`

```rust
fn derive_prefer_externalptr(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Derive `PreferExternalPtr`: emits an `IntoR` impl that wraps the value in
`ExternalPtr::new` before converting to SEXP.

The type must implement `TypedExternal` (typically via `#[derive(ExternalPtr)]`).
The generated `IntoR::Error` is `Infallible`.

### `list_derive::derive_prefer_list`

```rust
fn derive_prefer_list(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Derive `PreferList`: emits an `IntoR` impl that converts to R by first calling
`IntoList::into_list`, then `into_sexp`.

The type must also derive `IntoList` for this to compile. The generated
`IntoR::Error` is `Infallible` (list conversion is infallible for valid structs).

### `list_derive::derive_prefer_rnative`

```rust
fn derive_prefer_rnative(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Derive `PreferRNativeType`: emits an `IntoR` impl that wraps the value in
`AsRNative(self)` before calling `IntoR::into_sexp`.

This routes conversion through native R vector allocation, bypassing list/ExternalPtr
paths. The type must also implement `RNativeType` for the `AsRNative` wrapper to compile.
The generated `IntoR::Error` is `Infallible`.

### `list_derive::derive_try_from_list`

```rust
fn derive_try_from_list(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Derive `TryFromList` for structs (R -> Rust).

Generates an `impl TryFromList for T` that extracts struct fields from an R list:
- Named structs: extract by field name from a named R list
- Tuple structs: extract by position (index 0, 1, 2, ...)
- Unit structs: accept any list (no extraction needed)

Fields marked with `#[into_list(ignore)]` are filled with `Default::default()`.
Each non-ignored field's type must implement `TryFromSexp` (enforced via where-clause bounds).

Returns `Err` if applied to a non-struct type or if an unknown field attribute is found.

### `list_macro::expand_list`

```rust
fn expand_list(input: ListInput) -> proc_macro2::TokenStream
```

Expands a parsed `list!` invocation into a `List` constructor token stream.

For fully unnamed lists, generates `List::from_raw_values(...)`.
For named or mixed lists, generates `List::from_raw_pairs(...)` where
unnamed entries receive empty-string names.

Each entry value is converted via `IntoR::into_sexp()`.

### `match_arg_derive::derive_match_arg`

```rust
fn derive_match_arg(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

Main entry point for `#[derive(MatchArg)]`.

Generates three trait implementations:
- `impl MatchArg` -- provides `CHOICES` (static string slice), `from_choice`, `to_choice`
- `impl TryFromSexp` -- converts R character scalar to enum variant via `match_arg_from_sexp`
- `impl IntoR` -- converts enum variant to R character scalar via `to_choice().into_sexp()`

`impl IntoR for Vec<Self>` is provided automatically by the blanket
`impl<T: MatchArg> IntoR for Vec<T>` in `miniextendr-api::match_arg`,
so returning `Vec<EnumName>` from a `#[miniextendr]` function works
without any extra code in the user's crate.

Validates:
- Only enums are accepted (not structs or unions)
- Generic enums are rejected
- At least one variant is required
- Only fieldless (C-style) variants are allowed
- No duplicate choice names after renaming

Choice names default to variant identifiers, optionally transformed by
`#[match_arg(rename_all = "...")]` or overridden per-variant with
`#[match_arg(rename = "...")]`.

### `method_return_builder::condition_check_inline_block`

```rust
fn condition_check_inline_block(call_expr: &str, inner: &str, indent: &str) -> String
```

Generate an inline R error-check block for single-expression contexts (S7, S4).

Returns a multi-line block string: `{ .val <- <call_expr>; if (...) return(...); <inner> }`.
Used where the class system requires a single expression rather than separate lines
(e.g., S7 property definitions, S4 method bodies).

- `call_expr`: The `.Call()` expression to evaluate
- `inner`: The final expression to return after the error check passes
- `indent`: Leading whitespace for the inner lines (e.g., `"    "` for 4-space)

### `method_return_builder::condition_check_lines`

```rust
fn condition_check_lines(indent: &str) -> Vec<String>
```

Generate the R guard that re-raises a tagged Rust error/condition value.

Expects `.val` to already be assigned (e.g., `.val <- .Call(...)`). Emits a
single line indented by `indent`: when `.val` is a tagged `rust_condition_value`,
hand off to the shared helper and return from the enclosing function.

The helper dispatches on `.val$kind` (see
`miniextendr_api::error_value::kind` for canonical kind strings):

- `error` / `panic` / `result_err` / `none_err` / `conversion` (and any
  unknown kind) — `stop()` longjmps with the appropriate `rust_*` class
  layering.
- `warning` — `warning()` signals; the wrapper's surrounding `return(...)`
  propagates `invisible(NULL)` as the wrapper's result.
- `message` — `message()` signals; same propagation.
- `condition` — `signalCondition()` signals; same propagation.

### `method_return_builder::standalone_body`

```rust
fn standalone_body(call_expr: &str, final_return: &str, indent: &str) -> String
```

Generate a standalone-function R wrapper body.

Returns the full body string: `.val <- <call_expr>; if (...) return(...); <final_return>`.
Used for top-level `#[miniextendr]` functions (not class methods).

- `call_expr`: The `.Call()` expression to evaluate
- `final_return`: The expression to return (typically `".val"` or `"invisible(.val)"`)
- `indent`: Leading whitespace for the body lines (e.g., `"  "` for 2-space)

### `miniextendr_impl::env_class::generate_env_r_wrapper`

```rust
fn generate_env_r_wrapper(parsed_impl: &super::ParsedImpl) -> String
```

Generates the complete R wrapper string for an environment-based class.

Produces an R environment object (`new.env(parent = emptyenv())`) that serves as a
class namespace, with methods attached as `ClassName$method_name`. This pattern
supports both inherent methods and trait namespace dispatch via `$`/`[[`.

The generated code includes:
- Class environment: `ClassName <- new.env(parent = emptyenv())`
- Constructor: `ClassName$new(...)` that calls the Rust `new` function, sets
  `class(self) <- "ClassName"`, and returns the ExternalPtr as `self`
- Instance methods: `ClassName$method(x = self, ...)` using default-arg binding
  so that `$` dispatch re-parents the environment to make `self` visible
- Static methods: `ClassName$method(...)` that call Rust directly
- `$.ClassName` S3 method: dispatches `obj$method(...)` by looking up the method
  in the class environment, binding `self` for instance methods, and supporting
  trait namespace environments (nested envs with `.__mx_instance__` attributes)
- `[[.ClassName` alias: delegates to `$.ClassName`

Roxygen2 documentation is generated for the class, each method, and the
dispatch methods, with appropriate `@export`/`@keywords internal`/`@noRd` tags.

### `miniextendr_impl::expand_impl`

```rust
fn expand_impl(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream
```

Top-level entry point for expanding `#[miniextendr]` on impl blocks.

Dispatches between two cases:
1. **Inherent impls** (`impl Type { ... }`): Parses [`ImplAttrs`] and [`ParsedImpl`],
   then generates C wrappers, R wrapper code, `R_CallMethodDef` arrays, and
   `as.<class>()` trait impls for the chosen class system.
2. **Trait impls** (`impl Trait for Type { ... }`): Generates trait ABI vtables,
   cross-package shims, and R wrappers via
   [`expand_miniextendr_impl_trait`](crate::miniextendr_impl_trait::expand_miniextendr_impl_trait).

#### Arguments

* `attr` - The token stream inside `#[miniextendr(...)]` (class system, options)
* `item` - The full `impl` block token stream

#### Returns

A token stream containing the original impl block (with miniextendr attrs stripped),
C wrapper functions, an R wrapper string constant, a `R_CallMethodDef` array constant,
and any forwarding trait impls for `as.<class>()` coercion.

### `miniextendr_impl::generate_as_coercion_methods`

```rust
fn generate_as_coercion_methods(parsed_impl: &ParsedImpl) -> String
```

Generate R S3 method wrappers for `as.<class>()` coercion methods.

For each method with `#[miniextendr(as = "...")]`, generates an S3 method like:

```r
#' @export
#' @method as.data.frame MyType
as.data.frame.MyType <- function(x, ...) {
    .Call(C_MyType__as_data_frame, .call = match.call(), x)
}
```

This function is called by each class system generator to append the
`as.*` methods to the R wrapper output.

### `miniextendr_impl::generate_as_coercion_trait_impls`

```rust
fn generate_as_coercion_trait_impls(parsed_impl: &ParsedImpl) -> proc_macro2::TokenStream
```

Generate `impl RCoerce*` trait impls for methods with `#[miniextendr(as = "...")]`.

For each `as` coercion method, generates a forwarding trait impl:
```ignore
impl ::miniextendr_api::r_coerce::RCoerceDataFrame for MyType {
    fn as_data_frame(&self) -> Result<::miniextendr_api::List, ::miniextendr_api::r_coerce::RCoerceError> {
        self.as_data_frame()  // inherent method preferred over trait method
    }
}
```

Skips methods with extra parameters beyond `&self` (trait methods have fixed signatures)
and skips non-standard targets (like "tibble", "data.table") that don't have corresponding traits.

### `miniextendr_impl::generate_method_c_wrapper`

```rust
fn generate_method_c_wrapper(parsed_impl: &ParsedImpl, method: &ParsedMethod, r_wrappers_const: &syn::Ident) -> proc_macro2::TokenStream
```

Generate a C-callable wrapper function for a single method in an impl block.

Produces a `#[no_mangle] extern "C"` function named `C_{crate}_{Type}__{method}` that:
1. Accepts SEXP arguments (including `self_sexp` for instance methods)
2. Extracts `&self` / `&mut self` from an `ErasedExternalPtr` for instance methods
3. Converts SEXP arguments to Rust types
4. Calls the actual Rust method
5. Converts the return value back to SEXP

Thread strategy is determined automatically: instance methods always run on the main
thread (because `self_ref` is a non-Send borrow), while static methods use the worker
thread unless `unsafe(main_thread)` is specified.

Also emits an `R_CallMethodDef` constant for R routine registration, and appends
generated R wrapper code fragments to the `r_wrappers_const` string constant.

#### Arguments

* `parsed_impl` - The parsed impl block providing type identity, cfg attrs, and options
* `method` - The parsed method to generate a wrapper for
* `r_wrappers_const` - Identifier of the const that accumulates R wrapper code fragments

### `miniextendr_impl::r6_class::generate_r6_r_wrapper`

```rust
fn generate_r6_r_wrapper(parsed_impl: &super::ParsedImpl) -> String
```

Generates the complete R wrapper string for an R6-style class.

Produces an `R6::R6Class(...)` core definition followed by per-method
`$set()` blocks (see the module docs for the full `$set`-form shape, #369):
- `initialize` method (inline in `public`): calls the Rust `new` constructor,
  or accepts a pre-made `.ptr` when static methods return `Self` (factory pattern)
- Public methods: one `ClassName$set("public", "name", function(...) {...})`
  block per `&self`/`&mut self` instance method, each with its own roxygen
- Private methods (inline in `private`): methods marked with `#[miniextendr(private)]`
- Active bindings: one `ClassName$set("active", "name", function(...) {...})`
  block per getter/setter property (`#[miniextendr(r6(prop = "..."))]`)
- Private `.ptr` field: holds the `ExternalPtr` to the Rust struct
- Finalizer (inline in `private`): optional destructor called when the R6 object is garbage-collected
- Deep clone (inline in `private`): optional custom clone logic via `#[miniextendr(r6(deep_clone))]`
- Static methods: emitted as `ClassName$method_name <- function(...)` outside the class
- Class options: `lock_objects`, `lock_class`, `cloneable`, `portable`, `inherit`

Also generates roxygen2 documentation blocks for the class, its methods,
and active bindings.

### `miniextendr_impl::s3_class::generate_s3_r_wrapper`

```rust
fn generate_s3_r_wrapper(parsed_impl: &super::ParsedImpl) -> String
```

Generates the complete R wrapper string for an S3-style class.

Produces the following R code:
- Constructor: `new_<class>(...)` function that calls the Rust `new` constructor
  and wraps the result with `structure(.val, class = "<class>")`
- S3 generics: for each instance method, a `UseMethod()` generic is created
  (unless overriding an existing generic via `#[miniextendr(generic = "...")]`)
- S3 methods: `<generic>.<class>` functions dispatching to the Rust `.Call()` wrapper,
  with the ExternalPtr extracted from `x`
- Static methods: regular functions named `<class>_<method>(...)`
- Class environment: `ClassName <- new.env(parent = emptyenv())` for `Class$new()`
  syntax and trait namespace compatibility

Custom double-dispatch patterns (e.g., `vec_ptype2.a.b`) are supported via
`#[miniextendr(generic = "...", class = "...")]` attributes.

### `miniextendr_impl::s4_class::generate_s4_r_wrapper`

```rust
fn generate_s4_r_wrapper(parsed_impl: &super::ParsedImpl) -> String
```

Generates the complete R wrapper string for an S4-style class.

Produces the following R code:
- Class definition: `methods::setClass("<class>", slots = c(ptr = "externalptr"))`
  with a single `ptr` slot holding the `ExternalPtr` to the Rust struct
- Constructor function: `ClassName(...)` that calls the Rust `new` constructor
  and wraps the result with `methods::new("<class>", ptr = .val)`
- S4 generics: `methods::setGeneric(...)` for each instance method, guarded by
  a namespace-local `exists()` check (see #1158 for why not `isGeneric()`)
- S4 methods: `methods::setMethod("<generic>", "<class>", function(x, ...) ...)`
  dispatching to the Rust `.Call()` wrapper, extracting the ptr via `x@ptr`
- Static methods: regular functions named `<class>_<method>(...)`

Roxygen2 `@exportMethod`, `@importFrom methods`, and `@slot` tags are generated
as appropriate.

### `miniextendr_impl::s7_class::generate_s7_r_wrapper`

```rust
fn generate_s7_r_wrapper(parsed_impl: &super::ParsedImpl) -> String
```

Generates the complete R wrapper string for an S7-style class.

Produces the following R code:
- Class definition: `ClassName <- S7::new_class("ClassName", ...)` with a `.ptr` property
  of `class_any` holding the `ExternalPtr`, plus optional computed properties
- Constructor: inline in `new_class(constructor = function(...) ...)`, supports
  `.ptr` shortcut parameter for wrapping pre-built ExternalPtr returns
- Properties: `S7::new_property(...)` for each getter/setter/validator annotated
  with `#[miniextendr(s7(getter))]` etc., with support for class constraints,
  defaults, required, frozen, and deprecated modifiers
- Instance methods: `S7::new_generic(...)` + `S7::method(generic, class)` pairs
  dispatching to Rust `.Call()` wrappers via `x@.ptr`
- External generics: `S7::new_external_generic("pkg", "name")` for overriding
  generics from other packages
- Multiple dispatch: via `#[miniextendr(s7(dispatch = "x,y"))]`
- Fallback methods: `S7::method(generic, S7::class_any)` with `tryCatch` for
  safe slot access on non-S7 objects
- Static methods: regular functions named `ClassName_method(...)`
- Convert methods: `S7::method(convert, list(From, To))` for `convert_from`
  and `convert_to` annotations
- S7 parent/abstract: optional `parent` and `abstract = TRUE` in class definition

Roxygen2 documentation and `@importFrom S7 ...` tags are generated automatically.

### `miniextendr_impl::vctrs_class::generate_vctrs_r_wrapper`

```rust
fn generate_vctrs_r_wrapper(parsed_impl: &super::ParsedImpl) -> String
```

Generates the complete R wrapper string for a vctrs-compatible S3 class.

This is used when an `impl` block is annotated with `#[miniextendr(vctrs)]`.
Unlike the `#[derive(Vctrs)]` macro (which generates standalone S3 methods from
struct attributes), this generator produces class wrappers from `impl` block methods.

Produces the following R code:
- Constructor: `new_<class>(...)` that calls the Rust `new` constructor, then wraps
  the result with `vctrs::new_vctr()`, `vctrs::new_rcrd()`, or `vctrs::new_list_of()`
  depending on the `VctrsKind`
- `vec_ptype_abbr.<class>`: compact abbreviation for printing (if `abbr` is specified)
- `vec_ptype2.<class>.<class>`: self-coercion prototype (returns empty typed vector)
- `vec_cast.<class>.<class>`: identity cast (returns `x` unchanged)
- Instance methods: S3 generics + `<generic>.<class>` methods, with support for
  vctrs protocol overrides via `#[miniextendr(vctrs_protocol = "...")]` and
  double-dispatch class suffixes via `#[miniextendr(class = "...")]`
- Static methods: regular functions named `<class>_<method>(...)`

Roxygen2 documentation and `@importFrom vctrs ...` tags are generated automatically.
A class-level `@noRd` (or a plain `noexport` without `internal`) suppresses the Rd
contribution of every block — self-coercion methods, instance-method generics, and
instance/static method docs all collapse to `@noRd` (#1180), like the other five
class-system generators. S3 `S3method()` dispatch registration (`@method` +
`@export`) is kept unconditionally: vctrs generics dispatch from the vctrs
namespace, so an unregistered method would leave even a gated class non-functional
(and roxygen2 warns on any recognized-but-unregistered S3 method).

### `miniextendr_impl_trait::expand_miniextendr_impl_trait`

```rust
fn expand_miniextendr_impl_trait(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream
```

Expand `#[miniextendr]` applied to a trait implementation.

#### Arguments

* `attr` - Attribute arguments (currently unused)
* `item` - The impl block token stream

#### Returns

Expanded token stream containing:
- Original impl block
- Vtable static constant

#### Errors

Returns a compile error if:
- Not applied to a trait impl (`impl Trait for Type`)
- Applied to an inherent impl (`impl Type`)

### `miniextendr_impl_trait::expand_tpie`

```rust
fn expand_tpie(input: proc_macro::TokenStream) -> proc_macro::TokenStream
```

Entry point for the `__mx_trait_impl_expand!` proc macro.

Parses TPIE metadata tokens and generates C wrappers, R wrappers,
and call defs — the same outputs as a manual trait impl with method bodies.

### `miniextendr_trait::expand_trait`

```rust
fn expand_trait(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream
```

Expand `#[miniextendr]` applied to a trait definition.

#### Arguments

* `attr` - Attribute arguments (currently unused, reserved for future options)
* `item` - The trait definition token stream

#### Returns

Expanded token stream containing:
- Original trait definition
- Type tag constant
- Vtable struct
- View struct
- Method shims
- Vtable builder function

#### Errors

Returns a compile error if:
- Methods have unsupported signatures
- Methods are async

### `newtype_derive::derive_into_r`

```rust
fn derive_into_r(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

`#[derive(IntoR)]`: scalar forwarding `IntoR` + `IntoRNewtype` marker (for
`Option`/`Vec<Option>` blankets) + concrete `IntoRVecElement` (for `Vec`).

### `newtype_derive::derive_try_from_sexp`

```rust
fn derive_try_from_sexp(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>
```

`#[derive(TryFromSexp)]`: scalar forwarding `TryFromSexp` + `FromRNewtype` marker.

### `r_preconditions::build_precondition_checks`

```rust
fn build_precondition_checks(inputs: &syn::punctuated::Punctuated<syn::FnArg, $crate::token::Comma>, skip_params: &std::collections::HashSet<String>, opts: &PreconditionOptions) -> PreconditionOutput
```

Build precondition checks for a function's parameters.

Returns:
- **`static_checks`**: Lines forming a `stopifnot(...)` call for known types
- **`fallback_params`**: Parameters needing validation (unknown custom types)

Static checks produce R-side `stopifnot()`:
```r
stopifnot(
  "'a' must be numeric, logical, or raw" = is.numeric(a) || is.logical(a) || is.raw(a),
  "'a' must have length 1" = length(a) == 1L
)
```

Skips:
- `self`/`&self`/`&mut self` (receiver args)
- Parameters in `skip_params` (e.g., match_arg params already validated)
- Skip types (SEXP, Dots, ExternalPtr, etc.)

### `r_wrapper_builder::normalize_r_arg_ident`

```rust
fn normalize_r_arg_ident(rust_ident: &syn::Ident) -> syn::Ident
```

Normalizes Rust argument identifiers for R.

- Leading `_` → stripped (Rust convention for unused params)
- Leading `__` → stripped
- Otherwise → unchanged

#### Examples
- `_x` → `x`
- `_to` → `to`
- `__field` → `field`
- `value` → `value`

Note: We strip underscores rather than prefixing "unused" because R callers
(like vctrs) may use named arguments that must match the original name.

### `r_wrapper_builder::normalize_r_arg_string`

```rust
fn normalize_r_arg_string(name: &str) -> String
```

String form of [`normalize_r_arg_ident`] that skips the `syn::Ident` round-trip.

Most callers feed the result into `format!`/`HashMap` keys and immediately
`.to_string()` the returned ident — this avoids that allocation pair.

### `struct_enum_dispatch::expand_struct_or_enum`

```rust
fn expand_struct_or_enum(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream
```

Main dispatch entry point for `#[miniextendr]` on a struct or enum.

Attempts to parse the item as a struct first, then as an enum.
Dispatches to the appropriate derive path based on the parsed attributes
and item shape (field count, variant structure).

Returns the original item plus any generated trait implementations as a combined token stream.

### `typed_dataframe::expand_typed_dataframe`

```rust
fn expand_typed_dataframe(input: TypedDataframeInput) -> proc_macro2::TokenStream
```

Expand a parsed `typed_dataframe!` into the generated struct, impls, and
per-column accessors.

### `typed_external_macro::impl_typed_external`

```rust
fn impl_typed_external(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream>
```

Implementation of `impl_typed_external!`.

Accepts a concrete type path with generic arguments filled in
(e.g., `MyWrapper<i32>`, `TreeNode<String, Vec<u8>>`).

Returns a `TokenStream` containing:
- `impl TypedExternal for <type> { ... }`
- `impl IntoExternalPtr for <type> {}`

### `typed_list::expand_typed_list`

```rust
fn expand_typed_list(input: TypedListInput) -> proc_macro2::TokenStream
```

Expands a parsed `typed_list!` invocation into a `TypedListSpec` token stream.

Converts each [`ParsedEntry`] into a `TypedEntry` constructor call and wraps
the result in a `TypedListSpec { entries, allow_extra }` literal.

Returns a `TokenStream` that evaluates to `TypedListSpec` at runtime.

---

## Macros

### `#[derive(Altrep)]`

Helper attributes: `altrep`

Derive ALTREP registration for a data struct.

Generates `TypedExternal`, `AltrepClass`, `RegisterAltrep`, `IntoR`,
linkme registration entry, and `Ref`/`Mut` accessor types.

The struct must already have low-level ALTREP traits implemented.
For most use cases, prefer a family-specific derive:
`#[derive(AltrepInteger)]`, `#[derive(AltrepReal)]`, etc.
Use `#[altrep(manual)]` on a family derive to skip data trait generation
when you provide your own `AltrepLen` + `Alt*Data` impls.

#### Attributes

- `#[altrep(class = "Name")]` — custom ALTREP class name (defaults to struct name)

#### Example

```ignore
// Prefer family derives with manual:
#[derive(AltrepInteger)]
#[altrep(manual, class = "MyCustom", serialize)]
struct MyData { ... }

impl AltrepLen for MyData { ... }
impl AltIntegerData for MyData { ... }
```

### `#[derive(AltrepComplex)]`

Helper attributes: `altrep`

Derive macro for ALTREP complex vector data types.

Auto-implements `AltrepLen` and `AltComplexData` traits.
Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).

### `#[derive(AltrepInteger)]`

Helper attributes: `altrep`

Derive macro for ALTREP integer vector data types.

Auto-implements `AltrepLen`, `AltIntegerData`, and the low-level ALTREP
trait impls (`Altrep`, `AltVec`, `AltInteger`, `InferBase`).

#### Attributes

- `#[altrep(len = "field_name")]` - Specify length field (auto-detects "len" or "length")
- `#[altrep(elt = "field_name")]` - For constant vectors, specify which field provides elements
- `#[altrep(dataptr)]` - Enable direct data-pointer access
- `#[altrep(serialize)]` - Enable ALTREP serialization support
- `#[altrep(subset)]` - Enable `Extract_subset` optimization
- `#[altrep(no_lowlevel)]` - Skip the automatic low-level trait impls

#### Example (Constant Vector - Zero Boilerplate!)

```ignore
#[derive(ExternalPtr, AltrepInteger)]
#[altrep(elt = "value")]  // All elements return this field
pub struct ConstantIntData {
    value: i32,
    len: usize,
}

// That's it! 3 lines instead of 30!
// AltrepLen, AltIntegerData, and low-level impls are auto-generated

#[miniextendr(class = "ConstantInt")]
pub struct ConstantIntClass(pub ConstantIntData);
```

#### Example (Custom elt() - Override One Method)

```ignore
#[derive(ExternalPtr, AltrepInteger)]
pub struct ArithSeqData {
    start: i32,
    step: i32,
    len: usize,
}

// Auto-generates AltrepLen and stub AltIntegerData
// Just override elt() for custom logic:
impl AltIntegerData for ArithSeqData {
    fn elt(&self, i: usize) -> i32 {
        self.start + (i as i32) * self.step
    }
}
```

### `#[derive(AltrepList)]`

Helper attributes: `altrep`

Derive macro for ALTREP list vector data types.

Auto-implements `AltrepLen` and `AltListData` traits.
Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger),
except `dataptr` and `subset` which are not supported for list ALTREP.

### `#[derive(AltrepLogical)]`

Helper attributes: `altrep`

Derive macro for ALTREP logical vector data types.

Auto-implements `AltrepLen` and `AltLogicalData` traits.
Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).

### `#[derive(AltrepRaw)]`

Helper attributes: `altrep`

Derive macro for ALTREP raw vector data types.

Auto-implements `AltrepLen` and `AltRawData` traits.
Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).

### `#[derive(AltrepReal)]`

Helper attributes: `altrep`

Derive macro for ALTREP real vector data types.

Auto-implements `AltrepLen` and `AltRealData` traits.
Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).

### `#[derive(AltrepString)]`

Helper attributes: `altrep`

Derive macro for ALTREP string vector data types.

Auto-implements `AltrepLen` and `AltStringData` traits.
Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).

### `#[derive(DataFrameRow)]`

Helper attributes: `dataframe`

Derive `DataFrameRow`: generates a companion `*DataFrame` type with collection fields,
plus `IntoR` / `TryFromSexp` / `IntoDataFrame` impls for seamless R data.frame conversion.

#### Example

```ignore
#[derive(DataFrameRow)]
struct Measurement {
    time: f64,
    value: f64,
}

// Generates MeasurementDataFrame { time: Vec<f64>, value: Vec<f64> }
// plus conversion impls
```

#### Struct-level attributes

- `#[dataframe(name = "CustomDf")]` — custom name for the generated DataFrame type
- `#[dataframe(align)]` — pad shorter columns with NA to match longest
- `#[dataframe(tag = "my_tag")]` — attach a tag attribute to the data.frame
- `#[dataframe(conflicts = "string")]` — resolve conflicting column types as strings

#### Field-level attributes

- `#[dataframe(skip)]` — omit this field from the DataFrame
- `#[dataframe(rename = "col")]` — custom column name
- `#[dataframe(as_list)]` — keep collection as single list column (no expansion)
- `#[dataframe(expand)]` / `#[dataframe(unnest)]` — expand collection into suffixed columns
- `#[dataframe(width = N)]` — pin expansion width (shorter rows get NA)

#### Public surface (which verbs to call)

Every capability the derive provides has a documented, trait-based (or `std`)
verb — reach for these, not any incidental inherent plumbing:

- **Rows → R `data.frame`**: `rows.into_dataframe()?` (owned, GC-rooted
  `BuiltDataFrame`) or `rows.wrap_data_frame()` (deferred `IntoR` wrapper);
  parallel variant `rows.into_dataframe_par()?`. From the `IntoDataFrame` /
  `AsDataFrameExt` traits (both re-exported from `miniextendr_api::prelude`).
- **R `data.frame` → rows**: `Vec::<Row>::from_dataframe(&df)?` (parallel:
  `Vec::<Row>::from_dataframe_par(&df)?`), from the `FromDataFrame` trait — or
  the one-call `Row::try_from_dataframe(sexp)` reader on the row type.
- **Rows ↔ the pure-Rust columnar companion** (`<Row>DataFrame`, `Vec`-columns,
  no R involved): the `ColumnarFrame` trait (in the prelude) —
  `<Row>DataFrame::from_rows(rows)` / `from_rows_par(rows)` (parallel build of
  the *companion*, which `into_dataframe_par` does not give you) and, for
  row-iterable companions, `companion.into_rows()`. `Vec<Row>: Into<companion>`
  and the companion's `IntoIterator` are the equivalent `std` verbs.
- **Enum split representation**: `rows.into_dataframe_split()` returns one
  `data.frame` per variant as an R list (only that variant's columns — no NA
  fill), from the `IntoDataFrameSplit` trait (in the prelude). Enum rows
  only; struct derives don't partition.

The generated `<Row>DataFrame` / `<Row>DataFrameIter` types are intermediate
column-oriented companions; you rarely name them directly.

### `#[derive(ExternalPtr)]`

Helper attributes: `externalptr, r_data`

Derive macro for implementing `TypedExternal` on a type.

This makes the type compatible with `ExternalPtr<T>` for storing in R's external pointers.

#### Basic Usage

```ignore
use miniextendr_api::TypedExternal;

#[derive(ExternalPtr)]
struct MyData {
    value: i32,
}

// Now you can use ExternalPtr<MyData>
let ptr = ExternalPtr::new(MyData { value: 42 });
```

#### Trait ABI

Trait dispatch wrappers are automatically generated:

```ignore
use miniextendr_api::miniextendr;

#[derive(ExternalPtr)]
struct MyCounter {
    value: i32,
}

#[miniextendr]
impl Counter for MyCounter {
    fn value(&self) -> i32 { self.value }
    fn increment(&mut self) { self.value += 1; }
}
```

This generates additional infrastructure for type-erased trait dispatch:
- `__MxWrapperMyCounter` - Type-erased wrapper struct
- `__MX_BASE_VTABLE_MYCOUNTER` - Base vtable with drop/query
- `__mx_wrap_mycounter()` - Constructor returning `*mut mx_erased`

#### Generated Code (Basic)

For a type `MyData` without traits:

```ignore
impl TypedExternal for MyData {
    const TYPE_NAME: &'static str = "MyData";
    const TYPE_NAME_CSTR: &'static [u8] = b"MyData\0";
}
```

### `#[derive(IntoList)]`

Helper attributes: `into_list`

Derive `IntoList` for a struct (Rust → R list).

- Named structs → named R list: `list(x = 1L, y = 2L)`
- Tuple structs → unnamed R list: `list(1L, 2L)`
- Fields annotated `#[into_list(ignore)]` are skipped

### `#[derive(IntoR)]`

Derive `IntoR` for a single-field newtype: forward the Rust → R conversion to
the inner type.

Generates a scalar `IntoR` impl that delegates to the inner type, plus an
`IntoRNewtype` marker (powering the `Option<T>` / `Vec<Option<T>>` container
blankets) and a concrete `IntoRVecElement` impl (powering `Vec<T>`). See
`#[derive(TryFromSexp)]` for usage.

Do not derive both `IntoR` and `MatchArg` on the same type: both feed the
single `IntoR for Vec<T>` blanket slot and would collide (E0119).

### `#[derive(MatchArg)]`

Helper attributes: `match_arg`

Derive `MatchArg`: enables conversion between Rust enums and R character strings
with `match.arg` semantics (partial matching, informative errors).

#### Usage

```ignore
#[derive(Copy, Clone, MatchArg)]
enum Mode {
    Fast,
    Safe,
    Debug,
}
```

#### Attributes

- `#[match_arg(rename = "name")]` - Rename a variant's choice string
- `#[match_arg(rename_all = "snake_case")]` - Rename all variants (snake_case, kebab-case, lower, upper)

#### Generated Implementations

- `MatchArg` - Choice metadata and bidirectional conversion
- `TryFromSexp` - Convert R STRSXP/factor to enum (with partial matching)
- `IntoR` - Convert enum to R character scalar

### `#[derive(PreferDataFrame)]`

Derive `PreferDataFrame`: when a type implements both `IntoDataFrame` (via `DataFrameRow`)
and other conversion paths, this selects data.frame as the default `IntoR` conversion.

#### Example

```ignore
#[derive(DataFrameRow, PreferDataFrame)]
struct Obs { time: f64, value: f64 }
// IntoR produces data.frame(time = ..., value = ...)
```

### `#[derive(PreferExternalPtr)]`

Derive `PreferExternalPtr`: when a type implements both `ExternalPtr` and
other conversion paths (e.g., `IntoList`), this selects `ExternalPtr` wrapping
as the default `IntoR` conversion.

#### Example

```ignore
#[derive(ExternalPtr, IntoList, PreferExternalPtr)]
struct Model { weights: Vec<f64> }
// IntoR wraps as ExternalPtr (opaque R object), not list
```

### `#[derive(PreferList)]`

Derive `PreferList`: emits an `IntoR` impl selecting list as the type's default
Rust→R conversion (via `IntoList::into_list`).

A type carries exactly one representation default: stacking two `Prefer*`
derives is a compile error. Each `Prefer*` derive emits a fixed-name marker
const, so a second one triggers a guided `duplicate definitions with name
__miniextendr_conflicting_Prefer_derives__keep_ONE_or_use_call_site_As_wrappers`
error (alongside the raw conflicting-`IntoR`-impl error) — keep one `Prefer*`,
or drop them all and choose a representation per return value at the call site
with an `As*` wrapper (`AsList`, `AsExternalPtr`, `AsDataFrame`, ...).

#### Example

```ignore
#[derive(IntoList, PreferList)]
struct Config { verbose: bool, threads: i32 }
// IntoR produces list(verbose = TRUE, threads = 4L)
```

### `#[derive(PreferRNativeType)]`

Derive `PreferRNativeType`: when a newtype wraps an `RNativeType` and also
implements other conversions, this selects the native R vector conversion
as the default `IntoR` path.

#### Example

```ignore
#[derive(Copy, Clone, RNativeType, PreferRNativeType)]
struct Meters(f64);
// IntoR produces a numeric scalar, not an ExternalPtr
```

### `#[derive(RFactor)]`

Helper attributes: `r_factor`

Derive `RFactor`: enables conversion between Rust enums and R factors.

#### Usage

```ignore
#[derive(Copy, Clone, RFactor)]
enum Color {
    Red,
    Green,
    Blue,
}
```

#### Attributes

- `#[r_factor(rename = "name")]` - Rename a variant's level string
- `#[r_factor(rename_all = "snake_case")]` - Rename all variants (snake_case, kebab-case, lower, upper)

### `#[derive(RNativeType)]`

Derive macro for implementing `RNativeType` on a newtype wrapper.

This allows newtype wrappers around R native types to work with `Vec<T>`,
`&[T]` conversions and the `Coerce<R>` traits.
The inner type must implement `RNativeType`.

#### Supported Struct Forms

Both tuple structs and single-field named structs are supported:

```ignore
use miniextendr_api::RNativeType;

// Tuple struct (most common)
#[derive(Clone, Copy, RNativeType)]
struct UserId(i32);

// Named single-field struct
#[derive(Clone, Copy, RNativeType)]
struct Temperature { celsius: f64 }
```

#### Generated Code

For `struct UserId(i32)`, this generates:

```ignore
impl RNativeType for UserId {
    const SEXP_TYPE: SEXPTYPE = <i32 as RNativeType>::SEXP_TYPE;
    const R_NA: Self = UserId(<i32 as RNativeType>::R_NA);

    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        <i32 as RNativeType>::dataptr_mut(sexp).cast()
    }
}
```

#### Using the Newtype with Coerce

Once `RNativeType` is derived, you can implement `Coerce` to/from the newtype:

```ignore
impl Coerce<UserId> for i32 {
    fn coerce(self) -> UserId { UserId(self) }
}

let id: UserId = 42.coerce();
```

#### Requirements

- Must be a newtype struct (exactly one field, tuple or named)
- The inner type must implement `RNativeType` (`i32`, `f64`, `RLogical`, `u8`, `Rcomplex`)
- Should also derive `Copy` (required by `RNativeType: Copy`)

### `#[derive(TryFromList)]`

Helper attributes: `into_list`

Derive `TryFromList` for a struct (R list → Rust).

- Named structs: extract by field name
- Tuple structs: extract by position (0, 1, 2, ...)
- Fields annotated `#[into_list(ignore)]` are not read and are initialized with `Default::default()`

### `#[derive(TryFromSexp)]`

Derive `TryFromSexp` for a single-field newtype: forward the R → Rust
conversion to the inner type.

Generates a scalar `TryFromSexp` impl that delegates to the inner type (so the
newtype inherits its exact SEXPTYPE checks, NA policy, and error text), plus a
`FromRNewtype` marker impl. The marker lets `miniextendr-api`'s container
blankets light up `Vec<T>` / `Option<T>` / `Vec<Option<T>>` automatically.

#### Usage

```ignore
use uuid::Uuid;

#[derive(TryFromSexp)]            // R -> Rust only
struct Pattern(regex::Regex);

#[derive(TryFromSexp, IntoR)]     // round-trip; Vec/Option containers work too
struct UserId(Uuid);
```

Direction is chosen by which derive you list — derive only `TryFromSexp` for
inner types that read from R but cannot be written back (e.g. `regex::Regex`).

### `impl_typed_external!`

Generate `TypedExternal` and `IntoExternalPtr` impls for a concrete monomorphization
of a generic type.

Since `#[derive(ExternalPtr)]` rejects generic types, use this macro to generate
the necessary impls for a specific type instantiation.

#### Example

```ignore
struct Wrapper<T> { inner: T }

impl_typed_external!(Wrapper<i32>);
impl_typed_external!(Wrapper<String>);
```

### `list!`

Construct an R list from Rust values.

This macro provides a convenient way to create R lists in Rust code,
using R-like syntax. Values are converted to R objects via the [`IntoR`] trait.

#### Syntax

```ignore
// Named entries (like R's list())
list!(
    alpha = 1,
    beta = "hello",
    "my-name" = vec![1, 2, 3],
)

// Unnamed entries
list!(1, "hello", vec![1, 2, 3])

// Mixed (unnamed entries get empty string names)
list!(alpha = 1, 2, beta = "hello")

// Empty list
list!()
```

#### Examples

```ignore
use miniextendr_api::{list, IntoR};

// Create a named list
let my_list = list!(
    x = 42,
    y = "hello world",
    z = vec![1.0, 2.0, 3.0],
);

// In R this is equivalent to:
// list(x = 42L, y = "hello world", z = c(1, 2, 3))
```

[`IntoR`]: https://docs.rs/miniextendr-api/latest/miniextendr_api/into_r/trait.IntoR.html

### `#[miniextendr]`

Export Rust items to R.

`#[miniextendr]` can be applied to:
- `fn` items (generate C + R wrappers)
- `impl` blocks (generate R class methods)
- `trait` items (generate trait ABI metadata)
- ALTREP wrapper structs (generate `RegisterAltrep` impls)

#### Functions

```ignore
use miniextendr_api::miniextendr;

#[miniextendr]
fn add(a: i32, b: i32) -> i32 { a + b }
```

This produces a C wrapper `C_<crate>_add` and an R wrapper `add()`.
Registration is automatic via linkme distributed slices.

##### `extern "C-unwind"`

If the function is declared `extern "C-unwind"` and exported with
`#[no_mangle]` (2021), `#[unsafe(no_mangle)]` (2024), or `#[export_name = "..."]`,
the function itself is the C symbol and the R wrapper is prefixed with
`unsafe_` to signal bypassed safety (no worker isolation or conversion).

##### Variadics (`...`)

Use `...` as the last argument. The Rust parameter becomes `_dots: &Dots`.
Use `name: ...` to give it a custom name (e.g., `args: ...` → `args: &Dots`).

###### Typed Dots Validation

Use `#[miniextendr(dots = typed_list!(...))]` to automatically validate dots
and create a `dots_typed` variable with typed accessors:

```ignore
#[miniextendr(dots = typed_list!(x => numeric(), y => integer(), z? => character()))]
pub fn my_func(...) -> String {
    let x: f64 = dots_typed.get("x").expect("x");
    let y: i32 = dots_typed.get("y").expect("y");
    let z: Option<String> = dots_typed.get_opt("z").expect("z");
    format!("x={}, y={}", x, y)
}
```

Type specs: `numeric()`, `integer()`, `logical()`, `character()`, `list()`,
`raw()`, `complex()`, or `"class_name"` for class inheritance checks.
Add `(n)` for exact length: `numeric(4)`. Use `?` suffix for optional fields.
Use `@exact;` prefix for strict mode (reject extra fields).

##### Attributes

- `#[miniextendr(worker)]` — opt into worker-thread execution
- `#[miniextendr(invisible)]` / `#[miniextendr(visible)]` — control return visibility
- `#[miniextendr(check_interrupt)]` — check for user interrupt after call
- `#[miniextendr(coerce)]` — coerce R type before conversion (also usable per-parameter)
- `#[miniextendr(strict)]` — reject lossy conversions for i64/u64/isize/usize
- `#[miniextendr(unwrap_in_r)]` — return `Result<T, E>` to R without unwrapping
- `#[miniextendr(dots = typed_list!(...))]` — validate dots, create `dots_typed`
- `#[miniextendr(internal)]` — adds `@keywords internal` to R wrapper
- `#[miniextendr(noexport)]` — suppresses `@export` from R wrapper

#### Impl blocks (class systems)

Apply `#[miniextendr(env|r6|s7|s3|s4)]` to an `impl Type` block.
Use `#[miniextendr(label = "...")]` to disambiguate multiple impl blocks
on the same type.
Registration is automatic.

##### R6 Active Bindings

For R6 classes, use `#[miniextendr(r6(active))]` on methods to create
active bindings (computed properties accessed without parentheses):

```ignore
use miniextendr_api::miniextendr;

pub struct Rectangle {
    width: f64,
    height: f64,
}

#[miniextendr(r6)]
impl Rectangle {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// Returns the area (width * height).
    #[miniextendr(r6(active))]
    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    /// Regular method (requires parentheses).
    pub fn scale(&mut self, factor: f64) {
        self.width *= factor;
        self.height *= factor;
    }
}
```

In R:
```r
r <- Rectangle$new(3, 4)
r$area        # 12 (active binding - no parentheses!)
r$scale(2)    # Regular method call
r$area        # 24
```

Active bindings must be getter-only methods taking only `&self`.

##### S7 Properties

For S7 classes, use `#[miniextendr(s7(getter))]` and `#[miniextendr(s7(setter))]`
to create computed properties accessed via `@`:

```ignore
use miniextendr_api::{miniextendr, ExternalPtr};

#[derive(ExternalPtr)]
pub struct Range {
    start: f64,
    end: f64,
}

#[miniextendr(s7)]
impl Range {
    pub fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }

    /// Computed property (read-only): length of the range.
    #[miniextendr(s7(getter))]
    pub fn length(&self) -> f64 {
        self.end - self.start
    }

    /// Dynamic property getter.
    #[miniextendr(s7(getter, prop = "midpoint"))]
    pub fn get_midpoint(&self) -> f64 {
        (self.start + self.end) / 2.0
    }

    /// Dynamic property setter.
    #[miniextendr(s7(setter, prop = "midpoint"))]
    pub fn set_midpoint(&mut self, value: f64) {
        let half = self.length() / 2.0;
        self.start = value - half;
        self.end = value + half;
    }
}
```

In R:
```r
r <- Range(0, 10)
r@length     # 10 (computed, read-only)
r@midpoint   # 5 (dynamic property)
r@midpoint <- 20  # Adjusts start/end to center at 20
```

###### Property Attributes

- `#[miniextendr(s7(getter))]` - Read-only computed property
- `#[miniextendr(s7(getter, prop = "name"))]` - Named property getter
- `#[miniextendr(s7(setter, prop = "name"))]` - Named property setter
- `#[miniextendr(s7(getter, default = "0.0"))]` - Property with default value
- `#[miniextendr(s7(getter, required))]` - Required property (error if not provided)
- `#[miniextendr(s7(getter, frozen))]` - Property that can only be set once
- `#[miniextendr(s7(getter, deprecated = "Use X instead"))]` - Deprecated property
- `#[miniextendr(s7(validate))]` - Validator function for property

##### S7 Generic Dispatch Control

Control how S7 generics are created:

- `#[miniextendr(s7(no_dots))]` - Create strict generic without `...`
- `#[miniextendr(s7(dispatch = "x,y"))]` - Multi-dispatch on multiple arguments
- `#[miniextendr(s7(fallback))]` - Register method for `class_any` (catch-all).
  The generated R wrapper uses `tryCatch(x@.ptr, error = function(e) x)` to
  safely extract the self argument, so non-miniextendr objects won't crash with
  a slot-access error. Instead, incompatible objects produce a Rust type-conversion
  error when the method tries to interpret the argument as `&Self`.

```ignore
#[miniextendr(s7)]
impl MyClass {
    /// Strict generic: function(x) instead of function(x, ...)
    #[miniextendr(s7(no_dots))]
    pub fn strict_method(&self) -> i32 { 42 }

    /// Fallback method dispatched on class_any.
    /// Calling this on a non-MyClass object produces a type-conversion error,
    /// not a slot-access crash.
    #[miniextendr(s7(fallback))]
    pub fn describe(&self) -> String { "generic description".into() }
}
```

##### S7 Type Conversion (`convert`)

Use `convert_from` and `convert_to` to enable S7's `convert()` for type coercion:

```ignore
use miniextendr_api::{miniextendr, ExternalPtr};

#[derive(ExternalPtr)]
pub struct Celsius { value: f64 }

#[derive(ExternalPtr)]
pub struct Fahrenheit { value: f64 }

#[miniextendr(s7)]
impl Fahrenheit {
    pub fn new(value: f64) -> Self { Self { value } }

    /// Convert FROM Celsius TO Fahrenheit.
    /// Usage: S7::convert(celsius_obj, Fahrenheit)
    #[miniextendr(s7(convert_from = "Celsius"))]
    pub fn from_celsius(c: ExternalPtr<Celsius>) -> Self {
        Fahrenheit { value: c.value * 9.0 / 5.0 + 32.0 }
    }

    /// Convert FROM Fahrenheit TO Celsius.
    /// Usage: S7::convert(fahrenheit_obj, Celsius)
    #[miniextendr(s7(convert_to = "Celsius"))]
    pub fn to_celsius(&self) -> Celsius {
        Celsius { value: (self.value - 32.0) * 5.0 / 9.0 }
    }
}
```

In R:
```r
c <- Celsius(100)
f <- S7::convert(c, Fahrenheit)  # Uses convert_from
c2 <- S7::convert(f, Celsius)    # Uses convert_to
```

**Note:** Classes must be defined before they can be referenced in convert methods.
Define the "from" class before the "to" class to avoid forward reference issues.

#### Traits (ABI)

Apply `#[miniextendr]` to a trait to generate ABI metadata, then use
`#[miniextendr] impl Trait for Type`. Registration is automatic.

#### ALTREP

`#[miniextendr]` no longer generates ALTREP classes — `class`/`base`
attributes on a one-field struct are a compile error. Use the per-family
derives (`#[derive(AltrepInteger)]`, …) with `#[altrep(class = "...")]`
instead; registration is automatic there.

### `miniextendr_init!`

Generate the `R_init_*` entry point for a miniextendr R package.

This macro consolidates all package initialization into a single line.
It generates an `extern "C-unwind"` function that R calls when loading
the shared library.

#### Usage

```ignore
// Auto-detects package name from CARGO_CRATE_NAME (recommended):
miniextendr_api::miniextendr_init!();

// Or specify explicitly (for edge cases):
miniextendr_api::miniextendr_init!(mypkg);
```

The generated function calls `miniextendr_api::init::package_init` which
handles panic hooks, runtime init, locale assertion, ALTREP setup, trait ABI
registration, routine registration, and symbol locking.

### `r!`

Evaluate R code written as **Rust tokens**, validated at compile time.

`r!` takes a single R expression as a token stream, `stringify!`s it into a
static R source string at build time, and evaluates it via
`miniextendr_api::expression::r_eval_str` (the same protect-safe parse + eval
path as `r_str!`).

#### What you get today

Because the argument is a Rust token tree, the Rust front-end already
rejects **unbalanced delimiters** (`r!(f(1, 2)` won't compile) and
lexically invalid tokens before R ever sees the string — a cheap
compile-time guard over the pure-runtime `r_str!`. The source is lowered to
a `&'static str` (`stringify!`), so there is no `format!` allocation at the
call site.

This proc-macro additionally validates a conservative subset of known-bad
R syntax constructs (trailing binary operators, consecutive non-unary
binary operators, bare `if`/`while`/`for` without a body, etc.) and emits
a precise compile error pointing at the offending token. Empty (missing)
call arguments — `f(, x)`, `matrix(, 2, 2)` — are valid R and pass.

#### What is deferred

Direct `Rf_lang*` call-tree lowering (skipping the runtime parser entirely)
is tracked as a follow-up in issue #938 (item 2). Until then `r!` parses
its static string at first evaluation, exactly like `r_str!`.

#### Non-goals

A complete R grammar validator is not achievable over Rust tokens:
- Single-quoted strings (`'hello'`) and backtick-quoted names (`` `foo` ``)
  already die at the Rust lexer — nothing to validate.
- `%op%` tokenises as `%`, ident, `%` and is accepted without analysis.
- Anything the validator cannot confidently classify as wrong passes through
  unvalidated (conservative reject-only-known-bad design).

#### Forms

- `r!(R tokens…)` — evaluate in `R_GlobalEnv`.
- `r!(env: e; R tokens…)` — evaluate in the environment SEXP `e`. The
  leading `env: <expr> ;` is consumed as Rust, the rest is R source.

Both evaluate to `Result<SEXP, String>`; the `SEXP` is **unprotected**.

#### Safety

Expands to an `unsafe` block; the underlying FFI is `#[r_ffi_checked]`, so
calls from a worker thread are serialized onto the R thread.

#### Example

```ignore
let three = r!(1L + 2L)?;
let rows = r!(getFromNamespace(".theoph_rows", "dataframeflows")())?;
let in_env = r!(env: my_env; x + 1)?;
```

### `#[r_ffi_checked]`

Generate thread-safe wrappers for R FFI functions.

Apply this to an `extern "C-unwind"` block to generate, **for each
non-variadic function**, a pair of entry points:

- The original name (e.g. `Rf_allocVector`) — a safe Rust wrapper that
  runs directly on R's main thread, routes through
  `miniextendr_api::worker::with_r_thread` from an active miniextendr
  worker context, and panics for arbitrary off-main callers.
- A `*_unchecked` sibling (`Rf_allocVector_unchecked`) — the raw
  `extern "C-unwind"` declaration with no main-thread assertion and no
  worker round-trip.

User code should reach for the checked variant by default; the unchecked
sibling exists for three known-safe contexts:

1. **Inside ALTREP callbacks** — R is already calling us on the main
   thread, so the assertion would always pass and the route would
   deadlock the call back to R.
2. **Inside a `with_r_unwind_protect` body** — the guard has established
   main-thread context, and re-entering `with_r_thread` would nest two
   `R_UnwindProtect` frames (paying the longjmp-leak cost twice).
3. **Inside a `with_r_thread` body** — the assertion is redundant; you
   are already where you needed to be.

The build-time lint **MXL301** enforces this: calling `*_unchecked`
outside one of those three contexts is a compile-time error. Without the
`worker-thread` feature, the checked variant still enforces the recorded
main-thread contract; it simply has no worker route available.

#### Tradeoffs at a glance

| Variant | Asserts main thread | Routes to main | When to use |
|---|---|---|---|
| `Rf_foo` (checked) | yes (debug) | yes (from worker) | default |
| `Rf_foo_unchecked` | no | no | ALTREP callbacks, `with_r_unwind_protect`, `with_r_thread` |

#### Behavior

All non-variadic functions are routed to the main thread via `with_r_thread`
when called from a worker thread. The return value is wrapped in `Sendable`
and sent back to the caller. This applies to both value-returning functions
(SEXP, i32, etc.) and pointer-returning functions (`*const T`, `*mut T`).

Pointer-returning functions (like `INTEGER`, `REAL`) are safe to route because
the underlying SEXP must be GC-protected by the caller, and R's GC only runs
during R API calls which are serialized through `with_r_thread`.

#### Initialization Requirement

`miniextendr_runtime_init()` must be called before using any wrapped function.
Calling before initialization will panic with a descriptive error message.

#### Limitations

- Variadic functions are passed through unchanged (no wrapper)
- Statics are passed through unchanged
- Functions with `#[link_name]` are passed through unchanged

#### Example

```ignore
#[r_ffi_checked]
unsafe extern "C-unwind" {
    // Routed to main thread via with_r_thread when called from worker
    pub fn Rf_ScalarInteger(arg1: i32) -> SEXP;
    pub fn INTEGER(x: SEXP) -> *mut i32;
}
```

### `typed_dataframe!`

Define a compile-time-validated wrapper for an R `data.frame` input.

`typed_dataframe!` mirrors [`typed_list!`] for the data.frame shape:
declare the columns once, get a struct that implements `TryFromSexp`
(validating both the `data.frame` class and per-column SEXPTYPE) plus
per-column borrowed accessors that return `&[T]`.

#### Syntax

```ignore
typed_dataframe! {
    /// The shape we accept for the Theoph PK dataset.
    pub TheophDf {
        subject: i32,
        weight: f64,
        dose: f64,
        flag: Option<i32>,   // optional column
    }
}
```

For strict mode (reject any column not declared):
```ignore
typed_dataframe! {
    @exact;
    pub Strict { x: i32 }
}
```

#### Supported element types

v1 supports column element types that implement
`miniextendr_api::RNativeType`:

- `i32` — `INTSXP`
- `f64` — `REALSXP`
- `u8` — `RAWSXP`
- `miniextendr_api::RLogical` — `LGLSXP`
- `miniextendr_api::Rcomplex` — `CPLXSXP`

`String`/`&str` column types are not yet supported (character vectors
don't expose a contiguous slice). `bool` is also not yet supported as
a direct field type — use `RLogical` and convert per-element, or
follow the open follow-up issues from PR #698.

#### Generated API

For each `name: T` column the macro emits:
- `pub fn name(&self) -> &[T]` (required)
- `pub fn name(&self) -> Option<&[T]>` (optional, `Option<T>`)

Plus housekeeping:
- `pub fn nrow(&self) -> usize`
- `pub fn ncol(&self) -> usize` (count of *declared* columns)
- `pub fn as_sexp(&self) -> SEXP`

All borrowed accessors are bound to `&self`; the SEXP is protected
by the surrounding `#[miniextendr]` call wrapper while the struct is
alive.

#### Error reporting

`TryFromSexp::try_from_sexp` batches every per-column error into a
single `SexpError::InvalidValue`, so the R user sees one diagnostic
covering all missing or wrong-typed columns rather than a sequence of
stop-on-first-failure messages.

#### Example

```ignore
use miniextendr_api::{miniextendr, typed_dataframe};

typed_dataframe! {
    pub TheophDf {
        subject: i32,
        weight: f64,
        dose: f64,
    }
}

#[miniextendr]
pub fn theoph_nrow(df: TheophDf) -> i32 {
    // df.subject() -> &[i32], df.weight() -> &[f64]
    // Lengths are guaranteed equal across columns (data.frame invariant).
    df.nrow() as i32
}
```

[`typed_list!`]: macro@typed_list

### `typed_list!`

Create a `TypedListSpec` for validating `...` arguments or lists.

This macro provides ergonomic syntax for defining typed list specifications
that can be used with `Dots::typed()` to validate the structure of
`...` arguments passed from R.

#### Syntax

```text
typed_list!(
    name => type_spec,    // required field with type
    name? => type_spec,   // optional field with type
    name,                 // required field, any type
    name?,                // optional field, any type
)
```

For strict mode (no extra fields allowed):
```text
typed_list!(@exact; name => type_spec, ...)
```

#### Type Specifications

##### Base types (with optional length)
- `numeric()` / `numeric(4)` - Real/double vector
- `integer()` / `integer(4)` - Integer vector
- `logical()` / `logical(4)` - Logical vector
- `character()` / `character(4)` - Character vector
- `raw()` / `raw(4)` - Raw vector
- `complex()` / `complex(4)` - Complex vector
- `list()` / `list(4)` - List (VECSXP)

##### Special types
- `data_frame()` - Data frame
- `factor()` - Factor
- `matrix()` - Matrix
- `array()` - Array
- `function()` - Function
- `environment()` - Environment
- `null()` - NULL only
- `any()` - Any type

##### String literals
- `"numeric"`, `"integer"`, etc. - Same as call syntax
- `"data.frame"` - Data frame (alias)
- `"MyClass"` - Any other string is treated as a class name (uses `Rf_inherits`)

#### Examples

##### Basic usage

```ignore
use miniextendr_api::{miniextendr, typed_list, Dots};

#[miniextendr]
pub fn process_args(dots: ...) -> Result<i32, String> {
    let args = dots.typed(typed_list!(
        alpha => numeric(4),
        beta => list(),
        gamma? => "character",
    )).map_err(|e| e.to_string())?;

    let alpha: Vec<f64> = args.get("alpha").map_err(|e| e.to_string())?;
    Ok(alpha.len() as i32)
}
```

##### Strict mode

```ignore
// Reject any extra named fields
let args = dots.typed(typed_list!(@exact;
    x => numeric(),
    y => numeric(),
))?;
```

##### Class checking

```ignore
// Check for specific R class (uses Rf_inherits semantics)
let args = dots.typed(typed_list!(
    data => "data.frame",
    model => "lm",
))?;
```

##### Attribute sugar

Instead of calling `.typed()` manually, you can use `typed_list!` directly in the
`#[miniextendr]` attribute for automatic validation:

```ignore
#[miniextendr(dots = typed_list!(x => numeric(), y => numeric()))]
pub fn my_func(...) -> String {
    // `dots_typed` is automatically created and validated
    let x: f64 = dots_typed.get("x").expect("x");
    let y: f64 = dots_typed.get("y").expect("y");
    format!("x={}, y={}", x, y)
}
```

This injects validation at the start of the function body:
```ignore
let dots_typed = _dots.typed(typed_list!(...))
    .unwrap_or_else(|e| panic!("dots validation failed: {e}"));
```

See the [`#[miniextendr]`](macro@miniextendr) attribute documentation for more details.

