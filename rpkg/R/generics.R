# Documentation stubs for S3/S4/S7 generics exported by test types.
# These generics are defined conditionally in miniextendr-wrappers.R (auto-generated)
# and need an alias here so R CMD check finds their documentation.
# Exports are already registered by miniextendr-wrappers.R; this file only
# provides the man-page aliases that R CMD check requires.

#' Convert object to data frame
#' @param x An object.
#' @param ... Additional arguments.
#' @name as_data_frame
NULL

#' Convert object to list
#' @param x An object.
#' @param ... Additional arguments.
#' @name as_list
NULL

#' Convert object to character
#' @param x An object.
#' @param ... Additional arguments.
#' @name as_character
NULL

#' Convert object to numeric
#' @param x An object.
#' @param ... Additional arguments.
#' @name as_numeric
NULL

#' Convert object to integer
#' @param x An object.
#' @param ... Additional arguments.
#' @name as_integer
NULL

#' Get length
#' @param x An object.
#' @param ... Additional arguments.
#' @name len
NULL

#' Get value
#' @param x An object.
#' @param ... Additional arguments.
#' @name get_value
NULL

#' Get label
#' @param x An object.
#' @param ... Additional arguments.
#' @name label
NULL

#' Relabel an object
#' @param x An object.
#' @param ... Additional arguments.
#' @name relabel
NULL

#' Add via S3 dispatch
#' @param x An object.
#' @param ... Additional arguments.
#' @name s3_add
NULL

#' Increment via S3 dispatch
#' @param x An object.
#' @param ... Additional arguments.
#' @name s3_inc
NULL

#' Get S3 value
#' @param x An object.
#' @param ... Additional arguments.
#' @name s3_value
NULL

#' Return the constructor dots count via S3 dispatch
#' @param x An object.
#' @param ... Additional arguments.
#' @name impl_dots_s3_ctor_dots
NULL

#' Add a value and the number of method dots via S3 dispatch
#' @param x An object.
#' @param ... Additional arguments.
#' @name impl_dots_s3_add_with_dots
NULL

#' Get current mode (S7)
#' @param x An object.
#' @param ... Additional arguments.
#' @name current
NULL

#' Set mode (S7)
#' @param x An object.
#' @param ... Additional arguments.
#' @name set
NULL

#' Get current mode (S4)
#' @param x An object.
#' @param ... Additional arguments.
#' @name s4_mode_current
NULL

#' Set mode (S4)
#' @param x An object.
#' @param ... Additional arguments.
#' @name s4_mode_set
NULL

# Pipe-builder fixture generics (GreetingBuilder, PipeCounter). These S3
# generics are exported by the auto-generated wrappers under @rdname of their
# class, so they need a bare-name alias here for R CMD check.

#' Set the name to greet
#' @param x An object.
#' @param ... Additional arguments.
#' @name set_name
NULL

#' Set the trailing punctuation
#' @param x An object.
#' @param ... Additional arguments.
#' @name set_punctuation
NULL

#' Toggle whether the greeting is shouted
#' @param x An object.
#' @param ... Additional arguments.
#' @name set_loud
NULL

#' Render the configured value
#' @param x An object.
#' @param ... Additional arguments.
#' @name build
NULL

#' Add to a counter in place
#' @param x An object.
#' @param ... Additional arguments.
#' @name bump
NULL

#' Double a counter in place
#' @param x An object.
#' @param ... Additional arguments.
#' @name twice
NULL

#' Read the current value
#' @param x An object.
#' @param ... Additional arguments.
#' @name peek
NULL

# ConsumingBuilder fixture generics (consuming `self` receivers and fallible
# in-place builders). Same pattern as above: exported by the auto-generated
# wrappers under @rdname ConsumingBuilder, aliased here for R CMD check.

#' Add an amount, consuming and returning the builder
#' @param x An object.
#' @param ... Additional arguments.
#' @name with_amount
NULL

#' Add an amount, rejecting negative values
#' @param x An object.
#' @param ... Additional arguments.
#' @name try_amount
NULL

#' Add an amount, returning NULL past the cap
#' @param x An object.
#' @param ... Additional arguments.
#' @name maybe_amount
NULL

#' Add an amount in place, rejecting negative values
#' @param x An object.
#' @param ... Additional arguments.
#' @name checked_bump
NULL

#' Add an amount in place, returning NULL past the cap
#' @param x An object.
#' @param ... Additional arguments.
#' @name maybe_bump
NULL

#' Read the running total
#' @param x An object.
#' @param ... Additional arguments.
#' @name total
NULL

#' Consume the builder and return its total
#' @param x An object.
#' @param ... Additional arguments.
#' @name finish
NULL

# ClassedChecker fixture generic (classed `Result` errors). Exported by the
# auto-generated wrappers under @rdname ClassedChecker, aliased here for
# R CMD check.

#' Check a value against the bound, raising a classed error past it
#' @param x An object.
#' @param ... Additional arguments.
#' @name check_bound
NULL

# SerdeChecker fixture generic (`#[miniextendr(serde_error)]` on an S3
# method). Exported by the auto-generated wrappers under @rdname SerdeChecker,
# aliased here for R CMD check.

#' Check a value against the bound, raising a serde-derived classed error past it
#' @param x An object.
#' @param ... Additional arguments.
#' @name check_value
NULL

# SerdeChecker fixture generic (`serde_error(skip(...))` on an S3 method,
# #1457). Exported by the auto-generated wrappers under @rdname SerdeChecker,
# aliased here for R CMD check.

#' Parse text as a number, raising a classed error without the parser's `message` field
#' @param x An object.
#' @param ... Additional arguments.
#' @name parse_value
NULL
