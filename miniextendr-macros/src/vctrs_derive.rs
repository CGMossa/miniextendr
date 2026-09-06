//! # `#[derive(Vctrs)]` - Rust Structs ↔ vctrs S3 Classes
//!
//! This module implements the `#[derive(Vctrs)]` macro which generates
//! vctrs-compatible S3 classes from Rust structs.
//!
//! ## Usage
//!
//! ```ignore
//! #[derive(Vctrs)]
//! #[vctrs(class = "percent", base = "double")]
//! pub struct Percent {
//!     #[vctrs(data)]
//!     data: Vec<f64>,
//! }
//! ```
//!
//! ## Attributes
//!
//! ### Container-level
//!
//! - `#[vctrs(class = "name")]` - R class name (required)
//! - `#[vctrs(base = "double" | "integer" | "list" | "record")]` - Base vector type
//! - `#[vctrs(abbr = "pct")]` - Abbreviation for `vec_ptype_abbr`
//! - `#[vctrs(inherit_base = true | false)]` - Whether to include base type in class vector
//! - `#[vctrs(coerce = "double" | "integer" | ...)]` - Additional types this class coerces with
//! - `#[vctrs(extends = "parent")]` - Parent vctrs class to inherit from. Prepends
//!   the parent into the class vector (after this class, before `vctrs_vctr`) so
//!   unhandled S3 generics fall through to the parent's methods, and generates
//!   bidirectional `vec_ptype2`/`vec_cast` stubs (parent wins as the supertype).
//!   The parent must share this type's `base` vector type. May be repeated for
//!   multiple parents.
//!
//! ### Field-level
//!
//! - `#[vctrs(data)]` - Mark field as the underlying data (required for `IntoVctrs`)
//! - `#[vctrs(skip)]` - Skip field when generating record fields
//!
//! ## Generated S3 Methods
//!
//! The derive macro generates the following R S3 methods:
//!
//! - `format.<class>()` - Format for printing
//! - `vec_ptype_abbr.<class>()` - Abbreviation (if provided)
//! - `vec_ptype_full.<class>()` - Full type name
//! - `vec_proxy.<class>()` - Proxy for subsetting operations
//! - `vec_restore.<class>()` - Restore from proxy after subsetting
//! - `vec_ptype2.<class>.<class>()` - Self-coercion prototype
//! - `vec_cast.<class>.<class>()` - Self-cast (identity)
//!
//! For record types, additional field accessor methods are generated.
//!
//! ## Registration
//!
//! Types with `#[derive(Vctrs)]` are automatically registered via linkme
//! distributed slices. No manual module declaration is needed.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident};

/// Parsed container-level `#[vctrs(...)]` attributes from a struct definition.
///
/// These control how the struct maps to a vctrs S3 class, including the
/// R class name, underlying vector type, printing abbreviation, and
/// which optional vctrs protocol methods to generate.
#[derive(Default)]
struct VctrsAttrs {
    /// R class name (e.g., `"vctrs_percent"`). Required.
    class: Option<String>,
    /// Base vector type: `"double"`, `"integer"`, `"character"`, `"list"`, `"record"`, etc.
    /// Defaults to `"double"` if not specified.
    base: Option<String>,
    /// Short abbreviation for `vec_ptype_abbr` display (e.g., `"pct"`).
    abbr: Option<String>,
    /// Whether to include the base type in the class vector (`inherit_base_type` in `new_vctr`).
    inherit_base: Option<bool>,
    /// Additional R types to generate bidirectional coercion methods for
    /// (e.g., `"double"` generates `vec_ptype2` and `vec_cast` between class and double).
    coerce_with: Vec<String>,
    /// Parent vctrs class(es) this type inherits from (`extends = "parent"`).
    ///
    /// Each parent is prepended into the R class vector *after* this class but
    /// before `vctrs_vctr`, so unhandled S3 generics fall through to the
    /// parent's methods. Bidirectional `vec_ptype2`/`vec_cast` stubs between the
    /// child and each parent are generated so coercion resolves (parent wins —
    /// matching vctrs' rule that the supertype is the common type).
    extends: Vec<String>,
    /// For `list_of` base type: an R expression for the element prototype (e.g., `"integer()"`).
    ptype: Option<String>,
    /// Generate `vec_proxy_equal` S3 method for equality testing.
    proxy_equal: bool,
    /// Generate `vec_proxy_compare` S3 method for comparison and sorting.
    proxy_compare: bool,
    /// Generate `vec_proxy_order` S3 method for ordering (may differ from compare).
    proxy_order: bool,
    /// Generate `vec_arith` S3 methods for arithmetic operations (`+`, `-`, `*`, etc.).
    arith: bool,
    /// Generate `vec_math` S3 method for math functions (`abs`, `sqrt`, `log`, etc.).
    math: bool,
}

/// Parsed field-level `#[vctrs(...)]` attributes from a struct field.
#[derive(Default)]
struct VctrsFieldAttrs {
    /// When `true`, this field holds the underlying vector data used by `IntoVctrs`.
    /// Exactly one field should be marked with `#[vctrs(data)]`.
    is_data: bool,
    /// When `true`, this field is excluded from record field generation.
    /// Useful for internal caches or derived state that should not appear in the R record.
    skip: bool,
}

/// Information about a single named struct field, including its vctrs attributes.
struct FieldInfo {
    /// The field's identifier (name).
    ident: syn::Ident,
    /// Parsed `#[vctrs(...)]` attributes on this field.
    attrs: VctrsFieldAttrs,
}

/// Parses container-level `#[vctrs(...)]` attributes from a struct's attribute list.
///
/// Extracts all recognized keys (`class`, `base`, `abbr`, `inherit_base`, `coerce`,
/// `ptype`, `proxy_equal`, `proxy_compare`, `proxy_order`, `arith`, `math`).
/// Returns an error for unrecognized attribute keys.
fn parse_vctrs_attrs(attrs: &[syn::Attribute]) -> syn::Result<VctrsAttrs> {
    let mut result = VctrsAttrs::default();

    for attr in attrs {
        if attr.path().is_ident("vctrs") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("class") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.class = Some(value.value());
                } else if meta.path.is_ident("base") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.base = Some(value.value());
                } else if meta.path.is_ident("abbr") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.abbr = Some(value.value());
                } else if meta.path.is_ident("inherit_base") {
                    let value: syn::LitBool = meta.value()?.parse()?;
                    result.inherit_base = Some(value.value());
                } else if meta.path.is_ident("coerce") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.coerce_with.push(value.value());
                } else if meta.path.is_ident("extends") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.extends.push(value.value());
                } else if meta.path.is_ident("ptype") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.ptype = Some(value.value());
                } else if meta.path.is_ident("proxy_equal") {
                    result.proxy_equal = true;
                } else if meta.path.is_ident("proxy_compare") {
                    result.proxy_compare = true;
                } else if meta.path.is_ident("proxy_order") {
                    result.proxy_order = true;
                } else if meta.path.is_ident("arith") {
                    result.arith = true;
                } else if meta.path.is_ident("math") {
                    result.math = true;
                } else {
                    return Err(meta.error(
                        "unknown vctrs attribute; expected one of: class, base, abbr, inherit_base, coerce, extends, ptype, proxy_equal, proxy_compare, proxy_order, arith, math",
                    ));
                }
                Ok(())
            })?;
        }
    }

    Ok(result)
}

/// Parses field-level `#[vctrs(...)]` attributes from a struct field's attribute list.
///
/// Recognizes `data` (mark as the underlying data field) and `skip` (exclude from
/// record generation). Returns an error for unrecognized field attribute keys.
fn parse_vctrs_field_attrs(attrs: &[syn::Attribute]) -> syn::Result<VctrsFieldAttrs> {
    let mut result = VctrsFieldAttrs::default();

    for attr in attrs {
        if attr.path().is_ident("vctrs") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("data") {
                    result.is_data = true;
                } else if meta.path.is_ident("skip") {
                    result.skip = true;
                } else {
                    return Err(meta.error("unknown vctrs field attribute; expected: data, skip"));
                }
                Ok(())
            })?;
        }
    }

    Ok(result)
}

/// Extracts field names and their vctrs attributes from a struct's `DeriveInput`.
///
/// Returns an error for tuple structs (unnamed fields). Returns an empty vec
/// for unit structs or non-struct items.
fn extract_fields(input: &DeriveInput) -> syn::Result<Vec<FieldInfo>> {
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => return Ok(Vec::new()),
    };

    match fields {
        Fields::Named(named) => {
            let mut result = Vec::new();
            for field in &named.named {
                if let Some(ident) = &field.ident {
                    let attrs = parse_vctrs_field_attrs(&field.attrs)?;
                    result.push(FieldInfo {
                        ident: ident.clone(),
                        attrs,
                    });
                }
            }
            Ok(result)
        }
        Fields::Unnamed(_) => Err(syn::Error::new_spanned(
            fields,
            "vctrs types require named fields",
        )),
        Fields::Unit => Ok(Vec::new()),
    }
}

/// Maps a base type string (e.g., `"double"`, `"integer"`, `"record"`) to its
/// corresponding `SEXPTYPE` token stream. Returns `None` for unrecognized types.
fn base_to_sexptype(base: &str) -> Option<TokenStream> {
    match base {
        "double" | "numeric" => Some(quote! { ::miniextendr_api::SEXPTYPE::REALSXP }),
        "integer" => Some(quote! { ::miniextendr_api::SEXPTYPE::INTSXP }),
        "logical" => Some(quote! { ::miniextendr_api::SEXPTYPE::LGLSXP }),
        "character" => Some(quote! { ::miniextendr_api::SEXPTYPE::STRSXP }),
        "raw" => Some(quote! { ::miniextendr_api::SEXPTYPE::RAWSXP }),
        "list" => Some(quote! { ::miniextendr_api::SEXPTYPE::VECSXP }),
        "record" => Some(quote! { ::miniextendr_api::SEXPTYPE::VECSXP }),
        _ => None,
    }
}

/// Maps a base type string to its `VctrsKind` token stream.
///
/// `"record"` maps to `Rcrd`, `"list"` maps to `ListOf`, and all other types map to `Vctr`.
fn base_to_kind(base: &str) -> TokenStream {
    match base {
        "record" => quote! { ::miniextendr_api::vctrs::VctrsKind::Rcrd },
        "list" => quote! { ::miniextendr_api::vctrs::VctrsKind::ListOf },
        _ => quote! { ::miniextendr_api::vctrs::VctrsKind::Vctr },
    }
}

/// Configuration for generating R wrapper code for a vctrs S3 class.
///
/// Passed to [`generate_r_wrappers`] to control which S3 methods are emitted.
struct RWrapperOptions<'a> {
    /// R class name (e.g., `"percent"`).
    class: &'a str,
    /// Base vector type (e.g., `"double"`, `"record"`, `"list"`).
    base: &'a str,
    /// Optional abbreviation for `vec_ptype_abbr`.
    abbr: Option<&'a str>,
    /// Field names for record types (used in `format` and field accessor methods).
    record_fields: &'a [String],
    /// Additional R types to generate bidirectional coercion methods for.
    coerce_with: &'a [String],
    /// Parent vctrs class(es) this type inherits from (`extends = "parent"`).
    extends: &'a [String],
    /// Whether `inherit_base_type = TRUE` is passed to `new_vctr`.
    inherit_base: bool,
    /// For `list_of`: R expression for element prototype (e.g., `"integer()"`).
    ptype: Option<&'a str>,
    /// Whether to generate `vec_proxy_equal` method.
    proxy_equal: bool,
    /// Whether to generate `vec_proxy_compare` method.
    proxy_compare: bool,
    /// Whether to generate `vec_proxy_order` method.
    proxy_order: bool,
    /// Whether to generate `vec_arith` methods.
    arith: bool,
    /// Whether to generate `vec_math` method.
    math: bool,
}

/// Generate R wrapper code for vctrs S3 methods.
///
/// This generates the following S3 methods for the vctrs class:
/// - `format.<class>()` - Format for printing
/// - `vec_ptype_abbr.<class>()` - Abbreviation (if provided)
/// - `vec_ptype_full.<class>()` - Full type name
/// - `vec_proxy.<class>()` - Proxy for subsetting operations
/// - `vec_restore.<class>()` - Restore from proxy
/// - `vec_ptype2.<class>.<class>()` - Self-coercion prototype
/// - `vec_cast.<class>.<class>()` - Self-cast (identity)
///
/// For record types, it additionally generates:
/// - Field accessor `$` methods via vctrs infrastructure
///
/// For list_of types (base = "list"):
/// - Appropriate list handling methods
///
/// Optional methods (when enabled):
/// - `vec_proxy_equal.<class>()` - For equality testing
/// - `vec_proxy_compare.<class>()` - For comparison/sorting
/// - `vec_proxy_order.<class>()` - For ordering
/// - `vec_arith.<class>.<class>()` - For arithmetic operations
/// - `vec_math.<class>()` - For math functions
fn generate_r_wrappers(opts: &RWrapperOptions) -> String {
    let class = opts.class;
    let base = opts.base;
    let abbr = opts.abbr;
    let record_fields = opts.record_fields;
    let coerce_with = opts.coerce_with;
    let extends = opts.extends;
    let inherit_base = opts.inherit_base;
    // R expression for the class vector passed to new_vctr/new_rcrd/new_list_of.
    // With `extends`, the parent class(es) are prepended after the child so that
    // S3 dispatch + vctrs coercion fall through to the parent's methods.
    let class_vec = r_class_vector(class, extends);
    let ptype = opts.ptype;
    let proxy_equal = opts.proxy_equal;
    let proxy_compare = opts.proxy_compare;
    let proxy_order = opts.proxy_order;
    let arith = opts.arith;
    let math = opts.math;
    let mut r_code = String::new();

    // region: format.<class>
    if base == "record" {
        // Record format: paste fields together with separator
        let field_formats: Vec<String> = record_fields
            .iter()
            .map(|f| format!("vctrs::field(x, \"{f}\")"))
            .collect();
        let fields_str = field_formats.join(", \"/\", ");
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs field
#' @export
format.{class} <- function(x, ...) {{
  paste0({fields_str})
}}
"#
        ));
    } else if base == "list" {
        // List-of format: show type and format each element
        r_code.push_str(&format!(
            r#"
#' @export
format.{class} <- function(x, ...) {{
  vapply(unclass(x), function(elt) {{
    if (is.null(elt)) "<NULL>" else paste0("<", vctrs::vec_ptype_abbr(elt), "[", vctrs::vec_size(elt), "]>")
  }}, character(1))
}}
"#
        ));
    } else {
        // Simple vctr format: use underlying data representation
        // Use unclass() instead of vec_data() to avoid recursion (vec_data calls vec_proxy)
        r_code.push_str(&format!(
            r#"
#' @export
format.{class} <- function(x, ...) {{
  format(unclass(x), ...)
}}
"#
        ));
    }
    // endregion

    // region: vec_ptype_abbr.<class>
    if let Some(abbr) = abbr {
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_ptype_abbr
#' @export
vec_ptype_abbr.{class} <- function(x, ...) {{
  "{abbr}"
}}
"#
        ));
    }
    // endregion

    // region: vec_ptype_full.<class>
    r_code.push_str(&format!(
        r#"
#' @importFrom vctrs vec_ptype_full
#' @export
vec_ptype_full.{class} <- function(x, ...) {{
  "{class}"
}}
"#
    ));
    // endregion

    // region: vec_proxy.<class> - strip class for operations
    if base == "record" {
        // Record proxy: convert to data frame for vctrs operations
        // vctrs expects rcrd proxy to be a data frame with n = number of records
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_proxy new_data_frame
#' @export
vec_proxy.{class} <- function(x, ...) {{
  data <- unclass(x)
  vctrs::new_data_frame(data, n = length(data[[1L]]))
}}
"#
        ));
    } else if base == "list" {
        // List-of proxy: use list_of_proxy (wraps elements in list for df-column behavior)
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_proxy new_data_frame
#' @export
vec_proxy.{class} <- function(x, ...) {{
  # Wrap each element in a list so it becomes a df column
  vctrs::new_data_frame(list(elt = unclass(x)))
}}
"#
        ));
    } else {
        // Simple vctr proxy: strip class to get underlying data
        // Use unclass() instead of vec_data() to avoid recursion (vec_data calls vec_proxy)
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_proxy
#' @export
vec_proxy.{class} <- function(x, ...) {{
  unclass(x)
}}
"#
        ));
    }
    // endregion

    // region: vec_restore.<class> - restore class after subsetting
    if base == "record" {
        // Record restore: convert data frame back to rcrd
        // x is a data frame from vec_proxy, convert to list and wrap as rcrd
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_restore new_rcrd
#' @export
vec_restore.{class} <- function(x, to, ...) {{
  vctrs::new_rcrd(as.list(x), class = {class_vec})
}}
"#
        ));
    } else if base == "list" {
        // List-of restore: extract elt column and wrap as list_of
        let ptype_expr = ptype.unwrap_or("NULL");
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_restore new_list_of
#' @export
vec_restore.{class} <- function(x, to, ...) {{
  vctrs::new_list_of(x$elt, ptype = {ptype_expr}, class = {class_vec})
}}
"#
        ));
    } else {
        // Simple vctr restore: use new_vctr
        let inherit_str = if inherit_base { "TRUE" } else { "FALSE" };
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_restore new_vctr
#' @export
vec_restore.{class} <- function(x, to, ...) {{
  vctrs::new_vctr(x, class = {class_vec}, inherit_base_type = {inherit_str})
}}
"#
        ));
    }
    // endregion

    // region: vec_ptype2.<class>.<class> - self-coercion returns empty prototype
    if base == "record" {
        // Record ptype2: extract prototype from x using vctrs::vec_ptype
        // This is the cleanest way since we don't know field types at compile time
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_ptype2 vec_ptype
#' @export
vec_ptype2.{class}.{class} <- function(x, y, ...) {{
  vctrs::vec_ptype(x)
}}
"#
        ));
    } else if base == "list" {
        // List-of ptype2: use new_list_of with common ptype
        let _ptype_expr = ptype.unwrap_or("NULL");
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_ptype2 new_list_of vec_ptype_common
#' @export
vec_ptype2.{class}.{class} <- function(x, y, ...) {{
  ptype <- vctrs::vec_ptype_common(attr(x, "ptype"), attr(y, "ptype"))
  vctrs::new_list_of(list(), ptype = ptype, class = {class_vec})
}}
"#
        ));
    } else {
        // Simple vctr ptype2: return empty vector with class
        let inherit_str = if inherit_base { "TRUE" } else { "FALSE" };
        r_code.push_str(&format!(
            r#"
#' @importFrom vctrs vec_ptype2 new_vctr
#' @export
vec_ptype2.{class}.{class} <- function(x, y, ...) {{
  vctrs::new_vctr({base}(0), class = {class_vec}, inherit_base_type = {inherit_str})
}}
"#,
            base = base_to_r_constructor(base)
        ));
    }
    // endregion

    // region: vec_cast.<class>.<class> - self-cast is identity
    r_code.push_str(&format!(
        r#"
#' @importFrom vctrs vec_cast
#' @export
vec_cast.{class}.{class} <- function(x, to, ...) {{
  x
}}
"#
    ));
    // endregion

    // region: Generate coercion methods for other types (e.g., double, integer)
    for other_type in coerce_with {
        // vec_ptype2.<class>.<other> - class wins, return class prototype
        let inherit_str = if inherit_base { "TRUE" } else { "FALSE" };

        if base != "record" {
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_ptype2 new_vctr
#' @export
vec_ptype2.{class}.{other_type} <- function(x, y, ...) {{
  vctrs::new_vctr({base}(0), class = {class_vec}, inherit_base_type = {inherit_str})
}}
"#,
                base = base_to_r_constructor(base)
            ));

            // vec_ptype2.<other>.<class> - symmetric
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_ptype2 new_vctr
#' @export
vec_ptype2.{other_type}.{class} <- function(x, y, ...) {{
  vctrs::new_vctr({base}(0), class = {class_vec}, inherit_base_type = {inherit_str})
}}
"#,
                base = base_to_r_constructor(base)
            ));

            // vec_cast.<class>.<other> - cast other to class
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_cast new_vctr
#' @export
vec_cast.{class}.{other_type} <- function(x, to, ...) {{
  vctrs::new_vctr(as.{base}(x), class = {class_vec}, inherit_base_type = {inherit_str})
}}
"#,
                base = base_to_r_as_func(base)
            ));

            // vec_cast.<other>.<class> - cast class to other (strip class)
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_cast vec_data
#' @export
vec_cast.{other_type}.{class} <- function(x, to, ...) {{
  vctrs::vec_data(x)
}}
"#
            ));
        }
    }
    // endregion

    // region: extends - cross-coercion between this class and its parent(s)
    //
    // With `extends = "parent"`, the parent class already sits in the class
    // vector (see `class_vec`), so S3 generics the child does not override
    // (format, print, vec_ptype_abbr, ...) fall through to the parent's methods
    // automatically via R's NextMethod / class-vector dispatch.
    //
    // Coercion (vec_ptype2 / vec_cast), however, is double-dispatched on the
    // *exact* class pair, so it does not inherit through the class vector. We
    // therefore emit explicit child<->parent method pairs. Following vctrs'
    // rule that the common type of a subtype and its supertype is the supertype,
    // vec_ptype2 returns the PARENT prototype, and casts re-wrap the shared
    // base data (child and parent share `base` — an `extends` parent must have
    // the same base vector type).
    for parent in extends {
        if base != "record" && base != "list" {
            let inherit_str = if inherit_base { "TRUE" } else { "FALSE" };
            let ctor = base_to_r_constructor(base);

            // vec_ptype2.<class>.<parent> - parent (supertype) wins.
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_ptype2 new_vctr
#' @export
vec_ptype2.{class}.{parent} <- function(x, y, ...) {{
  vctrs::new_vctr({ctor}(0), class = "{parent}", inherit_base_type = {inherit_str})
}}
"#
            ));

            // vec_ptype2.<parent>.<class> - symmetric, parent still wins.
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_ptype2 new_vctr
#' @export
vec_ptype2.{parent}.{class} <- function(x, y, ...) {{
  vctrs::new_vctr({ctor}(0), class = "{parent}", inherit_base_type = {inherit_str})
}}
"#
            ));

            // vec_cast.<parent>.<class> - upcast child -> parent (re-wrap shared base).
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_cast new_vctr vec_data
#' @export
vec_cast.{parent}.{class} <- function(x, to, ...) {{
  vctrs::new_vctr(vctrs::vec_data(x), class = "{parent}", inherit_base_type = {inherit_str})
}}
"#
            ));

            // vec_cast.<class>.<parent> - downcast parent -> child (re-wrap shared base).
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_cast new_vctr vec_data
#' @export
vec_cast.{class}.{parent} <- function(x, to, ...) {{
  vctrs::new_vctr(vctrs::vec_data(x), class = {class_vec}, inherit_base_type = {inherit_str})
}}
"#
            ));
        }
    }
    // endregion

    // region: vec_proxy_equal.<class> - proxy for equality testing
    if proxy_equal {
        if base == "record" {
            // For records, use the data frame proxy (already suitable for equality)
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_equal new_data_frame
#' @export
vec_proxy_equal.{class} <- function(x, ...) {{
  data <- unclass(x)
  vctrs::new_data_frame(data, n = length(data[[1L]]))
}}
"#
            ));
        } else if base == "list" {
            // For list_of, compare element by element
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_equal
#' @export
vec_proxy_equal.{class} <- function(x, ...) {{
  # For list-of, use element-wise proxy
  lapply(unclass(x), vctrs::vec_proxy_equal)
}}
"#
            ));
        } else {
            // For simple vctrs, use underlying data
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_equal
#' @export
vec_proxy_equal.{class} <- function(x, ...) {{
  unclass(x)
}}
"#
            ));
        }
    }
    // endregion

    // region: vec_proxy_compare.<class> - proxy for comparison/sorting
    if proxy_compare {
        if base == "record" {
            // For records, use the data frame proxy
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_compare new_data_frame
#' @export
vec_proxy_compare.{class} <- function(x, ...) {{
  data <- unclass(x)
  vctrs::new_data_frame(data, n = length(data[[1L]]))
}}
"#
            ));
        } else if base == "list" {
            // List-of types generally can't be compared
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_compare stop_incompatible_type
#' @export
vec_proxy_compare.{class} <- function(x, ...) {{
  vctrs::stop_incompatible_type(x, x, x_arg = "", y_arg = "", action = "compare")
}}
"#
            ));
        } else {
            // For simple vctrs, use underlying data
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_compare
#' @export
vec_proxy_compare.{class} <- function(x, ...) {{
  unclass(x)
}}
"#
            ));
        }
    }
    // endregion

    // region: vec_proxy_order.<class> - proxy for ordering (may differ from compare)
    if proxy_order {
        if base == "record" {
            // For records, use the data frame proxy
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_order new_data_frame
#' @export
vec_proxy_order.{class} <- function(x, ...) {{
  data <- unclass(x)
  vctrs::new_data_frame(data, n = length(data[[1L]]))
}}
"#
            ));
        } else if base == "list" {
            // List-of types generally can't be ordered
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_order stop_incompatible_type
#' @export
vec_proxy_order.{class} <- function(x, ...) {{
  vctrs::stop_incompatible_type(x, x, x_arg = "", y_arg = "", action = "order")
}}
"#
            ));
        } else {
            // For simple vctrs, use underlying data
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_proxy_order
#' @export
vec_proxy_order.{class} <- function(x, ...) {{
  unclass(x)
}}
"#
            ));
        }
    }
    // endregion

    // region: vec_arith.<class> - arithmetic operations (double dispatch)
    if arith {
        // For numeric-backed vctrs, arithmetic returns the same class
        if base != "record" && base != "list" && base != "character" && base != "raw" {
            let inherit_str = if inherit_base { "TRUE" } else { "FALSE" };

            // Base dispatcher: vec_arith.<class> does secondary dispatch on y
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_arith
#' @method vec_arith {class}
#' @export
vec_arith.{class} <- function(op, x, y, ...) {{
  UseMethod("vec_arith.{class}", y)
}}
"#
            ));

            // Default fallback for unknown y types
            // Use @method tag to tell roxygen that vec_arith.{class} is the generic
            r_code.push_str(&format!(
                r#"
#' @method vec_arith.{class} default
#' @importFrom vctrs stop_incompatible_op
#' @export
vec_arith.{class}.default <- function(op, x, y, ...) {{
  vctrs::stop_incompatible_op(op, x, y)
}}
"#
            ));

            // vec_arith.<class>.<class>
            // Use @method tag for proper S3 registration
            r_code.push_str(&format!(
                r#"
#' @method vec_arith.{class} {class}
#' @importFrom vctrs vec_arith vec_arith_base new_vctr
#' @export
vec_arith.{class}.{class} <- function(op, x, y, ...) {{
  result <- vctrs::vec_arith_base(op, x, y)
  vctrs::new_vctr(result, class = {class_vec}, inherit_base_type = {inherit_str})
}}
"#
            ));

            // vec_arith.<class>.numeric (right-hand side)
            // Use @method tag for proper S3 registration
            r_code.push_str(&format!(
                r#"
#' @method vec_arith.{class} numeric
#' @importFrom vctrs vec_arith vec_arith_base new_vctr
#' @export
vec_arith.{class}.numeric <- function(op, x, y, ...) {{
  result <- vctrs::vec_arith_base(op, x, y)
  vctrs::new_vctr(result, class = {class_vec}, inherit_base_type = {inherit_str})
}}
"#
            ));

            // vec_arith.numeric.<class> (left-hand side numeric op class)
            // vctrs exports vec_arith.numeric, so we import it to register methods on it.
            // Use @method tag for proper S3 registration.
            r_code.push_str(&format!(
                r#"
#' @method vec_arith.numeric {class}
#' @importFrom vctrs vec_arith.numeric vec_arith_base new_vctr
#' @export
vec_arith.numeric.{class} <- function(op, x, y, ...) {{
  result <- vctrs::vec_arith_base(op, x, y)
  vctrs::new_vctr(result, class = {class_vec}, inherit_base_type = {inherit_str})
}}
"#
            ));

            // vec_arith.<class>.MISSING (unary operations like -x)
            // Use @method tag for proper S3 registration
            r_code.push_str(&format!(
                r#"
#' @method vec_arith.{class} MISSING
#' @importFrom vctrs vec_arith vec_arith_base new_vctr
#' @export
vec_arith.{class}.MISSING <- function(op, x, y, ...) {{
  result <- vctrs::vec_arith_base(op, x, y)
  vctrs::new_vctr(result, class = {class_vec}, inherit_base_type = {inherit_str})
}}
"#
            ));
        }
    }
    // endregion

    // region: vec_math.<class> - math operations (abs, sqrt, log, etc.)
    if math {
        // For numeric-backed vctrs, math returns the same class
        if base != "record" && base != "list" && base != "character" && base != "raw" {
            let inherit_str = if inherit_base { "TRUE" } else { "FALSE" };
            r_code.push_str(&format!(
                r#"
#' @importFrom vctrs vec_math vec_math_base new_vctr
#' @export
vec_math.{class} <- function(.fn, .x, ...) {{
  result <- vctrs::vec_math_base(.fn, .x, ...)
  vctrs::new_vctr(result, class = {class_vec}, inherit_base_type = {inherit_str})
}}
"#
            ));
        }
    }

    r_code
}

/// Builds the R class-vector expression for `new_vctr`/`new_rcrd`/`new_list_of`.
///
/// Without `extends`, this is a bare string `"class"`. With one or more parents
/// it becomes `c("class", "parent1", ...)`, so the parent classes sit between
/// the child and `vctrs_vctr` in the final class vector. That ordering is how
/// S3 inheritance and vctrs' `vec_ptype2` fall-through resolve to the parent's
/// methods when the child does not override them.
fn r_class_vector(class: &str, extends: &[String]) -> String {
    if extends.is_empty() {
        format!("\"{class}\"")
    } else {
        let parents = extends
            .iter()
            .map(|p| format!("\"{p}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!("c(\"{class}\", {parents})")
    }
}

/// Maps a base type string to the corresponding R constructor function name
/// (e.g., `"double"` -> `"double"`, `"record"` -> `"list"`).
/// Used when generating `new_vctr(double(0), ...)` calls in R wrapper code.
fn base_to_r_constructor(base: &str) -> &'static str {
    match base {
        "double" | "numeric" => "double",
        "integer" => "integer",
        "logical" => "logical",
        "character" => "character",
        "raw" => "raw",
        "list" => "list",
        "record" => "list",
        _ => "double",
    }
}

/// Maps a base type string to the corresponding R `as.*` coercion function name
/// (e.g., `"integer"` -> `"integer"` for `as.integer()`).
/// Used in `vec_cast` coercion methods.
fn base_to_r_as_func(base: &str) -> &'static str {
    match base {
        "double" | "numeric" => "double",
        "integer" => "integer",
        "logical" => "logical",
        "character" => "character",
        "raw" => "raw",
        _ => "double",
    }
}

/// Main entry point for `#[derive(Vctrs)]`.
///
/// Parses the struct's `#[vctrs(...)]` attributes and fields, validates constraints
/// (must be a non-generic struct with at least one named field and a `class` attribute),
/// and generates:
///
/// - `impl VctrsClass` -- class metadata (name, kind, base type, abbreviation)
/// - `impl IntoVctrs` -- if a `#[vctrs(data)]` field is present
/// - `impl VctrsRecord` -- if base is `"record"` (provides field names)
/// - `impl VctrsListOf` -- if base is `"list"` with a `ptype`
/// - `R_WRAPPERS_VCTRS_{TYPE}` const -- R S3 method wrapper code
pub fn derive_vctrs(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse struct-level attributes
    let attrs = parse_vctrs_attrs(&input.attrs)?;

    // Validate: must be a struct
    match &input.data {
        Data::Struct(_) => {}
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "#[derive(Vctrs)] can only be applied to structs (use #[derive(RFactor)] for enums)",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "#[derive(Vctrs)] can only be applied to structs",
            ));
        }
    };

    // Extract field information
    let fields = extract_fields(&input)?;

    // Reject generic structs — generated R code cannot be generic
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "Vctrs does not support generic structs",
        ));
    }

    // Reject empty structs
    if fields.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "Vctrs requires at least one field",
        ));
    }

    // Require class attribute
    let class_name = attrs.class.ok_or_else(|| {
        syn::Error::new_spanned(
            &input,
            "#[derive(Vctrs)] requires #[vctrs(class = \"name\")] attribute",
        )
    })?;

    // Get base type (default to "double")
    let base = attrs.base.as_deref().unwrap_or("double");

    // Validate base type
    let sexptype = base_to_sexptype(base).ok_or_else(|| {
        syn::Error::new_spanned(
            &input,
            format!(
                "unknown base type '{}'; expected one of: double, integer, logical, character, raw, list, record",
                base
            ),
        )
    })?;

    // Get VctrsKind
    let kind = base_to_kind(base);

    // Determine inherit_base_type
    // Default: true for list/record, false for others
    let inherit_base = attrs
        .inherit_base
        .unwrap_or(matches!(base, "list" | "record"));

    // Get abbreviation
    let abbr = match &attrs.abbr {
        Some(a) => quote! { Some(#a) },
        None => quote! { None },
    };

    // `extends = "parent"` parents become the type's additional classes, sitting
    // between CLASS_NAME and "vctrs_vctr" in the constructed class vector. This is
    // the Rust-construction side of inheritance; the R-side methods prepend the
    // same parents (see `r_class_vector`) so subset/coerce results carry them too.
    let extends = &attrs.extends;
    let additional_classes_impl = if extends.is_empty() {
        TokenStream::new()
    } else {
        quote! {
            fn additional_classes() -> &'static [&'static str] {
                &[#(#extends),*]
            }
        }
    };

    // Generate VctrsClass implementation
    let vctrs_class_impl = quote! {
        impl #impl_generics ::miniextendr_api::vctrs::VctrsClass for #name #ty_generics #where_clause {
            const CLASS_NAME: &'static str = #class_name;
            const KIND: ::miniextendr_api::vctrs::VctrsKind = #kind;
            const BASE_TYPE: Option<::miniextendr_api::SEXPTYPE> = Some(#sexptype);
            const INHERIT_BASE_TYPE: bool = #inherit_base;
            const ABBR: Option<&'static str> = #abbr;
            #additional_classes_impl
        }
    };

    // Find data field for IntoVctrs
    let data_field = fields.iter().find(|f| f.attrs.is_data);

    // Class slice passed to new_vctr/new_rcrd/new_list_of. Without `extends` this
    // is the unchanged `&[Self::CLASS_NAME]`; with `extends` it concatenates the
    // parent classes (CLASS_NAME first, then each parent) so the constructed
    // class vector matches the R-side `r_class_vector` ordering.
    let (class_slice_binding, class_slice_expr) = if extends.is_empty() {
        (TokenStream::new(), quote! { &[Self::CLASS_NAME] })
    } else {
        (
            quote! {
                let __class: ::std::vec::Vec<&'static str> =
                    ::std::iter::once(Self::CLASS_NAME)
                        .chain(Self::additional_classes().iter().copied())
                        .collect();
            },
            quote! { &__class },
        )
    };

    // Generate IntoVctrs implementation if data field is marked
    let into_vctrs_impl = if let Some(data_field) = data_field {
        let data_ident = &data_field.ident;

        match base {
            "record" => {
                // For records, we need to build a List from all non-skipped fields
                let record_fields: Vec<_> = fields.iter().filter(|f| !f.attrs.skip).collect();
                let field_names: Vec<String> = record_fields
                    .iter()
                    .map(|f| crate::naming::ident_name(&f.ident))
                    .collect();
                let field_idents: Vec<_> = record_fields.iter().map(|f| &f.ident).collect();

                quote! {
                    impl #impl_generics ::miniextendr_api::vctrs::IntoVctrs for #name #ty_generics #where_clause {
                        fn into_vctrs(self) -> Result<::miniextendr_api::SEXP, ::miniextendr_api::vctrs::VctrsBuildError> {
                            use ::miniextendr_api::IntoR;

                            // Get attrs before moving fields out of self
                            let attrs = self.attrs();

                            // Build the fields list from pairs.
                            // Each `into_sexp()` is rooted via `__scope.protect_raw`
                            // so prior field SEXPs stay alive across the next
                            // field's allocations — UAF otherwise
                            // (reviews/2026-05-07-gctorture-audit.md).
                            // SAFETY: into_vctrs runs on the R main thread.
                            let fields = unsafe {
                                let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
                                let pairs: Vec<(&str, ::miniextendr_api::SEXP)> = vec![
                                    #( (#field_names, __scope.protect_raw(self.#field_idents.into_sexp())), )*
                                ];
                                ::miniextendr_api::list::List::from_raw_pairs(pairs)
                            };

                            #class_slice_binding
                            ::miniextendr_api::vctrs::new_rcrd(
                                fields,
                                #class_slice_expr,
                                &attrs,
                            )
                        }
                    }
                }
            }
            "list" => {
                // For list_of types
                // The ptype is optional and passed via #[vctrs(ptype = "...")]
                // new_list_of requires: List, Option<SEXP> ptype, Option<i32> size, &[&str] class, &[(&str, SEXP)] attrs
                quote! {
                    impl #impl_generics ::miniextendr_api::vctrs::IntoVctrs for #name #ty_generics #where_clause {
                        fn into_vctrs(self) -> Result<::miniextendr_api::SEXP, ::miniextendr_api::vctrs::VctrsBuildError> {
                            use ::miniextendr_api::IntoR;
                            use ::miniextendr_api::list::List;

                            // Get attrs before moving data out of self
                            let attrs = self.attrs();
                            // Convert data to List type - safe because into_sexp() for Vec<Vec<T>> produces VECSXP
                            let data_sexp = self.#data_ident.into_sexp();
                            let list = unsafe { List::from_raw(data_sexp) };
                            // For list_of, we pass size = list length, ptype = None (handled in R wrapper)
                            let size = Some(list.len() as i32);
                            #class_slice_binding
                            ::miniextendr_api::vctrs::new_list_of(
                                list,
                                None,  // ptype - handled in R wrapper via attribute
                                size,
                                #class_slice_expr,
                                &attrs,
                            )
                        }
                    }
                }
            }
            _ => {
                // For simple vctrs (double, integer, etc.)
                quote! {
                    impl #impl_generics ::miniextendr_api::vctrs::IntoVctrs for #name #ty_generics #where_clause {
                        fn into_vctrs(self) -> Result<::miniextendr_api::SEXP, ::miniextendr_api::vctrs::VctrsBuildError> {
                            use ::miniextendr_api::IntoR;

                            // Get attrs before moving data out of self
                            let attrs = self.attrs();
                            let data = self.#data_ident.into_sexp();
                            #class_slice_binding
                            ::miniextendr_api::vctrs::new_vctr(
                                data,
                                #class_slice_expr,
                                &attrs,
                                Some(Self::INHERIT_BASE_TYPE),
                            )
                        }
                    }
                }
            }
        }
    } else {
        TokenStream::new()
    };

    // Generate VctrsRecord implementation if base is "record"
    let record_impl = if base == "record" {
        let field_names: Vec<String> = fields
            .iter()
            .filter(|f| !f.attrs.skip)
            .map(|f| crate::naming::ident_name(&f.ident))
            .collect();
        let field_name_strs: Vec<&str> = field_names.iter().map(|s| s.as_str()).collect();

        quote! {
            impl #impl_generics ::miniextendr_api::vctrs::VctrsRecord for #name #ty_generics #where_clause {
                fn field_names() -> &'static [&'static str] {
                    &[#(#field_name_strs),*]
                }
            }
        }
    } else {
        TokenStream::new()
    };

    // Generate VctrsListOf implementation if base is "list" and ptype is specified
    let listof_impl = if base == "list" {
        if let Some(ptype) = &attrs.ptype {
            quote! {
                impl #impl_generics ::miniextendr_api::vctrs::VctrsListOf for #name #ty_generics #where_clause {
                    fn ptype_expr() -> &'static str {
                        #ptype
                    }
                }
            }
        } else {
            TokenStream::new()
        }
    } else {
        TokenStream::new()
    };

    // Generate R wrapper code for vctrs S3 methods
    let record_field_names: Vec<String> = if base == "record" {
        fields
            .iter()
            .filter(|f| !f.attrs.skip)
            .map(|f| crate::naming::ident_name(&f.ident))
            .collect()
    } else {
        Vec::new()
    };
    let r_wrappers = generate_r_wrappers(&RWrapperOptions {
        class: &class_name,
        base,
        abbr: attrs.abbr.as_deref(),
        record_fields: &record_field_names,
        coerce_with: &attrs.coerce_with,
        extends: &attrs.extends,
        inherit_base,
        ptype: attrs.ptype.as_deref(),
        proxy_equal: attrs.proxy_equal,
        proxy_compare: attrs.proxy_compare,
        proxy_order: attrs.proxy_order,
        arith: attrs.arith,
        math: attrs.math,
    });
    let type_start = name.span().start();
    let source_line_lit = syn::LitInt::new(&type_start.line.to_string(), name.span());
    let source_col_lit = syn::LitInt::new(&(type_start.column + 1).to_string(), name.span());

    // Generate the R_WRAPPERS_VCTRS_{TYPE} const
    let name_upper = name.to_string().to_uppercase();
    let r_wrappers_const_ident = Ident::new(
        &format!("R_WRAPPERS_VCTRS_{}", name_upper),
        Span::call_site(),
    );
    let source_location_doc = crate::source_location_doc(name.span());

    Ok(quote! {
        #vctrs_class_impl
        #record_impl
        #listof_impl
        #into_vctrs_impl

        /// Generated R wrapper code for vctrs S3 methods (distributed slice).
        #[doc = #source_location_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[doc(hidden)]
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_R_WRAPPERS), linkme(crate = ::miniextendr_api::linkme))]
        static #r_wrappers_const_ident: ::miniextendr_api::registry::RWrapperEntry =
            ::miniextendr_api::registry::RWrapperEntry {
                priority: ::miniextendr_api::registry::RWrapperPriority::Vctrs,
                source_file: file!(),
                content: concat!(
                    "# Generated from Rust derive(Vctrs) on `",
                    stringify!(#name),
                    "` (",
                    file!(),
                    ":",
                    #source_line_lit,
                    ":",
                    #source_col_lit,
                    ")",
                    #r_wrappers
                ),
            };
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_vctr() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "percent", base = "double")]
            struct Percent {
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        assert!(code.contains("VctrsClass"));
        assert!(code.contains("CLASS_NAME"));
        assert!(code.contains("percent"));
        assert!(code.contains("REALSXP"));
    }

    #[test]
    fn test_simple_vctr_with_data_field() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "percent", base = "double")]
            struct Percent {
                #[vctrs(data)]
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        // Should generate VctrsClass
        assert!(code.contains("VctrsClass"));
        // Should generate IntoVctrs with data field
        assert!(code.contains("IntoVctrs"));
        assert!(code.contains("self . data"));
        assert!(code.contains("new_vctr"));
    }

    #[test]
    fn test_record_vctr() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "rational", base = "record")]
            struct Rational {
                n: Vec<i32>,
                d: Vec<i32>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        assert!(code.contains("VctrsClass"));
        assert!(code.contains("VctrsRecord"));
        assert!(code.contains("Rcrd"));
        assert!(code.contains("field_names"));
    }

    #[test]
    fn test_record_vctr_with_data() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "rational", base = "record")]
            struct Rational {
                #[vctrs(data)]
                n: Vec<i32>,
                d: Vec<i32>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        // Should generate all three traits
        assert!(code.contains("VctrsClass"));
        assert!(code.contains("VctrsRecord"));
        assert!(code.contains("IntoVctrs"));
        // Record uses new_rcrd
        assert!(code.contains("new_rcrd"));
        // Field names should be included
        assert!(code.contains("\"n\""));
        assert!(code.contains("\"d\""));
    }

    #[test]
    fn test_skip_field() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "rational", base = "record")]
            struct Rational {
                #[vctrs(data)]
                n: Vec<i32>,
                d: Vec<i32>,
                #[vctrs(skip)]
                cached: Option<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        // Field names should NOT include cached
        assert!(code.contains("\"n\""));
        assert!(code.contains("\"d\""));
        assert!(!code.contains("\"cached\""));
    }

    #[test]
    fn test_with_abbr() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "percent", base = "double", abbr = "pct")]
            struct Percent {
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        assert!(code.contains("pct"));
    }

    #[test]
    fn test_empty_struct_error() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "empty", base = "double")]
            struct Empty {}
        };

        let result = derive_vctrs(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires at least one field")
        );
    }

    #[test]
    fn test_generic_struct_error() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "generic", base = "double")]
            struct Generic<T> {
                data: Vec<T>,
            }
        };

        let result = derive_vctrs(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("does not support generic")
        );
    }

    #[test]
    fn test_missing_class_error() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(base = "double")]
            struct Percent {
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires #[vctrs(class")
        );
    }

    #[test]
    fn test_enum_error() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "color")]
            enum Color {
                Red,
                Green,
                Blue,
            }
        };

        let result = derive_vctrs(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("can only be applied to structs")
        );
    }

    #[test]
    fn test_r_wrappers_const_generated() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "percent", base = "double")]
            struct Percent {
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        // Should generate R_WRAPPERS_VCTRS_PERCENT const
        assert!(code.contains("R_WRAPPERS_VCTRS_PERCENT"));
        assert!(code.contains("pub const"));
    }

    #[test]
    fn test_r_wrappers_content_simple() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "percent",
            base: "double",
            abbr: Some("pct"),
            record_fields: &[],
            coerce_with: &[],
            extends: &[],
            inherit_base: false,
            ptype: None,
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: false,
            math: false,
        });

        // Should have format method using unclass (not vec_data to avoid recursion)
        assert!(r_code.contains("format.percent"));
        assert!(r_code.contains("unclass(x)"));

        // Should have vec_ptype_abbr since abbr provided
        assert!(r_code.contains("vec_ptype_abbr.percent"));
        assert!(r_code.contains("\"pct\""));

        // Should have vec_ptype_full
        assert!(r_code.contains("vec_ptype_full.percent"));

        // Should have vec_proxy using unclass (not vec_data to avoid recursion)
        assert!(r_code.contains("vec_proxy.percent"));

        // Should have vec_restore
        assert!(r_code.contains("vec_restore.percent"));

        // Should have self-coercion methods
        assert!(r_code.contains("vec_ptype2.percent.percent"));
        assert!(r_code.contains("vec_cast.percent.percent"));
    }

    #[test]
    fn test_r_wrappers_content_record() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "rational",
            base: "record",
            abbr: None,
            record_fields: &["n".to_string(), "d".to_string()],
            coerce_with: &[],
            extends: &[],
            inherit_base: true, // records default to inherit_base = true
            ptype: None,
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: false,
            math: false,
        });

        // Should have format method with vctrs::field accessors
        assert!(r_code.contains("format.rational"));
        assert!(r_code.contains("vctrs::field(x, \"n\")"));
        assert!(r_code.contains("vctrs::field(x, \"d\")"));

        // Should NOT have vec_ptype_abbr since no abbr
        assert!(!r_code.contains("vec_ptype_abbr.rational"));

        // Should have vec_ptype_full
        assert!(r_code.contains("vec_ptype_full.rational"));

        // Should have vec_proxy and vec_restore for records
        assert!(r_code.contains("vec_proxy.rational"));
        assert!(r_code.contains("vec_restore.rational"));
        // vec_proxy uses new_data_frame, vec_restore uses new_rcrd
        assert!(r_code.contains("new_data_frame"));
        assert!(r_code.contains("new_rcrd"));

        // Should have self-coercion (uses vec_ptype for records)
        assert!(r_code.contains("vec_ptype2.rational.rational"));
        assert!(r_code.contains("vec_ptype(x)"));
        assert!(r_code.contains("vec_cast.rational.rational"));
    }

    #[test]
    fn test_r_wrappers_no_abbr() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "mytype",
            base: "integer",
            abbr: None,
            record_fields: &[],
            coerce_with: &[],
            extends: &[],
            inherit_base: false,
            ptype: None,
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: false,
            math: false,
        });

        // Should NOT have vec_ptype_abbr
        assert!(!r_code.contains("vec_ptype_abbr.mytype"));

        // Should still have format and vec_ptype_full
        assert!(r_code.contains("format.mytype"));
        assert!(r_code.contains("vec_ptype_full.mytype"));
    }

    #[test]
    fn test_r_wrappers_with_coercion() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "percent",
            base: "double",
            abbr: Some("%"),
            record_fields: &[],
            coerce_with: &["double".to_string()],
            extends: &[],
            inherit_base: false,
            ptype: None,
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: false,
            math: false,
        });

        // Should have self-coercion
        assert!(r_code.contains("vec_ptype2.percent.percent"));
        assert!(r_code.contains("vec_cast.percent.percent"));

        // Should have coercion with double
        assert!(r_code.contains("vec_ptype2.percent.double"));
        assert!(r_code.contains("vec_ptype2.double.percent"));
        assert!(r_code.contains("vec_cast.percent.double"));
        assert!(r_code.contains("vec_cast.double.percent"));
    }

    #[test]
    fn test_r_wrappers_list_of() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "list_of_integers",
            base: "list",
            abbr: Some("list<int>"),
            record_fields: &[],
            coerce_with: &[],
            extends: &[],
            inherit_base: true,
            ptype: Some("integer()"),
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: false,
            math: false,
        });

        // Should have format for list_of
        assert!(r_code.contains("format.list_of_integers"));
        assert!(r_code.contains("vapply"));

        // Should have vec_ptype_abbr
        assert!(r_code.contains("vec_ptype_abbr.list_of_integers"));
        assert!(r_code.contains("list<int>"));

        // Should have vec_proxy for list_of
        assert!(r_code.contains("vec_proxy.list_of_integers"));
        assert!(r_code.contains("new_data_frame"));

        // Should have vec_restore with ptype
        assert!(r_code.contains("vec_restore.list_of_integers"));
        assert!(r_code.contains("new_list_of"));
        assert!(r_code.contains("integer()"));

        // Should have vec_ptype2 for list_of
        assert!(r_code.contains("vec_ptype2.list_of_integers.list_of_integers"));
        assert!(r_code.contains("vec_ptype_common"));
    }

    #[test]
    fn test_r_wrappers_proxy_methods() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "mynum",
            base: "double",
            abbr: None,
            record_fields: &[],
            coerce_with: &[],
            extends: &[],
            inherit_base: false,
            ptype: None,
            proxy_equal: true,
            proxy_compare: true,
            proxy_order: true,
            arith: false,
            math: false,
        });

        // Should have proxy_equal method
        assert!(r_code.contains("vec_proxy_equal.mynum"));

        // Should have proxy_compare method
        assert!(r_code.contains("vec_proxy_compare.mynum"));

        // Should have proxy_order method
        assert!(r_code.contains("vec_proxy_order.mynum"));
    }

    #[test]
    fn test_r_wrappers_arith_methods() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "mynum",
            base: "double",
            abbr: None,
            record_fields: &[],
            coerce_with: &[],
            extends: &[],
            inherit_base: false,
            ptype: None,
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: true,
            math: false,
        });

        // Should have base dispatcher for double dispatch
        assert!(r_code.contains("vec_arith.mynum <- function(op, x, y, ...)"));
        assert!(r_code.contains("UseMethod(\"vec_arith.mynum\", y)"));

        // Should have default fallback with @method tag
        assert!(r_code.contains("@method vec_arith.mynum default"));
        assert!(r_code.contains("vec_arith.mynum.default"));
        assert!(r_code.contains("stop_incompatible_op"));

        // Should have vec_arith methods with @method tags for proper S3 registration
        assert!(r_code.contains("@method vec_arith.mynum mynum"));
        assert!(r_code.contains("vec_arith.mynum.mynum"));
        assert!(r_code.contains("@method vec_arith.mynum numeric"));
        assert!(r_code.contains("vec_arith.mynum.numeric"));
        // vec_arith.numeric.mynum uses @method since vec_arith.numeric
        // is exported by vctrs (we import it)
        assert!(r_code.contains("@method vec_arith.numeric mynum"));
        assert!(r_code.contains("@importFrom vctrs vec_arith.numeric"));
        assert!(r_code.contains("vec_arith.numeric.mynum"));
        assert!(r_code.contains("@method vec_arith.mynum MISSING"));
        assert!(r_code.contains("vec_arith.mynum.MISSING"));
        assert!(r_code.contains("vec_arith_base"));
    }

    #[test]
    fn test_r_wrappers_math_methods() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "mynum",
            base: "double",
            abbr: None,
            record_fields: &[],
            coerce_with: &[],
            extends: &[],
            inherit_base: false,
            ptype: None,
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: false,
            math: true,
        });

        // Should have vec_math method
        assert!(r_code.contains("vec_math.mynum"));
        assert!(r_code.contains("vec_math_base"));
    }

    #[test]
    fn test_r_class_vector_helper() {
        // No parents -> bare string.
        assert_eq!(r_class_vector("percent", &[]), "\"percent\"");
        // One parent -> c("child", "parent").
        assert_eq!(
            r_class_vector("fancy_percent", &["percent".to_string()]),
            "c(\"fancy_percent\", \"percent\")"
        );
        // Multiple parents preserve order, child first.
        assert_eq!(
            r_class_vector("c", &["b".to_string(), "a".to_string()]),
            "c(\"c\", \"b\", \"a\")"
        );
    }

    #[test]
    fn test_r_wrappers_with_extends() {
        let r_code = generate_r_wrappers(&RWrapperOptions {
            class: "fancy_percent",
            base: "double",
            abbr: None,
            record_fields: &[],
            coerce_with: &[],
            extends: &["percent".to_string()],
            inherit_base: false,
            ptype: None,
            proxy_equal: false,
            proxy_compare: false,
            proxy_order: false,
            arith: false,
            math: false,
        });

        // The constructed class vector (in vec_restore / vec_ptype2) carries the
        // parent so S3 + coercion fall through.
        assert!(r_code.contains("class = c(\"fancy_percent\", \"percent\")"));

        // Cross-coercion stubs between child and parent are emitted.
        assert!(r_code.contains("vec_ptype2.fancy_percent.percent"));
        assert!(r_code.contains("vec_ptype2.percent.fancy_percent"));
        assert!(r_code.contains("vec_cast.percent.fancy_percent"));
        assert!(r_code.contains("vec_cast.fancy_percent.percent"));

        // ptype2 returns the parent (supertype) prototype.
        assert!(r_code.contains("class = \"percent\""));

        // The default format method is still present; fall-through to the
        // parent's format happens at runtime via the class vector.
        assert!(r_code.contains("format.fancy_percent"));
    }

    #[test]
    fn test_extends_emits_additional_classes_impl() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "fancy_percent", base = "double", extends = "percent")]
            struct FancyPercent {
                #[vctrs(data)]
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        // additional_classes() override is generated and lists the parent.
        assert!(code.contains("additional_classes"));
        assert!(code.contains("\"percent\""));
    }

    #[test]
    fn test_no_extends_keeps_class_name_slice() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "percent", base = "double")]
            struct Percent {
                #[vctrs(data)]
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        // Without extends, no additional_classes override and the slice stays
        // the unchanged &[Self::CLASS_NAME].
        assert!(!code.contains("additional_classes"));
        assert!(code.contains("Self :: CLASS_NAME"));
    }
}
