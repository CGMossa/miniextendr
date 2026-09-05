//! Test: `serde_error(rename(...))` may not target one of the condition's own
//! slots (`message`, `call`, `kind`).
//!
//! The rename would recreate the reserved-name collision the option exists to
//! avoid, so it is rejected at expansion time instead of at runtime. The option
//! grammar is checked before the error type is looked at, so `E` needs no
//! `Serialize` impl here.

use miniextendr_macros::miniextendr;

struct E;

#[miniextendr(serde_error(rename(detail = "message")))]
fn bad_serde_error_rename_reserved() -> Result<(), E> {
    Ok(())
}

fn main() {}
