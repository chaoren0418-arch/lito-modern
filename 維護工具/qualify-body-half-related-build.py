"""Retain one terminal related-test run and its original pixel evidence."""
from pathlib import Path
import argparse
import hashlib
import json
import re
import shutil
import zipfile
from PIL import Image

root = Path(__file__).resolve().parents[1]
stage = root / '驗證紀錄/current/native-draw-flags/body-half'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
parser = argparse.ArgumentParser()
parser.add_argument('--profile', choices=['Debug', 'Release'], required=True)
args = parser.parse_args()
profile = args.profile
pins_path = stage / 'strengthened-build-inputs-before.json'
pins = read(pins_path)
assert len(pins) == 962
for name, expected in pins.items():
    source = (root / name).resolve()
    assert source.is_relative_to(root)
    assert sha(source) == expected, name
log_path = stage / f'{profile}-strengthened-tests.jsonl'
log = log_path.read_text(encoding='utf-8-sig')
counts = [list(map(int, m)) for m in re.findall(
    r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', log)]
assert counts == [[5, 0, 0], [7, 0, 0], [2, 0, 0], [2, 0, 2]], counts
messages = []
for line in log.splitlines():
    if line.startswith('{'):
        messages.append(json.loads(line))
assert any(d.get('reason') == 'build-finished' and d['success'] for d in messages)
assert not any(d.get('reason') == 'compiler-message' and d['message']['level'] == 'error' for d in messages)
products = []
archive = stage / f'{profile}-strengthened-products.zip'
with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as z:
    for d in messages:
        if d.get('reason') != 'compiler-artifact' or not d.get('executable') or not d['profile'].get('test'):
            continue
        p = Path(d['executable']).resolve()
        assert p.is_relative_to(root / 'LiTo381-Modern/target/current')
        products.append({'suite': d['target']['name'], 'path': str(p), 'sha256': sha(p)})
        z.write(p, p.name)
assert {p['suite'] for p in products} == {'native_actor_states', 'native_actor_layers', 'native_type12_prop', 'native_world_entry'}
assert len(products) == 4
with zipfile.ZipFile(archive) as z:
    for p in products:
        assert hashlib.sha256(z.read(Path(p['path']).name)).hexdigest().upper() == p['sha256']
verification = root / 'LiTo381-Modern/target/current/verification'
evidence = stage / f'{profile}-strengthened-evidence'
evidence.mkdir(exist_ok=True)
report = read(verification / 'native-body-half-reference.json')
assert report['pixel_comparisons'] == report['owner_surface_comparisons'] == len(report['cases']) == 30
controls = report['visible_attachment_controls']
assert len(controls) == 2 and all(c['differing_pixels_from_wrong_whole_actor_half'] > 0 for c in controls)
assert report['body_only_cases_included'] and report['synthetic_attachment_pairing']
images = []
for row in report['cases']:
    p = Path(row['png']).resolve()
    assert p.parent == verification.resolve()
    with Image.open(p) as im:
        assert im.size == (320, 240)
        assert hashlib.sha256(im.convert('RGBA').tobytes()).hexdigest().upper() == row['rgba_sha256']
    shutil.copy2(p, evidence / p.name)
    images.append({'file': p.name, 'PNG_sha256': sha(p), 'RGBA_sha256': row['rgba_sha256']})
for name in ['native-body-half-reference.json', 'type12-869-body.json', 'type12-869-model-half.json',
             'native-actor-states-reference.json', 'native-actor-layer-reference.json']:
    shutil.copy2(verification / name, evidence / name)
validator_copy = evidence / 'qualify-body-half-related-build.py'
shutil.copy2(Path(__file__), validator_copy)
result = {'status': 'qualified_current_related16_and30_body_half_pixel_owner_cases',
          'profile': profile, 'source_files': len(pins), 'source_manifest_sha256': sha(pins_path),
          'test_log_sha256': sha(log_path), 'sources_unchanged': True, 'passed': 16, 'default_live_ignored': 2,
          'products': products, 'product_zip': str(archive), 'product_zip_sha256': sha(archive),
          'zip_readback_verified': True, 'images': images, 'visible_attachment_controls': controls,
          'verifier_sha256': sha(validator_copy), 'GUI_verified': False,
          'native_runtime_framebuffer_parity': False, 'complete_native_flags': False, 'whole_goal_complete': False}
(stage / f'{profile}-strengthened-qualified.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'status': result['status'], 'profile': profile, 'passed': 16, 'pixel_cases': 30,
                  'attachment_controls': controls}, ensure_ascii=False))
