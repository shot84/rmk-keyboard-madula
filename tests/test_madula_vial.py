import json
import pathlib
import re
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]

class MadulaVial(unittest.TestCase):
    def test_all_physical_and_virtual_positions_match_firmware(self):
        cfg = tomllib.loads((ROOT / "keyboard.toml").read_text())
        vial = json.loads((ROOT / "madula-vial.json").read_text())
        keys, encoders = [], []
        for row in vial["layouts"]["keymap"]:
            for item in row:
                if isinstance(item, str):
                    labels = item.split("\n")
                    position = tuple(map(int, labels[0].split(",")))
                    (encoders if labels[-1] == "e" else keys).append(position)
        expected = {tuple(map(int, pair)) for pair in
                    re.findall(r"\((\d+),(\d+)\)", cfg["layout"]["map"])}
        self.assertEqual(len(keys), len(set(keys)))
        self.assertEqual(set(keys), expected)
        self.assertEqual(len(keys), 60)  # 50 Cornix + 3 Madula + 7 gesture positions
        self.assertEqual(set(encoders), {(0,0),(0,1),(1,0),(1,1)})
        self.assertEqual(vial["matrix"], {"rows": cfg["layout"]["rows"], "cols": cfg["layout"]["cols"]})
        self.assertEqual(int(vial["vendorId"],16), cfg["keyboard"]["vendor_id"])
        self.assertEqual(int(vial["productId"],16), cfg["keyboard"]["product_id"])
