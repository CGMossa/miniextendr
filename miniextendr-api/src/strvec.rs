//! Thin wrapper around R character vector (`STRSXP`).
//!
//! Provides safe construction and element insertion for string vectors.

use std::borrow::Cow;
use std::marker::PhantomData;

use crate::SEXPTYPE::STRSXP;
use crate::from_r::{
    SexpError, SexpTypeError, TryFromSexp, charsxp_to_borrowed_str, charsxp_to_cow,
};
use crate::gc_protect::{OwnedProtect, ProtectScope, Protected};
use crate::into_r::IntoR;
use crate::{SEXP, SexpExt};

/// Borrowed view over an R character vector (`STRSXP`).
///
/// The `'a` lifetime leashes every `&str` this view hands out to the window in
/// which the underlying `STRSXP` stays GC-reachable. At a `#[miniextendr]`
/// boundary, write `StrVec<'_>`: R protects `.Call` arguments for the call's
/// duration, so the elided lifetime is bounded by the call and the borrows
/// cannot escape into an `ExternalPtr`, a global, or another thread — doing so
/// is a compile error (the `&str` would have to outlive `'a`). Callers that own
/// the rooting obligation reach for the `unsafe` `*_static` family instead.
///
/// `Copy` and cheap (a single `SEXP`); covariant in `'a`.
#[derive(Clone, Copy, Debug)]
pub struct StrVec<'a>(SEXP, PhantomData<&'a str>);

impl<'a> StrVec<'a> {
    /// Wrap an existing `STRSXP` without additional checks.
    ///
    /// # Safety
    ///
    /// Caller must ensure `sexp` is a valid character vector (`STRSXP`)
    /// that stays GC-reachable for the inferred lifetime `'a`.
    #[inline]
    pub const unsafe fn from_raw(sexp: SEXP) -> Self {
        StrVec(sexp, PhantomData)
    }

    /// Get the underlying `SEXP`.
    #[inline]
    pub const fn as_sexp(self) -> SEXP {
        self.0
    }

    /// Length of the character vector (number of elements).
    #[inline]
    pub fn len(self) -> isize {
        self.0.xlength()
    }

    /// Returns true if the vector is empty.
    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Get the CHARSXP at the given index.
    ///
    /// Returns `None` if out of bounds.
    #[inline]
    pub fn get_charsxp(self, idx: isize) -> Option<SEXP> {
        if idx < 0 || idx >= self.len() {
            return None;
        }
        Some(self.0.string_elt(idx))
    }

    /// Get the string at the given index (zero-copy), leashed to `'a`.
    ///
    /// Returns `None` if out of bounds or if the element is `NA_character_`.
    /// Panics if the CHARSXP is not UTF-8/ASCII. A UTF-8 process locale can
    /// still contain explicitly tagged Latin-1 strings; use
    /// [`get_cow`](Self::get_cow) when those must be translated.
    #[inline]
    pub fn get_str(self, idx: isize) -> Option<&'a str> {
        let charsxp = self.get_charsxp(idx)?;
        unsafe {
            if charsxp == SEXP::na_string() {
                return None;
            }
            // charsxp_to_borrowed_str fabricates &'static; covariance narrows it to &'a.
            Some(charsxp_to_borrowed_str(charsxp))
        }
    }

    /// Get the string at the given index as `Cow<str>` (encoding-safe), leashed to `'a`.
    ///
    /// Returns `Cow::Borrowed` for UTF-8 strings (zero-copy), `Cow::Owned` for
    /// non-UTF-8 strings (translated via `Rf_translateCharUTF8`).
    /// Returns `None` if out of bounds or `NA_character_`.
    #[inline]
    pub fn get_cow(self, idx: isize) -> Option<Cow<'a, str>> {
        let charsxp = self.get_charsxp(idx)?;
        unsafe {
            if charsxp == SEXP::na_string() {
                return None;
            }
            Some(charsxp_to_cow(charsxp))
        }
    }

    /// Get the string at the given index as `&'static str` — the courageous escape.
    ///
    /// Unlike [`get_str`](Self::get_str), the returned reference is **not** leashed
    /// to `'a`, so it can be stored in an `ExternalPtr`, a global, or sent across
    /// threads. The string data lives in R's CHARSXP; it is only valid while that
    /// `STRSXP` stays GC-reachable.
    ///
    /// # Safety
    ///
    /// The caller takes on the rooting obligation: the underlying `STRSXP` must be
    /// kept reachable by R (e.g. via the `prot` slot of the `ExternalPtr` it is
    /// stored in, or `R_PreserveObject`) for as long as the returned `&'static str`
    /// is used. Letting R GC the source is use-after-free.
    #[inline]
    pub unsafe fn get_str_static(self, idx: isize) -> Option<&'static str> {
        let charsxp = self.get_charsxp(idx)?;
        unsafe {
            if charsxp == SEXP::na_string() {
                return None;
            }
            Some(charsxp_to_borrowed_str(charsxp))
        }
    }

    /// Get the string at the given index as `Cow<'static, str>` — the courageous escape.
    ///
    /// `Cow` analogue of [`get_str_static`](Self::get_str_static); same rooting
    /// obligation for the borrowed (`Cow::Borrowed`) case.
    ///
    /// # Safety
    ///
    /// See [`get_str_static`](Self::get_str_static).
    #[inline]
    pub unsafe fn get_cow_static(self, idx: isize) -> Option<Cow<'static, str>> {
        let charsxp = self.get_charsxp(idx)?;
        unsafe {
            if charsxp == SEXP::na_string() {
                return None;
            }
            Some(charsxp_to_cow(charsxp))
        }
    }

    /// Iterate over UTF-8/ASCII elements as `Option<&str>` (leashed to `'a`).
    ///
    /// `NA_character_` elements yield `None`; UTF-8/ASCII strings yield
    /// `Some(&str)`. Panics on other encodings; use [`iter_cow`](Self::iter_cow)
    /// to translate them.
    #[inline]
    pub fn iter(self) -> StrVecIter<'a> {
        StrVecIter {
            vec: self,
            idx: 0,
            len: self.len(),
        }
    }

    /// Iterate over elements as `Option<Cow<str>>` (encoding-safe, leashed to `'a`).
    ///
    /// Like [`iter`](Self::iter) but handles non-UTF-8 CHARSXPs gracefully.
    #[inline]
    pub fn iter_cow(self) -> StrVecCowIter<'a> {
        StrVecCowIter {
            vec: self,
            idx: 0,
            len: self.len(),
        }
    }

    // region: Safe element insertion

    /// Set a CHARSXP at the given index, protecting it during insertion.
    ///
    /// This is the safe way to insert a freshly allocated CHARSXP into a string vector.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `charsxp` must be a valid CHARSXP (from `Rf_mkChar*` or `STRING_ELT`)
    /// - `self` must be a valid, protected STRSXP
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    #[inline]
    pub unsafe fn set_charsxp(self, idx: isize, charsxp: SEXP) {
        assert!(idx >= 0 && idx < self.len(), "index out of bounds");
        // SAFETY: caller guarantees R main thread and valid SEXPs
        unsafe {
            // Protect CHARSXP during SET_STRING_ELT.
            // Note: Rf_mkCharLenCE returns a CHARSXP that may be from the global
            // CHARSXP cache, but protection is still needed for newly allocated ones.
            let _guard = OwnedProtect::new(charsxp);
            self.0.set_string_elt(idx, charsxp);
        }
    }

    /// Set a CHARSXP without protecting it.
    ///
    /// # Safety
    ///
    /// In addition to the safety requirements of [`set_charsxp`](Self::set_charsxp):
    /// - The caller must ensure `charsxp` is already protected or from the
    ///   global CHARSXP cache.
    #[inline]
    pub unsafe fn set_charsxp_unchecked(self, idx: isize, charsxp: SEXP) {
        debug_assert!(idx >= 0 && idx < self.len(), "index out of bounds");
        // SAFETY: caller guarantees charsxp is protected/cached
        self.0.set_string_elt(idx, charsxp);
    }

    /// Set an element from a Rust string.
    ///
    /// Creates a CHARSXP from the string and inserts it safely.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `self` must be a valid, protected STRSXP
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    #[inline]
    pub unsafe fn set_str(self, idx: isize, s: &str) {
        assert!(idx >= 0 && idx < self.len(), "index out of bounds");
        // SAFETY: caller guarantees R main thread
        unsafe {
            let charsxp = SEXP::charsxp(s);
            // CHARSXP may be cached, but protect anyway for safety
            let _guard = OwnedProtect::new(charsxp);
            self.0.set_string_elt(idx, charsxp);
        }
    }

    /// Set an element to `NA_character_`.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `self` must be a valid, protected STRSXP
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    #[inline]
    pub unsafe fn set_na(self, idx: isize) {
        assert!(idx >= 0 && idx < self.len(), "index out of bounds");
        // R_NaString is a global constant, no protection needed
        self.0.set_string_elt(idx, SEXP::na_string());
    }

    /// Set an element from an optional string.
    ///
    /// `None` becomes `NA_character_`.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `self` must be a valid, protected STRSXP
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    #[inline]
    pub unsafe fn set_opt_str(self, idx: isize, s: Option<&str>) {
        match s {
            Some(s) => unsafe { self.set_str(idx, s) },
            None => unsafe { self.set_na(idx) },
        }
    }
    // endregion
}

// region: StrVec iterators

/// Iterator over `StrVec` elements as `Option<&str>` (leashed to `'a`).
///
/// Yields `None` for `NA_character_`, `Some(&str)` for valid strings.
/// Zero-copy — each `&str` borrows directly from R's CHARSXP.
pub struct StrVecIter<'a> {
    vec: StrVec<'a>,
    idx: isize,
    len: isize,
}

impl<'a> Iterator for StrVecIter<'a> {
    type Item = Option<&'a str>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }
        let charsxp = self.vec.0.string_elt(self.idx);
        self.idx += 1;
        if charsxp == SEXP::na_string() {
            Some(None)
        } else {
            Some(Some(unsafe { charsxp_to_borrowed_str(charsxp) }))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.len - self.idx) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for StrVecIter<'_> {}

/// Iterator over `StrVec` elements as `Option<Cow<'a, str>>`.
///
/// Like [`StrVecIter`] but handles non-UTF-8 CHARSXPs via `Rf_translateCharUTF8`.
pub struct StrVecCowIter<'a> {
    vec: StrVec<'a>,
    idx: isize,
    len: isize,
}

impl<'a> Iterator for StrVecCowIter<'a> {
    type Item = Option<Cow<'a, str>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }
        let charsxp = self.vec.0.string_elt(self.idx);
        self.idx += 1;
        if charsxp == SEXP::na_string() {
            Some(None)
        } else {
            Some(Some(unsafe { charsxp_to_cow(charsxp) }))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.len - self.idx) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for StrVecCowIter<'_> {}

impl<'a> IntoIterator for StrVec<'a> {
    type Item = Option<&'a str>;
    type IntoIter = StrVecIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// endregion

// region: StrVecBuilder - efficient batch string vector construction

/// Builder for constructing string vectors with efficient protection management.
///
/// # Example
///
/// ```ignore
/// unsafe fn build_strvec(strings: &[&str]) -> SEXP {
///     let scope = ProtectScope::new();
///     let builder = StrVecBuilder::new(&scope, strings.len() as isize);
///
///     for (i, s) in strings.iter().enumerate() {
///         builder.set_str(i as isize, s);
///     }
///
///     builder.into_sexp()
/// }
/// ```
pub struct StrVecBuilder<'a> {
    vec: SEXP,
    _scope: &'a ProtectScope,
}

impl<'a> StrVecBuilder<'a> {
    /// Create a new string vector builder with the given length.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn new(scope: &'a ProtectScope, len: usize) -> Self {
        // SAFETY: caller guarantees R main thread
        let vec = unsafe { scope.alloc_character(len).into_raw() };
        Self { vec, _scope: scope }
    }

    /// Create a new string vector builder via the **unchecked** FFI allocation path.
    ///
    /// `_unchecked` twin of [`new`](Self::new): the STRSXP is allocated with
    /// `Rf_allocVector_unchecked` (see [`ProtectScope::alloc_character_unchecked`]),
    /// bypassing the main-thread assertion. Use inside ALTREP callbacks,
    /// `with_r_unwind_protect`, or `with_r_thread` bodies, and pair element
    /// insertion with the `_unchecked` string-element setters.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread, in a context where the checked-FFI
    /// assertion is intentionally bypassed (see CLAUDE.md "FFI thread checking").
    #[inline]
    pub unsafe fn new_unchecked(scope: &'a ProtectScope, len: usize) -> Self {
        // SAFETY: caller guarantees R main thread in a checked-bypass context.
        let vec = unsafe { scope.alloc_character_unchecked(len).into_raw() };
        Self { vec, _scope: scope }
    }

    /// Set an element from a Rust string.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn set_str(&self, idx: isize, s: &str) {
        debug_assert!(idx >= 0 && idx < self.vec.xlength());
        let charsxp = SEXP::charsxp(s);
        self.vec.set_string_elt(idx, charsxp);
    }

    /// Set an element to `NA_character_`.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn set_na(&self, idx: isize) {
        debug_assert!(idx >= 0 && idx < self.vec.xlength());
        self.vec.set_string_elt(idx, SEXP::na_string());
    }

    /// Set an element from an optional string.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn set_opt_str(&self, idx: isize, s: Option<&str>) {
        match s {
            // SAFETY: caller guarantees R main thread
            Some(s) => unsafe { self.set_str(idx, s) },
            None => unsafe { self.set_na(idx) },
        }
    }

    /// Get the underlying SEXP.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.vec
    }

    /// Convert to a `StrVec` view, leashed to the builder's protect scope `'a`.
    #[inline]
    pub fn into_strvec(self) -> StrVec<'a> {
        StrVec(self.vec, PhantomData)
    }

    /// Convert to the underlying SEXP.
    #[inline]
    pub fn into_sexp(self) -> SEXP {
        self.vec
    }

    /// Get the length.
    #[inline]
    pub fn len(&self) -> isize {
        self.vec.xlength()
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
// endregion

// region: Trait implementations

impl IntoR for StrVec<'_> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.0
    }
}

impl<'a> TryFromSexp for StrVec<'a> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != STRSXP {
            return Err(SexpTypeError {
                expected: STRSXP,
                actual,
            }
            .into());
        }
        Ok(StrVec(sexp, PhantomData))
    }
}
// endregion

// region: ProtectedStrVec — GC-protected string vector with proper lifetimes

/// GC-protected view over an R character vector (`STRSXP`).
///
/// Unlike [`StrVec`] (which is `Copy` and trusts the caller for GC protection),
/// `ProtectedStrVec` wraps a [`Protected<'static, StrVec<'static>>`](crate::gc_protect::Protected)
/// that keeps the STRSXP alive — the `'static` on the inner view is sound here
/// because the protect guard *is* the rooting obligation. All borrowed data
/// (`&str`, iterators) it hands out has its lifetime tied to `&self`, not
/// `'static` — preventing use-after-GC bugs at compile time.
///
/// # When to use
///
/// - **`StrVec`**: for SEXP arguments to `.Call` (R protects them), or when you
///   manage protection yourself. Lightweight, `Copy`.
/// - **`ProtectedStrVec`**: when you allocate or receive an STRSXP and need to
///   keep it alive beyond the immediate scope. Not `Copy`.
///
/// # Example
///
/// ```ignore
/// #[miniextendr]
/// pub fn count_unique(strings: ProtectedStrVec) -> i32 {
///     let unique: HashSet<&str> = strings.iter()
///         .filter_map(|s| s)
///         .collect();
///     unique.len() as i32
/// }
/// ```
pub struct ProtectedStrVec {
    protected: Protected<'static, StrVec<'static>>,
    len: isize,
}

impl ProtectedStrVec {
    /// Create a protected view over an STRSXP.
    ///
    /// Calls `Rf_protect` on the SEXP. Use [`from_sexp_trusted`](Self::from_sexp_trusted)
    /// when the SEXP is already protected (e.g., `.Call` arguments) to avoid
    /// double-protecting.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid STRSXP.
    /// - Must be called from the R main thread.
    #[inline]
    pub unsafe fn new(sexp: SEXP) -> Self {
        let inner = unsafe { StrVec::from_raw(sexp) };
        let len = inner.len();
        Self {
            protected: unsafe { Protected::new(sexp, inner) },
            len,
        }
    }

    /// Create a view without adding GC protection.
    ///
    /// Use this when the SEXP is already protected by R (e.g., a `.Call`
    /// argument, or in a `ProtectScope`). Avoids the redundant
    /// `Rf_protect`/`Rf_unprotect` pair.
    ///
    /// The lifetime-bound `&str` borrows are still enforced — this only
    /// skips the protect stack push, not the safety guarantees.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid STRSXP.
    /// - `sexp` must remain GC-protected for the lifetime of this struct.
    /// - Must be called from the R main thread.
    #[inline]
    pub unsafe fn from_sexp_trusted(sexp: SEXP) -> Self {
        let inner = unsafe { StrVec::from_raw(sexp) };
        let len = inner.len();
        Self {
            protected: unsafe { Protected::from_trusted(sexp, inner) },
            len,
        }
    }

    /// Number of elements.
    #[inline]
    pub fn len(&self) -> isize {
        self.len
    }

    /// Whether the vector is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the string at index (zero-copy, lifetime tied to `&self`).
    ///
    /// Returns `None` for out-of-bounds or `NA_character_`.
    #[inline]
    pub fn get_str(&self, idx: isize) -> Option<&str> {
        // charsxp_to_borrowed_str returns &'static str, but lifetime elision
        // restricts it to &'_ (tied to &self) — correct: data lives
        // as long as the Protected guard keeps the STRSXP alive.
        self.protected.get().get_str(idx)
    }

    /// Get the string at index as `Cow<str>` (encoding-safe, lifetime tied to `&self`).
    #[inline]
    pub fn get_cow(&self, idx: isize) -> Option<Cow<'_, str>> {
        self.protected.get().get_cow(idx)
    }

    /// Iterate over elements as `Option<&str>` (lifetime tied to `&self`).
    #[inline]
    pub fn iter(&self) -> ProtectedStrVecIter<'_> {
        ProtectedStrVecIter {
            vec: self,
            idx: 0,
            len: self.len,
        }
    }

    /// Iterate over elements as `Option<Cow<str>>` (encoding-safe).
    #[inline]
    pub fn iter_cow(&self) -> ProtectedStrVecCowIter<'_> {
        ProtectedStrVecCowIter {
            vec: self,
            idx: 0,
            len: self.len,
        }
    }

    /// Get the underlying SEXP (still protected by this handle).
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.protected.get().as_sexp()
    }

    /// Get the inner `StrVec` view, leashed to `&self` (the protect guard).
    #[inline]
    pub fn as_strvec(&self) -> StrVec<'_> {
        *self.protected.get()
    }
}

/// Iterator over `ProtectedStrVec` with lifetime tied to the protection guard.
pub struct ProtectedStrVecIter<'a> {
    vec: &'a ProtectedStrVec,
    idx: isize,
    len: isize,
}

impl<'a> Iterator for ProtectedStrVecIter<'a> {
    type Item = Option<&'a str>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }
        let result = self.vec.get_str(self.idx);
        self.idx += 1;
        // get_str returns None for NA; we need to distinguish "end of iter" from "NA element"
        // Wrap: Some(None) = NA, Some(Some(&str)) = value, None = end
        Some(result)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.len - self.idx) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for ProtectedStrVecIter<'_> {}

/// Encoding-safe iterator over `ProtectedStrVec`.
pub struct ProtectedStrVecCowIter<'a> {
    vec: &'a ProtectedStrVec,
    idx: isize,
    len: isize,
}

impl<'a> Iterator for ProtectedStrVecCowIter<'a> {
    type Item = Option<Cow<'a, str>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }
        let result = self.vec.get_cow(self.idx);
        self.idx += 1;
        Some(result)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.len - self.idx) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for ProtectedStrVecCowIter<'_> {}

impl<'a> IntoIterator for &'a ProtectedStrVec {
    type Item = Option<&'a str>;
    type IntoIter = ProtectedStrVecIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoR for ProtectedStrVec {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.as_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(self.as_sexp())
    }
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.as_sexp()
    }
}

impl TryFromSexp for ProtectedStrVec {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != STRSXP {
            return Err(SexpTypeError {
                expected: STRSXP,
                actual,
            }
            .into());
        }
        // Use from_sexp_trusted: TryFromSexp is called from generated .Call
        // wrappers where R already protects the argument. No need to double-protect.
        Ok(unsafe { ProtectedStrVec::from_sexp_trusted(sexp) })
    }
}

impl std::fmt::Debug for ProtectedStrVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProtectedStrVec")
            .field("len", &self.len)
            .finish()
    }
}
// endregion
