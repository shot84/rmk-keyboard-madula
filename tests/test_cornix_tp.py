import pathlib
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]

class CornixTP(unittest.TestCase):
    def test_sensor_pins_are_not_scanned_as_keys(self):
        cfg = tomllib.loads((ROOT / "cornix-tp.toml").read_text())
        central = cfg["split"]["central"]
        self.assertEqual(central["matrix"]["direct_pins"], [["_", "_", "_"]])
        self.assertEqual((central["rows"], central["cols"]), (1, 3))

    def test_peripheral_protocol_and_virtual_bindings_match(self):
        tp = tomllib.loads((ROOT / "cornix-tp.toml").read_text())
        base = tomllib.loads((ROOT / "keyboard.toml").read_text())
        self.assertEqual(tp["split"]["peripheral"], base["split"]["peripheral"])
        import re
        self.assertEqual(tp["layout"]["rows"], base["layout"]["rows"])
        self.assertEqual(tp["layout"]["cols"], base["layout"]["cols"])
        tp_keys = re.findall(r"\(\d+,\d+\)", tp["layout"]["map"])
        base_keys = re.findall(r"\(\d+,\d+\)", base["layout"]["map"])
        self.assertEqual(base_keys, tp_keys + [f"(4,{i})" for i in range(10,14)])
        for layer in tp["keymap"]["layer"]:
            self.assertEqual(layer["keys"].split()[-7:], [
                "MouseBtn1", "MouseBtn2", "MouseBtn3", "MouseWheelUp",
                "MouseWheelDown", "MouseWheelLeft", "MouseWheelRight"])


class VialDefinition(unittest.TestCase):
    def test_layout_matches_active_inputs_and_encoders(self):
        import json
        import re
        cfg = tomllib.loads((ROOT / "cornix-tp.toml").read_text())
        vial = json.loads((ROOT / "vial.json").read_text())
        self.assertEqual(vial["matrix"], {"rows": 5, "cols": 14})
        keys, encoders = [], []
        for row in vial["layouts"]["keymap"]:
            for item in row:
                if isinstance(item, str):
                    parts = item.split("\n")
                    pair = tuple(map(int, parts[0].split(",")))
                    (encoders if parts[-1] == "e" else keys).append(pair)
        physical = [tuple(map(int, pair)) for pair in
                    re.findall(r"\((\d+),(\d+)\)", cfg["layout"]["map"])[:50]]
        self.assertEqual(len(keys), len(set(keys)))
        self.assertEqual(set(keys), set(physical) | {(4, c) for c in [3,4,6,7,8,9]})
        self.assertEqual(set(encoders), {(0,0),(0,1),(1,0),(1,1)})
        self.assertTrue(set(map(tuple, cfg["host"]["unlock_keys"])) <= set(physical))
        self.assertEqual(int(vial["vendorId"], 16), cfg["keyboard"]["vendor_id"])
        self.assertEqual(int(vial["productId"], 16), cfg["keyboard"]["product_id"])
