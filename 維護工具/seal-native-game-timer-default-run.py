"""Retain independently qualified default test logs alongside source/images."""
import argparse
import hashlib
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[1]
STAGE = ROOT / '驗證紀錄/current/native-game-timers/20261016-004010-201637/current-regression'
ap = argparse.ArgumentParser()
ap.add_argument('--profile', choices=['Debug', 'Release'], required=True)
args = ap.parse_args()
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))

def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()

qualification = STAGE / f'{args.profile}-default93-qualified.json'
q = read(qualification)
assert q['status'] == 'qualified_current_default_Rust_results' and q['failed'] == 0
assert q['profile'] == args.profile and q['integration_targets'] == 93 and q['products_retained'] == 94
run = Path(q['run']).resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/default-native-regression')
receipt = read(run / 'receipt.json')
assert sha(run / 'receipt.json') == q['receipt_sha256']
assert receipt['status'] == 'verified_current_default93_integration_targets_and_main_units'
assert sha(run / 'test-products.zip') == q['product_archive_sha256']
assert sha(run / 'compiled-source.zip') == receipt['compiled_source_archive_sha256']
files = {}
for path in run.rglob('*'):
    if path.is_file() and path.name not in {'test-products.zip', 'compiled-source.zip'}:
        assert not path.is_symlink() and path.resolve().is_relative_to(run)
        files['raw/' + path.relative_to(run).as_posix()] = path
files['qualified.json'] = qualification
files['qualifier.py'] = ROOT / '維護工具/qualify-native-game-timer-default-regression.py'
files['sealer.py'] = Path(__file__)
file_pins = {name: sha(path) for name, path in files.items()}
archive = STAGE / f'{args.profile}-default93-raw.zip'
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name, path in files.items():
        z.write(path, name)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist()) == set(file_pins)
    for name, expected in file_pins.items():
        with z.open(name) as stream:
            assert hashlib.file_digest(stream, 'sha256').hexdigest().upper() == expected
proof = {'status': 'qualified_default93_source_products_and_raw_retained',
         'profile': args.profile, 'passed': q['passed'], 'failed': 0, 'ignored': q['ignored'],
         'qualification_sha256': sha(qualification), 'source_files': receipt['source_files'],
         'source_manifest_sha256': q['current_source_manifest_sha256'],
         'normal_mains': q['normal_mains'], 'products_retained': 94,
         'raw_archive': str(archive), 'raw_archive_sha256': sha(archive),
         'raw_file_pins': file_pins, 'raw_archive_readback_verified': True,
         'compiled_source_archive': str(run / 'compiled-source.zip'),
         'compiled_source_archive_sha256': receipt['compiled_source_archive_sha256'],
         'product_archive': str(run / 'test-products.zip'),
         'product_archive_sha256': q['product_archive_sha256'],
         'full_library_unit_tests_verified': False, 'full_current_TCP_verified': False,
         'current_TW14_GUI_verified': False, 'whole_goal_complete': False}
destination = STAGE / f'{args.profile}-default93-retention.json'
assert not destination.exists()
destination.write_text(json.dumps(proof, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps({k: proof[k] for k in ['status', 'profile', 'passed', 'failed', 'ignored', 'raw_archive_readback_verified']}))
