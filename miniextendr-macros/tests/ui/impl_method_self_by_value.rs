//! Test: a smart-pointer `self` receiver is rejected (the R handle stores the
//! value itself, so only `self`, `&self`, `&mut self`, or `ExternalPtr<Self>`
//! receivers can be handed over). Bare `self` (consuming) is supported since
//! #1432 and is covered by the pass tests / rpkg fixtures instead.

use miniextendr_macros::miniextendr;

struct Counter(i32);

#[miniextendr]
impl Counter {
    fn consume(self: Box<Self>) -> i32 {
        self.0
    }
}

fn main() {}
