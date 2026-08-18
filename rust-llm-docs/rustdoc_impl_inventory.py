#!/usr/bin/env python3
"""
Inventory every trait `impl` in a rustdoc JSON, grouped by trait, with the
fully-resolved `for` type and source span (file:line).

Purpose: spot duplicate / near-duplicate conversion implementations
(e.g. `IntoR for Vec<T>`, `IntoR for Vec<Option<T>>`, `IntoR for &[T]`,
`IntoR for Box<[T]>`) that share a body and could be collapsed behind a
macro or a blanket impl.

Usage:
    python rustdoc_impl_inventory.py <doc.json> [--traits T1,T2,...]
        [--source-label LABEL] [--out FILE]

Without --traits, inventories ALL traits. With --traits, restricts to the
named traits (matched on the trait's terminal path segment).

Output is markdown: one section per trait, a table of (for-type, span,
generics, kind). A trailing "clustering" section groups for-types by their
source span so macro-generated families and hand-rolled duplicates are
obvious at a glance.
"""

import json
import sys
import argparse
from pathlib import Path
from collections import defaultdict, Counter

sys.path.insert(0, str(Path(__file__).parent))
from rustdoc_common import format_type


def trait_terminal(path: str) -> str:
    return path.rsplit("::", 1)[-1]


def fmt_generics(generics: dict, index: dict) -> str:
    """Render generic params + where predicates compactly."""
    params = generics.get("params", []) or []
    names = []
    for p in params:
        nm = p.get("name", "")
        if nm and nm not in ("'_",):
            names.append(nm)
    gp = f"<{', '.join(names)}>" if names else ""
    wheres = generics.get("where_predicates", []) or []
    nw = len(wheres)
    where_s = f" +{nw}wc" if nw else ""
    return gp + where_s


def span_str(item: dict) -> str:
    sp = item.get("span")
    if not sp:
        return "(no span)"
    fn = sp.get("filename", "?")
    beg = sp.get("begin", [0, 0])
    return f"{fn}:{beg[0]}"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("json")
    ap.add_argument("--traits", default="", help="comma-separated trait names to restrict to")
    ap.add_argument(
        "--source-label",
        default="",
        help="stable source path to print instead of the JSON input path",
    )
    ap.add_argument("--out", default="", help="output markdown file")
    args = ap.parse_args()

    data = json.load(open(args.json))
    index = data["index"]
    want = {t.strip() for t in args.traits.split(",") if t.strip()}

    # trait terminal -> list of records
    by_trait = defaultdict(list)

    for iid, it in index.items():
        inner = it.get("inner", {})
        imp = inner.get("impl")
        if not imp:
            continue
        tr = imp.get("trait")
        if not tr:
            continue  # inherent impl, skip
        term = trait_terminal(tr.get("path", ""))
        if want and term not in want:
            continue
        for_ty = format_type(imp.get("for"), index)
        rec = {
            "for": for_ty,
            "span": span_str(it),
            "generics": fmt_generics(imp.get("generics", {}), index),
            "blanket": imp.get("blanket_impl") is not None,
            "synthetic": imp.get("is_synthetic", False),
            "negative": imp.get("is_negative", False),
            "n_items": len(imp.get("items", []) or []),
        }
        by_trait[term].append(rec)

    lines = []
    lines.append("# Trait impl inventory")
    lines.append("")
    lines.append(f"Source: `{args.source_label or args.json}`")
    lines.append("")
    lines.append(f"Traits with impls: {len(by_trait)}")
    lines.append("")

    # summary table
    lines.append("## Summary (impl count per trait)")
    lines.append("")
    lines.append("| Trait | # impls | # non-blanket non-synthetic |")
    lines.append("|---|---|---|")
    trait_order = sorted(by_trait, key=lambda term: (-len(by_trait[term]), term))
    for term in trait_order:
        recs = by_trait[term]
        real = sum(1 for r in recs if not r["blanket"] and not r["synthetic"])
        lines.append(f"| `{term}` | {len(recs)} | {real} |")
    lines.append("")

    for term in trait_order:
        recs = by_trait[term]
        # Drop synthetic auto-trait noise AND blanket-impl instantiations.
        # Blanket impls (Tap, Pipe, Pointable, From/Into, ...) get one
        # rustdoc record per local type — hundreds of rows that carry no
        # dedup-audit signal. The summary table above still reports their
        # totals, so coverage stays discoverable.
        real = [r for r in recs if not r["synthetic"] and not r["blanket"]]
        if not real:
            continue
        lines.append(f"## `{term}` — {len(real)} impls")
        lines.append("")
        lines.append("| for-type | generics | kind | #items | span |")
        lines.append("|---|---|---|---|---|")
        for r in sorted(
            real,
            key=lambda r: (
                r["span"],
                r["for"],
                r["generics"],
                r["blanket"],
                r["negative"],
                r["n_items"],
            ),
        ):
            kind = []
            if r["blanket"]:
                kind.append("blanket")
            if r["negative"]:
                kind.append("neg")
            kind = ",".join(kind) or "concrete"
            forty = r["for"].replace("|", "\\|")
            lines.append(f"| `{forty}` | `{r['generics']}` | {kind} | {r['n_items']} | {r['span']} |")
        lines.append("")

        # clustering by span (file:line) — macro families share a span
        span_groups = defaultdict(list)
        for r in real:
            span_groups[r["span"]].append(r["for"])
        multi = {s: v for s, v in span_groups.items() if len(v) > 1}
        if multi:
            lines.append(f"### `{term}` — for-types sharing a source span (likely macro-expanded / co-located)")
            lines.append("")
            for s, fors in sorted(
                multi.items(), key=lambda item: (-len(item[1]), item[0])
            ):
                rendered = ", ".join(f"`{for_type}`" for for_type in sorted(fors))
                lines.append(f"- **{s}** ({len(fors)} impls): {rendered}")
            lines.append("")

    out = "\n".join(lines)
    if args.out:
        Path(args.out).write_text(out)
        print(f"Wrote {args.out} ({len(out)} bytes)")
    else:
        print(out)


if __name__ == "__main__":
    main()
