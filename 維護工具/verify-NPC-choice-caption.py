"""Check that a selected non-teleport caption remains visibly dark on its field."""
from pathlib import Path
import hashlib
import json
import sys
from PIL import Image

case = Path(sys.argv[1]).resolve()
receipt = json.loads((case / 'receipt.json').read_text(encoding='utf-8-sig'))
assert receipt['status'] == 'verified_native_rental_widget_flow'
shot = next(s for s in receipt['steps'] if s['role'] == 'a' and s['step'] == 'guide-citizen-page')
choice = shot['choice']
assert choice['visible'] and choice['index'] >= 0 and choice['label']
path = Path(shot['png'])
assert hashlib.sha256(path.read_bytes()).hexdigest().upper() == shot['png_sha256']
image = Image.open(path).convert('RGBA')
image = Image.alpha_composite(Image.new('RGBA', image.size, 'white'), image).convert('RGB')
left, top, right, bottom = choice['rect']
scale = choice['dpi'] / 96
area = image.crop((left + int(4 * scale), top + int(4 * scale),
                   right - int(24 * scale), bottom - int(4 * scale)))
dark = sum(max(rgb) < 150 for rgb in area.getdata())
assert dark >= 50, f'selected original caption is too faint: {dark} dark pixels'
result = dict(status='verified_selected_NPC_caption_has_visible_dark_text',
              exe_sha256=receipt['exe_sha256'], jar_sha256=receipt['jar_sha256'],
              label=choice['label'], dark_pixels=dark, observed_DPI=choice['dpi'],
              screenshot_sha256=shot['png_sha256'], verifier_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest().upper(),
              physical_input_verified=False, full_game_complete=False)
(case / 'NPC-caption-comparison.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps(result, ensure_ascii=False))
