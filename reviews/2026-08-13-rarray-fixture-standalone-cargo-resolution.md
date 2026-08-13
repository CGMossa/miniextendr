# Standalone rpkg cargo check missed repository patch configuration

## What was attempted

Compile-check the new RArray constructor dogfood fixture with `cargo lcheck`
against `rpkg/src/rust/Cargo.toml` before installing the package.

## What went wrong

Cargo resolved a `miniextendr-api` package without the `fast-default` feature
required by rpkg and stopped during dependency selection.

## Root cause

The standalone rpkg manifest needs the repository recipes' explicit local
`[patch]` configuration. A raw manifest invocation does not reproduce that
multi-workspace setup.

## Fix

Use `just rcmdinstall` for the public fixture build and runtime reproduction.
Use the repository-wide recipes and exact CI clippy commands for final gates.
