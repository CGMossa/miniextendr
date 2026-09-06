//! Compile-pass regression test for #1118.
//!
//! An inherent `#[miniextendr]` impl block and a trait `#[miniextendr]` impl
//! block on the *same* type, each carrying an impl-*level* method-only roxygen
//! tag (`@param` / `@return`) placed before the `#[miniextendr]` attribute.
//! Each block strips its tag and emits a `#[deprecated]` nudge const; before
//! the fix both consts were named `_MINIEXTENDR_IMPL_METHOD_TAG_WARN_<Type>_0`
//! and collided with `error[E0428]: the name ... is defined multiple times`.
//! Each nudge is now wrapped in its own anonymous `const _: () = { .. };`
//! scope, so the inner const names can stay fixed — an anonymous const block
//! is a fresh item namespace every expansion — and this compiles without any
//! per-block disambiguator.
//!
//! (Two *unlabeled* inherent impl blocks on one type is not a valid
//! reproduction: they also collide on `R_WRAPPERS_IMPL_<TYPE>`, which is
//! unrelated to the tag-const bug. Two *labeled* inherent blocks compile
//! since #1242 — see `impl_labeled_blocks_class_name_no_collision.rs`.
//! Inherent + trait is the real-world case exercised here.)

#![allow(dead_code)]

use miniextendr_api::{ExternalPtr, miniextendr};

#[miniextendr]
pub trait DualCounter {
    fn value(&self) -> i32;
}

#[derive(ExternalPtr)]
pub struct DualTagged {
    value: i32,
}

/// Constructor-side operations.
///
/// @param initial this belongs on a method, not the impl block
#[miniextendr(env)]
impl DualTagged {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
    fn get_value(&self) -> i32 {
        self.value
    }
}

/// Trait-side operations.
///
/// @return this belongs on a method, not the impl block
#[miniextendr(env)]
impl DualCounter for DualTagged {
    fn value(&self) -> i32 {
        self.value
    }
}

fn main() {}
