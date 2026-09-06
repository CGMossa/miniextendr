//! Test: two `serde_error(rename(...))` pairs may not share a target.
//!
//! Both fields would be spliced into the condition data under one name, of
//! which R's `e$name` reads only the first (#1459). The clash is visible in the
//! option grammar, so it is rejected at expansion time; a rename onto a name the
//! variant itself carries is only detectable at runtime and panics there.

use miniextendr_macros::miniextendr;

struct E;

#[miniextendr(serde_error(rename(message = "detail", note = "detail")))]
fn bad_serde_error_rename_target_twice() -> Result<(), E> {
    Ok(())
}

fn main() {}
