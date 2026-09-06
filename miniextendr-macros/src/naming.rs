//! Naming helpers for the static symbols the `#[miniextendr]` attribute emits.
//!
//! These formatters live in one place so the attribute macro and any later
//! consumer (e.g. a registration-chasing linter) can compute the exact same
//! identifiers from a source `syn::Ident`.

use syn::ext::IdentExt;

// region: raw identifiers
//
// Rust keywords are ordinary R names (`type`, `where`, `match`, `mod`, ...),
// so users spell them as raw identifiers: `fn r#match(r#where: &str)`. The
// `r#` prefix is Rust surface syntax only — `Display` for `syn::Ident` keeps it
// (`"r#where"`), which is neither a valid R name, nor a valid C symbol
// fragment, nor accepted by `syn::Ident::new` / `format_ident!`. Every place
// that turns a user identifier into a *name* (R formals, R function/method
// names, list/data-frame column names, C symbols, HashMap keys, synthesized
// Rust bindings) must go through `ident_name` / `unraw` instead of
// `to_string()` / direct interpolation. Token positions (`quote! { #ident }`)
// keep the raw ident — `r#where` is what the Rust side needs.

/// The user's identifier as a plain name: `r#where` → `"where"`, `x` → `"x"`.
pub(crate) fn ident_name(ident: &syn::Ident) -> String {
    ident.unraw().to_string()
}

/// The user's identifier without its `r#` prefix, for `format_ident!` /
/// `quote!` interpolation into *synthesized* identifiers (`__storage_where`).
/// Not for emitting the identifier itself — `where` is not a valid binding.
pub(crate) fn unraw(ident: &syn::Ident) -> syn::Ident {
    ident.unraw()
}

/// A *name* (a column / list-element name, from a user field or a `rename`)
/// as a Rust identifier for a generated struct field or binding. Keywords come
/// back raw so `type` emits `r#type` instead of an unparsable token stream.
pub(crate) fn name_ident(name: &str) -> syn::Ident {
    syn::parse_str::<syn::Ident>(name)
        .unwrap_or_else(|_| syn::Ident::new_raw(name, proc_macro2::Span::call_site()))
}
// endregion

/// Identifier for the generated `const &str` holding the R wrapper source.
///
/// Returns `R_WRAPPER_{RUST_IDENT}` (uppercased).
pub(crate) fn r_wrapper_const_ident_for(rust_ident: &syn::Ident) -> syn::Ident {
    let upper = ident_name(rust_ident).to_uppercase();
    quote::format_ident!("R_WRAPPER_{upper}")
}

// region: crate-prefixed C symbol naming (#1273)
//
// Every `#[miniextendr]`-emitted `#[no_mangle]` C symbol is prefixed with the
// consuming crate's name so that two packages loaded into the same webR
// session (which share one GOT under Emscripten's SIDE_MODULE model) can't
// have the first-loaded package's definition of a name silently win for a
// later package's identically-named export. Native `dyn.load` with
// `RTLD_LOCAL` was never affected — this is purely a wasm cross-package
// hygiene fix — but the prefix is applied unconditionally on every target so
// there's a single code path and no target-dependent symbol shape.
//
// Registration name and linker symbol name are kept identical everywhere
// (the invariant the wasm snapshot writer,
// `miniextendr-api/src/wasm_registry_writer.rs`, depends on to reconstruct
// `extern "C" { fn <name>(...); }` blocks from `R_CallMethodDef.name`) — so
// every formatter below produces the *one* string used for both the
// `#[no_mangle]` fn/static definition and its registration entry.

/// The consuming crate's name, used as the prefix for every macro-emitted
/// `#[no_mangle]` C symbol.
///
/// Reads `CARGO_CRATE_NAME`, which cargo sets (with hyphens already
/// normalized to underscores) for the rustc invocation that expands this
/// proc macro — the same precedent `miniextendr_init!()` relies on
/// (`lib.rs`, `miniextendr_init` fn). Real macro expansion (i.e. compiling a
/// downstream crate through cargo) always has this set.
///
/// Falls back to `CARGO_PKG_NAME` (normalized the same way, since it is
/// *not* auto-normalized) for the one context where `CARGO_CRATE_NAME` is
/// absent: this crate's own unit/insta tests call codegen functions directly
/// at test *runtime* rather than through real macro expansion, and cargo
/// does not forward `CARGO_CRATE_NAME` to a test binary's process
/// environment (verified empirically — only `CARGO_PKG_*` vars are). Falls
/// back further to the literal `"crate"` if even that is absent (e.g. a
/// direct `rustc` invocation bypassing cargo entirely).
pub(crate) fn crate_prefix() -> String {
    std::env::var("CARGO_CRATE_NAME").unwrap_or_else(|_| {
        std::env::var("CARGO_PKG_NAME")
            .map(|n| n.replace('-', "_"))
            .unwrap_or_else(|_| "crate".to_string())
    })
}

/// `C_<crate>_<fn>` — bare-fn C wrapper identifier (internal-wrapper arm
/// only; `extern`/`#[export_name]` fns pass through untouched and own their
/// own cross-package uniqueness).
pub(crate) fn bare_fn_c_wrapper_ident(rust_ident: &syn::Ident) -> syn::Ident {
    let prefix = crate_prefix();
    let rust_ident = unraw(rust_ident);
    quote::format_ident!("C_{prefix}_{rust_ident}")
}

/// `C_<crate>_<Type>__<method>`, or `C_<crate>_<Type>_<label>_<method>` when
/// the impl block carries a label — impl-method C wrapper identifier.
pub(crate) fn impl_method_c_wrapper_ident(
    type_ident: &syn::Ident,
    label: Option<&str>,
    method_ident: &syn::Ident,
) -> syn::Ident {
    let prefix = crate_prefix();
    let method_ident = unraw(method_ident);
    if let Some(label) = label {
        quote::format_ident!("C_{prefix}_{type_ident}_{label}_{method_ident}")
    } else {
        quote::format_ident!("C_{prefix}_{type_ident}__{method_ident}")
    }
}

/// `C_<crate>_<Type>__<Trait>__<member>` as a `String` — shared by
/// `TraitMethod` and `TraitConst`, and by both the ident and string forms
/// each needs, so the format string can't drift across the four call sites
/// that used to duplicate it.
pub(crate) fn trait_member_c_wrapper_string(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    member_ident: &syn::Ident,
) -> String {
    let prefix = crate_prefix();
    let member_ident = unraw(member_ident);
    format!("C_{prefix}_{type_ident}__{trait_name}__{member_ident}")
}

/// Same as [`trait_member_c_wrapper_string`], as a `syn::Ident` for Rust
/// token generation.
pub(crate) fn trait_member_c_wrapper_ident(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    member_ident: &syn::Ident,
) -> syn::Ident {
    quote::format_ident!(
        "{}",
        trait_member_c_wrapper_string(type_ident, trait_name, member_ident)
    )
}

/// `C_<crate>__mx_rdata_get_<Type>_<field>` — sidecar field getter C symbol.
pub(crate) fn sidecar_getter_c_name(type_name: &str, field_name: &str) -> String {
    let prefix = crate_prefix();
    format!("C_{prefix}__mx_rdata_get_{type_name}_{field_name}")
}

/// `C_<crate>__mx_rdata_set_<Type>_<field>` — sidecar field setter C symbol.
pub(crate) fn sidecar_setter_c_name(type_name: &str, field_name: &str) -> String {
    let prefix = crate_prefix();
    format!("C_{prefix}__mx_rdata_set_{type_name}_{field_name}")
}

/// `__mx_altrep_reg_<crate>_<Ident>` — ALTREP class registration fn.
pub(crate) fn altrep_reg_fn_ident(ident: &syn::Ident) -> syn::Ident {
    let prefix = crate_prefix();
    quote::format_ident!("__mx_altrep_reg_{prefix}_{ident}")
}

/// `__VTABLE_<CRATE>_<TRAIT>_FOR_<TYPE>` — trait-impl vtable static.
///
/// `trait_name_upper` / `type_name_str` are already uppercased by the
/// caller (matching the pre-existing convention); the crate prefix is
/// uppercased here to match.
pub(crate) fn vtable_static_ident(trait_name_upper: &str, type_name_str: &str) -> syn::Ident {
    let crate_upper = crate_prefix().to_uppercase();
    quote::format_ident!("__VTABLE_{crate_upper}_{trait_name_upper}_FOR_{type_name_str}")
}

/// `__vtshim_<crate>_<Type>__<Trait>__<method>` — trait-impl vtable shim fn.
pub(crate) fn vtshim_ident(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    method_ident: &syn::Ident,
) -> syn::Ident {
    let prefix = crate_prefix();
    let method_ident = unraw(method_ident);
    quote::format_ident!("__vtshim_{prefix}_{type_ident}__{trait_name}__{method_ident}")
}
// endregion

/// Convert a PascalCase string to snake_case.
///
/// Inserts an underscore before each uppercase letter (except the first),
/// then lowercases the entire result. For example, `"InProgress"` becomes
/// `"in_progress"`.
pub(crate) fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.char_indices() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.extend(c.to_lowercase());
    }
    out
}

/// Convert a PascalCase string to kebab-case (`InProgress` → `in-progress`).
pub(crate) fn to_kebab_case(s: &str) -> String {
    to_snake_case(s).replace('_', "-")
}

/// Apply a `rename_all` transformation to a variant name.
///
/// Supports `"snake_case"`, `"kebab-case"`, `"lower"`, `"upper"`. Returns the
/// name unchanged if `rename_all` is `None` or unrecognised.
pub(crate) fn apply_rename_all(name: &str, rename_all: Option<&str>) -> String {
    match rename_all {
        Some("snake_case") => to_snake_case(name),
        Some("kebab-case") => to_kebab_case(name),
        Some("lower") => name.to_lowercase(),
        Some("upper") => name.to_uppercase(),
        _ => name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // At unit-test runtime cargo does not forward CARGO_CRATE_NAME to the
    // test binary's process env, so crate_prefix() takes the CARGO_PKG_NAME
    // fallback: "miniextendr-macros" → "miniextendr_macros". Real macro
    // expansion (inside rustc) always sees CARGO_CRATE_NAME. The insta
    // snapshots pin the same value; if this test fails the snapshots are
    // stale too.
    #[test]
    fn crate_prefix_fallback_is_normalized_pkg_name() {
        assert_eq!(crate_prefix(), "miniextendr_macros");
    }

    #[test]
    fn c_symbols_are_crate_prefixed() {
        let ty = quote::format_ident!("Counter");
        let tr = quote::format_ident!("Resettable");
        let m = quote::format_ident!("reset");
        assert_eq!(
            bare_fn_c_wrapper_ident(&m).to_string(),
            "C_miniextendr_macros_reset"
        );
        assert_eq!(
            impl_method_c_wrapper_ident(&ty, None, &m).to_string(),
            "C_miniextendr_macros_Counter__reset"
        );
        assert_eq!(
            impl_method_c_wrapper_ident(&ty, Some("basic"), &m).to_string(),
            "C_miniextendr_macros_Counter_basic_reset"
        );
        assert_eq!(
            trait_member_c_wrapper_string(&ty, &tr, &m),
            "C_miniextendr_macros_Counter__Resettable__reset"
        );
        assert_eq!(
            trait_member_c_wrapper_ident(&ty, &tr, &m).to_string(),
            trait_member_c_wrapper_string(&ty, &tr, &m)
        );
        assert_eq!(
            sidecar_getter_c_name("Counter", "count"),
            "C_miniextendr_macros__mx_rdata_get_Counter_count"
        );
        assert_eq!(
            sidecar_setter_c_name("Counter", "count"),
            "C_miniextendr_macros__mx_rdata_set_Counter_count"
        );
        assert_eq!(
            altrep_reg_fn_ident(&ty).to_string(),
            "__mx_altrep_reg_miniextendr_macros_Counter"
        );
        assert_eq!(
            vtable_static_ident("RESETTABLE", "COUNTER").to_string(),
            "__VTABLE_MINIEXTENDR_MACROS_RESETTABLE_FOR_COUNTER"
        );
        assert_eq!(
            vtshim_ident(&ty, &tr, &m).to_string(),
            "__vtshim_miniextendr_macros_Counter__Resettable__reset"
        );
    }

    #[test]
    fn raw_identifiers_lose_their_prefix_in_names() {
        let raw: syn::Ident = syn::parse_quote!(r#where);
        let plain: syn::Ident = syn::parse_quote!(x);
        assert_eq!(raw.to_string(), "r#where", "Display keeps the prefix");
        assert_eq!(ident_name(&raw), "where");
        assert_eq!(ident_name(&plain), "x");
        assert_eq!(unraw(&raw).to_string(), "where");
        assert_eq!(
            quote::format_ident!("__storage_{}", unraw(&raw)),
            "__storage_where"
        );
        // The C symbol / const-name formatters unraw internally.
        assert_eq!(
            bare_fn_c_wrapper_ident(&syn::parse_quote!(r#match)).to_string(),
            "C_miniextendr_macros_match"
        );
        assert_eq!(
            r_wrapper_const_ident_for(&syn::parse_quote!(r#match)).to_string(),
            "R_WRAPPER_MATCH"
        );
        let ty: syn::Ident = syn::parse_quote!(Row);
        assert_eq!(
            impl_method_c_wrapper_ident(&ty, None, &syn::parse_quote!(r#type)).to_string(),
            "C_miniextendr_macros_Row__type"
        );
    }

    #[test]
    fn name_ident_makes_keywords_raw() {
        assert_eq!(name_ident("type").to_string(), "r#type");
        assert_eq!(name_ident("where").to_string(), "r#where");
        assert_eq!(name_ident("value").to_string(), "value");
        // Round-trips: a raw ident's name maps back to the same raw ident.
        let raw: syn::Ident = syn::parse_quote!(r#type);
        assert_eq!(name_ident(&ident_name(&raw)), raw);
    }

    #[test]
    fn snake_case() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("InProgress"), "in_progress");
        assert_eq!(to_snake_case("ABC"), "a_b_c");
        assert_eq!(to_snake_case("Red"), "red");
    }

    #[test]
    fn kebab_case() {
        assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
        assert_eq!(to_kebab_case("InProgress"), "in-progress");
    }

    #[test]
    fn rename_all() {
        assert_eq!(
            apply_rename_all("HelloWorld", Some("snake_case")),
            "hello_world"
        );
        assert_eq!(
            apply_rename_all("HelloWorld", Some("kebab-case")),
            "hello-world"
        );
        assert_eq!(apply_rename_all("HelloWorld", Some("lower")), "helloworld");
        assert_eq!(apply_rename_all("HelloWorld", Some("upper")), "HELLOWORLD");
        assert_eq!(apply_rename_all("HelloWorld", None), "HelloWorld");
    }
}
