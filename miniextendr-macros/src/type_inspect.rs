//! Lightweight type-introspection helpers shared by parsing and codegen.

/// Returns the `n`-th generic type argument from a path segment.
pub(crate) fn nth_type_argument(seg: &syn::PathSegment, n: usize) -> Option<&syn::Type> {
    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
        let mut count = 0;
        for arg in ab.args.iter() {
            if let syn::GenericArgument::Type(ty) = arg {
                if count == n {
                    return Some(ty);
                }
                count += 1;
            }
        }
    }
    None
}

/// Returns the first generic type argument from a path segment.
pub(crate) fn first_type_argument(seg: &syn::PathSegment) -> Option<&syn::Type> {
    nth_type_argument(seg, 0)
}

/// Returns the second generic type argument from a path segment.
pub(crate) fn second_type_argument(seg: &syn::PathSegment) -> Option<&syn::Type> {
    nth_type_argument(seg, 1)
}

/// Returns `true` if `ty` is syntactically `SEXP`.
#[inline]
pub(crate) fn is_sexp_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(p) if p
        .path
        .segments
        .last()
        .map(|s| s.ident == "SEXP")
        .unwrap_or(false))
}

/// Framework type names that hold R memory and are `!Send` by design.
///
/// Used only for thread-strategy selection: values of these types can neither
/// move into the worker closure nor cross back out of it (`run_on_worker`
/// requires `Send`), so any function touching one stays on the main thread
/// even under `worker-default`. Covers the raw `SEXP`, `AltrepSexp`, the
/// zero-copy R-backed views (`RDVector`, `RDMatrix`, `RndVec`, `RndMat`,
/// `ProtectedStrVec`), and the owned GC-rooted handles (`BuiltDataFrame`,
/// `DataFrameShape`). Arbitrary user `!Send` types can't be detected
/// syntactically — those need an explicit `no_worker`.
const MAIN_THREAD_BOUND: &[&str] = &[
    "SEXP",
    "AltrepSexp",
    "RDVector",
    "RDMatrix",
    "RndVec",
    "RndMat",
    "ProtectedStrVec",
    "BuiltDataFrame",
    "DataFrameShape",
];

/// Returns `true` if `ty` is an input type bound to the R main thread.
///
/// Checks only the outermost path segment: main-thread-bound inputs arrive
/// bare (`x: SEXP`, `v: RDVector<f64>`), never nested inside containers.
/// Return-type analysis keeps the narrower [`is_sexp_type`] and uses the
/// recursive [`is_main_thread_bound_return`] for thread selection.
#[inline]
pub(crate) fn is_main_thread_bound_input(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(p) if p
        .path
        .segments
        .last()
        .map(|s| MAIN_THREAD_BOUND.contains(&s.ident.to_string().as_str()))
        .unwrap_or(false))
}

/// Returns `true` if `ty` is (or contains) a main-thread-bound type anywhere
/// in a return position — e.g. `BuiltDataFrame`, `Result<BuiltDataFrame,
/// String>`, `Option<DataFrameShape>`, `Vec<BuiltDataFrame>`.
///
/// Unlike inputs, main-thread-bound returns routinely nest inside `Result` /
/// `Option` / containers, so this walks the whole type tree. Under
/// `worker-default` a function whose return type matches is forced onto the
/// main thread: the value owns R memory (`!Send`) and cannot cross back from
/// the worker (`run_on_worker` requires `T: Send`).
pub(crate) fn is_main_thread_bound_return(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(p) => p.path.segments.last().is_some_and(|seg| {
            if MAIN_THREAD_BOUND.contains(&seg.ident.to_string().as_str()) {
                return true;
            }
            if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                return ab.args.iter().any(|arg| {
                    matches!(arg, syn::GenericArgument::Type(t) if is_main_thread_bound_return(t))
                });
            }
            false
        }),
        syn::Type::Reference(r) => is_main_thread_bound_return(&r.elem),
        syn::Type::Paren(p) => is_main_thread_bound_return(&p.elem),
        syn::Type::Group(g) => is_main_thread_bound_return(&g.elem),
        syn::Type::Tuple(t) => t.elems.iter().any(is_main_thread_bound_return),
        syn::Type::Array(a) => is_main_thread_bound_return(&a.elem),
        syn::Type::Slice(s) => is_main_thread_bound_return(&s.elem),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::is_main_thread_bound_return;

    fn ty(s: &str) -> syn::Type {
        syn::parse_str(s).unwrap()
    }

    #[test]
    fn main_thread_bound_return_detects_nested_positions() {
        // Bare and fully-qualified paths
        assert!(is_main_thread_bound_return(&ty("BuiltDataFrame")));
        assert!(is_main_thread_bound_return(&ty(
            "miniextendr_api::dataframe::BuiltDataFrame"
        )));
        assert!(is_main_thread_bound_return(&ty("DataFrameShape")));
        assert!(is_main_thread_bound_return(&ty("SEXP")));
        // Nested inside Result / Option / containers / tuples
        assert!(is_main_thread_bound_return(&ty(
            "Result<BuiltDataFrame, String>"
        )));
        assert!(is_main_thread_bound_return(&ty(
            "Result<DataFrameShape, std::string::String>"
        )));
        assert!(is_main_thread_bound_return(&ty("Option<BuiltDataFrame>")));
        assert!(is_main_thread_bound_return(&ty("Vec<BuiltDataFrame>")));
        assert!(is_main_thread_bound_return(&ty("(i32, BuiltDataFrame)")));
        // Send-safe returns stay worker-eligible
        assert!(!is_main_thread_bound_return(&ty("i32")));
        assert!(!is_main_thread_bound_return(&ty(
            "Result<Vec<f64>, String>"
        )));
        assert!(!is_main_thread_bound_return(&ty("ExternalPtr<MyType>")));
        assert!(!is_main_thread_bound_return(&ty("DataFrame")));
    }
}

/// Container family for a `several_ok` parameter, returned by
/// [`classify_several_ok_container`].
#[derive(Debug, Clone)]
pub(crate) enum SeveralOkContainer {
    /// `Vec<T>`
    Vec,
    /// `Box<[T]>`
    BoxedSlice,
    /// `[T; N]` — the `usize` is the fixed array length N
    Array(usize),
    /// `&[T]` or `&mut [T]` — allocate `Vec<T>` then borrow
    BorrowedSlice,
}

/// Classify a `several_ok` parameter type into one of the four container
/// families and extract its inner element type `T`.
///
/// Returns `Some((container, inner_ty))` or `None` if the type is not one of
/// the four accepted container shapes.
pub(crate) fn classify_several_ok_container(
    ty: &syn::Type,
) -> Option<(SeveralOkContainer, &syn::Type)> {
    match ty {
        // Vec<T>
        syn::Type::Path(tp) => {
            let seg = tp.path.segments.last()?;
            if seg.ident == "Vec" {
                let inner = first_type_argument(seg)?;
                return Some((SeveralOkContainer::Vec, inner));
            }
            // Box<[T]>
            if seg.ident == "Box"
                && let syn::PathArguments::AngleBracketed(ab) = &seg.arguments
            {
                for arg in &ab.args {
                    if let syn::GenericArgument::Type(syn::Type::Slice(s)) = arg {
                        return Some((SeveralOkContainer::BoxedSlice, s.elem.as_ref()));
                    }
                }
            }
            None
        }
        // [T; N]
        syn::Type::Array(arr) => {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(n),
                ..
            }) = &arr.len
            {
                let n = n.base10_parse::<usize>().ok()?;
                return Some((SeveralOkContainer::Array(n), arr.elem.as_ref()));
            }
            None
        }
        // &[T] or &mut [T]
        syn::Type::Reference(r) => {
            if let syn::Type::Slice(s) = r.elem.as_ref() {
                return Some((SeveralOkContainer::BorrowedSlice, s.elem.as_ref()));
            }
            None
        }
        _ => None,
    }
}
