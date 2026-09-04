//! Fixture for functional (native-pipe) builder support.
//!
//! Demonstrates that an in-place Rust builder — `&mut self -> &mut Self`
//! methods plus a terminal `build(&self) -> T` — maps to pipe-friendly S3 free
//! functions in R, so the idiom
//!
//! ```r
//! greeting <- new_greetingbuilder() |>
//!   builder_set_name("World") |>
//!   builder_set_punctuation("!") |>
//!   builder_set_loud(TRUE)
//! builder_build(greeting)   # "HELLO, WORLD!"
//! ```
//!
//! works end-to-end. Each `&mut self -> &mut Self` step mutates the underlying
//! Rust value in place and the C wrapper returns the *same* ExternalPtr handle
//! (no clone), so the S3 object the user piped in flows unchanged through the
//! chain. The terminal `builder_build()` reads `&self` and returns a `String`,
//! converted to R via the usual `IntoR` path.

use miniextendr_api::miniextendr;

/// A small greeting builder used to exercise native-pipe (`|>`) chaining.
#[derive(miniextendr_api::ExternalPtr)]
pub struct GreetingBuilder {
    name: String,
    punctuation: String,
    loud: bool,
}

/// Builder for a greeting string, demonstrating functional pipe chaining.
///
/// The `builder_set_*` methods return `&mut Self`, so they compose under R's
/// native pipe operator `|>` as free functions taking the object first.
#[allow(clippy::new_without_default)]
#[miniextendr(s3)]
impl GreetingBuilder {
    /// Create a new greeting builder with empty defaults.
    pub fn new() -> Self {
        GreetingBuilder {
            name: String::new(),
            punctuation: String::from("."),
            loud: false,
        }
    }

    /// Set the name to greet. Returns the builder for chaining.
    /// @param name The name to greet.
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    /// Set the trailing punctuation. Returns the builder for chaining.
    /// @param punctuation The trailing punctuation string.
    pub fn set_punctuation(&mut self, punctuation: String) -> &mut Self {
        self.punctuation = punctuation;
        self
    }

    /// Toggle whether the greeting is shouted (upper-cased). Returns the builder.
    /// @param loud Whether to upper-case the greeting.
    pub fn set_loud(&mut self, loud: bool) -> &mut Self {
        self.loud = loud;
        self
    }

    /// Terminal step: render the configured greeting as a string.
    ///
    /// Takes `&self` (not `self`) so the R object remains valid afterwards, and
    /// returns a different type (`String`) converted to R via `IntoR`.
    pub fn build(&self) -> String {
        let name = if self.name.is_empty() {
            "world"
        } else {
            &self.name
        };
        let greeting = format!("Hello, {}{}", name, self.punctuation);
        if self.loud {
            greeting.to_uppercase()
        } else {
            greeting
        }
    }
}

/// An in-place counter demonstrating `&mut self -> &mut Self` on an integer
/// payload (no terminal type-change), so the chain returns the object itself.
#[derive(miniextendr_api::ExternalPtr)]
pub struct PipeCounter {
    value: i32,
}

/// Counter with pipe-friendly mutators returning `&mut Self`.
#[miniextendr(s3)]
impl PipeCounter {
    /// Create a counter starting at the given value.
    /// @param initial Initial counter value.
    pub fn new(initial: i32) -> Self {
        PipeCounter { value: initial }
    }

    /// Add `amount` to the counter in place. Returns the counter for chaining.
    /// @param amount Amount to add.
    pub fn bump(&mut self, amount: i32) -> &mut Self {
        self.value += amount;
        self
    }

    /// Double the counter in place. Returns the counter for chaining.
    pub fn twice(&mut self) -> &mut Self {
        self.value *= 2;
        self
    }

    /// Read the current value (terminal accessor).
    pub fn peek(&self) -> i32 {
        self.value
    }
}

// region: R6 self-ref builder fixture
//
// R6 is reference-semantic: a `&mut self -> &mut Self` builder must chain via
// `invisible(self)` so `obj$step(1)$step(2)$total()` reads through the *same*
// R6 wrapper environment. The codegen must NOT re-wrap via
// `R6PipeBuilder$new(.ptr = .val)` — that would mint a new R6 environment
// around the same pointer and break object identity. See issue #769.

/// A small accumulating builder exposed as an R6 class.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6PipeBuilder {
    total: i32,
}

/// R6 builder with a `&mut self -> &mut Self` step and a terminal accessor.
/// Chains as `b$add(1L)$add(2L)$total()`.
#[miniextendr(r6)]
impl R6PipeBuilder {
    /// Create a new builder starting at zero.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        R6PipeBuilder { total: 0 }
    }

    /// Add `amount` in place and return the builder for chaining.
    /// @param amount Amount to add.
    pub fn add(&mut self, amount: i32) -> &mut Self {
        self.total += amount;
        self
    }

    /// Terminal accessor: read the accumulated total.
    pub fn total(&self) -> i32 {
        self.total
    }
}
// endregion

// region: S4 self-ref builder fixture
//
// S4 self-ref builders generate free generics whose body returns the receiver
// `x` for chaining (S4 `ExternalPtr` objects are reference-semantic). Method
// names are auto-prefixed with `s4_`, so `add` -> `s4_add` (MXL111: never name
// the Rust method `s4_*`).

/// A small accumulating builder exposed as an S4 class.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S4PipeBuilder {
    total: i32,
}

/// S4 builder with a `&mut self -> &mut Self` step and a terminal accessor.
/// Chains under the native pipe as `b |> s4_add(1L) |> s4_add(2L) |> s4_total()`.
/// @aliases s4_add,S4PipeBuilder-method s4_total,S4PipeBuilder-method
#[miniextendr(s4, internal)]
impl S4PipeBuilder {
    /// Create a new builder starting at zero.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        S4PipeBuilder { total: 0 }
    }

    /// Add `amount` in place and return the builder for chaining.
    /// @param amount Amount to add.
    pub fn add(&mut self, amount: i32) -> &mut Self {
        self.total += amount;
        self
    }

    /// Terminal accessor: read the accumulated total.
    pub fn total(&self) -> i32 {
        self.total
    }
}
// endregion

// region: S7 self-ref builder fixture
//
// S7 self-ref builders generate free generics whose body returns the receiver
// `x` for chaining. S7 method names are NOT auto-prefixed, so we name them
// `s7_*` explicitly to keep the user-facing generics in a clean namespace.

/// A small accumulating builder exposed as an S7 class.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S7PipeBuilder {
    total: i32,
}

/// S7 builder with a `&mut self -> &mut Self` step and a terminal accessor.
/// Chains under the native pipe as `b |> s7_add(1L) |> s7_add(2L) |> s7_total()`.
#[miniextendr(s7, internal)]
impl S7PipeBuilder {
    /// Create a new builder starting at zero.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        S7PipeBuilder { total: 0 }
    }

    /// Add `amount` in place and return the builder for chaining.
    /// @param amount Amount to add.
    pub fn s7_add(&mut self, amount: i32) -> &mut Self {
        self.total += amount;
        self
    }

    /// Terminal accessor: read the accumulated total.
    pub fn s7_total(&self) -> i32 {
        self.total
    }
}
// endregion

// region: Cross-class return builder fixtures
//
// Terminal builder methods that materialize a different ExternalPtr-backed
// class should return a ready-to-use R wrapper, not a bare externalptr.

/// Board materialized by `R6CrossPlan::build`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6CrossBoard {
    width: i32,
    height: i32,
    seed: i32,
}

/// R6 board target for cross-class builder returns.
#[miniextendr(r6)]
impl R6CrossBoard {
    /// Create a board directly.
    /// @param width Board width.
    /// @param height Board height.
    /// @param seed Seed carried from the plan.
    pub fn new(width: i32, height: i32, seed: i32) -> Self {
        R6CrossBoard {
            width,
            height,
            seed,
        }
    }

    /// Count the cells in the board.
    pub fn cells(&self) -> i32 {
        self.width * self.height
    }

    /// Summarize the board dimensions and seed.
    pub fn signature(&self) -> String {
        format!("{}x{}@{}", self.width, self.height, self.seed)
    }
}

/// R6 builder whose terminal `build()` returns a different R6 class.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6CrossPlan {
    seed: i32,
}

/// R6 builder fixture for cross-class return wrapping.
#[miniextendr(r6)]
impl R6CrossPlan {
    /// Create a cross-class plan.
    /// @param seed Seed to carry into built boards.
    pub fn new(seed: i32) -> Self {
        R6CrossPlan { seed }
    }

    /// Materialize this plan into an `R6CrossBoard`.
    /// @param width Board width.
    /// @param height Board height.
    pub fn build(&self, width: i32, height: i32) -> R6CrossBoard {
        R6CrossBoard {
            width,
            height,
            seed: self.seed,
        }
    }

    /// Materialize this plan into an `S7CrossBoard` (R6 source, S7 target).
    ///
    /// The write-time resolver keys off the *returned* class, so the emitted
    /// wrapper uses S7's `S7CrossBoard(.ptr = .val)` constructor even though
    /// the enclosing method lives on an R6 class.
    /// @param width Board width.
    /// @param height Board height.
    pub fn build_s7(&self, width: i32, height: i32) -> S7CrossBoard {
        S7CrossBoard {
            cells: self.seed + (width * height),
        }
    }

    /// Materialize this plan into an `R6CrossBoard`, or `None` when `fail` is set.
    ///
    /// Exercises the `Option<Class>` cross-class return path: the C wrapper
    /// already unwraps `Option<T>` and raises on `None`
    /// (`ReturnHandling::OptionIntoRUnwrap`), so on `Some` the successful
    /// `.val` is a bare pointer wrapped with `R6CrossBoard`'s constructor,
    /// identical to the bare-class `build()` above.
    /// @param width Board width.
    /// @param height Board height.
    /// @param fail When true, returns `None` instead of a board.
    pub fn try_build(&self, width: i32, height: i32, fail: bool) -> Option<R6CrossBoard> {
        if fail {
            return None;
        }
        Some(R6CrossBoard {
            width,
            height,
            seed: self.seed,
        })
    }

    /// Materialize this plan into an `R6CrossBoard`, or an `Err` message when
    /// `fail` is set.
    ///
    /// Exercises the `Result<Class, E>` cross-class return path
    /// (`ReturnHandling::ResultIntoR`).
    /// @param width Board width.
    /// @param height Board height.
    /// @param fail When true, returns an `Err` instead of a board.
    pub fn checked_build(
        &self,
        width: i32,
        height: i32,
        fail: bool,
    ) -> Result<R6CrossBoard, String> {
        if fail {
            return Err(format!("checked_build failed for seed {}", self.seed));
        }
        Ok(R6CrossBoard {
            width,
            height,
            seed: self.seed,
        })
    }

    /// Materialize this plan into `count` boards (seeds `seed`, `seed + 1`, …).
    ///
    /// Exercises the `Vec<Class>` cross-class return path (#1284): the C
    /// wrapper converts via the `IntoRVecElement` impl emitted by
    /// `#[derive(ExternalPtr)]` into a bare list of external pointers, and the
    /// write-time list resolver wraps each element with
    /// `R6CrossBoard$new(.ptr = .el)` via `lapply`. `count = 0` returns
    /// `list()`.
    /// @param width Board width.
    /// @param height Board height.
    /// @param count Number of boards to build.
    pub fn build_many(&self, width: i32, height: i32, count: i32) -> Vec<R6CrossBoard> {
        (0..count)
            .map(|i| R6CrossBoard {
                width,
                height,
                seed: self.seed + i,
            })
            .collect()
    }

    /// Materialize this plan into `count` `S7CrossBoard`s (R6 source, S7
    /// element target).
    ///
    /// Mixed-system list return: the write-time resolver keys off the
    /// *element* class, so the emitted `lapply` wrap uses S7's
    /// `S7CrossBoard(.ptr = .el)` constructor even though the enclosing
    /// method lives on an R6 class.
    /// @param width Board width.
    /// @param height Board height.
    /// @param count Number of boards to build.
    pub fn build_many_s7(&self, width: i32, height: i32, count: i32) -> Vec<S7CrossBoard> {
        (0..count)
            .map(|i| S7CrossBoard {
                cells: self.seed + i + (width * height),
            })
            .collect()
    }

    /// Materialize this plan into `count` boards, or `None` when `fail` is set.
    ///
    /// Exercises the `Option<Vec<Class>>` cross-class return path: the C
    /// wrapper unwraps and raises on `None` (`ReturnHandling::OptionIntoRUnwrap`),
    /// so on `Some` the successful `.val` is the same bare list `build_many`
    /// produces, wrapped by the same `lapply` resolver.
    /// @param width Board width.
    /// @param height Board height.
    /// @param count Number of boards to build.
    /// @param fail When true, returns `None` instead of boards.
    pub fn try_build_many(
        &self,
        width: i32,
        height: i32,
        count: i32,
        fail: bool,
    ) -> Option<Vec<R6CrossBoard>> {
        if fail {
            return None;
        }
        Some(self.build_many(width, height, count))
    }

    /// Materialize this plan into `count` boards, or an `Err` message when
    /// `fail` is set.
    ///
    /// Exercises the `Result<Vec<Class>, E>` cross-class return path
    /// (`ReturnHandling::ResultIntoR`).
    /// @param width Board width.
    /// @param height Board height.
    /// @param count Number of boards to build.
    /// @param fail When true, returns an `Err` instead of boards.
    pub fn checked_build_many(
        &self,
        width: i32,
        height: i32,
        count: i32,
        fail: bool,
    ) -> Result<Vec<R6CrossBoard>, String> {
        if fail {
            return Err(format!("checked_build_many failed for seed {}", self.seed));
        }
        Ok(self.build_many(width, height, count))
    }
}

/// Board materialized by `S7CrossPlan::s7_cross_build`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S7CrossBoard {
    cells: i32,
}

/// S7 board target for cross-class builder returns.
#[miniextendr(s7)]
impl S7CrossBoard {
    /// Create a board directly.
    /// @param cells Number of cells.
    pub fn new(cells: i32) -> Self {
        S7CrossBoard { cells }
    }

    /// Count the cells in the board.
    pub fn s7_cross_cells(&self) -> i32 {
        self.cells
    }
}

/// S7 builder whose terminal method returns a different S7 class.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S7CrossPlan {
    seed: i32,
}

/// S7 builder fixture for cross-class return wrapping.
#[miniextendr(s7)]
impl S7CrossPlan {
    /// Create a cross-class plan.
    /// @param seed Seed added to built board cells.
    pub fn new(seed: i32) -> Self {
        S7CrossPlan { seed }
    }

    /// Materialize this plan into an `S7CrossBoard`.
    /// @param width Board width.
    /// @param height Board height.
    pub fn s7_cross_build(&self, width: i32, height: i32) -> S7CrossBoard {
        S7CrossBoard {
            cells: self.seed + (width * height),
        }
    }

    /// Materialize this plan into an `R6CrossBoard` (S7 source, R6 target).
    ///
    /// Mirror of [`R6CrossPlan::build_s7`]: the returned class drives the
    /// constructor choice, so this S7 method's wrapper builds the R6 target via
    /// `R6CrossBoard$new(.ptr = .val)`.
    /// @param width Board width.
    /// @param height Board height.
    pub fn s7_build_r6(&self, width: i32, height: i32) -> R6CrossBoard {
        R6CrossBoard {
            width,
            height,
            seed: self.seed,
        }
    }

    /// Materialize this plan into an `R6CrossBoard` (S7 source, R6 target),
    /// or `None` when `fail` is set.
    ///
    /// Mixed-system container return: proves the target-keyed resolver picks
    /// the R6 constructor for the `Some` arm even though this method lives on
    /// an S7 class, mirroring [`Self::s7_build_r6`] but through the
    /// `Option<Class>` cross-class return path.
    /// @param width Board width.
    /// @param height Board height.
    /// @param fail When true, returns `None` instead of a board.
    pub fn s7_try_build_r6(&self, width: i32, height: i32, fail: bool) -> Option<R6CrossBoard> {
        if fail {
            return None;
        }
        Some(R6CrossBoard {
            width,
            height,
            seed: self.seed,
        })
    }

    /// Materialize this plan into `count` `R6CrossBoard`s (S7 source, R6
    /// element target).
    ///
    /// Mixed-system list return (#1284): mirror of
    /// [`R6CrossPlan::build_many_s7`] — the element class drives the `lapply`
    /// wrap, so this S7 method's wrapper builds R6 targets via
    /// `R6CrossBoard$new(.ptr = .el)`.
    /// @param width Board width.
    /// @param height Board height.
    /// @param count Number of boards to build.
    pub fn s7_build_many_r6(&self, width: i32, height: i32, count: i32) -> Vec<R6CrossBoard> {
        (0..count)
            .map(|i| R6CrossBoard {
                width,
                height,
                seed: self.seed + i,
            })
            .collect()
    }
}
// endregion

// region: Env self-ref builder fixture
//
// Env classes share `ChainableMutation` semantics with R6: the method body
// returns `self` (the environment). Chains as `b$add(1L)$add(2L)$total()`
// through the `$` re-parenting dispatch.

/// A small accumulating builder exposed as an env-style class.
#[derive(miniextendr_api::ExternalPtr)]
pub struct EnvPipeBuilder {
    total: i32,
}

/// Env builder with a `&mut self -> &mut Self` step and a terminal accessor.
/// Chains as `b$add(1L)$add(2L)$total()`.
#[miniextendr(env)]
impl EnvPipeBuilder {
    /// Create a new builder starting at zero.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        EnvPipeBuilder { total: 0 }
    }

    /// Add `amount` in place and return the builder for chaining.
    /// @param amount Amount to add.
    pub fn add(&mut self, amount: i32) -> &mut Self {
        self.total += amount;
        self
    }

    /// Terminal accessor: read the accumulated total.
    pub fn total(&self) -> i32 {
        self.total
    }
}
// endregion

// region: consuming `self` builders (#1432) and fallible in-place steps (#1433)
//
// Libraries often write builders as `fn step(mut self, ..) -> Self` /
// `-> Result<Self, E>`. The wrapper supports them without a bridge type:
// `-> Self` moves the value out of the R handle, calls the step, and writes the
// result back into the SAME handle (identity preserved, like `&mut Self`);
// `-> Result<Self, E>` / `Option<Self>` run on a clone and overwrite only on
// success, so a failed step leaves the R object untouched; any other return is
// a terminal consume after which the handle errors on use.

/// S3 consuming builder over an integer total.
#[derive(Clone, miniextendr_api::ExternalPtr)]
pub struct ConsumingBuilder {
    total: i32,
}

/// Consuming builder exposed as pipe-friendly S3 free functions.
#[allow(clippy::new_without_default)]
#[miniextendr(s3)]
impl ConsumingBuilder {
    /// Start at zero.
    pub fn new() -> Self {
        ConsumingBuilder { total: 0 }
    }

    /// `self -> Self`: add `amount`, written back into the same R object.
    /// @param amount Amount to add.
    pub fn with_amount(mut self, amount: i32) -> Self {
        self.total += amount;
        self
    }

    /// `self -> Result<Self, E>`: negative amounts are rejected and the
    /// object is left unchanged.
    /// @param amount Amount to add (must be non-negative).
    pub fn try_amount(mut self, amount: i32) -> Result<Self, String> {
        if amount < 0 {
            return Err(format!("amount must be non-negative, got {amount}"));
        }
        self.total += amount;
        Ok(self)
    }

    /// `self -> Option<Self>`: `None` when the total would exceed 100.
    /// @param amount Amount to add.
    pub fn maybe_amount(mut self, amount: i32) -> Option<Self> {
        let next = self.total.checked_add(amount)?;
        if next > 100 {
            return None;
        }
        self.total = next;
        Some(self)
    }

    /// `&mut self -> Result<&mut Self, E>` (#1433): fallible in-place step.
    /// @param amount Amount to add (must be non-negative).
    pub fn checked_bump(&mut self, amount: i32) -> Result<&mut Self, String> {
        if amount < 0 {
            return Err(format!("amount must be non-negative, got {amount}"));
        }
        self.total += amount;
        Ok(self)
    }

    /// `&mut self -> Option<&mut Self>` (#1433).
    /// @param amount Amount to add.
    pub fn maybe_bump(&mut self, amount: i32) -> Option<&mut Self> {
        let next = self.total.checked_add(amount)?;
        if next > 100 {
            return None;
        }
        self.total = next;
        Some(self)
    }

    /// Read the total without consuming.
    pub fn total(&self) -> i32 {
        self.total
    }

    /// Terminal consume: returns the total and leaves the handle consumed.
    pub fn finish(self) -> i32 {
        self.total
    }
}

/// R6 consuming builder.
#[derive(Clone, miniextendr_api::ExternalPtr)]
pub struct R6ConsumingBuilder {
    total: i32,
}

/// Chains as `b$with_amount(1L)$try_amount(2L)$total()`.
#[allow(clippy::new_without_default)]
#[miniextendr(r6)]
impl R6ConsumingBuilder {
    /// Start at zero.
    pub fn new() -> Self {
        R6ConsumingBuilder { total: 0 }
    }

    /// `self -> Self` step.
    /// @param amount Amount to add.
    pub fn with_amount(mut self, amount: i32) -> Self {
        self.total += amount;
        self
    }

    /// `self -> Result<Self, E>` step.
    /// @param amount Amount to add (must be non-negative).
    pub fn try_amount(mut self, amount: i32) -> Result<Self, String> {
        if amount < 0 {
            return Err(format!("amount must be non-negative, got {amount}"));
        }
        self.total += amount;
        Ok(self)
    }

    /// Fallible in-place step (#1433).
    /// @param amount Amount to add (must be non-negative).
    pub fn checked_bump(&mut self, amount: i32) -> Result<&mut Self, String> {
        if amount < 0 {
            return Err(format!("amount must be non-negative, got {amount}"));
        }
        self.total += amount;
        Ok(self)
    }

    /// Read the total.
    pub fn total(&self) -> i32 {
        self.total
    }

    /// Terminal consume.
    pub fn finish(self) -> i32 {
        self.total
    }
}

/// Env consuming builder.
#[derive(Clone, miniextendr_api::ExternalPtr)]
pub struct EnvConsumingBuilder {
    total: i32,
}

/// Chains as `b$with_amount(1L)$try_amount(2L)$total()`.
#[allow(clippy::new_without_default)]
#[miniextendr(env)]
impl EnvConsumingBuilder {
    /// Start at zero.
    pub fn new() -> Self {
        EnvConsumingBuilder { total: 0 }
    }

    /// `self -> Self` step.
    /// @param amount Amount to add.
    pub fn with_amount(mut self, amount: i32) -> Self {
        self.total += amount;
        self
    }

    /// `self -> Result<Self, E>` step.
    /// @param amount Amount to add (must be non-negative).
    pub fn try_amount(mut self, amount: i32) -> Result<Self, String> {
        if amount < 0 {
            return Err(format!("amount must be non-negative, got {amount}"));
        }
        self.total += amount;
        Ok(self)
    }

    /// Fallible in-place step (#1433).
    /// @param amount Amount to add (must be non-negative).
    pub fn checked_bump(&mut self, amount: i32) -> Result<&mut Self, String> {
        if amount < 0 {
            return Err(format!("amount must be non-negative, got {amount}"));
        }
        self.total += amount;
        Ok(self)
    }

    /// Read the total.
    pub fn total(&self) -> i32 {
        self.total
    }

    /// Terminal consume.
    pub fn finish(self) -> i32 {
        self.total
    }
}
// endregion
