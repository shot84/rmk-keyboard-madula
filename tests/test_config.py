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
        self.assertEqual(self.cfg["keymap"]["layers"], 5)
        self.assertEqual(len(self.cfg["keymap"]["layer"]), 5)
        for layer in self.cfg["keymap"]["layer"]:
            self.assertEqual(len(layer["keys"].split()), 53)

    def test_ble_device_name_length(self):
        # RMK 0.9's BLE profile naming reserves space: max 22 UTF-8 bytes.
        self.assertLessEqual(len(self.cfg["keyboard"]["product_name"].encode("utf-8")), 22)

    def test_right_encoder_scroll(self):
        left, right = self.cfg["split"]["peripheral"]
        self.assertNotIn("input_device", left)
        encoder, = right["input_device"]["encoder"]
        self.assertEqual((encoder["pin_a"], encoder["pin_b"]), ("P1_04", "P1_06"))
        self.assertTrue(encoder["internal_pullup"])
        self.assertEqual(encoder["phase"], "resolution")
        self.assertEqual(encoder["resolution"], 4)
        matrix_pins = right["matrix"]["row_pins"] + right["matrix"]["col_pins"]
        self.assertNotIn(encoder["pin_a"], matrix_pins)
        self.assertNotIn(encoder["pin_b"], matrix_pins)
        for layer in self.cfg["keymap"]["layer"]:
            self.assertEqual(layer["encoders"], [["MouseWheelUp", "MouseWheelDown"]])

    def test_split_sleep_subscriber_capacity(self):
        # Pinned RMK 0.9 adds two split+BLE subscribers to the configured base.
        peripherals = len(self.cfg["split"]["peripheral"])
        self.assertGreaterEqual(self.cfg["event"]["sleep_state"]["subs"] + 2,
                                2 * peripherals)

    def test_madula_tap_hold_and_buttons(self):
        layers = [layer["keys"].split() for layer in self.cfg["keymap"]["layer"]]
        self.assertEqual(layers[0][12], "LT(3,Escape)")
        self.assertEqual(layers[0][38:44], [
            "MT(Escape,LCtrl)", "LGui", "LAlt", "LT(1,Space)", "LT(2,Space)", "Backspace"])
        self.assertEqual(layers[0][-3:], ["MouseBtn1", "MouseBtn3", "MouseBtn2"])
        for layer in layers[1:]:
            self.assertEqual(layer[-3:], ["_", "_", "_"])
        for layer in layers:
            for key in layer:
                match = re.match(r"(?:LT|MO)\((\d+)", key)
                if match:
                    self.assertLess(int(match[1]), len(layers))

    def test_flash_boundaries(self):
        # These conservative boundaries do NOT certify the installed bootloader.
        self.assertEqual(0x1000 + 844 * 1024, 0xD4000)
        storage = self.cfg["storage"]
        self.assertGreaterEqual(storage["start_addr"], 0xD4000)
        self.assertLessEqual(storage["start_addr"] + storage["num_sectors"] * 4096, 0xF4000)

    def test_usb_debug_features(self):
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
        rmk = manifest["dependencies"]["rmk"]
        self.assertFalse(rmk["default-features"])
        self.assertIn("usb_log", rmk["features"])
        self.assertNotIn("defmt", rmk["features"])
        self.assertIn("release_max_level_debug", manifest["dependencies"]["log"]["features"])
        self.assertEqual({binary["name"] for binary in manifest["bin"]},
                         {"madula-central", "cornix-left", "cornix-right", "xiao-ble-init"})


if __name__ == "__main__":
    unittest.main()
