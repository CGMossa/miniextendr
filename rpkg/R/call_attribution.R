# Hand-written delegates for the `call = caller` fixture pair in
# src/rust/call_attribution_demo.rs (docs/CALL_ATTRIBUTION.md, #1450). Both are
# package-internal; tests reach them through `:::`.

# Delegates to a `#[miniextendr(noexport, call = caller)]` entry point: a Rust
# error surfaces as `Error in call_attr_caller(value = -1L)`.
call_attr_caller <- function(value) {
  call_attr_caller_impl(value)
}

# Delegates to a default `noexport` entry point: the same error surfaces as
# `Error in call_attr_self_impl(x = value)`, naming the bridge.
call_attr_self <- function(value) {
  call_attr_self_impl(value)
}
