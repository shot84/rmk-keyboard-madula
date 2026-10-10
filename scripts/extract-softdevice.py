"""Extract legacy XIAO SoftDevice from its CURRENT.UF2 backup, excluding MBR.

Creates a recovery artifact only; never flashes it. Source must cover every
256-byte block in [0x1000, 0x27000). Use only with the backed-up XIAO.
"""
import argparse
import pathlib
import struct

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("backup", type=pathlib.Path)
parser.add_argument("output", type=pathlib.Path)
args = parser.parse_args()
data = args.backup.read_bytes()
if len(data) % 512:
    raise SystemExit("Invalid UF2 length")
payloads = {}
for offset in range(0, len(data), 512):
    block = data[offset:offset + 512]
    magic0, magic1, flags, address, size, _, _, _ = struct.unpack_from("<8I", block)
    if (magic0, magic1, struct.unpack_from("<I", block, 508)[0]) != (0x0A324655, 0x9E5D5157, 0x0AB16F30):
        raise SystemExit("Invalid UF2 magic")
    if 0x1000 <= address < 0x27000:
        if size != 256 or address % 256 or address in payloads or flags & 1:
            raise SystemExit("Invalid or duplicate SoftDevice block")
        payloads[address] = block[32:288]
addresses = list(range(0x1000, 0x27000, 256))
if set(payloads) != set(addresses):
    raise SystemExit("Incomplete SoftDevice backup")
with args.output.open("xb") as output:
    for index, address in enumerate(addresses):
        header = struct.pack("<8I", 0x0A324655, 0x9E5D5157, 0x2000,
                             address, 256, index, len(addresses), 0xADA52840)
        output.write(header + payloads[address] + bytes(220) + struct.pack("<I", 0x0AB16F30))
print(f"Extracted {len(addresses)} blocks; MBR and bootloader excluded: {args.output}")
