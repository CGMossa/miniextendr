//! Test: `serde_error(skip = "...")` is the wrong shape; `skip` takes a
//! parenthesised list of field names so several can be dropped at once. The
//! option grammar is checked before the error type is looked at, so `E` needs
//! no `Serialize` impl here.

use miniextendr_macros::miniextendr;

struct E;

#[miniextendr(serde_error(skip = "message"))]
fn bad_serde_error_skip_not_list() -> Result<(), E> {
    Ok(())
}

fn main() {}
