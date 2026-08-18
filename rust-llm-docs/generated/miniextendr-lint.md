# miniextendr_lint v0.1.0

miniextendr-lint: internal build-time lint helpers for the workspace.

This crate scans Rust sources for miniextendr macro usage and emits
cargo warnings with actionable diagnostics. It is intended for local
development and CI, not as a public API.

## Usage in build.rs

```ignore
fn main() {
    miniextendr_lint::build_script();
}
```

## Configuration
- Controlled by the `MINIEXTENDR_LINT` env var (enabled by default).
- Set it to `0`, `false`, `no`, or `off` to disable.

## Lint Codes

Each diagnostic carries a stable `MXL###` code. See [`LintCode`] for the full catalog.

---

## Modules

### `crate_index`

`pub mod crate_index;`

Shared crate index built from a single parse pass over all source files.

All lint rules operate on this index rather than re-parsing files.

### `diagnostic`

`pub mod diagnostic;`

Structured diagnostic output for lint rules.

### `helpers`

`pub mod helpers;`

Shared utility functions for lint rule implementations.

### `lint_code`

`pub mod lint_code;`

Stable lint rule identifiers.

Each rule has a code like `MXL008` that is grep-able and CI-friendly.

### `rules`

`pub mod rules;`

Lint rule implementations.

Each submodule contains one or more related lint checks. All rules operate
on the shared [`CrateIndex`] and produce
[`Diagnostic`] values.

### `rules::export_attrs`

`pub mod export_attrs;`

Export attribute redundancy checks.

- MXL203: `internal` + `noexport` redundancy.

### `rules::ffi_unchecked`

`pub mod ffi_unchecked;`

`_unchecked` FFI call outside guard context.

- MXL301: Warns on `sys::*_unchecked()` calls in user code.
  These bypass main-thread routing and must only be called inside
  `with_r_unwind_protect`, `with_r_thread`, or similar guard closures.

### `rules::fn_visibility`

`pub mod fn_visibility;`

Function visibility checks.

- MXL106: Non-pub function has `/// @export` (contradictory).

### `rules::impl_validation`

`pub mod impl_validation;`

Impl block validation: class system compatibility and label uniqueness.

- MXL008: Trait impl class system incompatible with inherent impl.
- MXL009: Multiple impl blocks for one type without labels.
- MXL010: Duplicate labels on impl blocks for one type.

### `rules::r_reserved_params`

`pub mod r_reserved_params;`

R reserved-word parameter name check.

- MXL110: A `#[miniextendr]` function or method has a parameter whose name
  is an R reserved word. The proc macro forwards parameter names verbatim
  into the generated R wrapper, so the wrapper will be syntactically invalid.

### `rules::rf_error`

`pub mod rf_error;`

Direct `Rf_error`/`Rf_errorcall` usage lint.

- MXL300: Warns on direct `Rf_error`/`Rf_errorcall` calls in user code.
  These longjmp through Rust frames, bypassing destructors unless wrapped in
  `R_UnwindProtect`. Prefer `panic!()` or `Err(...)` which produce structured
  R condition objects via the tagged-condition transport.

### `rules::s4_method_prefix`

`pub mod s4_method_prefix;`

MXL111: `s4_*` method name on `#[miniextendr(s4)]` impl.

S4 codegen auto-prepends `s4_` to every instance method name when generating
the R generic. A Rust method named `s4_foo` on an `#[miniextendr(s4)]` impl
produces an R generic `s4_s4_foo`, making it unreachable via the expected
`s4_foo(obj, ...)` call.

### `rules::trait_tag_collision`

`pub mod trait_tag_collision;`

MXL303: trait-impl vtable-symbol collision detection.

Each `#[miniextendr] impl Trait for Type` emits a vtable static named
`__VTABLE_{CRATE}_{TRAIT}_FOR_{TYPE}` (see
`miniextendr-macros/src/naming.rs::vtable_static_ident`, called from
`miniextendr-macros/src/miniextendr_impl_trait/vtable.rs`), where `{CRATE}`
is the consuming crate's uppercased name (#1273 webR cross-package symbol
uniqueness), the trait name is `trait_ident.to_uppercase()`, and the type
name is the last path segment uppercased (`type_to_uppercase_name`). The
static is emitted with `#[unsafe(no_mangle)]`, so two impls whose
`(TRAIT, TYPE)` pair collapses to the same uppercased symbol produce
**duplicate `no_mangle` symbols** — a hard linker failure whose message is
divorced from the source.

The lint compares symbols *within one crate*, where the crate prefix is a
constant — so the comparison here uses the crate-invariant
`__VTABLE_{TRAIT}_FOR_{TYPE}` suffix rather than reconstructing the prefix
(the lint runs from the user crate's `build.rs`, which has no
`CARGO_CRATE_NAME` for the crate being linted anyway). Verdicts are
identical either way.

Rust's coherence rules already forbid the *same* trait implemented twice for
the *same* type, so the only way two distinct impls collide is via the
macro's **case-folding**: e.g. `impl Counter for Foo` and `impl counter for
Foo`, or `impl Trait for Foo` and `impl Trait for foo`, all collapse to the
same `__VTABLE_…` symbol. This rule catches that before the linker does.

Scope: crate-wide. The two colliding impls may live in different files, so
the check aggregates across the whole [`CrateIndex`] rather than per-file.

Escape hatch: `// mxl::allow(MXL303)` on (or directly above) either impl
line suppresses the diagnostic for that impl. `MINIEXTENDR_LINT=0` disables
all rules.

### `rules::vctrs_self_ctor`

`pub mod vctrs_self_ctor;`

MXL120: vctrs constructor returns `Self` / named type, or impl has an instance-method receiver.

A `#[miniextendr(vctrs(...))]` impl has two hard invariants:

1. **Constructor return type** — the constructor (`fn new` or a method tagged
   `#[miniextendr(constructor)]`) must NOT return `Self`, `&Self`, `&mut Self`,
   `Box<Self>`, the named impl type, `Result<Self, _>`, or `Result<NamedType, _>`.
   The generated R wrapper passes the return value to `vctrs::new_vctr()` /
   `new_rcrd()` / `new_list_of()`, which require a plain vector payload, not an
   `ExternalPtr` (`EXTPTRSXP`).

2. **Instance receivers** — no method may carry any form of `self` receiver
   (`&self`, `&mut self`, `self`, `self: &ExternalPtr<Self>`, etc.).
   A vctrs object is an S3-classed base vector; there is no Rust `Self` stored
   inside the SEXP.  The C wrapper cannot reconstruct `Self` from a base vector,
   so calling an instance method would panic at runtime.

#### Mirror

The same checks fire as proc-macro hard errors in
`miniextendr-macros/src/miniextendr_impl.rs` (search `MXL120`).
This lint is defence-in-depth: it catches the mistake during the
build-time static-analysis pass (when the macro isn't being expanded,
e.g. lint-only IDE runs, third-party tooling).
Keep both implementations in sync: if the macro relaxes either check,
update this rule too.

### `rules::vec_into_sexp`

`pub mod vec_into_sexp;`

`into_sexp()` inside a `vec!`/array literal — use-after-free idiom.

- MXL302: Warns on `into_sexp()` / `into_sexp_unchecked()` calls that appear as
  elements *inside* a `vec!` or `&[...]` literal.

  Each `into_sexp` allocates a fresh SEXP. When several occur in one literal
  (`vec![(k, a.into_sexp()), (k, b.into_sexp())]`), nothing roots the earlier
  elements until the whole `Vec` reaches `List::from_raw_pairs`, so building a
  later element can trigger a GC that collects an earlier, still-unprotected one
  — a use-after-free. This recurred enough (#307, the 2026-05-07 gctorture audit)
  that the `IntoList` / `DataFrameRow` derives now wrap every element in
  `__scope.protect_raw(...)`; this lint stops new hand-written sites from
  reintroducing the raw form silently.

---

## Re-exports

### `pub use crate_index::CrateIndex;`

### `pub use crate_index::LintItem;`

### `pub use crate_index::LintKind;`

### `pub use diagnostic::Diagnostic;`

### `pub use diagnostic::Severity;`

### `pub use lint_code::LintCode;`

---

## Structs

### `LintReport`

```rust
pub struct LintReport
```

Result of running the lint over a crate source tree.

**Fields:**

- `files`: `Vec<std::path::PathBuf>`
  - Rust source files that were scanned.
- `diagnostics`: `Vec<Diagnostic>`
  - Structured diagnostics from all rules.
- `errors`: `Vec<String>`
  - Legacy string errors (derived from diagnostics, for backward compatibility).

### `crate_index::AttributedTraitImpl`

```rust
pub struct AttributedTraitImpl
```

**Fields:**

- `type_name`: `String`
- `trait_name`: `String`
- `class_system`: `Option<String>`
- `line`: `usize`
- `suppressed_mxl303`: `bool`
  - True when the impl carries a `// mxl::allow(MXL303)` escape-hatch comment

### `crate_index::CrateIndex`

```rust
pub struct CrateIndex
```

Shared parsed state for all lint rules.

**Fields:**

- `files`: `Vec<std::path::PathBuf>`
  - All scanned Rust source files.
- `file_data`: `std::collections::HashMap<std::path::PathBuf, FileData>`
  - Per-file parsed data.

**Inherent associated items:**

#### `build`

```rust
fn build(root: &Path) -> Result<Self, String>
```

Build the index from a crate root directory.

### `crate_index::FileData`

```rust
pub struct FileData
```

**Fields:**

- `miniextendr_items`: `Vec<LintItem>`
- `types_with_external_ptr`: `std::collections::HashSet<String>`
- `types_with_typed_external`: `std::collections::HashSet<String>`
- `inherent_impl_class_systems`: `std::collections::HashMap<String, (String, usize)>`
- `attributed_trait_impls`: `Vec<AttributedTraitImpl>`
- `impl_blocks_per_type`: `std::collections::HashMap<String, Vec<(Option<String>, usize)>>`
- `fn_visibility`: `std::collections::HashMap<String, bool>`
- `declared_child_mods`: `Vec<String>`
  - Simple `mod child;` declarations (by ident name).
- `path_redirected_mods`: `Vec<(String, String)>`
  - `#[path = "file.rs"] mod name;` declarations: (mod_name, file_path_str).
- `mod_decl_cfgs`: `std::collections::HashMap<String, Vec<String>>`
  - cfg attrs on `mod child;` declarations: mod_name -> cfg strings.
- `export_control`: `std::collections::HashMap<String, (bool, bool, usize)>`
  - (has_internal, has_noexport, line)
- `impl_methods`: `std::collections::HashMap<String, Vec<ImplMethodEntry>>`
  - Methods per inherent impl type: type_name → `Vec<ImplMethodEntry>`.
- `fn_doc_tags`: `std::collections::HashMap<String, Vec<String>>`
  - Known roxygen tags: "@noRd", "@export", "@keywords internal"
- `rf_error_calls`: `Vec<(String, usize)>`
  - Lines containing direct Rf_error/Rf_errorcall calls: (function_name, line_number).
- `ffi_unchecked_calls`: `Vec<(String, usize)>`
  - Lines containing `ffi::*_unchecked()` calls: (function_name, line_number).
- `vec_into_sexp_calls`: `Vec<(String, usize)>`
  - `into_sexp()` / `into_sexp_unchecked()` calls that appear *inside* a `vec!`/array
- `fn_param_names`: `std::collections::HashMap<String, Vec<(String, usize)>>`
  - Maps fn/method name → list of (param_name, line) for params that are R reserved words.

### `crate_index::ImplMethodEntry`

```rust
pub struct ImplMethodEntry
```

Per-method data collected during the crate-index pass for impl-method lint rules.

**Fields:**

- `method_name`: `String`
- `line`: `usize`
- `class_system`: `String`
- `return_type_str`: `String`
  - Stringified return type tokens (empty string = `()` / no explicit return).
- `receiver_kind`: `MethodReceiverKind`
  - Receiver kind detected from the method signature.
- `has_constructor_attr`: `bool`
  - True when the method carries `#[miniextendr(constructor)]`.

### `crate_index::LintItem`

```rust
pub struct LintItem
```

**Fields:**

- `kind`: `LintKind`
- `name`: `String`
- `label`: `Option<String>`
- `line`: `usize`

**Inherent associated items:**

#### `new`

```rust
fn new(kind: LintKind, name: String, line: usize) -> Self
```

#### `with_label`

```rust
fn with_label(kind: LintKind, name: String, label: Option<String>, line: usize) -> Self
```

### `diagnostic::Diagnostic`

```rust
pub struct Diagnostic
```

A single lint diagnostic with structured metadata.

**Fields:**

- `code`: `crate::lint_code::LintCode`
  - Stable rule code (e.g. `MXL106`).
- `severity`: `Severity`
  - Severity level.
- `path`: `std::path::PathBuf`
  - Source file path.
- `line`: `usize`
  - 1-based line number (0 if unknown).
- `message`: `String`
  - Primary diagnostic message.
- `help`: `Option<String>`
  - Optional fix guidance.

**Inherent associated items:**

#### `new`

```rust
fn new(code: LintCode, path: impl Into<PathBuf>, line: usize, message: String) -> Self
```

Create a new diagnostic with the rule's default severity.

#### `to_legacy_string`

```rust
fn to_legacy_string(self: &Self) -> String
```

Format as a legacy error string (for backward-compatible `LintReport::errors`).

#### `with_help`

```rust
fn with_help(self: Self, help: impl Into<String>) -> Self
```

Attach a help message.

### `helpers::MiniextendrImplAttrs`

```rust
pub struct MiniextendrImplAttrs
```

Parsed miniextendr attribute information for an impl block.

**Fields:**

- `class_system`: `Option<String>`
  - Class system (e.g., "r6", "s3", "s4", "s7", or empty for env)
- `label`: `Option<String>`
  - Optional label for distinguishing multiple impl blocks of the same type
- `internal`: `bool`
  - Has `internal` flag
- `noexport`: `bool`
  - Has `noexport` flag
- `strict`: `bool`
  - Has `strict` flag

---

## Enums

### `crate_index::LintKind`

```rust
pub enum LintKind
```

**Variants:**

- `Function`
- `Impl`
- `Struct`
- `TraitImpl`
- `Vctrs`

### `crate_index::MethodReceiverKind`

```rust
pub enum MethodReceiverKind
```

Receiver kind for an impl method, mirroring `ReceiverKind` in `miniextendr-macros`.

Mirror: `miniextendr-macros/src/miniextendr_impl.rs` — `ReceiverKind`.
Keep both in sync: if the macro relaxes one receiver kind, update this enum too.

**Variants:**

- `None`
  - No self — static / associated function.
- `Ref`
  - `&self`
- `RefMut`
  - `&mut self`
- `Value`
  - `self` (consuming)
- `ExternalPtrRef`
  - `self: &ExternalPtr<Self>`
- `ExternalPtrRefMut`
  - `self: &mut ExternalPtr<Self>`
- `ExternalPtrValue`
  - `self: ExternalPtr<Self>`

**Inherent associated items:**

#### `is_instance`

```rust
fn is_instance(self: Self) -> bool
```

Returns true if this is an instance receiver (any form of `self`).

Mirrors `ReceiverKind::is_instance` in `miniextendr-macros/src/miniextendr_impl.rs`.
`Value` (consuming `self`) is **excluded** — the macro treats consuming-`self` methods
separately: they are either constructors (`returns Self` or `#[miniextendr(constructor)]`)
or finalizers, not ordinary instance calls.  Including `Value` here would produce a
false-positive for a vctrs method with `#[miniextendr(constructor)]` that consumes `self`.

#### `spelling`

```rust
fn spelling(self: Self) -> &'static str
```

Human-readable spelling used in diagnostic messages.

### `diagnostic::Severity`

```rust
pub enum Severity
```

Diagnostic severity level.

**Variants:**

- `Info`
  - Migration hints and informational notes.
- `Warning`
  - Default for new rules; non-blocking.
- `Error`
  - CI-blocking in strict mode.

### `lint_code::LintCode`

```rust
pub enum LintCode
```

Stable lint rule identifier.

Display format is `MXL###`, derived directly from the variant name.

**Variants:**

- `MXL008`
  - Trait impl class system incompatible with inherent impl class system.
- `MXL009`
  - Multiple impl blocks for one type without labels.
- `MXL010`
  - Duplicate labels on impl blocks for one type.
- `MXL106`
  - Registered top-level function is not `pub`.
- `MXL110`
  - Parameter name is an R reserved word; codegen will produce invalid R syntax.
- `MXL111`
  - `s4_*` method name on `#[miniextendr(s4)]` impl — codegen auto-prepends `s4_`.
- `MXL120`
  - vctrs constructor returns `Self` / named type, or impl has an instance-method receiver.
- `MXL203`
  - `internal` + `noexport` redundancy.
- `MXL300`
  - Direct `Rf_error`/`Rf_errorcall` call in user code.
- `MXL301`
  - `_unchecked` FFI call outside guard context.
- `MXL302`
  - `into_sexp()` call inside a `vec!`/array literal — unprotected SEXP across allocations (UAF).
- `MXL303`
  - Two `#[miniextendr]` trait impls collapse to the same vtable symbol

**Inherent associated items:**

#### `default_severity`

```rust
fn default_severity(self: Self) -> super::diagnostic::Severity
```

Default severity for this rule.

---

## Functions

### `build_script`

```rust
fn build_script()
```

Entry point for build.rs. Runs the lint and prints cargo directives.

Controlled by `MINIEXTENDR_LINT` env var (enabled by default).
Set to `0`, `false`, `no`, or `off` to disable.

### `helpers::extract_cfg_attrs`

```rust
fn extract_cfg_attrs(attrs: &[syn::Attribute]) -> Vec<String>
```

Extract `#[cfg(...)]` attributes as normalized token strings.

### `helpers::extract_path_attr`

```rust
fn extract_path_attr(attrs: &[syn::Attribute]) -> Option<String>
```

Extract `#[path = "..."]` attribute value from a module declaration.

### `helpers::extract_roxygen_tags`

```rust
fn extract_roxygen_tags(attrs: &[syn::Attribute]) -> Vec<String>
```

Extract roxygen tags from doc-comment attributes.

Looks through `/// ...` comments for patterns like `@export`, `@noRd`,
`@keywords internal`, etc. Returns the tag names found.

### `helpers::has_derive`

```rust
fn has_derive(attrs: &[syn::Attribute], name: &str) -> bool
```

Returns true if the attribute list contains `#[derive(name)]`
or `#[derive(miniextendr_api::name)]` for the given derive `name`
(e.g. `"ExternalPtr"`, `"Altrep"`, `"Vctrs"`).

### `helpers::has_miniextendr_attr`

```rust
fn has_miniextendr_attr(attrs: &[syn::Attribute]) -> bool
```

Returns true when the attribute list contains `#[miniextendr]`.

### `helpers::impl_type_name`

```rust
fn impl_type_name(ty: &syn::Type) -> Option<String>
```

Extracts a displayable type name from an impl self type.

### `helpers::is_altrep_struct`

```rust
fn is_altrep_struct(item: &syn::ItemStruct) -> bool
```

Check if a struct with `#[miniextendr]` should be treated as ALTREP (needing `struct Name;` in module).

Returns true only for 1-field structs without explicit mode attrs (list, dataframe, externalptr).
Multi-field structs, structs with explicit mode attrs, and enums don't need module entries.

### `helpers::parse_miniextendr_impl_attrs`

```rust
fn parse_miniextendr_impl_attrs(attrs: &[syn::Attribute]) -> MiniextendrImplAttrs
```

Parse the #[miniextendr(...)] attribute to extract class system, label, and flags.

### `helpers::should_skip_dir`

```rust
fn should_skip_dir(path: &std::path::Path) -> bool
```

Returns whether a directory should be skipped during lint tree traversal.

### `lint_enabled`

```rust
fn lint_enabled(env_var: &str) -> Result<bool, String>
```

Returns whether the lint should run based on the given env var.

Defaults to `true` when the var is unset. Set to 0/false/no/off to disable.

### `rules::export_attrs::check`

```rust
fn check(index: &crate::crate_index::CrateIndex, diagnostics: &mut Vec<crate::diagnostic::Diagnostic>)
```

### `rules::ffi_unchecked::check`

```rust
fn check(index: &crate::crate_index::CrateIndex, diagnostics: &mut Vec<crate::diagnostic::Diagnostic>)
```

### `rules::fn_visibility::check`

```rust
fn check(index: &crate::crate_index::CrateIndex, diagnostics: &mut Vec<crate::diagnostic::Diagnostic>)
```

### `rules::impl_validation::check`

```rust
fn check(index: &crate::crate_index::CrateIndex, diagnostics: &mut Vec<crate::diagnostic::Diagnostic>)
```

### `rules::r_reserved_params::check`

```rust
fn check(index: &crate::crate_index::CrateIndex, diagnostics: &mut Vec<crate::diagnostic::Diagnostic>)
```

### `rules::rf_error::check`

```rust
fn check(index: &crate::crate_index::CrateIndex, diagnostics: &mut Vec<crate::diagnostic::Diagnostic>)
```

### `rules::run_all_rules`

```rust
fn run_all_rules(index: &crate::crate_index::CrateIndex) -> Vec<crate::diagnostic::Diagnostic>
```

Run all lint rules against the crate index, collecting diagnostics.

### `rules::s4_method_prefix::check`

```rust
fn check(index: &crate::crate_index::CrateIndex, diagnostics: &mut Vec<crate::diagnostic::Diagnostic>)
```

### `rules::trait_tag_collision::check`

```rust
fn check(index: &crate::crate_index::CrateIndex, diagnostics: &mut Vec<crate::diagnostic::Diagnostic>)
```

### `rules::vctrs_self_ctor::check`

```rust
fn check(index: &crate::crate_index::CrateIndex, diagnostics: &mut Vec<crate::diagnostic::Diagnostic>)
```

### `rules::vec_into_sexp::check`

```rust
fn check(index: &crate::crate_index::CrateIndex, diagnostics: &mut Vec<crate::diagnostic::Diagnostic>)
```

### `run`

```rust
fn run(root: impl AsRef<std::path::Path>) -> Result<LintReport, String>
```

Run the lint against the crate rooted at `root`.

If `root/src` exists, that directory is scanned. Otherwise `root` is scanned.
