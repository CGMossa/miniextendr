import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


HERE = Path(__file__).parent
EMPTY_GENERICS = {"params": [], "where_predicates": []}


def impl_item(item_id, trait, for_type, filename, line):
    return {
        "id": item_id,
        "crate_id": 0,
        "name": None,
        "docs": None,
        "visibility": "public",
        "span": {"filename": filename, "begin": [line, 1], "end": [line, 2]},
        "inner": {
            "impl": {
                "is_unsafe": False,
                "generics": EMPTY_GENERICS,
                "provided_trait_methods": [],
                "trait": {"path": trait, "id": 0, "args": None},
                "for": for_type,
                "items": [],
                "is_negative": False,
                "is_synthetic": False,
                "blanket_impl": None,
            }
        },
    }


def path_type(name, element):
    return {
        "resolved_path": {
            "path": name,
            "id": 0,
            "args": {
                "angle_bracketed": {
                    "args": [{"type": element}],
                    "constraints": [],
                }
            },
        }
    }


def run_renderer(script, items):
    with tempfile.TemporaryDirectory() as temp_dir:
        json_path = Path(temp_dir) / "fixture.json"
        json_path.write_text(json.dumps({"index": items}))
        result = subprocess.run(
            [sys.executable, str(HERE / script), str(json_path)],
            check=True,
            capture_output=True,
            text=True,
        )
    return result.stdout.replace(str(json_path), "fixture.json")


class InventoryOrderingTests(unittest.TestCase):
    def test_impl_inventory_does_not_depend_on_json_object_order(self):
        records = [
            impl_item(1, "TryFromSexp", {"primitive": "u8"}, "a.rs", 10),
            impl_item(2, "TryFromSexp", {"primitive": "i8"}, "a.rs", 10),
            impl_item(3, "TryFromSexp", {"primitive": "u16"}, "b.rs", 20),
            impl_item(4, "TryFromSexp", {"primitive": "i16"}, "b.rs", 20),
            impl_item(5, "IntoR", {"primitive": "u8"}, "c.rs", 30),
            impl_item(6, "IntoR", {"primitive": "i8"}, "c.rs", 30),
            impl_item(7, "IntoR", {"primitive": "u16"}, "d.rs", 40),
            impl_item(8, "IntoR", {"primitive": "i16"}, "d.rs", 40),
        ]
        forward = {str(record["id"]): record for record in records}
        reverse = {str(record["id"]): record for record in reversed(records)}

        self.assertEqual(
            run_renderer("rustdoc_impl_inventory.py", forward),
            run_renderer("rustdoc_impl_inventory.py", reverse),
        )

    def test_manual_inventory_does_not_depend_on_json_object_order(self):
        vec_types = [
            path_type("Vec", {"primitive": primitive})
            for primitive in ("i8", "i16", "i32")
        ]
        box_types = [
            path_type("Box", {"slice": {"primitive": primitive}})
            for primitive in ("u8", "u16", "u32")
        ]
        records = [
            impl_item(index, "TryFromSexp", for_type, "manual.rs", index)
            for index, for_type in enumerate(vec_types + box_types, start=1)
        ]
        forward = {str(record["id"]): record for record in records}
        reverse = {str(record["id"]): record for record in reversed(records)}

        self.assertEqual(
            run_renderer("rustdoc_manual_vs_macro.py", forward),
            run_renderer("rustdoc_manual_vs_macro.py", reverse),
        )


if __name__ == "__main__":
    unittest.main()
