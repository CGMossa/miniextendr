#' Public wrapper around a postfixed internal entry point
#'
#' The Rust side is `#[miniextendr(noexport, postfix = "_impl")] pub fn
#' export_control_delegate`, which generates the unexported
#' `export_control_delegate_impl()`. This hand-written function is the
#' documented public surface: it validates its input in R and delegates.
#'
#' @param x A single number.
#' @return `x` doubled, as an integer.
#' @examples
#' export_control_delegate(4)
#' @export
export_control_delegate <- function(x) {
  stopifnot(is.numeric(x), length(x) == 1L)
  export_control_delegate_impl(as.integer(x))
}
