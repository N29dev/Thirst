"""Check the PNG and ICO icons before building Tauri.

Python standard library only. In particular, verify the number of decoded RGBA
scanline bytes so a malformed image cannot pass CI and crash the app at startup.
"""
from pathlib import Path
from struct import unpack
import zlib

ROOT = Path(__file__).resolve().parent.parent / "src-tauri" / "icons"


def check_png(data: bytes) -> tuple[int, int]:
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "Invalid PNG signature"
    offset = 8
    width = height = 0
    payload = bytearray()
    saw_end = False
    while offset + 12 <= len(data):
        length = unpack(">I", data[offset:offset + 4])[0]
        kind = data[offset + 4:offset + 8]
        end = offset + 12 + length
        assert end <= len(data), "Truncated PNG chunk"
        value = data[offset + 8:offset + 8 + length]
        actual_crc = unpack(">I", data[offset + 8 + length:end])[0]
        expected_crc = zlib.crc32(kind + value) & 0xFFFFFFFF
        assert actual_crc == expected_crc, "PNG chunk checksum mismatch"
        if kind == b"IHDR":
            width, height, depth, color, compression, filter_method, interlace = unpack(">IIBBBBB", value)
            assert (depth, color, compression, filter_method, interlace) == (8, 6, 0, 0, 0), "Icon must be noninterlaced 8-bit RGBA"
        if kind == b"IDAT":
            payload.extend(value)
        if kind == b"IEND":
            saw_end = True
            break
        offset = end
    assert width > 0 and height > 0 and saw_end, "Incomplete PNG"
    decoded = zlib.decompress(payload)
    stride = 1 + width * 4
    assert len(decoded) == height * stride, (
        f"Decoded PNG RGBA length mismatch: got {len(decoded)} bytes; "
        f"expected {height * stride} for {width}x{height}"
    )
    assert all(decoded[y * stride] in range(5) for y in range(height)), "Invalid PNG filter byte"
    return width, height


def main() -> None:
    png = (ROOT / "icon.png").read_bytes()
    width, height = check_png(png)
    assert width == height and width >= 128, "App icon PNG must be square and at least 128px"
    ico = (ROOT / "icon.ico").read_bytes()
    reserved, icon_type, count = unpack("<HHH", ico[:6])
    assert (reserved, icon_type) == (0, 1) and count > 0, "Invalid ICO directory"
    found_rgba_png = False
    for index in range(count):
        entry = ico[6 + index * 16:6 + (index + 1) * 16]
        assert len(entry) == 16, "Truncated ICO entry"
        _, _, _, _, _, _, length, start = unpack("<BBBBHHII", entry)
        assert length > 0 and start + length <= len(ico), "Invalid ICO payload"
        if ico[start:start + 8] == b"\x89PNG\r\n\x1a\n":
            iw, ih = check_png(ico[start:start + length])
            found_rgba_png = found_rgba_png or (iw == ih == 256)
    assert found_rgba_png, "ICO must contain a full 256x256 RGBA PNG"
    print(f"Thirst icon assets OK: {width}x{height} PNG, valid ICO")


if __name__ == "__main__":
    main()
