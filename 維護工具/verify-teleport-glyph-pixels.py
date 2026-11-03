"""Compare owner-drawn GUI glyphs to independently decoded original IMG pixels."""
from pathlib import Path
import hashlib
import json
import sys
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
REFERENCE_RGBA = {
    7311: 'CDFFE5F32A3AAC9A70AC0BCC212092D99175FCBAD8EDBCFB870D19C36132829C',
    7312: '6ECEF518A516984FB65915492F139DA8DEC4B7C4BF130029A8F7548E12A7EB00',
}


def original_image(path, id):
    image = Image.open(path).convert('RGBA')
    assert image.size == (13, 14)
    assert hashlib.sha256(image.tobytes()).hexdigest().upper() == REFERENCE_RGBA[id]
    return image


def raster(reference, dpi, mode):
    width, height = [n * dpi // 96 for n in reference.size]
    if mode == 'pixel-centres':
        return reference.resize((width, height), Image.Resampling.NEAREST)
    image = Image.new('RGBA', (width, height))
    for y in range(height):
        for x in range(width):
            sx, sy = x * reference.width // width, y * reference.height // height
            image.putpixel((x, y), reference.getpixel((sx, sy)))
    return image


def find(frame, reference, bounds, dpi):
    left, top, right, bottom = bounds
    left, top = max(left, 0), max(top, 0)
    right, bottom = min(right, frame.width), min(bottom, frame.height)
    # The native renderer puts a 13px source glyph at the left of this field.
    right = min(right, left + 48 * dpi // 96)
    pixels = frame.load()
    for mode in ['pixel-centres', 'integer-edges']:
        image = raster(reference, dpi, mode)
        opaque = [(x, y, image.getpixel((x, y))[:3]) for y in range(image.height)
                  for x in range(image.width) if image.getpixel((x, y))[3]]
        clear = [(x, y) for y in range(image.height) for x in range(image.width)
                 if not image.getpixel((x, y))[3]]
        assert len(opaque) > 20 and clear
        for y in range(top, bottom - image.height + 1):
            for x in range(left, right - image.width + 1):
                if not all(pixels[x + dx, y + dy] == rgb for dx, dy, rgb in opaque):
                    continue
                background = pixels[x + clear[0][0], y + clear[0][1]]
                if all(pixels[x + dx, y + dy] == background for dx, dy in clear):
                    return dict(origin=[x, y], size=list(image.size), dpi=dpi,
                                sampling=mode, opaque_pixels=len(opaque), color_key_pixels=len(clear),
                                background_rgb=list(background))
    return None


def main(case):
    receipt = json.loads((case / 'receipt.json').read_text(encoding='utf-8-sig'))
    assert receipt['status'].startswith('verified_native_teleport_GUI_')
    refs = ROOT / '驗證紀錄/current/native-world-integration/teleport-page-images'
    reports = []
    for role in ['a', 'b']:
        steps = {s['step']: s for s in receipt['steps'] if s['role'] == role}
        for step, id in [('original-B-image', 7311), ('original-C-image', 7312)]:
            shot = steps[step]
            assert shot['glyph']['id'] == id and shot['controls']['npc_visible']
            png = Path(shot['png'])
            assert sha(png) == shot['png_sha256']
            reference = original_image(refs / f'{id}-color-key.png', id)
            frame = Image.open(png).convert('RGB')
            found = find(frame, reference, shot['glyph']['control_rect'], shot['glyph']['dpi'])
            assert found, f'{role} / {id}: original pixels not in source-owned combo field'
            reports.append(dict(role=role, id=id, screenshot=str(png), screenshot_sha256=sha(png),
                                reference_png_sha256=sha(refs / f'{id}-color-key.png'),
                                reference_rgba_sha256=REFERENCE_RGBA[id], **found))
        retired = steps['destination' if receipt['scenario'] == 'cross-endpoint' else 'stopped']
        assert not retired['controls']['npc_visible']
        frame = Image.open(retired['png']).convert('RGB')
        for step, id in [('original-B-image', 7311), ('original-C-image', 7312)]:
            original = steps[step]
            reference = original_image(refs / f'{id}-color-key.png', id)
            assert find(frame, reference, original['glyph']['control_rect'], original['glyph']['dpi']) is None
    result = dict(status='verified_original_7311_7312_GUI_pixels_and_retired_dialog_absence',
                  exe_sha256=receipt['exe_sha256'], jar_sha256=receipt['jar_sha256'],
                  receipt_sha256=sha(case / 'receipt.json'), verifier_sha256=sha(Path(__file__)),
                  matches=reports, matched_glyphs=4, role_retirements_verified=2,
                  changes_server_availability=False, original_client_paint_parity_verified=False,
                  physical_input_verified=False, full_game_complete=False)
    (case / 'teleport-glyph-comparison.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': result['status'], 'matched_glyphs': len(reports),
                      'observed_DPI': sorted({x['dpi'] for x in reports})}, ensure_ascii=False))


if __name__ == '__main__':
    main(Path(sys.argv[1]).resolve())
