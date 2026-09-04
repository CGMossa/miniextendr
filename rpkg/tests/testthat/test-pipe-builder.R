# Tests for functional (native-pipe) builder support.
#
# `&mut self -> &mut Self` builder methods on an `#[miniextendr(s3)]` impl
# generate pipe-friendly S3 free functions: the object is the first argument
# and the (same, mutated) object is returned, so the methods compose under R's
# native pipe operator `|>`. The generated S3 generic is named after the Rust
# method (e.g. `set_name`), dispatching on the object's class.

test_that("GreetingBuilder chains under |> and build() returns a String", {
  result <- new_greetingbuilder() |>
    set_name("World") |>
    set_punctuation("!") |>
    build()
  expect_equal(result, "Hello, World!")

  loud <- new_greetingbuilder() |>
    set_name("World") |>
    set_loud(TRUE) |>
    build()
  expect_equal(loud, "HELLO, WORLD.")

  # Defaults: empty name -> "world", default punctuation "."
  expect_equal(build(new_greetingbuilder()), "Hello, world.")
})

test_that("self-returning builder steps preserve object identity (in-place, no clone)", {
  b <- new_greetingbuilder()
  # Each step returns the SAME ExternalPtr handle wrapped in the same S3 object.
  out <- set_name(b, "Ada")
  expect_identical(out, b)
  expect_s3_class(out, "GreetingBuilder")
  # The mutation is visible through the original handle: building from `b`
  # after mutating `out` (same object) reflects the change.
  expect_equal(build(b), "Hello, Ada.")
})

test_that("PipeCounter mutates in place across a |> chain", {
  ctr <- new_pipecounter(1L) |>
    bump(4L) |>
    twice() |>   # (1 + 4) * 2 = 10
    bump(5L)     # 10 + 5 = 15
  expect_s3_class(ctr, "PipeCounter")
  expect_equal(peek(ctr), 15L)
})

test_that("PipeCounter self-ref steps return the same object", {
  ctr <- new_pipecounter(0L)
  expect_identical(bump(ctr, 3L), ctr)
  expect_identical(twice(ctr), ctr)
  # After bump(0, 3) = 3 then twice -> 6
  expect_equal(peek(ctr), 6L)
})

test_that("pipe-builder generics and methods are exported", {
  exports <- getNamespaceExports("miniextendr")
  for (fn in c(
    "new_greetingbuilder", "set_name", "set_punctuation",
    "set_loud", "build", "new_pipecounter", "bump",
    "twice", "peek"
  )) {
    expect_true(fn %in% exports, info = sprintf("`%s` missing from exports", fn))
  }
})

# ---------------------------------------------------------------------------
# Cross-class-system coverage for self-ref builders (#769)
#
# A `&mut self -> &mut Self` builder step plus a terminal accessor must chain
# on every impl-block class system, and must preserve object identity wherever
# the system is reference-semantic. R6/Env chain via `invisible(self)`;
# S4/S7 chain by returning the receiver `x` from the generated generic. The
# critical R6 guarantee: chaining must NOT mint a new R6 wrapper around the same
# pointer (that would break identity) — it returns the *same* environment.
# ---------------------------------------------------------------------------

test_that("R6PipeBuilder chains via invisible(self) and preserves identity", {
  b <- R6PipeBuilder$new()
  # `$add()` returns the same R6 object (invisible(self)), so we can chain and
  # the chain reads through the same wrapper.
  expect_equal(b$add(1L)$add(2L)$total(), 3L)

  # Identity: the value returned by a builder step IS the same R6 environment,
  # not a freshly minted wrapper around the same pointer.
  b2 <- R6PipeBuilder$new()
  stepped <- b2$add(5L)
  expect_identical(stepped, b2)
  # The mutation is visible through the original handle.
  expect_equal(b2$total(), 5L)
})

test_that("S4PipeBuilder chains under |> and preserves identity", {
  total <- miniextendr:::S4PipeBuilder() |>
    miniextendr:::s4_add(1L) |>
    miniextendr:::s4_add(2L) |>
    miniextendr:::s4_total()
  expect_equal(total, 3L)

  # Identity: the self-ref step returns the same S4 object (same ExternalPtr).
  b <- miniextendr:::S4PipeBuilder()
  stepped <- miniextendr:::s4_add(b, 5L)
  expect_identical(stepped, b)
  expect_equal(miniextendr:::s4_total(b), 5L)
})

test_that("S7PipeBuilder chains under |> and preserves identity", {
  total <- miniextendr:::S7PipeBuilder() |>
    miniextendr:::s7_add(1L) |>
    miniextendr:::s7_add(2L) |>
    miniextendr:::s7_total()
  expect_equal(total, 3L)

  # Identity: the self-ref step returns the same S7 object (same ExternalPtr).
  b <- miniextendr:::S7PipeBuilder()
  stepped <- miniextendr:::s7_add(b, 5L)
  expect_identical(stepped, b)
  expect_equal(miniextendr:::s7_total(b), 5L)
})

test_that("R6 builder build() wraps a different returned class", {
  plan <- R6CrossPlan$new(7L)

  board <- plan$build(4L, 5L)
  expect_true(inherits(board, "R6CrossBoard"))
  expect_equal(board$cells(), 20L)
  expect_equal(board$signature(), "4x5@7")

  expect_equal(plan$build(2L, 3L)$cells(), 6L)
})

test_that("S7 builder build wraps a different returned class", {
  plan <- S7CrossPlan(3L)

  board <- s7_cross_build(plan, 4L, 5L)
  expect_true(S7::S7_inherits(board, S7CrossBoard))
  expect_equal(s7_cross_cells(board), 23L)

  expect_equal(s7_cross_cells(s7_cross_build(plan, 2L, 3L)), 9L)
})

test_that("R6 method returning an S7 class wraps with the S7 constructor", {
  # Mixed-system return: source method lives on an R6 class, target is S7.
  # The write-time resolver keys off the returned class, so the wrapper must
  # build the S7 object (not R6).
  plan <- R6CrossPlan$new(3L)

  board <- plan$build_s7(4L, 5L)
  expect_true(S7::S7_inherits(board, S7CrossBoard))
  expect_equal(s7_cross_cells(board), 23L)
})

test_that("S7 method returning an R6 class wraps with the R6 constructor", {
  # Mixed-system return in the other direction: S7 source, R6 target.
  plan <- S7CrossPlan(7L)

  board <- s7_build_r6(plan, 4L, 5L)
  expect_true(inherits(board, "R6CrossBoard"))
  expect_equal(board$cells(), 20L)
  expect_equal(board$signature(), "4x5@7")
})

test_that("R6 builder try_build() wraps a usable classed object on Some", {
  plan <- R6CrossPlan$new(7L)

  board <- plan$try_build(4L, 5L, FALSE)
  expect_s3_class(board, "R6CrossBoard")
  expect_equal(board$cells(), 20L)
  expect_equal(board$signature(), "4x5@7")
})

test_that("R6 builder try_build() raises a rust_error on None", {
  plan <- R6CrossPlan$new(7L)

  expect_error(
    plan$try_build(4L, 5L, TRUE),
    class = "rust_error"
  )
})

test_that("R6 builder checked_build() wraps a usable classed object on Ok", {
  plan <- R6CrossPlan$new(3L)

  board <- plan$checked_build(2L, 3L, FALSE)
  expect_s3_class(board, "R6CrossBoard")
  expect_equal(board$cells(), 6L)
  expect_equal(board$signature(), "2x3@3")
})

test_that("R6 builder checked_build() raises with the fixture's message on Err", {
  plan <- R6CrossPlan$new(3L)

  expect_error(
    plan$checked_build(2L, 3L, TRUE),
    "checked_build failed for seed 3",
    fixed = TRUE,
    class = "rust_error"
  )
})

test_that("S7 method returning an Option<R6 class> wraps with the R6 constructor", {
  # Mixed-system container return: proves the target-keyed resolver on the
  # Option<Class> path even though this method lives on an S7 class.
  plan <- S7CrossPlan(7L)

  board <- s7_try_build_r6(plan, 4L, 5L, FALSE)
  expect_s3_class(board, "R6CrossBoard")
  expect_equal(board$cells(), 20L)
  expect_equal(board$signature(), "4x5@7")
})

test_that("S7 method returning an Option<R6 class> raises a rust_error on None", {
  plan <- S7CrossPlan(7L)

  expect_error(
    s7_try_build_r6(plan, 4L, 5L, TRUE),
    class = "rust_error"
  )
})

test_that("R6 builder build_many() wraps every element of a Vec<Class> return", {
  # #1284: Vec<Class> returns arrive as a list of wrapped class instances,
  # not bare external pointers.
  plan <- R6CrossPlan$new(7L)

  boards <- plan$build_many(4L, 5L, 3L)
  expect_type(boards, "list")
  expect_length(boards, 3L)
  for (b in boards) {
    expect_s3_class(b, "R6CrossBoard")
    expect_equal(b$cells(), 20L)
  }
  # Elements are distinct objects (seeds seed, seed + 1, ...).
  expect_equal(
    vapply(boards, function(b) b$signature(), character(1)),
    c("4x5@7", "4x5@8", "4x5@9")
  )
})

test_that("R6 builder build_many() returns an empty list for count = 0", {
  plan <- R6CrossPlan$new(7L)

  boards <- plan$build_many(4L, 5L, 0L)
  expect_identical(boards, list())
})

test_that("R6 method returning Vec<S7 class> wraps elements with the S7 constructor", {
  # Mixed-system list return: source method lives on an R6 class, elements
  # are S7. The write-time lapply resolver keys off the element class.
  plan <- R6CrossPlan$new(3L)

  boards <- plan$build_many_s7(4L, 5L, 2L)
  expect_length(boards, 2L)
  for (b in boards) {
    expect_true(S7::S7_inherits(b, S7CrossBoard))
  }
  expect_equal(
    vapply(boards, function(b) s7_cross_cells(b), integer(1)),
    c(23L, 24L)
  )
})

test_that("S7 method returning Vec<R6 class> wraps elements with the R6 constructor", {
  # Mixed-system list return in the other direction: S7 source, R6 elements.
  plan <- S7CrossPlan(7L)

  boards <- s7_build_many_r6(plan, 4L, 5L, 2L)
  expect_length(boards, 2L)
  for (b in boards) {
    expect_s3_class(b, "R6CrossBoard")
    expect_equal(b$cells(), 20L)
  }
  expect_equal(
    vapply(boards, function(b) b$signature(), character(1)),
    c("4x5@7", "4x5@8")
  )
})

test_that("R6 builder try_build_many() wraps a usable classed list on Some", {
  plan <- R6CrossPlan$new(7L)

  boards <- plan$try_build_many(4L, 5L, 2L, FALSE)
  expect_length(boards, 2L)
  for (b in boards) {
    expect_s3_class(b, "R6CrossBoard")
  }
  expect_equal(boards[[1L]]$signature(), "4x5@7")
})

test_that("R6 builder try_build_many() raises a rust_error on None", {
  plan <- R6CrossPlan$new(7L)

  expect_error(
    plan$try_build_many(4L, 5L, 2L, TRUE),
    class = "rust_error"
  )
})

test_that("R6 builder checked_build_many() wraps a usable classed list on Ok", {
  plan <- R6CrossPlan$new(3L)

  boards <- plan$checked_build_many(2L, 3L, 2L, FALSE)
  expect_length(boards, 2L)
  for (b in boards) {
    expect_s3_class(b, "R6CrossBoard")
    expect_equal(b$cells(), 6L)
  }
})

test_that("R6 builder checked_build_many() raises with the fixture's message on Err", {
  plan <- R6CrossPlan$new(3L)

  expect_error(
    plan$checked_build_many(2L, 3L, 2L, TRUE),
    "checked_build_many failed for seed 3",
    fixed = TRUE,
    class = "rust_error"
  )
})

test_that("EnvPipeBuilder chains via $ and preserves identity", {
  b <- EnvPipeBuilder$new()
  expect_equal(b$add(1L)$add(2L)$total(), 3L)

  # Identity: the self-ref step returns the same environment.
  b2 <- EnvPipeBuilder$new()
  stepped <- b2$add(5L)
  expect_identical(stepped, b2)
  expect_equal(b2$total(), 5L)
})

# ---------------------------------------------------------------------------
# Consuming `self` builders (#1432) and fallible in-place steps (#1433)
# ---------------------------------------------------------------------------

test_that("S3 consuming `self -> Self` steps write back into the same object", {
  b <- new_consumingbuilder()
  out <- with_amount(b, 3L)
  expect_identical(out, b)
  expect_equal(total(b), 3L)
  chained <- new_consumingbuilder() |> with_amount(1L) |> with_amount(2L)
  expect_equal(total(chained), 3L)
})

test_that("S3 consuming `self -> Result<Self, E>` leaves the object untouched on Err", {
  b <- new_consumingbuilder() |> try_amount(5L)
  expect_equal(total(b), 5L)
  expect_identical(try_amount(b, 1L), b)
  expect_error(try_amount(b, -1L), "non-negative", class = "rust_error")
  expect_equal(total(b), 6L)   # 5 + 1, the failed step did not apply
  expect_equal(total(new_consumingbuilder() |> try_amount(2L) |> try_amount(3L)), 5L)
})

test_that("S3 consuming `self -> Option<Self>` raises on None and keeps the value", {
  b <- new_consumingbuilder() |> maybe_amount(60L)
  expect_equal(total(b), 60L)
  expect_error(maybe_amount(b, 50L), "returned no value")
  expect_equal(total(b), 60L)
})

test_that("fallible in-place `&mut self -> Result<&mut Self, E>` returns the same handle", {
  b <- new_consumingbuilder()
  expect_identical(checked_bump(b, 4L), b)
  expect_equal(total(b), 4L)
  expect_error(checked_bump(b, -4L), "non-negative", class = "rust_error")
  expect_equal(total(b), 4L)
  expect_identical(maybe_bump(b, 6L), b)
  expect_equal(total(b), 10L)
  expect_error(maybe_bump(b, 100L), "returned no value")
  expect_equal(total(b), 10L)
})

test_that("a terminal consuming method leaves the handle consumed", {
  b <- new_consumingbuilder() |> with_amount(7L)
  expect_equal(finish(b), 7L)
  e <- tryCatch(total(b), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e), "consumed")
  e <- tryCatch(with_amount(b, 1L), error = function(e) e)
  expect_match(conditionMessage(e), "consumed")
})

test_that("R6 consuming builders chain via invisible(self) and keep identity", {
  b <- R6ConsumingBuilder$new()
  expect_identical(b$with_amount(2L), b)
  expect_equal(b$with_amount(1L)$try_amount(3L)$total(), 6L)
  expect_error(b$try_amount(-1L), "non-negative")
  expect_equal(b$total(), 6L)
  expect_identical(b$checked_bump(4L), b)
  expect_equal(b$total(), 10L)
  expect_equal(b$finish(), 10L)
  expect_error(b$total(), "consumed")
})

test_that("Env consuming builders chain and keep identity", {
  b <- EnvConsumingBuilder$new()
  expect_identical(b$with_amount(2L), b)
  expect_equal(b$with_amount(1L)$try_amount(3L)$total(), 6L)
  expect_error(b$try_amount(-1L), "non-negative")
  expect_equal(b$total(), 6L)
  expect_identical(b$checked_bump(4L), b)
  expect_equal(b$finish(), 10L)
  expect_error(b$total(), "consumed")
})
