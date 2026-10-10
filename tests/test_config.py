import pathlib
import re
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class BringupConfig(unittest.TestCase):
    def setUp(self):
        self.cfg = tomllib.loads((ROOT / "keyboard.toml").read_text())

    def test_three_boards_have_disjoint_regions(self):
        boards = [self.cfg["split"]["central"], *self.cfg["split"]["peripheral"]]
        self.assertEqual(len(boards), 3)
        used = set()
        for board in boards:
            region = {(r + board["row_offset"], c + board["col_offset"])
                      for r in range(board["rows"]) for c in range(board["cols"])}
            self.assertFalse(used & region)
            used |= region
        mapped = re.findall(r"\((\d+),(\d+)\)", self.cfg["layout"]["map"])
        self.assertEqual(len(mapped), 53)
        self.assertEqual(len(set(mapped)), 53)
        self.assertTrue({tuple(map(int, p)) for p in mapped} <= used)

    def test_source_pin_mapping(self):
        left, right = self.cfg["split"]["peripheral"]
        self.assertEqual(left["matrix"]["row_pins"], ["P0_30", "P0_31", "P0_29", "P0_02"])
        self.assertEqual(right["matrix"]["row_pins"], ["P1_09", "P0_28", "P0_03", "P1_10"])
        for board in (left, right):
            matrix = board["matrix"]
            self.assertFalse(matrix["row2col"])
            pins = matrix["row_pins"] + matrix["col_pins"]
            self.assertEqual(len(pins), len(set(pins)))
        direct = self.cfg["split"]["central"]["matrix"]
        self.assertEqual(direct["direct_pins"], [["P0_04", "P0_05", "P1_11"]])
        self.assertTrue(direct["direct_pin_low_active"])

    def test_keymap_count(self):
        for layer in self.cfg["keymap"]["layer"]:
            self.assertEqual(len(layer["keys"].split()), 53)

    def test_flash_boundaries(self):
        # These conservative boundaries do NOT certify the installed bootloader.
        self.assertEqual(0x27000 + 692 * 1024, 0xD4000)
        storage = self.cfg["storage"]
        self.assertGreaterEqual(storage["start_addr"], 0xD4000)
        self.assertLessEqual(storage["start_addr"] + storage["num_sectors"] * 4096, 0xF4000)


if __name__ == "__main__":
    unittest.main()
