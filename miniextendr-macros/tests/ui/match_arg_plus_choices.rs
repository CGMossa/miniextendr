//! Test: `match_arg` and `choices()` on the same parameter should error.
//!
//! `match_arg` resolves its choice list at write time from the parameter type's
//! `MatchArg` impl; `choices()` supplies a literal list for a string parameter.
//! Combined, the `match_arg` placeholder shadowed the literal default and, with
//! no `MatchArg` entry to resolve it, dangled in the generated R formals as an
//! undefined object.

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_both(#[miniextendr(match_arg, choices("fast", "slow"))] mode: String) -> String {
    mode
}

fn main() {}
