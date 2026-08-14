# LLM-doc generation assumes Cargo uses the repository-local `target/`

**Tracked as:** https://github.com/A2-ai/miniextendr/issues/1413

## Summary

`just llm-docs` runs every rustdoc command successfully and then fails while
rendering Markdown when Cargo is configured with a non-default target
directory:

```text
FileNotFoundError: target/doc/miniextendr_api.json
```

In the observed environment, `cargo metadata --no-deps --format-version 1`
reported `target_directory` as `/Users/elea/.cargo-target`. The generator
nevertheless hard-codes `target/doc/*.json` for the root workspace and
`cargo-revendor/target/doc/*.json` for the standalone workspace. The JSON is
created successfully in Cargo's configured target directory, then the renderer
looks in the wrong place.

## Reproduction

Configure Cargo's `build.target-dir` outside the checkout, then run:

```text
just llm-docs
```

Rustdoc completes, but `rustdoc_megadoc.py` fails on the first hard-coded
repository-local JSON path.

## Suggested fix

Make the generator own deterministic target directories for both workspaces
by setting `CARGO_TARGET_DIR` on the rustdoc commands and using those same
paths for rendering, or resolve each workspace's actual target directory from
Cargo metadata. Add a regression invocation with an external Cargo target-dir
configuration.

## Overlap audit

The existing LLM-doc reviews cover a stale sccache worktree and an absolute
cargo-revendor source path in generated inventories. Neither covers Cargo's
configured target directory. No open issue matched this failure.
