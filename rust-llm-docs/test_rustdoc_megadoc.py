import unittest

from rustdoc_common import format_function_signature, format_type
from rustdoc_megadoc import SUPPORTED_ITEM_KINDS, generate_megadoc


def item(item_id, name, inner, *, visibility="public"):
    return {
        "id": item_id,
        "crate_id": 0,
        "name": name,
        "docs": f"Docs for {name}." if name else "",
        "visibility": visibility,
        "inner": inner,
    }


EMPTY_GENERICS = {"params": [], "where_predicates": []}
EMPTY_HEADER = {
    "is_const": False,
    "is_async": False,
    "is_unsafe": False,
    "abi": "Rust",
}


class RustdocFormattingTests(unittest.TestCase):
    def test_full_function_signature(self):
        function = {
            "header": {**EMPTY_HEADER, "is_unsafe": True, "abi": {"C": {"unwind": True}}},
            "generics": {
                "params": [
                    {"name": "'a", "kind": {"lifetime": {"outlives": []}}},
                    {
                        "name": "T",
                        "kind": {
                            "type": {
                                "bounds": [
                                    {
                                        "trait_bound": {
                                            "trait": {"path": "Clone", "args": None},
                                            "generic_params": [],
                                            "modifier": "none",
                                        }
                                    }
                                ],
                                "default": None,
                                "is_synthetic": False,
                            }
                        },
                    },
                    {
                        "name": "N",
                        "kind": {
                            "const": {"type": {"primitive": "usize"}, "default": "4"}
                        },
                    },
                ],
                "where_predicates": [
                    {
                        "lifetime_predicate": {
                            "lifetime": "'a",
                            "outlives": ["'static"],
                        }
                    }
                ],
            },
            "sig": {
                "inputs": [("value", {"generic": "T"})],
                "output": {"primitive": "bool"},
                "is_c_variadic": True,
            },
        }

        self.assertEqual(
            format_function_signature(function, {}, "check"),
            "unsafe extern \"C-unwind\" fn check<'a, T: Clone, const N: usize = 4>"
            "(value: T, ...) -> bool where 'a: 'static",
        )

    def test_all_type_shapes_added_in_format_57(self):
        constrained = {
            "resolved_path": {
                "path": "Iterator",
                "args": {
                    "angle_bracketed": {
                        "args": [{"infer": None}],
                        "constraints": [
                            {
                                "name": "Item",
                                "args": None,
                                "binding": {
                                    "equality": {"type": {"primitive": "u8"}}
                                },
                            }
                        ],
                    }
                },
            }
        }
        self.assertEqual(format_type(constrained, {}), "Iterator<_, Item = u8>")
        self.assertEqual(format_type({"tuple": [{"primitive": "u8"}]}, {}), "(u8,)")
        self.assertEqual(format_type({"infer": None}, {}), "_")
        self.assertEqual(
            format_type(
                {
                    "pat": {
                        "type": {"primitive": "u32"},
                        "__pat_unstable_do_not_use": "1..",
                    }
                },
                {},
            ),
            "u32 is 1..",
        )
        self.assertEqual(
            format_type(
                {
                    "function_pointer": {
                        "sig": {
                            "inputs": [("value", {"primitive": "u8"})],
                            "output": {"primitive": "bool"},
                            "is_c_variadic": False,
                        },
                        "generic_params": [],
                        "header": {**EMPTY_HEADER, "abi": {"C": {"unwind": False}}},
                    }
                },
                {},
            ),
            'extern "C" fn(u8) -> bool',
        )

    def test_unknown_type_shape_fails_instead_of_leaking_python_repr(self):
        with self.assertRaisesRegex(ValueError, "future_type"):
            format_type({"future_type": {}}, {})


class MegadocCoverageTests(unittest.TestCase):
    def fixture(self):
        index = {
            "0": item(0, "fixture", {"module": {"is_crate": True, "items": []}}),
            "1": item(1, "submodule", {"module": {"is_crate": False, "items": []}}),
            "2": item(
                2,
                None,
                {
                    "use": {
                        "source": "other::Thing",
                        "name": "Renamed",
                        "id": None,
                        "is_glob": False,
                    }
                },
            ),
            "3": item(
                3,
                "dependency",
                {"extern_crate": {"name": "dependency", "rename": "dep"}},
            ),
            "4": item(
                4,
                "Bits",
                {
                    "union": {
                        "generics": EMPTY_GENERICS,
                        "fields": [5],
                        "impls": [],
                    }
                },
            ),
            "5": item(5, "byte", {"struct_field": {"primitive": "u8"}}),
            "6": item(
                6,
                "Holder",
                {
                    "struct": {
                        "kind": {"tuple": [7]},
                        "generics": EMPTY_GENERICS,
                        "impls": [8],
                    }
                },
            ),
            "7": item(7, None, {"struct_field": {"primitive": "u16"}}),
            "8": item(
                8,
                None,
                {
                    "impl": {
                        "trait": None,
                        "items": [9],
                        "is_synthetic": False,
                        "blanket_impl": None,
                    }
                },
                visibility="default",
            ),
            "9": item(
                9,
                "MAGIC",
                {"assoc_const": {"type": {"primitive": "u16"}, "value": "7"}},
            ),
            "10": item(
                10,
                "Choice",
                {
                    "enum": {
                        "generics": EMPTY_GENERICS,
                        "variants": [11],
                        "impls": [],
                    }
                },
            ),
            "11": item(
                11,
                "One",
                {
                    "variant": {
                        "kind": "plain",
                        "discriminant": {"expr": "1", "value": "1"},
                    }
                },
            ),
            "12": item(
                12,
                "Example",
                {
                    "trait": {
                        "is_unsafe": False,
                        "is_auto": False,
                        "generics": EMPTY_GENERICS,
                        "bounds": [],
                        "items": [13],
                    }
                },
            ),
            "13": item(
                13,
                "Output",
                {
                    "assoc_type": {
                        "generics": EMPTY_GENERICS,
                        "bounds": [],
                        "type": {"primitive": "u8"},
                    }
                },
            ),
            "14": item(
                14,
                "AliasTrait",
                {"trait_alias": {"generics": EMPTY_GENERICS, "params": []}},
            ),
            "15": item(
                15,
                "work",
                {
                    "function": {
                        "sig": {"inputs": [], "output": None, "is_c_variadic": False},
                        "generics": EMPTY_GENERICS,
                        "header": EMPTY_HEADER,
                        "has_body": True,
                    }
                },
            ),
            "16": item(16, "rules", {"macro": "macro_rules! rules"}),
            "17": item(
                17,
                "derive_it",
                {"proc_macro": {"kind": "derive", "helpers": ["helper"]}},
            ),
            "18": item(
                18,
                "ANSWER",
                {
                    "constant": {
                        "type": {"primitive": "u8"},
                        "const": {"expr": "42", "value": "42", "is_literal": True},
                    }
                },
            ),
            "19": item(
                19,
                "COUNTER",
                {
                    "static": {
                        "type": {"primitive": "u8"},
                        "is_mutable": True,
                        "is_unsafe": False,
                        "expr": "0",
                    }
                },
            ),
            "20": item(
                20,
                "Byte",
                {
                    "type_alias": {
                        "type": {"primitive": "u8"},
                        "generics": EMPTY_GENERICS,
                    }
                },
            ),
            "21": item(21, "Opaque", {"extern_type": None}),
            "22": item(22, "u8", {"primitive": {"name": "u8", "impls": []}}),
        }
        paths = {
            str(item_id): {"path": ["fixture", value.get("name") or "reexport"]}
            for item_id, value in ((int(key), value) for key, value in index.items())
        }
        return {
            "root": 0,
            "crate_version": "1.0.0",
            "index": index,
            "paths": paths,
        }

    def test_every_item_enum_variant_is_accounted_for(self):
        self.assertEqual(
            SUPPORTED_ITEM_KINDS,
            {
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
            },
        )

    def test_megadoc_renders_top_level_and_nested_item_kinds(self):
        rendered = generate_megadoc(self.fixture())
        for heading in (
            "## Modules",
            "## Re-exports",
            "## Extern crates",
            "## Unions",
            "## Structs",
            "## Enums",
            "## Traits",
            "## Trait aliases",
            "## Functions",
            "## Macros",
            "## Constants",
            "## Statics",
            "## Type aliases",
            "## Extern types",
            "## Primitive types",
        ):
            self.assertIn(heading, rendered)
        self.assertIn("pub use other::Thing as Renamed;", rendered)
        self.assertIn("fn work()", rendered)
        self.assertIn("const MAGIC: u16 = 7", rendered)
        self.assertIn("discriminant: `1`", rendered)
        self.assertIn("Helper attributes: `helper`", rendered)

    def test_unknown_schema_item_fails_instead_of_disappearing(self):
        data = self.fixture()
        data["index"]["99"] = item(99, "Future", {"future_item": {}})
        with self.assertRaisesRegex(ValueError, "future_item"):
            generate_megadoc(data)

    def test_reexport_order_does_not_depend_on_json_object_order(self):
        root = item(0, "fixture", {"module": {"is_crate": True, "items": []}})
        alpha = item(
            1,
            None,
            {
                "use": {
                    "source": "alpha::Thing",
                    "name": "Thing",
                    "id": None,
                    "is_glob": False,
                }
            },
        )
        zeta = item(
            2,
            None,
            {
                "use": {
                    "source": "zeta::Thing",
                    "name": "Thing",
                    "id": None,
                    "is_glob": False,
                }
            },
        )

        def render(ordered_items):
            return generate_megadoc(
                {
                    "root": 0,
                    "crate_version": "1.0.0",
                    "index": {str(value["id"]): value for value in ordered_items},
                    "paths": {"0": {"path": ["fixture"]}},
                }
            )

        forward = render([root, alpha, zeta])
        reverse = render([zeta, alpha, root])
        self.assertEqual(forward, reverse)
        self.assertLess(forward.index("alpha::Thing"), forward.index("zeta::Thing"))


if __name__ == "__main__":
    unittest.main()
