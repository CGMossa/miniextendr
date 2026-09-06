//! Test: the bare `#[miniextendr(serde_error)]` flag is rejected.
//!
//! Under the API crate's `serde` feature every `Result<T, E>` whose
//! `E: Serialize + Display` is already classed from its serde shape, so the
//! flag would switch nothing on. The attribute only carries options
//! (`serde_error(tag = "..", prefix = "..", skip(..), rename(..))`); the bare
//! form and `serde_error = true/false` point there instead of being accepted
//! as no-ops. The option grammar is checked before the error type is looked
//! at, so `E` needs no `Serialize` impl here.

use miniextendr_macros::miniextendr;

struct E;

#[miniextendr(serde_error)]
fn bad_serde_error_bare() -> Result<(), E> {
    Ok(())
}

#[miniextendr(serde_error = false)]
fn bad_serde_error_off() -> Result<(), E> {
    Ok(())
}

fn main() {}
