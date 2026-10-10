"""Convert an objcopy-produced RMK SDC binary to UF2. Does not flash devices."""
import argparse
import pathlib
import struct


def convert(data):
    base, limit = 0x1000, 0xD4000
    if len(data) < 8 or base + len(data) > limit:
        raise ValueError("Binary outside application bounds")
    stack, entry = struct.unpack_from("<II", data)
    if not (0x20000000 <= stack <= 0x20040000 and stack % 8 == 0
            and base <= (entry & ~1) < base + len(data) and entry & 1):
        raise ValueError("Invalid Cortex-M application vectors")
    count = (len(data) + 255) // 256
    blocks = []
    for index in range(count):
        payload = data[index * 256:(index + 1) * 256].ljust(256, b"\xff")
        header = struct.pack("<8I", 0x0A324655, 0x9E5D5157, 0x2000,
                             base + index * 256, 256, index, count, 0xADA52840)
        blocks.append(header + payload + bytes(220) + struct.pack("<I", 0x0AB16F30))
    return b"".join(blocks)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=pathlib.Path)
    parser.add_argument("output", type=pathlib.Path)
    args = parser.parse_args()
    result = convert(args.binary.read_bytes())
    args.output.write_bytes(result)
    print(f"Created {len(result)} UF2 bytes; base=0x1000, MBR/bootloader excluded")
