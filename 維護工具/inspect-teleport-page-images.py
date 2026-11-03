"""Read the two original TeleportMe IMG members without altering game resources."""
from pathlib import Path
import hashlib
import json
import struct
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
SOURCE = Path(r'D:\lineage開發\0-客戶端\LiTo')
OUT = ROOT / '驗證紀錄/current/native-world-integration/teleport-page-images'


def sha(data):
    return hashlib.sha256(data).hexdigest().upper()


def main():
    OUT.mkdir(exist_ok=True)
    idx = (SOURCE / 'LTsurf.idx').read_bytes()
    count, = struct.unpack_from('<I', idx)
    assert len(idx) == 4 + count * 28
    rows = {}
    for at in range(4, len(idx), 28):
        name = idx[at + 4:at + 24].split(b'\0', 1)[0].lower()
        assert name not in rows
        rows[name] = (struct.unpack_from('<I', idx, at)[0], struct.unpack_from('<I', idx, at + 24)[0])
    reports = []
    sheet = Image.new('RGB', (420, 190), (48, 48, 48))
    draw = ImageDraw.Draw(sheet)
    with (SOURCE / 'LTsurf.pak').open('rb') as pak:
        length = pak.seek(0, 2)
        for column, id in enumerate([7311, 7312]):
            name = f'{id}.img'
            offset, size = rows[name.encode()]
            assert 8 <= size <= 4096 and offset + size <= length
            pak.seek(offset)
            raw = pak.read(size)
            width, height, flag, key = struct.unpack_from('<HHHH', raw)
            assert 0 < width <= 64 and 0 < height <= 64 and flag <= 1
            assert len(raw) == 8 + width * height * 2
            words = struct.unpack('<' + 'H' * (width * height), raw[8:])
            assert all(v < 32768 for v in words)
            previews = {}
            for row, mode in enumerate(['opaque', 'color-key']):
                rgba = bytes(channel for v in words for channel in
                             ((0, 0, 0, 0) if mode == 'color-key' and v == key else
                              tuple(((v >> shift & 31) << 3) | ((v >> shift & 31) >> 2)
                                    for shift in [10, 5, 0]) + (255,)))
                image = Image.frombytes('RGBA', (width, height), rgba)
                path = OUT / f'{id}-{mode}.png'
                image.save(path)
                previews[mode] = dict(path=str(path), png_sha256=sha(path.read_bytes()), rgba_sha256=sha(rgba))
                draw.text((column * 210 + 8, row * 92 + 4), f'{id}.img / {mode}', fill='white')
                scaled = image.resize((width * 4, height * 4), Image.Resampling.NEAREST)
                sheet.paste(scaled, (column * 210 + 8, row * 92 + 25), scaled)
            (OUT / name).write_bytes(raw)
            reports.append(dict(id=id, member=name, offset=offset, bytes=size,
                                sha256=sha(raw), dimensions=[width, height], flag=flag,
                                color_key_rgb555=key, key_pixels=words.count(key), previews=previews))
    sheet.save(OUT / 'preview.png')
    doc = ROOT / 'LiTo381-Modern/tests/fixtures/343-teleportme-c.html'
    assert all(f'<img src=#{id}>'.encode() in doc.read_bytes() for id in [7311, 7312])
    report = dict(status='original_teleport_page_IMG_members_located_and_decoded_preview_only',
                  HTML_fixture_sha256=sha(doc.read_bytes()), index=str(SOURCE / 'LTsurf.idx'),
                  index_sha256=sha(idx), images=reports,
                  decoder_reference='LiTo381-Modern/src/modern_lito/native_offline_menu_skin.rs:12',
                  numeric_TBT_identity_is_not_HTML_IMG_identity=True,
                  page_paint_mode_verified=False, availability_state_mapping_verified=False,
                  native_GUI_integrated=False, full_game_complete=False)
    (OUT / 'receipt.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': report['status'], 'images': [{k: x[k] for k in ['member', 'sha256', 'dimensions']} for x in reports], 'output': str(OUT)}, ensure_ascii=False))


if __name__ == '__main__':
    main()
