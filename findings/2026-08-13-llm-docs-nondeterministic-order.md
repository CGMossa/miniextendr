# Rust API reports inherited rustdoc JSON object order

## Finding

Generating the same Rust API corpus in separate worktrees produced different
Markdown ordering. A factor-only API removal changed thousands of unrelated
lines: root re-exports moved, equal-sized trait sections swapped, impls sharing
a source span changed order, and equal-sized clustering groups moved.

The renderers did call `sorted`, but several keys were not total orderings:

- megadoc items were sorted only by `qualified_name`; rustdoc `use` items often
  have no path entry and all collapsed to `Unknown`;
- trait inventories sorted sections and span clusters only by descending count;
- impl rows sorted only by source span, even though macro-expanded impl families
  deliberately share a span; and
- manual-vs-macro shapes sorted only by member count.

Python's stable sort preserved the input order for every tie. Rustdoc JSON uses
an object for its item index, and that object order differed across builds, so
the generated corpus was not reproducible across worktrees.

## Resolution

Use semantic secondary keys for every tied ordering: re-export source/name,
trait name, impl type and metadata, span text, sorted cluster members, and shape
name. Regression tests render forward and reversed JSON indexes and require
byte-identical megadoc, impl-inventory, and manual-vs-macro output. Regenerate
the corpus once into the canonical order.
