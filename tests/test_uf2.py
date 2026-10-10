import importlib.util
import pathlib
import struct
import unittest

path = pathlib.Path(__file__).resolve().parents[1] / "scripts/make-uf2.py"
spec = importlib.util.spec_from_file_location("make_uf2", path)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class Uf2Conversion(unittest.TestCase):
    def test_valid_vectors_and_padding(self):
        data = struct.pack("<II", 0x20010000, 0x1009) + bytes(249)
        result = module.convert(data)
        self.assertEqual(len(result), 1024)
        self.assertEqual(struct.unpack_from("<I", result, 12)[0], 0x1000)
        self.assertEqual(struct.unpack_from("<I", result, 524)[0], 0x1100)
        self.assertEqual(result[545:800], b"\xff" * 255)

    def test_invalid_images_rejected(self):
        for data in (b"", bytes(8), struct.pack("<II", 0x20010000, 0x1008),
                     struct.pack("<II", 0x20010000, 0x27001) + bytes(256),
                     struct.pack("<II", 0x20010000, 0x1009) + bytes(0xD4000)):
            with self.assertRaises(ValueError):
                module.convert(data)
