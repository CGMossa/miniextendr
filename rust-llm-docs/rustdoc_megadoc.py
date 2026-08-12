#!/usr/bin/env python3
"""
Generate a single comprehensive markdown file from rustdoc JSON.

Covers every public API item kind represented by rustdoc JSON format 57.
Container items (fields, variants, associated items, and inherent impls) are
rendered under their parent; trait impl blocks are emitted by the companion
``rustdoc_impl_inventory.py`` report. Item headings are module-qualified so
same-named items in different modules stay distinguishable, and markdown
headings inside doc comments are demoted below the containing section's level.

Usage:
    python rustdoc_megadoc.py <input.json> [output.md]
"""

import json
import sys
from pathlib import Path

from rustdoc_common import (
    demote_headings,
    format_function_signature,
    format_generic_bound,
    format_generics,
    format_path,
    format_type,
)

# Heading level used for individual item entries (### `module::Name`).
ITEM_LEVEL = 3
# Heading level used for members of an item (#### `method`).
MEMBER_LEVEL = 4

# All variants of rustdoc_json_types::ItemEnum in rustdoc JSON format 57.
# Fail on additions instead of silently dropping a new Rust item kind.
SUPPORTED_ITEM_KINDS = {
    "module",
    "extern_crate",
    "use",
    "union",
    "struct",
    "struct_field",
    "enum",
    "variant",
    "function",
    "trait",
    "trait_alias",
    "impl",
    "type_alias",
    "constant",
    "static",
    "extern_type",
    "macro",
    "proc_macro",
    "primitive",
    "assoc_const",
    "assoc_type",
}


def qualified_name(item: dict, paths: dict) -> str:
    """Module-qualified item name (crate segment dropped), e.g. `list::List`.

    Falls back to the bare name for items absent from the `paths` table
    (e.g. impl members).
    """
    entry = paths.get(str(item.get("id", "")), {})
    segments = entry.get("path")
    if segments and len(segments) > 1:
        return "::".join(segments[1:])
    return item.get("name") or "Unknown"


def item_kind(item: dict) -> str:
    """Return the single rustdoc ``ItemEnum`` key for an item."""
    inner = item.get("inner", {})
    return next(iter(inner), "")


def item_sort_key(item: dict, paths: dict) -> tuple[str, ...]:
    """Stable semantic ordering independent of rustdoc JSON object order."""
    inner = item.get("inner", {})
    use = inner.get("use", {})
    return (
        qualified_name(item, paths),
        item_kind(item),
        use.get("source", ""),
        use.get("name") or "",
        "1" if use.get("is_glob") else "0",
        item.get("name") or "",
        item.get("docs") or "",
    )


def validate_item_kinds(index: dict, crate_id_filter: int | None) -> None:
    """Reject rustdoc schema additions that the renderer has not audited."""
    seen = {
        item_kind(item)
        for item in index.values()
        if crate_id_filter is None or item.get("crate_id") == crate_id_filter
    }
    unknown = sorted(seen - SUPPORTED_ITEM_KINDS)
    if unknown:
        raise ValueError(f"unsupported rustdoc item kinds: {', '.join(unknown)}")


def append_docs(lines: list, docs: str, base_level: int) -> None:
    if docs:
        lines.append(demote_headings(docs, base_level))
        lines.append("")


def collect_member_ids(index: dict) -> set:
    """IDs of items that live inside an impl block or a trait definition.

    Used to tell module-level functions apart from methods and trait members.
    """
    members = set()
    for item in index.values():
        inner = item.get("inner", {})
        container = inner.get("impl") or inner.get("trait")
        if container:
            members.update(str(mid) for mid in container.get("items", []) or [])
    return members


def generate_megadoc(
    data: dict, crate_only: bool = True, include_reexports: bool = True
) -> str:
    """Generate a comprehensive markdown document from rustdoc JSON."""
    index = data.get("index", {})
    paths = data.get("paths", {})
    root_id = data.get("root")
    crate_version = data.get("crate_version", "unknown")

    # Get crate info
    root = index.get(str(root_id), {})
    crate_name = root.get("name", "unknown")
    crate_docs = root.get("docs", "")

    lines = []
    lines.append(f"# {crate_name} v{crate_version}")
    lines.append("")
    append_docs(lines, crate_docs, base_level=1)

    # Collect items by kind
    kinds = {
        "modules": [],
        "extern_crates": [],
        "uses": [],
        "unions": [],
        "structs": [],
        "enums": [],
        "traits": [],
        "trait_aliases": [],
        "functions": [],
        "macros": [],
        "constants": [],
        "statics": [],
        "type_aliases": [],
        "extern_types": [],
        "primitives": [],
    }
    member_ids = collect_member_ids(index)
    crate_id_filter = 0 if crate_only else None
    validate_item_kinds(index, crate_id_filter)

    for key, item in index.items():
        if crate_id_filter is not None and item.get("crate_id") != crate_id_filter:
            continue
        if item.get("visibility") != "public":
            continue

        inner = item.get("inner", {})
        if "module" in inner:
            if str(key) != str(root_id):
                kinds["modules"].append(item)
        elif "extern_crate" in inner:
            kinds["extern_crates"].append(item)
        elif "use" in inner:
            if include_reexports:
                kinds["uses"].append(item)
        elif "union" in inner:
            kinds["unions"].append(item)
        elif "struct" in inner:
            kinds["structs"].append(item)
        elif "enum" in inner:
            kinds["enums"].append(item)
        elif "trait" in inner:
            kinds["traits"].append(item)
        elif "trait_alias" in inner:
            kinds["trait_aliases"].append(item)
        elif "function" in inner:
            if key not in member_ids:
                kinds["functions"].append(item)
        elif "macro" in inner or "proc_macro" in inner:
            kinds["macros"].append(item)
        elif "constant" in inner:
            kinds["constants"].append(item)
        elif "static" in inner:
            kinds["statics"].append(item)
        elif "type_alias" in inner:
            kinds["type_aliases"].append(item)
        elif "extern_type" in inner:
            kinds["extern_types"].append(item)
        elif "primitive" in inner:
            kinds["primitives"].append(item)

    def emit_section(title: str, items: list, document) -> None:
        if not items:
            return
        lines.append("---")
        lines.append("")
        lines.append(f"## {title}")
        lines.append("")
        for item in sorted(items, key=lambda item: item_sort_key(item, paths)):
            lines.extend(document(item, index, paths))

    emit_section("Modules", kinds["modules"], document_module)
    emit_section("Re-exports", kinds["uses"], document_use)
    emit_section("Extern crates", kinds["extern_crates"], document_extern_crate)
    emit_section("Structs", kinds["structs"], document_struct)
    emit_section("Unions", kinds["unions"], document_union)
    emit_section("Enums", kinds["enums"], document_enum)
    emit_section("Traits", kinds["traits"], document_trait)
    emit_section("Trait aliases", kinds["trait_aliases"], document_trait_alias)
    emit_section("Functions", kinds["functions"], document_function)
    emit_section("Macros", kinds["macros"], document_macro)
    emit_section("Constants", kinds["constants"], document_constant)
    emit_section("Statics", kinds["statics"], document_static)
    emit_section("Type aliases", kinds["type_aliases"], document_type_alias)
    emit_section("Extern types", kinds["extern_types"], document_extern_type)
    emit_section("Primitive types", kinds["primitives"], document_primitive)

    return "\n".join(lines)


def item_heading(item: dict, paths: dict) -> list:
    return [f"{'#' * ITEM_LEVEL} `{qualified_name(item, paths)}`", ""]


def declaration(kind: str, item: dict, info: dict, index: dict) -> str:
    """Render the declaration prefix shared by named generic items."""
    params, where_clause = format_generics(info.get("generics", {}), index)
    return f"pub {kind} {item.get('name', '?')}{params}{where_clause}"


def document_module(item: dict, index: dict, paths: dict) -> list:
    lines = item_heading(item, paths)
    lines.extend([f"`pub mod {item.get('name', '?')};`", ""])
    append_docs(lines, item.get("docs", ""), ITEM_LEVEL)
    return lines


def document_use(item: dict, index: dict, paths: dict) -> list:
    info = item.get("inner", {}).get("use", {})
    source = info.get("source", "?")
    name = info.get("name")
    if info.get("is_glob"):
        rendered = f"pub use {source}::*;"
    elif name and name != source.rsplit("::", 1)[-1]:
        rendered = f"pub use {source} as {name};"
    else:
        rendered = f"pub use {source};"
    lines = [f"{'#' * ITEM_LEVEL} `{rendered}`", ""]
    append_docs(lines, item.get("docs", ""), ITEM_LEVEL)
    return lines


def document_extern_crate(item: dict, index: dict, paths: dict) -> list:
    info = item.get("inner", {}).get("extern_crate", {})
    name = info.get("name", item.get("name", "?"))
    rename = info.get("rename")
    rendered = f"pub extern crate {name}"
    if rename:
        rendered += f" as {rename}"
    lines = [f"{'#' * ITEM_LEVEL} `{rendered};`", ""]
    append_docs(lines, item.get("docs", ""), ITEM_LEVEL)
    return lines


def field_lines(
    field_ids: list, index: dict, *, named: bool, public_only: bool
) -> list[str]:
    """Render named or tuple fields from their rustdoc item IDs."""
    rendered = []
    for position, fid in enumerate(field_ids):
        if fid is None:
            rendered.append(f"- `{position}`: *(private or hidden)*")
            continue
        field = index.get(str(fid), {})
        if public_only and field.get("visibility") != "public":
            continue
        field_type = field.get("inner", {}).get("struct_field", {})
        label = field.get("name", "?") if named else str(position)
        rendered.append(f"- `{label}`: `{format_type(field_type, index)}`")
        docs = field.get("docs", "")
        if docs:
            rendered.append(f"  - {docs.split(chr(10))[0]}")
    return rendered


def append_fields(lines: list, rendered: list[str]) -> None:
    if rendered:
        lines.extend(["**Fields:**", ""])
        lines.extend(rendered)
        lines.append("")


def inherent_items(type_info: dict, index: dict) -> list:
    """Public items from inherent (non-trait) impl blocks."""
    items = []
    for impl_id in type_info.get("impls", []):
        impl_item = index.get(str(impl_id), {})
        impl_info = impl_item.get("inner", {}).get("impl", {})
        if impl_info.get("trait") is not None:
            continue
        for member_id in impl_info.get("items", []):
            member = index.get(str(member_id), {})
            if member.get("visibility") != "public":
                continue
            if item_kind(member) in {"function", "assoc_const", "assoc_type"}:
                items.append(member)
    return items


def assoc_item_signature(item: dict, index: dict) -> str:
    """Render an associated const or type declaration."""
    name = item.get("name", "?")
    inner = item.get("inner", {})
    if "assoc_const" in inner:
        info = inner["assoc_const"]
        rendered = f"const {name}: {format_type(info.get('type'), index)}"
        if info.get("value") is not None:
            rendered += f" = {info['value']}"
        return rendered

    info = inner.get("assoc_type", {})
    params, where_clause = format_generics(info.get("generics", {}), index)
    rendered = f"type {name}{params}"
    bounds = [format_generic_bound(bound, index) for bound in info.get("bounds", [])]
    if bounds:
        rendered += f": {' + '.join(bounds)}"
    if info.get("type") is not None:
        rendered += f" = {format_type(info['type'], index)}"
    return rendered + where_clause


def document_assoc_item(item: dict, index: dict) -> list:
    lines = [f"{'#' * MEMBER_LEVEL} `{item.get('name', '?')}`", ""]
    lines.extend(["```rust", assoc_item_signature(item, index), "```", ""])
    append_docs(lines, item.get("docs", ""), MEMBER_LEVEL)
    return lines


def append_inherent_items(lines: list, type_info: dict, index: dict) -> None:
    items = inherent_items(type_info, index)
    if not items:
        return
    lines.extend(["**Inherent associated items:**", ""])
    for item in sorted(items, key=lambda value: value.get("name", "")):
        if "function" in item.get("inner", {}):
            lines.extend(document_method(item, index))
        else:
            lines.extend(document_assoc_item(item, index))


def document_struct(struct: dict, index: dict, paths: dict) -> list:
    """Generate documentation for a struct."""
    lines = item_heading(struct, paths)
    struct_info = struct.get("inner", {}).get("struct", {})
    lines.extend(["```rust", declaration("struct", struct, struct_info, index), "```", ""])
    append_docs(lines, struct.get("docs", ""), ITEM_LEVEL)

    # Document fields
    kind = struct_info.get("kind", {})
    if isinstance(kind, dict) and "plain" in kind:
        append_fields(
            lines,
            field_lines(
                kind["plain"].get("fields", []), index, named=True, public_only=True
            ),
        )
    elif isinstance(kind, dict) and "tuple" in kind:
        append_fields(
            lines,
            field_lines(kind["tuple"], index, named=False, public_only=True),
        )

    append_inherent_items(lines, struct_info, index)

    return lines


def document_union(item: dict, index: dict, paths: dict) -> list:
    """Generate documentation for a union."""
    lines = item_heading(item, paths)
    info = item.get("inner", {}).get("union", {})
    lines.extend(["```rust", declaration("union", item, info, index), "```", ""])
    append_docs(lines, item.get("docs", ""), ITEM_LEVEL)
    append_fields(
        lines,
        field_lines(info.get("fields", []), index, named=True, public_only=True),
    )
    append_inherent_items(lines, info, index)
    return lines


def document_enum(enum: dict, index: dict, paths: dict) -> list:
    """Generate documentation for an enum."""
    lines = item_heading(enum, paths)
    enum_info = enum.get("inner", {}).get("enum", {})
    lines.extend(["```rust", declaration("enum", enum, enum_info, index), "```", ""])
    append_docs(lines, enum.get("docs", ""), ITEM_LEVEL)

    # Document variants
    variant_ids = enum_info.get("variants", [])
    if variant_ids:
        lines.append("**Variants:**")
        lines.append("")
        for vid in variant_ids:
            variant = index.get(str(vid), {})
            variant_name = variant.get("name", "?")
            variant_docs = variant.get("docs", "")
            variant_info = variant.get("inner", {}).get("variant", {})
            kind = variant_info.get("kind", {})

            # Format variant kind
            if "plain" in kind:
                lines.append(f"- `{variant_name}`")
            elif "tuple" in kind:
                field_ids = kind["tuple"]
                fields = []
                for fid in field_ids:
                    if fid is None:
                        fields.append("_")
                    else:
                        field = index.get(str(fid), {})
                        field_type = field.get("inner", {}).get("struct_field", {})
                        fields.append(format_type(field_type, index))
                lines.append(f"- `{variant_name}({', '.join(fields)})`")
            elif "struct" in kind:
                fields = []
                for fid in kind["struct"].get("fields", []):
                    field = index.get(str(fid), {})
                    field_name = field.get("name", "?")
                    field_type = field.get("inner", {}).get("struct_field", {})
                    fields.append(f"{field_name}: {format_type(field_type, index)}")
                lines.append(f"- `{variant_name} {{ {', '.join(fields)} }}`")

            discriminant = variant_info.get("discriminant")
            if discriminant:
                lines.append(f"  - discriminant: `{discriminant.get('expr', '?')}`")

            if variant_docs:
                lines.append(f"  - {variant_docs.split(chr(10))[0]}")
        lines.append("")

    append_inherent_items(lines, enum_info, index)

    return lines


def document_trait(trait: dict, index: dict, paths: dict) -> list:
    """Generate documentation for a trait: docs plus required/provided members."""
    lines = item_heading(trait, paths)
    trait_info = trait.get("inner", {}).get("trait", {})
    qualifiers = ["pub"]
    if trait_info.get("is_unsafe"):
        qualifiers.append("unsafe")
    if trait_info.get("is_auto"):
        qualifiers.append("auto")
    params, where_clause = format_generics(trait_info.get("generics", {}), index)
    signature = f"{' '.join(qualifiers)} trait {trait.get('name', '?')}{params}"
    bounds = [
        format_generic_bound(bound, index) for bound in trait_info.get("bounds", [])
    ]
    if bounds:
        signature += f": {' + '.join(bounds)}"
    signature += where_clause
    lines.extend(["```rust", signature, "```", ""])
    append_docs(lines, trait.get("docs", ""), ITEM_LEVEL)

    required, provided, assoc = [], [], []
    for mid in trait_info.get("items", []) or []:
        member = index.get(str(mid), {})
        inner = member.get("inner", {})
        name = member.get("name", "?")
        if "function" in inner:
            func = inner["function"]
            sig = format_function_signature(func, index, name)
            docs = member.get("docs", "")
            first_line = f"  - {docs.split(chr(10))[0]}" if docs else None
            target = provided if func.get("has_body") else required
            target.append((sig, first_line))
        elif "assoc_type" in inner:
            assoc.append((assoc_item_signature(member, index), member.get("docs", "")))
        elif "assoc_const" in inner:
            assoc.append((assoc_item_signature(member, index), member.get("docs", "")))

    for title, group in (("Required methods", required), ("Provided methods", provided)):
        if group:
            lines.append(f"**{title}:**")
            lines.append("")
            for sig, doc_line in group:
                lines.append(f"- `{sig}`")
                if doc_line:
                    lines.append(doc_line)
            lines.append("")
    if assoc:
        lines.append("**Associated items:**")
        lines.append("")
        for signature, docs in assoc:
            lines.append(f"- `{signature}`")
            if docs:
                lines.append(f"  - {docs.split(chr(10))[0]}")
        lines.append("")

    return lines


def document_trait_alias(item: dict, index: dict, paths: dict) -> list:
    """Generate documentation for an unstable trait alias."""
    lines = item_heading(item, paths)
    info = item.get("inner", {}).get("trait_alias", {})
    params, where_clause = format_generics(info.get("generics", {}), index)
    bounds = [format_generic_bound(bound, index) for bound in info.get("params", [])]
    signature = (
        f"pub trait {item.get('name', '?')}{params} = {' + '.join(bounds)}"
        f"{where_clause};"
    )
    lines.extend(["```rust", signature, "```", ""])
    append_docs(lines, item.get("docs", ""), ITEM_LEVEL)
    return lines


def document_function(func_item: dict, index: dict, paths: dict) -> list:
    """Generate documentation for a module-level function."""
    lines = item_heading(func_item, paths)
    func = func_item.get("inner", {}).get("function", {})
    sig = format_function_signature(func, index, func_item.get("name", "?"))
    lines.append("```rust")
    lines.append(sig)
    lines.append("```")
    lines.append("")
    append_docs(lines, func_item.get("docs", ""), ITEM_LEVEL)
    return lines


def document_macro(macro: dict, index: dict, paths: dict) -> list:
    """Generate documentation for a macro_rules! or proc-macro."""
    name = macro.get("name", "?")
    inner = macro.get("inner", {})
    if "proc_macro" in inner:
        kind = inner["proc_macro"].get("kind", "bang")
        label = {"attr": f"#[{name}]", "derive": f"#[derive({name})]"}.get(
            kind, f"{name}!"
        )
    else:
        label = f"{name}!"
    lines = [f"{'#' * ITEM_LEVEL} `{label}`", ""]
    if "proc_macro" in inner:
        helpers = inner["proc_macro"].get("helpers", []) or []
        if helpers:
            lines.extend([f"Helper attributes: `{', '.join(helpers)}`", ""])
    append_docs(lines, macro.get("docs", ""), ITEM_LEVEL)
    return lines


def _value_entry(item: dict, type_key: str, paths: dict, index: dict) -> list:
    """Shared renderer for constants and statics: name, type, docs."""
    info = item.get("inner", {}).get(type_key, {})
    ty = format_type(info.get("type"), index)
    name = item.get("name", "?")
    if type_key == "constant":
        declaration_text = f"pub const {name}: {ty}"
        value = info.get("const", {}).get("expr")
        if value:
            declaration_text += f" = {value}"
    else:
        qualifiers = ["pub"]
        if info.get("is_unsafe"):
            qualifiers.append("unsafe")
        qualifiers.append("static")
        if info.get("is_mutable"):
            qualifiers.append("mut")
        declaration_text = f"{' '.join(qualifiers)} {name}: {ty}"
        if info.get("expr"):
            declaration_text += f" = {info['expr']}"
    lines = item_heading(item, paths)
    lines.extend(["```rust", f"{declaration_text};", "```", ""])
    append_docs(lines, item.get("docs", ""), ITEM_LEVEL)
    return lines


def document_constant(item: dict, index: dict, paths: dict) -> list:
    return _value_entry(item, "constant", paths, index)


def document_static(item: dict, index: dict, paths: dict) -> list:
    return _value_entry(item, "static", paths, index)


def document_type_alias(item: dict, index: dict, paths: dict) -> list:
    info = item.get("inner", {}).get("type_alias", {})
    ty = format_type(info.get("type"), index)
    params, where_clause = format_generics(info.get("generics", {}), index)
    lines = [
        f"{'#' * ITEM_LEVEL} `{qualified_name(item, paths)}`",
        "",
        "```rust",
        f"pub type {item.get('name', '?')}{params}{where_clause} = {ty};",
        "```",
        "",
    ]
    append_docs(lines, item.get("docs", ""), ITEM_LEVEL)
    return lines


def document_extern_type(item: dict, index: dict, paths: dict) -> list:
    lines = item_heading(item, paths)
    lines.extend(["```rust", f"extern type {item.get('name', '?')};", "```", ""])
    append_docs(lines, item.get("docs", ""), ITEM_LEVEL)
    return lines


def document_primitive(item: dict, index: dict, paths: dict) -> list:
    lines = item_heading(item, paths)
    info = item.get("inner", {}).get("primitive", {})
    lines.extend([f"Primitive type: `{info.get('name', item.get('name', '?'))}`", ""])
    append_docs(lines, item.get("docs", ""), ITEM_LEVEL)
    append_inherent_items(lines, info, index)
    return lines


def document_method(method: dict, index: dict) -> list:
    """Generate documentation for a method."""
    lines = []
    name = method.get("name", "Unknown")
    docs = method.get("docs", "")
    func = method.get("inner", {}).get("function", {})

    sig = format_function_signature(func, index, name)
    lines.append(f"{'#' * MEMBER_LEVEL} `{name}`")
    lines.append("")
    lines.append("```rust")
    lines.append(sig)
    lines.append("```")
    lines.append("")
    append_docs(lines, docs, MEMBER_LEVEL)

    return lines


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2]) if len(sys.argv) > 2 else input_path.with_suffix(".md")

    print(f"Loading {input_path}...")
    with open(input_path) as f:
        data = json.load(f)

    print("Generating documentation...")
    markdown = generate_megadoc(data)

    print(f"Writing {output_path}...")
    with open(output_path, "w") as f:
        f.write(markdown)

    print("Done!")


if __name__ == "__main__":
    main()
