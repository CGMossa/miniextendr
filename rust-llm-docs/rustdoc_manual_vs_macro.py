#!/usr/bin/env python3
"""
Find hand-rolled (unique-span) trait impls whose container shape an existing
macro already stamps out for other element types — i.e. manual impls that a
macro could absorb.

Heuristic: impls that share a source span are macro-expanded; impls alone at a
span are hand-rolled. For each conversion trait we normalise the `for`-type to a
container skeleton (`Vec<_>`, `Box<[_]>`, `Option<_>`, …) and report, per shape,
how many members are hand-rolled vs macro-generated. A shape with BOTH (or with
many hand-rolled members) is a consolidation candidate.

Usage:
    python rustdoc_manual_vs_macro.py <doc.json> [--traits T1,T2,...]
        [--source-label LABEL] [--out FILE]
"""

import json
import re
import sys
import argparse
from pathlib import Path
from collections import defaultdict

sys.path.insert(0, str(Path(__file__).parent))
from rustdoc_common import format_type

SHAPE_RULES = [
    (r"^Vec<Option<.*>>$", "Vec<Option<_>>"),
    (r"^Vec<.*>$", "Vec<_>"),
    (r"^Box<\[Option<.*>\]>$", "Box<[Option<_>]>"),
    (r"^Box<\[.*\]>$", "Box<[_]>"),
    (r"^&.*\[.*\]$", "&[_]"),
    (r"^Option<.*>$", "Option<_>"),
    (r"^HashSet<.*>$", "HashSet<_>"),
    (r"^BTreeSet<.*>$", "BTreeSet<_>"),
    (r"^HashMap<.*>$", "HashMap<_>"),
    (r"^BTreeMap<.*>$", "BTreeMap<_>"),
]


def container_shape(s: str) -> str:
    s = s.strip()
    for pat, repl in SHAPE_RULES:
        if re.match(pat, s):
            return repl
    return s  # scalar / named / other


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("json")
    ap.add_argument("--traits", default="TryFromSexp,IntoR,IntoRAs,Coerce,TryCoerce")
    ap.add_argument(
        "--source-label",
        default="",
        help="stable source path to print instead of the JSON input path",
    )
    ap.add_argument("--out", default="")
    args = ap.parse_args()

    data = json.load(open(args.json))
    index = data["index"]
    traits = [t.strip() for t in args.traits.split(",") if t.strip()]

    recs = defaultdict(list)
    spancount = defaultdict(lambda: defaultdict(int))
    for iid, it in index.items():
        imp = it.get("inner", {}).get("impl")
        if not imp or not imp.get("trait"):
            continue
        term = imp["trait"]["path"].rsplit("::", 1)[-1]
        if term not in traits or imp.get("is_synthetic"):
            continue
        for_s = format_type(imp.get("for"), index)
        sp = it.get("span") or {}
        span = f"{sp.get('filename', '?')}:{sp.get('begin', [0])[0]}" if sp else "(none)"
        recs[term].append((for_s, span, len(imp.get("items", []) or [])))
        spancount[term][span] += 1

    out = [
        "# Manual-vs-macro conversion impls",
        "",
        f"Source: `{args.source_label or args.json}`",
        "",
        "> **Caveat**: the \"macro already exists for this shape\" flags count",
        "> feature-gated optionals (jiff, uuid, bitvec, ...) as absorbable",
        "> hand-rolled impls, but most have bespoke element conversions a",
        "> shape-macro cannot express. Treat the headline counts as an upper",
        "> bound on the dedup opportunity, not a work list.",
        "",
    ]
    for term in traits:
        rs = recs.get(term, [])
        shape_macro = defaultdict(int)
        manual_by_shape = defaultdict(list)
        for for_s, span, n in rs:
            sh = container_shape(for_s)
            if spancount[term][span] > 1:
                shape_macro[sh] += 1
            else:
                manual_by_shape[sh].append((for_s, span, n))
        out.append(f"## {term}")
        out.append("")
        for sh in sorted(
            manual_by_shape, key=lambda shape: (-len(manual_by_shape[shape]), shape)
        ):
            macro_n = shape_macro.get(sh, 0)
            members = manual_by_shape[sh]
            if macro_n == 0 and len(members) < 3:
                continue
            flag = "  <== macro already exists for this shape" if macro_n else ""
            out.append(f"- shape `{sh}`: **{len(members)} hand-rolled**, {macro_n} macro-generated{flag}")
            for for_s, span, n in sorted(members):
                out.append(f"    - `{for_s}` ({n} items) — {span}")
        out.append("")

    text = "\n".join(out)
    if args.out:
        Path(args.out).write_text(text)
        print(f"Wrote {args.out} ({len(text)} bytes)")
    else:
        print(text)


if __name__ == "__main__":
    main()
